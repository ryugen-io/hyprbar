use crate::config::BarConfig;
use crate::modules::logging::log_info;
use anyhow::{Context, Result};
use hypr_conf::{
    collect_source_graph, expand_source_expression_to_path, has_glob_chars, parse_source_value,
    resolve_source_targets, source_expression_matches_path,
};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::debug;

pub fn handle_autostart(config: &BarConfig) -> Result<()> {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let home_dir = PathBuf::from(home);
    let hypr_dir = home_dir.join(".config").join("hypr");
    let hyprland_conf = hypr_dir.join("hyprland.conf");
    let default_include = hypr_dir.join("hyprbar.conf");

    if !hypr_dir.exists() {
        fs::create_dir_all(&hypr_dir).context("Failed to create Hypr config directory")?;
    }

    debug!("Resolving Hyprland source graph from {:?}", hyprland_conf);
    let source_graph = collect_config_graph(&hyprland_conf, &home_dir);
    let explicit_sources = collect_explicit_hyprbar_sources(&source_graph, &home_dir);
    let glob_sources = collect_globbed_hyprbar_files(&source_graph, &home_dir);
    let enabled = !explicit_sources.is_empty() || !glob_sources.is_empty();

    if enabled {
        debug!("Hyprbar autostart is enabled. Disabling...");

        let mut managed_targets = explicit_sources;
        managed_targets.extend(glob_sources);
        managed_targets.insert(default_include.clone());

        for file in &source_graph {
            if let Ok(content) = fs::read_to_string(file) {
                let cleaned = remove_explicit_hyprbar_source_lines(
                    &content,
                    file.parent().unwrap_or_else(|| Path::new("/")),
                    &home_dir,
                );
                if cleaned != content {
                    fs::write(file, cleaned)
                        .with_context(|| format!("Failed to update {:?}", file))?;
                }
            }
        }

        for target in managed_targets {
            if target.exists() {
                fs::remove_file(&target)
                    .with_context(|| format!("Failed to remove include file {:?}", target))?;
            }
        }

        let msg = config.label("bar_autostart_disabled", "Hyprland autostart disabled");
        log_info("AUTOSTART", msg);
    } else {
        debug!("Hyprbar autostart is disabled. Enabling...");

        let include_target = find_preferred_glob_target(&source_graph, &home_dir)
            .unwrap_or_else(|| default_include.clone());
        if let Some(parent) = include_target.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create include dir {:?}", parent))?;
        }

        fs::write(&include_target, hyprbar_autostart_snippet())
            .with_context(|| format!("Failed to write include file {:?}", include_target))?;

        let covered_by_existing_glob =
            is_path_covered_by_glob(&source_graph, &home_dir, &include_target);
        if !covered_by_existing_glob {
            let main_conf = fs::read_to_string(&hyprland_conf).unwrap_or_default();
            let source_line = format!("source = {}", include_target.display());
            let updated_main_conf =
                append_source_line_if_missing(&main_conf, &source_line, &home_dir);
            fs::write(&hyprland_conf, updated_main_conf)
                .context("Failed to update hyprland.conf")?;
            log_info(
                "AUTOSTART",
                &format!("Added source line to: {}", hyprland_conf.display()),
            );
        }

        let msg = config.label("bar_autostart_enabled", "Hyprland autostart enabled");
        log_info("AUTOSTART", msg);
        log_info(
            "AUTOSTART",
            &format!("Managed include: {}", include_target.display()),
        );
    }

    Ok(())
}

fn hyprbar_autostart_snippet() -> String {
    [
        "# Managed by hyprbar --autostart",
        "# Toggle off by running hyprbar --autostart again",
        "exec-once = hyprbar --start",
        "",
    ]
    .join("\n")
}

fn collect_config_graph(root: &Path, home_dir: &Path) -> Vec<PathBuf> {
    collect_source_graph(root, home_dir)
}

fn append_source_line_if_missing(content: &str, source_line: &str, home_dir: &Path) -> String {
    let base = home_dir.join(".config").join("hypr");

    if content.lines().any(|line| {
        parse_source_value(line)
            .map(|value| source_targets_hyprbar(value, &base, home_dir))
            .unwrap_or(false)
    }) {
        return content.to_string();
    }

    let mut out = content.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(source_line);
    out.push('\n');
    out
}

