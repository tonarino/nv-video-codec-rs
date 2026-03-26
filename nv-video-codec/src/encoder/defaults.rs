use super::nvencoder::{NV_ENC_CONFIG_VER, NV_ENC_PRESET_CONFIG_VER};
use nv_video_codec_sys::{
    _NV_ENC_MV_PRECISION, NV_ENC_CONFIG, NV_ENC_PARAMS_FRAME_FIELD_MODE, NV_ENC_PRESET_CONFIG,
};

pub trait CustomDefault {
    fn default() -> Self;
}

impl CustomDefault for NV_ENC_CONFIG {
    fn default() -> Self {
        Self {
            version: NV_ENC_CONFIG_VER,
            frameFieldMode: NV_ENC_PARAMS_FRAME_FIELD_MODE::NV_ENC_PARAMS_FRAME_FIELD_MODE_FRAME,
            profileGUID: Default::default(),
            gopLength: Default::default(),
            frameIntervalP: Default::default(),
            monoChromeEncoding: Default::default(),
            mvPrecision: _NV_ENC_MV_PRECISION::NV_ENC_MV_PRECISION_DEFAULT,
            rcParams: Default::default(),
            encodeCodecConfig: Default::default(),
            reserved: [0; 278],
            reserved2: [std::ptr::null_mut(); 64],
        }
    }
}

impl CustomDefault for NV_ENC_PRESET_CONFIG {
    fn default() -> Self {
        Self {
            version: NV_ENC_PRESET_CONFIG_VER,
            presetCfg: CustomDefault::default(),
            reserved1: [0; 255],
            reserved2: [std::ptr::null_mut(); 64],
        }
    }
}
