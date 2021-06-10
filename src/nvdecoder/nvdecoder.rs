use nv_video_codec_sys;

pub struct Frame {}
pub struct NvDecoder {}

impl NvDecoder {
    pub fn new() -> Self {
        todo!()
    }

    /// Returns the number of frames decoded
    ///
    /// # Arguments
    /// * arg
    pub fn decode() -> usize {
        todo!()
    }

    pub fn get_frame() -> Frame {
        todo!()
    }

    pub fn get_locked_frame() -> Frame {
        todo!()
    }

    pub fn unlock_frame(frame: &mut Frame) {
        todo!()
    }

    pub fn set_reconfig_params() -> Result<(), ()> {
        todo!()
    }
}

impl Drop for NvDecoder {
    fn drop(&mut self) {
        todo!()
    }
}
