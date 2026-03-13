use crate::decoder::{Frame, FrameAllocator, FrameInfo, NvDecoder};
use std::collections::VecDeque;

pub struct DecodingOutput<'a, A: FrameAllocator> {
    pub frames: VecDeque<Frame<'a, A>>,
    pub frame_info: FrameInfo,
    decoder: &'a mut NvDecoder<A>,
}

impl<'a, A: FrameAllocator> DecodingOutput<'a, A> {
    pub fn new(
        frames: VecDeque<Frame<'a, A>>,
        frame_info: FrameInfo,
        decoder: &'a mut NvDecoder<A>,
    ) -> Self {
        Self { frames, frame_info, decoder }
    }
}

impl<'a, A: FrameAllocator> Drop for DecodingOutput<'a, A> {
    fn drop(&mut self) {
        self.decoder.reclaim_frames(self.frames.drain(..));
    }
}
