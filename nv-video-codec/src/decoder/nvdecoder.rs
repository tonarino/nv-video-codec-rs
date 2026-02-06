use super::{
    types::{ChromaFormat, Codec, CreateFlags, DeinterlaceMode, Dim, Rect, SurfaceFormat},
    FrameData,
};
use crate::{common::cuda_result::IntoCudaResult, decoder::NvDecoderBuilder};
use ffi::{
    cuMemAllocPitch_v2, cuMemAlloc_v2, cuMemFree_v2, cuMemcpy2DAsync_v2, cuStreamSynchronize,
    cudaVideoCreateFlags_enum, cuvidCreateDecoder, cuvidCtxLockCreate, cuvidCtxLockDestroy,
    cuvidDecodePicture, cuvidDecodeStatus_enum, cuvidDestroyDecoder, cuvidDestroyVideoParser,
    cuvidGetDecodeStatus, cuvidGetDecoderCaps, cuvidMapVideoFrame64, cuvidParseVideoData,
    cuvidUnmapVideoFrame64, CUdeviceptr, CUmemorytype_enum, CUstream, CUvideoctxlock,
    CUvideodecoder,
    CUvideopacketflags::{self, CUVID_PKT_ENDOFSTREAM, CUVID_PKT_TIMESTAMP},
    CUvideoparser, CUDA_MEMCPY2D, CUVIDDECODECAPS, CUVIDDECODECREATEINFO, CUVIDEOFORMAT,
    CUVIDGETDECODESTATUS, CUVIDOPERATINGPOINTINFO, CUVIDPARSERDISPINFO, CUVIDPARSERPARAMS,
    CUVIDPICPARAMS, CUVIDPROCPARAMS, CUVIDSOURCEDATAPACKET,
};
use nv_video_codec_sys as ffi;
use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};
use rustacuda::context::{Context, ContextHandle, ContextStack};
use std::{
    collections::VecDeque,
    convert::TryInto,
    os::raw::{c_int, c_ulong, c_void},
    sync::Arc,
    time::Instant,
};

use super::{DecoderPacketFlags, Frame, NvDecoderError};

pub struct NvDecoder<'a> {
    parser: CUvideoparser,
    decoder: CUvideodecoder,
    context: Context,
    use_device_frame: bool,
    codec: Codec,
    chroma_format: ChromaFormat,
    output_format: SurfaceFormat,
    video_format: CUVIDEOFORMAT,
    device_frame_pitched: bool,
    crop_rect: Rect,
    resize_dim: Dim,
    max_width: u32,
    max_height: u32,
    ctx_lock: CUvideoctxlock,
    video_info: String,
    bitdepth_minus_8: i32,
    bpp: i32,
    display_rect: Rect,
    device_frame_pitch: usize,

    decoded_frames: usize,
    decoded_frames_returned: usize,
    allocated_frames: usize,
    stream: CUstream,
    /// need mutex to cover callbacks
    frames: Arc<Mutex<VecDeque<Frame<'a>>>>,
    picture_decode_index_mapping: [usize; 32],
    decoded_pictures: usize,
    operating_point: usize,

    /// output dimensions
    width: u32,
    luma_height: u32,
    chroma_height: u32,
    num_chroma_planes: u32,

    /// height of the mapped surface
    surface_height: u64,
    surface_width: u64,
}

/// decoder refers to an NvDecoder
/// Callback function to be registered for getting a callback when decoding of sequence starts
unsafe extern "C" fn handle_video_sequence_proc(
    decoder: *mut c_void,
    video_format: *mut CUVIDEOFORMAT,
) -> c_int {
    debug_assert!(!decoder.is_null());
    (decoder as *mut NvDecoder).as_mut().unwrap().handle_video_sequence(video_format)
}

