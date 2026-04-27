use super::{
    types::{ChromaFormat, Codec, CreateFlags, DeinterlaceMode, Dim, Rect, SurfaceFormat},
    DecoderPacketFlags, NvDecoderError,
};
use crate::{
    common::{cuda_result::IntoCudaResult, util::ContextStack},
    decoder::{
        frame::{info::FrameInfo, Buffer as _, DecodingOutput, Frame, FrameAllocator, OwnedFrame},
        NvDecoderBuilder,
    },
};
use cudarc::driver::CudaContext;
use ffi::{
    cuMemcpy2DAsync_v2, cuStreamSynchronize, cudaVideoCreateFlags_enum, cuvidCreateDecoder,
    cuvidCtxLockCreate, cuvidCtxLockDestroy, cuvidDecodePicture, cuvidDecodeStatus_enum,
    cuvidDestroyDecoder, cuvidDestroyVideoParser, cuvidGetDecodeStatus, cuvidGetDecoderCaps,
    cuvidMapVideoFrame64, cuvidParseVideoData, cuvidUnmapVideoFrame64, CUdeviceptr,
    CUmemorytype_enum, CUstream, CUvideoctxlock, CUvideodecoder,
    CUvideopacketflags::{self, CUVID_PKT_ENDOFSTREAM, CUVID_PKT_TIMESTAMP},
    CUvideoparser, CUDA_MEMCPY2D, CUVIDDECODECAPS, CUVIDDECODECREATEINFO, CUVIDEOFORMAT,
    CUVIDGETDECODESTATUS, CUVIDOPERATINGPOINTINFO, CUVIDPARSERDISPINFO, CUVIDPARSERPARAMS,
    CUVIDPICPARAMS, CUVIDPROCPARAMS, CUVIDSOURCEDATAPACKET,
};
use nv_video_codec_sys as ffi;
use std::{
    collections::VecDeque,
    convert::TryInto,
    os::raw::{c_int, c_ulong, c_void},
    sync::Arc,
    time::Instant,
};

pub struct NvDecoder<A: FrameAllocator> {
    parser: CUvideoparser,
    decoder: CUvideodecoder,
    context: Arc<CudaContext>,
    codec: Codec,
    chroma_format: ChromaFormat,
    video_format: CUVIDEOFORMAT,
    crop_rect: Rect,
    resize_dim: Dim,
    max_width: u32,
    max_height: u32,
    ctx_lock: CUvideoctxlock,
    bitdepth_minus_8: i32,
    display_rect: Rect,

    decoded_frames: usize,
    decoded_frames_returned: usize,
    allocated_frames: usize,
    stream: CUstream,
    // TODO(mbernat): This used to be wrapped in Arc<Mutex<_>>, find out why.
    frames: VecDeque<OwnedFrame<A>>,
    picture_decode_index_mapping: [usize; 32],
    decoded_pictures: usize,
    operating_point: usize,

    /// height of the mapped surface
    surface_height: u64,
    surface_width: u64,

    frame_info: Option<FrameInfo>,
    video_info: String,
}

/// decoder refers to an NvDecoder
/// Callback function to be registered for getting a callback when decoding of sequence starts
unsafe extern "C" fn handle_video_sequence_proc<A: FrameAllocator>(
    decoder: *mut c_void,
    video_format: *mut CUVIDEOFORMAT,
) -> c_int {
    debug_assert!(!decoder.is_null());
    (decoder as *mut NvDecoder<A>).as_mut().unwrap().handle_video_sequence(video_format)
}

/// decoder refers to an NvDecoder
/// Callback function to be registered for getting a callback when a decoded frame is ready to be decoded
unsafe extern "C" fn handle_picture_decode_proc<A: FrameAllocator>(
    decoder: *mut c_void,
    pic_params: *mut CUVIDPICPARAMS,
) -> c_int {
    debug_assert!(!decoder.is_null());
    (decoder as *mut NvDecoder<A>).as_mut().unwrap().handle_picture_decode(pic_params)
}

