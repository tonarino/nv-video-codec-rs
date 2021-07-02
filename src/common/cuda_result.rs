use ffi::cudaError_enum;
use nv_video_codec_sys as ffi;

pub type CudaResult<T> = Result<T, CudaError>;

#[derive(Debug)]
pub enum CudaError {
    InvalidValue,
    OutOfMemory,
    NotInitialized,
    Deinitialized,
    ProfilerDisabled,
    ProfilerNotInitialized,
    ProfilerAlreadyStarted,
    ProfilerAlreadyStopped,
    StubLibrary,
    NoDevice,
    InvalidDevice,
    DeviceNotLicensed,
    InvalidImage,
    InvalidContext,
    ContextAlreadyCurrent,
    MapFailed,
    UnmapFailed,
    ArrayIsMapped,
    AlreadyMapped,
    NoBinaryForGpu,
    AlreadyAcquired,
    NotMapped,
    NotMappedAsArray,
    NotMappedAsPointer,
    EccUncorrectable,
    UnsupportedLimit,
    ContextAlreadyInUse,
    PeerAccessUnsupported,
    InvalidPtx,
    InvalidGraphicsContext,
    NvlinkUncorrectable,
    JitCompilerNotFound,
    UnsupportedPtxVersion,
    JitCompilationDisabled,
    InvalidSource,
    FileNotFound,
    SharedObjectSymbolNotFound,
    SharedObjectInitFailed,
    OperatingSystem,
    InvalidHandle,
    IllegalState,
    NotFound,
    NotReady,
    IllegalAddress,
    LaunchOutOfResources,
    LaunchTimeout,
    LaunchIncompatibleTexturing,
    PeerAccessAlreadyEnabled,
    PeerAccessNotEnabled,
    PrimaryContextActive,
    ContextIsDestroyed,
    Assert,
    TooManyPeers,
    HostMemoryAlreadyRegistered,
    HostMemoryNotRegistered,
    HardwareStackError,
    IllegalInstruction,
    MisalignedAddress,
    InvalidAddressSpace,
    InvalidPc,
    LaunchFailed,
    CooperativeLaunchTooLarge,
    NotPermitted,
    NotSupported,
    SystemNotReady,
    SystemDriverMismatch,
    CompatNotSupportedOnDevice,
    StreamCaptureUnsupported,
    StreamCaptureInvalidated,
    StreamCaptureMerge,
    StreamCaptureUnmatched,
    StreamCaptureUnjoined,
    StreamCaptureIsolation,
    StreamCaptureImplicit,
    CapturedEvent,
    StreamCaptureWrongThread,
    Timeout,
    GraphExecUpdateFailure,
    Unknown,
}

pub trait IntoCudaResult<T> {
    fn into_cuda_result(self) -> CudaResult<T>;
}

impl<T> IntoCudaResult<T> for CudaResult<T> {
    fn into_cuda_result(self) -> CudaResult<T> {
        self
    }
}

