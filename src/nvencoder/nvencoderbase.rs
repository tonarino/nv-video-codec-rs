use std::marker::PhantomData;

use nv_video_codec_sys::{
    NvEncodeAPICreateInstance, NvEncodeAPIGetMaxSupportedVersion, GUID, NVENCAPI_MAJOR_VERSION,
    NVENCAPI_MINOR_VERSION, NVENCAPI_VERSION, NVENCSTATUS, NV_ENCODE_API_FUNCTION_LIST,
    NV_ENC_BUFFER_FORMAT, NV_ENC_CODEC_H264_GUID, NV_ENC_CODEC_HEVC_GUID, NV_ENC_CONFIG,
    NV_ENC_DEVICE_TYPE, NV_ENC_INITIALIZE_PARAMS, NV_ENC_INPUT_PTR,
    NV_ENC_INPUT_RESOURCE_OPENGL_TEX, NV_ENC_INPUT_RESOURCE_TYPE,
    NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS, NV_ENC_OUTPUT_PTR, NV_ENC_PARAMS_RC_MODE,
    NV_ENC_PRESET_CONFIG, NV_ENC_REGISTERED_PTR, NV_ENC_TUNING_INFO, _NV_ENC_QP,
};

use super::{
    resource_manager::NvEncoderResourceManager, BufferFormat, IntoNvEncResult, NvEncError,
    NvEncoderError,
};

const fn nvenc_api_struct_version(version: u32) -> u32 {
    NVENCAPI_VERSION | ((version) << 16) | (0x7 << 28)
}

const NV_ENCODE_API_FUNCTION_LIST_VERSION: u32 = nvenc_api_struct_version(2);
const NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS_VER: u32 = nvenc_api_struct_version(1);
const NV_ENC_INITIALIZE_PARAMS_VER: u32 = nvenc_api_struct_version(5) | (1 << 31);
const NV_ENC_CONFIG_VER: u32 = nvenc_api_struct_version(7) | (1 << 31);
const NV_ENC_PRESET_CONFIG_VER: u32 = nvenc_api_struct_version(4) | (1 << 31);

