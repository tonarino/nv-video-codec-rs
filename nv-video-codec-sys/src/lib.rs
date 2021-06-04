#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::redundant_static_lifetimes)]
// broken links from bindgen
#![allow(rustdoc::broken_intra_doc_links)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub use root::{
    cudaVideoCodec, cudaVideoCodec_enum, CUcontext, Dim, NvDecoder, NvEncoder, Rect, NVENCSTATUS,
    NV_ENC_BUFFER_FORMAT, NV_ENC_CAPS, NV_ENC_INITIALIZE_PARAMS, NV_ENC_INPUT_PTR,
    NV_ENC_INPUT_RESOURCE_TYPE, NV_ENC_OUTPUT_PTR, NV_ENC_PIC_PARAMS, NV_ENC_RECONFIGURE_PARAMS,
    NV_ENC_REGISTERED_PTR, NV_ENC_TUNING_INFO,
};

use cxx::{type_id, ExternType};

unsafe impl ExternType for root::NvDecoder {
    type Id = type_id!("NvDecoder");
    type Kind = cxx::kind::Opaque;
}

// unsafe impl ExternType for root::CUcontext {
//     type Id = type_id!("CUcontext");
//     type Kind = cxx::kind::Opaque;
// }

// unsafe impl ExternType for root::Dim {
//     type Id = type_id!("Dim");
//     type Kind = cxx::kind::Opaque;
// }

#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("Video_Codec_SDK_11.0.10/Samples/NvCodec/NvDecoder/NvDecoder.h");
        include!("Video_Codec_SDK_11.0.10/Samples/NvCodec/NvEncoder/NvEncoder.h");

        type NvDecoder = crate::root::NvDecoder;
        // type CUcontext = crate::root::CUcontext;
        // type cudaVideoCodec;
        // type Rect;
        // type Dim = crate::root::Dim;

        // fn GetContext(self: &NvDecoder) -> UniquePtr<CUcontext>;
        fn GetWidth(self: &NvDecoder) -> i32;
    }
}
