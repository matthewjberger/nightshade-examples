pub const MAX_IMAGE_SIZE: usize = 4096;
pub const TRANSPARENT: u16 = 0xff00;

#[derive(Debug)]
pub enum ImageError {
    TooLarge(usize, usize),
    MissingData(String),
    Corrupt(String),
}

impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageError::TooLarge(w, h) => write!(f, "Image too large: {}x{}", w, h),
            ImageError::MissingData(msg) => write!(f, "Missing data: {}", msg),
            ImageError::Corrupt(msg) => write!(f, "Corrupt image: {}", msg),
        }
    }
}

impl std::error::Error for ImageError {}

pub struct Image {
    width: usize,
    height: usize,
    x_offset: isize,
    pixels: Vec<u16>,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Result<Self, ImageError> {
        if width > MAX_IMAGE_SIZE || height > MAX_IMAGE_SIZE {
            return Err(ImageError::TooLarge(width, height));
        }
        Ok(Self {
            width,
            height,
            x_offset: 0,
            pixels: vec![TRANSPARENT; width * height],
        })
    }

    pub fn from_buffer(buffer: &[u8]) -> Result<Self, ImageError> {
        if buffer.len() < 8 {
            return Err(ImageError::MissingData("Buffer too small".into()));
        }

        let width = u16::from_le_bytes([buffer[0], buffer[1]]) as usize;
        let height = u16::from_le_bytes([buffer[2], buffer[3]]) as usize;

        if width > MAX_IMAGE_SIZE || height > MAX_IMAGE_SIZE {
            return Err(ImageError::TooLarge(width, height));
        }

        let x_offset = i16::from_le_bytes([buffer[4], buffer[5]]) as isize;

        let mut pixels = vec![TRANSPARENT; width * height];

        let column_offsets_start = 8;
        let column_offsets_end = column_offsets_start + width * 4;

        if buffer.len() < column_offsets_end {
            return Err(ImageError::MissingData("Missing column offsets".into()));
        }

        for column_index in 0..width {
            let offset_pos = column_offsets_start + column_index * 4;
            let offset = u32::from_le_bytes([
                buffer[offset_pos],
                buffer[offset_pos + 1],
                buffer[offset_pos + 2],
                buffer[offset_pos + 3],
            ]) as usize;

            if offset >= buffer.len() {
                return Err(ImageError::Corrupt(format!(
                    "Invalid column offset {} for column {}",
                    offset, column_index
                )));
            }

            let mut pos = offset;
            loop {
                if pos >= buffer.len() {
                    break;
                }

                let row_start = buffer[pos] as usize;
                if row_start == 255 {
                    break;
                }

                if pos + 1 >= buffer.len() {
                    break;
                }
                let run_length = buffer[pos + 1] as usize;

                if row_start + run_length > height {
                    return Err(ImageError::Corrupt(format!(
                        "Run extends past image height at column {}",
                        column_index
                    )));
                }

                pos += 3;

                for row in 0..run_length {
                    if pos >= buffer.len() {
                        break;
                    }
                    let pixel_index = (row_start + row) * width + column_index;
                    if pixel_index < pixels.len() {
                        pixels[pixel_index] = buffer[pos] as u16;
                    }
                    pos += 1;
                }

                pos += 1;
            }
        }

        Ok(Self {
            width,
            height,
            x_offset,
            pixels,
        })
    }

    pub fn blit(&mut self, source: &Self, offset: (isize, isize), ignore_transparency: bool) {
        let (offset_x, offset_y) = offset;

        if offset_x >= self.width as isize || offset_y >= self.height as isize {
            return;
        }

        let y_start = if offset_y < 0 {
            (-offset_y) as usize
        } else {
            0
        };
        let x_start = if offset_x < 0 {
            (-offset_x) as usize
        } else {
            0
        };

        let y_end = if self.height as isize > source.height as isize + offset_y {
            source.height
        } else {
            (self.height as isize - offset_y) as usize
        };

        let x_end = if self.width as isize > source.width as isize + offset_x {
            source.width
        } else {
            (self.width as isize - offset_x) as usize
        };

        for source_y in y_start..y_end {
            let dest_y = (source_y as isize + offset_y) as usize;
            for source_x in x_start..x_end {
                let dest_x = (source_x as isize + offset_x) as usize;
                let source_pixel = source.pixels[source_y * source.width + source_x];

                if ignore_transparency || source_pixel != TRANSPARENT {
                    self.pixels[dest_y * self.width + dest_x] = source_pixel;
                }
            }
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn num_pixels(&self) -> usize {
        self.pixels.len()
    }

    pub fn pixels(&self) -> &[u16] {
        &self.pixels
    }

    pub fn into_pixels(self) -> Vec<u16> {
        self.pixels
    }

    pub fn x_offset(&self) -> isize {
        self.x_offset
    }
}
