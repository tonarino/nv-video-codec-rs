use super::{
    types::{Codec, Dim, Rect},
    NvDecoder, NvDecoderError,
};
use cudarc::driver::CudaContext;
use std::sync::Arc;

pub struct NvDecoderBuilder {
    pub(super) context: Arc<CudaContext>,
    pub(super) use_device_frame: bool,
    pub(super) codec: Codec,
    pub(super) low_latency: bool,
    pub(super) device_frame_pitched: bool,
    pub(super) crop_rect: Rect,
    pub(super) resize_dim: Dim,
    pub(super) max_width: u32,
    pub(super) max_height: u32,
    pub(super) clock_rate: u32,
}

impl NvDecoderBuilder {
    builder_field_setter!(use_device_frame: bool);

    builder_field_setter!(low_latency: bool);

    builder_field_setter!(device_frame_pitched: bool);

    builder_field_setter!(crop_rect: Rect);

    builder_field_setter!(resize_dim: Dim);

    builder_field_setter!(max_width: u32);

    builder_field_setter!(max_height: u32);

    builder_field_setter!(clock_rate: u32);

    pub fn new(context: Arc<CudaContext>, codec: Codec) -> Self {
        Self {
            context,
            use_device_frame: false,
            codec,
            low_latency: false,
            device_frame_pitched: false,
            crop_rect: Default::default(),
            resize_dim: Default::default(),
            max_width: 0,
            max_height: 0,
            clock_rate: 1000,
        }
    }

    pub fn build<'a>(self) -> Result<Box<NvDecoder<'a>>, NvDecoderError> {
        NvDecoder::new(self)
    }
}
