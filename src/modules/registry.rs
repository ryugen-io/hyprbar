use anyhow::{Context, Result};
use postcard::{from_bytes, to_allocvec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Registry {
    pub plugins: HashMap<String, PluginEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PluginMetadata {
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginEntry {
    pub path: PathBuf,
    pub enabled: bool,
    pub installed_at: SystemTime,
    #[serde(default)]
    pub metadata: PluginMetadata,
}

impl Registry {
    /// Loads the registry from the standard location.
    /// Creates an empty registry if the file doesn't exist.
    pub fn load() -> Result<Self> {
        let path = get_registry_path();

        if !path.exists() {
            return Ok(Self::default());
        }

        let bytes = fs::read(&path).context("Failed to read registry file")?;

        // If file is empty, return default
        if bytes.is_empty() {
            return Ok(Self::default());
        }

        let registry = from_bytes(&bytes).context("Failed to deserialize registry")?;
        Ok(registry)
    }

    /// Saves the registry to disk.
    pub fn save(&self) -> Result<()> {
        let path = get_registry_path();

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create registry directory")?;
        }

        let bytes = to_allocvec(self).context("Failed to serialize registry")?;
        fs::write(&path, bytes).context("Failed to write registry file")?;
        Ok(())
    }

    /// Register a plugin (installing or updating).
    pub fn install(&mut self, name: String, path: PathBuf, metadata: PluginMetadata) -> Result<()> {
        let entry = PluginEntry {
            path,
            enabled: true, // Auto-enable on install
            installed_at: SystemTime::now(),
            metadata,
        };
        self.plugins.insert(name, entry);
        self.save()
    }

    /// Enable a plugin by name.
    pub fn enable(&mut self, name: &str) -> Result<()> {
        if let Some(entry) = self.plugins.get_mut(name) {
            entry.enabled = true;
            self.save()?;
            Ok(())
        } else {
            anyhow::bail!("Plugin '{}' not found", name);
        }
    }

    /// Disable a plugin by name.
    pub fn disable(&mut self, name: &str) -> Result<()> {
        if let Some(entry) = self.plugins.get_mut(name) {
            entry.enabled = false;
            self.save()?;
            Ok(())
        } else {
            anyhow::bail!("Plugin '{}' not found", name);
        }
    }
}

fn get_registry_path() -> PathBuf {
    dirs::data_local_dir()
        .map(|p| p.join("hyprbar/dishes/registry.bin"))
        .unwrap_or_else(|| PathBuf::from("registry.bin")) // Fallback (shouldn't happen on Linux)
}
