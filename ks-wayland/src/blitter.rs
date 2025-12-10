use k_lib::config::Cookbook;
use k_lib::factory::ColorResolver;
use ratatui::buffer::Buffer;
use ratatui::style::Color;

pub fn blit_buffer_to_pixels(
    buffer: &Buffer,
    pixels: &mut [u8],
    _width: u32,
    _height: u32,
    cookbook: &Cookbook,
) {
    // Assumption: pixels is ARGB u32 buffer (byte array)
    // 1 cell = 1 pixel for now (MVP)
    // Later we will implement font rendering where 1 cell = WxH pixels

    // Safety check
    let len = pixels.len();

    for (i, cell) in buffer.content().iter().enumerate() {
        let offset = i * 4;
        if offset + 4 > len {
            break;
        }

        // Get BG color
        let bg_color = match cell.bg {
            Color::Reset => cookbook
                .theme
                .colors
                .get("bg")
                .map(|s| {
                    let cc = ColorResolver::hex_to_color(s);
                    Color::Rgb(cc.r, cc.g, cc.b)
                })
                .unwrap_or(Color::Black),
            c => c,
        };

        // Convert Color to ARGB
        // crate::color::to_mnemomic? No, we need a helper.
        // Assuming ColorResolver or manual mapping.

        let (r, g, b) = color_to_rgb(bg_color);
        let a = 255u8;

        // Write to pixel buffer (Little Endian: B G R A)
        pixels[offset] = b;
        pixels[offset + 1] = g;
        pixels[offset + 2] = r;
        pixels[offset + 3] = a;
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
