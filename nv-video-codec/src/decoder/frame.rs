use crate::{common::IntoCudaResult, decoder::FrameInfo};
use nv_video_codec_sys::{
    cuMemAllocPitch_v2, cuMemAlloc_v2, cuMemFree_v2, CUdeviceptr, CUmemorytype, CUmemorytype_enum,
};
use std::marker::PhantomData;

pub trait FrameAllocator {
    type Buffer: RawBuffer;

    fn alloc(frame_info: &FrameInfo, device_frame_pitch: &mut usize) -> Self::Buffer;

    fn free(data: &mut Self::Buffer);

    fn memory_type() -> CUmemorytype;
}

pub struct HostFrameAllocator;

impl FrameAllocator for HostFrameAllocator {
    type Buffer = Vec<u8>;

    fn alloc(frame_info: &FrameInfo, _device_frame_pitch: &mut usize) -> Self::Buffer {
        vec![0; frame_info.frame_size() as usize]
    }

    fn free(_data: &mut Self::Buffer) {
        // Handled by `Drop`.
    }

    fn memory_type() -> CUmemorytype {
        CUmemorytype_enum::CU_MEMORYTYPE_HOST
    }
}

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

    fn free(data: &mut Self::Buffer) {
        unsafe {
            cuMemFree_v2(data.ptr as CUdeviceptr)
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

    fn free(data: &mut Self::Buffer) {
        // TODO(mbernat): Make sure this is valid for pitched device frames.
        unsafe {
            cuMemFree_v2(data.ptr as CUdeviceptr)
                .into_cuda_result()
                .expect("Failure on nvdecoder frame free");
        }
    }

    fn memory_type() -> CUmemorytype {
        CUmemorytype_enum::CU_MEMORYTYPE_DEVICE
    }
}

pub trait RawBuffer {
    type Slice<'a>;

    fn as_mut_ptr(&mut self) -> *mut u8;

    /// # Safety
    ///
    /// Self::Slice<'a> must be valid for 'a.
    unsafe fn as_slice<'a>(&'a self) -> Self::Slice<'a>;

    fn from_slice<'a>(slice: Self::Slice<'a>) -> Self;
}

impl RawBuffer for Vec<u8> {
    type Slice<'a> = &'a [u8];

    fn as_mut_ptr(&mut self) -> *mut u8 {
        Vec::as_mut_ptr(self)
    }

    unsafe fn as_slice<'a>(&'a self) -> Self::Slice<'a> {
        Vec::as_slice(self)
    }

    fn from_slice<'a>(slice: Self::Slice<'a>) -> Self {
        slice.to_vec()
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

pub struct RawFrame<A: FrameAllocator> {
    pub timestamp: i64,
    pub data: A::Buffer,
}

impl<A: FrameAllocator> RawFrame<A> {
    /// # Safety
    ///
    /// Memory backed by `self` has to be valid for `'a`.
    pub unsafe fn from_raw_parts<'a>(&'a self) -> Frame<'a, A> {
        // SAFETY: Caller guarantees self.data lives for 'a.
        let data = unsafe { self.data.as_slice() };

        Frame { timestamp: self.timestamp, data }
    }

    pub fn into_raw_parts<'a>(frame: Frame<'a, A>) -> Self {
        let data = RawBuffer::from_slice(frame.data);

        RawFrame { timestamp: frame.timestamp, data }
    }
}

pub struct Frame<'a, A: FrameAllocator> {
    pub timestamp: i64,
    pub data: <A::Buffer as RawBuffer>::Slice<'a>,
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
