use thiserror::Error;

use crate::common::CudaError;

#[derive(Debug, Error)]
pub enum NvDecoderError {
    #[error("CUDA driver API error: {0:?}")]
    CudaError(CudaError),
}

impl From<CudaError> for NvDecoderError {
    fn from(e: CudaError) -> NvDecoderError {
        NvDecoderError::CudaError(e)
    }
}
