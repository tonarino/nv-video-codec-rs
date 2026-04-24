use super::{
    nvencoder::NvEncoder, resource_manager::NvEncoderResourceManager, types::BufferFormat,
    NvEncoderResult,
};
use crate::{
    common::{util::ContextStack, IntoCudaResult},
    encoder::nvencoder::{Device, Input, NvEncInputFrame, NvEncoderSettings},
};
use cuda_gl_interop::{CudaSliceMut, Size};
use cudarc::driver::{
    sys::{cuMemcpy2D_v2, CUdeviceptr, CUmemorytype_enum, CUDA_MEMCPY2D},
    CudaContext,
};
use nv_video_codec_sys::{cuMemAllocPitch_v2, cuMemFree_v2, _NV_ENC_DEVICE_TYPE};
use std::{
    ffi::c_void,
    ops::{Deref, DerefMut},
    ptr::null_mut,
    sync::Arc,
};

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

impl NvEncoderCuda {
    pub fn new(context: Arc<CudaContext>, settings: NvEncoderSettings) -> NvEncoderResult<Self> {
        Ok(Self {
            encoder: NvEncoder::new(
                _NV_ENC_DEVICE_TYPE::NV_ENC_DEVICE_TYPE_CUDA,
                context.cu_ctx() as *mut Device,
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
            let resource_ptr = input_frame.ptr();
            if !resource_ptr.is_null() {
                unsafe {
                    cuMemFree_v2(resource_ptr as CUdeviceptr);
                }
            }
        }
        self.input_frames.clear();

        for reference_frame in self.reference_frames.iter() {
            let resource_ptr = reference_frame.ptr();
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

impl<'a> From<&'a mut NvEncInputFrame> for CudaSliceMut<'a> {
    fn from(frame: &'a mut NvEncInputFrame) -> Self {
        let resolution = Size::new(frame.width(), frame.height());
        let buffer = frame.ptr() as *mut c_void;
        let pitch = frame.pitch() as usize;

        // SAFETY: Input frames are valid for writing until we call `encode_frame`.
        unsafe { CudaSliceMut::new(buffer, pitch, resolution) }
    }
}

pub struct NvEncoderCudaResourceManager {}

impl NvEncoderResourceManager for NvEncoderCudaResourceManager {
    type InputResource = CUdeviceptr;
    // TODO(mbernat): Use a different type here, this one is only valid for `BufferFormat::ABGR`.
    type InputResourceRef<'a> = CudaSliceMut<'a>;
    type ResourceContext = Arc<CudaContext>;

    fn allocate_input_buffers(
        encoder: &mut NvEncoder<Self>,
        num_input_buffers: u32,
    ) -> NvEncoderResult<()> {
        if !encoder.is_hw_encoder_initialized() {
            // TODO(efyang): make this an error
            panic!("Encoder device not initialized");
        }
        let num_buffers = if encoder.motion_estimation_only { 2 } else { 1 };
        let pixel_format = encoder.get_pixel_format();

        // Pitch shared by all the frame allocations.
        let mut pitch = 0;

        for buffer_index in 0..num_buffers {
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
                let mut device_frame_pitch = 0;
                let element_size_in_bytes = 16;

                unsafe {
                    cuMemAllocPitch_v2(
                        &raw mut device_frame_ptr,
                        &raw mut device_frame_pitch,
                        pixel_format.get_width_in_bytes(encoder.get_max_encode_width())? as usize,
                        (encoder.get_max_encode_height() + chroma_height) as usize,
                        element_size_in_bytes,
                    )
                    .into_cuda_result()
                    .unwrap();
                }

                if pitch == 0 {
                    pitch = device_frame_pitch;
                } else {
                    assert_eq!(pitch, device_frame_pitch);
                }

                input_frames.push(device_frame_ptr);
            }

            ContextStack::pop().unwrap();

            // TODO: do not leak but store until `release_input_buffers()` is called.
            let input_frame_ptrs =
                input_frames.leak().iter_mut().map(|input_frame| *input_frame as *mut Input);

            let is_reference_frame = buffer_index == 1;

            encoder.register_input_resources(
                    input_frame_ptrs,
                    nv_video_codec_sys::NV_ENC_INPUT_RESOURCE_TYPE::NV_ENC_INPUT_RESOURCE_TYPE_CUDADEVICEPTR,
                    encoder.get_max_encode_width(),
                    encoder.get_max_encode_height(),
                    pitch as u32,
                    pixel_format,
                    is_reference_frame
                )?;
        }
        Ok(())
    }

    fn release_input_buffers(encoder: &mut NvEncoder<Self>) -> NvEncoderResult<()> {
        encoder.release_cuda_resources()
    }
}

pub fn upload_nv12_data_to_cuda_resource(
    data: &[u8],
    resource: CudaSliceMut<'_>,
    width: u32,
    height: u32,
) {
    // NV12 data layout:
    // - 8-bit width x height luma plane
    // - 2 8-bit width/2 x height/2 chroma planes (each chroma value is shared by a 2x2 pixel block)
    //   - when the chroma planes are interleaved, this results in 1 8-bit width x height/2 plane
    let width = width as usize;
    let luma_height = height as usize;
    let chroma_height = height as usize / 2;

    let m = CUDA_MEMCPY2D {
        srcMemoryType: CUmemorytype_enum::CU_MEMORYTYPE_HOST,
        srcHost: &raw const *data as *const c_void,
        srcPitch: width,
        dstMemoryType: CUmemorytype_enum::CU_MEMORYTYPE_DEVICE,
        dstDevice: resource.buffer() as CUdeviceptr,
        dstPitch: resource.pitch(),
        WidthInBytes: width,
        Height: luma_height + chroma_height,
        srcXInBytes: 0,
        srcY: 0,
        srcDevice: 0,
        srcArray: null_mut(),
        dstXInBytes: 0,
        dstY: 0,
        dstHost: null_mut(),
        dstArray: null_mut(),
    };

    unsafe {
        cuMemcpy2D_v2(&raw const m);
    }
}
