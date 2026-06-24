use super::{NvEncError, NvEncoderError};
use crate::guids::EncodeCodec;
use ffi::_NV_ENC_BUFFER_FORMAT;
use nv_video_codec_sys::{
    self as ffi, NV_ENC_CONFIG, NV_ENC_MULTI_PASS, NV_ENC_PARAMS_RC_MODE, NV_ENC_PIC_FLAGS,
    NV_ENC_QP_MAP_MODE, NV_ENC_TUNING_INFO,
};

pub use nv_video_codec_config::{
    EncodeMultiPass, EncodeQpMapMode, EncodeRateControl, EncodeRateControlMode, EncodeTuningInfo,
    NvEncoderParams,
};

ffi_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum BufferFormat = _NV_ENC_BUFFER_FORMAT
    cvt_err: BufferFormatConvertError
    {
        UNDEFINED = NV_ENC_BUFFER_FORMAT_UNDEFINED
        NV12 = NV_ENC_BUFFER_FORMAT_NV12
        YV12 = NV_ENC_BUFFER_FORMAT_YV12
        IYUV = NV_ENC_BUFFER_FORMAT_IYUV
        YUV444 = NV_ENC_BUFFER_FORMAT_YUV444
        YUV420_10BIT = NV_ENC_BUFFER_FORMAT_YUV420_10BIT
        YUV444_10BIT = NV_ENC_BUFFER_FORMAT_YUV444_10BIT
        ARGB = NV_ENC_BUFFER_FORMAT_ARGB
        ARGB10 = NV_ENC_BUFFER_FORMAT_ARGB10
        AYUV = NV_ENC_BUFFER_FORMAT_AYUV
        ABGR = NV_ENC_BUFFER_FORMAT_ABGR
        ABGR10 = NV_ENC_BUFFER_FORMAT_ABGR10
        U8 = NV_ENC_BUFFER_FORMAT_U8
    }
}

impl BufferFormat {
    pub fn get_width_in_bytes(&self, width: u32) -> Result<u32, NvEncoderError> {
        match &self {
            Self::NV12 | Self::YV12 | Self::IYUV | Self::YUV444 => Ok(width),
            Self::YUV420_10BIT | Self::YUV444_10BIT => Ok(width * 2),
            Self::ARGB | Self::ARGB10 | Self::AYUV | Self::ABGR | Self::ABGR10 => Ok(width * 4),
            _ => Err(NvEncError::InvalidParam.into()),
        }
    }

    pub fn get_num_chroma_planes(&self) -> Result<u32, NvEncoderError> {
        match &self {
            Self::NV12 | Self::YUV420_10BIT => Ok(1),
            Self::YV12 | Self::IYUV | Self::YUV444 | Self::YUV444_10BIT => Ok(2),
            Self::ARGB | Self::ARGB10 | Self::AYUV | Self::ABGR | Self::ABGR10 => Ok(0),
            _ => Err(NvEncError::InvalidParam.into()),
        }
    }

    pub fn get_chroma_pitch(&self, luma_pitch: u32) -> Result<u32, NvEncoderError> {
        match &self {
            Self::NV12 | Self::YUV420_10BIT | Self::YUV444 | Self::YUV444_10BIT => Ok(luma_pitch),
            Self::YV12 | Self::IYUV => Ok(luma_pitch.div_ceil(2)),
            Self::ARGB | Self::ARGB10 | Self::AYUV | Self::ABGR | Self::ABGR10 => Ok(0),
            _ => Err(NvEncError::InvalidParam.into()),
        }
    }

    pub fn get_chroma_subplane_offsets(
        &self,
        pitch: u32,
        height: u32,
    ) -> Result<Vec<u32>, NvEncoderError> {
        match &self {
            Self::NV12 | Self::YUV420_10BIT => Ok(vec![pitch * height]),
            Self::YV12 | Self::IYUV => Ok(vec![
                pitch * height,
                pitch * height + self.get_chroma_pitch(pitch)? * self.get_chroma_height(height)?,
            ]),
            Self::YUV444 | Self::YUV444_10BIT => Ok(vec![pitch * height, 2 * pitch * height]),
            Self::ARGB | Self::ARGB10 | Self::AYUV | Self::ABGR | Self::ABGR10 => Ok(vec![]),
            _ => Err(NvEncError::InvalidParam.into()),
        }
    }

