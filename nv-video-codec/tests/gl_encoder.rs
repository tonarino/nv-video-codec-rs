extern crate anyhow;
extern crate gl;
extern crate log;
extern crate simple_logger;

use anyhow::Result;
use glutin::{event_loop::EventLoop, platform::unix::EventLoopExtUnix, Context, PossiblyCurrent};
use nv_video_codec::{
    encoder::{
        types::BufferFormat, EncodePicFlags, EncodeRateControl, EncodeRateControlMode,
        EncodeTuningInfo, NvEncoderExt, NvEncoderGL, NvEncoderParams, NvEncoderSettings,
    },
    guids::{EncodeCodec, EncodePreset},
};
use simple_logger::SimpleLogger;
use std::{
    io::Write,
    time::{Duration, Instant},
};

#[path = "utils.rs"]
#[macro_use]
mod utils;

struct GlEncoderContext {
    encoder: NvEncoderGL,

    // 'context' should be the last field in the struct to ensure
    // it is dropped last.
    _context: Context<PossiblyCurrent>,
}

fn util_init_encoder(width: u32, height: u32, format: BufferFormat) -> Result<GlEncoderContext> {
    let event_loop: EventLoop<()> = EventLoop::new_any_thread();
    let context_builder = glutin::ContextBuilder::new();
    let size = glutin::dpi::PhysicalSize { width, height };

    let context = unsafe {
        context_builder.build_headless(&event_loop, size).unwrap().make_current().unwrap()
    };
    gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

    let settings = NvEncoderSettings::new(width, height, format);
    let encoder = NvEncoderGL::new(settings).expect("Could not create NvEncoderGl");

    Ok(GlEncoderContext { encoder, _context: context })
}

fn util_create_encoder(encoder: &mut NvEncoderGL) -> Result<()> {
    let params = NvEncoderParams {
        codec: EncodeCodec::Hevc,
        // preset guid seems to have no real effect on the speed???
        // needs testing as well
        preset: EncodePreset::P3,
        // can't really see a difference between ULTRA_LOW_LATENCY and LOW_LATENCY???
        // ULTRA_LOW might be like 0.5ms faster at times?
        // needs testing on dev installation
        tuning_info: EncodeTuningInfo::UltraLowLatency,
        frame_rate: 60,
        // required for use with ffmpeg, not with nvcodec
        repeat_spspps: true,
        rate_control: EncodeRateControl {
            mode: EncodeRateControlMode::ConstantBitrate,
            low_delay_key_frame_scale: 1,
            average_bit_rate: 13_000_000,
            enable_aq: true,
        },
    };

    encoder.create_encoder(params)?;

    Ok(())
}

#[test]
fn init_encoder() -> Result<()> {
    let _encoder_context = util_init_encoder(1280, 720, BufferFormat::NV12)?;

    Ok(())
}

#[test]
fn create_encoder() -> Result<()> {
    let mut encoder_context = util_init_encoder(1280, 720, BufferFormat::NV12)?;
    util_create_encoder(&mut encoder_context.encoder)?;

    Ok(())
}

#[test]
fn encode_single_frame_grayscale() -> Result<()> {
    let (width, height) = (1280, 720);
    let mut encoder_context = util_init_encoder(width, height, BufferFormat::NV12)?;
    let encoder = &mut encoder_context.encoder;
    util_create_encoder(encoder)?;

    let data = include_bytes!("../resources/test/decode_out_grayscale.nv12");
    assert_eq!(data.len(), encoder.get_frame_size()? as usize);

    let resource = encoder.get_next_input_resource();
    // TODO: remove these hacks
    unsafe {
        gl::BindTexture(resource.target, resource.texture);
        gl::TexSubImage2D(
            resource.target,
            0,                         // level
            0,                         // x offset
            0,                         // y offset
            width as i32,              // width
            (height * 3 / 2) as i32,   // height
            gl::RED,                   // format (single-channel)
            gl::UNSIGNED_BYTE,         // type
            data.as_ptr() as *const _, // data
        );
        gl::BindTexture(resource.target, 0);
    }
    let mut packet = Vec::new();
    encoder.encode_frame(&mut packet, EncodePicFlags::empty())?;

    let mut f = std::fs::File::create("encode_out_grayscale.hevc")?;
    for frame in &packet {
        f.write_all(frame)?;
    }

    encoder.end_encode(&mut packet)?;
    assert_eq!(0, packet.len());

    Ok(())
}

#[test]
fn encode_multi_frame_3k() -> Result<()> {
    let _ = SimpleLogger::new().init();
    let (width, height) = (3088, 2076);
    let mut encoder_context = util_init_encoder(width, height, BufferFormat::NV12)?;
    let encoder = &mut encoder_context.encoder;
    util_create_encoder(encoder)?;

    let data = include_bytes!("../resources/test/decode_out_3k.nv12");
    assert_eq!(data.len(), encoder.get_frame_size()? as usize);

    let mut f = std::fs::File::create("encode_out_3k.hevc")?;
    let mut packet = Vec::new();

    #[cfg(feature = "torture")]
    const NUM_TORTURE_FRAMES: usize = 500;
    #[cfg(not(feature = "torture"))]
    const NUM_TORTURE_FRAMES: usize = 5;

    let mut total_time = Duration::from_millis(0);
    let mut blocked_time = Duration::from_millis(0);
    let mut frames_encoded = 0;
    // force intra-frame and force per-frame metadata
    let pic_flags = EncodePicFlags::FORCE_IDR | EncodePicFlags::SEQUENCE_HEADER;
    for _ in 0..NUM_TORTURE_FRAMES {
        let start_time = Instant::now();
        encoder.encode_frame_from_data(data, width, height, pic_flags, &mut packet)?;

        frames_encoded += 1;
        total_time += start_time.elapsed();
        blocked_time += start_time.elapsed();
        if frames_encoded % 500 == 0 {
            info_ctx!(
                "encode_multi",
                "Encoded last 500 frames in {:?}, {:?} per frame",
                blocked_time,
                blocked_time / 500
            );
            blocked_time = Duration::from_millis(0);
        }
        while start_time.elapsed() < Duration::from_millis(1000) / 60 {
            std::thread::sleep(Duration::from_micros(10));
        }
    }
    info_ctx!(
        "encode_multi",
        "Encoded {} frames in {:?}, {:?} per frame",
        NUM_TORTURE_FRAMES,
        total_time,
        total_time / NUM_TORTURE_FRAMES as u32
    );

    for frame in &packet {
        f.write_all(frame)?;
    }

    encoder.end_encode(&mut packet)?;
    assert_eq!(0, packet.len());

    Ok(())
}
