use crate::modules::logging::{log_debug, log_info};
use anyhow::{Context, Result};
use hyprink::config::Config;
use std::path::Path;
use std::process::Stdio;
use tokio::fs;
use tokio::process::Command;

pub async fn compile_widget(path: &Path, _config_ink: &Config) -> Result<()> {
    log_info("BUILD", &format!("Compiling widget: {:?}", path));

    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {:?}", path));
    }

    let file_stem = path
        .file_stem()
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;
    let widget_name = file_stem.to_string_lossy().to_string();

    // Isolated temp dir prevents cross-contamination between concurrent builds
    // and avoids stale artifacts from previous failed builds.
    let temp_dir = std::env::temp_dir().join(format!("hyprbar_build_{}", widget_name));
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).await?;
    }
    fs::create_dir_all(&temp_dir).await?;

    log_debug("BUILD", &format!("Building in temp dir: {:?}", temp_dir));

    let status = Command::new("cargo")
        .arg("init")
        .arg("--lib")
        .arg("--name")
        .arg(&widget_name)
        .current_dir(&temp_dir)
        .status()
        .await
        .context("Failed to run cargo init")?;

    if !status.success() {
        return Err(anyhow::anyhow!("cargo init failed"));
    }

    // cdylib produces a .so that libloading can dlopen at runtime.
    let cargo_toml_path = temp_dir.join("Cargo.toml");
    let mut cargo_toml = fs::read_to_string(&cargo_toml_path).await?;
    if !cargo_toml.contains("[lib]") {
        cargo_toml.push_str("\n[lib]\ncrate-type = [\"cdylib\"]\n");
        fs::write(&cargo_toml_path, cargo_toml).await?;
    }

    // Plugins need hyprbar for Widget trait + prelude, ratatui for rendering,
    // tachyonfx for visual effects. These are always needed regardless of widget.
    let hyprbar_path = std::env::current_dir()?;

    let status = Command::new("cargo")
        .arg("add")
        .arg("hyprbar")
        .arg("--path")
        .arg(&hyprbar_path)
        .current_dir(&temp_dir)
        .status()
        .await
        .context("Failed to add hyprbar dependency")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to add hyprbar"));
    }

    let status = Command::new("cargo")
        .arg("add")
        .arg("ratatui@0.30.0")
        .current_dir(&temp_dir)
        .status()
        .await
        .context("Failed to add ratatui dependency")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to add ratatui"));
    }

    // sendable feature is required because Widget: Send + Sync, and tachyonfx
    // effects must be transferable across the plugin .so boundary.
    let status = Command::new("cargo")
        .arg("add")
        .arg("tachyonfx@0.23.0")
        .arg("--features")
        .arg("sendable,std-duration")
        .current_dir(&temp_dir)
        .status()
        .await
        .context("Failed to add tachyonfx dependency")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to add tachyonfx"));
    }

    // Metadata and dependencies are declared in `//!` doc comments so that
    // the source file is self-describing — no separate manifest required.
    let source_content = fs::read_to_string(path).await?;
    let mut metadata_json = serde_json::json!({
        "name": "Unknown",
        "description": "",
        "author": "",
        "version": "0.0.1",
        "has_popup": false
    });

    let mut dependencies = Vec::new();

    for line in source_content.lines() {
        if let Some(comment) = line.trim().strip_prefix("//!")
            && let Some((key, value)) = comment.split_once(':')
        {
            let key = key.trim().to_lowercase();
            let value = value.trim();

            if key == "dependency" {
                if let Some((dep_name, dep_version)) = value.split_once('=') {
                    let dep_name = dep_name.trim();
                    let dep_version = dep_version.trim().trim_matches('"').trim_matches('\'');
                    dependencies.push((dep_name.to_string(), dep_version.to_string()));
                }
            } else if key == "has-popup" || key == "has_popup" {
                // has_popup gates popup vtable calls across .so boundaries
                // to avoid unstable ABI segfaults.
                if let Some(obj) = metadata_json.as_object_mut() {
                    obj["has_popup"] = serde_json::Value::Bool(value.eq_ignore_ascii_case("true"));
                }
            } else if let Some(obj) = metadata_json.as_object_mut()
                && obj.contains_key(&key)
            {
                obj[&key] = serde_json::Value::String(value.to_string());
            }
        }
    }

    for (dep_name, dep_version) in dependencies {
        log_info(
            "BUILD",
            &format!("Adding dependency: {} = {}", dep_name, dep_version),
        );

        let status = Command::new("cargo")
            .arg("add")
            .arg(format!("{}@{}", dep_name, dep_version))
            .current_dir(&temp_dir)
            .status()
            .await
            .context(format!("Failed to add dependency {}", dep_name))?;

        if !status.success() {
            return Err(anyhow::anyhow!("Failed to add dependency: {}", dep_name));
        }
    }

    // Source is written as-is — metadata lives in the sidecar JSON instead of
    // being injected as a C ABI export, eliminating unsafe from plugin code.
    let src_path = temp_dir.join("src/lib.rs");
    fs::write(&src_path, &source_content).await?;

    log_info("BUILD", "Running cargo build...");
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(&temp_dir)
        .stdout(Stdio::null())
        .status()
        .await
        .context("Failed to run cargo build")?;

    if !status.success() {
        return Err(anyhow::anyhow!("cargo build failed"));
    }

    let artifact_name = format!("lib{}.so", widget_name);
    let artifact_path = temp_dir.join("target/release").join(&artifact_name);

    if !artifact_path.exists() {
        return Err(anyhow::anyhow!("Artifact not found at {:?}", artifact_path));
    }

    let widgets_dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine XDG data directory"))?
        .join("hyprbar/widgets");

    fs::create_dir_all(&widgets_dir).await?;

    let so_name = format!("{}.so", widget_name);
    let json_name = format!("{}.json", widget_name);
    let target_so = widgets_dir.join(&so_name);
    let target_json = widgets_dir.join(&json_name);

    fs::copy(&artifact_path, &target_so).await?;

    // Sidecar JSON is co-located with the .so so that plugin_loader can read
    // metadata without dlopen — eliminating all unsafe from the metadata path.
    let json_str = serde_json::to_string_pretty(&metadata_json)?;
    fs::write(&target_json, json_str).await?;

    log_info(
        "BUILD",
        &format!("Widget compiled successfully: {}", target_so.display()),
    );

    Ok(())
}
