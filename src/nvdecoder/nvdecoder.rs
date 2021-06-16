use crate::common::{
    CudaResult, CudaVideoChromaFormat, CudaVideoCodec, CudaVideoCreateFlags,
    CudaVideoDeinterlaceMode, CudaVideoSurfaceFormat, Dim, IntoCudaResult, Rect,
};
use nv_video_codec_sys::{
    self, CUstream, CUvideodecoder,
    CUvideopacketflags::{self, CUVID_PKT_ENDOFSTREAM, CUVID_PKT_TIMESTAMP},
    __BindgenBitfieldUnit, cuArray3DCreate_v2,
    cudaVideoCodec_enum::cudaVideoCodec_NV12,
    cudaVideoSurfaceFormat_enum::{cudaVideoSurfaceFormat_NV12, cudaVideoSurfaceFormat_P016},
    cuvidCtxLockCreate, cuvidCtxLockDestroy, cuvidDecodePicture, cuvidDestroyVideoParser,
    cuvidGetDecoderCaps, cuvidParseVideoData, CUVIDDECODECAPS, CUVIDDECODECREATEINFO,
    CUVIDEOFORMAT, CUVIDOPERATINGPOINTINFO, CUVIDPARSERDISPINFO, CUVIDPARSERPARAMS, CUVIDPICPARAMS,
    CUVIDSOURCEDATAPACKET, _CUVIDDECODECAPS,
};
use parking_lot::{Mutex, MutexGuard};
use rustacuda::context::{Context, ContextHandle, ContextStack};
use std::{
    collections::VecDeque, ffi::c_void, mem::MaybeUninit, ops::Deref, os::raw::c_ulong, sync::Arc,
};

use super::{DecoderPacketFlags, NvDecoderError};

pub struct Frame {
    timestamp: i64,
    data: Vec<u8>,
}

pub struct NvDecoderBuilder {
    context: Context,
    use_device_frame: bool,
    codec: CudaVideoCodec,
    low_latency: bool,
    device_frame_pitched: bool,
    crop_rect: Option<Rect>,
    resize_dim: Option<Dim>,
    max_width: u32,
    max_height: u32,
    clock_rate: u32,
}

impl NvDecoderBuilder {
    builder_field_setter!(low_latency: bool);

    builder_field_setter!(device_frame_pitched: bool);

    builder_field_setter_opt!(crop_rect: Rect);

