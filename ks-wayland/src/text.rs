use anyhow::{Context, Result};
use fontdue::{Font, FontSettings};
use log::{debug, warn};
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct TextRenderer {
    font: Font,
    pub char_width: usize,
    pub char_height: usize,
}

impl TextRenderer {
    pub fn new(font_path: Option<&str>) -> Result<Self> {
        // Default fonts to try if none specified
        let default_fonts = [
            "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
            "/usr/share/fonts/liberation/LiberationMono-Regular.ttf",
            "/usr/share/fonts/TTF/LiberationMono-Regular.ttf",
        ];

        let font_data = if let Some(path_str) = font_path {
            debug!("Loading user font: {}", path_str);

            // Try direct path first
            let mut path = std::path::PathBuf::from(path_str);
            if !path.exists() {
                // Try resolving via fc-match
                if let Some(resolved) = resolve_font_via_fc_match(path_str) {
                    debug!("Resolved '{}' to '{}'", path_str, resolved);
                    path = std::path::PathBuf::from(resolved);
                }
            }

            match fs::read(&path) {
                Ok(data) => Some(data),
                Err(e) => {
                    warn!(
                        "Failed to read user font '{}' (checked path: {:?}): {}. Falling back to system defaults.",
                        path_str, path, e
                    );
                    None
                }
            }
        } else {
            None
        };

        let font_data = if let Some(data) = font_data {
            data
        } else {
            // Try defaults
            let mut data = None;
            for path in default_fonts {
                if Path::new(path).exists() {
                    debug!("Loading default font: {}", path);
                    if let Ok(d) = fs::read(path) {
                        data = Some(d);
                        break;
                    }
                }
            }

            data.ok_or_else(|| {
                anyhow::anyhow!("No default font found. Please configure a font in sink.toml")
            })?
        };

        let font = Font::from_bytes(font_data, FontSettings::default())
            .map_err(|msg| anyhow::anyhow!("{}", msg))
            .context("Failed to parse font")?;

        // Calculate metrics for a standard character (e.g., 'A') at fixed size
        // We assume a fixed grid for now (Mono)
        let size = 16.0; // Px
        let metrics = font.metrics('M', size);

        // We enforce a fixed cell size for the TUI grid.
        // For a monospace font, width should be consistent.
        // Height we can pick based on line height.
        let char_width = metrics.width.max(1) as usize; // Simplified
        let char_height = (size * 1.2) as usize; // Line height

        Ok(Self {
            font,
            char_width,
            char_height,
        })
    }

    pub fn rasterize(&self, c: char, size: f32) -> Option<(fontdue::Metrics, Vec<u8>)> {
        if c == ' ' {
            return None;
        }
        Some(self.font.rasterize(c, size))
    }
}

fn resolve_font_via_fc_match(font_name: &str) -> Option<String> {
    // Run: fc-match --format=%{file} "font_name"
    match Command::new("fc-match")
        .arg("--format=%{file}")
        .arg(font_name)
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(path);
                }
            }
        }
        Err(e) => debug!("Failed to run fc-match: {}", e),
    }
    None
}
