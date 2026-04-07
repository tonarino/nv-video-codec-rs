# nv-video-codec-rs

This project wraps NVIDIA Video Codec SDK 11.0 for use in Rust. Both unsafe FFI and safe
higher-level bindings are provided.

## Setup

The project requires the Video Codec SDK to be installed.

### Arch Linux

The SDK libraries are available in the official `nvidia-utils` package.

```
sudo pacman -S nvidia-utils
```

## Safe bindings overview

### Encoding

The main interface is provided by `NvEncoder<ResourceManager>`.

```rust
impl<ResourceManager> NvEncoder<ResourceManager> {
    pub fn create_encoder(&mut self, encoder_params: &NV_ENC_INITIALIZE_PARAMS) -> NvEncoderResult<()>;
    pub fn destroy_encoder(&mut self) -> NvEncoderResult<()>;
    pub fn get_next_input_frame(&mut self) -> &mut NvEncInputFrame;
    pub fn get_next_input_resource(&mut self) -> &mut ResourceManager::InputResource;
    pub fn encode_frame(
        &mut self,
        packet: &mut Vec<&[u8]>,
        pic_params: Option<NV_ENC_PIC_PARAMS>,
    ) -> NvEncoderResult<()>;
    pub fn end_encode(&mut self, packet: &mut Vec<&[u8]>) -> NvEncoderResult<()>;
    pub fn create_default_encoder_params(
        &mut self,
        codec_guid: GUID,
        preset_guid: GUID,
        tuning_info: NV_ENC_TUNING_INFO,
    ) -> NvEncoderResult<NV_ENC_INITIALIZE_PARAMS>;
}
```

Method `get_next_input_resource()` provides a GPU resource that should be used as a target
to upload the frame data. Afterwards, `encode_frame()` provides packets of the encoded frame.

### NvEncoderResourceManager

This trait specifies what kind of resources are used to back the frames used for encoding.

```rust
pub trait NvEncoderResourceManager {
    type InputResource;

    fn allocate_input_buffers(
        encoder: &mut NvEncoder<Self>,
        num_input_buffers: u32,
    ) -> Result<(), NvEncoderError>;

    fn release_input_buffers(encoder: &mut NvEncoder<Self>) -> Result<(), NvEncoderError>;
}
```

#### `NvEncoderGL`

This is a thin wrapper over `NvEncoder<NvEncoderGLResourceManager>` that can be used to feed the encoder with OpenGL textures.

This type additionally implements `fn NvEncoderExt::encode_frame_from_data()` that combines
`get_next_input_frame()`, OpenGL texture upload and `encode_frame()` as a convenience.

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
