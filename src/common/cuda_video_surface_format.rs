use ffi::cudaVideoSurfaceFormat_enum;
use nv_video_codec_sys as ffi;

ffi_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum CudaVideoSurfaceFormat = cudaVideoSurfaceFormat_enum
    cvt_err: CudaVideoSurfaceFormatConvertError
    {
        NV12 = cudaVideoSurfaceFormat_NV12
        P016 = cudaVideoSurfaceFormat_P016
        YUV444 = cudaVideoSurfaceFormat_YUV444
        YUV444_16bit = cudaVideoSurfaceFormat_YUV444_16Bit
    }
}

impl CudaVideoSurfaceFormat {
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
