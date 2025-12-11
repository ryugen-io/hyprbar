use anyhow::Result;
use k_lib::config::Cookbook;
use k_lib::logger;
use std::path::Path;
use tokio::fs;

pub async fn load_dish(path: &Path, cookbook: &Cookbook) -> Result<()> {
    // Install .dish
    logger::log_to_terminal(
        cookbook,
        "info",
        "LOAD",
        &format!("Loading dish: {:?}", path),
    );
    let data_dir = dirs::data_local_dir().unwrap().join("kitchnsink/dishes");
    fs::create_dir_all(&data_dir).await?;

    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
    let target = data_dir.join(file_name);

    fs::copy(path, &target).await?;
    logger::log_to_terminal(
        cookbook,
        "info",
        "LOAD",
        &format!("Dish installed to: {:?}", target),
    );
    Ok(())
}
