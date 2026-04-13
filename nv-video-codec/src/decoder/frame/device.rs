use crate::{
    common::IntoCudaResult as _,
    decoder::frame::{Buffer, FrameAllocator},
};
use nv_video_codec_sys::{
    cuMemAllocPitch_v2, cuMemAlloc_v2, cuMemFree_v2, CUdeviceptr, CUmemorytype, CUmemorytype_enum,
};
use std::marker::PhantomData;

/// An allocator that produces frames backed by the CUDA device memory.
pub struct DeviceFrameAllocator;

impl FrameAllocator for DeviceFrameAllocator {
    type FrameBuffer = DeviceBuffer;

    fn alloc(width_in_bytes: usize, height_in_rows: usize) -> Self::FrameBuffer {
        DeviceBuffer::alloc(width_in_bytes, height_in_rows)
    }

    fn memory_type() -> CUmemorytype {
        CUmemorytype_enum::CU_MEMORYTYPE_DEVICE
    }
}

/// An allocator that produces frames backed by the CUDA device memory.
///
/// The allocations are pitched (rows are padded).
pub struct PitchedDeviceFrameAllocator;

impl FrameAllocator for PitchedDeviceFrameAllocator {
    type FrameBuffer = DeviceBuffer;

    fn alloc(width_in_bytes: usize, height_in_rows: usize) -> Self::FrameBuffer {
        DeviceBuffer::alloc_pitched(width_in_bytes, height_in_rows)
    }

    fn memory_type() -> CUmemorytype {
        CUmemorytype_enum::CU_MEMORYTYPE_DEVICE
    }
}

/// An owned CUDA memory buffer.
///
/// TODO(mbernat): Upstream allocation methods into `cuda_gl_interop::CudaBuffer`.
pub struct DeviceBuffer {
    ptr: *mut u8,
    pitch: usize,
    size: usize,
}

impl DeviceBuffer {
    fn alloc(width_in_bytes: usize, height_in_rows: usize) -> Self {
        let mut ptr: CUdeviceptr = 0;
        let size = width_in_bytes * height_in_rows;
        let pitch = width_in_bytes;

        unsafe {
            cuMemAlloc_v2(&raw mut ptr, size).into_cuda_result().unwrap();
        }

        Self { ptr: ptr as *mut u8, pitch, size }
    }

    fn alloc_pitched(width_in_bytes: usize, height_in_rows: usize) -> Self {
        let mut ptr: CUdeviceptr = 0;
        let mut pitch = 0;
        let size = width_in_bytes * height_in_rows;

        // TODO(efyang): this should be a specialized type, pitched allocation is not like a normal array
        // refer to https://stackoverflow.com/questions/16119943/how-and-when-should-i-use-pitched-pointer-with-the-cuda-api
        unsafe {
            cuMemAllocPitch_v2(&raw mut ptr, &raw mut pitch, width_in_bytes, height_in_rows, 16)
                .into_cuda_result()
                .unwrap();
        }

        DeviceBuffer { ptr: ptr as *mut u8, pitch, size }
    }

    fn free(&mut self) {
        unsafe {
            cuMemFree_v2(self.ptr as CUdeviceptr)
                .into_cuda_result()
                .expect("Failure on nvdecoder frame free");
        }
    }

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

impl Drop for DeviceBuffer {
    fn drop(&mut self) {
        self.free()
    }
}

impl Buffer for DeviceBuffer {
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

/// A slice of CUDA device memory guaranteed to be valid for `'a`.
///
/// TODO(mbernat): Replace by cuda_gl_interop::CudaSlice
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
