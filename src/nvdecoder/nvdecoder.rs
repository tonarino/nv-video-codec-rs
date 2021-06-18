use crate::{
    common::{
        CudaResult, CudaVideoChromaFormat, CudaVideoCodec, CudaVideoCreateFlags,
        CudaVideoDeinterlaceMode, CudaVideoSurfaceFormat, Dim, IntoCudaResult, Rect,
    },
    nvdecoder::FrameData,
};
use nv_video_codec_sys::{
    self, CUdeviceptr, CUmemorytype_enum, CUstream, CUvideoctxlock, CUvideodecoder,
    CUvideopacketflags::{self, CUVID_PKT_ENDOFSTREAM, CUVID_PKT_TIMESTAMP},
    CUvideoparser, __BindgenBitfieldUnit, cuArray3DCreate_v2, cuMemAllocPitch_v2, cuMemAlloc_v2,
    cuMemFree_v2, cuMemcpy2DAsync_v2, cuStreamSynchronize,
    cudaVideoCodec_enum::cudaVideoCodec_NV12,
    cudaVideoSurfaceFormat_enum::{cudaVideoSurfaceFormat_NV12, cudaVideoSurfaceFormat_P016},
    cuvidCreateDecoder, cuvidCtxLockCreate, cuvidCtxLockDestroy, cuvidDecodePicture,
    cuvidDecodeStatus_enum, cuvidDestroyDecoder, cuvidDestroyVideoParser, cuvidGetDecodeStatus,
    cuvidGetDecoderCaps, cuvidMapVideoFrame64, cuvidParseVideoData, cuvidUnmapVideoFrame64, size_t,
    CUDA_MEMCPY2D, CUVIDDECODECAPS, CUVIDDECODECREATEINFO, CUVIDEOFORMAT, CUVIDGETDECODESTATUS,
    CUVIDOPERATINGPOINTINFO, CUVIDPARSERDISPINFO, CUVIDPARSERPARAMS, CUVIDPICPARAMS,
    CUVIDPROCPARAMS, CUVIDSOURCEDATAPACKET, _CUVIDDECODECAPS,
};
use parking_lot::{Mutex, MutexGuard};
use rustacuda::context::{Context, ContextHandle, ContextStack};
use std::{
    borrow::BorrowMut, collections::VecDeque, ffi::c_void, mem::MaybeUninit, ops::Deref,
    os::raw::c_ulong, sync::Arc, time::Instant,
};

use super::{DecoderPacketFlags, Frame, NvDecoderError};

pub struct NvDecoderBuilder {
    context: Context,
    use_device_frame: bool,
    codec: CudaVideoCodec,
    low_latency: bool,
    device_frame_pitched: bool,
    crop_rect: Rect,
    resize_dim: Dim,
    max_width: u32,
    max_height: u32,
    clock_rate: u32,
}

impl NvDecoderBuilder {
    builder_field_setter!(low_latency: bool);

    builder_field_setter!(device_frame_pitched: bool);

    builder_field_setter!(crop_rect: Rect);

    builder_field_setter!(resize_dim: Dim);

    builder_field_setter!(max_width: u32);

    builder_field_setter!(max_height: u32);

    builder_field_setter!(clock_rate: u32);

    pub fn new(context: Context, use_device_frame: bool, codec: CudaVideoCodec) -> Self {
        Self {
            context,
            use_device_frame,
            codec,
            low_latency: false,
            device_frame_pitched: false,
            crop_rect: Default::default(),
            resize_dim: Default::default(),
            max_width: 0,
            max_height: 0,
            clock_rate: 1000,
        }
    }

    pub fn build<'a>(self) -> Result<NvDecoder<'a>, NvDecoderError> {
        NvDecoder::new(
            self.context,
            self.use_device_frame,
            self.codec,
            self.low_latency,
            self.device_frame_pitched,
            self.crop_rect,
            self.resize_dim,
            self.max_width,
            self.max_height,
            self.clock_rate,
        )
    }
}

