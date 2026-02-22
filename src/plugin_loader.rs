use crate::modules::registry::{PluginMetadata, Registry};
use crate::widget::{PopupRequest, Widget};
use libloading::Library;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::collections::HashMap;
use std::path::Path;

type WidgetCreator = unsafe extern "Rust" fn() -> Box<dyn Widget>;

/// Popup calls through the Rust vtable across .so boundaries are unreliable
/// (unstable ABI), so we gate them behind an explicit opt-in from the sidecar JSON.
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

/// Graceful fallback to defaults when .json sidecar is absent, so
/// pre-sidecar plugins (compiled before this change) still load correctly.
fn load_metadata(so_path: &Path) -> PluginMetadata {
    let json_path = so_path.with_extension("json");
    match std::fs::read_to_string(&json_path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => PluginMetadata::default(),
    }
}

pub struct PluginManager {
    // Must outlive `creators` — dropping a Library invalidates all fn pointers from it.
    libraries: Vec<Library>,
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

    /// `register_if_missing` — auto-register unknown plugins (enabled by default).
    /// `check_enabled` — skip plugins disabled in the registry.
    pub fn load_plugin<P: AsRef<Path>>(
        &mut self,
        path: P,
        register_if_missing: bool,
        check_enabled: bool,
    ) -> anyhow::Result<()> {
        let path_ref = path.as_ref();

        // Registry uses filename as key because it maps 1:1 to disk
        // (widget internal name could differ from filename).
        let file_name = path_ref
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();

        // Early-out avoids dlopen for disabled plugins, saving startup time.
        if check_enabled {
            if let Some(entry) = self.registry.plugins.get(&file_name) {
                if !entry.enabled {
                    return Ok(());
                }
            } else if !register_if_missing {
                return Ok(());
            }
        }

        // Metadata comes from a sidecar JSON file — avoids needing C ABI exports
        // and eliminates ~25 lines of unsafe that the old _plugin_metadata path required.
        let metadata = load_metadata(path_ref);
        let has_popup = metadata.has_popup;

        if register_if_missing {
            self.registry
                .install(file_name.clone(), path_ref.to_path_buf(), metadata)?;
        }

        // SAFETY: The .so is produced by our build pipeline (`compile_widget`) and is
        // guaranteed to export `_create_widget` with the correct signature.
        let lib = unsafe { Library::new(path_ref)? };
        let func = unsafe { lib.get::<WidgetCreator>(b"_create_widget")? };

        // SAFETY: `lib` is pushed to `self.libraries` below, so the function pointer
        // and the vtable of the returned Box<dyn Widget> remain valid for the
        // lifetime of PluginManager.
        let temp_widget = unsafe { func() };
        let name = temp_widget.name().to_string();

        let func_ptr = *func;
        self.libraries.push(lib);
        self.creators.insert(name.clone(), (func_ptr, has_popup));

        Ok(())
    }
}

impl crate::widget::WidgetProvider for PluginManager {
    fn create_widget(&self, name: &str) -> Option<Box<dyn Widget>> {
        if let Some(&(creator, has_popup)) = self.creators.get(name) {
            // SAFETY: The corresponding Library is kept alive in `self.libraries`,
            // so the function pointer is still valid.
            let inner = unsafe { creator() };
            return Some(Box::new(PluginWidget { inner, has_popup }));
        }
        None
    }
}
