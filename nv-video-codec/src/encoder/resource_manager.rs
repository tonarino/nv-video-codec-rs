use super::{nvencoder::NvEncoder, NvEncoderError};

pub trait NvEncoderResourceManager {
    type ResourceContext;
    type InputResource;

    fn allocate_input_buffers(
        encoder: &mut NvEncoder<Self>,
        num_input_buffers: u32,
    ) -> Result<(), NvEncoderError>;

    fn release_input_buffers(encoder: &mut NvEncoder<Self>) -> Result<(), NvEncoderError>;
}
