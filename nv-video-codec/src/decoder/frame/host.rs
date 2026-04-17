use crate::decoder::frame::{Buffer, FrameAllocator};
use nv_video_codec_sys::{CUmemorytype, CUmemorytype_enum};
use std::ops::Deref;

/// An allocator that produces frames backed by the host memory.
pub struct HostFrameAllocator;

impl FrameAllocator for HostFrameAllocator {
    type FrameBuffer = HostBuffer;

    fn alloc(width_in_bytes: usize, height_in_rows: usize) -> Self::FrameBuffer {
        let size = width_in_bytes * height_in_rows;

        HostBuffer { data: vec![0; size], pitch: width_in_bytes }
    }

    fn memory_type() -> CUmemorytype {
        CUmemorytype_enum::CU_MEMORYTYPE_HOST
    }
}

pub struct HostBuffer {
    data: Vec<u8>,
    pitch: usize,
}

impl HostBuffer {}

impl Buffer for HostBuffer {
    type Slice<'a> = HostSlice<'a>;

    unsafe fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }

    fn pitch(&self) -> usize {
        self.pitch
    }

    fn as_slice<'a>(&'a self) -> Self::Slice<'a> {
        HostSlice { data: self.data.as_slice(), _pitch: self.pitch }
    }
}

pub struct HostSlice<'a> {
    data: &'a [u8],
    _pitch: usize,
}

impl<'a> Deref for HostSlice<'a> {
    type Target = &'a [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
