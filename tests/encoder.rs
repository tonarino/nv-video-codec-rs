extern crate anyhow;
extern crate log;
extern crate simple_logger;

#[path = "utils.rs"]
#[macro_use]
mod utils;
use glutin::{event_loop::EventLoop, platform::unix::EventLoopExtUnix};
use utils::init_cuda_ctx;

use std::time::Duration;

use simple_logger::SimpleLogger;

use anyhow::Result;
use nv_video_codec_rs::nvencoder::types::BufferFormat;

use nv_video_codec_rs::nvencoder::NvEncoderGLBuilder;

#[test]
fn init_encoder() -> Result<()> {
    // let context = init_cuda_ctx()?;

    let event_loop: EventLoop<()> = EventLoop::new_any_thread();
    let context_builder = glutin::ContextBuilder::new();
    let size = glutin::dpi::PhysicalSize { width: 1280, height: 720 };

    let _context =
        unsafe { context_builder.build_headless(&event_loop, size).unwrap().make_current() };

    let encoder = NvEncoderGLBuilder::new(1280, 720, BufferFormat::YV12)
        .build()
        .expect("Could not create NvEncoderGl");
    std::mem::drop(encoder);
    Ok(())
}
