use ffi::NVENCSTATUS;
use nv_video_codec_sys as ffi;
use thiserror::Error;

use crate::common::CudaError;

#[derive(Debug, Error)]
pub enum NvEncoderError {
    #[error("CUDA driver API error: {0:?}")]
    CudaError(CudaError),
    #[error("Encode error: {0:?}")]
    NvEncError(NvEncError),
}

impl From<CudaError> for NvEncoderError {
    fn from(e: CudaError) -> NvEncoderError {
        NvEncoderError::CudaError(e)
    }
}

// TODO: tack strings onto errors
impl From<NvEncError> for NvEncoderError {
    fn from(e: NvEncError) -> NvEncoderError {
        NvEncoderError::NvEncError(e)
    }
}

#[derive(Debug)]
pub enum NvEncError {
    NoEncodeDevice,
    UnsupportedDevice,
    InvalidEncoderDevice,
    InalidDevice,
    DeviceNoLongerExists,
    InvalidPointer,
    InvalidEvent,
    InvalidParam,
    InvalidCall,
    OutOfMemory,
    EncoderNotInitialized,
    UnsupportedParam,
    LockBusy,
    NotEnoughBuffer,
    InvalidVersion,
    MapFailed,
    NeedMoreInput,
    EncoderBusy,
    EventNotRegistered,
    ErrGeneric,
    IncompatibleClientKey,
    Unimplemented,
    ResourceRegisterFailed,
    ResourceNotRegistered,
    ResourceNotMapped,
}

pub(super) trait IntoNvEncResult {
    fn into_nvenc_result(self) -> Result<(), NvEncError>;
}

impl IntoNvEncResult for NVENCSTATUS {
    fn into_nvenc_result(self) -> Result<(), NvEncError> {
        match self {
            NVENCSTATUS::NV_ENC_SUCCESS => Ok(()),
            NVENCSTATUS::NV_ENC_ERR_NO_ENCODE_DEVICE => Err(NvEncError::NoEncodeDevice),
            NVENCSTATUS::NV_ENC_ERR_UNSUPPORTED_DEVICE => Err(NvEncError::UnsupportedDevice),
            NVENCSTATUS::NV_ENC_ERR_INVALID_ENCODERDEVICE => Err(NvEncError::InvalidEncoderDevice),
            NVENCSTATUS::NV_ENC_ERR_INVALID_DEVICE => Err(NvEncError::InalidDevice),
            NVENCSTATUS::NV_ENC_ERR_DEVICE_NOT_EXIST => Err(NvEncError::DeviceNoLongerExists),
            NVENCSTATUS::NV_ENC_ERR_INVALID_PTR => Err(NvEncError::InvalidPointer),
            NVENCSTATUS::NV_ENC_ERR_INVALID_EVENT => Err(NvEncError::InvalidEvent),
            NVENCSTATUS::NV_ENC_ERR_INVALID_PARAM => Err(NvEncError::InvalidParam),
            NVENCSTATUS::NV_ENC_ERR_INVALID_CALL => Err(NvEncError::InvalidCall),
            NVENCSTATUS::NV_ENC_ERR_OUT_OF_MEMORY => Err(NvEncError::OutOfMemory),
            NVENCSTATUS::NV_ENC_ERR_ENCODER_NOT_INITIALIZED => {
                Err(NvEncError::EncoderNotInitialized)
            },
            NVENCSTATUS::NV_ENC_ERR_UNSUPPORTED_PARAM => Err(NvEncError::UnsupportedParam),
            NVENCSTATUS::NV_ENC_ERR_LOCK_BUSY => Err(NvEncError::LockBusy),
            NVENCSTATUS::NV_ENC_ERR_NOT_ENOUGH_BUFFER => Err(NvEncError::NotEnoughBuffer),
            NVENCSTATUS::NV_ENC_ERR_INVALID_VERSION => Err(NvEncError::InvalidVersion),
            NVENCSTATUS::NV_ENC_ERR_MAP_FAILED => Err(NvEncError::MapFailed),
            NVENCSTATUS::NV_ENC_ERR_NEED_MORE_INPUT => Err(NvEncError::NeedMoreInput),
            NVENCSTATUS::NV_ENC_ERR_ENCODER_BUSY => Err(NvEncError::EncoderBusy),
            NVENCSTATUS::NV_ENC_ERR_EVENT_NOT_REGISTERD => Err(NvEncError::EventNotRegistered),
            NVENCSTATUS::NV_ENC_ERR_GENERIC => Err(NvEncError::ErrGeneric),
            NVENCSTATUS::NV_ENC_ERR_INCOMPATIBLE_CLIENT_KEY => {
                Err(NvEncError::IncompatibleClientKey)
            },
            NVENCSTATUS::NV_ENC_ERR_UNIMPLEMENTED => Err(NvEncError::Unimplemented),
            NVENCSTATUS::NV_ENC_ERR_RESOURCE_REGISTER_FAILED => {
                Err(NvEncError::ResourceRegisterFailed)
            },
            NVENCSTATUS::NV_ENC_ERR_RESOURCE_NOT_REGISTERED => {
                Err(NvEncError::ResourceNotRegistered)
            },
            NVENCSTATUS::NV_ENC_ERR_RESOURCE_NOT_MAPPED => Err(NvEncError::ResourceNotMapped),
        }
    }
}
