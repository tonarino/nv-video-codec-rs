use crate::decoder::{Frame, FrameInfo};
use std::collections::VecDeque;

pub struct DecodingOutput<'a> {
    pub frames: VecDeque<Frame<'a>>,
    pub frame_info: FrameInfo,
}

impl<'a> DecodingOutput<'a> {
    pub fn new(frames: VecDeque<Frame<'a>>, frame_info: FrameInfo) -> Self {
        Self { frames, frame_info }
    }
}