/// decoder refers to an NvDecoder
/// Callback function to be registered for getting a callback when a decoded frame is available for display
unsafe extern "C" fn handle_picture_display_proc<A: FrameAllocator>(
    decoder: *mut c_void,
    disp_info: *mut CUVIDPARSERDISPINFO,
) -> c_int {
    debug_assert!(!decoder.is_null());
    (decoder as *mut NvDecoder<A>).as_mut().unwrap().handle_picture_display(disp_info)
}

/// decoder refers to an NvDecoder
/// Callback function to be registered for getting a callback to get operating point when AV1 SVC sequence header start.
unsafe extern "C" fn handle_operating_point_proc<A: FrameAllocator>(
    decoder: *mut c_void,
    op_info: *mut CUVIDOPERATINGPOINTINFO,
) -> c_int {
    debug_assert!(!decoder.is_null());
    (decoder as *mut NvDecoder<A>).as_mut().unwrap().handle_operating_point(op_info)
}

fn do_within_context<F, T>(context: &CudaContext, mut func: F)
where
    F: FnMut() -> T,
    T: IntoCudaResult<()>,
{
    ContextStack::push(context).unwrap();
    func().into_cuda_result().expect("Cuda NVDEC api call failure");
    ContextStack::pop().unwrap();
}

impl<A: FrameAllocator> NvDecoder<A> {
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
        do_within_context(&self.context, || unsafe { cuvidGetDecoderCaps(&raw mut decode_caps) });

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

        if self.frame_info.is_none() {
            // cuvidCreateDecoder() has been called before, and now there's possible config change
            // L229
            // TODO(efyang) - technically not needed for our application, but should be done
            // todo!()
        }

        // eCodec has been set in the constructor (for parser). Here it's set again for potential correction
        self.codec = video_format.codec.try_into().unwrap();
        self.chroma_format = video_format.chroma_format.try_into().unwrap();
        self.bitdepth_minus_8 = video_format.bit_depth_luma_minus8 as i32;
        let bpp = if self.bitdepth_minus_8 > 0 { 2 } else { 1 };

        // Set the output surface format same as chroma format
        let output_format =
            if matches!(self.chroma_format, ChromaFormat::YUV420 | ChromaFormat::Monochrome) {
                if video_format.bit_depth_luma_minus8 != 0 {
                    SurfaceFormat::P016
                } else {
                    SurfaceFormat::NV12
                }
            } else if matches!(self.chroma_format, ChromaFormat::YUV444) {
                if video_format.bit_depth_luma_minus8 != 0 {
                    SurfaceFormat::YUV444_16bit
                } else {
                    SurfaceFormat::YUV444
                }
            } else if matches!(self.chroma_format, ChromaFormat::YUV422) {
                // no 4:2:2 output format supported yet so make 420 default
                SurfaceFormat::NV12
            } else {
                // fall back to NV12
                SurfaceFormat::NV12
            };

        // TODO(efyang) : create safe wrapper over VideoFormat
        self.video_format = video_format;

        let mut video_decode_create_info = CUVIDDECODECREATEINFO {
            CodecType: video_format.codec,
            ChromaFormat: video_format.chroma_format,
            OutputFormat: output_format.into(),
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

        let mut width = 0;
        let mut luma_height = 0;

        if (self.crop_rect.right == 0 || self.crop_rect.bottom == 0)
            && (self.resize_dim.width == 0 || self.resize_dim.height == 0)
        {
            width = (video_format.display_area.right - video_format.display_area.left) as u32;
            luma_height = (video_format.display_area.bottom - video_format.display_area.top) as u32;
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
                width = self.resize_dim.width as u32;
                luma_height = self.resize_dim.height as u32;
            }

            // TODO(efyang) change rect and dim to be u32
            if self.crop_rect.right != 0 && self.crop_rect.bottom != 0 {
                video_decode_create_info.display_area.left = self.crop_rect.left as i16;
                video_decode_create_info.display_area.top = self.crop_rect.top as i16;
                video_decode_create_info.display_area.right = self.crop_rect.right as i16;
                video_decode_create_info.display_area.bottom = self.crop_rect.bottom as i16;
                width = (self.crop_rect.right - self.crop_rect.left) as u32;
                luma_height = (self.crop_rect.bottom - self.crop_rect.top) as u32;
            }
            video_decode_create_info.ulTargetWidth = width as u64;
            video_decode_create_info.ulTargetHeight = luma_height as u64;
        }

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