/// decoder refers to an NvDecoder
/// Callback function to be registered for getting a callback when a decoded frame is ready to be decoded
unsafe extern "C" fn handle_picture_decode_proc(
    decoder: *mut c_void,
    pic_params: *mut CUVIDPICPARAMS,
) -> c_int {
    debug_assert!(!decoder.is_null());
    (decoder as *mut NvDecoder).as_mut().unwrap().handle_picture_decode(pic_params)
}

/// decoder refers to an NvDecoder
/// Callback function to be registered for getting a callback when a decoded frame is available for display
unsafe extern "C" fn handle_picture_display_proc(
    decoder: *mut c_void,
    disp_info: *mut CUVIDPARSERDISPINFO,
) -> c_int {
    debug_assert!(!decoder.is_null());
    (decoder as *mut NvDecoder).as_mut().unwrap().handle_picture_display(disp_info)
}

/// decoder refers to an NvDecoder
/// Callback function to be registered for getting a callback to get operating point when AV1 SVC sequence header start.
unsafe extern "C" fn handle_operating_point_proc(
    decoder: *mut c_void,
    op_info: *mut CUVIDOPERATINGPOINTINFO,
) -> c_int {
    debug_assert!(!decoder.is_null());
    (decoder as *mut NvDecoder).as_mut().unwrap().handle_operating_point(op_info)
}

fn do_within_context<F, T>(context: &Context, mut func: F)
where
    F: FnMut() -> T,
    T: IntoCudaResult<()>,
{
    ContextStack::push(context).unwrap();
    func().into_cuda_result().expect("Cuda NVDEC api call failure");
    ContextStack::pop().unwrap();
}

impl<'a> NvDecoder<'a> {
    pub fn builder(context: Context, codec: Codec) -> NvDecoderBuilder {
        NvDecoderBuilder::new(context, codec)
    }

