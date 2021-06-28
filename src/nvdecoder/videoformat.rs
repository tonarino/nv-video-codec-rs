use nv_video_codec_sys::CUVIDEOFORMAT;

// TODO(efyang) implement wrapper for videoformat
pub struct VideoFormat {}

impl From<CUVIDEOFORMAT> for VideoFormat {
    fn from(format: CUVIDEOFORMAT) -> Self {
        Self {}
    }
}
