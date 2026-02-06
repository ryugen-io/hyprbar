use anyhow::{Context, Result};
use hyprink::config::Config;
use std::path::Path;
use std::process::Stdio;
use tokio::fs;
use tokio::process::Command;

pub async fn compile_widget(path: &Path, _config_ink: &Config) -> Result<()> {
    // Compile .rs to .so widget plugin
    hyprlog::internal::info("BUILD", &format!("Compiling widget: {:?}", path));

    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {:?}", path));
    }

    let file_stem = path
        .file_stem()
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;
    let widget_name = file_stem.to_string_lossy().to_string();

    // Create temp directory
    let temp_dir = std::env::temp_dir().join(format!("hyprbar_build_{}", widget_name));
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).await?;
    }
    fs::create_dir_all(&temp_dir).await?;

    hyprlog::internal::debug("BUILD", &format!("Building in temp dir: {:?}", temp_dir));

    // 1. Cargo init
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

    // 2. Configure [lib] crate-type
    let cargo_toml_path = temp_dir.join("Cargo.toml");
    let mut cargo_toml = fs::read_to_string(&cargo_toml_path).await?;
    if !cargo_toml.contains("[lib]") {
        cargo_toml.push_str("\n[lib]\ncrate-type = [\"cdylib\"]\n");
        fs::write(&cargo_toml_path, cargo_toml).await?;
    }

    // 3. Add dependencies via cargo add
    // Add hyprbar as dependency for Widget trait
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

    // Add ratatui
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

    // Add tachyonfx with sendable feature for Send + Sync
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

    // 4. Parse Metadata & Append Code
    let source_content = fs::read_to_string(path).await?;
    let mut metadata_json = serde_json::json!({
        "name": "Unknown",
        "description": "",
        "author": "",
        "version": "0.0.1"
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
            } else if let Some(obj) = metadata_json.as_object_mut()
                && obj.contains_key(&key)
            {
                obj[&key] = serde_json::Value::String(value.to_string());
            }
        }
    }

    // 4b. Install Custom Dependencies
    for (dep_name, dep_version) in dependencies {
        hyprlog::internal::info(
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

    // Only inject metadata if not already present
    let mut final_source = source_content.clone();
    if !source_content.contains("_plugin_metadata") {
        let json_str = metadata_json.to_string();
        let escaped_json = json_str.replace('"', "\\\"");

        let injected_code = format!(
            r#"
#[unsafe(no_mangle)]
pub extern "C" fn _plugin_metadata() -> *const std::ffi::c_char {{
    static META: &[u8] = b"{}\0";
    META.as_ptr() as *const _
}}
"#,
            escaped_json
        );
        final_source.push_str(&injected_code);
    }

    let src_path = temp_dir.join("src/lib.rs");
    fs::write(&src_path, final_source).await?;

    // 5. Build
    hyprlog::internal::info("BUILD", "Running cargo build...");
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

    // 6. Copy artifact
    let artifact_name = format!("lib{}.so", widget_name);
    let artifact_path = temp_dir.join("target/release").join(&artifact_name);

    if !artifact_path.exists() {
        return Err(anyhow::anyhow!("Artifact not found at {:?}", artifact_path));
    }

    // Output to XDG data directory
    let widgets_dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine XDG data directory"))?
        .join("hyprbar/widgets");

    fs::create_dir_all(&widgets_dir).await?;

    let output_name = format!("{}.so", widget_name);
    let target_path = widgets_dir.join(&output_name);
    fs::copy(&artifact_path, &target_path).await?;

    hyprlog::internal::info(
        "BUILD",
        &format!("Widget compiled successfully: {}", target_path.display()),
    );

    Ok(())
}
