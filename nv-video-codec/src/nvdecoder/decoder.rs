use crate::nvdecoder::{types::Codec, DecoderPacketFlags, NvDecoder};
use rustacuda::context::CurrentContext;

const DECODE_TRIES: usize = 3;

pub struct Decoder<'a> {
    decoder: Box<NvDecoder<'a>>,
    codec: Codec,
    width: u32,
    height: u32,
    frame_counter: i64,
}

impl<'a> Decoder<'a> {
    pub fn new(codec: Codec, width: u32, height: u32) -> Self {
        let current_context = CurrentContext::get_current()
            .expect("No current CUDA context present when creating an NvDecoder");
        let decoder = NvDecoder::builder(current_context, codec)
            .use_device_frame(true)
            .build()
            .expect("Couldn't construct NvDecoder");

        Self { decoder, codec, width, height, frame_counter: 0 }
    }

    /// Decodes the encoded data in `compressed` and returns the number of bytes decoded.
    pub fn decode(&mut self, compressed: &[u8], dst: &mut [u8]) -> Option<DecodeMetadata> {
        let mut num_frames_decoded = 0;
        let mut i = 0;

        while i < DECODE_TRIES && num_frames_decoded == 0 {
            num_frames_decoded = self
                .decoder
                .decode(compressed, DecoderPacketFlags::END_OF_PICTURE, self.frame_counter)
                .expect("Error decoding frame");
            self.frame_counter += 1;
            i += 1;
        }

        // TODO(bschwind)
        if let Some(frame) = self.decoder.get_frame() {
            None
        } else {
            None
        }
    }

    fn internal_decode(&mut self, compressed: &[u8]) -> Option<DecodeMetadata> {
        None
    }
}

pub struct DecodeMetadata {
    num_bytes: usize,
    is_iframe: bool,
}