    builder_field_setter_opt!(resize_dim: Dim);

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
            crop_rect: None,
            resize_dim: None,
            max_width: 0,
            max_height: 0,
            clock_rate: 1000,
        }
    }

    pub fn build(self) -> Result<NvDecoder, NvDecoderError> {
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

define_opaque_pointer_type!(CUvideoparser);
define_opaque_pointer_type!(CUvideoctxlock);

pub struct NvDecoder {
    parser: *mut CUvideoparser,
    decoder: *mut CUvideodecoder,
    context: Context,
    use_device_frame: bool,
    codec: CudaVideoCodec,
    chroma_format: CudaVideoChromaFormat,
    output_format: CudaVideoSurfaceFormat,
    video_format: CUVIDEOFORMAT,
    device_frame_pitched: bool,
    crop_rect: Option<Rect>,
    resize_dim: Option<Dim>,
    max_width: u32,
    max_height: u32,
    ctx_lock: *mut CUvideoctxlock,
    video_info: String,
    bitdepth_minus_8: i32,
    bpp: i32,

    n_decoded_frame: usize,
    n_decoded_frame_returned: usize,
    stream: CUstream,
    /// need mutex to cover callbacks
    frames: Arc<Mutex<VecDeque<Frame>>>,
    n_pic_num_in_decode_order: [usize; 32],
    n_decode_pic_cnt: usize,
    n_operating_point: usize,

    /// output dimensions
    width: u32,
    luma_height: u32,
    chroma_height: u32,
    num_chroma_planes: u32,
}

/// decoder refers to an NvDecoder
/// Callback function to be registered for getting a callback when decoding of sequence starts
unsafe extern "C" fn handle_video_sequence_proc(
    decoder: *mut c_void,
    video_format: *mut CUVIDEOFORMAT,
) -> i32 {
    debug_assert!(!decoder.is_null());
    (decoder as *mut NvDecoder).as_mut().unwrap().handle_video_sequence(video_format)
}

/// decoder refers to an NvDecoder
/// Callback function to be registered for getting a callback when a decoded frame is ready to be decoded
unsafe extern "C" fn handle_picture_decode_proc(
    decoder: *mut c_void,
    pic_params: *mut CUVIDPICPARAMS,
) -> i32 {
    debug_assert!(!decoder.is_null());
    (decoder as *mut NvDecoder).as_mut().unwrap().handle_picture_decode(pic_params)
}

/// decoder refers to an NvDecoder
/// Callback function to be registered for getting a callback when a decoded frame is available for display
unsafe extern "C" fn handle_picture_display_proc(
    decoder: *mut c_void,
    disp_info: *mut CUVIDPARSERDISPINFO,
) -> i32 {
    debug_assert!(!decoder.is_null());
    (decoder as *mut NvDecoder).as_mut().unwrap().handle_picture_display(disp_info)
}

/// decoder refers to an NvDecoder
/// Callback function to be registered for getting a callback to get operating point when AV1 SVC sequence header start.
unsafe extern "C" fn handle_operating_point_proc(
    decoder: *mut c_void,
    op_info: *mut CUVIDOPERATINGPOINTINFO,
) -> i32 {
    debug_assert!(!decoder.is_null());
    (decoder as *mut NvDecoder).as_mut().unwrap().handle_operating_point(op_info)
}

fn do_ctxpush_cuvidfunc<F, T>(context: &Context, mut func: F)
where
    F: FnMut() -> T,
    T: IntoCudaResult<()>,
{
    ContextStack::push(context).unwrap();
    func().into_cuda_result().expect("Cuda NVDEC api call failure");
    ContextStack::pop().unwrap();
}

impl NvDecoder {
    // TODO(efyang) : switch these over to result types and just handle the results
    fn handle_video_sequence(&mut self, raw_video_format: *mut CUVIDEOFORMAT) -> i32 {
        let video_format;
        unsafe {
            video_format = *raw_video_format;
        }
        self.video_info = format!("Video Input Information:\t{:?}", raw_video_format);

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
        video_decode_create_info.vidLock = self.ctx_lock as nv_video_codec_sys::CUvideoctxlock;
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

        todo!()
    }

    fn handle_picture_decode(&mut self, pic_params: *mut CUVIDPICPARAMS) -> i32 {
        debug_assert!(!self.decoder.is_null());
        debug_assert!(!pic_params.is_null());
        unsafe {
            self.n_pic_num_in_decode_order[(*pic_params).CurrPicIdx as usize] =
                self.n_decode_pic_cnt;
        }
        self.n_decode_pic_cnt += 1;

        do_ctxpush_cuvidfunc(&self.context, || unsafe {
            cuvidDecodePicture(self.decoder as nv_video_codec_sys::CUvideodecoder, pic_params)
        });

        return 1;
    }

    fn handle_picture_display(&mut self, disp_info: *mut CUVIDPARSERDISPINFO) -> i32 {
        todo!()
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

impl NvDecoder {
    fn new(
        context: Context,
        use_device_frame: bool,
        codec: CudaVideoCodec,
        low_latency: bool,
        device_frame_pitched: bool,
        crop_rect: Option<Rect>,
        resize_dim: Option<Dim>,
        max_width: u32,
        max_height: u32,
        clock_rate: u32,
    ) -> Result<Self, NvDecoderError> {
        let ctx_lock = unsafe {
            let mut ctx_lock: MaybeUninit<*mut CUvideoctxlock> = MaybeUninit::uninit();
            cuvidCtxLockCreate(
                ctx_lock.as_mut_ptr() as *mut nv_video_codec_sys::CUvideoctxlock,
                context.get_inner() as *mut nv_video_codec_sys::CUctx_st,
            )
            .into_cuda_result()?;
            ctx_lock.assume_init()
        };

        // we create the decoder first with a null parser because the parser needs
        // a reference to the decoder for callbacks, and then create the parser with the reference
        // and then set the parser to the actual instantiated one
        let mut this = Self {
            parser: std::ptr::null_mut() as *mut CUvideoparser,
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

        let parser = unsafe {
            let mut parser: MaybeUninit<*mut CUvideoparser> = MaybeUninit::uninit();
            nv_video_codec_sys::cuvidCreateVideoParser(
                parser.as_mut_ptr() as *mut nv_video_codec_sys::CUvideoparser,
                &mut params,
            );
            parser.assume_init()
        };
        this.parser = parser;

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
            cuvidParseVideoData(
                self.parser as nv_video_codec_sys::CUvideoparser,
                &mut packet as *mut CUVIDSOURCEDATAPACKET,
            )
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

    pub fn get_frame<'a>(&'a mut self) -> Option<Box<dyn Deref<Target = Frame> + 'a>> {
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

    pub fn unlock_frame(&mut self, mut frame: Frame) {
        let mut frames_locked = self.frames.lock();
        frame.timestamp = 0;
        frames_locked.push_back(frame);
    }

    pub fn set_reconfig_params() -> Result<(), ()> {
        todo!()
    }
}

impl Drop for NvDecoder {
    fn drop(&mut self) {
        if !self.parser.is_null() {
            unsafe {
                let err = cuvidDestroyVideoParser(self.parser as nv_video_codec_sys::CUvideoparser);
                err.into_cuda_result().expect("Failure on nvdecoder parser destroy");
            }
        }

        unsafe {
            let err = cuvidCtxLockDestroy(self.ctx_lock as nv_video_codec_sys::CUvideoctxlock);
            err.into_cuda_result().expect("Failure on nvdecoder ctx lock destroy");
        }
    }
}
