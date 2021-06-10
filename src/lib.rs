pub extern crate nv_video_codec_sys;

#[macro_use]
extern crate rustacuda;

#[macro_use]
extern crate rustacuda_derive;
extern crate rustacuda_core;

#[macro_use]
pub mod common;
pub mod nvdecoder;
pub mod nvencoder;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
