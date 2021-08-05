extern crate anyhow;
extern crate gl;
extern crate log;
extern crate simple_logger;

#[path = "utils.rs"]
#[macro_use]
mod utils;

use std::{
    io::Write,
    ops::{Deref, DerefMut},
};

use anyhow::Result;
use glutin::{event_loop::EventLoop, platform::unix::EventLoopExtUnix, PossiblyCurrent};
use nv_video_codec_rs::nvencoder::{
    types::BufferFormat, NvEncoder, NvEncoderGL, NvEncoderGLBuilder,
};
use nv_video_codec_sys::{guids, NV_ENC_TUNING_INFO};

struct EncoderWithContext((NvEncoderGL, glutin::Context<PossiblyCurrent>));

impl Drop for EncoderWithContext {
    fn drop(&mut self) {
        // Make sure the encoder is dropped before the context.
        drop(&mut self.0 .0);
        drop(&mut self.0 .1);
    }
}

impl Deref for EncoderWithContext {
    type Target = NvEncoderGL;

    fn deref(&self) -> &Self::Target {
        &self.0 .0
    }
}

impl DerefMut for EncoderWithContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0 .0
    }
}

fn util_init_encoder(width: u32, height: u32, format: BufferFormat) -> Result<EncoderWithContext> {
    let event_loop: EventLoop<()> = EventLoop::new_any_thread();
    let context_builder = glutin::ContextBuilder::new();
    let size = glutin::dpi::PhysicalSize { width, height };

    let context = unsafe {
        context_builder.build_headless(&event_loop, size).unwrap().make_current().unwrap()
    };
    gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

    let encoder = NvEncoderGLBuilder::new(width, height, format)
        .build()
        .expect("Could not create NvEncoderGl");
    Ok(EncoderWithContext((encoder, context)))
}

fn util_create_encoder(encoder: &mut EncoderWithContext) -> Result<()> {
    let params = encoder.create_default_encoder_params(
        guids::NV_ENC_CODEC_HEVC_GUID,
        guids::NV_ENC_PRESET_P3_GUID,
        NV_ENC_TUNING_INFO::NV_ENC_TUNING_INFO_LOW_LATENCY,
    )?;
    encoder.create_encoder(&params)?;
    Ok(())
}

#[test]
fn init_encoder() -> Result<()> {
    let _ = util_init_encoder(1280, 720, BufferFormat::NV12)?;
    Ok(())
}

#[test]
fn create_encoder() -> Result<()> {
    let mut encoder = util_init_encoder(1280, 720, BufferFormat::NV12)?;
    util_create_encoder(&mut encoder)?;

    Ok(())
}

#[test]
fn encode_single_frame_grayscale() -> Result<()> {
    let (width, height) = (1280, 720);
    let mut encoder = util_init_encoder(width, height, BufferFormat::NV12)?;
    util_create_encoder(&mut encoder)?;

    let data = include_bytes!("../resources/test/decode_out_grayscale.nv12");
    assert_eq!(data.len(), encoder.get_frame_size()? as usize);

    let encoder_input_frame = encoder.get_next_input_frame();
    let resource = encoder_input_frame.input_ptr_as_gltex();
    // TODO: remove these hacks
    unsafe {
        gl::BindTexture((*resource).target, (*resource).texture);
        gl::TexSubImage2D(
            (*resource).target,
            0,                         // level
            0,                         // x offset
            0,                         // y offset
            width as i32,              // width
            (height * 3 / 2) as i32,   // height
            gl::RED,                   // format (single-channel)
            gl::UNSIGNED_BYTE,         // type
            data.as_ptr() as *const _, // data
        );
        gl::BindTexture((*resource).target, 0);
    }
    let mut packet = Vec::new();
    encoder.encode_frame(&mut packet, None)?;

    let mut f = std::fs::File::create("encode_out_grayscale.hevc")?;
    for frame in &packet {
        f.write_all(&frame)?;
    }

    encoder.end_encode(&mut packet)?;
    assert_eq!(0, packet.len());

    Ok(())
}

#[test]
fn encode_multi_frame_grayscale() -> Result<()> {
    let (width, height) = (1280, 720);
    let mut encoder = util_init_encoder(width, height, BufferFormat::NV12)?;
    util_create_encoder(&mut encoder)?;

    let data = include_bytes!("../resources/test/decode_out_grayscale.nv12");
    assert_eq!(data.len(), encoder.get_frame_size()? as usize);

    let mut f = std::fs::File::create("encode_out_grayscale_10frames.hevc")?;
    let mut packet = Vec::new();

    for _ in 0..10 {
        let encoder_input_frame = encoder.get_next_input_frame();
        let resource = encoder_input_frame.input_ptr_as_gltex();
        // TODO: remove these hacks
        unsafe {
            gl::BindTexture((*resource).target, (*resource).texture);
            gl::TexSubImage2D(
                (*resource).target,
                0,                         // level
                0,                         // x offset
                0,                         // y offset
                width as i32,              // width
                (height * 3 / 2) as i32,   // height
                gl::RED,                   // format (single-channel)
                gl::UNSIGNED_BYTE,         // type
                data.as_ptr() as *const _, // data
            );
            gl::BindTexture((*resource).target, 0);
        }
        encoder.encode_frame(&mut packet, None)?;
        for frame in &packet {
            f.write_all(&frame)?;
        }
    }

    encoder.end_encode(&mut packet)?;
    assert_eq!(0, packet.len());

    Ok(())
}