    // TODO(efyang) : switch these over to result types and just handle the results
    // also potentially have special struct for each return type for these callbacks and translate them
    /* Return value from HandleVideoSequence() are interpreted as   :
     *  0: fail, 1: succeeded, > 1: override dpb size of parser (set by CUVIDPARSERPARAMS::ulMaxNumDecodeSurfaces while creating parser)
     */
    fn handle_video_sequence(&mut self, video_format: *mut CUVIDEOFORMAT) -> i32 {
        let session_init_start = Instant::now();

        let video_format = unsafe { *video_format };

        let decode_surface = video_format.min_num_decode_surfaces;

        // TODO(efyang)
        // for our use cases, we don't need pretty much all of this stuff after we've
        // initialized properly if we're assuming the same format thereafter
        // this step takes 3ms, which is unacceptable for repeated usage
        // we temporarily skip here, this should be changed depending on how we decide
        // to build this (full featured or tailored to use case)
        if !(self.decoder.is_null()) {
            return video_format.min_num_decode_surfaces as i32;
        }

        self.video_info = format!("Video Input Information:\n{:#?}", video_format);
        let mut decode_caps = CUVIDDECODECAPS {
            eCodecType: video_format.codec,
            eChromaFormat: video_format.chroma_format,
            nBitDepthMinus8: video_format.bit_depth_luma_minus8 as u32,
            ..Default::default()
        };
        do_within_context(&self.context, || unsafe {
            cuvidGetDecoderCaps(&mut decode_caps as *mut CUVIDDECODECAPS)
        });

        if decode_caps.bIsSupported == 0 {
            // eprintln!("Codec not supported on this GPU");
            // return decode_surface as i32;
            panic!("Codec not supported on this GPU");
        }

        if video_format.coded_width > decode_caps.nMaxWidth
            || video_format.coded_height > decode_caps.nMaxHeight
        {
            panic!(
                "Resolution: {}x{}
                Max supported (wxh): {}x{}
                Resolution not supported on this GPU",
                video_format.coded_width,
                video_format.coded_height,
                decode_caps.nMaxWidth,
                decode_caps.nMaxHeight
            );
        }

        let mb_count = (video_format.coded_width >> 4) * (video_format.coded_height >> 4);
        if mb_count > decode_caps.nMaxMBCount {
            panic!(
                "MBCount: {}
                Max supported MBCount: {}
                MBCount not supported on this GPU",
                mb_count, decode_caps.nMaxMBCount,
            );
        }

        if self.width != 0 && self.luma_height != 0 && self.chroma_height != 0 {
            // cuvidCreateDecoder() has been called before, and now there's possible config change
            // L229
            // TODO(efyang) - technically not needed for our application, but should be done
            // todo!()
        }

        // eCodec has been set in the constructor (for parser). Here it's set again for potential correction
        self.codec = video_format.codec.try_into().unwrap();
        self.chroma_format = video_format.chroma_format.try_into().unwrap();
        self.bitdepth_minus_8 = video_format.bit_depth_luma_minus8 as i32;
        self.bpp = if self.bitdepth_minus_8 > 0 { 2 } else { 1 };

        // Set the output surface format same as chroma format
        if matches!(self.chroma_format, ChromaFormat::YUV420 | ChromaFormat::Monochrome) {
            self.output_format = if video_format.bit_depth_luma_minus8 != 0 {
                SurfaceFormat::P016
            } else {
                SurfaceFormat::NV12
            };
        } else if matches!(self.chroma_format, ChromaFormat::YUV444) {
            self.output_format = if video_format.bit_depth_luma_minus8 != 0 {
                SurfaceFormat::YUV444_16bit
            } else {
                SurfaceFormat::YUV444
            };
        } else if matches!(self.chroma_format, ChromaFormat::YUV422) {
            // no 4:2:2 output format supported yet so make 420 default
            self.output_format = SurfaceFormat::NV12;
        }

        // TODO(efyang) : create safe wrapper over VideoFormat
        self.video_format = video_format;

        let mut video_decode_create_info = CUVIDDECODECREATEINFO {
            CodecType: video_format.codec,
            ChromaFormat: video_format.chroma_format,
            OutputFormat: self.output_format.into(),
            bitDepthMinus8: video_format.bit_depth_luma_minus8 as u64,
            DeinterlaceMode: if video_format.progressive_sequence != 0 {
                DeinterlaceMode::Weave.into()
            } else {
                DeinterlaceMode::Adaptive.into()
            },
            ulNumOutputSurfaces: 2,
            // With PreferCUVID, JPEG is still decoded by CUDA while video is decoded by NVDEC hardware
            ulCreationFlags: cudaVideoCreateFlags_enum::from(CreateFlags::PreferCUVID).0 as u64,
            ulNumDecodeSurfaces: decode_surface as u64,
            vidLock: self.ctx_lock,
            ulWidth: video_format.coded_width as u64,
            ulHeight: video_format.coded_height as u64,
            ..Default::default()
        };

        // AV1 has max width/height of sequence in sequence header
        if matches!(video_format.codec.try_into().unwrap(), Codec::AV1)
            && video_format.seqhdr_data_length > 0
        {
            // dont overwrite if it is already set from cmdline or reconfig.txt
            // L280
            todo!()
        }

        self.max_width = std::cmp::max(self.max_width, video_format.coded_width);
        self.max_height = std::cmp::max(self.max_height, video_format.coded_height);
        video_decode_create_info.ulMaxWidth = self.max_width as u64;
        video_decode_create_info.ulMaxHeight = self.max_height as u64;

        if (self.crop_rect.right == 0 || self.crop_rect.bottom == 0)
            && (self.resize_dim.width == 0 || self.resize_dim.height == 0)
        {
            self.width = (video_format.display_area.right - video_format.display_area.left) as u32;
            self.luma_height =
                (video_format.display_area.bottom - video_format.display_area.top) as u32;
            video_decode_create_info.ulTargetWidth = video_format.coded_width as u64;
            video_decode_create_info.ulTargetHeight = video_format.coded_height as u64;
        } else {
            if self.resize_dim.width != 0 && self.resize_dim.height != 0 {
                video_decode_create_info.display_area.left = video_format.display_area.left as i16;
                video_decode_create_info.display_area.top = video_format.display_area.top as i16;
                video_decode_create_info.display_area.right =
                    video_format.display_area.right as i16;
                video_decode_create_info.display_area.bottom =
                    video_format.display_area.bottom as i16;
                self.width = self.resize_dim.width as u32;
                self.luma_height = self.resize_dim.height as u32;
            }

            // TODO(efyang) change rect and dim to be u32
            if self.crop_rect.right != 0 && self.crop_rect.bottom != 0 {
                video_decode_create_info.display_area.left = self.crop_rect.left as i16;
                video_decode_create_info.display_area.top = self.crop_rect.top as i16;
                video_decode_create_info.display_area.right = self.crop_rect.right as i16;
                video_decode_create_info.display_area.bottom = self.crop_rect.bottom as i16;
                self.width = (self.crop_rect.right - self.crop_rect.left) as u32;
                self.luma_height = (self.crop_rect.bottom - self.crop_rect.top) as u32;
            }
            video_decode_create_info.ulTargetWidth = self.width as u64;
            video_decode_create_info.ulTargetHeight = self.luma_height as u64;
        }

        self.chroma_height =
            f64::ceil(self.luma_height as f64 * self.output_format.chroma_height_factor()) as u32;
        self.num_chroma_planes = self.output_format.chroma_plane_count() as u32;
        self.surface_height = video_decode_create_info.ulTargetHeight;
        self.surface_width = video_decode_create_info.ulTargetWidth;
        self.display_rect.bottom = video_decode_create_info.display_area.bottom as usize;
        self.display_rect.top = video_decode_create_info.display_area.top as usize;
        self.display_rect.left = video_decode_create_info.display_area.left as usize;
        self.display_rect.right = video_decode_create_info.display_area.right as usize;

        // TODO(efyang) print decoding params
        self.video_info += &format!("Video Decoding Params:\n{:#?}", video_decode_create_info);

        // TODO(efyang)
        // don't know why this isn't in the original code, but this reduces runtime immensely
        // (why recreate the decoder if not needed and can just reconfigure?)
        // this should be cleaned up later with an optional type, along with other things
        if self.decoder.is_null() {
            let decoder_ptr = &mut self.decoder;
            do_within_context(&self.context, || unsafe {
                cuvidCreateDecoder(decoder_ptr, &mut video_decode_create_info)
            });
        }

        println!("Session Initialization Time: {:?}", session_init_start.elapsed());

        decode_surface as i32
    }

