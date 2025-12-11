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

    // 4. Copy source file
    fs::copy(path, temp_dir.join("src/lib.rs")).await?;

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
