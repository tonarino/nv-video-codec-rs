use crate::decoder::types::SurfaceFormat;

// Frame format and dimensions
#[derive(Clone)]
pub struct FrameInfo {
    output_format: SurfaceFormat,
    bpp: u32,

    /// output dimensions
    width: u32,
    luma_height: u32,
    chroma_height: u32,
    num_chroma_planes: u32,

    video_info: String,
}

impl FrameInfo {
    /// Panics when the `width` or the `luma_height` is 0.
    pub fn new(
        output_format: SurfaceFormat,
        bpp: u32,
        width: u32,
        luma_height: u32,
        video_info: String,
    ) -> Self {
        assert!(width != 0);
        assert!(luma_height != 0);

        let chroma_height =
            f64::ceil(luma_height as f64 * output_format.chroma_height_factor()) as u32;
        let num_chroma_planes = output_format.chroma_plane_count() as u32;

        Self {
            output_format,
            bpp,

            width,
            luma_height,
            chroma_height,
            num_chroma_planes,

            video_info,
        }
    }

    /// Bytes per pixel.
    pub fn bpp(&self) -> u32 {
        self.bpp
    }

    pub fn width(&self) -> u32 {
        if self.width % 2 == 1
            && matches!(self.output_format, SurfaceFormat::NV12 | SurfaceFormat::P016)
        {
            // Add 1 to odd numbers: these 4:2:0 formats require an even width.
            self.width + 1
        } else {
            self.width
        }
    }

    pub fn width_in_bytes(&self) -> usize {
        (self.width() * self.bpp()) as usize
    }

    pub fn height(&self) -> u32 {
        self.luma_height
    }

    pub fn luma_height(&self) -> u32 {
        self.luma_height
    }

    pub fn chroma_height(&self) -> u32 {
        self.chroma_height
    }

    pub fn height_in_rows(&self) -> u32 {
        self.luma_height + self.chroma_height * self.num_chroma_planes
    }

    pub fn num_chroma_planes(&self) -> u32 {
        self.num_chroma_planes
    }

    pub fn video_info(&self) -> &str {
        &self.video_info
    }

    pub fn frame_size(&self) -> u32 {
        self.width() * self.height_in_rows() * self.bpp
    }
}
