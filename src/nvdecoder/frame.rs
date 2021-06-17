pub struct Frame<'a> {
    pub timestamp: i64,
    pub data: FrameData<'a>,
}

pub enum FrameData<'a> {
    Owned(Vec<u8>),
    Device(&'a mut [u8]),
    // DevicePitched(&'a [u8]),
}

impl<'a> FrameData<'a> {
    pub fn as_mut(&mut self) -> &mut [u8] {
        match self {
            Self::Owned(v) => v,
            Self::Device(s) => *s,
        }
    }
}
