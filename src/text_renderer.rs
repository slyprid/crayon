use crate::glyphs::{get_glyph, GLYPH_HEIGHT, GLYPH_WIDTH};

#[inline]
fn put_pixel(
    frame: &mut [u8],
    fb_width: usize,
    fb_height: usize,
    x: i32,
    y: i32,
    rgba: [u8; 4],
) {
    if x < 0 || y < 0 || x >= fb_width as i32 || y >= fb_height as i32 {
        return;
    }
    let idx = ((y as usize * fb_width + x as usize) * 4) as usize;
    frame[idx..idx + 4].copy_from_slice(&rgba);
}

pub fn draw_char(
    frame: &mut [u8],
    fb_width: usize,
    fb_height: usize,
    x: i32,
    y: i32,
    ch: char,
    fg: [u8; 4],
    bg: Option<[u8; 4]>,
) {
    let glyph = get_glyph(ch); // [i8; 96], row-major 8x12

    for row in 0..GLYPH_HEIGHT {
        for col in 0..GLYPH_WIDTH {
            let v = glyph[row * GLYPH_WIDTH + col];
            let px = x + col as i32;
            let py = y + row as i32;

            if v != 0 {
                put_pixel(frame, fb_width, fb_height, px, py, fg);
            } else if let Some(bg_col) = bg {
                put_pixel(frame, fb_width, fb_height, px, py, bg_col);
            }
        }
    }
}

pub fn draw_text(
    frame: &mut [u8],
    fb_width: usize,
    fb_height: usize,
    mut x: i32,
    y: i32,
    text: &str,
    fg: [u8; 4],
    bg: Option<[u8; 4]>,
    spacing: i32, // e.g. 1
) {
    for ch in text.chars() {
        draw_char(frame, fb_width, fb_height, x, y, ch, fg, bg);
        x += GLYPH_WIDTH as i32 + spacing;
    }
}