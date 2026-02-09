// TODO: check these attributes are still needed
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::redundant_static_lifetimes)]
#![allow(clippy::too_many_arguments)]
// broken links from bindgen
#![allow(rustdoc::broken_intra_doc_links)]
// https://github.com/rust-lang/rust-bindgen/issues/1651
#![allow(deref_nullptr)]
// https://github.com/rust-lang/rust-bindgen/issues/2807
#![allow(unnecessary_transmutes)]
// https://github.com/rust-lang/rust-bindgen/issues/2807
#![allow(clippy::useless_transmute)]
#![allow(clippy::missing_safety_doc)]
// https://github.com/rust-lang/rust-bindgen/issues/3053
#![allow(clippy::ptr_offset_with_cast)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub mod guids;
