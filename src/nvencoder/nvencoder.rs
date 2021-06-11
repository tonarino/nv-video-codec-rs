use nv_video_codec_sys::{
    NvEncodeAPICreateInstance, NvEncodeAPIGetMaxSupportedVersion, NVENCAPI_MAJOR_VERSION,
    NVENCAPI_MINOR_VERSION, NVENCSTATUS, NV_ENCODE_API_FUNCTION_LIST, NV_ENC_BUFFER_FORMAT,
    NV_ENC_CONFIG, NV_ENC_DEVICE_TYPE, NV_ENC_INITIALIZE_PARAMS, NV_ENC_INPUT_PTR,
    NV_ENC_INPUT_RESOURCE_TYPE, NV_ENC_OUTPUT_PTR, NV_ENC_REGISTERED_PTR,
};
use std::mem::MaybeUninit;

const NVENCAPI_VERSION: u32 = NVENCAPI_MAJOR_VERSION | (NVENCAPI_MINOR_VERSION << 24);

const fn nvenc_api_struct_version(version: u32) -> u32 {
    NVENCAPI_VERSION | ((version) << 16) | (0x7 << 28)
}

const NV_ENCODE_API_FUNCTION_LIST_VERSION: u32 = nvenc_api_struct_version(2);

pub enum Error {
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

trait IntoResult {
    fn into_result(self) -> Result<(), Error>;
}

impl IntoResult for NVENCSTATUS {
    fn into_result(self) -> Result<(), Error> {
        match self {
            NVENCSTATUS::NV_ENC_SUCCESS => Ok(()),
            NVENCSTATUS::NV_ENC_ERR_NO_ENCODE_DEVICE => Err(Error::NoEncodeDevice),
            NVENCSTATUS::NV_ENC_ERR_UNSUPPORTED_DEVICE => Err(Error::UnsupportedDevice),
            NVENCSTATUS::NV_ENC_ERR_INVALID_ENCODERDEVICE => Err(Error::InvalidEncoderDevice),
            NVENCSTATUS::NV_ENC_ERR_INVALID_DEVICE => Err(Error::InalidDevice),
            NVENCSTATUS::NV_ENC_ERR_DEVICE_NOT_EXIST => Err(Error::DeviceNoLongerExists),
            NVENCSTATUS::NV_ENC_ERR_INVALID_PTR => Err(Error::InvalidPointer),
            NVENCSTATUS::NV_ENC_ERR_INVALID_EVENT => Err(Error::InvalidEvent),
            NVENCSTATUS::NV_ENC_ERR_INVALID_PARAM => Err(Error::InvalidParam),
            NVENCSTATUS::NV_ENC_ERR_INVALID_CALL => Err(Error::InvalidCall),
            NVENCSTATUS::NV_ENC_ERR_OUT_OF_MEMORY => Err(Error::OutOfMemory),
            NVENCSTATUS::NV_ENC_ERR_ENCODER_NOT_INITIALIZED => Err(Error::EncoderNotInitialized),
            NVENCSTATUS::NV_ENC_ERR_UNSUPPORTED_PARAM => Err(Error::UnsupportedParam),
            NVENCSTATUS::NV_ENC_ERR_LOCK_BUSY => Err(Error::LockBusy),
            NVENCSTATUS::NV_ENC_ERR_NOT_ENOUGH_BUFFER => Err(Error::NotEnoughBuffer),
            NVENCSTATUS::NV_ENC_ERR_INVALID_VERSION => Err(Error::InvalidVersion),
            NVENCSTATUS::NV_ENC_ERR_MAP_FAILED => Err(Error::MapFailed),
            NVENCSTATUS::NV_ENC_ERR_NEED_MORE_INPUT => Err(Error::NeedMoreInput),
            NVENCSTATUS::NV_ENC_ERR_ENCODER_BUSY => Err(Error::EncoderBusy),
            NVENCSTATUS::NV_ENC_ERR_EVENT_NOT_REGISTERD => Err(Error::EventNotRegistered),
            NVENCSTATUS::NV_ENC_ERR_GENERIC => Err(Error::ErrGeneric),
            NVENCSTATUS::NV_ENC_ERR_INCOMPATIBLE_CLIENT_KEY => Err(Error::IncompatibleClientKey),
            NVENCSTATUS::NV_ENC_ERR_UNIMPLEMENTED => Err(Error::Unimplemented),
            NVENCSTATUS::NV_ENC_ERR_RESOURCE_REGISTER_FAILED => Err(Error::ResourceRegisterFailed),
            NVENCSTATUS::NV_ENC_ERR_RESOURCE_NOT_REGISTERED => Err(Error::ResourceNotRegistered),
            NVENCSTATUS::NV_ENC_ERR_RESOURCE_NOT_MAPPED => Err(Error::ResourceNotMapped),
        }
    }
}

#[repr(C)]
struct EncoderHandle {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
struct CompletionEvent {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
struct Device {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
struct Input {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

pub struct NvEncoder {
    motion_estimation_only: bool,
    output_in_video_memory: bool,
    encoder_handle: *mut EncoderHandle, // Originally a void pointer
    nv_encode_api_function_list: NV_ENCODE_API_FUNCTION_LIST,
    input_frames: Vec<NvEncInputFrame>,
    registered_resources: Vec<NV_ENC_REGISTERED_PTR>,
    reference_frames: Vec<NvEncInputFrame>,
    registered_resources_for_reference: Vec<NV_ENC_REGISTERED_PTR>,
    mapped_input_buffers: Vec<NV_ENC_INPUT_PTR>,
    mapped_ref_buffers: Vec<NV_ENC_INPUT_PTR>,
    completion_event: Vec<*mut CompletionEvent>, // Originally a void pointer
    to_send: i32,
    got: i32,
    encoder_buffer: i32,
    output_delay: i32,
    width: u32,
    height: u32,
    buffer_format: NV_ENC_BUFFER_FORMAT,
    device: *mut Device, // Originally a void pointer
    device_type: NV_ENC_DEVICE_TYPE,
    initialize_params: NV_ENC_INITIALIZE_PARAMS,
    encode_config: NV_ENC_CONFIG,
    encoder_initialized: bool,
    extra_output_delay: u32,
    bitstream_output_buffer: Vec<NV_ENC_OUTPUT_PTR>,
    motion_vector_data_output_buffer: Vec<NV_ENC_OUTPUT_PTR>,
    max_encode_width: u32,
    max_encode_height: u32,
}

impl NvEncoder {
    pub fn new() -> Self {
        Self::load_nv_enc_api();
    }

