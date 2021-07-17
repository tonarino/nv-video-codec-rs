use nv_video_codec_sys::{
    GUID, NV_ENC_CAPS, NV_ENC_DEVICE_TYPE, NV_ENC_INITIALIZE_PARAMS, NV_ENC_PIC_PARAMS,
    NV_ENC_TUNING_INFO,
};

use super::{
    nvencoderbase::{Device, NvEncInputFrame, NvEncoderBase},
    resource_manager::NvEncoderResourceManager,
    NvEncoderError,
};
use crate::nvencoder::BufferFormat;

pub type NvEncoderResult<T> = Result<T, NvEncoderError>;
pub trait NvEncoder {
    fn create_encoder(&mut self, encoder_params: &NV_ENC_INITIALIZE_PARAMS) -> NvEncoderResult<()>;

    fn destroy_encoder(&mut self) -> NvEncoderResult<()>;

    // not implemented for now
    // pub fn reconfigure(&mut self, ) -> EncoderResult<bool>;

    fn get_next_input_frame(&mut self) -> &NvEncInputFrame;

    fn encode_frame(
        &mut self,
        packet: &mut Vec<Vec<u8>>,
        pic_params: Option<NV_ENC_PIC_PARAMS>,
    ) -> NvEncoderResult<()>;

    fn end_encode(&mut self, packet: &mut Vec<Vec<u8>>) -> NvEncoderResult<()>;

    fn get_capability_value(
        &mut self,
        codec_guid: GUID,
        caps_to_query: NV_ENC_CAPS,
    ) -> NvEncoderResult<(NV_ENC_CAPS, i32)>;

    fn get_device(&self) -> Option<&Device>;

    fn get_device_type(&self) -> NV_ENC_DEVICE_TYPE;

    fn get_encode_width(&self) -> u32;

    fn get_encode_height(&self) -> u32;

    fn get_frame_size(&self) -> NvEncoderResult<u32>;

    // unimplemented for now
    fn create_default_encoder_params(
        &self,
        codec_guid: GUID,
        preset_guid: GUID,
        tuning_info: NV_ENC_TUNING_INFO,
    ) -> NvEncoderResult<NV_ENC_INITIALIZE_PARAMS>;

    fn get_initialize_params(&self) -> NvEncoderResult<NV_ENC_INITIALIZE_PARAMS>;

    // not implemented
    // fn run_motion_estimation()

    fn get_next_reference_frame(&self) -> &NvEncInputFrame;

    // not gonna implement this for now, not needed (i think?)
    // fn get_sequence_params()

    fn get_pixel_format(&self) -> BufferFormat;

    fn get_encoder_buffer_count(&self) -> i32;
}

pub(super) trait NvEncoderImplementer {
    fn internal_encoder<ResourceManager>(&mut self) -> NvEncoderBase<ResourceManager>
    where
        ResourceManager: NvEncoderResourceManager + ?Sized;
}

impl<T> NvEncoder for T
where
    T: NvEncoderImplementer,
{
    fn create_encoder(&mut self, encoder_params: &NV_ENC_INITIALIZE_PARAMS) -> NvEncoderResult<()> {
        todo!()
    }

    fn destroy_encoder(&mut self) -> NvEncoderResult<()> {
        todo!()
    }

    fn get_next_input_frame(&mut self) -> &NvEncInputFrame {
        todo!()
    }

    fn encode_frame(
        &mut self,
        packet: &mut Vec<Vec<u8>>,
        pic_params: Option<NV_ENC_PIC_PARAMS>,
    ) -> NvEncoderResult<()> {
        todo!()
    }

    fn end_encode(&mut self, packet: &mut Vec<Vec<u8>>) -> NvEncoderResult<()> {
        todo!()
    }

    fn get_capability_value(
        &mut self,
        codec_guid: GUID,
        caps_to_query: NV_ENC_CAPS,
    ) -> NvEncoderResult<(NV_ENC_CAPS, i32)> {
        todo!()
    }

    fn get_device(&self) -> Option<&Device> {
        todo!()
    }

    fn get_device_type(&self) -> NV_ENC_DEVICE_TYPE {
        todo!()
    }

    fn get_encode_width(&self) -> u32 {
        todo!()
    }

    fn get_encode_height(&self) -> u32 {
        todo!()
    }

    fn get_frame_size(&self) -> NvEncoderResult<u32> {
        todo!()
    }

    fn create_default_encoder_params(
        &self,
        codec_guid: GUID,
        preset_guid: GUID,
        tuning_info: NV_ENC_TUNING_INFO,
    ) -> NvEncoderResult<NV_ENC_INITIALIZE_PARAMS> {
        todo!()
    }

    fn get_initialize_params(&self) -> NvEncoderResult<NV_ENC_INITIALIZE_PARAMS> {
        todo!()
    }

    fn get_next_reference_frame(&self) -> &NvEncInputFrame {
        todo!()
    }

    fn get_pixel_format(&self) -> BufferFormat {
        todo!()
    }

    fn get_encoder_buffer_count(&self) -> i32 {
        todo!()
    }
}
