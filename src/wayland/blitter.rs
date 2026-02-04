use crate::wayland::text::TextRenderer;
use cosmic_text::{Attrs, Buffer, Color as CosmicColor, Family, Metrics, Shaping};
use hyprink::config::Config;
use hyprink::factory::ColorResolver;

use ratatui::buffer::Buffer as RatatuiBuffer;
use ratatui::style::Color;

pub fn blit_buffer_to_pixels(
    buffer: &RatatuiBuffer,
    pixels: &mut [u8],
    width: u32,
    height: u32,
    config_ink: &Config,
    text_renderer: &mut TextRenderer, // Mutable for SwashCache/FontSystem
    bg_color_hex: &str,
) {
    // 1. Fill background (same as before)
    let default_bg_color = ColorResolver::hex_to_color(bg_color_hex);
    let (bg_r, bg_g, bg_b) = (default_bg_color.r, default_bg_color.g, default_bg_color.b);

    for chunk in pixels.chunks_exact_mut(4) {
        chunk[0] = bg_b;
        chunk[1] = bg_g;
        chunk[2] = bg_r;
        chunk[3] = 255;
    }

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

    let fb_width = width as usize;
    let fb_height = height as usize;

    // 2. Iterate by ROW to find contiguous text runs
    for y_cell in 0..grid_height {
        let mut current_run = String::new();
        let mut current_fg = Color::Reset;
        let mut current_bg = Color::Reset;
        let mut run_start_x = 0;

        for x_cell in 0..grid_width {
            let i = y_cell * grid_width + x_cell;
            let cell = &buffer.content()[i];

            let cell_fg = cell.fg;
            let cell_bg = cell.bg;
            let symbol = cell.symbol();

            // Check if we should break the run
            if current_run.is_empty() {
                current_run.push_str(symbol);
                current_fg = cell_fg;
                current_bg = cell_bg;
                run_start_x = x_cell;
            } else if cell_fg == current_fg && cell_bg == current_bg {
                current_run.push_str(symbol);
            } else {
                // Style mismatch, flush current run
                flush_run(
                    &current_run,
                    run_start_x,
                    x_cell - run_start_x,
                    y_cell,
                    current_fg,
                    current_bg,
                    text_renderer,
                    config_ink,
                    pixels,
                    fb_width,
                    fb_height,
                    start_y_offset,
                    char_w,
                    char_h,
                    bg_r,
                    bg_g,
                    bg_b,
                );

                // Start new run
                current_run.clear();
                current_run.push_str(symbol);
                current_fg = cell_fg;
                current_bg = cell_bg;
                run_start_x = x_cell;
            }
        }

        // Flush end of row
        if !current_run.is_empty() {
            flush_run(
                &current_run,
                run_start_x,
                grid_width - run_start_x,
                y_cell,
                current_fg,
                current_bg,
                text_renderer,
                config_ink,
                pixels,
                fb_width,
                fb_height,
                start_y_offset,
                char_w,
                char_h,
                bg_r,
                bg_g,
                bg_b,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn flush_run(
    text: &str,
    start_x_cell: usize,
    width_in_cells: usize,
    y_cell: usize,
    fg: Color,
    bg: Color,
    text_renderer: &mut TextRenderer,
    config_ink: &Config,
    pixels: &mut [u8],
    width: usize,
    height: usize,
    start_y_offset: usize,
    char_w: usize,
    char_h: usize,
    _default_bg_r: u8,
    _default_bg_g: u8,
    _default_bg_b: u8,
) {
    let resolved_fg = resolve_color(fg, config_ink, Color::White);
    let resolved_bg = resolve_color(bg, config_ink, Color::Reset); // Reset means transparent/default

    /* debug!(
        "Flush run: '{}' (fg={:?}->{:?}, bg={:?}->{:?})",
        text, fg, resolved_fg, bg, resolved_bg
    ); */

    // Draw Background rect
    if bg != Color::Reset {
        let (br, bg, bb) = color_to_rgb(resolved_bg);
        // Correctly use the passed width in cells
        let run_width_cells = width_in_cells;

        let rect_x = start_x_cell * char_w;
        let rect_y = start_y_offset + y_cell * char_h;
        let rect_w = run_width_cells * char_w;
        let rect_h = char_h;

        for y in 0..rect_h {
            for x in 0..rect_w {
                let px_x = rect_x + x;
                let px_y = rect_y + y;
                if px_x >= width || px_y >= height {
                    continue;
                }

                let offset = (px_y * width + px_x) * 4;
                if offset + 4 <= pixels.len() {
                    pixels[offset] = bb;
                    pixels[offset + 1] = bg;
                    pixels[offset + 2] = br;
                    pixels[offset + 3] = 255;
                }
            }
        }
    }

    // Draw Text with Cosmic Text
    let font_size = text_renderer.font_size;
    let line_height = font_size * 1.2;

    // debug!("Flush run: '{}' (bg={:?}) Cells: {} ", text, bg, width_in_cells);

    let mut buffer = Buffer::new(
        &mut text_renderer.font_system,
        Metrics::new(font_size, line_height),
    );

    // Shape text
    buffer.set_text(
        &mut text_renderer.font_system,
        text,
        &Attrs::new().family(Family::Name(&text_renderer.font_family)),
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(&mut text_renderer.font_system, false);

    // Rasterize
    let (fr, fg, fb) = color_to_rgb(resolved_fg);
    let cosmic_color = CosmicColor::rgb(fr, fg, fb);

    // Run callback
    let draw_x_base = (start_x_cell * char_w) as i32;
    let draw_y_base = (start_y_offset + y_cell * char_h) as i32;
    #[allow(clippy::unnecessary_cast)]
    let stride = width as usize;
    let width_i32 = width as i32;
    let height_i32 = height as i32;

    buffer.draw(
        &mut text_renderer.font_system,
        &mut text_renderer.swash_cache,
        cosmic_color,
        |x, y, w, h, color| {
            for dy in 0..h {
                for dx in 0..w {
                    let px_x = draw_x_base + x + dx as i32;
                    let px_y = draw_y_base + y + dy as i32;

                    if px_x < 0 || px_x >= width_i32 || px_y < 0 || px_y >= height_i32 {
                        continue;
                    }

                    let offset = (px_y as usize * stride + px_x as usize) * 4;
                    if offset + 4 > pixels.len() {
                        continue;
                    }

                    let alpha = color.a();
                    if alpha > 0 {
                        let bg_b = pixels[offset];
                        let bg_g = pixels[offset + 1];
                        let bg_r = pixels[offset + 2];

                        let a_f = alpha as f32 / 255.0;
                        let inv_a = 1.0 - a_f;

                        let r = (color.r() as f32 * a_f + bg_r as f32 * inv_a) as u8;
                        let g = (color.g() as f32 * a_f + bg_g as f32 * inv_a) as u8;
                        let b = (color.b() as f32 * a_f + bg_b as f32 * inv_a) as u8;

                        pixels[offset] = b;
                        pixels[offset + 1] = g;
                        pixels[offset + 2] = r;
                        pixels[offset + 3] = 255;
                    }
                }
            }
        },
    );
}

// Helpers
fn resolve_color(c: Color, config_ink: &Config, default: Color) -> Color {
    match c {
        Color::Reset => {
            if default == Color::White {
                // Try looking up 'fg' from theme
                config_ink
                    .theme
                    .colors
                    .get("fg")
                    .map(|s| {
                        let cc = ColorResolver::hex_to_color(s);
                        Color::Rgb(cc.r, cc.g, cc.b)
                    })
                    .unwrap_or(Color::White)
            } else {
                default
            }
        }
        c => c,
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
        Color::Indexed(i) => (i, i, i),
        Color::Reset => (0, 0, 0),
    }
}
