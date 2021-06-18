pub extern crate nv_video_codec_sys;

#[macro_use]
extern crate thiserror;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate rustacuda;

#[macro_use]
extern crate rustacuda_derive;
extern crate rustacuda_core;

extern crate parking_lot;

#[macro_use]
pub mod common;
pub mod nvdecoder;
pub mod nvencoder;