    pub fn create_encoder() {
        unimplemented!()
    }

    pub fn destroy_encoder() {
        unimplemented!()
    }

    pub fn reconfigure() -> bool {
        unimplemented!()
    }

    pub fn get_next_input_frame() {}

    pub fn encode_frame() {
        unimplemented!()
    }

    pub fn end_encode() {
        unimplemented!()
    }

    pub fn get_capability_value() {
        unimplemented!()
    }

    pub fn get_device() {
        unimplemented!()
    }

    pub fn get_device_type() {
        unimplemented!()
    }

    pub fn get_encode_width() {
        unimplemented!()
    }

    pub fn get_encode_height() {
        unimplemented!()
    }

    pub fn get_frame_size() {
        unimplemented!()
    }

    pub fn create_default_encoder_params() {
        unimplemented!()
    }

    pub fn get_initialize_params() {
        unimplemented!()
    }

    pub fn run_motion_estimation() {
        unimplemented!()
    }

    pub fn get_next_reference_frame() {
        unimplemented!()
    }

    pub fn get_sequence_params() {
        unimplemented!()
    }

    // Protected

    fn is_hw_encoder_initialized() {
        unimplemented!()
    }

    fn register_input_resources() {
        unimplemented!()
    }

    fn unregister_input_resources() {
        unimplemented!()
    }

    fn register_resource() {
        unimplemented!()
    }

    fn get_max_encode_width() -> u32 {
        unimplemented!()
    }

    fn get_max_encode_height() -> u32 {
        unimplemented!()
    }

    fn get_completion_event() {
        unimplemented!()
    }

    fn get_pixel_format() {
        unimplemented!()
    }

    fn do_encode() {
        unimplemented!()
    }

    fn do_motion_estimation() {
        unimplemented!()
    }

    fn map_resources() {
        unimplemented!()
    }

    fn wait_for_completion_event() {
        unimplemented!()
    }

    fn send_eos() {
        unimplemented!()
    }

    // Private
    fn is_zero_delay() -> bool {
        unimplemented!()
    }

    fn load_nv_enc_api() -> Result<NV_ENCODE_API_FUNCTION_LIST, Error> {
        let version = 0u32;
        let current_version = (NVENCAPI_MAJOR_VERSION << 4) | NVENCAPI_MINOR_VERSION;
        unsafe {
            NvEncodeAPIGetMaxSupportedVersion(&mut version as *mut _).into_result()?;
        }

        if current_version > version {
            return Err(Error::InvalidVersion);
        }

        let mut nvenc_api = MaybeUninit::<NV_ENCODE_API_FUNCTION_LIST>::uninit();
        nvenc_api.as_mut_ptr().write(NV_ENCODE_API_FUNCTION_LIST_VERSION);

        unsafe {
            NvEncodeAPICreateInstance(nvenc_api.as_mut_ptr()).into_result()?;
        }

        Ok(nvenc_api.assume_init())
    }

    fn get_encoded_packet() {
        unimplemented!()
    }

    fn initialize_bitstream_buffer() {
        unimplemented!()
    }

    fn destroy_bitstream_buffer() {
        unimplemented!()
    }

    fn initialize_mv_output_buffer() {
        unimplemented!()
    }

    fn destroy_mv_output_buffer() {
        unimplemented!()
    }

    fn destroy_hw_encoder() {
        unimplemented!()
    }

    fn flush_encoder() {
        unimplemented!()
    }
}

// Static functions
impl NvEncoder {
    pub fn get_chroma_sub_plane_offsets() {
        unimplemented!()
    }

    pub fn get_chroma_pitch() {
        unimplemented!()
    }

    pub fn get_num_chroma_planes() {
        unimplemented!()
    }

    pub fn get_chroma_width_in_bytes() {
        unimplemented!()
    }

    pub fn get_chroma_height() {
        unimplemented!()
    }

    pub fn get_width_in_bytes() {
        unimplemented!()
    }

    pub fn get_encoder_buffer_count() {
        unimplemented!()
    }
}

impl Drop for NvEncoder {
    fn drop(&mut self) {}
}

struct NvEncInputFrame {
    input_ptr: *mut Input, // Originally a void pointer
    chroma_offsets: [u32; 2],
    num_chroma_planes: u32,
    pitch: u32,
    chroma_pitch: u32,
    buffer_format: NV_ENC_BUFFER_FORMAT,
    resource_type: NV_ENC_INPUT_RESOURCE_TYPE,
}
