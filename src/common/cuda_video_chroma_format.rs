use ffi::cudaVideoChromaFormat_enum;
use nv_video_codec_sys as ffi;

ffi_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum CudaVideoChromaFormat = cudaVideoChromaFormat_enum
    cvt_err: CudaVideoChromaFormatConvertError
    {
        YUV420 = cudaVideoChromaFormat_420
        YUV422 = cudaVideoChromaFormat_422
        YUV444 = cudaVideoChromaFormat_444
        Monochrome = cudaVideoChromaFormat_Monochrome
    }
}
