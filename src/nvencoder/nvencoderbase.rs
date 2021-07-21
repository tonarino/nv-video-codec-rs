#![allow(unused_variables, dead_code)]
use std::marker::PhantomData;

use nv_video_codec_sys::{
    NvEncodeAPICreateInstance, NvEncodeAPIGetMaxSupportedVersion, GUID, NVENCAPI_MAJOR_VERSION,
    NVENCAPI_MINOR_VERSION, NVENCAPI_VERSION, NVENC_INFINITE_GOPLENGTH,
    NV_ENCODE_API_FUNCTION_LIST, NV_ENC_BUFFER_USAGE, NV_ENC_CAPS, NV_ENC_CAPS_PARAM,
    NV_ENC_CODEC_H264_GUID, NV_ENC_CODEC_HEVC_GUID, NV_ENC_CONFIG, NV_ENC_CREATE_BITSTREAM_BUFFER,
    NV_ENC_CREATE_MV_BUFFER, NV_ENC_DEVICE_TYPE, NV_ENC_INITIALIZE_PARAMS, NV_ENC_INPUT_PTR,
    NV_ENC_INPUT_RESOURCE_OPENGL_TEX, NV_ENC_INPUT_RESOURCE_TYPE, NV_ENC_LOCK_BITSTREAM,
    NV_ENC_MAP_INPUT_RESOURCE, NV_ENC_MEONLY_PARAMS, NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS,
    NV_ENC_OUTPUT_PTR, NV_ENC_PARAMS_RC_MODE, NV_ENC_PIC_PARAMS, NV_ENC_PRESET_CONFIG, NV_ENC_QP,
    NV_ENC_REGISTERED_PTR, NV_ENC_REGISTER_RESOURCE, NV_ENC_TUNING_INFO, _NV_ENC_PIC_FLAGS,
    _NV_ENC_PIC_STRUCT, _NV_ENC_QP,
};

use super::{
    resource_manager::NvEncoderResourceManager, BufferFormat, IntoNvEncResult, NvEncError,
    NvEncoder, NvEncoderError, NvEncoderResult,
};

const fn nvenc_api_struct_version(version: u32) -> u32 {
    NVENCAPI_VERSION | ((version) << 16) | (0x7 << 28)
}

const NV_ENCODE_API_FUNCTION_LIST_VERSION: u32 = nvenc_api_struct_version(2);
const NV_ENC_OPEN_ENCODE_SESSION_EX_PARAMS_VER: u32 = nvenc_api_struct_version(1);
const NV_ENC_INITIALIZE_PARAMS_VER: u32 = nvenc_api_struct_version(5) | (1 << 31);
const NV_ENC_CONFIG_VER: u32 = nvenc_api_struct_version(7) | (1 << 31);
const NV_ENC_PRESET_CONFIG_VER: u32 = nvenc_api_struct_version(4) | (1 << 31);
const NV_ENC_CAPS_PARAM_VER: u32 = nvenc_api_struct_version(1);
const NV_ENC_PIC_PARAMS_VER: u32 = nvenc_api_struct_version(4) | (1 << 31);
const NV_ENC_LOCK_BITSTREAM_VER: u32 = nvenc_api_struct_version(1);
const NV_ENC_CREATE_BITSTREAM_BUFFER_VER: u32 = nvenc_api_struct_version(1);
const NV_ENC_CREATE_MV_BUFFER_VER: u32 = nvenc_api_struct_version(1);
const NV_ENC_MAP_INPUT_RESOURCE_VER: u32 = nvenc_api_struct_version(4);

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

