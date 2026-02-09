# nv-video-codec-rs

This project wraps NVIDIA Video Codec SDK 11.0 for use in Rust. Both unsafe FFI and safe
higher-level bindings are provided.

## Safe bindings overview

### Encoding

The main interface is provided by the `NvEncoder` trait.

```rust
pub trait NvEncoder {
    fn create_encoder(&mut self, encoder_params: &NV_ENC_INITIALIZE_PARAMS) -> NvEncoderResult<()>;
    fn destroy_encoder(&mut self) -> NvEncoderResult<()>;
    fn get_next_input_frame(&mut self) -> &mut NvEncInputFrame;
    fn encode_frame(
        &mut self,
        packet: &mut Vec<&[u8]>,
        pic_params: Option<NV_ENC_PIC_PARAMS>,
    ) -> NvEncoderResult<()>;
    fn end_encode(&mut self, packet: &mut Vec<&[u8]>) -> NvEncoderResult<()>;
    fn create_default_encoder_params(
        &mut self,
        codec_guid: GUID,
        preset_guid: GUID,
        tuning_info: NV_ENC_TUNING_INFO,
    ) -> NvEncoderResult<NV_ENC_INITIALIZE_PARAMS>;
}
```

Method `get_next_input_frame()` basically provides a GPU device pointer that can be used as a target
to upload the frame data. Afterwards, `encode_frame()` provides packets of the encoded frame.

#### `NvEncoderGL`

The `NvEncoder` interface is implemented by `NvEncoderGL` that can be used to feed the
encoder with OpenGL textures.

This type additionally implements `fn NvEncoderExt::encode_frame_from_data()` that combines
`get_next_input_frame()`, OpenGL texture upload and `encode_frame()` as a convenience.

Implementation-wise this type is just a thin wrapper around `NvEncoderBase` that provides basic
OpenGL texture management.

#### Beyond OpenGL

`NvEncoderBase` requires an implementation of `NvEncoderResourceManager` to allocate and release
the input buffers. At first sight it looks like it could be used to create non-OpenGL-based encoders
but internally it uses `NvEncoderBase::register_input_resources()`, which in turns relies on
`NV_ENC_INPUT_RESOURCE_TYPE_OPENGL_TEX`. It should be possible to generalize this to also work with
`NV_ENC_INPUT_RESOURCE_TYPE_CUDADEVICEPTR` and `NV_ENC_INPUT_RESOURCE_TYPE_CUDAARRAY`, if desired.

### Decoding

The decoder uses a very different architecture (possibly reflecting the underlying SDK).

```rust
struct NvDecoder { ... }
impl NvDecoder {
    pub fn builder(context: Context, codec: Codec) -> NvDecoderBuilder { ... }
    pub fn decode(
        &mut self,
        data: &[u8],
        flags: DecoderPacketFlags,
        timestamp: i64,
    ) -> Result<usize, NvDecoderError> { ... }
    pub fn get_frame(&mut self) -> Option<MappedMutexGuard<'_, Frame<'a>>> { ... }
    pub fn get_width(&self) -> u32 { ... }
    pub fn get_height(&self) -> u32 { ... }
}
```

1. `NvDecoder` can be obtained with `NvDecoder::builder().x().y().z().build()`, provided a
   `rustacuda` `Context` has been created.
1. The decoder is then fed with `NvDecoder::decode(data, ...)`, which makes it parse and process
   the `data`.
1. If decoding is successful the results can be queried with methods like `NvDecoder::get_width()`,
   `NvDecoder::get_height()` and `NvDecoder::get_frame()` to get the actual data.
   - The frame data locality can be controlled with `NvDecoderBuilder::use_device_frame()`.
   - There does not seem to be any code yet to process the device pointers when opting for
     `use_device_frame(true)`.
