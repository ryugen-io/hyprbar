use anyhow::{Context, Result};
use k_lib::config::Cookbook;
use k_lib::logger;
use ks_core::config::SinkConfig;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;

pub fn load_sink_config(cookbook: &Cookbook) -> SinkConfig {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let config_dir = PathBuf::from(home).join(".config").join("kitchnsink");
    let config_path = config_dir.join("sink.toml"); // User requested "sink.toml"

    if config_path.exists() {
        match load_recursive_config(&config_path, &config_dir, cookbook) {
            Ok(value) => match value.try_into() {
                Ok(cfg) => return cfg,
                Err(e) => logger::log_to_terminal(
                    cookbook,
                    "error",
                    "CONFIG",
                    &format!("Failed to deserialize config: {}", e),
                ),
            },
            Err(e) => logger::log_to_terminal(
                cookbook,
                "error",
                "CONFIG",
                &format!("Failed to load config with includes: {}", e),
            ),
        }
    }

    // Fallback
    SinkConfig::default()
}

fn load_recursive_config(path: &Path, base_dir: &Path, cookbook: &Cookbook) -> Result<Value> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {:?}", path))?;

    let mut root_value: Value =
        toml::from_str(&content).with_context(|| format!("Failed to parse TOML in {:?}", path))?;

    // Check for "include" key and collect paths to avoid concurrent borrow
    let mut include_paths = Vec::new();
    if let Some(includes) = root_value.get("include").and_then(|v| v.as_array()) {
        for include_val in includes {
            if let Some(include_str) = include_val.as_str() {
                include_paths.push(include_str.to_string());
            }
        }
    }

    for include_str in include_paths {
        let include_path = base_dir.join(&include_str);
        if include_path.exists() {
            let included_value = load_recursive_config(&include_path, base_dir, cookbook)?;
            merge_toml_values(&mut root_value, included_value, cookbook);
        } else {
            logger::log_to_terminal(
                cookbook,
                "warn",
                "CONFIG",
                &format!("Included config not found: {:?}", include_path),
            );
        }
    }

    Ok(root_value)
}

fn merge_toml_values(base: &mut Value, other: Value, cookbook: &Cookbook) {
    match (base, other) {
        (Value::Table(base_map), Value::Table(other_map)) => {
            for (k, v) in other_map {
                match base_map.get_mut(&k) {
                    Some(base_val) => {
                        // Warn on duplicate keys (except 'include' which we processed, or specific merges)
                        if k != "include" && !k.starts_with("dish") {
                            // Suppress warning for tables that we are about to merge deeply
                            if !base_val.is_table() || !v.is_table() {
                                logger::log_to_terminal(
                                    cookbook,
                                    "warn",
                                    "CONFIG",
                                    &format!(
                                        "Duplicate key '{}' being overwritten during merge.",
                                        k
                                    ),
                                );
                            }
                        }
                        merge_toml_values(base_val, v, cookbook)
                    }
                    None => {
                        base_map.insert(k, v);
                    }
                }
            }
        }
        (base_val, other_val) => {
            *base_val = other_val;
        }
    }
}

pub fn get_socket_path() -> PathBuf {
    let runtime_dir = env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::temp_dir());
    runtime_dir.join("kitchnsink-debug.sock")
}
