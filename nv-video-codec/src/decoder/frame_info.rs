use crate::decoder::types::SurfaceFormat;

// Frame format and dimensions
#[derive(Clone)]
pub struct FrameInfo {
    pub output_format: SurfaceFormat,
    pub video_info: String,
    pub(super) bpp: i32,

    /// output dimensions
    pub(super) width: u32,
    pub(super) luma_height: u32,
    pub(super) chroma_height: u32,
    pub(super) num_chroma_planes: u32,
}

impl FrameInfo {
    pub fn new() -> Self {
        Self {
            output_format: SurfaceFormat::NV12,
            bpp: 1,
            video_info: "".to_string(),

            width: 0,
            luma_height: 0,
            chroma_height: 0,
            num_chroma_planes: 0,
        }
    }

    pub fn get_width(&self) -> u32 {
        assert!(self.width != 0);
        if self.width % 2 == 1
            && matches!(self.output_format, SurfaceFormat::NV12 | SurfaceFormat::P016)
        {
            // Add 1 to odd numbers: these 4:2:0 formats require an even width.
            self.width + 1
        } else {
            self.width
        }
    }

    pub fn get_height(&self) -> u32 {
        assert!(self.luma_height != 0);
        self.luma_height
    }

    pub fn get_frame_size(&self) -> u32 {
        assert!(self.width != 0);
        self.get_width()
            * (self.luma_height + self.chroma_height * self.num_chroma_planes)
            * self.bpp as u32
    }
}

impl Default for FrameInfo {
    fn default() -> Self {
        Self::new()
    }
}
