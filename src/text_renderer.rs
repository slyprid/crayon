use crate::glyphs::{get_glyph, glyph_pixels, GLYPH_HEIGHT, GLYPH_WIDTH};

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

/// Draw text and interpret common escape/control characters:
/// - '\n' => next line (x reset)
/// - '\r' => carriage return (x reset, same line)
/// - '\t' => tab to next TAB_WIDTH stop
/// - '\0' => ignored
///
/// Also wraps when reaching framebuffer width.
pub fn draw_text(
    frame: &mut [u8],
    fb_width: usize,
    fb_height: usize,
    x: i32,
    y: i32,
    text: &str,
    fg: [u8; 4],
    bg: Option<[u8; 4]>,
    spacing: i32,
) {
    const TAB_WIDTH: i32 = 4;
    let line_height = GLYPH_HEIGHT as i32 + 1;
    let cell_w = GLYPH_WIDTH as i32 + spacing;

    let start_x = x;
    let mut cx = x;
    let mut cy = y;

    // raw-escape parser state: previous char was '\'
    let mut escaping = false;

    // helper closure to emit one "logical" char/control
    let mut emit = |ch: char, cx: &mut i32, cy: &mut i32| {
        match ch {
            '\n' => {
                *cx = start_x;
                *cy += line_height;
            }
            '\r' => {
                *cx = start_x;
            }
            '\t' => {
                let rel = (*cx - start_x).max(0);
                let col = if cell_w > 0 { rel / cell_w } else { 0 };
                let next_tab_col = ((col / TAB_WIDTH) + 1) * TAB_WIDTH;
                *cx = start_x + next_tab_col * cell_w;
            }
            '\0' => {}
            _ => {
                if *cx + GLYPH_WIDTH as i32 > fb_width as i32 {
                    *cx = start_x;
                    *cy += line_height;
                }
                if *cy + GLYPH_HEIGHT as i32 > fb_height as i32 {
                    return;
                }
                draw_char(frame, fb_width, fb_height, *cx, *cy, ch, fg, bg);
                *cx += cell_w;
            }
        }
    };

    for ch in text.chars() {
        if escaping {
            // interpret two-char escapes from raw text
            let parsed = match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '0' => '\0',
                '\\' => '\\', // this is the part you asked for
                _ => ch,      // unknown escape: just render char literally
            };
            emit(parsed, &mut cx, &mut cy);
            escaping = false;
            continue;
        }

        if ch == '\\' {
            escaping = true;
        } else {
            emit(ch, &mut cx, &mut cy);
        }
    }

    // trailing '\' at end of string -> render it literally
    if escaping {
        emit('\\', &mut cx, &mut cy);
    }
}

/// Draw a glyph by glyph index using glyph pixel coordinate arrays.
/// Assumes `crate::glyphs::glyph_pixels(glyph_id)` returns `Option<&'static [(u8,u8)]>`.
pub fn draw_glyph(
    frame: &mut [u8],
    fb_width: usize,
    fb_height: usize,
    x: i32,
    y: i32,
    glyph_id: u16,
    fg: [u8; 4],
    _bg: Option<[u8; 4]>,
    scale_x: u32,
    scale_y: u32,
) {
    let pixels = crate::glyphs::glyph_pixels(glyph_id as usize);

    let ssx = scale_x as i32; 
    let ssy = scale_y as i32;

    for pair in pixels.chunks_exact(2) {
        let gx = pair[0];
        let gy = pair[1];

        // skip unused sentinel entries (commonly -1,-1)
        if gx < 0 || gy < 0 {
            continue;
        }

        let gx = gx as i32;
        let gy = gy as i32;

        for sy in 0..ssy {
            for sx in 0..ssx {
                let px = x + gx * ssx + sx;
                let py = y + gy * ssy + sy;

                if px < 0 || py < 0 {
                    continue;
                }

                let pxu = px as usize;
                let pyu = py as usize;
                if pxu >= fb_width || pyu >= fb_height {
                    continue;
                }

                let idx = (pyu * fb_width + pxu) * 4;
                frame[idx..idx + 4].copy_from_slice(&fg);
            }
        }
    }
}