use anyhow::{Context, Result};
use k_lib::config::Cookbook;
use k_lib::logger;
use std::path::Path;
use std::process::Stdio;
use tokio::fs;
use tokio::process::Command;

pub async fn wash_dish(path: &Path, cookbook: &Cookbook) -> Result<()> {
    // Compile .rs to .dish
    logger::log_to_terminal(
        cookbook,
        "info",
        "WASH",
        &format!("Washing dish: {:?}", path),
    );

    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {:?}", path));
    }

    let file_stem = path
        .file_stem()
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;
    let dish_name = file_stem.to_string_lossy().to_string();

    // Create temp directory
    let temp_dir = std::env::temp_dir().join(format!("kitchnsink_build_{}", dish_name));
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).await?;
    }
    fs::create_dir_all(&temp_dir).await?;

    logger::log_to_terminal(
        cookbook,
        "debug",
        "WASH",
        &format!("Building in temp dir: {:?}", temp_dir),
    );

    // 1. Cargo init
    let status = Command::new("cargo")
        .arg("init")
        .arg("--lib")
        .arg("--name")
        .arg(&dish_name)
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
    let ks_core_path = std::env::current_dir()?.join("ks-core");

    // Add ks-core
    let status = Command::new("cargo")
        .arg("add")
        .arg("ks-core")
        .arg("--path")
        .arg(&ks_core_path)
        .current_dir(&temp_dir)
        .status()
        .await
        .context("Failed to add ks-core dependency")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to add ks-core"));
    }

    // Add ratatui
    let status = Command::new("cargo")
        .arg("add")
        .arg("ratatui@0.29.0")
        .current_dir(&temp_dir)
        .status()
        .await
        .context("Failed to add ratatui dependency")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to add ratatui"));
    }

    // Add tachyonfx
    let status = Command::new("cargo")
        .arg("add")
        .arg("tachyonfx@0.21.0")
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
                // Format: crate = "version" or crate = { ... }
                // We need to parse this string to get the crate name for `cargo add`.
                // However, `cargo add` expects `cargo add name@version` or similar.
                // The format in the file is likely `name = "version"`.
                // For simplest integration with `cargo add`, we might need to parse the key/value from the string.
                // But wait, `cargo add` is robust.
                // Actually, the `wash.mojo` simply injects them into Cargo.toml.
                // Here we are using `cargo add` commands.
                // Let's parse the `name = "version"` string.
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
        logger::log_to_terminal(
            cookbook,
            "info",
            "WASH",
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

    let json_str = metadata_json.to_string();
    // Escape quote for C string literal if needed (serde_json to_string produces valid JSON string, but we are putting it into a Rust string literal.
    // Actually, we can use a raw string literal if we are careful, or just escape quotes.
    // JSON dquote is \". Rust string needs \\".
    // Safest is to use raw string literal `b"..."` for bytes.
    // But if JSON contains `"` it's fine inside standard string if escaped.
    // Let's rely on format! debug which escapes? No.
    // We construct the source code string.

    // Proper JSON string escaping for Rust source code:
    // " -> \"
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

    let mut final_source = source_content;
    final_source.push_str(&injected_code);

    let src_path = temp_dir.join("src/lib.rs");
    fs::write(&src_path, final_source).await?;

    // 5. Build
    logger::log_to_terminal(cookbook, "info", "WASH", "Running cargo build...");
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
    let artifact_name = format!("lib{}.so", dish_name);
    let artifact_path = temp_dir.join("target/release").join(&artifact_name);

    if !artifact_path.exists() {
        return Err(anyhow::anyhow!("Artifact not found at {:?}", artifact_path));
    }

    let output_name = format!("{}.dish", dish_name);
    let target_path = std::env::current_dir()?.join(&output_name);
    fs::copy(&artifact_path, &target_path).await?;

    logger::log_to_terminal(
        cookbook,
        "info",
        "WASH",
        &format!("Dish compiled successfully: {}", target_path.display()),
    );

    Ok(())
}
