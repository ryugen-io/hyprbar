use anyhow::Result;
use k_lib::config::Cookbook;
use k_lib::logger;
use std::path::Path;
use tokio::fs;

pub async fn load_dish(path: &Path, cookbook: &Cookbook) -> Result<()> {
    // Install .dish
    logger::log_to_terminal(
        cookbook,
        "info",
        "LOAD",
        &format!("Loading dish: {:?}", path),
    );
    let data_dir = dirs::data_local_dir().unwrap().join("kitchnsink/dishes");
    fs::create_dir_all(&data_dir).await?;

    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
    let target = data_dir.join(file_name);

    fs::copy(path, &target).await?;
    logger::log_to_terminal(
        cookbook,
        "info",
        "LOAD",
        &format!("Dish installed to: {:?}", target),
    );

    // Update Registry
    // Note: Registry uses blocking IO, but we are in async context.
    // For a CLI tool this is acceptable, or use tokio::task::spawn_blocking.
    // Given it's a small file, blocking is fine here.
    let root_str = file_name.to_str().unwrap_or_default().to_string();
    // Registry expects the key to be the file name (e.g. "battery.dish")
    // or relative path? PluginManager uses file_name() as key.
    // Let's use file_name() string.

    // Load library to get metadata
    let metadata = unsafe {
        // We must load it to read the symbol
        // libloading requires full path
        match libloading::Library::new(&target) {
            Ok(lib) => {
                let func: Option<
                    libloading::Symbol<unsafe extern "C" fn() -> *const std::ffi::c_char>,
                > = lib.get(b"_plugin_metadata").ok();
                if let Some(f) = func {
                    let ptr = f();
                    if !ptr.is_null() {
                        let c_str = std::ffi::CStr::from_ptr(ptr);
                        serde_json::from_str(&c_str.to_string_lossy()).unwrap_or_default()
                    } else {
                        crate::modules::registry::PluginMetadata::default()
                    }
                } else {
                    crate::modules::registry::PluginMetadata::default()
                }
            }
            Err(e) => {
                logger::log_to_terminal(
                    cookbook,
                    "warn",
                    "LOAD",
                    &format!("Failed to read metadata from plugin: {}", e),
                );
                crate::modules::registry::PluginMetadata::default()
            }
        }
    };

    // We need to import Registry.
    // Use crate::modules::registry::Registry;
    // But we are in modules/install.rs, so crate::modules::registry works.
    use crate::modules::registry::Registry;
    let mut registry = Registry::load()?;
    registry.install(root_str, target, metadata)?;

    Ok(())
}
