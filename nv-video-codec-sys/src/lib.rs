#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::redundant_static_lifetimes)]
// broken links from bindgen
#![allow(rustdoc::broken_intra_doc_links)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub use root::{
    cudaVideoCodec, Dim, NvDecoder, NvEncoder, Rect, NVENCSTATUS, NV_ENC_BUFFER_FORMAT,
    NV_ENC_CAPS, NV_ENC_INITIALIZE_PARAMS, NV_ENC_INPUT_PTR, NV_ENC_INPUT_RESOURCE_TYPE,
    NV_ENC_OUTPUT_PTR, NV_ENC_PIC_PARAMS, NV_ENC_RECONFIGURE_PARAMS, NV_ENC_REGISTERED_PTR,
    NV_ENC_TUNING_INFO,
};
