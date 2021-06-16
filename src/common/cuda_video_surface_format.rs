use nv_video_codec_sys::cudaVideoSurfaceFormat_enum;

ffi_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum CudaVideoSurfaceFormat = cudaVideoSurfaceFormat_enum
    {
        NV12 = cudaVideoSurfaceFormat_NV12
        P016 = cudaVideoSurfaceFormat_P016
        YUV444 = cudaVideoSurfaceFormat_YUV444
        YUV444_16bit = cudaVideoSurfaceFormat_YUV444_16Bit
    }
}
