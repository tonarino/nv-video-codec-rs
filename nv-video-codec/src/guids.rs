use nv_video_codec_sys::{
    guids::{
        NV_ENC_CODEC_H264_GUID, NV_ENC_CODEC_HEVC_GUID, NV_ENC_CODEC_PROFILE_AUTOSELECT_GUID,
        NV_ENC_PRESET_P1_GUID, NV_ENC_PRESET_P3_GUID, NV_ENC_PRESET_P7_GUID,
    },
    GUID,
};

pub use nv_video_codec_config::{EncodeCodec, EncodePreset, EncodeProfile};

pub(crate) trait IntoGuid {
    fn into_guid(self) -> GUID;
}

impl IntoGuid for EncodeCodec {
    fn into_guid(self) -> GUID {
        match self {
            EncodeCodec::H264 => NV_ENC_CODEC_H264_GUID,
            EncodeCodec::Hevc => NV_ENC_CODEC_HEVC_GUID,
        }
    }
}

impl IntoGuid for EncodeProfile {
    fn into_guid(self) -> GUID {
        match self {
            EncodeProfile::AutoSelect => NV_ENC_CODEC_PROFILE_AUTOSELECT_GUID,
        }
    }
}

impl IntoGuid for EncodePreset {
    fn into_guid(self) -> GUID {
        match self {
            EncodePreset::P1 => NV_ENC_PRESET_P1_GUID,
            EncodePreset::P3 => NV_ENC_PRESET_P3_GUID,
            EncodePreset::P7 => NV_ENC_PRESET_P7_GUID,
        }
    }
}
