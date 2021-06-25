extern crate anyhow;

use rustacuda::{
    context::{Context, ContextFlags},
    device::Device,
};

use anyhow::Result;
use nv_video_codec_rs::nvdecoder::{DecoderPacketFlags, NvDecoderBuilder};

fn init_cuda_ctx() -> Result<Context> {
    rustacuda::init(rustacuda::CudaFlags::empty())?;
    let device = Device::get_device(0)?;
    let context =
        Context::create_and_push(ContextFlags::MAP_HOST | ContextFlags::SCHED_AUTO, device)?;
    Ok(context)
}

#[test]
fn init_decoder() -> Result<()> {
    let context = init_cuda_ctx()?;
    let decoder =
        NvDecoderBuilder::new(context, false, nv_video_codec_rs::common::Codec::HEVC).build()?;
    std::mem::drop(decoder);
    Ok(())
}

#[test]
fn decode_h265_720p_grayscale() -> Result<()> {
    let context = init_cuda_ctx()?;
    let mut decoder =
        NvDecoderBuilder::new(context, false, nv_video_codec_rs::common::Codec::HEVC).build()?;
    let data = include_bytes!("../resources/test/single_i_frame.hevc");

    let start = std::time::Instant::now();
    let frames_decoded = decoder.decode(
        data,
        DecoderPacketFlags::END_OF_PICTURE | DecoderPacketFlags::END_OF_STREAM,
        0,
    )?;
    println!("Decoder output dimensions: {}x{}", decoder.get_width(), decoder.get_height());
    assert!(decoder.get_width() == 1280);
    assert!(decoder.get_height() == 720);
    println!(
        "frames decoded: {}, in {:?}.\n{}",
        frames_decoded,
        start.elapsed(),
        decoder.get_video_info()
    );
    assert!(frames_decoded > 0);
    let frame = decoder.get_frame().unwrap();
    println!("Got frame of size: {}", frame.data.as_ref().len());
    assert!(frame.data.as_ref().len() > 0);
    // std::fs::write("decode_out_grayscale.nv12", &frame.data)?;

    Ok(())
}

#[test]
fn decode_h265_720p_color() -> Result<()> {
    let context = init_cuda_ctx()?;
    let mut decoder =
        NvDecoderBuilder::new(context, false, nv_video_codec_rs::common::Codec::HEVC).build()?;
    let data = include_bytes!("../resources/test/single_i_frame_color.hevc");

    let start = std::time::Instant::now();
    let frames_decoded = decoder.decode(
        data,
        DecoderPacketFlags::END_OF_PICTURE | DecoderPacketFlags::END_OF_STREAM,
        0,
    )?;
    println!("Decoder output dimensions: {}x{}", decoder.get_width(), decoder.get_height());
    assert!(decoder.get_width() == 1280);
    assert!(decoder.get_height() == 720);
    println!(
        "frames decoded: {}, in {:?}.\n{}",
        frames_decoded,
        start.elapsed(),
        decoder.get_video_info()
    );
    assert!(frames_decoded > 0);
    let frame = decoder.get_frame().unwrap();
    println!("Got frame of size: {}", frame.data.as_ref().len());
    assert!(frame.data.as_ref().len() > 0);
    // std::fs::write("decoder_out_color.nv12", &frame.data)?;

    Ok(())
}

#[test]
fn decode_h265_720p_torture() -> Result<()> {
    const NUM_TORTURE_FRAMES: usize = 1000;
    let context = init_cuda_ctx()?;
    let mut decoder =
        NvDecoderBuilder::new(context, false, nv_video_codec_rs::common::Codec::HEVC).build()?;
    let data = include_bytes!("../resources/test/single_i_frame_3k.hevc");

    let start = std::time::Instant::now();
    let mut timestamp = 0;
    let mut total_frames_decoded = 0;
    for _ in 0..NUM_TORTURE_FRAMES {
        let frames_decoded = decoder.decode(
            data,
            DecoderPacketFlags::END_OF_PICTURE | DecoderPacketFlags::END_OF_STREAM,
            timestamp,
        )?;

        let _ = decoder.get_frame().unwrap();
        timestamp += 1;
        total_frames_decoded += frames_decoded;
    }

    let time = start.elapsed();

    println!("Decoder output dimensions: {}x{}", decoder.get_width(), decoder.get_height());
    assert!(decoder.get_width() == 3088);
    assert!(decoder.get_height() == 2076);
    assert!(total_frames_decoded > 0);

    println!(
        "frames decoded: {}, in {:?}, avg time per frame: {:?}.\n{}",
        total_frames_decoded,
        time,
        time / NUM_TORTURE_FRAMES as u32,
        decoder.get_video_info()
    );

    Ok(())
}
