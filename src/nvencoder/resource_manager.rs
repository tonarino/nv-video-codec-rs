use super::{nvencoder::NvEncoder, NvEncoderError};

pub trait NvEncoderResourceManager {
    fn allocate_input_buffers(
        encoder: &mut NvEncoder,
        num_input_buffers: u32,
    ) -> Result<(), NvEncoderError> {
        todo!()
    }

    fn release_input_buffers(encoder: &mut NvEncoder) -> Result<(), NvEncoderError> {
        todo!()
    }
}
