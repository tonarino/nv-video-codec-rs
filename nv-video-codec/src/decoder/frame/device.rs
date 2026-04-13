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

    fn alloc(frame_info: &FrameInfo) -> Self::Buffer {
        let mut ptr: CUdeviceptr = 0;
        let pitch = frame_info.width_in_bytes();
        let size = frame_info.frame_size() as usize;

        unsafe {
            cuMemAlloc_v2(&mut ptr, size).into_cuda_result().unwrap();
        }

        RawDeviceBuffer { ptr: ptr as *mut u8, pitch, size }
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
    type Buffer = RawDeviceBuffer;

    fn alloc(frame_info: &FrameInfo) -> Self::Buffer {
        let mut ptr: CUdeviceptr = 0;
        let mut pitch = 0;
        let size = frame_info.frame_size() as usize;

        // TODO(efyang): this should be a specialized type, pitched allocation is not like a normal array
        // refer to https://stackoverflow.com/questions/16119943/how-and-when-should-i-use-pitched-pointer-with-the-cuda-api
        unsafe {
            cuMemAllocPitch_v2(
                &raw mut ptr,
                &raw mut pitch,
                frame_info.width_in_bytes(),
                frame_info.height_in_rows() as usize,
                16,
            )
            .into_cuda_result()
            .unwrap();
        }

        RawDeviceBuffer { ptr: ptr as *mut u8, pitch, size }
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

    fn pitch(&self) -> usize {
        self.pitch
    }

    unsafe fn as_slice<'a>(&'a self) -> Self::Slice<'a> {
        // SAFETY: `as_slice` caller guarantees the device slice is valid for `'a`.
        unsafe { self.as_device_slice() }
    }
}

pub struct RawDeviceBuffer {
    pub ptr: *mut u8,
    pub pitch: usize,
    pub size: usize,
}

impl RawDeviceBuffer {
    /// # Safety
    ///
    /// Device memory backed by `self` has to be valid for `'a`.
    unsafe fn as_device_slice<'a>(&'a self) -> DeviceSlice<'a> {
        DeviceSlice {
            ptr: self.ptr,
            pitch: self.pitch,
            _size: self.size,
            _phantom_data: PhantomData,
        }
    }
}

/// A slice of GPU device memory guaranteed to be valid for `'a`.
pub struct DeviceSlice<'a> {
    ptr: *mut u8,
    pitch: usize,
    _size: usize,
    _phantom_data: PhantomData<&'a ()>,
}

impl<'a> DeviceSlice<'a> {
    pub fn ptr(&self) -> *const u8 {
        self.ptr as *const u8
    }

    pub fn pitch(&self) -> usize {
        self.pitch
    }
}
