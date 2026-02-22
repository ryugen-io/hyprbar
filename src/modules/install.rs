use anyhow::Result;
use hyprink::config::Config;
use std::path::Path;
use tokio::fs;

pub async fn install_widget(path: &Path, _config_ink: &Config) -> Result<()> {
    hyprlog::internal::info("INSTALL", &format!("Installing widget: {:?}", path));
    let data_dir = dirs::data_local_dir().unwrap().join("hyprbar/widgets");
    fs::create_dir_all(&data_dir).await?;

    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
    let target_so = data_dir.join(file_name);

    fs::copy(path, &target_so).await?;

    // Sidecar JSON must travel with the .so so plugin_loader can read metadata
    // without dlopen — keeping the entire install path free of unsafe.
    let json_source = path.with_extension("json");
    let json_target = target_so.with_extension("json");
    if json_source.exists() {
        fs::copy(&json_source, &json_target).await?;
    }

    hyprlog::internal::info("INSTALL", &format!("Widget installed to: {:?}", target_so));

    // Graceful fallback to defaults when sidecar is absent, so pre-sidecar
    // plugins still install and appear in the registry with blank metadata.
    let metadata = match fs::read_to_string(&json_target).await {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => {
            hyprlog::internal::warn("INSTALL", "No sidecar metadata found, using defaults");
            crate::modules::registry::PluginMetadata::default()
        }
    };

    let root_str = file_name.to_str().unwrap_or_default().to_string();

    use crate::modules::registry::Registry;
    let mut registry = Registry::load()?;
    registry.install(root_str, target_so, metadata)?;

    Ok(())
}
