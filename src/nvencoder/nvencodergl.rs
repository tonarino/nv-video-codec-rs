use core::num;

use super::{
    nvencoderbase::NvEncoderBase, resource_manager::NvEncoderResourceManager, types::BufferFormat,
    NvEncoderError,
};
use nv_video_codec_sys::{
    NV_ENC_BUFFER_FORMAT, NV_ENC_INPUT_RESOURCE_OPENGL_TEX, _NV_ENC_DEVICE_TYPE,
};

pub struct NvEncoderGL {
    encoder: NvEncoderBase<NvEncoderGLResourceManager>,
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
            encoder: NvEncoderBase::new(
                _NV_ENC_DEVICE_TYPE::NV_ENC_DEVICE_TYPE_OPENGL,
                std::ptr::null_mut(),
                width,
                height,
                buffer_format,
                extra_output_delay,
                motion_extimation_only,
                false,
            )?,
        })
    }
}

impl NvEncoderBase<NvEncoderGLResourceManager> {
    fn release_gl_resources(&mut self) -> Result<(), NvEncoderError> {
        if self.encoder_handle.is_null() {
            return Ok(());
        }
        self.unregister_input_resources();

        for input_frame in self.input_frames.iter() {
            let resource_ptr = input_frame.input_ptr as *mut NV_ENC_INPUT_RESOURCE_OPENGL_TEX;
            if !resource_ptr.is_null() {
                unsafe { gl::DeleteTextures(1, &(*resource_ptr).texture) }
                // TODO(efyang) check for possible memory leak here (vs original delete)
            }
        }
        self.input_frames.clear();

        for reference_frame in self.reference_frames.iter() {
            let resource_ptr = reference_frame.input_ptr as *mut NV_ENC_INPUT_RESOURCE_OPENGL_TEX;
            if !resource_ptr.is_null() {
                unsafe { gl::DeleteTextures(1, &(*resource_ptr).texture) }
            }
        }
        self.reference_frames.clear();
        Ok(())
    }
}

pub(super) struct NvEncoderGLResourceManager {}

impl NvEncoderResourceManager for NvEncoderGLResourceManager {
    fn allocate_input_buffers(
        encoder: &mut NvEncoderBase<Self>,
        num_input_buffers: u32,
    ) -> Result<(), NvEncoderError> {
        if !encoder.is_hw_encoder_initialized() {
            // TODO(efyang): make this an error
            panic!("Encoder device not initialized");
        }
        let num_count = if encoder.motion_estimation_only { 2 } else { 1 };
        let pixel_format = encoder.get_pixel_format();
        for count in 0..num_count {
            let mut input_frames = Vec::new();
            for _ in 0..num_input_buffers {
                let mut tex = 0;
                unsafe {
                    gl::GenTextures(1, &mut tex);
                    gl::BindTexture(gl::TEXTURE_RECTANGLE, tex);
                }

                let chroma_height =
                    if matches!(pixel_format, BufferFormat::YV12 | BufferFormat::IYUV) {
                        pixel_format.get_num_chroma_planes()?
                            * pixel_format.get_chroma_height(encoder.get_max_encode_height())?
                    } else {
                        pixel_format.get_chroma_height(encoder.get_max_encode_height())?
                    };

                unsafe {
                    gl::TexImage2D(
                        gl::TEXTURE_RECTANGLE,
                        0,
                        gl::R8 as i32,
                        (pixel_format.get_width_in_bytes(encoder.get_max_encode_width())?) as i32,
                        (encoder.get_max_encode_height() + chroma_height) as i32,
                        0,
                        gl::RED,
                        gl::UNSIGNED_BYTE,
                        std::ptr::null_mut(),
                    );
                    gl::BindTexture(gl::TEXTURE_RECTANGLE, 0);
                }

                let resource = NV_ENC_INPUT_RESOURCE_OPENGL_TEX {
                    texture: tex,
                    target: gl::TEXTURE_RECTANGLE,
                };

                input_frames.push(resource);
            }

            encoder.register_input_resources(
                &input_frames,
                nv_video_codec_sys::NV_ENC_INPUT_RESOURCE_TYPE::NV_ENC_INPUT_RESOURCE_TYPE_OPENGL_TEX,
                encoder.get_max_encode_width(),
                encoder.get_max_encode_width(),
                pixel_format.get_width_in_bytes(encoder.get_max_encode_width())?,
                pixel_format,
                count == 1
            );
        }
        Ok(())
    }

    fn release_input_buffers(encoder: &mut NvEncoderBase<Self>) -> Result<(), NvEncoderError> {
        encoder.release_gl_resources()
    }
}
