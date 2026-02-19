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
    /// Panics when the `width` or the `luma_height` is 0.
    pub fn new(
        output_format: SurfaceFormat,
        bpp: i32,
        video_info: String,
        width: u32,
        luma_height: u32,
    ) -> Self {
        assert!(width != 0);
        assert!(luma_height != 0);

        let chroma_height =
            f64::ceil(luma_height as f64 * output_format.chroma_height_factor()) as u32;
        let num_chroma_planes = output_format.chroma_plane_count() as u32;

        Self {
            output_format,
            bpp,
            video_info,

            width,
            luma_height,
            chroma_height,
            num_chroma_planes,
        }
    }

    pub fn get_width(&self) -> u32 {
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
        self.luma_height
    }

    pub fn get_frame_size(&self) -> u32 {
        self.get_width()
            * (self.luma_height + self.chroma_height * self.num_chroma_planes)
            * self.bpp as u32
    }
}
