use crate::{
    common::IntoCudaResult as _,
    decoder::frame::{info::FrameInfo, FrameAllocator, RawBuffer},
};
use nv_video_codec_sys::{
    cuMemAllocPitch_v2, cuMemAlloc_v2, cuMemFree_v2, CUdeviceptr, CUmemorytype, CUmemorytype_enum,
};
use std::marker::PhantomData;

pub struct DeviceFrameAllocator;

impl FrameAllocator for DeviceFrameAllocator {
    type Buffer = RawDeviceBuffer;

    fn alloc(frame_info: &FrameInfo, _device_frame_pitch: &mut usize) -> Self::Buffer {
        let mut frame_data_device_ptr: CUdeviceptr = 0;
        let len = frame_info.frame_size() as usize;

        unsafe {
            cuMemAlloc_v2(&mut frame_data_device_ptr, len).into_cuda_result().unwrap();
        }

        RawDeviceBuffer { ptr: frame_data_device_ptr as *mut u8, len }
    }

    fn free(buffer: &mut Self::Buffer) {
        unsafe {
            cuMemFree_v2(buffer.ptr as CUdeviceptr)
                .into_cuda_result()
                .expect("Failure on nvdecoder frame free");
        }
    }

    fn memory_type() -> CUmemorytype {
        CUmemorytype_enum::CU_MEMORYTYPE_DEVICE
    }
}

pub struct PitchedDeviceFrameAllocator;

impl FrameAllocator for PitchedDeviceFrameAllocator {
    // TODO(mbernat): Check if we need a different type here.
    type Buffer = RawDeviceBuffer;

    fn alloc(frame_info: &FrameInfo, device_frame_pitch: &mut usize) -> Self::Buffer {
        let mut frame_data_device_ptr: CUdeviceptr = 0;
        let len = frame_info.frame_size() as usize;

        // TODO(efyang): this should be a specialized type, pitched allocation is not like a normal array
        // refer to https://stackoverflow.com/questions/16119943/how-and-when-should-i-use-pitched-pointer-with-the-cuda-api
        unsafe {
            cuMemAllocPitch_v2(
                &mut frame_data_device_ptr,
                device_frame_pitch,
                (frame_info.width() * frame_info.bpp() as u32) as usize,
                (frame_info.luma_height()
                    + frame_info.chroma_height() * frame_info.num_chroma_planes())
                    as usize,
                16,
            )
            .into_cuda_result()
            .unwrap();
        }

        RawDeviceBuffer { ptr: frame_data_device_ptr as *mut u8, len }
    }

    fn free(buffer: &mut Self::Buffer) {
        // TODO(mbernat): Make sure this is valid for pitched device frames.
        unsafe {
            cuMemFree_v2(buffer.ptr as CUdeviceptr)
                .into_cuda_result()
                .expect("Failure on nvdecoder frame free");
        }
    }

    fn memory_type() -> CUmemorytype {
        CUmemorytype_enum::CU_MEMORYTYPE_DEVICE
    }
}

impl RawBuffer for RawDeviceBuffer {
    type Slice<'a> = DeviceSlice<'a>;

    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr
    }

    unsafe fn as_slice<'a>(&'a self) -> Self::Slice<'a> {
        // SAFETY: `as_slice` caller guarantees the device slice is valid for `'a`.
        unsafe { self.as_device_slice() }
    }

    fn from_slice<'a>(slice: Self::Slice<'a>) -> Self {
        slice.into_raw_device_buffer()
    }
}

pub struct RawDeviceBuffer {
    pub ptr: *mut u8,
    pub len: usize,
}

impl RawDeviceBuffer {
    /// # Safety
    ///
    /// Device memory backed by `self` has to be valid for `'a`.
    unsafe fn as_device_slice<'a>(&'a self) -> DeviceSlice<'a> {
        DeviceSlice { ptr: self.ptr, len: self.len, _phantom_data: PhantomData }
    }
}

/// A slice of GPU device memory guaranteed to be valid for `'a`.
pub struct DeviceSlice<'a> {
    ptr: *mut u8,
    len: usize,
    _phantom_data: PhantomData<&'a ()>,
}

impl<'a> DeviceSlice<'a> {
    fn into_raw_device_buffer(self) -> RawDeviceBuffer {
        RawDeviceBuffer { ptr: self.ptr, len: self.len }
    }

    pub fn ptr(&self) -> *const u8 {
        self.ptr as *const u8
    }
}