    pub fn get_chroma_height(&self, luma_height: u32) -> Result<u32, NvEncoderError> {
        match &self {
            Self::YV12 | Self::IYUV | Self::NV12 | Self::YUV420_10BIT => Ok(luma_height + 1),
            Self::YUV444 | Self::YUV444_10BIT => Ok(luma_height),
            Self::ARGB | Self::ARGB10 | Self::AYUV | Self::ABGR | Self::ABGR10 => Ok(0),
            _ => Err(NvEncError::InvalidParam.into()),
        }
    }

    pub fn get_chroma_width_in_bytes(&self, luma_width: u32) -> Result<u32, NvEncoderError> {
        match &self {
            Self::YV12 | Self::IYUV => Ok(luma_width.div_ceil(2)),
            Self::NV12 => Ok(luma_width),
            Self::YUV420_10BIT => Ok(2 * luma_width),
            Self::YUV444 => Ok(luma_width),
            Self::YUV444_10BIT => Ok(2 * luma_width),
            Self::ARGB | Self::ARGB10 | Self::AYUV | Self::ABGR | Self::ABGR10 => Ok(0),
            _ => Err(NvEncError::InvalidParam.into()),
        }
    }
}

bitflags! {
    pub struct EncodePicFlags: u32 {
        /// Encode the current picture as an Intra picture.
        const FORCE_INTRA = NV_ENC_PIC_FLAGS::NV_ENC_PIC_FLAG_FORCEINTRA.0;
        /// Encode the current picture as an IDR picture. This flag is only valid when Picture type
        /// decision (PTD) is taken by the encoder.
        const FORCE_IDR = NV_ENC_PIC_FLAGS::NV_ENC_PIC_FLAG_FORCEIDR.0;
        /// Write the sequence and picture header in encoded bitstream of the current picture.
        const SEQUENCE_HEADER = NV_ENC_PIC_FLAGS::NV_ENC_PIC_FLAG_OUTPUT_SPSPPS.0;
        /// Indicates end of the input stream.
        const END_OF_STREAM = NV_ENC_PIC_FLAGS::NV_ENC_PIC_FLAG_EOS.0;
    }
}

/// This is the same as the standard [`From`] trait, which we cannot use because of the orphan rule.
pub(crate) trait FromConfig<T> {
    fn from_config(value: T) -> Self;
}

/// This is the same as the standard [`Into`] trait, which we cannot use because of the orphan rule.
pub(crate) trait IntoFfi<T> {
    fn into_ffi(self) -> T;
}

/// Implement Into for everything that implements From the other way.
impl<T, U: FromConfig<T>> IntoFfi<U> for T {
    fn into_ffi(self) -> U {
        U::from_config(self)
    }
}

impl FromConfig<EncodeRateControlMode> for NV_ENC_PARAMS_RC_MODE {
    fn from_config(value: EncodeRateControlMode) -> Self {
        use NV_ENC_PARAMS_RC_MODE as MODE;

        match value {
            EncodeRateControlMode::ConstantQp => MODE::NV_ENC_PARAMS_RC_CONSTQP,
            EncodeRateControlMode::VariableBitrate => MODE::NV_ENC_PARAMS_RC_VBR,
            EncodeRateControlMode::ConstantBitrate => MODE::NV_ENC_PARAMS_RC_CBR,
        }
    }
}

