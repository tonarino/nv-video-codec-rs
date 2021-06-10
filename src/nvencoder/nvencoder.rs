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

pub struct NvEncoder {
    motion_estimation_only: bool,
    output_in_video_memory: bool,
    // encoder_handler: *mut void,
    // nv_encode_api_function_list: NV_ENCODE_API_FUNCTION_LIST,
    input_frames: Vec<NvEncInputFrame>,
    // registered_resources: Vec<NV_ENC_REGISTERED_PTR>,
    reference_frames: Vec<NvEncInputFrame>,
    // registered_resources_for_reference: Vec<NV_ENC_REGISTERED_PTR>,
    // mapped_input_buffers: Vec<NV_ENC_INPUT_PTR>,
    // mapped_ref_buffers: Vec<NV_ENC_INPUT_PTR>,
    // completion_event: Vec<*mut void>,
    to_send: i32,
    got: i32,
    encoder_buffer: i32,
    output_delay: i32,
    width: u32,
    height: u32,
    // buffer_format: NV_ENC_BUFFER_FORMAT,
    // device: *mut void,
    // device_type: NV_ENC_DEVICE_TYPE,
    // initialize_params: NV_ENC_INITIALIZE_PARAMS,
    // encode_config: NV_ENC_CONFIG,
    encoder_initialized: bool,
    extra_output_delay: u32,
    // bitstream_output_buffer: Vec<NV_ENC_OUTPUT_PTR>,
    // motion_vector_data_output_buffer: Vec<NV_ENC_OUTPUT_PTR>,
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

    fn load_nv_enc_api() -> NV_ENCODE_API_FUNCTION_LIST {
        unimplemented!()
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
    // input_ptr: *mut void,
    chroma_offsets: [u32; 2],
    num_chroma_planes: u32,
    pitch: u32,
    chroma_pitch: u32,
    // buffer_format: NV_ENC_BUFFER_FORMAT,
    // resource_type: NV_ENC_INPUT_RESOURCE_TYPE,
}
