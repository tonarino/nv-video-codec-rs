use rustacuda::context::Context;

use crate::common::{Codec, Dim, Rect};

use super::{NvDecoder, NvDecoderError};

pub struct NvDecoderBuilder {
    context: Context,
    use_device_frame: bool,
    codec: Codec,
    low_latency: bool,
    device_frame_pitched: bool,
    crop_rect: Rect,
    resize_dim: Dim,
    max_width: u32,
    max_height: u32,
    clock_rate: u32,
}

impl NvDecoderBuilder {
    builder_field_setter!(low_latency: bool);

    builder_field_setter!(device_frame_pitched: bool);

    builder_field_setter!(crop_rect: Rect);

    builder_field_setter!(resize_dim: Dim);

    builder_field_setter!(max_width: u32);

    builder_field_setter!(max_height: u32);

    builder_field_setter!(clock_rate: u32);

    pub fn new(context: Context, use_device_frame: bool, codec: Codec) -> Self {
        Self {
            context,
            use_device_frame,
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
        NvDecoder::new(
            self.context,
            self.use_device_frame,
            self.codec,
            self.low_latency,
            self.device_frame_pitched,
            self.crop_rect,
            self.resize_dim,
            self.max_width,
            self.max_height,
            self.clock_rate,
        )
    }
}
