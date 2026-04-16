use super::{
    nvencoder::NvEncoder, resource_manager::NvEncoderResourceManager, types::BufferFormat,
    NvEncoderResult,
};
use crate::encoder::nvencoder::{Input, NvEncInputFrame, NvEncoderSettings};
use nv_video_codec_sys::{
    _NV_ENC_DEVICE_TYPE, _NV_ENC_INPUT_RESOURCE_OPENGL_TEX, NV_ENC_INPUT_RESOURCE_OPENGL_TEX,
};
use std::ops::{Deref, DerefMut};

pub struct NvEncoderGL {
    encoder: NvEncoder<NvEncoderGLResourceManager>,
}

impl Deref for NvEncoderGL {
    type Target = NvEncoder<NvEncoderGLResourceManager>;

    fn deref(&self) -> &Self::Target {
        &self.encoder
    }
}

impl DerefMut for NvEncoderGL {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.encoder
    }
}

impl NvEncoderGL {
    pub fn new(settings: NvEncoderSettings) -> NvEncoderResult<Self> {
        // TODO: remove this unwrap
        Ok(Self {
            encoder: NvEncoder::new(
                _NV_ENC_DEVICE_TYPE::NV_ENC_DEVICE_TYPE_OPENGL,
                std::ptr::null_mut(),
                (),
                settings,
            )?,
        })
    }
}

impl NvEncoder<NvEncoderGLResourceManager> {
    fn release_gl_resources(&mut self) -> NvEncoderResult<()> {
        if self.encoder_handle.is_null() {
            return Ok(());
        }
        self.unregister_input_resources()?;

        for input_frame in self.input_frames.iter() {
            let resource_ptr = input_frame.ptr() as *mut NV_ENC_INPUT_RESOURCE_OPENGL_TEX;
            if !resource_ptr.is_null() {
                unsafe { gl::DeleteTextures(1, &(*resource_ptr).texture) }
                // TODO(efyang) check for possible memory leak here (vs original delete)
            }
        }
        self.input_frames.clear();

        for reference_frame in self.reference_frames.iter() {
            let resource_ptr = reference_frame.ptr() as *mut NV_ENC_INPUT_RESOURCE_OPENGL_TEX;
            if !resource_ptr.is_null() {
                unsafe { gl::DeleteTextures(1, &(*resource_ptr).texture) }
            }
        }
        self.reference_frames.clear();
        Ok(())
    }
}

// TODO: wrap the user facing type
impl<'a> From<&'a mut NvEncInputFrame> for &'a mut NV_ENC_INPUT_RESOURCE_OPENGL_TEX {
    fn from(frame: &'a mut NvEncInputFrame) -> Self {
        let resource_ptr = frame.ptr() as *mut NV_ENC_INPUT_RESOURCE_OPENGL_TEX;

        // SAFETY: The input resources are valid for 'a.
        unsafe { resource_ptr.as_mut() }.expect("Input resource to exist")
    }
}

pub struct NvEncoderGLResourceManager {}

impl NvEncoderResourceManager for NvEncoderGLResourceManager {
    type InputResource = NV_ENC_INPUT_RESOURCE_OPENGL_TEX;
    type InputResourceRef<'a> = &'a mut NV_ENC_INPUT_RESOURCE_OPENGL_TEX;
    type ResourceContext = ();

    fn allocate_input_buffers(
        encoder: &mut NvEncoder<Self>,
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

                let resource = NV_ENC_INPUT_RESOURCE_OPENGL_TEX {
                    texture: tex,
                    target: gl::TEXTURE_RECTANGLE,
                };

                input_frames.push(resource);
            }

            // TODO: do not leak but store until `release_input_buffers()` is called.
            let input_frame_ptrs = input_frames
                .leak()
                .iter_mut()
                .map(|input_frame| input_frame as *mut _ as *mut Input);

            encoder.register_input_resources(
                    input_frame_ptrs,
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

    fn release_input_buffers(encoder: &mut NvEncoder<Self>) -> NvEncoderResult<()> {
        encoder.release_gl_resources()
    }
}

pub fn upload_nv12_data_to_texture_resource(
    data: &[u8],
    // TODO: wrap FFI type
    resource: &mut _NV_ENC_INPUT_RESOURCE_OPENGL_TEX,
    width: u32,
    height: u32,
) {
    // NV12 data layout:
    // - 8-bit width x height luma plane
    // - 2 2-bit width x height chroma planes (each chroma value is shared by a 2x2 pixel block)
    //   - when the chroma planes are interleaved, this results in 1 4-bit width x height / 2 plane
    let width = width as i32;
    let luma_height = height as i32;
    let chroma_height = height as i32 / 2;

    unsafe {
        gl::BindTexture(resource.target, resource.texture);
        gl::TexSubImage2D(
            resource.target,
            0,                           // level
            0,                           // x offset
            0,                           // y offset
            width,                       // width
            luma_height + chroma_height, // height
            gl::RED,                     // format (single-channel)
            gl::UNSIGNED_BYTE,           // type
            data.as_ptr() as *const _,   // data
        );
        gl::BindTexture(resource.target, 0);
    }
}
