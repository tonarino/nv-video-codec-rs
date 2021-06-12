use crate::common::{CudaVideoCodec, Dim, IntoCudaResult, Rect};
use nv_video_codec_sys::{
    self, __BindgenBitfieldUnit, cuvidCtxLockCreate, cuvidCtxLockDestroy, cuvidDestroyVideoParser,
    CUVIDPARSERPARAMS,
};
use rustacuda::context::{Context, ContextHandle};
use std::mem::MaybeUninit;

use super::NvDecoderError;

pub struct Frame {}

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

    builder_field_setter!(low_latency: bool);
    builder_field_setter!(device_frame_pitched: bool);
    builder_field_setter_opt!(crop_rect: Rect);
    builder_field_setter_opt!(resize_dim: Dim);
    builder_field_setter!(max_width: u32);
    builder_field_setter!(max_height: u32);
    builder_field_setter!(clock_rate: u32);

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

#[repr(C)]
pub struct CUvideoparser {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
pub struct CUvideoctxlock {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

pub struct NvDecoder {
    parser: *mut CUvideoparser,
    context: Context,
    use_device_frame: bool,
    codec: CudaVideoCodec,
    device_frame_pitched: bool,
    crop_rect: Option<Rect>,
    resize_dim: Option<Dim>,
    max_width: u32,
    max_height: u32,
    ctx_lock: *mut CUvideoctxlock,
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
        // TODO: handle errors
        let mut params = CUVIDPARSERPARAMS {
            CodecType: codec.into(),
            ulMaxNumDecodeSurfaces: 1,
            ulClockRate: clock_rate,
            ulMaxDisplayDelay: if low_latency { 0 } else { 1 },

            // TODO: callbacks
            pUserData: std::ptr::null_mut(),
            pfnSequenceCallback: None,
            pfnDecodePicture: None,
            pfnDisplayPicture: None,
            pfnGetOperatingPoint: None,

            // TODO: other stuff not mentioned: sane defaults?
            // most likely broken tbh
            _bitfield_1: __BindgenBitfieldUnit::new([0; 4]),
            _bitfield_align_1: [0; 0],
            ulErrorThreshold: 0,
            uReserved1: [0; 4],
            pvReserved2: [std::ptr::null_mut(); 6],
            pExtVideoInfo: std::ptr::null_mut(),
        };

        let ctx_lock = unsafe {
            let mut ctx_lock: MaybeUninit<*mut CUvideoctxlock> = MaybeUninit::uninit();
            cuvidCtxLockCreate(
                ctx_lock.as_mut_ptr() as *mut nv_video_codec_sys::CUvideoctxlock,
                context.get_inner() as *mut nv_video_codec_sys::CUctx_st,
            )
            .into_cuda_result()?;
            ctx_lock.assume_init()
        };

        let parser = unsafe {
            let mut parser: MaybeUninit<*mut CUvideoparser> = MaybeUninit::uninit();
            nv_video_codec_sys::cuvidCreateVideoParser(
                parser.as_mut_ptr() as *mut nv_video_codec_sys::CUvideoparser,
                &mut params,
            );
            parser.assume_init()
        };

        Ok(Self {
            parser,
            context,
            use_device_frame,
            codec,
            device_frame_pitched,
            crop_rect,
            resize_dim,
            max_width,
            max_height,
            ctx_lock,
        })
    }

    /// Returns the number of frames decoded
    ///
    /// # Arguments
    /// * arg
    pub fn decode() -> usize {
        todo!()
    }

    pub fn get_frame() -> Frame {
        todo!()
    }

    pub fn get_locked_frame() -> Frame {
        todo!()
    }

    pub fn unlock_frame(frame: &mut Frame) {
        todo!()
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
                err.into_cuda_result()
                    .expect("Failure on nvdecoder parser destroy");
            }
        }

        unsafe {
            let err = cuvidCtxLockDestroy(self.ctx_lock as nv_video_codec_sys::CUvideoctxlock);
            err.into_cuda_result()
                .expect("Failure on nvdecoder ctx lock destroy");
        }
    }
}
