use crate::widget::{PopupRequest, Widget};
use libloading::{Library, Symbol};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::collections::HashMap;
use std::path::Path;

// Type of the creator function in the plugin
type WidgetCreator = unsafe extern "Rust" fn() -> Box<dyn Widget>;

use crate::modules::registry::Registry;

/// Wraps a plugin-loaded widget to safely intercept popup methods.
/// Popup calls through the Rust vtable across .so boundaries are unreliable
/// (unstable ABI), so we only forward them if the plugin explicitly exports
/// `_has_popup() -> true` via the stable C ABI.
struct PluginWidget {
    inner: Box<dyn Widget>,
    has_popup: bool,
}

impl Widget for PluginWidget {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        state: &crate::state::BarState,
        dt: std::time::Duration,
    ) {
        self.inner.render(area, buf, state, dt);
    }

    fn update(&mut self, dt: std::time::Duration, state: &crate::state::BarState) {
        self.inner.update(dt, state);
    }

    fn width(&self, state: &crate::state::BarState) -> u16 {
        self.inner.width(state)
    }

    fn set_instance_config(&mut self, name: String) {
        self.inner.set_instance_config(name);
    }

    fn handle_event(&mut self, event: crate::event::WidgetEvent) {
        self.inner.handle_event(event);
    }

    fn popup_request(&self) -> Option<PopupRequest> {
        if !self.has_popup {
            return None;
        }
        self.inner.popup_request()
    }

    fn render_popup(&mut self, area: Rect, buf: &mut Buffer, state: &crate::state::BarState) {
        if !self.has_popup {
            return;
        }
        self.inner.render_popup(area, buf, state);
    }
}

pub struct PluginManager {
    libraries: Vec<Library>,                          // Keep libs loaded
    creators: HashMap<String, (WidgetCreator, bool)>, // (creator, has_popup)
    pub registry: Registry,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        let registry = Registry::load().unwrap_or_default();
        Self {
            libraries: Vec::new(),
            creators: HashMap::new(),
            registry,
        }
    }

    /// Loads a plugin from a path.
    /// If register_if_missing is true, it adds it to the registry (enabled).
    /// If check_enabled is true, it only loads if enabled in registry.
    pub fn load_plugin<P: AsRef<Path>>(
        &mut self,
        path: P,
        register_if_missing: bool,
        check_enabled: bool,
    ) -> anyhow::Result<()> {
        let path_ref = path.as_ref();

        // 1. Resolve path to string/filename for registry key
        // Using file_stem as key or just the filename
        let file_name = path_ref
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();

        // 2. Registry Check
        if check_enabled {
            if let Some(entry) = self.registry.plugins.get(&file_name) {
                if !entry.enabled {
                    return Ok(()); // Skip disabled
                }
            } else if register_if_missing {
                // Fallthrough to load and register
            } else {
                return Ok(()); // Not in registry, and not registering -> skip
            }
        } else if register_if_missing {
            // Fallthrough to load and register
        }

        unsafe {
            let lib = Library::new(path_ref)?;

            // Extract Metadata if registering
            if register_if_missing {
                // Try to get metadata function
                let metadata_func: Option<
                    Symbol<unsafe extern "C" fn() -> *const std::ffi::c_char>,
                > = lib.get(b"_plugin_metadata").ok();

                let metadata = if let Some(func) = metadata_func {
                    let ptr = func();
                    if !ptr.is_null() {
                        let c_str = std::ffi::CStr::from_ptr(ptr);
                        let s = c_str.to_string_lossy();
                        serde_json::from_str(&s).unwrap_or_default()
                    } else {
                        crate::modules::registry::PluginMetadata::default()
                    }
                } else {
                    crate::modules::registry::PluginMetadata::default()
                };

                // Add to registry with extracted metadata
                self.registry
                    .install(file_name.clone(), path_ref.to_path_buf(), metadata)?;
            }

            let func: Symbol<WidgetCreator> = lib.get(b"_create_widget")?;

            // Check if plugin explicitly declares popup support (stable C ABI)
            let has_popup = lib
                .get::<extern "C" fn() -> bool>(b"_has_popup")
                .map(|f| f())
                .unwrap_or(false);

            // Invoke once to get the widget name (internal name, not filename)
            // Note: Registry uses filename as key currently.
            // This might cause mismatch if filename != widget name.
            // But for simple "enable/disable", filename is safer as it maps to disk.
            let temp_widget = func();
            let name = temp_widget.name().to_string();

            // Store the raw function pointer.
            // The library is kept alive in `self.libraries`, so this is safe *enough*.
            let func_ptr = *func;

            self.libraries.push(lib);
            self.creators.insert(name.clone(), (func_ptr, has_popup));
        }
        Ok(())
    }
}

impl crate::widget::WidgetProvider for PluginManager {
    fn create_widget(&self, name: &str) -> Option<Box<dyn Widget>> {
        if let Some(&(creator, has_popup)) = self.creators.get(name) {
            unsafe {
                let inner = creator();
                return Some(Box::new(PluginWidget { inner, has_popup }));
            }
        }
        None
    }
}
