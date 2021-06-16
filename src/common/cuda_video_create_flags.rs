use nv_video_codec_sys::cudaVideoCreateFlags_enum;

ffi_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum CudaVideoCreateFlags = cudaVideoCreateFlags_enum
    {
        Default = cudaVideoCreate_Default
        PreferCUDA = cudaVideoCreate_PreferCUDA
        PreferDXVA = cudaVideoCreate_PreferDXVA
        PreferCUVID = cudaVideoCreate_PreferCUVID
    }
}