impl<ResourceManager> NvEncoder for NvEncoderBase<ResourceManager>
where
    ResourceManager: NvEncoderResourceManager + ?Sized,
{
    fn create_encoder(&mut self, encoder_params: &NV_ENC_INITIALIZE_PARAMS) -> NvEncoderResult<()> {
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
                self.initialize_mv_output_buffer()?;
            }
        } else if !self.output_in_video_memory {
            self.bitstream_output_buffer.resize(self.encoder_buffer as usize, std::ptr::null_mut());
            self.initialize_bitstream_buffer()?;
        }

        ResourceManager::allocate_input_buffers(self, self.encoder_buffer as u32)?;
        Ok(())
    }

    fn destroy_encoder(&mut self) -> NvEncoderResult<()> {
        if self.encoder_handle.is_null() {
            return Ok(());
        }

        ResourceManager::release_input_buffers(self)?;
        self.destroy_hw_encoder()?;
        Ok(())
    }

    // not implementing for now
    // pub fn reconfigure() -> bool {
    //     unimplemented!()
    // }

    // TODO: make this (and get_next_reference_frame) optional
    fn get_next_input_frame(&mut self) -> &NvEncInputFrame {
        // TODO(efyang): make this return value lifetime'd
        &self.input_frames[(self.to_send % self.encoder_buffer) as usize]
    }

    fn encode_frame(
        &mut self,
        packet: &mut Vec<Vec<u8>>,
        pic_params: Option<NV_ENC_PIC_PARAMS>,
    ) -> NvEncoderResult<()> {
        packet.clear();
        if !self.is_hw_encoder_initialized() {
            return Err(NvEncError::NoEncodeDevice.into());
        }

        let buffer_index = (self.to_send % self.encoder_buffer) as u32;
        self.map_resources(buffer_index)?;

        let encode_status = self.do_encode(
            self.mapped_input_buffers[buffer_index as usize],
            self.bitstream_output_buffer[buffer_index as usize],
            pic_params,
        );

        match encode_status {
            Ok(_) | Err(NvEncoderError::NvEncError(NvEncError::NeedMoreInput)) => {
                self.to_send += 1;
                self.get_encoded_packet(packet, true)?;
            },
            _ => {
                encode_status?;
            },
        }
        unimplemented!()
    }

    fn end_encode(&mut self, packet: &mut Vec<Vec<u8>>) -> NvEncoderResult<()> {
        packet.clear();
        if !self.is_hw_encoder_initialized() {
            return Err(NvEncError::EncoderNotInitialized.into());
        }
        self.send_eos()?;

        self.get_encoded_packet(packet, false)?;
        unimplemented!()
    }

    fn get_capability_value(
        &mut self,
        codec_guid: GUID,
        caps_to_query: NV_ENC_CAPS,
    ) -> NvEncoderResult<(NV_ENC_CAPS, i32)> {
        // TODO (efyang): make this return better
        if self.encoder_handle.is_null() {
            return Err(NvEncError::EncoderNotInitialized.into());
        }
        let mut caps_param = NV_ENC_CAPS_PARAM {
            version: NV_ENC_CAPS_PARAM_VER,
            capsToQuery: caps_to_query,
            ..Default::default()
        };

        let mut v = 0;
        unsafe {
            self.nv_encode_api_function_list.nvEncGetEncodeCaps.unwrap()(
                self.encoder_handle as *mut _,
                codec_guid,
                &mut caps_param,
                &mut v,
            )
            .into_nvenc_result()?;
        }
        Ok((caps_param.capsToQuery, v))
    }

    fn get_device(&self) -> Option<&Device> {
        unsafe { self.device.as_ref() }
    }

    fn get_device_type(&self) -> NV_ENC_DEVICE_TYPE {
        self.device_type
    }

    fn get_encode_width(&self) -> u32 {
        self.width
    }

    fn get_encode_height(&self) -> u32 {
        self.height
    }

    fn get_frame_size(&self) -> NvEncoderResult<u32> {
        match self.get_pixel_format() {
            BufferFormat::YV12 | BufferFormat::IYUV | BufferFormat::NV12 => Ok(self
                .get_encode_width()
                * (self.get_encode_height() + (self.get_encode_height() + 1) / 2)),
            BufferFormat::YUV420_10BIT => Ok(2
                * self.get_encode_width()
                * (self.get_encode_height() + (self.get_encode_height() + 1) / 2)),
            BufferFormat::YUV444 => Ok(self.get_encode_width() * self.get_encode_height() * 3),
            BufferFormat::YUV444_10BIT => {
                Ok(2 * self.get_encode_width() * self.get_encode_height() * 3)
            },
            BufferFormat::ARGB
            | BufferFormat::ARGB10
            | BufferFormat::AYUV
            | BufferFormat::ABGR
            | BufferFormat::ABGR10 => Ok(4 * self.get_encode_height() * self.get_encode_width()),
            _ => Err(NvEncError::InvalidParam.into()),
        }
    }

    fn create_default_encoder_params(
        &mut self,
        codec_guid: GUID,
        preset_guid: GUID,
        tuning_info: NV_ENC_TUNING_INFO,
    ) -> NvEncoderResult<NV_ENC_INITIALIZE_PARAMS> {
        if self.encoder_handle.is_null() {
            return Err(NvEncError::NoEncodeDevice.into());
        }

        let mut encode_config = NV_ENC_CONFIG { version: NV_ENC_CONFIG_VER, ..Default::default() };

        // nvpipe doesn't even use this
        let mut initialize_params = NV_ENC_INITIALIZE_PARAMS {
            version: NV_ENC_INITIALIZE_PARAMS_VER, // TODO(efyang) actual const func for this
            encodeGUID: codec_guid,
            presetGUID: preset_guid,
            encodeWidth: self.width,
            encodeHeight: self.height,
            darWidth: self.width,
            darHeight: self.height,
            frameRateNum: 30, // TODO(efyang): possible optimization?
            frameRateDen: 1,
            enablePTD: 1,
            encodeConfig: &mut encode_config,
            maxEncodeWidth: self.width,
            maxEncodeHeight: self.height,
            ..Default::default()
        };

        initialize_params.set_enableMEOnlyMode(self.motion_estimation_only as u32);
        initialize_params.set_enableOutputInVidmem(self.output_in_video_memory as u32);

        let mut preset_config = NV_ENC_PRESET_CONFIG {
            version: NV_ENC_PRESET_CONFIG_VER,
            presetCfg: NV_ENC_CONFIG { version: NV_ENC_CONFIG_VER, ..Default::default() },
            ..Default::default()
        };
        unsafe {
            self.nv_encode_api_function_list.nvEncGetEncodePresetConfig.unwrap()(
                self.encoder_handle as *mut _,
                codec_guid,
                preset_guid,
                &mut preset_config,
            )
            .into_nvenc_result()?;
        }

        unsafe {
            *initialize_params.encodeConfig = preset_config.presetCfg.clone();
            (*initialize_params.encodeConfig).frameIntervalP = 1;
            (*initialize_params.encodeConfig).gopLength = NVENC_INFINITE_GOPLENGTH;
            (*initialize_params.encodeConfig).rcParams.rateControlMode =
                NV_ENC_PARAMS_RC_MODE::NV_ENC_PARAMS_RC_CONSTQP;
        }

        if self.motion_estimation_only {
            initialize_params.tuningInfo = tuning_info;
            let mut preset_config = NV_ENC_PRESET_CONFIG {
                version: NV_ENC_PRESET_CONFIG_VER,
                presetCfg: NV_ENC_CONFIG { version: NV_ENC_CONFIG_VER, ..Default::default() },
                ..Default::default()
            };
            unsafe {
                self.nv_encode_api_function_list.nvEncGetEncodePresetConfigEx.unwrap()(
                    self.encoder_handle as *mut _,
                    codec_guid,
                    preset_guid,
                    tuning_info,
                    &mut preset_config,
                )
                .into_nvenc_result()?;
                *initialize_params.encodeConfig = preset_config.presetCfg.clone();
            }
        } else {
            self.encode_config.version = NV_ENC_CONFIG_VER;
            self.encode_config.rcParams.rateControlMode =
                NV_ENC_PARAMS_RC_MODE::NV_ENC_PARAMS_RC_CONSTQP;
            self.encode_config.rcParams.constQP =
                NV_ENC_QP { qpInterP: 28, qpInterB: 31, qpIntra: 25 };
        }

        unsafe {
            if initialize_params.encodeGUID == NV_ENC_CODEC_H264_GUID {
                if matches!(self.buffer_format, BufferFormat::YUV444 | BufferFormat::YUV444_10BIT) {
                    (*initialize_params.encodeConfig)
                        .encodeCodecConfig
                        .h264Config
                        .chromaFormatIDC = 3;
                }
                (*initialize_params.encodeConfig).encodeCodecConfig.h264Config.idrPeriod =
                    (*initialize_params.encodeConfig).gopLength;
            } else if initialize_params.encodeGUID == NV_ENC_CODEC_HEVC_GUID {
                (*initialize_params.encodeConfig)
                    .encodeCodecConfig
                    .hevcConfig
                    .set_pixelBitDepthMinus8(
                        if matches!(
                            self.buffer_format,
                            BufferFormat::YUV420_10BIT | BufferFormat::YUV444_10BIT
                        ) {
                            2
                        } else {
                            0
                        },
                    );
                if matches!(self.buffer_format, BufferFormat::YUV444 | BufferFormat::YUV444_10BIT) {
                    (*initialize_params.encodeConfig)
                        .encodeCodecConfig
                        .hevcConfig
                        .set_chromaFormatIDC(3);
                }
                (*initialize_params.encodeConfig).encodeCodecConfig.hevcConfig.idrPeriod =
                    (*initialize_params.encodeConfig).gopLength;
            }
        }

        Ok(initialize_params)
    }

    fn get_initialize_params(&self) -> NvEncoderResult<NV_ENC_INITIALIZE_PARAMS> {
        if self.initialize_params.encodeConfig.is_null() {
            return Err(NvEncError::InvalidPointer.into());
        }
        Ok(self.initialize_params)
    }

    // not gonna implement this for now, not needed
    // pub fn run_motion_estimation() {
    //     unimplemented!()
    // }

    fn get_next_reference_frame(&self) -> &NvEncInputFrame {
        &self.reference_frames[(self.to_send as usize) % (self.encoder_buffer as usize)]
    }

    // not gonna implement this for now, not needed (i think?)
    // pub fn get_sequence_params() {
    //     unimplemented!()
    // }

    fn get_pixel_format(&self) -> BufferFormat {
        self.buffer_format
    }

    fn get_encoder_buffer_count(&self) -> i32 {
        self.encoder_buffer
    }
}