        self.frame_info = Some(FrameInfo::new(output_format, bpp, width, luma_height));

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

        {
            let pic_params =
                // SAFETY: `pic_params` points to valid data and there is no aliasing.
                unsafe { pic_params.as_ref().expect("pic_params to be set when decoding") };

            self.picture_decode_index_mapping[pic_params.CurrPicIdx as usize] =
                self.decoded_pictures;
            self.decoded_pictures += 1;

            let frame_info = self.frame_info.as_mut().expect("frame_info to be set when decoding");
            *(frame_info.intra_pic_flag_mut()) = pic_params.intra_pic_flag == 1;
        }

        do_within_context(&self.context, || unsafe {
            cuvidDecodePicture(self.decoder, pic_params)
        });

        1
    }

    /* Return value from HandlePictureDisplay() are interpreted as:
     *  0: fail, >=1: succeeded
     */
    fn handle_picture_display(&mut self, disp_info: *mut CUVIDPARSERDISPINFO) -> i32 {
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

        let frame_info =
            self.frame_info.as_ref().expect("Frame info to be set by `handle_video_sequence()`");

        let working_frame_index = self.decoded_frames;
        self.decoded_frames += 1;

        // NOTE: this block takes negligible time
        if self.decoded_frames > self.frames.len() {
            // Not enough frames in stock
            self.allocated_frames += 1;
            let data = A::alloc(frame_info.width_in_bytes(), frame_info.height_in_rows() as usize);
            self.frames.push_back(OwnedFrame { timestamp: disp_info.timestamp, buffer: data });
        }

        let working_frame = &mut self.frames[working_frame_index];

        // SAFETY: The buffer pointer is only used to copy the luma and chroma data to it below.
        // In particular, it's not used to deallocate or otherwise invalidate the buffer.
        let working_frame_ptr = unsafe { working_frame.buffer.as_mut_ptr() };

        // NOTE: memcpys take about 1ms total here
        // Copy luma plane
        let mut m = CUDA_MEMCPY2D {
            srcMemoryType: CUmemorytype_enum::CU_MEMORYTYPE_DEVICE,
            srcDevice: src_frame,
            srcPitch: src_pitch as usize,
            dstMemoryType: A::memory_type(),
            dstHost: working_frame_ptr as *mut c_void,
            dstDevice: working_frame_ptr as CUdeviceptr,
            dstPitch: working_frame.buffer.pitch(),
            WidthInBytes: frame_info.width_in_bytes(),
            Height: frame_info.luma_height() as usize,
            ..Default::default()
        };
        unsafe {
            cuMemcpy2DAsync_v2(&m, self.stream).into_cuda_result().unwrap();
        }

        // Copy chroma plane
        // NVDEC output has luma height aligned by 2. Adjust chroma offset by aligning height
        m.srcDevice =
            (src_frame + (src_pitch as u64 * ((self.surface_height + 1) & !1))) as CUdeviceptr;
        m.dstHost = ((working_frame_ptr) as CUdeviceptr
            + (m.dstPitch as u64 * frame_info.luma_height() as u64))
            as *mut c_void;
        m.dstDevice = m.dstHost as CUdeviceptr;
        m.Height = frame_info.chroma_height() as usize;
        unsafe {
            cuMemcpy2DAsync_v2(&m, self.stream).into_cuda_result().unwrap();
        }

        if frame_info.num_chroma_planes() == 2 {
            m.srcDevice = (src_frame + (src_pitch as u64 * ((self.surface_height + 1) & !1) * 2))
                as CUdeviceptr;
            m.dstHost = ((working_frame_ptr) as CUdeviceptr
                + (m.dstPitch as u64 * frame_info.luma_height() as u64 * 2))
                as *mut c_void;
            m.dstDevice = m.dstHost as CUdeviceptr;
            m.Height = frame_info.chroma_height() as usize;
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
            cuvidCtxLockCreate(&mut ctx_lock, builder.context.cu_ctx() as *mut ffi::CUctx_st)
                .into_cuda_result()?;
            ctx_lock
        };

        // we create the decoder first with a null parser because the parser needs
        // a reference to the decoder for callbacks, and then create the parser with the reference
        // and then set the parser to the actual instantiated one
        let mut this = Box::new(Self {
            parser: std::ptr::null_mut(),
            context: builder.context,
            codec: builder.codec,
            crop_rect: builder.crop_rect,
            resize_dim: builder.resize_dim,
            max_width: builder.max_width,
            max_height: builder.max_height,
            ctx_lock,
            bitdepth_minus_8: 0,
            chroma_format: ChromaFormat::YUV420,
            video_format: Default::default(),
            decoded_frames: 0,
            decoded_frames_returned: 0,
            allocated_frames: 0,
            stream: std::ptr::null_mut(),
            frames: VecDeque::new(),
            picture_decode_index_mapping: [0; 32],
            decoded_pictures: 0,
            decoder: std::ptr::null_mut(),
            operating_point: 0,
            display_rect: Default::default(),
            surface_height: 0,
            surface_width: 0,
            frame_info: None,
            video_info: String::new(),
        });

        // TODO: handle errors
        let mut params = CUVIDPARSERPARAMS {
            CodecType: builder.codec.into(),
            ulMaxNumDecodeSurfaces: 1,
            ulClockRate: builder.clock_rate,
            ulMaxDisplayDelay: if builder.low_latency { 0 } else { 1 },

            pUserData: &raw mut *this as *mut c_void,
            pfnSequenceCallback: Some(handle_video_sequence_proc::<A>),
            pfnDecodePicture: Some(handle_picture_decode_proc::<A>),
            pfnDisplayPicture: Some(handle_picture_display_proc::<A>),
            pfnGetOperatingPoint: Some(handle_operating_point_proc::<A>),

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

    pub fn decode_one(
        &mut self,
        packet_data: &[u8],
        packet_flags: DecoderPacketFlags,
        packet_timestamp: i64,
    ) -> Result<DecodingOutput<Option<Frame<'_, A>>>, NvDecoderError> {
        self.decode_packet(packet_data, packet_flags, packet_timestamp)?;
        let frame = self.frames.front().map(|raw| raw.from_raw_parts());

        Ok(DecodingOutput { frames: frame, frame_count: 1, frame_info: self.frame_info })
    }

    pub fn decode_many(
        &mut self,
        packet_data: &[u8],
        packet_flags: DecoderPacketFlags,
        packet_timestamp: i64,
    ) -> Result<DecodingOutput<impl Iterator<Item = Frame<'_, A>>>, NvDecoderError> {
        self.decode_packet(packet_data, packet_flags, packet_timestamp)?;
        let frames = self.frames.iter().map(|raw| raw.from_raw_parts());

        Ok(DecodingOutput { frames, frame_count: self.frames.len(), frame_info: self.frame_info })
    }

    fn decode_packet(
        &mut self,
        packet_data: &[u8],
        packet_flags: DecoderPacketFlags,
        packet_timestamp: i64,
    ) -> Result<(), NvDecoderError> {
        self.decoded_frames = 0;
        self.decoded_frames_returned = 0;
        let flags: CUvideopacketflags::Type = packet_flags.into();
        let mut packet = CUVIDSOURCEDATAPACKET {
            flags: (flags as u32 | CUVID_PKT_TIMESTAMP) as c_ulong,
            payload_size: packet_data.len() as u64,
            payload: packet_data.as_ptr(),
            timestamp: packet_timestamp,
        };

        if packet_data.is_empty() {
            packet.flags |= CUVID_PKT_ENDOFSTREAM as c_ulong;
        }

        unsafe {
            cuvidParseVideoData(self.parser, &raw mut packet).into_cuda_result()?;
        }

        self.stream = std::ptr::null_mut();

        Ok(())
    }

    pub fn set_reconfig_params() -> Result<(), NvDecoderError> {
        todo!()
    }
}

impl<A: FrameAllocator> Drop for NvDecoder<A> {
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

        self.frames.drain(..);

        ContextStack::pop().unwrap();
        unsafe {
            let err = cuvidCtxLockDestroy(self.ctx_lock);
            err.into_cuda_result().expect("Failure on nvdecoder ctx lock destroy");
        }
        println!("Session Deinitialization Time: {:?}", session_deinit_start.elapsed());
    }
}
