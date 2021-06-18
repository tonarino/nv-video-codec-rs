extern crate ac_ffmpeg;
extern crate anyhow;
use ac_ffmpeg::{
    format::{
        demuxer::{Demuxer, DemuxerWithStreamInfo},
        io::IO,
    },
    Error,
};
use anyhow::Result;
use std::{
    env,
    fs::{self, File},
};

fn open_input(path: &str) -> Result<DemuxerWithStreamInfo<File>, Error> {
    let input = File::open(path)
        .map_err(|err| Error::new(format!("unable to open input file {}: {}", path, err)))?;

    let io = IO::from_seekable_read_stream(input);

    Demuxer::builder().build(io)?.find_stream_info(None).map_err(|(_, err)| err)
}

/// Print information about a given input file.
fn print_info(input: &str) -> Result<(), Error> {
    let demuxer = open_input(input)?;

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

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    print_info(&args[1])?;
    let mut demux = open_input(&args[1])?;

    if let Some(packet) = demux.take()? {
        print!(
            "  packet (stream #{}, timestamp: {}, size: {})",
            packet.stream_index(),
            packet.pts().as_f32().unwrap_or(0f32),
            packet.data().len()
        );
        fs::write(&args[2], packet.data())?;
    }

    Ok(())
}