pub struct NvDecoder<'a> {
    parser: CUvideoparser,
    decoder: CUvideodecoder,
    context: Context,
    use_device_frame: bool,
    codec: CudaVideoCodec,
    chroma_format: CudaVideoChromaFormat,
    output_format: CudaVideoSurfaceFormat,
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
    device_frame_pitch: size_t,

    n_decoded_frame: usize,
    n_decoded_frame_returned: usize,
    n_frame_alloc: usize,
    stream: CUstream,
    /// need mutex to cover callbacks
    frames: Arc<Mutex<VecDeque<Frame<'a>>>>,
    n_pic_num_in_decode_order: [usize; 32],
    n_decode_pic_cnt: usize,
    n_operating_point: usize,

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
) -> i32 {
    println!("handle_video_sequence_proc");
    debug_assert!(!decoder.is_null());
    (decoder as *mut NvDecoder).as_mut().unwrap().handle_video_sequence(video_format)
}

/// decoder refers to an NvDecoder
/// Callback function to be registered for getting a callback when a decoded frame is ready to be decoded
unsafe extern "C" fn handle_picture_decode_proc(
    decoder: *mut c_void,
    pic_params: *mut CUVIDPICPARAMS,
) -> i32 {
    println!("handle_picture_decode_proc");
    debug_assert!(!decoder.is_null());
    (decoder as *mut NvDecoder).as_mut().unwrap().handle_picture_decode(pic_params)
}

/// decoder refers to an NvDecoder
/// Callback function to be registered for getting a callback when a decoded frame is available for display
unsafe extern "C" fn handle_picture_display_proc(
    decoder: *mut c_void,
    disp_info: *mut CUVIDPARSERDISPINFO,
) -> i32 {
    println!("handle_picture_display_proc");
    debug_assert!(!decoder.is_null());
    (decoder as *mut NvDecoder).as_mut().unwrap().handle_picture_display(disp_info)
}

/// decoder refers to an NvDecoder
/// Callback function to be registered for getting a callback to get operating point when AV1 SVC sequence header start.
unsafe extern "C" fn handle_operating_point_proc(
    decoder: *mut c_void,
    op_info: *mut CUVIDOPERATINGPOINTINFO,
) -> i32 {
    println!("handle_operating_point_proc");
    debug_assert!(!decoder.is_null());
    (decoder as *mut NvDecoder).as_mut().unwrap().handle_operating_point(op_info)
}

fn do_ctxpush_cuvidfunc<'a, F, T>(context: &'a Context, mut func: F)
where
    F: FnMut() -> T,
    T: IntoCudaResult<()>,
{
    ContextStack::push(context).unwrap();
    func().into_cuda_result().expect("Cuda NVDEC api call failure");
    ContextStack::pop().unwrap();
}

