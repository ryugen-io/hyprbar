# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
just build          # Build the binary
just run            # Run with debug logging (RUST_LOG=debug)
just check          # Format check + clippy lints
just fmt            # Format code
just install        # Install binary locally
just pre-commit     # MANDATORY before commits
cargo build --release  # Optimized release build

# CLI Wrappers
just start          # Start daemon
just stop           # Stop daemon
just restart        # Restart daemon
just autostart      # Configure autostart
just debug          # Run in debug mode
just list           # List plugins
just enable <name>  # Enable plugin
just disable <name> # Disable plugin
just compile <path> # Compile widget (.rs -> .so)
just install-widget <path> # Install widget (.so)
just launch         # Run TUI/Server
just version        # Show version
```

## Architecture

**hyprbar** is a Wayland status bar built with ratatui for TUI rendering and smithay-client-toolkit for Wayland layer shell integration.

### Project Structure

Single crate with modules:
- `src/renderer/` - Rendering logic (split into mod.rs, input.rs, layout.rs, popup.rs, types.rs, widgets.rs)
- `src/wayland/` - Wayland integration via SCTK - layer shell, event handling, text rendering with fontdue
- `src/modules/` - Bootstrap, runner, config, logging, wayland_integration
- `src/bin/hyprbar.rs` - Binary entry point with CLI (clap)

### Widget System

Widgets implement the `Widget` trait (`src/widget.rs`):

```rust
pub trait Widget: Send + Sync {
    fn name(&self) -> &str;
    fn render(&self, area: Rect, buf: &mut Buffer, state: &BarState, dt: Duration);
    fn width(&self, state: &BarState) -> u16;
    fn update(&mut self, dt: Duration, state: &BarState);
    fn handle_event(&mut self, event: WidgetEvent);
    fn popup_request(&self) -> Option<PopupRequest>;
    fn render_popup(&self, area: Rect, buf: &mut Buffer, state: &BarState);
    fn set_instance_config(&mut self, alias: String);
}
```

Widgets are loaded as dynamic plugins (.so) from `~/.local/share/hyprbar/widgets/`.

### Configuration

- **Config file**: `~/.config/hypr/hyprbar.conf` (TOML)
- **Theme/Colors**: Via `hyprink` library
- **Logging**: Via `hyprlog` library

Config struct hierarchy: `BarConfig` -> `WindowConfig`, `StyleConfig`, `LayoutConfig`, `PopupConfig`

### Rendering Flow

1. `BarState` holds runtime data plus config
2. `BarRenderer` manages ratatui `Buffer`, widgets per section (left/center/right), popup state
3. `WaylandState` blits buffer to Wayland surface via SCTK layer shell

### Key Dependencies

- `hyprink`: Theme/color configuration
- `hyprlog`: Logging utilities
- `ratatui`: TUI buffer/widget rendering
- `tachyonfx`: Visual effects (fade, etc.)
- `smithay-client-toolkit`: Wayland protocols
- `fontdue`: Font rasterization

## Debug Mode

Run with `--debug` to spawn a separate terminal tailing logs via Unix socket (`hyprbar-debug.sock`).

## Code Style

- Files MUST be under 500 LOC - split into modules if larger
- Use `use crate::modules::logging::*;` for logging (all functions: log_debug, log_info, log_warn, log_error)
- Run `just pre-commit` before every commit
