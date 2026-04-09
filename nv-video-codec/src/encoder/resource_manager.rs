use super::{nvencoder::NvEncoder, NvEncoderError};
use crate::encoder::nvencoder::NvEncInputFrame;

pub trait NvEncoderResourceManager {
    type InputResource;
    type InputResourceRef<'a>: From<&'a mut NvEncInputFrame>;
    type ResourceContext;

    fn allocate_input_buffers(
        encoder: &mut NvEncoder<Self>,
        num_input_buffers: u32,
    ) -> Result<(), NvEncoderError>;

    fn release_input_buffers(encoder: &mut NvEncoder<Self>) -> Result<(), NvEncoderError>;
}
