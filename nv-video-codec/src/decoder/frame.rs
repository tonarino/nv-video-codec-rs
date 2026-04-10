use crate::decoder::frame::info::FrameInfo;
use nv_video_codec_sys::CUmemorytype;

pub mod device;
pub mod host;
pub mod info;

pub trait FrameAllocator {
    type Buffer: RawBuffer;

    fn alloc(frame_info: &FrameInfo, device_frame_pitch: &mut usize) -> Self::Buffer;

    fn free(buffer: &mut Self::Buffer);

    fn memory_type() -> CUmemorytype;
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

pub struct RawFrame<A: FrameAllocator> {
    pub timestamp: i64,
    pub buffer: A::Buffer,
}

impl<A: FrameAllocator> RawFrame<A> {
    /// # Safety
    ///
    /// Memory backed by `self` has to be valid for `'a`.
    pub unsafe fn from_raw_parts<'a>(&'a self) -> Frame<'a, A> {
        // SAFETY: Caller guarantees self.buffer lives for 'a.
        let slice = unsafe { self.buffer.as_slice() };

        Frame { timestamp: self.timestamp, slice }
    }

    pub fn into_raw_parts<'a>(frame: Frame<'a, A>) -> Self {
        let buffer = RawBuffer::from_slice(frame.slice);

        RawFrame { timestamp: frame.timestamp, buffer }
    }
}

pub struct Frame<'a, A: FrameAllocator> {
    pub timestamp: i64,
    pub slice: <A::Buffer as RawBuffer>::Slice<'a>,
}

pub struct DecodingOutput<F> {
    pub frames: F,
    pub frame_count: usize,
    pub frame_info: FrameInfo,
}
