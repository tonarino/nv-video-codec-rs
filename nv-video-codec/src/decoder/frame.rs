pub struct Frame<'a> {
    pub timestamp: i64,
    pub data: FrameData<'a>,
}

pub enum FrameData<'a> {
    Owned(Vec<u8>),
    // TODO(mbernat): Device slices belong to a GPU address space, it's likely a UB to make Rust
    // slices out of them.
    Device(&'a mut [u8]),
    // DevicePitched(&'a [u8]),
}

impl<'a> AsMut<[u8]> for FrameData<'a> {
    fn as_mut(&mut self) -> &mut [u8] {
        match self {
            Self::Owned(v) => v,
            Self::Device(s) => s,
        }
    }
}

impl AsRef<[u8]> for FrameData<'_> {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Owned(v) => v,
            Self::Device(s) => s,
        }
    }
}
