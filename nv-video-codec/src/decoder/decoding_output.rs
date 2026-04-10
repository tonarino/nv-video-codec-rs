use crate::decoder::FrameInfo;

pub struct DecodingOutput<I> {
    pub frames: I,
    pub frame_count: usize,
    pub frame_info: FrameInfo,
}