impl<'a> NvDecoder<'a> {
    // TODO(efyang) : switch these over to result types and just handle the results
    fn handle_video_sequence(&mut self, video_format: *mut CUVIDEOFORMAT) -> i32 {
        println!("Handle video sequence");
        let session_init_start = Instant::now();

        let video_format = unsafe { *video_format };
        self.video_info = format!("Video Input Information:\n{:?}", video_format);

        let decode_surface = video_format.min_num_decode_surfaces;

        let mut decode_caps = CUVIDDECODECAPS::default();
        decode_caps.eCodecType = video_format.codec;
        decode_caps.eChromaFormat = video_format.codec;
        decode_caps.nBitDepthMinus8 = video_format.bit_depth_luma_minus8 as u32;
        do_ctxpush_cuvidfunc(&self.context, || unsafe {
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
            todo!()
        }

        // eCodec has been set in the constructor (for parser). Here it's set again for potential correction
        self.codec = video_format.codec.into();
        self.chroma_format = video_format.chroma_format.into();
        self.bitdepth_minus_8 = video_format.bit_depth_luma_minus8 as i32;
        self.bpp = if self.bitdepth_minus_8 > 0 { 2 } else { 1 };

        // Set the output surface format same as chroma format
        if matches!(
            self.chroma_format,
            CudaVideoChromaFormat::YUV420 | CudaVideoChromaFormat::Monochrome
        ) {
            self.output_format = if video_format.bit_depth_luma_minus8 != 0 {
                CudaVideoSurfaceFormat::P016
            } else {
                CudaVideoSurfaceFormat::NV12
            };
        } else if matches!(self.chroma_format, CudaVideoChromaFormat::YUV444) {
            self.output_format = if video_format.bit_depth_luma_minus8 != 0 {
                CudaVideoSurfaceFormat::YUV444_16bit
            } else {
                CudaVideoSurfaceFormat::YUV444
            };
        } else if matches!(self.chroma_format, CudaVideoChromaFormat::YUV422) {
            // no 4:2:2 output format supported yet so make 420 default
            self.output_format = CudaVideoSurfaceFormat::NV12;
        }

        // TODO(efyang) : create safe wrapper over VideoFormat
        self.video_format = video_format;

        let mut video_decode_create_info = CUVIDDECODECREATEINFO::default();
        video_decode_create_info.CodecType = video_format.codec;
        video_decode_create_info.ChromaFormat = video_format.chroma_format;
        video_decode_create_info.OutputFormat = self.output_format.into();
        video_decode_create_info.bitDepthMinus8 = video_format.bit_depth_luma_minus8 as u64;
        if video_format.progressive_sequence != 0 {
            video_decode_create_info.DeinterlaceMode = CudaVideoDeinterlaceMode::Weave.into();
        } else {
            video_decode_create_info.DeinterlaceMode = CudaVideoDeinterlaceMode::Adaptive.into();
        }
        video_decode_create_info.ulNumOutputSurfaces = 2;
        // With PreferCUVID, JPEG is still decoded by CUDA while video is decoded by NVDEC hardware
        video_decode_create_info.ulCreationFlags = {
            let cf: u32 = CudaVideoCreateFlags::PreferCUVID.into();
            cf as u64
        };
        video_decode_create_info.ulNumDecodeSurfaces = decode_surface as u64;
        video_decode_create_info.vidLock = self.ctx_lock;
        video_decode_create_info.ulWidth = video_format.coded_width as u64;
        video_decode_create_info.ulHeight = video_format.coded_height as u64;
        // AV1 has max width/height of sequence in sequence header
        if matches!(video_format.codec.into(), CudaVideoCodec::AV1)
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

        if !(self.crop_rect.right != 0 && self.crop_rect.bottom != 0)
            && !(self.resize_dim.width != 0 && self.resize_dim.height != 0)
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
        self.video_info += &format!("Video Decoding Params:\n{:?}", video_decode_create_info);

        let decoder_ptr = &mut self.decoder;
        do_ctxpush_cuvidfunc(&self.context, || unsafe {
            cuvidCreateDecoder(decoder_ptr, &mut video_decode_create_info)
        });

        println!(
            "Session Initialization Time: {} seconds",
            session_init_start.elapsed().as_secs_f64()
        );

        decode_surface as i32
    }

    fn handle_picture_decode(&mut self, pic_params: *mut CUVIDPICPARAMS) -> i32 {
        println!("Handle picture decode");
        debug_assert!(!self.decoder.is_null());
        debug_assert!(!pic_params.is_null());
        unsafe {
            self.n_pic_num_in_decode_order[(*pic_params).CurrPicIdx as usize] =
                self.n_decode_pic_cnt;
        }
        self.n_decode_pic_cnt += 1;

        do_ctxpush_cuvidfunc(&self.context, || unsafe {
            cuvidDecodePicture(self.decoder, pic_params)
        });

        return 1;
    }

    fn handle_picture_display(&'a mut self, disp_info: *mut CUVIDPARSERDISPINFO) -> i32 {
        println!("Handle picture display");
        debug_assert!(!disp_info.is_null());
        let disp_info = unsafe { *disp_info };
        let mut video_processing_parameters = CUVIDPROCPARAMS::default();
        video_processing_parameters.progressive_frame = disp_info.progressive_frame;
        video_processing_parameters.second_field = disp_info.repeat_first_field + 1;
        video_processing_parameters.top_field_first = disp_info.top_field_first;
        video_processing_parameters.unpaired_field =
            if disp_info.repeat_first_field < 0 { 1 } else { 0 };
        video_processing_parameters.output_stream = self.stream;

        let mut src_frame: CUdeviceptr = 0;
        let mut src_pitch = 0;

        // TODO(efyang) : figure out how to make cuvid do_ctxpush_cuvidfunc lifetimes work with this
        ContextStack::push(&self.context).unwrap();
        unsafe {
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
                self.n_pic_num_in_decode_order[disp_info.picture_index as usize]
            );
        }

        let decoded_frame_ptr: *mut u8;
        {
            let mut frames = self.frames.lock();
            self.n_decoded_frame += 1;
            if self.n_decoded_frame > frames.len() {
                // Not enough frames in stock
                self.n_frame_alloc += 1;
                let frame_data: &mut [u8];
                if self.use_device_frame {
                    let mut frame_data_device_ptr: CUdeviceptr = 0;
                    if self.device_frame_pitched {
                        // refer to https://stackoverflow.com/questions/16119943/how-and-when-should-i-use-pitched-pointer-with-the-cuda-api
                        todo!();
                        // unsafe {
                        //     cuMemAllocPitch_v2(
                        //         &mut frame_data_device_ptr,
                        //         &mut self.device_frame_pitch,
                        //         (self.get_width() * self.bpp as u32) as size_t,
                        //         (self.luma_height + self.chroma_height * self.num_chroma_planes)
                        //             as size_t,
                        //         16,
                        //     )
                        //     .into_cuda_result()?;
                        // }
                    } else {
                        unsafe {
                            cuMemAlloc_v2(&mut frame_data_device_ptr, self.get_frame_size() as u64)
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

        // Copy luma plane
        let mut m = CUDA_MEMCPY2D::default();
        m.srcMemoryType = CUmemorytype_enum::CU_MEMORYTYPE_DEVICE;
        m.srcDevice = src_frame;
        m.srcPitch = src_pitch as u64;
        m.dstMemoryType = if self.use_device_frame {
            CUmemorytype_enum::CU_MEMORYTYPE_DEVICE
        } else {
            CUmemorytype_enum::CU_MEMORYTYPE_HOST
        };
        m.dstHost = decoded_frame_ptr as *mut c_void;
        m.dstDevice = decoded_frame_ptr as CUdeviceptr;
        m.dstPitch = if self.device_frame_pitch != 0 {
            self.device_frame_pitch
        } else {
            (self.get_width() * self.bpp as u32) as u64
        };
        m.WidthInBytes = (self.get_width() * self.bpp as u32) as u64;
        m.Height = self.luma_height as u64;
        unsafe {
            cuMemcpy2DAsync_v2(&m, self.stream).into_cuda_result().unwrap();
        }

        // Copy chroma plane
        // NVDEC output has luma height aligned by 2. Adjust chroma offset by aligning height
        m.srcDevice =
            (src_frame + (src_pitch as u64 * ((self.surface_height + 1) & !1))) as CUdeviceptr;
        m.dstHost = ((decoded_frame_ptr) as CUdeviceptr + (m.dstPitch * self.luma_height as u64))
            as *mut c_void;
        m.dstDevice = m.dstHost as CUdeviceptr;
        m.Height = self.chroma_height as u64;
        unsafe {
            cuMemcpy2DAsync_v2(&m, self.stream).into_cuda_result().unwrap();
        }

        if self.num_chroma_planes == 2 {
            m.srcDevice = (src_frame + (src_pitch as u64 * ((self.surface_height + 1) & !1) * 2))
                as CUdeviceptr;
            m.dstHost = ((decoded_frame_ptr) as CUdeviceptr
                + (m.dstPitch * self.luma_height as u64 * 2))
                as *mut c_void;
            m.dstDevice = m.dstHost as CUdeviceptr;
            m.Height = self.chroma_height as u64;
            unsafe {
                cuMemcpy2DAsync_v2(&m, self.stream).into_cuda_result().unwrap();
            }
        }
        unsafe {
            cuStreamSynchronize(self.stream).into_cuda_result().unwrap();
        }

        ContextStack::pop().unwrap();
        // timestamp already set earlier

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

            if op_info.codec == CudaVideoCodec::AV1.into()
                && op_info.__bindgen_anon_1.av1.operating_points_cnt > 1
            {
                if self.n_operating_point
                    >= op_info.__bindgen_anon_1.av1.operating_points_cnt as usize
                {
                    self.n_operating_point = 0;
                }

                println!(
                    "AV1 SVC clip: operating point count {}  ",
                    op_info.__bindgen_anon_1.av1.operating_points_cnt
                );
                todo!()
            }
        }

        return -1;
    }
}

impl<'a> NvDecoder<'a> {
    fn new(
        context: Context,
        use_device_frame: bool,
        codec: CudaVideoCodec,
        low_latency: bool,
        device_frame_pitched: bool,
        crop_rect: Rect,
        resize_dim: Dim,
        max_width: u32,
        max_height: u32,
        clock_rate: u32,
    ) -> Result<Self, NvDecoderError> {
        let ctx_lock = unsafe {
            let mut ctx_lock = std::ptr::null_mut();
            cuvidCtxLockCreate(
                &mut ctx_lock,
                context.get_inner() as *mut nv_video_codec_sys::CUctx_st,
            )
            .into_cuda_result()?;
            ctx_lock
        };

        // we create the decoder first with a null parser because the parser needs
        // a reference to the decoder for callbacks, and then create the parser with the reference
        // and then set the parser to the actual instantiated one
        let mut this = Self {
            parser: std::ptr::null_mut(),
            context,
            use_device_frame,
            codec,
            device_frame_pitched,
            crop_rect,
            resize_dim,
            max_width,
            max_height,
            ctx_lock,
            bitdepth_minus_8: 0,
            chroma_format: CudaVideoChromaFormat::YUV420,
            output_format: CudaVideoSurfaceFormat::NV12,
            video_format: Default::default(),
            video_info: "".to_string(),
            n_decoded_frame: 0,
            n_decoded_frame_returned: 0,
            n_frame_alloc: 0,
            device_frame_pitch: 0,
            stream: std::ptr::null_mut(),
            frames: Arc::new(Mutex::new(VecDeque::new())),
            n_pic_num_in_decode_order: [0; 32],
            n_decode_pic_cnt: 0,
            decoder: std::ptr::null_mut(),
            n_operating_point: 0,
            width: 0,
            luma_height: 0,
            chroma_height: 0,
            num_chroma_planes: 0,
            bpp: 1,
            display_rect: Default::default(),
            surface_height: 0,
            surface_width: 0,
        };

        // TODO: handle errors
        let mut params = CUVIDPARSERPARAMS {
            CodecType: codec.into(),
            ulMaxNumDecodeSurfaces: 1,
            ulClockRate: clock_rate,
            ulMaxDisplayDelay: if low_latency { 0 } else { 1 },

            pUserData: &mut this as *mut NvDecoder as *mut c_void,
            pfnSequenceCallback: Some(handle_video_sequence_proc),
            pfnDecodePicture: Some(handle_picture_decode_proc),
            pfnDisplayPicture: Some(handle_picture_display_proc),
            pfnGetOperatingPoint: Some(handle_operating_point_proc),

            // TODO: other stuff not mentioned: sane defaults?
            // most likely broken tbh
            _bitfield_1: CUVIDPARSERPARAMS::new_bitfield_1(1, 31),
            _bitfield_align_1: [0; 0],
            ulErrorThreshold: 0,
            uReserved1: [0; 4],
            pvReserved2: [std::ptr::null_mut(); 6],
            pExtVideoInfo: std::ptr::null_mut(),
        };

        dbg!(this.parser);
        unsafe {
            nv_video_codec_sys::cuvidCreateVideoParser(&mut this.parser, &mut params)
                .into_cuda_result()?;
        }
        dbg!(this.parser);

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
        self.n_decoded_frame = 0;
        self.n_decoded_frame_returned = 0;
        let flags: CUvideopacketflags::Type = flags.into();
        let mut packet = CUVIDSOURCEDATAPACKET {
            flags: (flags as u32 | CUVID_PKT_TIMESTAMP as u32) as c_ulong,
            payload_size: data.len() as u64,
            payload: data.as_ptr(),
            timestamp,
        };

        if data.len() == 0 {
            packet.flags |= CUVID_PKT_ENDOFSTREAM as c_ulong;
        }

        unsafe {
            cuvidParseVideoData(self.parser, &mut packet as *mut CUVIDSOURCEDATAPACKET)
                .into_cuda_result()?;
        }

        self.stream = std::ptr::null_mut();

        Ok(self.n_decoded_frame)
    }

    // TODO(efyang): which implementation to use?
    // pub fn get_frame<'a>(&'a mut self) -> Option<MappedMutexGuard<'a, Frame>> {
    //     if self.n_decoded_frame > 0 {
    //         let frames_locked = self.frames.lock();
    //         self.n_decoded_frame -= 1;
    //         self.n_decoded_frame_returned += 1;
    //         Some(MutexGuard::map(frames_locked, |frames| {
    //             &mut frames[self.n_decoded_frame_returned as usize]
    //         }))
    //     } else {
    //         None
    //     }
    // }

    // another possible race condition in the original code
    pub fn get_frame(&'a mut self) -> Option<Box<dyn Deref<Target = Frame<'a>> + 'a>> {
        if self.n_decoded_frame > 0 {
            let frames_locked = self.frames.lock();
            self.n_decoded_frame -= 1;
            self.n_decoded_frame_returned += 1;
            Some(Box::new(MutexGuard::map(frames_locked, |frames| {
                &mut frames[self.n_decoded_frame_returned as usize]
            })))
        } else {
            None
        }
    }

    /// Note: the locked/unlocked api is like the following:
    /// If a frame can be used by the decoder, then it is considered unlocked (anything inside of self.frames)
    /// A frame is locked when it cannot be used by the decoder (it will be removed from the internal framebuffer)
    /// In this way, one can return used frames to the decoder by unlocking them to avoid excessive memory allocations.
    pub fn get_locked_frame(&mut self) -> Option<Frame> {
        if self.n_decoded_frame > 0 {
            let mut frames_locked = self.frames.lock();
            self.n_decoded_frame -= 1;
            frames_locked.pop_front()
        } else {
            None
        }
    }

    pub fn unlock_frame(&'a mut self, mut frame: Frame<'a>) {
        let mut frames_locked = self.frames.lock();
        frame.timestamp = 0;
        frames_locked.push_back(frame);
    }

    pub fn get_width(&self) -> u32 {
        assert!(self.width != 0);
        if matches!(self.output_format, CudaVideoSurfaceFormat::NV12 | CudaVideoSurfaceFormat::P016)
        {
            (self.width + 1) & !1
        } else {
            self.width
        }
    }

    pub fn get_frame_size(&self) -> u32 {
        assert!(self.width != 0);
        self.get_width()
            * (self.luma_height + self.chroma_height * self.num_chroma_planes)
            * self.bpp as u32
    }

    pub fn set_reconfig_params() -> Result<(), ()> {
        todo!()
    }

    pub fn get_video_info(&self) -> &str {
        &self.video_info
    }
}

impl<'a> Drop for NvDecoder<'a> {
    fn drop(&mut self) {
        let session_deinit_start = Instant::now();
        if !self.parser.is_null() {
            unsafe {
                let err = cuvidDestroyVideoParser(self.parser as nv_video_codec_sys::CUvideoparser);
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
        println!(
            "Session Deinitialization Time: {} seconds",
            session_deinit_start.elapsed().as_secs_f64()
        );
    }
}
