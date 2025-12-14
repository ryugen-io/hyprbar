use anyhow::{Context, Result};
use k_lib::config::Cookbook;
use k_lib::logger;
use ks_lib::config::SinkConfig;
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

    // Check for "include" key and collect paths
    let mut include_patterns = Vec::new();
    if let Some(includes) = root_value.get("include").and_then(|v| v.as_array()) {
        for include_val in includes {
            if let Some(include_str) = include_val.as_str() {
                include_patterns.push(include_str.to_string());
            }
        }
    }

    // Expand globs and merge
    for pattern_str in include_patterns {
        let pattern_path = base_dir.join(&pattern_str);

        // If it looks like a glob (or just a path), try globbing it
        // We use pattern_path.to_string_lossy() because glob::glob expects a string pattern
        let pattern = pattern_path.to_string_lossy();

        match glob::glob(&pattern) {
            Ok(paths) => {
                let mut found_any = false;
                for entry in paths {
                    match entry {
                        Ok(path) => {
                            found_any = true;
                            // Recursively load the included file
                            // Note: base_dir for the included file should probably be its parent,
                            // but sticking to main config_dir as base is safer for relative includes inside includes?
                            // Standard practice: relative paths are relative to the file they are in.
                            // But here we passed base_dir. Let's assume includes are relative to config root.
                            let included_value = load_recursive_config(&path, base_dir, cookbook)?;
                            merge_toml_values(&mut root_value, included_value, cookbook);
                        }
                        Err(e) => {
                            logger::log_to_terminal(
                                cookbook,
                                "warn",
                                "CONFIG",
                                &format!("Glob error for pattern '{}': {}", pattern_str, e),
                            );
                        }
                    }
                }

                if !found_any {
                    logger::log_to_terminal(
                        cookbook,
                        "warn",
                        "CONFIG",
                        &format!("No files matched include pattern: '{}'", pattern_str),
                    );
                }
            }
            Err(e) => {
                logger::log_to_terminal(
                    cookbook,
                    "warn",
                    "CONFIG",
                    &format!("Invalid glob pattern '{}': {}", pattern_str, e),
                );
            }
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
    let runtime_dir = dirs::runtime_dir().unwrap_or_else(env::temp_dir);

    runtime_dir.join("kitchnsink-debug.sock")
}

pub fn get_pid_file_path() -> PathBuf {
    let runtime_dir = dirs::runtime_dir().unwrap_or_else(env::temp_dir);

    runtime_dir.join("kitchnsink.pid")
}
