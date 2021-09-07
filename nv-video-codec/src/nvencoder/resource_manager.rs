use super::{nvencoderbase::NvEncoderBase, NvEncoderError};

pub(super) trait NvEncoderResourceManager {
    fn allocate_input_buffers(
        encoder: &mut NvEncoderBase<Self>,
        num_input_buffers: u32,
    ) -> Result<(), NvEncoderError>;

    fn release_input_buffers(encoder: &mut NvEncoderBase<Self>) -> Result<(), NvEncoderError>;
}
