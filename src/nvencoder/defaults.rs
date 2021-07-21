use nv_video_codec_sys::{NV_ENC_CONFIG, NV_ENC_PARAMS_FRAME_FIELD_MODE, _NV_ENC_MV_PRECISION};

pub trait CustomDefault {
    fn default() -> Self;
}

impl CustomDefault for NV_ENC_CONFIG {
    fn default() -> Self {
        Self {
            version: Default::default(),
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
