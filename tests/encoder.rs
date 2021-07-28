extern crate anyhow;
extern crate log;
extern crate simple_logger;

#[path = "utils.rs"]
#[macro_use]
mod utils;
use glutin::{event_loop::EventLoop, platform::unix::EventLoopExtUnix, PossiblyCurrent};
use nv_video_codec_sys::{guids, NV_ENC_TUNING_INFO};
use utils::init_cuda_ctx;

use std::time::Duration;

use simple_logger::SimpleLogger;

use anyhow::Result;
use nv_video_codec_rs::nvencoder::types::BufferFormat;

use nv_video_codec_rs::nvencoder::{NvEncoder, NvEncoderGL, NvEncoderGLBuilder};

fn util_create_encoder() -> Result<(NvEncoderGL, glutin::Context<PossiblyCurrent>)> {
    let event_loop: EventLoop<()> = EventLoop::new_any_thread();
    let context_builder = glutin::ContextBuilder::new();
    let size = glutin::dpi::PhysicalSize { width: 1280, height: 720 };

    let context = unsafe {
        context_builder.build_headless(&event_loop, size).unwrap().make_current().unwrap()
    };
    gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

    let encoder = NvEncoderGLBuilder::new(1280, 720, BufferFormat::IYUV)
        .build()
        .expect("Could not create NvEncoderGl");
    Ok((encoder, context))
}

// #[test]
// fn init_encoder() -> Result<()> {
//     let _ = util_create_encoder()?;
//     Ok(())
// }

#[test]
fn create_encoder() -> Result<()> {
    let (mut encoder, _context) = util_create_encoder()?;
    let params = encoder.create_default_encoder_params(
        guids::NV_ENC_CODEC_HEVC_GUID,
        guids::NV_ENC_PRESET_P3_GUID,
        NV_ENC_TUNING_INFO::NV_ENC_TUNING_INFO_LOW_LATENCY,
    )?;
    encoder.create_encoder(&params)?;

    Ok(())
}
