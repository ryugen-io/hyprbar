use ks_core::prelude::*;
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::path::Path;

// Type of the creator function in the plugin
type DishCreator = unsafe extern "Rust" fn() -> Box<dyn Dish>;

pub struct PluginManager {
    libraries: Vec<Library>, // Keep libs loaded
    creators: HashMap<String, DishCreator>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            libraries: Vec::new(),
            creators: HashMap::new(),
        }
    }

    pub fn load_plugin<P: AsRef<Path>>(&mut self, path: P) -> anyhow::Result<()> {
        unsafe {
            let lib = Library::new(path.as_ref())?;
            let func: Symbol<DishCreator> = lib.get(b"_create_dish")?;

            // Invoke once to get the name
            let temp_dish = func();
            let name = temp_dish.name().to_string();

            // Store the raw function pointer.
            // The library is kept alive in `self.libraries`, so this is safe *enough*.
            let func_ptr = *func;

            self.libraries.push(lib);
            self.creators.insert(name.clone(), func_ptr);
            log::info!("Loaded plugin dish: {}", name);
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