    /* Return value from HandlePictureDecode() are interpreted as:
     *  0: fail, >=1: succeeded
     */
    fn handle_picture_decode(&mut self, pic_params: *mut CUVIDPICPARAMS) -> i32 {
        // NOTE: this function takes basically no time (~100us), no real point of optimizing this
        debug_assert!(!self.decoder.is_null());
        debug_assert!(!pic_params.is_null());
        unsafe {
            self.picture_decode_index_mapping[(*pic_params).CurrPicIdx as usize] =
                self.decoded_pictures;
        }
        self.decoded_pictures += 1;

        do_within_context(&self.context, || unsafe {
            cuvidDecodePicture(self.decoder, pic_params)
        });

        1
    }

    /* Return value from HandlePictureDisplay() are interpreted as:
     *  0: fail, >=1: succeeded
     */
    fn handle_picture_display(&'a mut self, disp_info: *mut CUVIDPARSERDISPINFO) -> i32 {
        debug_assert!(!self.decoder.is_null());
        debug_assert!(!disp_info.is_null());
        let disp_info = unsafe { *disp_info };
        let mut video_processing_parameters = CUVIDPROCPARAMS {
            progressive_frame: disp_info.progressive_frame,
            second_field: disp_info.repeat_first_field + 1,
            top_field_first: disp_info.top_field_first,
            unpaired_field: if disp_info.repeat_first_field < 0 { 1 } else { 0 },
            output_stream: self.stream,
            ..Default::default()
        };

        let mut src_frame: CUdeviceptr = 0;
        let mut src_pitch = 0;

        // TODO(efyang) : figure out how to make cuvid do_ctxpush_cuvidfunc lifetimes work
        // here with this
        ContextStack::push(&self.context).unwrap();
        unsafe {
            // NOTE: this call takes about 1.6ms (about half the total time of this func)
            // TODO(efyang): optimization in final implementation
            // from https://docs.nvidia.com/video-technologies/video-codec-sdk/nvdec-video-decoder-api-prog-guide/#preparing-the-decoded-frame-for-further-processing
            // When using NVIDIA parser from NVDECODE API, the application can
            // implement a producer-consumer queue between decoding thread (as producer)
            // and mapping thread (as consumer). The queue can contain picture indexes
            // (or other unique identifiers) for frames being decoded. Parser can run on
            // decoding thread. Decoding thread can add the picture index to the queue in
            // display callback and return immediately from callback to continue decoding
            // subsequent frames as they become available. On the other side, mapping thread
            // will monitor the queue. If it sees the queue has non-zero length, it will dequeue
            // the entry and call cuvidMapVideoFrame(…) with nPicIdx as the picture index.
            // Decoding thread must ensure to not reuse the corresponding decode picture buffer
            // for storing the decoded output until its entry is consumed and freed by mapping thread.
            cuvidMapVideoFrame64(
                self.decoder,
                disp_info.picture_index,
                &mut src_frame,
                &mut src_pitch,
                &mut video_processing_parameters,
            )
            .into_cuda_result()
            .unwrap();
        }

        let mut decode_status = CUVIDGETDECODESTATUS::default();
        let decode_result = unsafe {
            // NOTE: this call takes negligible time (as one would expect - 2-4us)
            cuvidGetDecodeStatus(self.decoder, disp_info.picture_index, &mut decode_status)
                .into_cuda_result()
        };
        if decode_result.is_ok()
            && matches!(
                decode_status.decodeStatus,
                cuvidDecodeStatus_enum::cuvidDecodeStatus_Error
                    | cuvidDecodeStatus_enum::cuvidDecodeStatus_Error_Concealed
            )
        {
            eprintln!(
                "Decode Error occurred for picture {}",
                self.picture_decode_index_mapping[disp_info.picture_index as usize]
            );
        }

        // NOTE: this block takes negligible time
        let decoded_frame_ptr: *mut u8;
        {
            let mut frames = self.frames.lock();
            self.decoded_frames += 1;
            if self.decoded_frames > frames.len() {
                // Not enough frames in stock
                self.allocated_frames += 1;
                let frame_data: &mut [u8];
                if self.use_device_frame {
                    let mut frame_data_device_ptr: CUdeviceptr = 0;
                    if self.device_frame_pitched {
                        // TODO(efyang): this should be a specialized type, pitched allocation is not like a normal array
                        // refer to https://stackoverflow.com/questions/16119943/how-and-when-should-i-use-pitched-pointer-with-the-cuda-api
                        unsafe {
                            cuMemAllocPitch_v2(
                                &mut frame_data_device_ptr,
                                &mut self.device_frame_pitch,
                                (self.get_width() * self.bpp as u32) as usize,
                                (self.luma_height + self.chroma_height * self.num_chroma_planes)
                                    as usize,
                                16,
                            )
                            .into_cuda_result()
                            .unwrap();
                        }
                        todo!();
                    } else {
                        unsafe {
                            cuMemAlloc_v2(
                                &mut frame_data_device_ptr,
                                self.get_frame_size() as usize,
                            )
                            .into_cuda_result()
                            .unwrap();
                        }
                    }
                    unsafe {
                        frame_data = std::slice::from_raw_parts_mut(
                            frame_data_device_ptr as *mut u8,
                            self.get_frame_size() as usize,
                        );
                    }
                    frames.push_back(Frame {
                        timestamp: disp_info.timestamp,
                        data: FrameData::Device(frame_data),
                    })
                } else {
                    let frame_data = vec![0; self.get_frame_size() as usize];
                    frames.push_back(Frame {
                        timestamp: disp_info.timestamp,
                        data: FrameData::Owned(frame_data),
                    })
                }
            }
            let frame_len = frames.len();
            // WARNING: This is a potential data race, as the mutex is unlocked when
            // decoded_frame_ptr is being worked with. This is present in the original code, so we copy that here
            // TODO(efyang) fix!
            decoded_frame_ptr = frames[frame_len - 1].data.as_mut().as_mut_ptr();
        }

        // NOTE: memcpys take about 1ms total here
        // Copy luma plane
        let mut m = CUDA_MEMCPY2D {
            srcMemoryType: CUmemorytype_enum::CU_MEMORYTYPE_DEVICE,
            srcDevice: src_frame,
            srcPitch: src_pitch as usize,
            dstMemoryType: if self.use_device_frame {
                CUmemorytype_enum::CU_MEMORYTYPE_DEVICE
            } else {
                CUmemorytype_enum::CU_MEMORYTYPE_HOST
            },
            dstHost: decoded_frame_ptr as *mut c_void,
            dstDevice: decoded_frame_ptr as CUdeviceptr,
            dstPitch: if self.device_frame_pitch != 0 {
                self.device_frame_pitch
            } else {
                (self.get_width() * self.bpp as u32) as usize
            },
            WidthInBytes: (self.get_width() * self.bpp as u32) as usize,
            Height: self.luma_height as usize,
            ..Default::default()
        };
        unsafe {
            cuMemcpy2DAsync_v2(&m, self.stream).into_cuda_result().unwrap();
        }

        // Copy chroma plane
        // NVDEC output has luma height aligned by 2. Adjust chroma offset by aligning height
        m.srcDevice =
            (src_frame + (src_pitch as u64 * ((self.surface_height + 1) & !1))) as CUdeviceptr;
        m.dstHost = ((decoded_frame_ptr) as CUdeviceptr
            + (m.dstPitch as u64 * self.luma_height as u64)) as *mut c_void;
        m.dstDevice = m.dstHost as CUdeviceptr;
        m.Height = self.chroma_height as usize;
        unsafe {
            cuMemcpy2DAsync_v2(&m, self.stream).into_cuda_result().unwrap();
        }

        if self.num_chroma_planes == 2 {
            m.srcDevice = (src_frame + (src_pitch as u64 * ((self.surface_height + 1) & !1) * 2))
                as CUdeviceptr;
            m.dstHost = ((decoded_frame_ptr) as CUdeviceptr
                + (m.dstPitch as u64 * self.luma_height as u64 * 2))
                as *mut c_void;
            m.dstDevice = m.dstHost as CUdeviceptr;
            m.Height = self.chroma_height as usize;
            unsafe {
                cuMemcpy2DAsync_v2(&m, self.stream).into_cuda_result().unwrap();
            }
        }
        // NOTE: this call takes negligible time (about 2us)
        unsafe {
            cuStreamSynchronize(self.stream).into_cuda_result().unwrap();
        }

        ContextStack::pop().unwrap();
        // timestamp already set earlier

        // NOTE: this call takes negligible time (about 2us)
        unsafe {
            cuvidUnmapVideoFrame64(self.decoder, src_frame).into_cuda_result().unwrap();
        }

        1
    }