impl FromConfig<EncodeMultiPass> for NV_ENC_MULTI_PASS {
    fn from_config(value: EncodeMultiPass) -> Self {
        use NV_ENC_MULTI_PASS as PASS;

        match value {
            EncodeMultiPass::Disabled => PASS::NV_ENC_MULTI_PASS_DISABLED,
            EncodeMultiPass::TwoPassQuarterResolution => PASS::NV_ENC_TWO_PASS_QUARTER_RESOLUTION,
            EncodeMultiPass::TwoPassFullResolution => PASS::NV_ENC_TWO_PASS_FULL_RESOLUTION,
        }
    }
}

impl FromConfig<EncodeQpMapMode> for NV_ENC_QP_MAP_MODE {
    fn from_config(value: EncodeQpMapMode) -> Self {
        use NV_ENC_QP_MAP_MODE as MODE;

        match value {
            EncodeQpMapMode::Disabled => MODE::NV_ENC_QP_MAP_DISABLED,
            EncodeQpMapMode::Emphasis => MODE::NV_ENC_QP_MAP_EMPHASIS,
            EncodeQpMapMode::Delta => MODE::NV_ENC_QP_MAP_DELTA,
        }
    }
}

impl FromConfig<EncodeTuningInfo> for NV_ENC_TUNING_INFO {
    fn from_config(value: EncodeTuningInfo) -> Self {
        match value {
            EncodeTuningInfo::HighQuality => NV_ENC_TUNING_INFO::NV_ENC_TUNING_INFO_HIGH_QUALITY,
            EncodeTuningInfo::LowLatency => NV_ENC_TUNING_INFO::NV_ENC_TUNING_INFO_LOW_LATENCY,
            EncodeTuningInfo::UltraLowLatency => {
                NV_ENC_TUNING_INFO::NV_ENC_TUNING_INFO_ULTRA_LOW_LATENCY
            },
            EncodeTuningInfo::Lossless => NV_ENC_TUNING_INFO::NV_ENC_TUNING_INFO_LOSSLESS,
        }
    }
}

pub(crate) fn apply_params_to_encode_config(
    params: NvEncoderParams,
    encode_config: &mut NV_ENC_CONFIG,
) {
    encode_config.rcParams.rateControlMode = params.rate_control.mode.into_ffi();
    encode_config.rcParams.lowDelayKeyFrameScale = params.rate_control.low_delay_key_frame_scale;
    encode_config.rcParams.averageBitRate = params.rate_control.bit_rate;
    encode_config.rcParams.maxBitRate = params.rate_control.bit_rate;
    encode_config.rcParams.vbvBufferSize = params.rate_control.vbv_buffer_size_bits;
    encode_config.rcParams.vbvInitialDelay = params.rate_control.vbv_buffer_initial_delay;
    encode_config.rcParams.set_enableAQ(params.rate_control.enable_aq as u32);
    encode_config.rcParams.multiPass = params.rate_control.multi_pass.into_ffi();
    encode_config.rcParams.qpMapMode = params.qp_map_mode.into_ffi();

    match params.codec {
        // SAFETY: We checked the codec is H264, so we can access the union field.
        EncodeCodec::H264 => unsafe {
            let h264 = &mut encode_config.encodeCodecConfig.h264Config;
            h264.set_repeatSPSPPS(params.repeat_spspps as u32);
            if params.ltr_num_frames > 0 {
                h264.set_enableLTR(1);
                h264.ltrNumFrames = params.ltr_num_frames;
                // 0 = Per Picture mode (preferred), 1 = Trust mode (discouraged)
                h264.ltrTrustMode = params.ltr_trust_mode;
            }
        },
        // SAFETY: We checked the codec is HEVC, so we can access the union field.
        EncodeCodec::Hevc => unsafe {
            let hevc = &mut encode_config.encodeCodecConfig.hevcConfig;
            hevc.set_repeatSPSPPS(params.repeat_spspps as u32);
            if params.ltr_num_frames > 0 {
                hevc.set_enableLTR(1);
                hevc.ltrNumFrames = params.ltr_num_frames;
                // 0 = Per Picture mode (preferred), 1 = Trust mode (discouraged)
                hevc.ltrTrustMode = params.ltr_trust_mode;
            }
        },
    }
}
