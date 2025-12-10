use crate::text::TextRenderer;
use k_lib::config::Cookbook;
use k_lib::factory::ColorResolver;
use ratatui::buffer::Buffer;
use ratatui::style::Color;

pub fn blit_buffer_to_pixels(
    buffer: &Buffer,
    pixels: &mut [u8],
    width: u32,
    height: u32,
    cookbook: &Cookbook,
    text_renderer: &TextRenderer,
    bg_color_hex: &str,
) {
    // Assumption: pixels is ARGB u32 buffer (byte array)

    // 1. Fill entire background with default BG color
    // 1. Fill entire background with default BG color
    let default_bg_color = ColorResolver::hex_to_color(bg_color_hex);
    let (bg_r, bg_g, bg_b) = (default_bg_color.r, default_bg_color.g, default_bg_color.b);

    for chunk in pixels.chunks_exact_mut(4) {
        chunk[0] = bg_b;
        chunk[1] = bg_g;
        chunk[2] = bg_r;
        chunk[3] = 255;
    }

    // 2. Calculate offsets to center the grid
    let area = buffer.area();
    let grid_width = area.width as usize;
    let grid_height = area.height as usize;

    let char_w = text_renderer.char_width;
    let char_h = text_renderer.char_height;

    let total_content_height = grid_height * char_h;
    let start_y_offset = if (height as usize) > total_content_height {
        (height as usize - total_content_height) / 2
    } else {
        0
    };

    // Safety check
    let len = pixels.len();

    for (i, cell) in buffer.content().iter().enumerate() {
        let x_cell = i % grid_width;
        let y_cell = i / grid_width;

        let x_px_start = x_cell * char_w;
        let y_px_start = start_y_offset + y_cell * char_h;

        // Skip if outside bounds
        if x_px_start >= width as usize || y_px_start >= height as usize {
            continue;
        }

        // Get colors
        let bg_color = match cell.bg {
            Color::Reset => Color::Rgb(bg_r, bg_g, bg_b), // Already filled, but needed for blending if glyph
            c => c,
        };

        let fg_color = match cell.fg {
            Color::Reset => cookbook
                .theme
                .colors
                .get("fg")
                .map(|s| {
                    let cc = ColorResolver::hex_to_color(s);
                    Color::Rgb(cc.r, cc.g, cc.b)
                })
                .unwrap_or(Color::White),
            c => c,
        };

        // Rasterize Glyph
        let symbol = cell.symbol();
        let c = symbol.chars().next().unwrap_or(' ');

        let raster_result = text_renderer.rasterize(c, 16.0);

        // Check if we need to draw background
        if cell.bg != Color::Reset {
            let (cell_bg_r, cell_bg_g, cell_bg_b) = color_to_rgb(bg_color);
            for y in 0..char_h {
                for x in 0..char_w {
                    let px_x = x_px_start + x;
                    let px_y = y_px_start + y;

                    if px_x >= width as usize || px_y >= height as usize {
                        continue;
                    }

                    let offset = (px_y * width as usize + px_x) * 4;
                    if offset + 4 > len {
                        continue;
                    }

                    pixels[offset] = cell_bg_b;
                    pixels[offset + 1] = cell_bg_g;
                    pixels[offset + 2] = cell_bg_r;
                    pixels[offset + 3] = 255;
                }
            }
        }

        // Draw Glyph on top (Blending)
        if let Some((metrics, bitmap)) = &raster_result
            && !bitmap.is_empty()
        {
            let (fg_r, fg_g, fg_b) = color_to_rgb(fg_color);

            for y in 0..char_h {
                for x in 0..char_w {
                    let px_x = x_px_start + x;
                    let px_y = y_px_start + y;

                    if px_x >= width as usize || px_y >= height as usize {
                        continue;
                    }

                    // Map x/y to bitmap coords
                    let g_x = x as i32;
                    let g_y = y as i32 - (char_h as i32 - metrics.height as i32) / 2;

                    if g_x >= 0
                        && g_x < metrics.width as i32
                        && g_y >= 0
                        && g_y < metrics.height as i32
                    {
                        let b_idx = (g_y * metrics.width as i32 + g_x) as usize;
                        if b_idx < bitmap.len() {
                            let alpha = bitmap[b_idx];
                            if alpha > 0 {
                                let offset = (px_y * width as usize + px_x) * 4;
                                if offset + 4 <= len {
                                    // Read current background (either Fill or CellBG)
                                    let base_b = pixels[offset];
                                    let base_g = pixels[offset + 1];
                                    let base_r = pixels[offset + 2];

                                    let a_f = alpha as f32 / 255.0;
                                    let inv_a = 1.0 - a_f;

                                    let r = (fg_r as f32 * a_f + base_r as f32 * inv_a) as u8;
                                    let g = (fg_g as f32 * a_f + base_g as f32 * inv_a) as u8;
                                    let b = (fg_b as f32 * a_f + base_b as f32 * inv_a) as u8;

                                    pixels[offset] = b;
                                    pixels[offset + 1] = g;
                                    pixels[offset + 2] = r;
                                    pixels[offset + 3] = 255;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn color_to_rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Black => (0, 0, 0),
        Color::Red => (255, 0, 0),
        Color::Green => (0, 255, 0),
        Color::Yellow => (255, 255, 0),
        Color::Blue => (0, 0, 255),
        Color::Magenta => (255, 0, 255),
        Color::Cyan => (0, 255, 255),
        Color::Gray => (128, 128, 128),
        Color::DarkGray => (64, 64, 64),
        Color::LightRed => (255, 128, 128),
        Color::LightGreen => (128, 255, 128),
        Color::LightYellow => (255, 255, 128),
        Color::LightBlue => (128, 128, 255),
        Color::LightMagenta => (255, 128, 255),
        Color::LightCyan => (128, 255, 255),
        Color::White => (255, 255, 255),
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Indexed(i) => (i, i, i), // TODO: Palette lookup
        Color::Reset => (0, 0, 0),
    }
}
