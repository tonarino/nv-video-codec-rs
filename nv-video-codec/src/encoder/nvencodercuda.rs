use super::{
    nvencoder::NvEncoder, resource_manager::NvEncoderResourceManager, types::BufferFormat,
    NvEncoderExt, NvEncoderResult,
};
use crate::{
    common::IntoCudaResult,
    encoder::{
        nvencoder::{Device, Input, NvEncoderSettings},
        EncodePicFlags,
    },
};
use nv_video_codec_sys::{cuMemAllocPitch_v2, cuMemFree_v2, CUdeviceptr, _NV_ENC_DEVICE_TYPE};
use rustacuda::{
    context::{ContextHandle as _, ContextStack},
    prelude::Context,
};
use std::ops::{Deref, DerefMut};

pub struct NvEncoderCuda {
    encoder: NvEncoder<NvEncoderCudaResourceManager>,
}

impl Deref for NvEncoderCuda {
    type Target = NvEncoder<NvEncoderCudaResourceManager>;

    fn deref(&self) -> &Self::Target {
        &self.encoder
    }
}

impl DerefMut for NvEncoderCuda {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.encoder
    }
}

impl NvEncoderExt for NvEncoderCuda {
    fn encode_frame_from_data(
        &mut self,
        _data: &[u8],
        _width: u32,
        _height: u32,
        pic_flags: EncodePicFlags,
        output_packet_buffer: &mut Vec<&[u8]>,
    ) -> NvEncoderResult<()> {
        let _resource = self.get_next_input_resource();

        // TODO: Copy data to resource

        self.encode_frame(output_packet_buffer, pic_flags)
    }
}

impl NvEncoderCuda {
    pub fn new(context: Context, settings: NvEncoderSettings) -> NvEncoderResult<Self> {
        Ok(Self {
            encoder: NvEncoder::new(
                _NV_ENC_DEVICE_TYPE::NV_ENC_DEVICE_TYPE_CUDA,
                context.get_inner() as *mut Device,
                context,
                settings,
            )?,
        })
    }
}

impl NvEncoder<NvEncoderCudaResourceManager> {
    fn release_cuda_resources(&mut self) -> NvEncoderResult<()> {
        if self.encoder_handle.is_null() {
            return Ok(());
        }

        self.unregister_input_resources()?;

        ContextStack::push(&self.resource_context).unwrap();

        for input_frame in self.input_frames.iter() {
            let resource_ptr = input_frame.input_ptr;
            if !resource_ptr.is_null() {
                unsafe {
                    cuMemFree_v2(resource_ptr as CUdeviceptr);
                }
            }
        }
        self.input_frames.clear();

        for reference_frame in self.reference_frames.iter() {
            let resource_ptr = reference_frame.input_ptr;
            if !resource_ptr.is_null() {
                unsafe {
                    cuMemFree_v2(resource_ptr as CUdeviceptr);
                }
            }
        }
        self.reference_frames.clear();

        ContextStack::pop().unwrap();

        Ok(())
    }
}

pub struct NvEncoderCudaResourceManager {}

impl NvEncoderResourceManager for NvEncoderCudaResourceManager {
    type InputResource = CUdeviceptr;
    type ResourceContext = Context;

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

        // NvEncoderCuda stores the pitch as a class member. Should we?
        let mut device_frame_pitch = 0;

        for count in 0..num_count {
            ContextStack::push(&encoder.resource_context).unwrap();

            let mut input_frames = Vec::new();
            for _ in 0..num_input_buffers {
                let chroma_height =
                    if matches!(pixel_format, BufferFormat::YV12 | BufferFormat::IYUV) {
                        pixel_format.get_num_chroma_planes()?
                            * pixel_format.get_chroma_height(encoder.get_max_encode_height())?
                    } else {
                        pixel_format.get_chroma_height(encoder.get_max_encode_height())?
                    };

                let mut device_frame_ptr: CUdeviceptr = 0;

                unsafe {
                    cuMemAllocPitch_v2(
                        &raw mut device_frame_ptr,
                        &raw mut device_frame_pitch,
                        pixel_format.get_width_in_bytes(encoder.get_max_encode_width())? as usize,
                        (encoder.get_max_encode_height() + chroma_height) as usize,
                        16,
                    )
                    .into_cuda_result()
                    .unwrap();
                }

                input_frames.push(device_frame_ptr);
            }

            ContextStack::pop().unwrap();

            // TODO: do not leak but store until `release_input_buffers()` is called.
            let input_frame_ptrs =
                input_frames.leak().iter_mut().map(|input_frame| *input_frame as *mut Input);

            encoder.register_input_resources(
                    input_frame_ptrs,
                    nv_video_codec_sys::NV_ENC_INPUT_RESOURCE_TYPE::NV_ENC_INPUT_RESOURCE_TYPE_CUDADEVICEPTR,
                    encoder.get_max_encode_width(),
                    encoder.get_max_encode_height(),
                    device_frame_pitch as u32,
                    pixel_format,
                    count == 1
                )?;
        }
        Ok(())
    }

    fn release_input_buffers(encoder: &mut NvEncoder<Self>) -> NvEncoderResult<()> {
        encoder.release_cuda_resources()
    }
}
