extern crate anyhow;
extern crate gl;
extern crate log;
extern crate simple_logger;

use anyhow::Result;
use cudarc::driver::{sys::CUctx_flags, CudaContext};
use nv_video_codec::{
    decoder::{
        frame::host::HostFrameAllocator, types::Codec, DecoderPacketFlags, NvDecoderBuilder,
    },
    encoder::{
        nvencodercuda::{upload_nv12_data_to_cuda_resource, NvEncoderCuda},
        types::BufferFormat,
        EncodeMultiPass, EncodePicFlags, EncodeQpMapMode, EncodeRateControl, EncodeRateControlMode,
        EncodeTuningInfo, LtrTrustMode, NvEncoderParams, NvEncoderSettings,
    },
    guids::{EncodeCodec, EncodePreset},
};
use simple_logger::SimpleLogger;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

#[path = "utils.rs"]
#[macro_use]
mod utils;

fn init_cuda_ctx() -> Result<Arc<CudaContext>> {
    let context = CudaContext::new(0)?;
    context.set_flags(CUctx_flags::CU_CTX_MAP_HOST)?;
    Ok(context)
}

fn util_init_encoder(width: u32, height: u32, format: BufferFormat) -> Result<NvEncoderCuda> {
    let context = init_cuda_ctx()?;
    let settings = NvEncoderSettings::new(width, height, format);

    let encoder = NvEncoderCuda::new(context, settings).expect("Could not create NvEncoderCuda");
    Ok(encoder)
}

