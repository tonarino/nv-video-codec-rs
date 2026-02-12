use nv_video_codec_sys::CUdeviceptr;

#[derive(Clone)]
pub struct Frame {
    pub timestamp: i64,
    pub data: FrameData,
}

#[derive(Clone)]
pub enum FrameData {
    Owned(Vec<u8>),
    Device(DeviceSlice),
    // DevicePitched(&'a [u8]),
}

#[derive(Clone)]
pub struct DeviceSlice {
    ptr: CUdeviceptr,
    size: usize,
}

impl DeviceSlice {
    pub fn new(ptr: CUdeviceptr, size: usize) -> Self {
        Self { ptr, size }
    }
}

impl AsMut<[u8]> for FrameData {
    fn as_mut(&mut self) -> &mut [u8] {
        match self {
            Self::Owned(v) => v,
            Self::Device(s) => {
                // TODO(mbernat): I have no idea why we are trying to interpret device slices as
                // Rust slices, these are different address spaces, so this the Rust slice is at
                // best meaningless, at worst a UB.
                unsafe { std::slice::from_raw_parts_mut(s.ptr as *mut u8, s.size) }
            },
        }
    }
}

impl AsRef<[u8]> for FrameData {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Owned(v) => v,
            Self::Device(s) => {
                // TODO(mbernat): I have no idea why we are trying to interpret device slices as
                // Rust slices, these are different address spaces, so this the Rust slice is at
                // best meaningless, at worst a UB.
                unsafe { std::slice::from_raw_parts_mut(s.ptr as *mut u8, s.size) }
            },
        }
    }
}
