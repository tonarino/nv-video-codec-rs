use nv_video_codec_sys::{
    guids::{
        NV_ENC_CODEC_H264_GUID, NV_ENC_CODEC_HEVC_GUID, NV_ENC_CODEC_PROFILE_AUTOSELECT_GUID,
        NV_ENC_PRESET_P1_GUID, NV_ENC_PRESET_P3_GUID, NV_ENC_PRESET_P7_GUID,
    },
    GUID,
};

#[non_exhaustive]
pub enum EncodeCodec {
    H264,
    Hevc,
}

impl EncodeCodec {
    pub(crate) fn as_guid(&self) -> GUID {
        match self {
            EncodeCodec::H264 => NV_ENC_CODEC_H264_GUID,
            EncodeCodec::Hevc => NV_ENC_CODEC_HEVC_GUID,
        }
    }
}

#[non_exhaustive]
pub enum EncodeProfile {
    AutoSelect,
}

impl EncodeProfile {
    pub(crate) fn as_guid(&self) -> GUID {
        match self {
            EncodeProfile::AutoSelect => NV_ENC_CODEC_PROFILE_AUTOSELECT_GUID,
        }
    }
}

/// ```
///                     Preset
/// Tuning Info         P1 P2 P3 P4 P5 P6 P7
/// High Quality        Y  Y  N  N  N  N  N
/// Low Latency         Y  Y  Y  Y  N  N  N
/// Ultra Low Latency   Y  Y  Y  Y  N  N  N
/// ```
///
/// https://docs.nvidia.com/video-technologies/video-codec-sdk/13.0/nvenc-video-encoder-api-prog-guide/index.html#multi-nvenc-split-frame-encoding-in-hevc-and-av1
#[non_exhaustive]
pub enum EncodePreset {
    P1,
    P3,
    P7,
}

impl EncodePreset {
    pub(crate) fn as_guid(&self) -> GUID {
        match self {
            EncodePreset::P1 => NV_ENC_PRESET_P1_GUID,
            EncodePreset::P3 => NV_ENC_PRESET_P3_GUID,
            EncodePreset::P7 => NV_ENC_PRESET_P7_GUID,
        }
    }
}
