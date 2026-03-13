use crate::{common::IntoCudaResult, decoder::FrameInfo};
use nv_video_codec_sys::{cuMemAllocPitch_v2, cuMemAlloc_v2, cuMemFree_v2, CUdeviceptr};
use std::marker::PhantomData;

pub trait FrameAllocator {
    type RawData: Raw;
    type Data<'a>;

    fn alloc(frame_info: &FrameInfo, device_frame_pitch: &mut usize) -> Self::RawData;

    fn free(data: &mut Self::RawData);
}

pub struct HostFrameAllocator;

impl FrameAllocator for HostFrameAllocator {
    type Data<'a> = Vec<u8>;
    type RawData = Vec<u8>;

    fn alloc(frame_info: &FrameInfo, _device_frame_pitch: &mut usize) -> Self::RawData {
        vec![0; frame_info.frame_size() as usize]
    }

    fn free(_data: &mut Self::RawData) {
        // Handled by `Drop`.
    }
}

pub struct DeviceFrameAllocator;

impl FrameAllocator for DeviceFrameAllocator {
    type Data<'a> = DeviceSlice<'a>;
    type RawData = RawDeviceSlice;

    fn alloc(frame_info: &FrameInfo, _device_frame_pitch: &mut usize) -> Self::RawData {
        let mut frame_data_device_ptr: CUdeviceptr = 0;
        let len = frame_info.frame_size() as usize;

        unsafe {
            cuMemAlloc_v2(&mut frame_data_device_ptr, len).into_cuda_result().unwrap();
        }

        RawDeviceSlice { ptr: frame_data_device_ptr as *mut u8, len }
    }

    fn free(data: &mut Self::RawData) {
        unsafe {
            cuMemFree_v2(data.ptr as CUdeviceptr)
                .into_cuda_result()
                .expect("Failure on nvdecoder frame free");
        }
    }
}

pub struct PitchedDeviceFrameAllocator;

impl FrameAllocator for PitchedDeviceFrameAllocator {
    // TODO(mbernat): Check if we need different types here.
    type Data<'a> = DeviceSlice<'a>;
    type RawData = RawDeviceSlice;

    fn alloc(frame_info: &FrameInfo, device_frame_pitch: &mut usize) -> Self::RawData {
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

        RawDeviceSlice { ptr: frame_data_device_ptr as *mut u8, len }
    }

    fn free(data: &mut Self::RawData) {
        // TODO(mbernat): Make sure this is valid for pitched device frames.
        unsafe {
            cuMemFree_v2(data.ptr as CUdeviceptr)
                .into_cuda_result()
                .expect("Failure on nvdecoder frame free");
        }
    }
}

pub trait Raw {
    fn as_mut_ptr(&mut self) -> *mut u8;
}

impl Raw for Vec<u8> {
    fn as_mut_ptr(&mut self) -> *mut u8 {
        Vec::as_mut_ptr(self)
    }
}

impl Raw for RawDeviceSlice {
    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr
    }
}

pub struct RawFrame<T: FrameAllocator> {
    pub timestamp: i64,
    pub data: T::RawData,
}

pub struct Frame<'a, T: FrameAllocator> {
    pub timestamp: i64,
    pub data: T::Data<'a>,
}

/// A GPU device slice guaranteed to be valid for `'a`.
pub struct DeviceSlice<'a> {
    ptr: *mut u8,
    len: usize,
    _phantom_data: PhantomData<&'a ()>,
}

impl<'a> DeviceSlice<'a> {
    /// Safety: The caller guarantees that the slice is valid for `'a`.
    pub unsafe fn new(ptr: *mut u8, len: usize) -> Self {
        Self { ptr, len, _phantom_data: PhantomData }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.ptr
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

pub struct RawDeviceSlice {
    pub ptr: *mut u8,
    pub len: usize,
}
