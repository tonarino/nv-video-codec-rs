extern crate anyhow;

use std::{fs::File, path::PathBuf, thread::sleep, time::Duration};

use rustacuda::{
    context::{Context, ContextFlags},
    device::Device,
};

use anyhow::Result;
use nv_video_codec_rs::nvdecoder::{DecoderPacketFlags, NvDecoderBuilder};

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
        NvDecoderBuilder::new(context, false, nv_video_codec_rs::common::CudaVideoCodec::HEVC)
            .build()?;
    std::mem::drop(decoder);
    Ok(())
}

// for a demuxer full-file test
// #[test]
// fn decode_h265() -> Result<()> {
//     rustacuda::init(rustacuda::CudaFlags::empty())?;
//     let device = Device::get_device(0)?;
//     let context =
//         Context::create_and_push(ContextFlags::MAP_HOST | ContextFlags::SCHED_AUTO, device)?;
//     let mut decoder =
//         NvDecoderBuilder::new(context, false, nv_video_codec_rs::common::CudaVideoCodec::HEVC)
//             .build()?;

//     let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
//     let input_path = root.join("resources/test/Big_Buck_Bunny_1080_10s_1MB_H265.mp4");

//     print_info(input_path.to_str().unwrap())?;
//     let mut demuxer = open_input(input_path.to_str().unwrap())?;

//     let mut total_decoded = 0;
//     let mut n = 0;
//     while let Some(packet) = demuxer.take()? {
//         print!(
//             "  packet (stream #{}, timestamp: {}, size: {})",
//             packet.stream_index(),
//             packet.pts().as_f32().unwrap_or(0f32),
//             packet.data().len()
//         );
//         let frames_decoded =
//             decoder.decode(packet.data(), DecoderPacketFlags::END_OF_PICTURE, n)?;
//         total_decoded += frames_decoded;
//         println!(
//             " : Decoded {} frames this packet, decoded {} total frames",
//             frames_decoded, total_decoded
//         );
//         if n == 0 && frames_decoded > 0 {
//             println!("{}", decoder.get_video_info());
//         }
//         n += 1;
//     }

//     // sleep(Duration::from_secs(1));

//     Ok(())
// }

#[test]
fn decode_h265() -> Result<()> {
    rustacuda::init(rustacuda::CudaFlags::empty())?;
    let device = Device::get_device(0)?;
    let context =
        Context::create_and_push(ContextFlags::MAP_HOST | ContextFlags::SCHED_AUTO, device)?;
    let mut decoder =
        NvDecoderBuilder::new(context, false, nv_video_codec_rs::common::CudaVideoCodec::HEVC)
            .build()?;

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let input_path = root.join("resources/test/Big_Buck_Bunny_1080_10s_1MB_F1.H265");
    let data = std::fs::read(input_path)?;

    let frames_decoded = decoder.decode(&data, DecoderPacketFlags::END_OF_PICTURE, 0)?;
    println!("frames decoded: {}, video info: {}", frames_decoded, decoder.get_video_info());

    Ok(())
}

#[test]
fn parse_h265() -> Result<()> {
    let context = unsafe { nv_video_codec_sys::CreateCudaContext(0) };
    assert!(!context.is_null());

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let input_path = root.join("resources/test/Big_Buck_Bunny_1080_10s_1MB_F1.H265");
    let data = std::fs::read(input_path)?;

    unsafe { nv_video_codec_sys::ParseFrame(data.as_ptr(), data.len() as i32) };

    Ok(())
}