    /* Called when the parser encounters sequence header for AV1 SVC content
     *  return value interpretation:
     *      < 0 : fail, >=0: succeeded (bit 0-9: currOperatingPoint, bit 10-10: bDispAllLayer, bit 11-30: reserved, must be set 0)
     */
    fn handle_operating_point(&mut self, op_info: *mut CUVIDOPERATINGPOINTINFO) -> i32 {
        debug_assert!(!op_info.is_null());
        unsafe {
            let op_info = *op_info;

            if op_info.codec == Codec::AV1.into()
                && op_info.__bindgen_anon_1.av1.operating_points_cnt > 1
            {
                if self.operating_point
                    >= op_info.__bindgen_anon_1.av1.operating_points_cnt as usize
                {
                    self.operating_point = 0;
                }

                println!(
                    "AV1 SVC clip: operating point count {}  ",
                    op_info.__bindgen_anon_1.av1.operating_points_cnt
                );
                todo!()
            }
        }

        -1
    }

    pub(super) fn new(builder: NvDecoderBuilder) -> Result<Box<Self>, NvDecoderError> {
        let ctx_lock = unsafe {
            let mut ctx_lock = std::ptr::null_mut();
            cuvidCtxLockCreate(&mut ctx_lock, builder.context.get_inner() as *mut ffi::CUctx_st)
                .into_cuda_result()?;
            ctx_lock
        };

        // we create the decoder first with a null parser because the parser needs
        // a reference to the decoder for callbacks, and then create the parser with the reference
        // and then set the parser to the actual instantiated one
        let mut this = Box::new(Self {
            parser: std::ptr::null_mut(),
            context: builder.context,
            use_device_frame: builder.use_device_frame,
            codec: builder.codec,
            device_frame_pitched: builder.device_frame_pitched,
            crop_rect: builder.crop_rect,
            resize_dim: builder.resize_dim,
            max_width: builder.max_width,
            max_height: builder.max_height,
            ctx_lock,
            bitdepth_minus_8: 0,
            chroma_format: ChromaFormat::YUV420,
            output_format: SurfaceFormat::NV12,
            video_format: Default::default(),
            video_info: "".to_string(),
            decoded_frames: 0,
            decoded_frames_returned: 0,
            allocated_frames: 0,
            device_frame_pitch: 0,
            stream: std::ptr::null_mut(),
            frames: Arc::new(Mutex::new(VecDeque::new())),
            picture_decode_index_mapping: [0; 32],
            decoded_pictures: 0,
            decoder: std::ptr::null_mut(),
            operating_point: 0,
            width: 0,
            luma_height: 0,
            chroma_height: 0,
            num_chroma_planes: 0,
            bpp: 1,
            display_rect: Default::default(),
            surface_height: 0,
            surface_width: 0,
        });

        // TODO: handle errors
        let mut params = CUVIDPARSERPARAMS {
            CodecType: builder.codec.into(),
            ulMaxNumDecodeSurfaces: 1,
            ulClockRate: builder.clock_rate,
            ulMaxDisplayDelay: if builder.low_latency { 0 } else { 1 },

            pUserData: &mut *this as *mut NvDecoder as *mut c_void,
            pfnSequenceCallback: Some(handle_video_sequence_proc),
            pfnDecodePicture: Some(handle_picture_decode_proc),
            pfnDisplayPicture: Some(handle_picture_display_proc),
            pfnGetOperatingPoint: Some(handle_operating_point_proc),

            // TODO: other stuff not mentioned: sane defaults?
            // most likely broken tbh
            _bitfield_1: CUVIDPARSERPARAMS::new_bitfield_1(0, 0),
            _bitfield_align_1: [0; 0],
            ulErrorThreshold: 0,
            uReserved1: [0; 4],
            pvReserved2: [std::ptr::null_mut(); 6],
            pExtVideoInfo: std::ptr::null_mut(),
        };

        unsafe {
            ffi::cuvidCreateVideoParser(&mut this.parser, &mut params).into_cuda_result()?;
        }

        Ok(this)
    }

