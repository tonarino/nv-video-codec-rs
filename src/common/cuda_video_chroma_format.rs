use nv_video_codec_sys::cudaVideoChromaFormat_enum;

ffi_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum CudaVideoChromaFormat = cudaVideoChromaFormat_enum {
        YUV420 = cudaVideoChromaFormat_420
        YUV422 = cudaVideoChromaFormat_422
        YUV444 = cudaVideoChromaFormat_444
        Monochrome = cudaVideoChromaFormat_Monochrome
    }
}
