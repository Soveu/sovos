#![no_std]

use core::fmt;

const COLORS_BYTES: usize = 4;
pub const FONT_X: usize = 9;
pub const FONT_Y: usize = 16;
const FONT: [u8; FONT_X * FONT_Y * 256 * COLORS_BYTES] = *include_bytes!("../font.raw");

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Mode {
    Scroll    = 0,
    Overwrite = 1,
}

#[repr(C)]
pub struct Framebuffer {
    pub base: *mut u8,

    pub scanline_width: usize,

    pub max_x: u16,
    pub max_y: u16,

    pub cursor_x: u16,
    pub cursor_y: u16,

    pub mode: Mode,
}

impl Framebuffer {
    fn advance_cursor_y(&mut self) {
        self.cursor_y += 1;
        if self.cursor_y == self.max_y {
            self.cursor_y = 0;
        }
    }

    fn newline_overwrite(&mut self) {
        self.cursor_x = 0;
        self.advance_cursor_y();
        let fb_line_base =
            self.cursor_y as usize * FONT_Y * self.scanline_width * COLORS_BYTES;
        let fb_line_len = FONT_Y * self.scanline_width * COLORS_BYTES;
        unsafe {
            let p = self.base.add(fb_line_base as usize);
            for i in 0..fb_line_len {
                p.add(i).write_volatile(0u8);
            }
        }
    }

    pub fn draw_letter_overwrite(&mut self, b: u8) {
        if b == b'\n' {
            return self.newline_overwrite();
        }
        if b == b'\t' {
            self.draw_letter_overwrite(b' ');
            self.draw_letter_overwrite(b' ');
            self.draw_letter_overwrite(b' ');
            self.draw_letter_overwrite(b' ');
            return;
        }

        let x = self.cursor_x as usize;
        let y = self.cursor_y as usize;
        let font_x = (b % 16) as usize;
        let font_y = (b / 16) as usize;
        let p = self.base;

        for fy in 0..FONT_Y {
            for fx in 0..FONT_X {
                unsafe {
                    let fb_coord =
                        (y * FONT_Y + fy) * self.scanline_width + fx + x * FONT_X;
                    let font_pixel_x = font_x * FONT_X + fx;
                    let font_pixel_y = font_y * FONT_Y + fy;
                    let font_pixel = font_pixel_y * 16 * FONT_X + font_pixel_x;
                    p.add(COLORS_BYTES * fb_coord + 0)
                        .write_volatile(FONT[COLORS_BYTES * font_pixel + 0]);
                    p.add(COLORS_BYTES * fb_coord + 1)
                        .write_volatile(FONT[COLORS_BYTES * font_pixel + 1]);
                    p.add(COLORS_BYTES * fb_coord + 2)
                        .write_volatile(FONT[COLORS_BYTES * font_pixel + 2]);
                    p.add(COLORS_BYTES * fb_coord + 3)
                        .write_volatile(FONT[COLORS_BYTES * font_pixel + 3]);
                }
            }
        }

        self.cursor_x += 1;
        if self.cursor_x == self.max_x {
            self.newline_overwrite();
        }
    }

    pub fn write_bytes(&mut self, s: &[u8]) {
        if self.mode == Mode::Scroll {
            todo!();
        }
        for x in s {
            self.draw_letter_overwrite(*x);
        }
    }
}

impl fmt::Write for Framebuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_bytes(s.as_bytes());
        Ok(())
    }
}
