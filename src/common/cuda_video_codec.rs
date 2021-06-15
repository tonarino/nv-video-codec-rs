#[derive(Clone, Copy, PartialEq)]
pub enum CudaVideoCodec {
    MPEG1,
    MPEG2,
    MPEG4,
    VC1,
    H264,
    JPEG,
    H264SVC,
    H264MVC,
    HEVC,
    VP8,
    VP9,
    AV1,
    YUV420,
    YV12,
    NV12,
    YUYV,
    UYVY,
}

impl Into<nv_video_codec_sys::cudaVideoCodec> for CudaVideoCodec {
    fn into(self) -> nv_video_codec_sys::cudaVideoCodec {
        match self {
            CudaVideoCodec::MPEG1 => nv_video_codec_sys::cudaVideoCodec_enum::cudaVideoCodec_MPEG1,
            CudaVideoCodec::MPEG2 => nv_video_codec_sys::cudaVideoCodec_enum::cudaVideoCodec_MPEG2,
            CudaVideoCodec::MPEG4 => nv_video_codec_sys::cudaVideoCodec_enum::cudaVideoCodec_MPEG4,
            CudaVideoCodec::VC1 => nv_video_codec_sys::cudaVideoCodec_enum::cudaVideoCodec_VC1,
            CudaVideoCodec::H264 => nv_video_codec_sys::cudaVideoCodec_enum::cudaVideoCodec_H264,
            CudaVideoCodec::JPEG => nv_video_codec_sys::cudaVideoCodec_enum::cudaVideoCodec_JPEG,
            CudaVideoCodec::H264SVC => {
                nv_video_codec_sys::cudaVideoCodec_enum::cudaVideoCodec_H264_SVC
            },
            CudaVideoCodec::H264MVC => {
                nv_video_codec_sys::cudaVideoCodec_enum::cudaVideoCodec_H264_MVC
            },
            CudaVideoCodec::HEVC => nv_video_codec_sys::cudaVideoCodec_enum::cudaVideoCodec_HEVC,
            CudaVideoCodec::VP8 => nv_video_codec_sys::cudaVideoCodec_enum::cudaVideoCodec_VP8,
            CudaVideoCodec::VP9 => nv_video_codec_sys::cudaVideoCodec_enum::cudaVideoCodec_VP9,
            CudaVideoCodec::AV1 => nv_video_codec_sys::cudaVideoCodec_enum::cudaVideoCodec_AV1,
            CudaVideoCodec::YUV420 => {
                nv_video_codec_sys::cudaVideoCodec_enum::cudaVideoCodec_YUV420
            },
            CudaVideoCodec::YV12 => nv_video_codec_sys::cudaVideoCodec_enum::cudaVideoCodec_YV12,
            CudaVideoCodec::NV12 => nv_video_codec_sys::cudaVideoCodec_enum::cudaVideoCodec_NV12,
            CudaVideoCodec::YUYV => nv_video_codec_sys::cudaVideoCodec_enum::cudaVideoCodec_YUYV,
            CudaVideoCodec::UYVY => nv_video_codec_sys::cudaVideoCodec_enum::cudaVideoCodec_UYVY,
        }
    }
}
