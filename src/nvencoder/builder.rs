use super::{BufferFormat, NvEncoderError, NvEncoderGL};
// use rustacuda::context::Context;

// pub struct NvEncoderCudaBuilder {
//     context: Context,
// }

// impl NvEncoderCudaBuilder {
//     // builder_field_setter!(low_latency: bool);
//     pub fn new(context: Context) -> Self {
//         Self { context }
//     }

//     // pub fn build<'a>(self) -> Result<Box<NvEncoder<'a>>, NvEncoderError> {
//     //     NvEncoder::new()
//     // }
// }

pub struct NvEncoderGLBuilder {
    width: u32,
    height: u32,
    buffer_format: BufferFormat,
    extra_output_delay: u32,
    motion_estimation_only: bool,
}

impl NvEncoderGLBuilder {
    builder_field_setter!(extra_output_delay: u32);

    builder_field_setter!(motion_estimation_only: bool);

    pub fn new(width: u32, height: u32, buffer_format: BufferFormat) -> Self {
        // Note: this was originally set to 3 (was it from the NvCodec example?)
        let extra_output_delay = 0;

        Self { width, height, buffer_format, extra_output_delay, motion_estimation_only: false }
    }

    pub fn build<'a>(self) -> Result<NvEncoderGL, NvEncoderError> {
        NvEncoderGL::new(
            self.width,
            self.height,
            self.buffer_format,
            self.extra_output_delay,
            self.motion_estimation_only,
        )
    }
}