fn remove_explicit_hyprbar_source_lines(content: &str, base_dir: &Path, home_dir: &Path) -> String {
    let mut out = String::new();
    for line in content.lines() {
        let drop_line = parse_source_value(line)
            .map(|value| source_targets_hyprbar(value, base_dir, home_dir))
            .unwrap_or(false);

        if !drop_line {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

fn collect_explicit_hyprbar_sources(source_graph: &[PathBuf], home_dir: &Path) -> HashSet<PathBuf> {
    let mut out = HashSet::new();

    for file in source_graph {
        let content = match fs::read_to_string(file) {
            Ok(content) => content,
            Err(_) => continue,
        };

        let base_dir = file.parent().unwrap_or_else(|| Path::new("/"));
        for line in content.lines() {
            if let Some(source_value) = parse_source_value(line)
                && source_targets_hyprbar(source_value, base_dir, home_dir)
            {
                for target in resolve_source_targets(source_value, base_dir, home_dir) {
                    if target.file_name().and_then(|n| n.to_str()) == Some("hyprbar.conf") {
                        out.insert(target);
                    }
                }
            }
        }
    }

    out
}

fn collect_globbed_hyprbar_files(source_graph: &[PathBuf], home_dir: &Path) -> HashSet<PathBuf> {
    let mut out = HashSet::new();

    for file in source_graph {
        let content = match fs::read_to_string(file) {
            Ok(content) => content,
            Err(_) => continue,
        };

        let base_dir = file.parent().unwrap_or_else(|| Path::new("/"));
        for line in content.lines() {
            let Some(source_value) = parse_source_value(line) else {
                continue;
            };

            let expanded = expand_source_expression_to_path(source_value, base_dir, home_dir);
            let expanded = expanded.to_string_lossy();
            if !has_glob_chars(&expanded) {
                continue;
            }

            for entry in resolve_source_targets(source_value, base_dir, home_dir) {
                if entry.file_name().and_then(|n| n.to_str()) == Some("hyprbar.conf") {
                    out.insert(entry);
                }
            }
        }
    }

    out
}

fn find_preferred_glob_target(source_graph: &[PathBuf], home_dir: &Path) -> Option<PathBuf> {
    for file in source_graph {
        let content = fs::read_to_string(file).ok()?;
        let base_dir = file.parent().unwrap_or_else(|| Path::new("/"));

        for line in content.lines() {
            let Some(source_value) = parse_source_value(line) else {
                continue;
            };

            let expanded = expand_source_expression_to_path(source_value, base_dir, home_dir);
            let expanded = expanded.to_string_lossy();
            if !has_glob_chars(&expanded) {
                continue;
            }

            let Some(parent) = Path::new(expanded.as_ref()).parent() else {
                continue;
            };

            let parent_str = parent.display().to_string();
            if has_glob_chars(&parent_str) {
                continue;
            }

            let candidate = parent.join("hyprbar.conf");
            if source_expression_matches_path(source_value, base_dir, home_dir, &candidate) {
                return Some(candidate);
            }
        }
    }

    None
}

fn is_path_covered_by_glob(source_graph: &[PathBuf], home_dir: &Path, target: &Path) -> bool {
    for file in source_graph {
        let content = match fs::read_to_string(file) {
            Ok(content) => content,
            Err(_) => continue,
        };

        let base_dir = file.parent().unwrap_or_else(|| Path::new("/"));
        for line in content.lines() {
            let Some(source_value) = parse_source_value(line) else {
                continue;
            };

            let expanded = expand_source_expression_to_path(source_value, base_dir, home_dir);
            let expanded = expanded.to_string_lossy();
            if !has_glob_chars(&expanded) {
                continue;
            }

            if source_expression_matches_path(source_value, base_dir, home_dir, target) {
                return true;
            }
        }
    }

    false
}

fn source_targets_hyprbar(value: &str, base_dir: &Path, home_dir: &Path) -> bool {
    if has_glob_chars(value) {
        return false;
    }

    resolve_source_targets(value, base_dir, home_dir)
        .iter()
        .any(|path| path.file_name().and_then(|n| n.to_str()) == Some("hyprbar.conf"))
}