impl IntoCudaResult<()> for cudaError_enum {
    fn into_cuda_result(self) -> CudaResult<()> {
        match self {
            cudaError_enum::CUDA_SUCCESS => Ok(()),
            cudaError_enum::CUDA_ERROR_INVALID_VALUE => Err(CudaError::InvalidValue),
            cudaError_enum::CUDA_ERROR_OUT_OF_MEMORY => Err(CudaError::OutOfMemory),
            cudaError_enum::CUDA_ERROR_NOT_INITIALIZED => Err(CudaError::NotInitialized),
            cudaError_enum::CUDA_ERROR_DEINITIALIZED => Err(CudaError::Deinitialized),
            cudaError_enum::CUDA_ERROR_PROFILER_DISABLED => Err(CudaError::ProfilerDisabled),
            cudaError_enum::CUDA_ERROR_PROFILER_NOT_INITIALIZED => {
                Err(CudaError::ProfilerNotInitialized)
            },
            cudaError_enum::CUDA_ERROR_PROFILER_ALREADY_STARTED => {
                Err(CudaError::ProfilerAlreadyStarted)
            },
            cudaError_enum::CUDA_ERROR_PROFILER_ALREADY_STOPPED => {
                Err(CudaError::ProfilerAlreadyStopped)
            },
            cudaError_enum::CUDA_ERROR_STUB_LIBRARY => Err(CudaError::StubLibrary),
            cudaError_enum::CUDA_ERROR_NO_DEVICE => Err(CudaError::NoDevice),
            cudaError_enum::CUDA_ERROR_INVALID_DEVICE => Err(CudaError::InvalidDevice),
            cudaError_enum::CUDA_ERROR_DEVICE_NOT_LICENSED => Err(CudaError::DeviceNotLicensed),
            cudaError_enum::CUDA_ERROR_INVALID_IMAGE => Err(CudaError::InvalidImage),
            cudaError_enum::CUDA_ERROR_INVALID_CONTEXT => Err(CudaError::InvalidContext),
            cudaError_enum::CUDA_ERROR_CONTEXT_ALREADY_CURRENT => {
                Err(CudaError::ContextAlreadyCurrent)
            },
            cudaError_enum::CUDA_ERROR_MAP_FAILED => Err(CudaError::MapFailed),
            cudaError_enum::CUDA_ERROR_UNMAP_FAILED => Err(CudaError::UnmapFailed),
            cudaError_enum::CUDA_ERROR_ARRAY_IS_MAPPED => Err(CudaError::ArrayIsMapped),
            cudaError_enum::CUDA_ERROR_ALREADY_MAPPED => Err(CudaError::AlreadyMapped),
            cudaError_enum::CUDA_ERROR_NO_BINARY_FOR_GPU => Err(CudaError::NoBinaryForGpu),
            cudaError_enum::CUDA_ERROR_ALREADY_ACQUIRED => Err(CudaError::AlreadyAcquired),
            cudaError_enum::CUDA_ERROR_NOT_MAPPED => Err(CudaError::NotMapped),
            cudaError_enum::CUDA_ERROR_NOT_MAPPED_AS_ARRAY => Err(CudaError::NotMappedAsArray),
            cudaError_enum::CUDA_ERROR_NOT_MAPPED_AS_POINTER => Err(CudaError::NotMappedAsPointer),
            cudaError_enum::CUDA_ERROR_ECC_UNCORRECTABLE => Err(CudaError::EccUncorrectable),
            cudaError_enum::CUDA_ERROR_UNSUPPORTED_LIMIT => Err(CudaError::UnsupportedLimit),
            cudaError_enum::CUDA_ERROR_CONTEXT_ALREADY_IN_USE => {
                Err(CudaError::ContextAlreadyInUse)
            },
            cudaError_enum::CUDA_ERROR_PEER_ACCESS_UNSUPPORTED => {
                Err(CudaError::PeerAccessUnsupported)
            },
            cudaError_enum::CUDA_ERROR_INVALID_PTX => Err(CudaError::InvalidPtx),
            cudaError_enum::CUDA_ERROR_INVALID_GRAPHICS_CONTEXT => {
                Err(CudaError::InvalidGraphicsContext)
            },
            cudaError_enum::CUDA_ERROR_NVLINK_UNCORRECTABLE => Err(CudaError::NvlinkUncorrectable),
            cudaError_enum::CUDA_ERROR_JIT_COMPILER_NOT_FOUND => {
                Err(CudaError::JitCompilerNotFound)
            },
            cudaError_enum::CUDA_ERROR_UNSUPPORTED_PTX_VERSION => {
                Err(CudaError::UnsupportedPtxVersion)
            },
            cudaError_enum::CUDA_ERROR_JIT_COMPILATION_DISABLED => {
                Err(CudaError::JitCompilationDisabled)
            },
            cudaError_enum::CUDA_ERROR_INVALID_SOURCE => Err(CudaError::InvalidSource),
            cudaError_enum::CUDA_ERROR_FILE_NOT_FOUND => Err(CudaError::FileNotFound),
            cudaError_enum::CUDA_ERROR_SHARED_OBJECT_SYMBOL_NOT_FOUND => {
                Err(CudaError::SharedObjectSymbolNotFound)
            },
            cudaError_enum::CUDA_ERROR_SHARED_OBJECT_INIT_FAILED => {
                Err(CudaError::SharedObjectInitFailed)
            },
            cudaError_enum::CUDA_ERROR_OPERATING_SYSTEM => Err(CudaError::OperatingSystem),
            cudaError_enum::CUDA_ERROR_INVALID_HANDLE => Err(CudaError::InvalidHandle),
            cudaError_enum::CUDA_ERROR_ILLEGAL_STATE => Err(CudaError::IllegalState),
            cudaError_enum::CUDA_ERROR_NOT_FOUND => Err(CudaError::NotFound),
            cudaError_enum::CUDA_ERROR_NOT_READY => Err(CudaError::NotReady),
            cudaError_enum::CUDA_ERROR_ILLEGAL_ADDRESS => Err(CudaError::IllegalAddress),
            cudaError_enum::CUDA_ERROR_LAUNCH_OUT_OF_RESOURCES => {
                Err(CudaError::LaunchOutOfResources)
            },
            cudaError_enum::CUDA_ERROR_LAUNCH_TIMEOUT => Err(CudaError::LaunchTimeout),
            cudaError_enum::CUDA_ERROR_LAUNCH_INCOMPATIBLE_TEXTURING => {
                Err(CudaError::LaunchIncompatibleTexturing)
            },
            cudaError_enum::CUDA_ERROR_PEER_ACCESS_ALREADY_ENABLED => {
                Err(CudaError::PeerAccessAlreadyEnabled)
            },
            cudaError_enum::CUDA_ERROR_PEER_ACCESS_NOT_ENABLED => {
                Err(CudaError::PeerAccessNotEnabled)
            },
            cudaError_enum::CUDA_ERROR_PRIMARY_CONTEXT_ACTIVE => {
                Err(CudaError::PrimaryContextActive)
            },
            cudaError_enum::CUDA_ERROR_CONTEXT_IS_DESTROYED => Err(CudaError::ContextIsDestroyed),
            cudaError_enum::CUDA_ERROR_ASSERT => Err(CudaError::Assert),
            cudaError_enum::CUDA_ERROR_TOO_MANY_PEERS => Err(CudaError::TooManyPeers),
            cudaError_enum::CUDA_ERROR_HOST_MEMORY_ALREADY_REGISTERED => {
                Err(CudaError::HostMemoryAlreadyRegistered)
            },
            cudaError_enum::CUDA_ERROR_HOST_MEMORY_NOT_REGISTERED => {
                Err(CudaError::HostMemoryNotRegistered)
            },
            cudaError_enum::CUDA_ERROR_HARDWARE_STACK_ERROR => Err(CudaError::HardwareStackError),
            cudaError_enum::CUDA_ERROR_ILLEGAL_INSTRUCTION => Err(CudaError::IllegalInstruction),
            cudaError_enum::CUDA_ERROR_MISALIGNED_ADDRESS => Err(CudaError::MisalignedAddress),
            cudaError_enum::CUDA_ERROR_INVALID_ADDRESS_SPACE => Err(CudaError::InvalidAddressSpace),
            cudaError_enum::CUDA_ERROR_INVALID_PC => Err(CudaError::InvalidPc),
            cudaError_enum::CUDA_ERROR_LAUNCH_FAILED => Err(CudaError::LaunchFailed),
            cudaError_enum::CUDA_ERROR_COOPERATIVE_LAUNCH_TOO_LARGE => {
                Err(CudaError::CooperativeLaunchTooLarge)
            },
            cudaError_enum::CUDA_ERROR_NOT_PERMITTED => Err(CudaError::NotPermitted),
            cudaError_enum::CUDA_ERROR_NOT_SUPPORTED => Err(CudaError::NotSupported),
            cudaError_enum::CUDA_ERROR_SYSTEM_NOT_READY => Err(CudaError::SystemNotReady),
            cudaError_enum::CUDA_ERROR_SYSTEM_DRIVER_MISMATCH => {
                Err(CudaError::SystemDriverMismatch)
            },
            cudaError_enum::CUDA_ERROR_COMPAT_NOT_SUPPORTED_ON_DEVICE => {
                Err(CudaError::CompatNotSupportedOnDevice)
            },
            cudaError_enum::CUDA_ERROR_STREAM_CAPTURE_UNSUPPORTED => {
                Err(CudaError::StreamCaptureUnsupported)
            },
            cudaError_enum::CUDA_ERROR_STREAM_CAPTURE_INVALIDATED => {
                Err(CudaError::StreamCaptureInvalidated)
            },
            cudaError_enum::CUDA_ERROR_STREAM_CAPTURE_MERGE => Err(CudaError::StreamCaptureMerge),
            cudaError_enum::CUDA_ERROR_STREAM_CAPTURE_UNMATCHED => {
                Err(CudaError::StreamCaptureUnmatched)
            },
            cudaError_enum::CUDA_ERROR_STREAM_CAPTURE_UNJOINED => {
                Err(CudaError::StreamCaptureUnjoined)
            },
            cudaError_enum::CUDA_ERROR_STREAM_CAPTURE_ISOLATION => {
                Err(CudaError::StreamCaptureIsolation)
            },
            cudaError_enum::CUDA_ERROR_STREAM_CAPTURE_IMPLICIT => {
                Err(CudaError::StreamCaptureImplicit)
            },
            cudaError_enum::CUDA_ERROR_CAPTURED_EVENT => Err(CudaError::CapturedEvent),
            cudaError_enum::CUDA_ERROR_STREAM_CAPTURE_WRONG_THREAD => {
                Err(CudaError::StreamCaptureWrongThread)
            },
            cudaError_enum::CUDA_ERROR_TIMEOUT => Err(CudaError::Timeout),
            cudaError_enum::CUDA_ERROR_GRAPH_EXEC_UPDATE_FAILURE => {
                Err(CudaError::GraphExecUpdateFailure)
            },
            cudaError_enum::CUDA_ERROR_UNKNOWN => Err(CudaError::Unknown),
        }
    }
}
