use super::{
    nvencoderbase::NvEncoderBase, resource_manager::NvEncoderResourceManager, types::BufferFormat,
    NvEncoder, NvEncoderExt, NvEncoderGLBuilder, NvEncoderResult,
};
use nv_video_codec_sys::{
    _NV_ENC_DEVICE_TYPE, NV_ENC_INPUT_RESOURCE_OPENGL_TEX, NV_ENC_PIC_PARAMS,
};

pub struct NvEncoderGL {
    encoder: NvEncoderBase<NvEncoderGLResourceManager>,
}

impl_nvencoder_wrapper_type!(NvEncoderGL, NvEncoderGLResourceManager);

impl NvEncoderExt for NvEncoderGL {
    fn encode_frame_from_data(
        &mut self,
        data: &[u8],
        width: u32,
        height: u32,
        pic_params: Option<NV_ENC_PIC_PARAMS>,
        output_packet_buffer: &mut Vec<&[u8]>,
    ) -> NvEncoderResult<()> {
        let encoder_input_frame = self.get_next_input_frame();
        let resource = encoder_input_frame.input_ptr_as_gltex();
        // TODO: remove these hacks
        unsafe {
            gl::BindTexture((*resource).target, (*resource).texture);
            gl::TexSubImage2D(
                (*resource).target,
                0,                         // level
                0,                         // x offset
                0,                         // y offset
                width as i32,              // width
                (height * 3 / 2) as i32,   // height
                gl::RED,                   // format (single-channel)
                gl::UNSIGNED_BYTE,         // type
                data.as_ptr() as *const _, // data
            );
            gl::BindTexture((*resource).target, 0);
        }

        self.encode_frame(output_packet_buffer, pic_params)
    }
}

impl NvEncoderGL {
    pub fn builder(width: u32, height: u32, buffer_format: BufferFormat) -> NvEncoderGLBuilder {
        NvEncoderGLBuilder::new(width, height, buffer_format)
    }

    pub fn new(
        width: u32,
        height: u32,
        buffer_format: BufferFormat,
        extra_output_delay: u32,
        motion_extimation_only: bool,
    ) -> NvEncoderResult<Self> {
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
    fn release_gl_resources(&mut self) -> NvEncoderResult<()> {
        if self.encoder_handle.is_null() {
            return Ok(());
        }
        self.unregister_input_resources()?;

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
    type InputResource = NV_ENC_INPUT_RESOURCE_OPENGL_TEX;

    fn allocate_input_buffers(
        encoder: &mut NvEncoderBase<Self>,
        num_input_buffers: u32,
    ) -> NvEncoderResult<()> {
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

                let resource = Box::new(NV_ENC_INPUT_RESOURCE_OPENGL_TEX {
                    texture: tex,
                    target: gl::TEXTURE_RECTANGLE,
                });

                input_frames.push(resource);
            }

            encoder.register_input_resources(
                    // TODO: do not leak but store until `release_input_buffers()` is called.
                    input_frames.leak(),
                    nv_video_codec_sys::NV_ENC_INPUT_RESOURCE_TYPE::NV_ENC_INPUT_RESOURCE_TYPE_OPENGL_TEX,
                    encoder.get_max_encode_width(),
                    encoder.get_max_encode_height(),
                    pixel_format.get_width_in_bytes(encoder.get_max_encode_width())?,
                    pixel_format,
                    count == 1
                )?;
        }
        Ok(())
    }

    fn release_input_buffers(encoder: &mut NvEncoderBase<Self>) -> NvEncoderResult<()> {
        encoder.release_gl_resources()
    }
}
