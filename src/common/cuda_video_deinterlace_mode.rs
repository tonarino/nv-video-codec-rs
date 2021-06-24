use ffi::cudaVideoDeinterlaceMode_enum;
use nv_video_codec_sys as ffi;

ffi_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum CudaVideoDeinterlaceMode = cudaVideoDeinterlaceMode_enum
    cvt_err: CudaVideoDeinterlaceModeConvertError
    {
        Adaptive = cudaVideoDeinterlaceMode_Adaptive
        Bob = cudaVideoDeinterlaceMode_Bob
        Weave = cudaVideoDeinterlaceMode_Weave
    }
}
