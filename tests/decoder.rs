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

    let start = std::time::Instant::now();
    let frames_decoded = decoder.decode(
        data,
        DecoderPacketFlags::END_OF_PICTURE | DecoderPacketFlags::END_OF_STREAM,
        0,
    )?;
    println!(
        "frames decoded: {}, in {} seconds. video info: {}",
        frames_decoded,
        start.elapsed().as_secs_f64(),
        decoder.get_video_info()
    );
    let frame = decoder.get_frame().unwrap();
    std::fs::write("out.frame", &frame.data)?;

    Ok(())
}
