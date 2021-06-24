use ffi::cudaVideoCreateFlags_enum;
use nv_video_codec_sys as ffi;

ffi_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum CudaVideoCreateFlags = cudaVideoCreateFlags_enum
    cvt_err: CudaVideoCreateFlagsConvertError
    {
        Default = cudaVideoCreate_Default
        PreferCUDA = cudaVideoCreate_PreferCUDA
        PreferDXVA = cudaVideoCreate_PreferDXVA
        PreferCUVID = cudaVideoCreate_PreferCUVID
    }
}
