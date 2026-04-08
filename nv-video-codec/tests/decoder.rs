extern crate anyhow;
extern crate log;
extern crate simple_logger;

use anyhow::Result;
use nv_video_codec::decoder::{
    DecoderPacketFlags, DeviceFrameAllocator, FrameAllocator, HostFrameAllocator, NvDecoderBuilder,
};
use rustacuda::{
    context::{Context, ContextFlags},
    device::Device,
};
use simple_logger::SimpleLogger;
use std::time::Duration;

#[path = "utils.rs"]
#[macro_use]
mod utils;

// TODO(efyang) make this behaviour configurable and part of the library
// From NVPipe: Some cuvid implementations have one frame latency. Refeed frame into pipeline in this case.
const DECODE_TRIES: usize = 3;

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
    let decoder = NvDecoderBuilder::new(context, nv_video_codec::decoder::types::Codec::HEVC)
        .build::<HostFrameAllocator>()?;
    std::mem::drop(decoder);
    Ok(())
}

fn run_basic_decode(
    test_name: &str,
    data: &[u8],
    expected_width: u32,
    expected_height: u32,
) -> Result<Vec<u8>> {
    let _ = SimpleLogger::new().init();
    let context = init_cuda_ctx()?;
    let mut decoder = NvDecoderBuilder::new(context, nv_video_codec::decoder::types::Codec::HEVC)
        .build::<HostFrameAllocator>()?;

    let start = std::time::Instant::now();
    let packet_timestamp = -1;
    let mut decoding_output =
        decoder.decode(data, DecoderPacketFlags::END_OF_PICTURE, packet_timestamp)?;
    let mut i = 0;
    // TODO(mbernat): This loop is very random, try to understand it better.
    // It has something to do with the latency settings and the decoding output for the current
    // packet only being available in the later `decode()` calls.
    while i < DECODE_TRIES && decoding_output.frames.is_empty() {
        let packet_timestamp = i as i64;
        drop(decoding_output);
        decoding_output =
            decoder.decode(data, DecoderPacketFlags::END_OF_PICTURE, packet_timestamp)?;

        i += 1;
    }
    let frame_info = &decoding_output.frame_info;
    info_ctx!(
        test_name,
        "Decoder output dimensions: {}x{}",
        frame_info.width(),
        frame_info.height()
    );
    assert!(frame_info.width() == expected_width);
    assert!(frame_info.height() == expected_height);
    info_ctx!(
        test_name,
        "frames decoded: {}, in {:?}",
        decoding_output.frames.len(),
        start.elapsed(),
    );
    assert!(!frame_info.video_info().is_empty());
    let frame = decoding_output.frames.pop_front().unwrap();
    let frame_slice = frame.data.as_slice();
    info_ctx!(test_name, "Got frame of size: {}", frame_slice.len());
    assert!(!frame_slice.is_empty());

    // NOTE: frames can be checked with https://rawpixels.net/
    // std::fs::write("decode_out_grayscale.nv12", &frame.data)?;

    let mut out_vec = Vec::new();
    out_vec.extend_from_slice(frame_slice);
    Ok(out_vec)
}

#[test]
fn decode_h265_720p_basic_grayscale() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_grayscale.hevc");
    run_basic_decode("decode_h265_720p_basic_grayscale", data, 1280, 720)?;
    Ok(())
}

#[test]
fn decode_h265_720p_basic_color() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_color.hevc");
    run_basic_decode("decode_h265_720p_basic_color", data, 1280, 720)?;
    Ok(())
}

#[test]
fn decode_h265_3k_basic() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_3k.hevc");
    let frame = run_basic_decode("decode_h265_3k_basic", data, 3088, 2076)?;
    assert!(&frame[..10].iter().all(|&x| x == 173));
    Ok(())
}

#[test]
fn decode_h265_3k_device() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_3k.hevc");
    let frame = run_basic_decode("decode_h265_3k_device", data, 3088, 2076)?;
    assert!(&frame[..10].iter().all(|&x| x == 173));
    Ok(())
}

