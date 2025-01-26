mod constants;
use core::{
    fmt::{self, Write},
    ptr,
};
use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use constants::font_constants;
use constants::font_constants::{BACKUP_CHAR, CHAR_RASTER_HEIGHT, FONT_WEIGHT};
use noto_sans_mono_bitmap::{get_raster, RasterizedChar};

const LINE_SPACING: usize = 2;
const LETTER_SPACING: usize = 0;
const BORDER_PADDING: usize = 1;

/// Returns the raster of the given char or the raster of [font_constants::BACKUP_CHAR].
fn get_char_raster(c: char) -> RasterizedChar {
    get_raster(c, FONT_WEIGHT, CHAR_RASTER_HEIGHT)
        .unwrap_or_else(|| get_raster(BACKUP_CHAR, FONT_WEIGHT, CHAR_RASTER_HEIGHT)
            .expect("Should get raster of backup char."))
}

/// Allows logging text to a pixel-based framebuffer.
pub struct FrameBufferWriter {
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
    x_pos: usize,
    y_pos: usize,
    text_color: [u8; 3],
}

impl FrameBufferWriter {
    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let mut logger = Self {
            framebuffer,
            info,
            x_pos: BORDER_PADDING,
            y_pos: BORDER_PADDING,
            text_color: [255, 255, 255],
        };
        logger.clear(); // Reset framebuffer at initialization
        logger
    }

    pub fn set_text_color(&mut self, color: [u8; 3]) {
        self.text_color = color;
    }

    fn newline(&mut self) {
        self.y_pos += font_constants::CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
        self.carriage_return();
    }

    fn carriage_return(&mut self) {
        self.x_pos = BORDER_PADDING;
    }

    /// Erases all text on the screen. Resets self.x_pos and self.y_pos.
    pub fn clear(&mut self) {
        self.x_pos = BORDER_PADDING;
        self.y_pos = BORDER_PADDING;
        self.framebuffer.fill(0);
    }

    fn width(&self) -> usize {
        self.info.width
    }

    fn height(&self) -> usize {
        self.info.height
    }

    /// Writes a single char to the framebuffer. Takes care of special control characters,
    /// such as newlines and carriage returns.
    fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            'c' => {
            // Change the color to blue
            self.set_text_color([0, 0, 255]); // Example RGB for blue
        }
            '\t' => {
        // Handle a tab by moving the x position forward
        let tab_size = 4; // Define how many spaces a tab represents
        let tab_width = font_constants::CHAR_RASTER_WIDTH * tab_size;

        // Move the x position forward, making sure not to overflow the line
        self.x_pos += tab_width;

        // If the x position goes beyond the screen width, move to the next line
        if self.x_pos >= self.width() {
            self.newline(); // Move to a new line
        }
    }
            '\r' => self.carriage_return(),
            c => {
                let new_xpos = self.x_pos + font_constants::CHAR_RASTER_WIDTH;
                if new_xpos >= self.width() {
                    self.newline();
                }

                let new_ypos = self.y_pos + font_constants::CHAR_RASTER_HEIGHT.val() + BORDER_PADDING;
                if new_ypos >= self.height() {
                    self.clear();
                }

                self.write_rendered_char(get_char_raster(c));
            }
        }
    }

    /// Prints a rendered char into the framebuffer.
    /// Updates self.x_pos.
    fn write_rendered_char(&mut self, rendered_char: RasterizedChar) {
        for (y, row) in rendered_char.raster().iter().enumerate() {
            for (x, intensity) in row.iter().enumerate() {
                self.write_pixel(self.x_pos + x, self.y_pos + y, *intensity);
            }
        }
        self.x_pos += rendered_char.width() + LETTER_SPACING;
    }
    


   
    
    fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
        if intensity == 0 {
            // Skip rendering for the background
            return;
        }
    
        let pixel_offset = y * self.info.stride + x;
        let color = match self.info.pixel_format {
            PixelFormat::Rgb => [self.text_color[0], self.text_color[1], self.text_color[2], 0],
            PixelFormat::Bgr => [self.text_color[2], self.text_color[1], self.text_color[0], 0],
            PixelFormat::U8 => [if intensity > 200 { 0xf } else { 0 }, 0, 0, 0],
            other => {
                self.info.pixel_format = PixelFormat::Rgb;
                panic!("Pixel format {:?} not supported in logger", other);
            }
        };
        let bytes_per_pixel = self.info.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        let _ = unsafe { ptr::read_volatile(&self.framebuffer[byte_offset]);
        };
    }
    
}

unsafe impl Send for FrameBufferWriter {}
unsafe impl Sync for FrameBufferWriter {}

impl Write for FrameBufferWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}
