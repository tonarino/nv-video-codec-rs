extern crate anyhow;

use rustacuda::{
    context::{Context, ContextFlags},
    device::Device,
};

use anyhow::Result;
use nv_video_codec_rs::nvdecoder::{DecoderPacketFlags, NvDecoderBuilder};

#[test]
fn init_decoder() -> Result<()> {
    rustacuda::init(rustacuda::CudaFlags::empty())?;
    let device = Device::get_device(0)?;
    let context =
        Context::create_and_push(ContextFlags::MAP_HOST | ContextFlags::SCHED_AUTO, device)?;
    let decoder =
        NvDecoderBuilder::new(context, false, nv_video_codec_rs::common::Codec::HEVC).build()?;
    std::mem::drop(decoder);
    Ok(())
}

#[test]
fn decode_h265() -> Result<()> {
    rustacuda::init(rustacuda::CudaFlags::empty())?;
    let device = Device::get_device(0)?;
    let context =
        Context::create_and_push(ContextFlags::MAP_HOST | ContextFlags::SCHED_AUTO, device)?;
    let mut decoder =
        NvDecoderBuilder::new(context, false, nv_video_codec_rs::common::Codec::HEVC).build()?;

    let data = include_bytes!("../resources/test/single_i_frame.hevc");

    let frames_decoded = decoder.decode(data, DecoderPacketFlags::END_OF_PICTURE, 0)?;
    println!("frames decoded: {}, video info: {}", frames_decoded, decoder.get_video_info());

    Ok(())
}
