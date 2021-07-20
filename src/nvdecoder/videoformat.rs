use std::convert::TryFrom;

use num_enum::TryFromPrimitive;
use nv_video_codec_sys::CUVIDEOFORMAT;

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u32)]
pub enum Codec {
    Av1 = 11,
    H264 = 4,
    Hevc = 8,
    Jpeg = 5,
    Vp8 = 9,
    Vp9 = 10,
    Nv12 = 1314271538,
    Uyvy = 1431918169,
    Yuv420 = 1230591318,
    Yuyv = 1498765654,
    Yv12 = 1498820914,
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum Scan {
    Interlaced = 0,
    Progressive = 1,
}

pub struct FrameRate {
    numerator: u32,
    demoninator: u32,
}

pub struct Rect<T> {
    x: T,
    y: T,
}

pub struct PositionRect<T> {
    top_left: Rect<T>,
    bottom_right: Rect<T>,
}

// TODO(efyang) implement wrapper for videoformat
//TODO(mcginty): ignoring `video_signal_description` field for now.
pub struct VideoFormat {
    codec: Codec,
    frame_rate: FrameRate,
    /// Scan (progressive or interlaced)
    scan: Scan,
    /// Luma bit depth (in bits, typically: 8, 10, 12)
    bit_depth_luma: u8,
    /// Chroma bit depth (in bits, typically: 8, 10, 12)
    bit_depth_chroma: u8,
    min_num_decode_surfaces: u8,
    codec_size: Rect<u32>,
    display_area: PositionRect<i32>,
    display_aspect_ratio: Rect<i32>,
    /// Bitrate (bytes per second)
    bitrate: u32,
    seqhdr_data_length: u32,
}

impl TryFrom<CUVIDEOFORMAT> for VideoFormat {
    type Error = &'static str;

    fn try_from(format: CUVIDEOFORMAT) -> Result<Self, Self::Error> {
        Ok(Self {
            codec: Codec::try_from(format.codec).map_err(|_| "failed to identify codec")?,
            frame_rate: FrameRate {
                numerator: format.frame_rate.numerator,
                demoninator: format.frame_rate.denominator,
            },
            scan: Scan::try_from(format.progressive_sequence)
                .map_err(|_| "failed to identify codec")?,
            bit_depth_luma: format.bit_depth_luma_minus8 + 8,
            bit_depth_chroma: format.bit_depth_chroma_minus8 + 8,
            min_num_decode_surfaces: format.min_num_decode_surfaces,
            codec_size: Rect { x: format.coded_width, y: format.coded_height },
            display_area: PositionRect {
                top_left: Rect { x: format.display_area.left, y: format.display_area.top },
                bottom_right: Rect { x: format.display_area.right, y: format.display_area.bottom },
            },
            display_aspect_ratio: Rect { x: format.display_aspect_ratio.x, y: format.display_aspect_ratio.y },
            bitrate: format.bitrate,
            seqhdr_data_length: format.seqhdr_data_length,
        })
    }
}
