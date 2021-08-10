extern crate anyhow;
extern crate log;
extern crate simple_logger;

#[path = "utils.rs"]
#[macro_use]
mod utils;

use anyhow::Result;
use nv_video_codec_rs::nvdecoder::{DecoderPacketFlags, NvDecoderBuilder};
use rustacuda::{
    context::{Context, ContextFlags},
    device::Device,
};
use simple_logger::SimpleLogger;
use std::time::Duration;

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

// TODO(efyang) make this behaviour configurable and part of the library
// From NVPipe: Some cuvid implementations have one frame latency. Refeed frame into pipeline in this case.
const DECODE_TRIES: usize = 3;
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
    let mut frames_decoded = 0;
    let mut i = 0;
    while i < DECODE_TRIES && frames_decoded == 0 {
        frames_decoded = decoder.decode(data, DecoderPacketFlags::END_OF_PICTURE, 0)?;
        i += 1;
    }
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
    use_locking: bool,
    frame_rate: Option<f64>, // frames/sec
) -> Result<()> {
    let _ = SimpleLogger::new().init();
    let context = init_cuda_ctx()?;
    let mut decoder =
        NvDecoderBuilder::new(context, use_device_frame, nv_video_codec_rs::common::Codec::HEVC)
            .build()?;

    let mut total_time = Duration::from_millis(0);
    let mut blocked_time = Duration::from_millis(0);
    let mut timestamp = 0;
    let mut total_frames_decoded = 0;
    for _ in 0..NUM_TORTURE_FRAMES {
        if let Some(frame_rate) = frame_rate {
            std::thread::sleep(Duration::from_secs_f64(1.0 / frame_rate));
        }
        let start = std::time::Instant::now();
        let mut frames_decoded = 0;

        // TODO(efyang) make this behaviour configurable and part of the library
        // From NVPipe: Some cuvid implementations have one frame latency. Refeed frame into pipeline in this case.
        let mut i = 0;
        while i < DECODE_TRIES && frames_decoded == 0 {
            frames_decoded = decoder.decode(data, DecoderPacketFlags::END_OF_PICTURE, timestamp)?;
            i += 1;
        }

        if !use_locking {
            let _ = decoder.get_frame().unwrap();
        } else {
            let frame = decoder.get_locked_frame().unwrap();
            decoder.unlock_frame(frame);
        }
        total_time += start.elapsed();
        blocked_time += start.elapsed();

        timestamp += 1;
        total_frames_decoded += frames_decoded;
        if total_frames_decoded % 1000 == 0 {
            info_ctx!(
                test_name,
                "Decoded {} frames so far, average time/frame: {:?}",
                total_frames_decoded,
                blocked_time / 1000
            );
            blocked_time = Duration::from_millis(0);
        }
    }

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
        "frames decoded: {}, in {:?}, avg time/frame in past 1000 frames: {:?}",
        total_frames_decoded,
        total_time,
        total_time / NUM_TORTURE_FRAMES as u32,
    );

    Ok(())
}

#[test]
#[cfg(feature = "torture")]
fn decode_h265_3k_basic_torture() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_3k.hevc");
    run_torture_test("decode_h265_3k_basic_torture", data, 3088, 2076, false, false, None)
}

#[test]
#[cfg(feature = "torture")]
fn decode_h265_3k_device_torture() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_3k.hevc");
    run_torture_test("decode_h265_3k_device_torture", data, 3088, 2076, true, false, None)
}

#[test]
#[cfg(feature = "torture")]
fn decode_h265_3k_device_framelock_torture() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_3k.hevc");
    run_torture_test("decode_h265_3k_device_framelock_torture", data, 3088, 2076, true, true, None)
}

#[test]
#[cfg(feature = "torture")]
fn decode_h265_3k_device_torture_60fps() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_3k.hevc");
    run_torture_test(
        "decode_h265_3k_device_torture_60fps",
        data,
        3088,
        2076,
        true,
        false,
        Some(60.0),
    )
}