#[repr(C)]
pub(super) struct EncoderHandle {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
struct CompletionEvent {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

// TODO(bschwind) - Obtain this device pointer internally, with a call to cuCtxGetCurrent()
#[repr(C)]
pub struct Device {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
pub(super) struct Input {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

pub(super) struct NvEncoderBase<ResourceManager>
where
    ResourceManager: NvEncoderResourceManager + ?Sized,
{
    pub(super) motion_estimation_only: bool,
    output_in_video_memory: bool,
    pub(super) encoder_handle: *mut EncoderHandle, // Originally a void pointer
    nv_encode_api_function_list: NV_ENCODE_API_FUNCTION_LIST,
    pub(super) input_frames: Vec<NvEncInputFrame>,
    pub(super) registered_resources: Vec<NV_ENC_REGISTERED_PTR>,
    pub(super) reference_frames: Vec<NvEncInputFrame>,
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
    buffer_format: BufferFormat,
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
    _resource_manager: PhantomData<ResourceManager>,
}

impl<ResourceManager> NvEncoderBase<ResourceManager>
where
    ResourceManager: NvEncoderResourceManager + ?Sized,
{
    pub fn new(
        device_type: NV_ENC_DEVICE_TYPE,
        device: *mut Device,
        width: u32,
        height: u32,
        buffer_format: BufferFormat,
        extra_output_delay: u32,
        motion_estimation_only: bool,
        output_in_video_memory: bool,
    ) -> Result<Self, NvEncoderError> {
        let enc_api = Self::load_nv_enc_api()?;

        if enc_api.nvEncOpenEncodeSession.is_none() {
            return Err(NvEncError::NoEncodeDevice.into());
        }

        let mut encode_session_ex_params = NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS {
            version: NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS_VER,
            device: device as *mut std::os::raw::c_void,
            deviceType: device_type,
            apiVersion: NVENCAPI_VERSION,
            ..NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS::default()
        };

        let mut encoder_handle: *mut EncoderHandle = std::ptr::null_mut();
        let encoder_handle_ptr: *mut *mut EncoderHandle = &mut encoder_handle;

        unsafe {
            enc_api.nvEncOpenEncodeSessionEx.unwrap()(
                &mut encode_session_ex_params as *mut _,
                encoder_handle_ptr as *mut _,
            )
            .into_nvenc_result()?;
        }

        Ok(Self {
            motion_estimation_only,
            output_in_video_memory,
            encoder_handle,
            nv_encode_api_function_list: enc_api,
            input_frames: Vec::new(),
            registered_resources: Vec::new(),
            reference_frames: Vec::new(),
            registered_resources_for_reference: Vec::new(),
            mapped_input_buffers: Vec::new(),
            mapped_ref_buffers: Vec::new(),
            completion_event: Vec::new(),
            to_send: 0,
            got: 0,
            encoder_buffer: 0,
            output_delay: 0,
            width,
            height,
            buffer_format,
            device,
            device_type,
            initialize_params: NV_ENC_INITIALIZE_PARAMS::default(),
            encode_config: NV_ENC_CONFIG::default(),
            encoder_initialized: false,
            extra_output_delay,
            bitstream_output_buffer: Vec::new(),
            motion_vector_data_output_buffer: Vec::new(),
            max_encode_width: width,
            max_encode_height: height,
            _resource_manager: PhantomData,
        })
    }

    /// This function is used to initialize the encoder session.
    /// Application must call this function to initialize the encoder, before
    /// starting to encode any frames.
    pub fn create_encoder(
        &mut self,
        encoder_params: &NV_ENC_INITIALIZE_PARAMS,
    ) -> Result<(), NvEncoderError> {
        if self.encoder_handle.is_null() {
            return Err(NvEncError::NoEncodeDevice.into());
        }

        if encoder_params.encodeWidth == 0 || encoder_params.encodeHeight == 0 {
            return Err(NvEncError::InvalidParam.into());
        }

        unsafe {
            if encoder_params.encodeGUID != NV_ENC_CODEC_H264_GUID
                && encoder_params.encodeGUID != NV_ENC_CODEC_HEVC_GUID
            {
                return Err(NvEncError::InvalidParam.into());
            }

            if encoder_params.encodeGUID == NV_ENC_CODEC_H264_GUID
                && matches!(
                    self.buffer_format,
                    BufferFormat::YUV420_10BIT | BufferFormat::YUV444_10BIT
                )
            {
                return Err(NvEncError::InvalidParam.into());
            }

            if encoder_params.encodeGUID == NV_ENC_CODEC_H264_GUID
                && matches!(self.buffer_format, BufferFormat::YUV444)
                && (*encoder_params.encodeConfig).encodeCodecConfig.h264Config.chromaFormatIDC != 3
            {
                return Err(NvEncError::InvalidParam.into());
            }

            if encoder_params.encodeGUID == NV_ENC_CODEC_HEVC_GUID {
                let yuv10_bit_format = matches!(
                    self.buffer_format,
                    BufferFormat::YUV420_10BIT | BufferFormat::YUV444_10BIT
                );
                if yuv10_bit_format
                    && (*encoder_params.encodeConfig)
                        .encodeCodecConfig
                        .hevcConfig
                        .pixelBitDepthMinus8()
                        != 2
                {
                    return Err(NvEncError::InvalidParam.into());
                }

                if matches!(self.buffer_format, BufferFormat::YUV444 | BufferFormat::YUV444_10BIT)
                    && (*encoder_params.encodeConfig).encodeCodecConfig.hevcConfig.chromaFormatIDC()
                        != 3
                {
                    return Err(NvEncError::InvalidParam.into());
                }
            }
        }

        self.initialize_params = encoder_params.clone();
        self.initialize_params.version = NV_ENC_INITIALIZE_PARAMS_VER;

        if !encoder_params.encodeConfig.is_null() {
            unsafe {
                self.encode_config = (*encoder_params.encodeConfig).clone();
            }
            self.encode_config.version = NV_ENC_CONFIG_VER;
        } else {
            let mut preset_config = NV_ENC_PRESET_CONFIG {
                version: NV_ENC_PRESET_CONFIG_VER,
                presetCfg: NV_ENC_CONFIG { version: NV_ENC_CONFIG_VER, ..Default::default() },
                ..Default::default()
            };
            if !self.motion_estimation_only {
                unsafe {
                    self.nv_encode_api_function_list.nvEncGetEncodePresetConfigEx.unwrap()(
                        self.encoder_handle as *mut _,
                        encoder_params.encodeGUID,
                        encoder_params.presetGUID,
                        encoder_params.tuningInfo,
                        &mut preset_config,
                    )
                    .into_nvenc_result()?;
                }
                self.encode_config = preset_config.presetCfg.clone();
            } else {
                self.encode_config.version = NV_ENC_CONFIG_VER;
                self.encode_config.rcParams.rateControlMode =
                    NV_ENC_PARAMS_RC_MODE::NV_ENC_PARAMS_RC_CONSTQP;
                self.encode_config.rcParams.constQP =
                    _NV_ENC_QP { qpInterP: 28, qpInterB: 31, qpIntra: 25 };
            }
        }
        self.initialize_params.encodeConfig = &mut self.encode_config;

        unsafe {
            self.nv_encode_api_function_list.nvEncInitializeEncoder.unwrap()(
                self.encoder_handle as *mut _,
                &mut self.initialize_params,
            )
            .into_nvenc_result()?;
        }

        self.encoder_initialized = true;
        self.width = self.initialize_params.encodeWidth;
        self.height = self.initialize_params.encodeHeight;
        self.max_encode_width = self.initialize_params.maxEncodeWidth;
        self.max_encode_height = self.initialize_params.maxEncodeHeight;

        // TODO(efyang): convert this to a usize
        self.encoder_buffer = self.encode_config.frameIntervalP
            + self.encode_config.rcParams.lookaheadDepth as i32
            + self.extra_output_delay as i32;
        self.output_delay = self.encoder_buffer - 1;
        self.mapped_input_buffers.resize(self.encoder_buffer as usize, std::ptr::null_mut());

        if !self.output_in_video_memory {
            self.completion_event.resize(self.encoder_buffer as usize, std::ptr::null_mut());
        }

        if self.motion_estimation_only {
            self.mapped_ref_buffers.resize(self.encoder_buffer as usize, std::ptr::null_mut());
            if !self.output_in_video_memory {
                self.initialize_mv_output_buffer();
            }
        } else if !self.output_in_video_memory {
            self.bitstream_output_buffer.resize(self.encoder_buffer as usize, std::ptr::null_mut());
            self.initialize_bitstream_buffer();
        }

        ResourceManager::allocate_input_buffers(self, self.encoder_buffer as u32)?;
        Ok(())
    }

    /// This function is used to destroy the encoder session.
    /// Application must call this function to destroy the encoder session and
    /// clean up any allocated resources. The application must call EndEncode()
    /// function to get any queued encoded frames before calling DestroyEncoder().
    pub fn destroy_encoder(&mut self) -> Result<(), NvEncoderError> {
        if self.encoder_handle.is_null() {
            return Ok(());
        }

        ResourceManager::release_input_buffers(self)?;
        self.destroy_hw_encoder();
        Ok(())
    }

    pub fn reconfigure() -> bool {
        unimplemented!()
    }

    /// This function is used to get the next available input buffer.
    /// Applications must call this function to obtain a pointer to the next
    /// input buffer. The application must copy the uncompressed data to the
    /// input buffer and then call EncodeFrame() function to encode it.
    pub fn get_next_input_frame(&mut self) -> &NvEncInputFrame {
        // TODO(efyang): make this return value lifetime'd
        &self.input_frames[(self.to_send % self.encoder_buffer) as usize]
    }

    /// This function is used to encode a frame.
    /// Applications must call EncodeFrame() function to encode the uncompressed
    /// data, which has been copied to an input buffer obtained from the
    /// GetNextInputFrame() function.
    pub fn encode_frame(&mut self) {
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

    pub fn create_default_encoder_params(
        &self,
        codec_guid: GUID,
        preset_guid: GUID,
        tuning_info: NV_ENC_TUNING_INFO,
    ) -> Result<NV_ENC_INITIALIZE_PARAMS, NvEncoderError> {
        if self.encoder_handle.is_null() {
            return Err(NvEncError::NoEncodeDevice.into());
        }

        let mut encode_config =
            NV_ENC_CONFIG { version: (7 << 16 | 0x7 << 28 | 1 << 31), ..Default::default() };

        // nvpipe doesn't even use this
        todo!()
        // Ok(NV_ENC_INITIALIZE_PARAMS {
        //     version: (5 << 16 | 0x7 << 28 | 1 << 31), // TODO(efyang) actual const func for this
        //     encodeGUID: codec_guid,
        //     presetGUID: preset_guid,
        //     encodeWidth: self.width,
        //     encodeHeight: self.height,
        //     darWidth: self.width,
        //     darHeight: self.height,
        //     frameRateNum: 30, // TODO(efyang): possible optimization?
        //     frameRateDen: 1,
        //     enablePTD: 1,
        //     encodeConfig: &mut encode_config,
        //     maxEncodeWidth: self.width,
        //     maxEncodeHeight: self.height,
        //     _bitfield_1: NV_ENC_INITIALIZE_PARAMS::new_bitfield_1(
        //         0,
        //         0,
        //         0,
        //         self.motion_estimation_only as u32,
        //         0,
        //         self.output_in_video_memory as u32,
        //         0,
        //     ),
        //     ..Default::default()
        // })
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

    pub(super) fn is_hw_encoder_initialized(&mut self) -> bool {
        unimplemented!()
    }

    pub(super) fn register_input_resources(
        &mut self,
        input_frames: &[NV_ENC_INPUT_RESOURCE_OPENGL_TEX],
        resource_type: NV_ENC_INPUT_RESOURCE_TYPE,
        width: u32,
        height: u32,
        pitch: u32,
        buffer_format: BufferFormat,
        reference_frame: bool,
    ) {
        unimplemented!()
    }

    pub(super) fn unregister_input_resources(&mut self) {
        unimplemented!()
    }

    fn register_resource() {
        unimplemented!()
    }

    pub fn get_max_encode_width(&self) -> u32 {
        self.max_encode_width
    }

    pub fn get_max_encode_height(&self) -> u32 {
        self.max_encode_height
    }

    fn get_completion_event() {
        unimplemented!()
    }

    pub fn get_pixel_format(&self) -> BufferFormat {
        self.buffer_format
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

    fn load_nv_enc_api() -> Result<NV_ENCODE_API_FUNCTION_LIST, NvEncoderError> {
        let mut version = 0u32;
        let current_version = (NVENCAPI_MAJOR_VERSION << 4) | NVENCAPI_MINOR_VERSION;
        unsafe {
            NvEncodeAPIGetMaxSupportedVersion(&mut version as *mut _).into_nvenc_result()?;
        }

        if current_version > version {
            return Err(NvEncError::InvalidVersion.into());
        }

        let mut nvenc_api = NV_ENCODE_API_FUNCTION_LIST {
            version: NV_ENCODE_API_FUNCTION_LIST_VERSION,
            ..NV_ENCODE_API_FUNCTION_LIST::default()
        };

        unsafe {
            NvEncodeAPICreateInstance(&mut nvenc_api as *mut _).into_nvenc_result()?;
        }

        Ok(nvenc_api)
    }

    fn get_encoded_packet() {
        unimplemented!()
    }

    fn initialize_bitstream_buffer(&mut self) {
        unimplemented!()
    }

    fn destroy_bitstream_buffer(&mut self) {
        unimplemented!()
    }

    fn initialize_mv_output_buffer(&mut self) {
        unimplemented!()
    }

    fn destroy_mv_output_buffer(&mut self) {
        unimplemented!()
    }

    fn destroy_hw_encoder(&mut self) {
        unimplemented!()
    }

    fn flush_encoder() {
        unimplemented!()
    }

    pub fn get_encoder_buffer_count() {
        unimplemented!()
    }
}

impl<ResourceManager> Drop for NvEncoderBase<ResourceManager>
where
    ResourceManager: NvEncoderResourceManager + ?Sized,
{
    fn drop(&mut self) {
        ResourceManager::release_input_buffers(self).unwrap();
        self.destroy_hw_encoder();
    }
}

pub(super) struct NvEncInputFrame {
    pub(super) input_ptr: *mut Input, // Originally a void pointer
    chroma_offsets: [u32; 2],
    num_chroma_planes: u32,
    pitch: u32,
    chroma_pitch: u32,
    buffer_format: BufferFormat,
    resource_type: NV_ENC_INPUT_RESOURCE_TYPE,
}
