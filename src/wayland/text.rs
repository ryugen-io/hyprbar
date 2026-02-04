use anyhow::Result;
use cosmic_text::fontdb::{Database, Source};
use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache};
use std::path::PathBuf;
use std::process::Command;

pub struct TextRenderer {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub char_width: usize,
    pub char_height: usize,
    pub font_family: String,
    pub font_size: f32,
}

impl TextRenderer {
    pub fn new(font_path: Option<&str>, font_size: f32) -> Result<Self> {
        let swash_cache = SwashCache::new();

        // Store the requested family name (or default to "Monospace")
        let font_family = font_path.unwrap_or("Monospace").to_string();

        // 1. Determine which font to load
        let font_to_load = if let Some(path_str) = font_path {
            hyprlog::internal::debug("FONT", &format!("Requesting user font: {}", path_str));
            let path = PathBuf::from(path_str);
            if path.exists() {
                Some(path)
            } else if let Some(resolved) = resolve_font_via_fc_match(path_str) {
                hyprlog::internal::debug(
                    "FONT",
                    &format!("Resolved '{}' to '{}'", path_str, resolved),
                );
                Some(PathBuf::from(resolved))
            } else {
                hyprlog::internal::warn(
                    "FONT",
                    &format!(
                        "Could not find font '{}', falling back to defaults.",
                        path_str
                    ),
                );
                // Try to resolve monospace as fallback
                resolve_font_via_fc_match("monospace").map(PathBuf::from)
            }
        } else {
            // No font specified - resolve system monospace
            hyprlog::internal::debug("FONT", "No font specified, resolving system monospace");
            resolve_font_via_fc_match("monospace").map(PathBuf::from)
        };

        // 2. Initialize FontSystem with EMPTY database to avoid scanning system fonts (slow!)
        let mut db = Database::new();

        // Load the specific font file if found
        if let Some(path) = font_to_load {
            if let Ok(data) = std::fs::read(&path) {
                db.load_font_source(Source::Binary(std::sync::Arc::new(data)));
                hyprlog::internal::debug("FONT", &format!("Loaded font file: {:?}", path));
            } else {
                hyprlog::internal::warn("FONT", &format!("Failed to read font file: {:?}", path));
            }
        } else {
            hyprlog::internal::warn("FONT", "No specific font loaded. Text might not render.");
        }

        let mut font_system = FontSystem::new_with_locale_and_db("en-US".into(), db);

        // 3. Setup Metrics (Fixed Grid)
        let line_height = font_size * 1.2;

        // Create a dummy buffer to measure 'M' width for grid size
        let mut buffer = Buffer::new(&mut font_system, Metrics::new(font_size, line_height));

        // We set a default family to ensure we measure something reasonable.
        // Even if we loaded a custom font, we usually need to specify it by name
        // We use the requested family name here.
        buffer.set_text(
            &mut font_system,
            "M",
            &Attrs::new().family(Family::Name(&font_family)),
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut font_system, false);

        let iter = buffer.layout_runs().next();
        let char_width = if let Some(run) = iter {
            run.line_w.ceil() as usize
        } else {
            (font_size * 0.6) as usize // Fallback
        };

        // Ensure at least 1px
        let char_width = char_width.max(1);
        let char_height = line_height.ceil() as usize;

        hyprlog::internal::debug(
            "FONT",
            &format!(
                "TextRenderer initialized. Grid: {}x{}",
                char_width, char_height
            ),
        );

        // Debug loaded faces
        for face in font_system.db().faces() {
            hyprlog::internal::debug(
                "FONT",
                &format!(
                    "Loaded face: {:?} (Families: {:?})",
                    face.post_script_name, face.families
                ),
            );
        }

        Ok(Self {
            font_system,
            swash_cache,
            char_width,
            char_height,
            font_family,
            font_size,
        })
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
        Err(e) => hyprlog::internal::debug("FONT", &format!("Failed to run fc-match: {}", e)),
    }
    None
}