    /// Returns the number of frames decoded
    ///
    /// # Arguments
    /// * arg
    pub fn decode(
        &mut self,
        data: &[u8],
        flags: DecoderPacketFlags,
        timestamp: i64,
    ) -> Result<usize, NvDecoderError> {
        self.decoded_frames = 0;
        self.decoded_frames_returned = 0;
        let flags: CUvideopacketflags::Type = flags.into();
        let mut packet = CUVIDSOURCEDATAPACKET {
            flags: (flags as u32 | CUVID_PKT_TIMESTAMP as u32) as c_ulong,
            payload_size: data.len() as u64,
            payload: data.as_ptr(),
            timestamp,
        };

        if data.is_empty() {
            packet.flags |= CUVID_PKT_ENDOFSTREAM as c_ulong;
        }

        unsafe {
            cuvidParseVideoData(self.parser, &mut packet as *mut CUVIDSOURCEDATAPACKET)
                .into_cuda_result()?;
        }

        self.stream = std::ptr::null_mut();

        Ok(self.decoded_frames)
    }

    // Another possible race condition in the original code here
    // should be solved with the use of the mutexguard
    pub fn get_frame(&mut self) -> Option<MappedMutexGuard<'_, Frame<'a>>> {
        if self.decoded_frames > 0 {
            let frames_locked = self.frames.lock();
            self.decoded_frames -= 1;
            self.decoded_frames_returned += 1;
            Some(MutexGuard::map(frames_locked, |frames| {
                &mut frames[self.decoded_frames_returned - 1]
            }))
        } else {
            None
        }
    }

