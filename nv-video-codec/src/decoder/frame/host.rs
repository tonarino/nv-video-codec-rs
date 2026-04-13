use crate::decoder::frame::{Buffer, FrameAllocator};
use nv_video_codec_sys::{CUmemorytype, CUmemorytype_enum};

/// An allocator that produces frames backed by the host memory.
pub struct HostFrameAllocator;

impl FrameAllocator for HostFrameAllocator {
    type FrameBuffer = Vec<u8>;

    fn alloc(width_in_bytes: usize, height_in_rows: usize) -> Self::FrameBuffer {
        let size = width_in_bytes * height_in_rows;

        vec![0; size]
    }

    fn memory_type() -> CUmemorytype {
        CUmemorytype_enum::CU_MEMORYTYPE_HOST
    }
}

impl Buffer for Vec<u8> {
    type Slice<'a> = &'a [u8];

    fn as_mut_ptr(&mut self) -> *mut u8 {
        Vec::as_mut_ptr(self)
    }

    fn pitch(&self) -> usize {
        0
    }

    unsafe fn as_slice<'a>(&'a self) -> Self::Slice<'a> {
        Vec::as_slice(self)
    }
}
