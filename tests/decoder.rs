extern crate anyhow;
extern crate log;
extern crate simple_logger;

#[path = "utils.rs"]
#[macro_use]
mod utils;

use rustacuda::{
    context::{Context, ContextFlags},
    device::Device,
};
use simple_logger::SimpleLogger;

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

fn run_basic_decode(
    test_name: &str,
    data: &[u8],
    expected_width: u32,
    expected_height: u32,
    use_device_frame: bool,
) -> Result<Vec<u8>> {
    let _ = SimpleLogger::new().init();
    let context = init_cuda_ctx()?;
    let mut decoder =
        NvDecoderBuilder::new(context, use_device_frame, nv_video_codec_rs::common::Codec::HEVC)
            .build()?;

    let start = std::time::Instant::now();
    let frames_decoded = decoder.decode(
        data,
        DecoderPacketFlags::END_OF_PICTURE | DecoderPacketFlags::END_OF_STREAM,
        0,
    )?;
    info_ctx!(
        test_name,
        "Decoder output dimensions: {}x{}",
        decoder.get_width(),
        decoder.get_height()
    );
    assert!(decoder.get_width() == expected_width);
    assert!(decoder.get_height() == expected_height);
    info_ctx!(test_name, "frames decoded: {}, in {:?}", frames_decoded, start.elapsed(),);
    assert!(decoder.get_video_info().len() > 0);
    assert!(frames_decoded > 0);
    let frame = decoder.get_frame().unwrap();
    info_ctx!(test_name, "Got frame of size: {}", frame.data.as_ref().len());
    assert!(frame.data.as_ref().len() > 0);

    // NOTE: frames can be checked with https://rawpixels.net/
    // std::fs::write("decode_out_grayscale.nv12", &frame.data)?;

    let mut out_vec = Vec::new();
    out_vec.extend_from_slice(frame.data.as_ref());
    Ok(out_vec)
}

#[test]
fn decode_h265_720p_basic_grayscale() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_grayscale.hevc");
    run_basic_decode("decode_h265_720p_basic_grayscale", data, 1280, 720, false)?;
    Ok(())
}

#[test]
fn decode_h265_720p_basic_color() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_color.hevc");
    run_basic_decode("decode_h265_720p_basic_color", data, 1280, 720, false)?;
    Ok(())
}

#[test]
fn decode_h265_3k_basic() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_3k.hevc");
    let frame = run_basic_decode("decode_h265_3k_basic", data, 3088, 2076, false)?;
    assert!(&frame[..10].iter().all(|&x| x == 173));
    Ok(())
}

#[test]
fn decode_h265_3k_device() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_3k.hevc");
    let frame = run_basic_decode("decode_h265_3k_device", data, 3088, 2076, false)?;
    assert!(&frame[..10].iter().all(|&x| x == 173));
    Ok(())
}

const NUM_TORTURE_FRAMES: usize = 10000;
fn run_torture_test(
    test_name: &str,
    data: &[u8],
    expected_width: u32,
    expected_height: u32,
    use_device_frame: bool,
) -> Result<()> {
    let _ = SimpleLogger::new().init();
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

    info_ctx!(
        test_name,
        "Decoder output dimensions: {}x{}",
        decoder.get_width(),
        decoder.get_height()
    );
    assert!(decoder.get_width() == expected_width);
    assert!(decoder.get_height() == expected_height);
    assert!(total_frames_decoded > 0);
    assert!(decoder.get_video_info().len() > 0);

    info_ctx!(
        test_name,
        "frames decoded: {}, in {:?}, avg time per frame: {:?}",
        total_frames_decoded,
        time,
        time / NUM_TORTURE_FRAMES as u32,
    );

    Ok(())
}

#[test]
fn decode_h265_3k_basic_torture() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_3k.hevc");
    run_torture_test("decode_h265_3k_basic_torture", data, 3088, 2076, false)
}

// TODO(efyang): use log for cleaner test output
#[test]
fn decode_h265_3k_device_torture() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_3k.hevc");
    run_torture_test("decode_h265_3k_device_torture", data, 3088, 2076, true)
}
