#![expect(non_upper_case_globals)]
#![expect(non_camel_case_types)]
#![expect(non_snake_case)]
#![expect(clippy::too_many_arguments)]
// https://github.com/rust-lang/rust-bindgen/issues/2807
#![expect(unnecessary_transmutes)]
// https://github.com/rust-lang/rust-bindgen/issues/2807
#![expect(clippy::useless_transmute)]
#![expect(clippy::missing_safety_doc)]
// https://github.com/rust-lang/rust-bindgen/issues/3053
#![expect(clippy::ptr_offset_with_cast)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub mod guids;
