use ffi::{
    cudaVideoChromaFormat_enum, cudaVideoCodec_enum, cudaVideoCreateFlags_enum,
    cudaVideoDeinterlaceMode_enum, cudaVideoSurfaceFormat_enum,
};
use nv_video_codec_sys as ffi;

// *************** BASIC UTILITY TYPES ****************
// ---------------------------------------------------------------
/// Rect (inverted y)
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Rect<T> {
    pub left: T,
    pub top: T,
    pub right: T,
    pub bottom: T,
}

// ---------------------------------------------------------------
/// Dimensions (pixels)
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Dim<T> {
    pub width: T,
    pub height: T,
}

// ---------------------------------------------------------------

// ******************  FFI TYPES  *********************
// ---------------------------------------------------------------
// Chroma Format
ffi_enum! {
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub enum ChromaFormat = cudaVideoChromaFormat_enum
    cvt_err: ChromaFormatConvertError
    {
        YUV420 = cudaVideoChromaFormat_420
        YUV422 = cudaVideoChromaFormat_422
        YUV444 = cudaVideoChromaFormat_444
        Monochrome = cudaVideoChromaFormat_Monochrome
    }
}

// ---------------------------------------------------------------
// Video Codec
ffi_enum! {
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub enum Codec = cudaVideoCodec_enum
    cvt_err: CodecConvertError
    {
        MPEG1 = cudaVideoCodec_MPEG1
        MPEG2 = cudaVideoCodec_MPEG2
        MPEG4 = cudaVideoCodec_MPEG4
        VC1 = cudaVideoCodec_VC1
        H264 = cudaVideoCodec_H264
        JPEG = cudaVideoCodec_JPEG
        H264SVC = cudaVideoCodec_H264_SVC
        H264MVC = cudaVideoCodec_H264_MVC
        HEVC = cudaVideoCodec_HEVC
        VP8 = cudaVideoCodec_VP8
        VP9 = cudaVideoCodec_VP9
        AV1 = cudaVideoCodec_AV1
        YUV420 = cudaVideoCodec_YUV420
        YV12 = cudaVideoCodec_YV12
        NV12 = cudaVideoCodec_NV12
        YUYV = cudaVideoCodec_YUYV
        UYVY = cudaVideoCodec_UYVY
    }
}

// ---------------------------------------------------------------
// Video Create Flags
ffi_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum CreateFlags = cudaVideoCreateFlags_enum
    cvt_err: CreateFlagsConvertError
    {
        Default = cudaVideoCreate_Default
        PreferCUDA = cudaVideoCreate_PreferCUDA
        PreferDXVA = cudaVideoCreate_PreferDXVA
        PreferCUVID = cudaVideoCreate_PreferCUVID
    }
}

// ---------------------------------------------------------------
// Deinterlace Mode
ffi_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum DeinterlaceMode = cudaVideoDeinterlaceMode_enum
    cvt_err: DeinterlaceModeConvertError
    {
        Adaptive = cudaVideoDeinterlaceMode_Adaptive
        Bob = cudaVideoDeinterlaceMode_Bob
        Weave = cudaVideoDeinterlaceMode_Weave
    }
}

// ---------------------------------------------------------------
// Surface Format
ffi_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum SurfaceFormat = cudaVideoSurfaceFormat_enum
    cvt_err: SurfaceFormatConvertError
    {
        NV12 = cudaVideoSurfaceFormat_NV12
        P016 = cudaVideoSurfaceFormat_P016
        YUV444 = cudaVideoSurfaceFormat_YUV444
        YUV444_16bit = cudaVideoSurfaceFormat_YUV444_16Bit
    }
}

impl SurfaceFormat {
    pub fn chroma_height_factor(&self) -> f64 {
        match &self {
            Self::NV12 | Self::P016 => 0.5,
            Self::YUV444 | Self::YUV444_16bit => 1.0,
        }
    }

    pub fn chroma_plane_count(&self) -> usize {
        match &self {
            Self::NV12 | Self::P016 => 1,
            Self::YUV444 | Self::YUV444_16bit => 2,
        }
    }
}
