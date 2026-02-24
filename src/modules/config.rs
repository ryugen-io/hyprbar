use crate::config::BarConfig;
use crate::modules::logging::{log_error, log_warn};
use anyhow::{Context, Result};
use hypr_conf::{ConfigMetaSpec, load_toml_with_includes, resolve_config_path_strict};
use std::env;
use std::path::{Path, PathBuf};
use toml::Value;

const TYPE_VALUE: &str = "bar";
const CONFIG_EXTENSIONS: &[&str] = &["conf"];

pub fn load_bar_config() -> BarConfig {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let home_path = PathBuf::from(&home);
    let config_dir = home_path.join(".config").join("hypr");
    let Some(config_path) = resolve_bar_config_path(&config_dir) else {
        log_warn(
            "CONFIG",
            "No config with required metadata header found; using defaults.",
        );
        return BarConfig::default();
    };

    match load_recursive_config(&config_path, &home_path) {
        Ok(value) => match value.try_into() {
            Ok(cfg) => return cfg,
            Err(e) => log_error("CONFIG", &format!("Failed to deserialize config: {}", e)),
        },
        Err(e) => log_error(
            "CONFIG",
            &format!("Failed to load config with includes: {}", e),
        ),
    }

    // Missing or broken config shouldn't prevent the bar from starting — use sensible defaults.
    BarConfig::default()
}

fn resolve_bar_config_path(config_dir: &Path) -> Option<PathBuf> {
    let default_path = config_dir.join("hyprbar.conf");
    let spec = ConfigMetaSpec::for_type(TYPE_VALUE, CONFIG_EXTENSIONS);
    resolve_config_path_strict(config_dir, &default_path, &spec)
}

fn load_recursive_config(path: &Path, home_dir: &Path) -> Result<Value> {
    load_toml_with_includes(path, "include", home_dir)
        .with_context(|| format!("Failed to load config with include graph from {:?}", path))
}

pub fn get_socket_path() -> PathBuf {
    let runtime_dir = dirs::runtime_dir().unwrap_or_else(env::temp_dir);

    runtime_dir.join("hyprbar-debug.sock")
}

pub fn get_pid_file_path() -> PathBuf {
    let runtime_dir = dirs::runtime_dir().unwrap_or_else(env::temp_dir);

    runtime_dir.join("hyprbar.pid")
}
