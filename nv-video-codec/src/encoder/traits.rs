use crate::encoder::{EncodePicFlags, NvEncoderError};

pub type NvEncoderResult<T> = Result<T, NvEncoderError>;

/// Extended NvEncoder trait providing higher level functions
pub trait NvEncoderExt {
    fn encode_frame_from_data(
        &mut self,
        data: &[u8],
        width: u32,
        height: u32,
        pic_params: EncodePicFlags,
        output_packet_buffer: &mut Vec<&[u8]>,
    ) -> NvEncoderResult<()>;
}