fn run_torture_test<FA: FrameAllocator>(
    test_name: &str,
    data: &[u8],
    expected_width: u32,
    expected_height: u32,
    frame_rate: Option<f64>, // frames/sec
) -> Result<()> {
    #[cfg(feature = "torture")]
    const NUM_TORTURE_FRAMES: i64 = 10000;
    #[cfg(not(feature = "torture"))]
    const NUM_TORTURE_FRAMES: i64 = 10;

    let _ = SimpleLogger::new().init();
    let context = init_cuda_ctx()?;

    let mut decoder = NvDecoderBuilder::new(context, nv_video_codec::decoder::types::Codec::HEVC)
        .build::<FA>()?;

    let mut total_time = Duration::from_millis(0);
    let mut blocked_time = Duration::from_millis(0);
    let mut total_frames_decoded = 0;
    for timestamp in 0..NUM_TORTURE_FRAMES {
        if let Some(frame_rate) = frame_rate {
            std::thread::sleep(Duration::from_secs_f64(1.0 / frame_rate));
        }
        let start = std::time::Instant::now();

        let packet_timestamp = -1;
        let mut decoding_output =
            decoder.decode(data, DecoderPacketFlags::END_OF_PICTURE, packet_timestamp)?;
        let mut i = 0;
        // TODO(mbernat): This loop is very random, try to understand it better.
        // It has something to do with the latency settings and the decoding output for the current
        // packet only being available in the later `decode()` calls.
        while i < DECODE_TRIES && decoding_output.frames.is_empty() {
            let packet_timestamp = i as i64;
            drop(decoding_output);
            decoding_output =
                decoder.decode(data, DecoderPacketFlags::END_OF_PICTURE, packet_timestamp)?;
            i += 1;
        }

        let _ = decoding_output.frames[0];

        total_time += start.elapsed();
        blocked_time += start.elapsed();

        total_frames_decoded += decoding_output.frames.len();
        if total_frames_decoded % 1000 == 0 {
            info_ctx!(
                test_name,
                "Decoded {} frames so far, average time/frame: {:?}",
                total_frames_decoded,
                blocked_time / 1000
            );
            blocked_time = Duration::from_millis(0);
        }

        if timestamp == 0 {
            let frame_info = &decoding_output.frame_info;

            info_ctx!(
                test_name,
                "Decoder output dimensions: {}x{}",
                frame_info.width(),
                frame_info.height()
            );
            assert!(frame_info.width() == expected_width);
            assert!(frame_info.height() == expected_height);
            assert!(total_frames_decoded > 0);
            assert!(!frame_info.video_info().is_empty());
        }
    }

    info_ctx!(
        test_name,
        "frames decoded: {}, in {:?}, avg time/frame in past {} frames: {:?}",
        total_frames_decoded,
        total_time,
        NUM_TORTURE_FRAMES,
        total_time / NUM_TORTURE_FRAMES as u32,
    );

    Ok(())
}

#[test]
fn decode_h265_3k_basic_torture() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_3k.hevc");
    run_torture_test::<HostFrameAllocator>("decode_h265_3k_basic_torture", data, 3088, 2076, None)
}

#[test]
fn decode_h265_3k_device_torture() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_3k.hevc");
    run_torture_test::<DeviceFrameAllocator>(
        "decode_h265_3k_device_torture",
        data,
        3088,
        2076,
        None,
    )
}

#[test]
fn decode_h265_3k_device_framelock_torture() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_3k.hevc");
    run_torture_test::<DeviceFrameAllocator>(
        "decode_h265_3k_device_framelock_torture",
        data,
        3088,
        2076,
        None,
    )
}

#[test]
fn decode_h265_3k_device_torture_60fps() -> Result<()> {
    let data = include_bytes!("../resources/test/single_i_frame_3k.hevc");
    run_torture_test::<DeviceFrameAllocator>(
        "decode_h265_3k_device_torture_60fps",
        data,
        3088,
        2076,
        Some(60.0),
    )
}
