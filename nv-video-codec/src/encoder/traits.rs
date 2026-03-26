use crate::encoder::NvEncoderError;
use nv_video_codec_sys::NV_ENC_PIC_PARAMS;

pub type NvEncoderResult<T> = Result<T, NvEncoderError>;

/// Extended NvEncoder trait providing higher level functions
pub trait NvEncoderExt {
    fn encode_frame_from_data(
        &mut self,
        data: &[u8],
        width: u32,
        height: u32,
        pic_params: Option<NV_ENC_PIC_PARAMS>,
        output_packet_buffer: &mut Vec<&[u8]>,
    ) -> NvEncoderResult<()>;
}
