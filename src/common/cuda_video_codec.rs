use nv_video_codec_sys::cudaVideoCodec_enum;

ffi_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum CudaVideoCodec = cudaVideoCodec_enum
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
