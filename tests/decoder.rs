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

fn run_basic_decode(data: &[u8], expected_width: u32, expected_height: u32) -> Result<()> {
    let context = init_cuda_ctx()?;
    let mut decoder =
        NvDecoderBuilder::new(context, false, nv_video_codec_rs::common::Codec::HEVC).build()?;

    let start = std::time::Instant::now();
    let frames_decoded = decoder.decode(
        data,
        DecoderPacketFlags::END_OF_PICTURE | DecoderPacketFlags::END_OF_STREAM,
        0,
    )?;
    println!("Decoder output dimensions: {}x{}", decoder.get_width(), decoder.get_height());
    assert!(decoder.get_width() == expected_width);
    assert!(decoder.get_height() == expected_height);
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
fn decode_h265_720p_basic_grayscale() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_grayscale.hevc");
    run_basic_decode(data, 1280, 720)
}

#[test]
fn decode_h265_720p_basic_color() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_color.hevc");
    run_basic_decode(data, 1280, 720)
}

#[test]
fn decode_h265_3k_basic() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_3k.hevc");
    run_basic_decode(data, 3088, 2076)
}

fn run_torture_test(
    data: &[u8],
    expected_width: u32,
    expected_height: u32,
    use_device_frame: bool,
) -> Result<()> {
    const NUM_TORTURE_FRAMES: usize = 1000;
    let context = init_cuda_ctx()?;
    let mut decoder =
        NvDecoderBuilder::new(context, use_device_frame, nv_video_codec_rs::common::Codec::HEVC)
            .build()?;

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
    assert!(decoder.get_width() == expected_width);
    assert!(decoder.get_height() == expected_height);
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

#[test]
fn decode_h265_3k_basic_torture() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_3k.hevc");
    run_torture_test(data, 3088, 2076, false)
}

// TODO(efyang): use log for cleaner test output
#[test]
fn decode_h265_3k_device_torture() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_3k.hevc");
    run_torture_test(data, 3088, 2076, true)
}
