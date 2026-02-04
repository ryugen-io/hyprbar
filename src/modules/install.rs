use anyhow::Result;
use hyprink::config::Config;
use std::path::Path;
use tokio::fs;

pub async fn install_widget(path: &Path, _config_ink: &Config) -> Result<()> {
    // Install .so widget plugin
    hyprlog::internal::info("INSTALL", &format!("Installing widget: {:?}", path));
    let data_dir = dirs::data_local_dir().unwrap().join("hyprbar/widgets");
    fs::create_dir_all(&data_dir).await?;

    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
    let target = data_dir.join(file_name);

    fs::copy(path, &target).await?;
    hyprlog::internal::info("INSTALL", &format!("Widget installed to: {:?}", target));

    // Update Registry
    let root_str = file_name.to_str().unwrap_or_default().to_string();

    // Load library to get metadata
    let metadata = unsafe {
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
                hyprlog::internal::warn(
                    "INSTALL",
                    &format!("Failed to read metadata from plugin: {}", e),
                );
                crate::modules::registry::PluginMetadata::default()
            }
        }
    };

    use crate::modules::registry::Registry;
    let mut registry = Registry::load()?;
    registry.install(root_str, target, metadata)?;

    Ok(())
}
