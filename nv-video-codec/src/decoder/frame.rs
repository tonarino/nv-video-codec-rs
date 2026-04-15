use crate::decoder::frame::info::FrameInfo;
use nv_video_codec_sys::CUmemorytype;

pub mod device;
pub mod host;
pub mod info;

/// This trait determines the frame allocation strategy.
pub trait FrameAllocator {
    /// Type of the memory [`Buffer`] backing the frame's image data.
    type FrameBuffer: Buffer;

    /// Allocate a buffer for a frame of the given size.
    fn alloc(width_in_bytes: usize, height_in_rows: usize) -> Self::FrameBuffer;

    /// CUDA memory type, primarily 'host' or 'device'.
    fn memory_type() -> CUmemorytype;
}

/// This trait describes buffers produced by a [`FrameAllocator`].
pub trait Buffer {
    /// Type representing an immutable slice of the buffer's memory.
    type Slice<'a>
    where
        Self: 'a;

    /// # Safety
    ///
    /// The caller promises that the pointer will not be used to invalidate the buffer.
    ///
    /// NOTE: This technically does not need to be unsafe (compare [`Vec::as_mut_ptr()`]) because
    /// it's the pointer's use, not existence, that causes problems. Still, it doesn't hurt to put
    /// extra checks in place to make people more careful.
    unsafe fn as_mut_ptr(&mut self) -> *mut u8;

    fn pitch(&self) -> usize;

    fn as_slice<'a>(&'a self) -> Self::Slice<'a>;
}

/// An owned and timestamped frame buffer.
pub struct OwnedFrame<A: FrameAllocator> {
    pub timestamp: i64,
    pub buffer: A::FrameBuffer,
}

impl<A: FrameAllocator> OwnedFrame<A> {
    pub fn from_raw_parts<'a>(&'a self) -> Frame<'a, A> {
        let slice = self.buffer.as_slice();

        Frame { timestamp: self.timestamp, slice }
    }
}

/// A borrowed and timestamped slice of a frame buffer.
pub struct Frame<'a, A: FrameAllocator>
where
    A::FrameBuffer: 'a,
{
    pub timestamp: i64,
    pub slice: <A::FrameBuffer as Buffer>::Slice<'a>,
}

pub struct DecodingOutput<F> {
    pub frames: F,
    pub frame_count: usize,
    pub frame_info: Option<FrameInfo>,
}
