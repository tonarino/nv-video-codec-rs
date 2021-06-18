extern crate ac_ffmpeg;
extern crate anyhow;

use std::{fs::File, path::PathBuf, thread::sleep, time::Duration};

use ac_ffmpeg::{
    format::{
        demuxer::{Demuxer, DemuxerWithStreamInfo, SeekTarget},
        io::IO,
    },
    time::Timestamp,
    Error,
};
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

fn open_input(path: &str) -> Result<DemuxerWithStreamInfo<File>, Error> {
    let input = File::open(path)
        .map_err(|err| Error::new(format!("unable to open input file {}: {}", path, err)))?;

    let io = IO::from_seekable_read_stream(input);

    Demuxer::builder().build(io)?.find_stream_info(None).map_err(|(_, err)| err)
}

/// Print information about a given input file.
fn print_info(input: &str) -> Result<(), Error> {
    let mut demuxer = open_input(input)?;

    for (index, stream) in demuxer.streams().iter().enumerate() {
        let params = stream.codec_parameters();

        println!("Stream #{}:", index);
        println!("  duration: {}", stream.duration().as_f64().unwrap_or(0f64));

        if let Some(params) = params.as_audio_codec_parameters() {
            println!("  type: audio");
            println!("  codec: {}", params.decoder_name().unwrap_or("N/A"));
            println!("  sample format: {}", params.sample_format().name());
            println!("  sample rate: {}", params.sample_rate());
            println!("  channels: {}", params.channel_layout().channels());
        } else if let Some(params) = params.as_video_codec_parameters() {
            println!("  type: video");
            println!("  codec: {}", params.decoder_name().unwrap_or("N/A"));
            println!("  width: {}", params.width());
            println!("  height: {}", params.height());
            println!("  pixel format: {}", params.pixel_format().name());
        } else {
            println!("  type: unknown");
        }
    }

    Ok(())
}

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
    let input_path = root.join("resources/test/Big_Buck_Bunny_1080_10s_1MB_H265.mp4");

    print_info(input_path.to_str().unwrap())?;
    let mut demuxer = open_input(input_path.to_str().unwrap())?;

    let mut total_decoded = 0;
    let mut n = 0;
    while let Some(packet) = demuxer.take()? {
        print!(
            "  packet (stream #{}, timestamp: {}, size: {})",
            packet.stream_index(),
            packet.pts().as_f32().unwrap_or(0f32),
            packet.data().len()
        );
        let frames_decoded =
            decoder.decode(packet.data(), DecoderPacketFlags::END_OF_PICTURE, n)?;
        total_decoded += frames_decoded;
        println!(
            " : Decoded {} frames this packet, decoded {} total frames",
            frames_decoded, total_decoded
        );
        if n == 0 && frames_decoded > 0 {
            println!("{}", decoder.get_video_info());
        }
        n += 1;
    }

    // sleep(Duration::from_secs(1));

    Ok(())
}
