use crate::config::BarConfig;
use anyhow::{Context, Result};
use hyprink::config::Config;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;

pub fn load_bar_config(_config_ink: &Config) -> BarConfig {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let config_dir = PathBuf::from(home).join(".config").join("hypr");
    let config_path = config_dir.join("hyprbar.conf");

    if config_path.exists() {
        match load_recursive_config(&config_path, &config_dir) {
            Ok(value) => match value.try_into() {
                Ok(cfg) => return cfg,
                Err(e) => hyprlog::internal::error(
                    "CONFIG",
                    &format!("Failed to deserialize config: {}", e),
                ),
            },
            Err(e) => hyprlog::internal::error(
                "CONFIG",
                &format!("Failed to load config with includes: {}", e),
            ),
        }
    }

    // Missing or broken config shouldn't prevent the bar from starting — use sensible defaults.
    BarConfig::default()
}

fn load_recursive_config(path: &Path, base_dir: &Path) -> Result<Value> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {:?}", path))?;

    let mut root_value: Value =
        toml::from_str(&content).with_context(|| format!("Failed to parse TOML in {:?}", path))?;

    // Includes must be collected before mutation so we don't borrow root_value while modifying it.
    let mut include_patterns = Vec::new();
    if let Some(includes) = root_value.get("include").and_then(|v| v.as_array()) {
        for include_val in includes {
            if let Some(include_str) = include_val.as_str() {
                include_patterns.push(include_str.to_string());
            }
        }
    }

    // Hyprland-style includes support globs, so each pattern may resolve to multiple files.
    for pattern_str in include_patterns {
        let pattern_path = base_dir.join(&pattern_str);

        // glob::glob requires &str, but Path gives OsStr — lossy conversion is acceptable since config paths are always UTF-8.
        let pattern = pattern_path.to_string_lossy();

        match glob::glob(&pattern) {
            Ok(paths) => {
                let mut found_any = false;
                for entry in paths {
                    match entry {
                        Ok(path) => {
                            found_any = true;
                            // Included files may themselves contain includes — recursion mirrors Hyprland's source directive.
                            let included_value = load_recursive_config(&path, base_dir)?;
                            merge_toml_values(&mut root_value, included_value);
                        }
                        Err(e) => {
                            hyprlog::internal::warn(
                                "CONFIG",
                                &format!("Glob error for pattern '{}': {}", pattern_str, e),
                            );
                        }
                    }
                }

                if !found_any {
                    hyprlog::internal::warn(
                        "CONFIG",
                        &format!("No files matched include pattern: '{}'", pattern_str),
                    );
                }
            }
            Err(e) => {
                hyprlog::internal::warn(
                    "CONFIG",
                    &format!("Invalid glob pattern '{}': {}", pattern_str, e),
                );
            }
        }
    }

    Ok(root_value)
}

fn merge_toml_values(base: &mut Value, other: Value) {
    match (base, other) {
        (Value::Table(base_map), Value::Table(other_map)) => {
            for (k, v) in other_map {
                match base_map.get_mut(&k) {
                    Some(base_val) => {
                        // "include" is consumed above; "dish" keys are widget instances that legitimately overlap across files.
                        if k != "include" && !k.starts_with("dish") {
                            // Tables merge recursively — warning on table overlap would be noisy and misleading.
                            if !base_val.is_table() || !v.is_table() {
                                hyprlog::internal::warn(
                                    "CONFIG",
                                    &format!(
                                        "Duplicate key '{}' being overwritten during merge.",
                                        k
                                    ),
                                );
                            }
                        }
                        merge_toml_values(base_val, v)
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
    let runtime_dir = dirs::runtime_dir().unwrap_or_else(env::temp_dir);

    runtime_dir.join("hyprbar-debug.sock")
}

pub fn get_pid_file_path() -> PathBuf {
    let runtime_dir = dirs::runtime_dir().unwrap_or_else(env::temp_dir);

    runtime_dir.join("hyprbar.pid")
}