    /// Note: the locked/unlocked api is like the following:
    /// If a frame can be used by the decoder, then it is considered unlocked (anything inside of self.frames)
    /// A frame is locked when it cannot be used by the decoder (it will be removed from the internal framebuffer)
    /// In this way, one can return used frames to the decoder by unlocking them to avoid excessive memory allocations.
    pub fn get_locked_frame(&mut self) -> Option<Frame<'a>> {
        if self.decoded_frames > 0 {
            let mut frames_locked = self.frames.lock();
            self.decoded_frames -= 1;
            frames_locked.pop_front()
        } else {
            None
        }
    }

    pub fn unlock_frame(&mut self, mut frame: Frame<'a>) {
        let mut frames_locked = self.frames.lock();
        frame.timestamp = 0;
        frames_locked.push_back(frame);
    }

    pub fn get_width(&self) -> u32 {
        assert!(self.width != 0);
        if matches!(self.output_format, SurfaceFormat::NV12 | SurfaceFormat::P016) {
            (self.width + 1) & !1
        } else {
            self.width
        }
    }

    pub fn get_height(&self) -> u32 {
        assert!(self.luma_height != 0);
        self.luma_height
    }

    pub fn get_frame_size(&self) -> u32 {
        assert!(self.width != 0);
        self.get_width()
            * (self.luma_height + self.chroma_height * self.num_chroma_planes)
            * self.bpp as u32
    }

    pub fn set_reconfig_params() -> Result<(), NvDecoderError> {
        todo!()
    }

    pub fn get_video_info(&self) -> &str {
        &self.video_info
    }

    pub fn get_output_format(&self) -> SurfaceFormat {
        self.output_format
    }
}

impl<'a> Drop for NvDecoder<'a> {
    fn drop(&mut self) {
        let session_deinit_start = Instant::now();
        if !self.parser.is_null() {
            unsafe {
                let err = cuvidDestroyVideoParser(self.parser as ffi::CUvideoparser);
                err.into_cuda_result().expect("Failure on nvdecoder parser destroy");
            }
        }

        if !self.decoder.is_null() {
            unsafe {
                let err = cuvidDestroyDecoder(self.decoder);
                err.into_cuda_result().expect("Failure on nvdecoder decoder destroy");
            }
        }

        for frame in self.frames.lock().iter_mut() {
            if self.use_device_frame {
                unsafe {
                    cuMemFree_v2(frame.data.as_mut().as_mut_ptr() as CUdeviceptr)
                        .into_cuda_result()
                        .expect("Failure on nvdecoder frame free");
                }
            }
        }

        ContextStack::pop().unwrap();
        unsafe {
            let err = cuvidCtxLockDestroy(self.ctx_lock);
            err.into_cuda_result().expect("Failure on nvdecoder ctx lock destroy");
        }
        println!("Session Deinitialization Time: {:?}", session_deinit_start.elapsed());
    }
}