impl<ResourceManager> NvEncoderBase<ResourceManager>
where
    ResourceManager: NvEncoderResourceManager + ?Sized,
{
    pub(super) fn new(
        device_type: NV_ENC_DEVICE_TYPE,
        device: *mut Device,
        width: u32,
        height: u32,
        buffer_format: BufferFormat,
        extra_output_delay: u32,
        motion_estimation_only: bool,
        output_in_video_memory: bool,
    ) -> NvEncoderResult<Self> {
        let enc_api = Self::load_nv_enc_api()?;

        println!("made enc_api");

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

        println!("before nvEncOpenEncodeSessionEx done");
        unsafe {
            enc_api.nvEncOpenEncodeSessionEx.unwrap()(
                &mut encode_session_ex_params as *mut _,
                encoder_handle_ptr as *mut _,
            )
            .into_nvenc_result()?;
        }

        println!("nvEncOpenEncodeSessionEx done");

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

    // Protected

    pub(super) fn is_hw_encoder_initialized(&mut self) -> bool {
        unimplemented!()
    }

    pub(super) fn register_input_resources(
        &mut self,
        input_frames: &mut [NV_ENC_INPUT_RESOURCE_OPENGL_TEX], // TODO: make this not mut
        resource_type: NV_ENC_INPUT_RESOURCE_TYPE,
        width: u32,
        height: u32,
        pitch: u32,
        buffer_format: BufferFormat,
        reference_frame: bool,
    ) -> NvEncoderResult<()> {
        for input_frame in input_frames.iter_mut() {
            let registered_ptr = self.register_resource(
                input_frame,
                resource_type,
                width,
                height,
                pitch,
                buffer_format,
                NV_ENC_BUFFER_USAGE::NV_ENC_INPUT_IMAGE,
            )?;

            let mut chroma_offsets =
                self.buffer_format.get_chroma_subplane_offsets(pitch, height)?;
            chroma_offsets.resize(2, 0);
            // TODO(efyang): make input_ptr restricted as an enum, or just straight up opengl tex
            let registered_input_frame = NvEncInputFrame {
                input_ptr: input_frame as *mut NV_ENC_INPUT_RESOURCE_OPENGL_TEX as *mut Input,
                chroma_offsets: [chroma_offsets[0], chroma_offsets[1]],
                num_chroma_planes: self.buffer_format.get_num_chroma_planes()?,
                pitch,
                chroma_pitch: self.buffer_format.get_chroma_pitch(pitch)?,
                buffer_format: self.buffer_format,
                resource_type,
            };

            if reference_frame {
                self.registered_resources_for_reference.push(registered_ptr);
                self.reference_frames.push(registered_input_frame);
            } else {
                self.registered_resources.push(registered_ptr);
                self.input_frames.push(registered_input_frame);
            }
        }
        unimplemented!()
    }

    pub(super) fn unregister_input_resources(&mut self) -> NvEncoderResult<()> {
        self.flush_encoder();

        if self.motion_estimation_only {
            for &mapped_ref_buffer in self.mapped_ref_buffers.iter().filter(|p| !p.is_null()) {
                unsafe {
                    self.nv_encode_api_function_list.nvEncUnmapInputResource.unwrap()(
                        self.encoder_handle as *mut _,
                        mapped_ref_buffer,
                    )
                    .into_nvenc_result()?;
                }
            }
        }
        self.mapped_ref_buffers.clear();

        for &mapped_input_buffer in self.mapped_input_buffers.iter().filter(|p| !p.is_null()) {
            unsafe {
                self.nv_encode_api_function_list.nvEncUnmapInputResource.unwrap()(
                    self.encoder_handle as *mut _,
                    mapped_input_buffer,
                )
                .into_nvenc_result()?;
            }
        }
        self.mapped_input_buffers.clear();

        for &registered_resource in self.registered_resources.iter().filter(|&p| !p.is_null()) {
            unsafe {
                self.nv_encode_api_function_list.nvEncUnregisterResource.unwrap()(
                    self.encoder_handle as *mut _,
                    registered_resource,
                )
                .into_nvenc_result()?;
            }
        }
        self.registered_resources.clear();

        for &registered_resource_for_reference in
            self.registered_resources_for_reference.iter().filter(|&p| !p.is_null())
        {
            unsafe {
                self.nv_encode_api_function_list.nvEncUnregisterResource.unwrap()(
                    self.encoder_handle as *mut _,
                    registered_resource_for_reference,
                )
                .into_nvenc_result()?;
            }
        }
        self.registered_resources_for_reference.clear();

        Ok(())
    }

    fn register_resource(
        &mut self,
        buffer: &mut NV_ENC_INPUT_RESOURCE_OPENGL_TEX,
        resource_type: NV_ENC_INPUT_RESOURCE_TYPE,
        width: u32,
        height: u32,
        pitch: u32,
        buffer_format: BufferFormat,
        buffer_usage: NV_ENC_BUFFER_USAGE,
    ) -> NvEncoderResult<NV_ENC_REGISTERED_PTR> {
        let mut register_resource = NV_ENC_REGISTER_RESOURCE {
            resourceType: resource_type,
            resourceToRegister: buffer as *mut NV_ENC_INPUT_RESOURCE_OPENGL_TEX as *mut _,
            width,
            height,
            pitch,
            bufferFormat: buffer_format.into(),
            bufferUsage: buffer_usage,
            ..Default::default()
        };
        unsafe {
            self.nv_encode_api_function_list.nvEncRegisterResource.unwrap()(
                self.encoder_handle as *mut _,
                &mut register_resource,
            )
            .into_nvenc_result()?;
        }
        Ok(register_resource.registeredResource)
    }

    pub(super) fn get_max_encode_width(&self) -> u32 {
        self.max_encode_width
    }

    pub(super) fn get_max_encode_height(&self) -> u32 {
        self.max_encode_height
    }

    fn get_completion_event(&mut self, event_idx: u32) -> *mut CompletionEvent {
        if self.completion_event.len() == self.encoder_buffer as usize {
            self.completion_event[event_idx as usize]
        } else {
            std::ptr::null_mut()
        }
    }

    fn do_encode(
        &mut self,
        input_buffer: NV_ENC_INPUT_PTR,
        output_buffer: NV_ENC_OUTPUT_PTR,
        pic_params: Option<NV_ENC_PIC_PARAMS>,
    ) -> NvEncoderResult<()> {
        let mut pic_params = NV_ENC_PIC_PARAMS {
            version: NV_ENC_PIC_PARAMS_VER,
            pictureStruct: _NV_ENC_PIC_STRUCT::NV_ENC_PIC_STRUCT_FRAME,
            inputBuffer: input_buffer,
            bufferFmt: self.get_pixel_format().into(),
            inputWidth: self.get_encode_width(),
            inputHeight: self.get_encode_height(),
            outputBitstream: output_buffer,
            completionEvent: self
                .get_completion_event((self.to_send as u32) % (self.encoder_buffer as u32))
                as *mut _,
            ..pic_params.unwrap_or_default()
        };
        unsafe {
            self.nv_encode_api_function_list.nvEncEncodePicture.unwrap()(
                self.encoder_handle as *mut _,
                &mut pic_params,
            )
            .into_nvenc_result()?;
        }

        Ok(())
    }

    fn do_motion_estimation(
        &mut self,
        input_buffer: NV_ENC_INPUT_PTR,
        input_buffer_for_reference: NV_ENC_INPUT_PTR,
        output_buffer: NV_ENC_OUTPUT_PTR,
    ) -> NvEncoderResult<()> {
        // TODO: change to_send and encoder_buffer to u32
        let mut me_params = NV_ENC_MEONLY_PARAMS {
            inputBuffer: input_buffer,
            referenceFrame: input_buffer_for_reference,
            inputWidth: self.get_encode_width(),
            inputHeight: self.get_encode_height(),
            mvBuffer: output_buffer,
            completionEvent: self
                .get_completion_event((self.to_send as u32) % (self.encoder_buffer as u32))
                as *mut _,
            ..Default::default()
        };
        unsafe {
            self.nv_encode_api_function_list.nvEncRunMotionEstimationOnly.unwrap()(
                self.encoder_handle as *mut _,
                &mut me_params,
            )
            .into_nvenc_result()?;
        }
        Ok(())
    }

    fn map_resources(&mut self, buffer_index: u32) -> NvEncoderResult<()> {
        // TODO: a lot of these functions follow the same make a struct and then send it,
        // this could probably be wrapped up into actual rust functions, especially to separate out the version info
        let mut map_input_resource = NV_ENC_MAP_INPUT_RESOURCE {
            version: NV_ENC_MAP_INPUT_RESOURCE_VER,
            registeredResource: self.registered_resources[buffer_index as usize],
            ..Default::default()
        };

        unsafe {
            self.nv_encode_api_function_list.nvEncMapInputResource.unwrap()(
                self.encoder_handle as *mut _,
                &mut map_input_resource,
            )
            .into_nvenc_result()?;
        }
        self.mapped_input_buffers[buffer_index as usize] = map_input_resource.mappedResource;

        if self.motion_estimation_only {
            map_input_resource.registeredResource =
                self.registered_resources_for_reference[buffer_index as usize];
            unsafe {
                self.nv_encode_api_function_list.nvEncMapInputResource.unwrap()(
                    self.encoder_handle as *mut _,
                    &mut map_input_resource,
                )
                .into_nvenc_result()?;
                self.mapped_ref_buffers[buffer_index as usize] = map_input_resource.mappedResource;
            }
        }
        Ok(())
    }

    fn wait_for_completion_event(&self, _event: i32) {
        // does nothing on linux
    }

    fn send_eos(&mut self) -> NvEncoderResult<()> {
        let mut pic_params = NV_ENC_PIC_PARAMS {
            version: NV_ENC_PIC_PARAMS_VER,
            encodePicFlags: _NV_ENC_PIC_FLAGS::NV_ENC_PIC_FLAG_EOS,
            completionEvent: self
                .get_completion_event((self.to_send as u32) % (self.encoder_buffer as u32))
                as *mut _,
            ..Default::default()
        };
        unsafe {
            self.nv_encode_api_function_list.nvEncEncodePicture.unwrap()(
                self.encoder_handle as *mut _,
                &mut pic_params,
            )
            .into_nvenc_result()?;
        }
        Ok(())
    }

    // Private
    fn is_zero_delay(&self) -> bool {
        self.output_delay == 0
    }

    fn load_nv_enc_api() -> NvEncoderResult<NV_ENCODE_API_FUNCTION_LIST> {
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

    /// This is a private function which is used to get the output packets
    ///       from the encoder HW.
    /// This is called by DoEncode() function. If there is buffering enabled,
    /// this may return without any output data.
    // output_buffer is self.bitstream_output_buffer for now, as that's what is used in the code we're actually using
    fn get_encoded_packet(
        &mut self,
        packet: &mut Vec<Vec<u8>>,
        output_delay: bool,
    ) -> NvEncoderResult<()> {
        let mut i = 0;
        let end =
            if self.output_delay != 0 { self.to_send - self.output_delay } else { self.to_send };
        while self.got < end {
            let packet_index = (self.got % self.encoder_buffer) as usize;
            self.wait_for_completion_event(packet_index as i32);

            let mut lock_bitstream_data = NV_ENC_LOCK_BITSTREAM {
                version: NV_ENC_LOCK_BITSTREAM_VER,
                outputBitstream: self.bitstream_output_buffer[packet_index],
                ..Default::default()
            };
            lock_bitstream_data.set_doNotWait(false as u32);
            unsafe {
                self.nv_encode_api_function_list.nvEncLockBitstream.unwrap()(
                    self.encoder_handle as *mut _,
                    &mut lock_bitstream_data,
                )
                .into_nvenc_result()?;
            }

            let data_ptr = lock_bitstream_data.bitstreamBufferPtr as *mut u8;
            if packet.len() < i + 1 {
                packet.push(Vec::new());
            }
            packet[i].clear();
            unsafe {
                packet[i] = Vec::from_raw_parts(
                    data_ptr,
                    lock_bitstream_data.bitstreamSizeInBytes as usize,
                    lock_bitstream_data.bitstreamSizeInBytes as usize,
                );
            }
            i += 1;

            unsafe {
                self.nv_encode_api_function_list.nvEncUnlockBitstream.unwrap()(
                    self.encoder_handle as *mut _,
                    self.mapped_input_buffers[packet_index],
                )
                .into_nvenc_result()?;
            }

            if !self.mapped_input_buffers[packet_index].is_null() {
                unsafe {
                    self.nv_encode_api_function_list.nvEncUnmapInputResource.unwrap()(
                        self.encoder_handle as *mut _,
                        self.mapped_input_buffers[packet_index],
                    )
                    .into_nvenc_result()?;
                }
                self.mapped_input_buffers[packet_index] = std::ptr::null_mut();
            }

            if self.motion_estimation_only && !self.mapped_ref_buffers[packet_index].is_null() {
                unsafe {
                    self.nv_encode_api_function_list.nvEncUnmapInputResource.unwrap()(
                        self.encoder_handle as *mut _,
                        self.mapped_ref_buffers[packet_index],
                    )
                    .into_nvenc_result()?;
                }
                self.mapped_ref_buffers[packet_index] = std::ptr::null_mut();
            }

            self.got += 1;
        }
        Ok(())
    }

    // TODO: get rid of the resize bit above this function call
    fn initialize_bitstream_buffer(&mut self) -> NvEncoderResult<()> {
        for i in 0..self.encoder_buffer {
            let mut create_bitstream_buffer = NV_ENC_CREATE_BITSTREAM_BUFFER {
                version: NV_ENC_CREATE_BITSTREAM_BUFFER_VER,
                ..Default::default()
            };

            unsafe {
                self.nv_encode_api_function_list.nvEncCreateBitstreamBuffer.unwrap()(
                    self.encoder_handle as *mut _,
                    &mut create_bitstream_buffer,
                )
                .into_nvenc_result()?;
            }
            self.bitstream_output_buffer[i as usize] = create_bitstream_buffer.bitstreamBuffer;
        }
        Ok(())
    }

    fn destroy_bitstream_buffer(&mut self) -> NvEncoderResult<()> {
        for &bitstream_output_buffer in &self.bitstream_output_buffer {
            if !bitstream_output_buffer.is_null() {
                unsafe {
                    self.nv_encode_api_function_list.nvEncDestroyBitstreamBuffer.unwrap()(
                        self.encoder_handle as *mut _,
                        bitstream_output_buffer,
                    )
                    .into_nvenc_result()?;
                }
            }
        }
        self.bitstream_output_buffer.clear();
        Ok(())
    }

    fn initialize_mv_output_buffer(&mut self) -> NvEncoderResult<()> {
        for _ in 0..self.encoder_buffer {
            let mut create_mv_buffer = NV_ENC_CREATE_MV_BUFFER {
                version: NV_ENC_CREATE_MV_BUFFER_VER,
                ..Default::default()
            };
            unsafe {
                self.nv_encode_api_function_list.nvEncCreateMVBuffer.unwrap()(
                    self.encoder_handle as *mut _,
                    &mut create_mv_buffer,
                )
                .into_nvenc_result()?;
            }
            self.motion_vector_data_output_buffer.push(create_mv_buffer.mvBuffer);
        }
        Ok(())
    }

    fn destroy_mv_output_buffer(&mut self) -> NvEncoderResult<()> {
        for &mv_output_buffer in &self.motion_vector_data_output_buffer {
            if !mv_output_buffer.is_null() {
                unsafe {
                    self.nv_encode_api_function_list.nvEncDestroyBitstreamBuffer.unwrap()(
                        self.encoder_handle as *mut _,
                        mv_output_buffer,
                    )
                    .into_nvenc_result()?;
                }
            }
        }
        self.motion_vector_data_output_buffer.clear();
        Ok(())
    }

    fn destroy_hw_encoder(&mut self) -> NvEncoderResult<()> {
        if self.encoder_handle.is_null() {
            return Err(NvEncError::EncoderNotInitialized.into());
        }

        if self.motion_estimation_only {
            self.destroy_mv_output_buffer()?;
        } else {
            self.destroy_bitstream_buffer()?;
        }

        // TODO: wrap encoder handle in opaque pointer
        unsafe {
            self.nv_encode_api_function_list.nvEncDestroyEncoder.unwrap()(
                self.encoder_handle as *mut _,
            )
            .into_nvenc_result()?;
        }
        self.encoder_handle = std::ptr::null_mut();
        self.encoder_initialized = false;

        Ok(())
    }

    fn flush_encoder(&mut self) {
        if !self.motion_estimation_only && !self.output_in_video_memory {
            // from original code:
            // Incase of error it is possible for buffers still mapped to encoder.
            // flush the encoder queue and then unmapped it if any surface is still mapped
            // TODO: this seems bad lol wtf
            let mut packet = Vec::new();
            let _ = self.end_encode(&mut packet);
        }
    }
}

impl<ResourceManager> Drop for NvEncoderBase<ResourceManager>
where
    ResourceManager: NvEncoderResourceManager + ?Sized,
{
    fn drop(&mut self) {
        ResourceManager::release_input_buffers(self).unwrap();
        self.destroy_hw_encoder().unwrap();
    }
}

// TODO: clean this struct up
pub struct NvEncInputFrame {
    pub(super) input_ptr: *mut Input, // Originally a void pointer
    chroma_offsets: [u32; 2],
    num_chroma_planes: u32,
    pitch: u32,
    chroma_pitch: u32,
    buffer_format: BufferFormat,
    resource_type: NV_ENC_INPUT_RESOURCE_TYPE,
}
