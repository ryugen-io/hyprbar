use ks_core::config::SinkConfig;
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn load_sink_config() -> SinkConfig {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let config_path = PathBuf::from(home)
        .join(".config")
        .join("kitchnsink")
        .join("sink.toml"); // User requested "sink.toml"

    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(cfg) => return cfg,
                Err(e) => eprintln!("Failed to parse config: {}", e),
            },
            Err(e) => eprintln!("Failed to read config: {}", e),
        }
    }

    // Fallback
    SinkConfig::default()
}

pub fn get_socket_path() -> PathBuf {
    let runtime_dir = env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::temp_dir());
    runtime_dir.join("kitchnsink-debug.sock")
}
