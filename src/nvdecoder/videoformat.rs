use nv_video_codec_sys::CUVIDEOFORMAT;

pub struct VideoFormat {}

impl From<CUVIDEOFORMAT> for VideoFormat {
    fn from(format: CUVIDEOFORMAT) -> Self {
        Self {}
    }
}
