use ks_core::prelude::*;
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::path::Path;

// Type of the creator function in the plugin
type DishCreator = unsafe extern "Rust" fn() -> Box<dyn Dish>;

use crate::modules::registry::Registry;

pub struct PluginManager {
    libraries: Vec<Library>, // Keep libs loaded
    creators: HashMap<String, DishCreator>,
    pub registry: Registry,
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

            let func: Symbol<DishCreator> = lib.get(b"_create_dish")?;

            // Invoke once to get the dish name (internal name, not filename)
            // Note: Registry uses filename as key currently.
            // This might cause mismatch if filename != dish name.
            // But for simple "enable/disable", filename is safer as it maps to disk.
            let temp_dish = func();
            let name = temp_dish.name().to_string();

            // Store the raw function pointer.
            // The library is kept alive in `self.libraries`, so this is safe *enough*.
            let func_ptr = *func;

            self.libraries.push(lib);
            self.creators.insert(name.clone(), func_ptr);
        }
        Ok(())
    }
}

impl ks_core::dish::DishProvider for PluginManager {
    fn create_dish(&self, name: &str) -> Option<Box<dyn Dish>> {
        if let Some(creator) = self.creators.get(name) {
            unsafe {
                return Some(creator());
            }
        }
        None
    }
}
