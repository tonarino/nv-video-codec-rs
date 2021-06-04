pub extern crate nv_video_codec_sys;

#[macro_use]
extern crate rustacuda;

#[macro_use]
extern crate rustacuda_derive;
extern crate rustacuda_core;

pub mod common;
pub mod decoder;
pub mod encoder;

#[cfg(test)]
mod tests {
    use crate::decoder::cuda_video_codec::CudaVideoCodec;
    use crate::decoder::nvdecoder::NvDecoder;
    use rustacuda::prelude::*;
    use std::error::Error;

    fn create_decoder() -> Result<NvDecoder, Box<dyn Error>> {
        // Initialize the CUDA API
        rustacuda::init(CudaFlags::empty())?;

        // Get the first device
        let device = Device::get_device(0)?;

        // Create a context associated to this device
        let context =
            Context::create_and_push(ContextFlags::MAP_HOST | ContextFlags::SCHED_AUTO, device)?;

        let decoder = NvDecoder::new(&context, true, CudaVideoCodec::H264);
        Ok(decoder)
    }

    #[test]
    fn decoder_init() -> Result<(), Box<dyn Error>> {
        let decoder = create_decoder()?;
        Ok(())
    }

    #[test]
    fn decoder_width() -> Result<(), Box<dyn Error>> {
        let decoder = create_decoder()?;
        dbg!(decoder.get_width());
        Ok(())
    }
}
