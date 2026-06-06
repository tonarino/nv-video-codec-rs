use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub enum EncodeCodec {
    H264,
    #[default]
    Hevc,
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub enum EncodeProfile {
    #[default]
    AutoSelect,
}

/// ```ignore
///                     Preset
/// Tuning Info         P1 P2 P3 P4 P5 P6 P7
/// High Quality        Y  Y  N  N  N  N  N
/// Low Latency         Y  Y  Y  Y  N  N  N
/// Ultra Low Latency   Y  Y  Y  Y  N  N  N
/// ```
///
/// https://docs.nvidia.com/video-technologies/video-codec-sdk/13.0/nvenc-video-encoder-api-prog-guide/index.html#multi-nvenc-split-frame-encoding-in-hevc-and-av1
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub enum EncodePreset {
    P1,
    #[default]
    P3,
    P7,
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub enum EncodeRateControlMode {
    #[default]
    ConstantQp,
    VariableBitrate,
    ConstantBitrate,
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub enum EncodeMultiPass {
    #[default]
    Disabled,
    TwoPassQuarterResolution,
    TwoPassFullResolution,
}

/// Tuning information of NVENC encoding (not applicable to H264 and HEVC MEOnly mode).
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub enum EncodeTuningInfo {
    HighQuality,
    #[default]
    LowLatency,
    UltraLowLatency,
    Lossless,
}

/// Controls how a per-block quality map is applied when encoding a frame.
///
/// This overrides the encoder's per-block Quantization Parameter (QP)
/// — determining how aggressively a block is compressed.
/// Lower QP means higher quality; higher QP means lower quality.
///
/// The map can be provided in the optional `qp_delta_map` argument of `NvEncoder::encode_frame()`.
/// It is a flat array of one `i8` per block:
/// - H.264: 16×16 pixels
/// - HEVC: 32×32 pixels
///
/// See the NVENC SDK docs for `NV_ENC_QP_MAP_MODE`.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub enum EncodeQpMapMode {
    /// Quality map is ignored; passing `Some(...)` to `qp_delta_map` is safe but has no effect.
    #[default]
    Disabled,
    /// Each value is an emphasis level. The encoder maps the level to
    /// a relative QP modifier internally.
    ///
    /// Only supported for H.264.
    Emphasis,
    /// Each value is a signed QP delta in the range -128 to 127
    /// applied on top of the rate-control-chosen QP for that block.
    /// Negative values increase quality; positive values decrease it.
    ///
    /// Supported for both H.264 and HEVC.
    Delta,
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub struct EncodeRateControl {
    pub mode: EncodeRateControlMode,
    pub low_delay_key_frame_scale: u8,
    pub bit_rate: u32,
    pub vbv_buffer_size_bits: u32,
    pub vbv_buffer_initial_delay: u32,
    pub enable_aq: bool,
    pub multi_pass: EncodeMultiPass,
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub struct NvEncoderParams {
    pub codec: EncodeCodec,
    pub preset: EncodePreset,
    pub tuning_info: EncodeTuningInfo,
    pub frame_rate_numerator: u32,
    pub frame_rate_denominator: u32,
    pub repeat_spspps: bool,
    pub rate_control: EncodeRateControl,
    pub qp_map_mode: EncodeQpMapMode,
    pub enable_ltr: bool,
    pub ltr_num_frames: u32,
    pub ltr_trust_mode: u32,
}
