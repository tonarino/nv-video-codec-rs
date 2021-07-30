extern crate anyhow;
extern crate gl;
extern crate log;
extern crate simple_logger;

#[path = "utils.rs"]
#[macro_use]
mod utils;

use std::io::Write;

use anyhow::Result;
use glutin::{event_loop::EventLoop, platform::unix::EventLoopExtUnix, PossiblyCurrent};
use nv_video_codec_rs::nvencoder::{
    types::BufferFormat, NvEncoder, NvEncoderGL, NvEncoderGLBuilder,
};
use nv_video_codec_sys::{guids, NV_ENC_INPUT_RESOURCE_OPENGL_TEX, NV_ENC_TUNING_INFO};

fn util_create_encoder(
    width: u32,
    height: u32,
    format: BufferFormat,
) -> Result<(NvEncoderGL, glutin::Context<PossiblyCurrent>)> {
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
    Ok((encoder, context))
}

#[test]
fn init_encoder() -> Result<()> {
    let _ = util_create_encoder(1280, 720, BufferFormat::NV12)?;
    Ok(())
}

#[test]
fn create_encoder() -> Result<()> {
    let (mut encoder, _context) = util_create_encoder(1280, 720, BufferFormat::NV12)?;
    let params = encoder.create_default_encoder_params(
        guids::NV_ENC_CODEC_HEVC_GUID,
        guids::NV_ENC_PRESET_P3_GUID,
        NV_ENC_TUNING_INFO::NV_ENC_TUNING_INFO_LOW_LATENCY,
    )?;
    encoder.create_encoder(&params)?;

    Ok(())
}

#[test]
fn encode_basic_grayscale() -> Result<()> {
    let (mut encoder, _context) = util_create_encoder(1280, 720, BufferFormat::NV12)?;
    let params = encoder.create_default_encoder_params(
        guids::NV_ENC_CODEC_HEVC_GUID,
        guids::NV_ENC_PRESET_P3_GUID,
        NV_ENC_TUNING_INFO::NV_ENC_TUNING_INFO_LOW_LATENCY,
    )?;
    encoder.create_encoder(&params)?;

    let data = include_bytes!("../resources/test/decode_out_grayscale.nv12");
    let frame_size = encoder.get_frame_size()?;
    dbg!(data.len(), frame_size);
    let mut host_frame = vec![0; frame_size as usize];
    host_frame.clone_from_slice(data);

    let encoder_input_frame = encoder.get_next_input_frame();
    let resource = encoder_input_frame.input_ptr_as_gltex();
    // TODO: remove these hacks
    unsafe {
        gl::BindTexture((*resource).target, (*resource).texture);
        gl::TexSubImage2D(
            (*resource).target,
            0,
            0,
            0,
            1280,
            720 * 3 / 2,
            gl::RED,
            gl::UNSIGNED_BYTE,
            host_frame.as_mut_ptr() as *mut _,
        );
        gl::BindTexture((*resource).target, 0);
    }
    let mut packet = Vec::new();
    encoder.encode_frame(&mut packet, None)?;
    encoder.end_encode(&mut packet)?;

    let mut f = std::fs::File::create("encode_out_grayscale.hevc")?;
    for frame in packet {
        f.write_all(&frame)?;
    }

    Ok(())
}
