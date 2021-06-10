#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::redundant_static_lifetimes)]
// broken links from bindgen
#![allow(rustdoc::broken_intra_doc_links)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