fn common_encoder_params() -> NvEncoderParams {
    NvEncoderParams {
        codec: EncodeCodec::Hevc,
        // preset guid seems to have no real effect on the speed???
        // needs testing as well
        preset: EncodePreset::P3,
        // can't really see a difference between ULTRA_LOW_LATENCY and LOW_LATENCY???
        // ULTRA_LOW might be like 0.5ms faster at times?
        // needs testing on dev installation
        tuning_info: EncodeTuningInfo::UltraLowLatency,
        frame_rate_numerator: 60,
        frame_rate_denominator: 1,
        // required for use with ffmpeg, not with nvcodec
        repeat_spspps: true,
        rate_control: EncodeRateControl {
            mode: EncodeRateControlMode::ConstantBitrate,
            low_delay_key_frame_scale: 1,
            bit_rate: 16_000_000,
            enable_aq: true,
            multi_pass: EncodeMultiPass::TwoPassFullResolution,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn util_create_encoder(encoder: &mut NvEncoderCuda) -> Result<()> {
    encoder.create_encoder(common_encoder_params())?;
    Ok(())
}

#[test]
fn init_encoder() -> Result<()> {
    let _encoder = util_init_encoder(1280, 720, BufferFormat::NV12)?;

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

    encoder.set_bitrate_and_frame_rate(10_000_000, 30, 1)?;

    let data = include_bytes!("../resources/test/decode_out_grayscale.nv12");
    assert_eq!(data.len(), encoder.get_frame_size()? as usize);

    let _resource = encoder.get_next_input_resource();
    // TODO: Copy data to resource

    let mut packet = Vec::new();
    encoder.encode_frame(&mut packet, EncodePicFlags::empty(), None, None, None, 0)?;
    assert_eq!(packet.len(), 1);

    encoder.end_encode(&mut packet)?;
    assert_eq!(0, packet.len());

    Ok(())
}

#[test]
fn encode_multi_frame_3k() -> Result<()> {
    let _ = SimpleLogger::new().init();
    let (width, height) = (3088, 2076);
    let mut encoder = util_init_encoder(width, height, BufferFormat::NV12)?;
    util_create_encoder(&mut encoder)?;

    let data = include_bytes!("../resources/test/decode_out_3k.nv12");
    assert_eq!(data.len(), encoder.get_frame_size()? as usize);

    let mut packet = Vec::new();

    #[cfg(feature = "torture")]
    const NUM_FRAMES_TO_ENCODE: usize = 500;
    #[cfg(not(feature = "torture"))]
    const NUM_FRAMES_TO_ENCODE: usize = 20;

    const MAX_BITRATE: u32 = 50_000_000;

    let mut total_time = Duration::from_millis(0);
    let mut blocked_time = Duration::from_millis(0);
    let mut frames_encoded = 0;

    let mut force_i_frame = true;

    for i in 0..NUM_FRAMES_TO_ENCODE {
        let start_time = Instant::now();

        if i.is_multiple_of(5) {
            // Test encoding with progressively increasing bitrate.
            let bitrate = MAX_BITRATE / NUM_FRAMES_TO_ENCODE as u32 * (i as u32 + 1);
            encoder.set_bitrate_and_frame_rate(bitrate, 60, 1)?;
        }

        let resource = encoder.get_next_input_resource();
        upload_nv12_data_to_cuda_resource(data, resource, width, height);

        let pic_flags = if force_i_frame {
            // force intra-frame and per-frame metadata
            EncodePicFlags::FORCE_IDR | EncodePicFlags::SEQUENCE_HEADER
        } else {
            EncodePicFlags::empty()
        };
        encoder.encode_frame(&mut packet, pic_flags, None, None, None, 0)?;
        assert_eq!(packet.len(), 1);

        if !packet.is_empty() {
            force_i_frame = false;
        }

        if !packet.is_empty() {
            log::info!("packet.len() = {}, packet[0].len() = {}", packet.len(), packet[0].len());
        }

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
    }
    info_ctx!(
        "encode_multi",
        "Encoded {} frames in {:?}, {:?} per frame",
        NUM_FRAMES_TO_ENCODE,
        total_time,
        total_time / NUM_FRAMES_TO_ENCODE as u32
    );

    encoder.end_encode(&mut packet)?;
    assert_eq!(0, packet.len());

    Ok(())
}

#[test]
fn encode_qp_map_disabled() -> Result<()> {
    let mut encoder = util_init_encoder(1280, 720, BufferFormat::NV12)?;
    util_create_encoder(&mut encoder)?;

    let mut packet = Vec::new();
    encoder.encode_frame(&mut packet, EncodePicFlags::empty(), Some(&[0i8; 100]), None, None, 0)?;
    assert_eq!(packet.len(), 1);

    Ok(())
}

fn util_create_encoder_with(
    encoder: &mut NvEncoderCuda,
    codec: EncodeCodec,
    qp_map_mode: EncodeQpMapMode,
) -> Result<()> {
    encoder.create_encoder(NvEncoderParams { codec, qp_map_mode, ..common_encoder_params() })?;
    Ok(())
}

#[test]
fn encode_qp_map_delta_hevc() -> Result<()> {
    let mut encoder = util_init_encoder(1280, 720, BufferFormat::NV12)?;
    util_create_encoder_with(&mut encoder, EncodeCodec::Hevc, EncodeQpMapMode::Delta)?;

    let mut packet = Vec::new();

    // Correct size for 32×32 CTBs: ceil(1280/32) * ceil(720/32) = 920.
    encoder.encode_frame(
        &mut packet,
        EncodePicFlags::empty(),
        Some(&vec![0i8; 920]),
        None,
        None,
        0,
    )?;
    assert_eq!(packet.len(), 1);

    // Non-zero deltas are accepted.
    encoder.encode_frame(
        &mut packet,
        EncodePicFlags::empty(),
        Some(&vec![5i8; 920]),
        None,
        None,
        0,
    )?;
    assert_eq!(packet.len(), 1);
    encoder.encode_frame(
        &mut packet,
        EncodePicFlags::empty(),
        Some(&vec![-5i8; 920]),
        None,
        None,
        0,
    )?;
    assert_eq!(packet.len(), 1);

    // Wrong size is rejected (too small).
    let result = encoder.encode_frame(
        &mut packet,
        EncodePicFlags::empty(),
        Some(&vec![0i8; 100]),
        None,
        None,
        0,
    );
    assert!(result.is_err());

    // Wrong size assuming 64×64 CTBs: ceil(1280/64) * ceil(720/64) = 240.
    let result = encoder.encode_frame(
        &mut packet,
        EncodePicFlags::empty(),
        Some(&vec![0i8; 240]),
        None,
        None,
        0,
    );
    assert!(result.is_err());

    Ok(())
}

#[test]
fn encode_qp_map_delta_hevc_odd_resolution() -> Result<()> {
    let mut encoder = util_init_encoder(200, 200, BufferFormat::NV12)?;
    util_create_encoder_with(&mut encoder, EncodeCodec::Hevc, EncodeQpMapMode::Delta)?;

    let mut packet = Vec::new();

    // ceil(200/32) * ceil(200/32) = 7 * 7 = 49.
    encoder.encode_frame(
        &mut packet,
        EncodePicFlags::empty(),
        Some(&vec![0i8; 49]),
        None,
        None,
        0,
    )?;
    assert_eq!(packet.len(), 1);

    // Wrong size is still rejected.
    let result = encoder.encode_frame(
        &mut packet,
        EncodePicFlags::empty(),
        Some(&vec![0i8; 16]),
        None,
        None,
        0,
    );
    assert!(result.is_err());

    Ok(())
}

fn util_create_encoder_ltr(
    encoder: &mut NvEncoderCuda,
    ltr_num_frames: u32,
    ltr_trust_mode: LtrTrustMode,
) -> Result<()> {
    encoder.create_encoder(NvEncoderParams {
        ltr_num_frames,
        ltr_trust_mode,
        ..common_encoder_params()
    })?;
    Ok(())
}

#[test]
fn encode_ltr_round_trip() -> Result<()> {
    let mut encoder = util_init_encoder(1280, 720, BufferFormat::NV12)?;
    util_create_encoder_ltr(&mut encoder, 4, LtrTrustMode::PerPicture)?;

    let data = include_bytes!("../resources/test/decode_out_grayscale.nv12");
    let (w, h) = (1280, 720);
    let mut packet = Vec::new();
    let mut bitstream: Vec<Vec<u8>> = Vec::new();

    // Frame 0: IDR + mark index 0 as LTR
    upload_nv12_data_to_cuda_resource(data, encoder.get_next_input_resource(), w, h);
    encoder.encode_frame(
        &mut packet,
        EncodePicFlags::FORCE_IDR | EncodePicFlags::SEQUENCE_HEADER,
        None,
        Some(0),
        None,
        0,
    )?;
    bitstream.extend(packet.iter().map(|p| p.to_vec()));

    // Frames 1-3: use index 0 as LTR
    for ts in 1..=3 {
        upload_nv12_data_to_cuda_resource(data, encoder.get_next_input_resource(), w, h);
        encoder.encode_frame(&mut packet, EncodePicFlags::empty(), None, None, Some(1), ts)?;
        bitstream.extend(packet.iter().map(|p| p.to_vec()));
    }

    // Frame 4: mark index 1 as LTR, use both LTR 0+1
    upload_nv12_data_to_cuda_resource(data, encoder.get_next_input_resource(), w, h);
    encoder.encode_frame(&mut packet, EncodePicFlags::empty(), None, Some(1), Some(0b11), 4)?;
    bitstream.extend(packet.iter().map(|p| p.to_vec()));

    // Frame 5: use index 1 as LTR
    upload_nv12_data_to_cuda_resource(data, encoder.get_next_input_resource(), w, h);
    encoder.encode_frame(&mut packet, EncodePicFlags::empty(), None, None, Some(0b10), 5)?;
    bitstream.extend(packet.iter().map(|p| p.to_vec()));

    // Verify invalidate_ref_frames doesn't error.
    encoder.invalidate_ref_frames(0)?;

    encoder.end_encode(&mut packet)?;
    bitstream.extend(packet.iter().map(|p| p.to_vec()));

    // Decode the LTR-encoded bitstream.
    let context = init_cuda_ctx()?;
    let mut decoder = NvDecoderBuilder::new(context, Codec::HEVC)
        .low_latency(true)
        .build::<HostFrameAllocator>()?;

    let mut decoded = 0;
    for (i, frame_data) in bitstream.iter().enumerate() {
        if frame_data.is_empty() {
            continue;
        }
        let output = decoder.decode_one(frame_data, DecoderPacketFlags::empty(), i as i64)?;
        if output.frames.is_some() {
            decoded += 1;
        }
    }

    // Check we decoded at least one frame.
    assert!(decoded > 0, "no frames decoded from LTR bitstream");
    Ok(())
}
