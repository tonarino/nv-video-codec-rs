use nv_video_codec_sys::cudaVideoDeinterlaceMode_enum;

ffi_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum CudaVideoDeinterlaceMode = cudaVideoDeinterlaceMode_enum
    {
        Adaptive = cudaVideoDeinterlaceMode_Adaptive
        Bob = cudaVideoDeinterlaceMode_Bob
        Weave = cudaVideoDeinterlaceMode_Weave
    }
}
