use crate::decoder::frame::{info::FrameInfo, FrameAllocator, RawBuffer};
use nv_video_codec_sys::{CUmemorytype, CUmemorytype_enum};

pub struct HostFrameAllocator;

impl FrameAllocator for HostFrameAllocator {
    type Buffer = Vec<u8>;

    fn alloc(frame_info: &FrameInfo, _device_frame_pitch: &mut usize) -> Self::Buffer {
        vec![0; frame_info.frame_size() as usize]
    }

    fn free(_buffer: &mut Self::Buffer) {
        // Handled by `Drop`.
    }

    fn memory_type() -> CUmemorytype {
        CUmemorytype_enum::CU_MEMORYTYPE_HOST
    }
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
