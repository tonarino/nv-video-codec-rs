use std::convert::TryFrom;

use num_enum::TryFromPrimitive;
use nv_video_codec_sys::CUVIDEOFORMAT;
use crate::common::{Dim, Rect, types::{ChromaFormat, Codec}};

#[derive(Copy, Clone, Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum Scan {
    Interlaced = 0,
    Progressive = 1,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct FrameRate {
    numerator: u32,
    demoninator: u32,
}

// TODO(mcginty): ignoring complicated `video_signal_description` struct for now.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VideoFormat {
    pub codec: Codec,
    pub frame_rate: FrameRate,
    /// Scan (progressive or interlaced)
    pub scan: Scan,
    pub chroma_format: ChromaFormat,
    /// Luma bit depth (in bits, typically: 8, 10, 12)
    pub bit_depth_luma: u8,
    /// Chroma bit depth (in bits, typically: 8, 10, 12)
    pub bit_depth_chroma: u8,
    pub min_num_decode_surfaces: u8,
    pub coded_size: Dim<u32>,
    pub display_area: Rect<i32>,
    pub display_aspect_ratio: Dim<i32>,
    /// Bitrate (bytes per second)
    pub bitrate: u32,
    pub seqhdr_data_length: u32,
}

impl Default for VideoFormat {
    fn default() -> Self {
        Self {
            codec: Codec::HEVC,
            scan: Scan::Progressive,
            chroma_format: ChromaFormat::YUV420,
            ..Default::default()
        }
    }
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
            chroma_format: ChromaFormat::try_from(format.chroma_format).map_err(|_| "failed to identify chroma format")?,
            bit_depth_luma: format.bit_depth_luma_minus8 + 8,
            bit_depth_chroma: format.bit_depth_chroma_minus8 + 8,
            min_num_decode_surfaces: format.min_num_decode_surfaces,
            coded_size: Dim { width: format.coded_width, height: format.coded_height },
            display_area: Rect {
                top: format.display_area.top,
                left: format.display_area.left,
                bottom: format.display_area.bottom,
                right: format.display_area.right,
            },
            display_aspect_ratio: Dim { width: format.display_aspect_ratio.x, height: format.display_aspect_ratio.y },
            bitrate: format.bitrate,
            seqhdr_data_length: format.seqhdr_data_length,
        })
    }
}
