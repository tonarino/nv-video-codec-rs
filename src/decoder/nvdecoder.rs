use crate::decoder::cuda_video_codec::CudaVideoCodec;
use rustacuda::context::Context;
use rustacuda::context::ContextHandle;

pub struct Frame {}
pub struct NvDecoder {
    inner: nv_video_codec_sys::NvDecoder,
}

impl NvDecoder {
    pub fn new(context: &Context, use_device_frame: bool, codec: CudaVideoCodec) -> Self {
        Self {
            inner: unsafe {
                nv_video_codec_sys::NvDecoder::new(
                    context.get_inner() as nv_video_codec_sys::CUcontext,
                    use_device_frame,
                    codec.into(),
                    false,
                    false,
                    std::ptr::null(),
                    std::ptr::null(),
                    0,
                    0,
                    1000,
                )
            },
        }
    }

    pub fn get_width(&self) -> usize {
        self.inner.GetWidth() as usize
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
        unsafe {
            self.inner.destruct();
        }
    }
}
