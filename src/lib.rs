pub extern crate nv_video_codec_sys;

#[macro_use]
extern crate thiserror;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate rustacuda;

#[macro_use]
extern crate rustacuda_derive;
extern crate rustacuda_core;

extern crate parking_lot;

#[macro_use]
pub mod common;
pub mod nvdecoder;
pub mod nvencoder;

#[cfg(test)]
mod tests {
    extern crate anyhow;

    use rustacuda::{
        context::{Context, ContextFlags},
        device::Device,
    };

    use crate::nvdecoder::NvDecoderBuilder;
    use anyhow::Result;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn init_decoder() -> Result<()> {
        rustacuda::init(rustacuda::CudaFlags::empty())?;
        let device = Device::get_device(0)?;
        let context =
            Context::create_and_push(ContextFlags::MAP_HOST | ContextFlags::SCHED_AUTO, device)?;
        let decoder =
            NvDecoderBuilder::new(context, false, crate::common::CudaVideoCodec::HEVC).build()?;
        std::mem::drop(decoder);
        Ok(())
    }
}
