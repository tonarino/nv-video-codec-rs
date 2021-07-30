use nv_video_codec_sys::{
    GUID, NV_ENC_CAPS, NV_ENC_DEVICE_TYPE, NV_ENC_INITIALIZE_PARAMS, NV_ENC_PIC_PARAMS,
    NV_ENC_TUNING_INFO,
};

use super::{
    nvencoderbase::{Device, NvEncInputFrame},
    NvEncoderError,
};
use crate::nvencoder::BufferFormat;

pub type NvEncoderResult<T> = Result<T, NvEncoderError>;
pub trait NvEncoder {
    fn create_encoder(&mut self, encoder_params: &NV_ENC_INITIALIZE_PARAMS) -> NvEncoderResult<()>;

    fn destroy_encoder(&mut self) -> NvEncoderResult<()>;

    // not implemented for now
    // pub fn reconfigure(&mut self, ) -> EncoderResult<bool>;

    fn get_next_input_frame(&mut self) -> &mut NvEncInputFrame;

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

    fn create_default_encoder_params(
        &mut self,
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

#[macro_export]
macro_rules! impl_nvencoder_wrapper_type {
    ($name:ident, $resourcemanager:ty) => {
        pub struct $name {
            encoder: NvEncoderBase<$resourcemanager>,
        }

        impl NvEncoder for $name {
            /// This function is used to initialize the encoder session.
            /// Application must call this function to initialize the encoder, before
            /// starting to encode any frames.
            fn create_encoder(
                &mut self,
                encoder_params: &nv_video_codec_sys::NV_ENC_INITIALIZE_PARAMS,
            ) -> NvEncoderResult<()> {
                self.encoder.create_encoder(encoder_params)
            }

            /// This function is used to destroy the encoder session.
            /// Application must call this function to destroy the encoder session and
            /// clean up any allocated resources. The application must call EndEncode()
            /// function to get any queued encoded frames before calling DestroyEncoder().
            fn destroy_encoder(&mut self) -> NvEncoderResult<()> {
                self.encoder.destroy_encoder()
            }

            /// This function is used to get the next available input buffer.
            /// Applications must call this function to obtain a pointer to the next
            /// input buffer. The application must copy the uncompressed data to the
            /// input buffer and then call EncodeFrame() function to encode it.
            fn get_next_input_frame(
                &mut self,
            ) -> &mut crate::nvencoder::nvencoderbase::NvEncInputFrame {
                self.encoder.get_next_input_frame()
            }

            /// This function is used to encode a frame.
            /// Applications must call EncodeFrame() function to encode the uncompressed
            /// data, which has been copied to an input buffer obtained from the
            /// GetNextInputFrame() function.
            fn encode_frame(
                &mut self,
                packet: &mut Vec<Vec<u8>>,
                pic_params: Option<nv_video_codec_sys::NV_ENC_PIC_PARAMS>,
            ) -> NvEncoderResult<()> {
                self.encoder.encode_frame(packet, pic_params)
            }

            /// This function to flush the encoder queue.
            /// The encoder might be queuing frames for B picture encoding or lookahead;
            /// the application must call EndEncode() to get all the queued encoded frames
            /// from the encoder. The application must call this function before destroying
            /// an encoder session.
            fn end_encode(&mut self, packet: &mut Vec<Vec<u8>>) -> NvEncoderResult<()> {
                self.encoder.end_encode(packet)
            }

            /// This function is used to query hardware encoder capabilities.
            /// Applications can call this function to query capabilities like maximum encode
            /// dimensions, support for lookahead or the ME-only mode etc.
            fn get_capability_value(
                &mut self,
                codec_guid: nv_video_codec_sys::GUID,
                caps_to_query: nv_video_codec_sys::NV_ENC_CAPS,
            ) -> NvEncoderResult<(nv_video_codec_sys::NV_ENC_CAPS, i32)> {
                self.encoder.get_capability_value(codec_guid, caps_to_query)
            }

            /// This function is used to get the current device on which encoder is running.
            fn get_device(&self) -> Option<&crate::nvencoder::nvencoderbase::Device> {
                self.encoder.get_device()
            }

            /// This function is used to get the current device type which encoder is running.
            fn get_device_type(&self) -> nv_video_codec_sys::NV_ENC_DEVICE_TYPE {
                self.encoder.get_device_type()
            }

            /// This function is used to get the current encode width.
            /// The encode width can be modified by Reconfigure() function.
            fn get_encode_width(&self) -> u32 {
                self.encoder.get_encode_width()
            }

            /// This function is used to get the current encode height.
            /// The encode width can be modified by Reconfigure() function.
            fn get_encode_height(&self) -> u32 {
                self.encoder.get_encode_height()
            }

            /// This function is used to get the current frame size based on pixel format.
            fn get_frame_size(&self) -> NvEncoderResult<u32> {
                self.encoder.get_frame_size()
            }

            fn create_default_encoder_params(
                &mut self,
                codec_guid: nv_video_codec_sys::GUID,
                preset_guid: nv_video_codec_sys::GUID,
                tuning_info: nv_video_codec_sys::NV_ENC_TUNING_INFO,
            ) -> NvEncoderResult<nv_video_codec_sys::NV_ENC_INITIALIZE_PARAMS> {
                self.encoder.create_default_encoder_params(codec_guid, preset_guid, tuning_info)
            }

            fn get_initialize_params(
                &self,
            ) -> NvEncoderResult<nv_video_codec_sys::NV_ENC_INITIALIZE_PARAMS> {
                self.encoder.get_initialize_params()
            }

            fn get_next_reference_frame(
                &self,
            ) -> &crate::nvencoder::nvencoderbase::NvEncInputFrame {
                self.encoder.get_next_reference_frame()
            }

            fn get_pixel_format(&self) -> BufferFormat {
                self.encoder.get_pixel_format()
            }

            fn get_encoder_buffer_count(&self) -> i32 {
                self.encoder.get_encoder_buffer_count()
            }
        }
    };
}
