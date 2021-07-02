use core::num;

use super::{nvencoder::NvEncoder, types::BufferFormat, NvEncoderError, NvEncoderResourceManager};
use nv_video_codec_sys::{
    NV_ENC_BUFFER_FORMAT, NV_ENC_INPUT_RESOURCE_OPENGL_TEX, _NV_ENC_DEVICE_TYPE,
};

pub struct NvEncoderGL {
    encoder: NvEncoder,
}

impl NvEncoderGL {
    pub fn new(
        width: u32,
        height: u32,
        buffer_format: BufferFormat,
        extra_output_delay: u32,
        motion_extimation_only: bool,
    ) -> Result<Self, NvEncoderError> {
        // TODO: remove this unwrap
        Ok(Self {
            encoder: NvEncoder::new(
                _NV_ENC_DEVICE_TYPE::NV_ENC_DEVICE_TYPE_OPENGL,
                std::ptr::null_mut(),
                width,
                height,
                buffer_format,
                extra_output_delay,
                motion_extimation_only,
                false,
            )
            .unwrap(),
        })
    }
}

impl NvEncoderGL {
    fn release_gl_resources(&mut self) -> Result<(), NvEncoderError> {
        if self.encoder.encoder_handle.is_null() {
            return Ok(());
        }
        self.encoder.unregister_input_resources();

        for input_frame in self.encoder.input_frames.iter() {
            let pResource = input_frame.input_ptr as *mut NV_ENC_INPUT_RESOURCE_OPENGL_TEX;
            if !pResource.is_null() {
                // pResource.texture
            }
        }
        todo!()
    }
}

pub struct NvEncoderGLResourceManager {}

impl NvEncoderResourceManager for NvEncoderGLResourceManager {
    fn allocate_input_buffers(
        encoder: &mut NvEncoder,
        num_input_buffers: u32,
    ) -> Result<(), NvEncoderError> {
        if !encoder.is_hw_encoder_initialized() {
            // TODO(efyang): make this an error
            panic!("Encoder device not initialized");
        }
        let num_count = if encoder.motion_estimation_only { 2 } else { 1 };
        for count in 0..num_count {
            let mut tex = 0;
            unsafe {
                gl::GenTextures(1, &mut tex);
                gl::BindTexture(gl::TEXTURE_RECTANGLE, tex);
            }
        }
        todo!()
    }

    fn release_input_buffers(encoder: &mut NvEncoder) -> Result<(), NvEncoderError> {
        todo!()
    }
}

impl Drop for NvEncoderGLResourceManager {
    fn drop(&mut self) {}
}

impl Drop for NvEncoderGL {
    fn drop(&mut self) {}
}
