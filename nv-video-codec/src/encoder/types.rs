use super::{NvEncError, NvEncoderError};
use crate::guids::{EncodeCodec, EncodePreset};
use ffi::_NV_ENC_BUFFER_FORMAT;
use nv_video_codec_sys::{
    self as ffi, NV_ENC_PARAMS_RC_MODE, NV_ENC_PIC_FLAGS, NV_ENC_TUNING_INFO,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EncodeRateControlMode {
    ConstantQp,
    VariableBitrate,
    ConstantBitrate,
}

impl From<EncodeRateControlMode> for NV_ENC_PARAMS_RC_MODE {
    fn from(value: EncodeRateControlMode) -> Self {
        match value {
            EncodeRateControlMode::ConstantQp => NV_ENC_PARAMS_RC_MODE::NV_ENC_PARAMS_RC_CONSTQP,
            EncodeRateControlMode::VariableBitrate => NV_ENC_PARAMS_RC_MODE::NV_ENC_PARAMS_RC_VBR,
            EncodeRateControlMode::ConstantBitrate => NV_ENC_PARAMS_RC_MODE::NV_ENC_PARAMS_RC_CBR,
        }
    }
}

/// Tuning information of NVENC encoding (not applicable to H264 and HEVC MEOnly mode).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EncodeTuningInfo {
    HighQuality,
    LowLatency,
    UltraLowLatency,
    Lossless,
}

impl From<EncodeTuningInfo> for NV_ENC_TUNING_INFO {
    fn from(value: EncodeTuningInfo) -> Self {
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

pub struct EncodeRateControl {
    pub mode: EncodeRateControlMode,
    pub low_delay_key_frame_scale: u8,
    pub average_bit_rate: u32,
    pub enable_aq: bool,
}

pub struct NvEncoderParams {
    pub codec: EncodeCodec,
    pub preset: EncodePreset,
    pub tuning_info: EncodeTuningInfo,
    pub frame_rate: u32,
    pub repeat_spspps: bool,
    pub rate_control: EncodeRateControl,
}
