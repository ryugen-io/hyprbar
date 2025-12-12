# Project Update: Visual Glitch Fix & Sub-Dish Support

## Recent Changes (2025-12-12)

### 1. Visual Glitch Fix (Smart Scaling)
- **Issue**: The `ks-wayland` blitter was using a hardcoded font size (16.0), ignoring dynamic scaling logic.
- **Fix**: Exposed `font_size` in `TextRenderer` and updated `blitter.rs` to use it.
- **Result**: Text now renders correctly at any bar height (e.g., 20px, 30px).

### 2. Sub-Dish Support (Instance Config)
- **Feature**: Verified and enhanced support for aliased plugin instances (e.g., `text_area.text_2`).
- **Implementation**:
    - **Core**: Added support for dot-notation (`.`) in `init_dishes` alongside legacy `#`.
    - **Plugins**: Updated `examples/text_area.rs` to implement `set_instance_config` and look up nested config tables.
    - **Config**: Verified `sink.toml` structure `[dish.text_area.alias]`.

### 3. Input Handling
- **Feature**: Added `DishEvent` enum (Enter, Leave, Motion, Click).
- **Implementation**: Wired up hit-testing in `BarRenderer` and input capture in `ks-wayland`.

### 4. Smart Layout Configuration
- New `sink.toml` options:
    - `[window]` -> `height_rows`: Define height by rows of text (e.g., 1 row, 2 rows).
    - `[window]` -> `scale_font`: Auto-scale font to fit height.
    - `[window]` -> `pixel_font`: Enable integer-step scaling for pixel fonts.
