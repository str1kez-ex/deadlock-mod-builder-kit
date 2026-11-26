use crate::installer::find_executable;
use crate::models::{AppConfig, Tool};
use anyhow::{Context, Result};
use std::path::Path;

const CONFIG_FILENAME: &str = "config.json";

pub fn default_tools() -> Vec<Tool> {
    vec![
        Tool {
            id: "csdk13".into(),
            name: "CSDK13 (Deadlock SDK)".into(),
            description: "Source 2 Community SDK for Deadlock with compilation toolchain, Hammer++, and schema decompilers.".into(),
            category: "SDK & Compilers".into(),
            download_url: "https://drive.usercontent.google.com/download?id=1bAM-S06aLAzX6mUOVdkhCoznj7SvwMZC&export=download&authuser=0&confirm=t".into(),
            file_type: "7z".into(),
            install_subdir: "tools/csdk13".into(),
            executable: "CSDK13.exe".into(),
            installed: false,
            size_estimate: "~1.1 GB".into(),
            version_tag: "v13".into(),
        },
        Tool {
            id: "source2viewer".into(),
            name: "Source 2 Viewer (VRF)".into(),
            description: "Valve Resource Format inspector. Inspect, export, and decompile vmdl_c models, vtex_c textures, and vsnd_c audio.".into(),
            category: "Asset Inspector".into(),
            download_url: "https://github.com/ValveResourceFormat/ValveResourceFormat/releases/download/21.1/Source2Viewer.exe".into(),
            file_type: "exe".into(),
            install_subdir: "tools/source2viewer".into(),
            executable: "Source2Viewer.exe".into(),
            installed: false,
            size_estimate: "~42 MB".into(),
            version_tag: "v21.1".into(),
        },
    ]
}

pub fn update_installed_flags(config: &mut AppConfig) {
    for tool in config.tools.iter_mut() {
        let tool_dir = config.install_dir.join(&tool.install_subdir);
        tool.installed = find_executable(&tool_dir, &tool.executable).is_some();
    }
}

pub fn load_config(install_dir: &Path) -> AppConfig {
    let config_path = install_dir.join(CONFIG_FILENAME);
    let mut config = if config_path.exists() {
        let raw = std::fs::read_to_string(&config_path).unwrap_or_default();
        serde_json::from_str(&raw).unwrap_or_else(|_| make_default_config(install_dir))
    } else {
        make_default_config(install_dir)
    };

    // Filter out removed tools
    config.tools.retain(|t| t.id != "blender" && t.id != "deadlock_mod_builder");

    // Replace or sync tools to ensure latest descriptions & URLs are active
    let defs = default_tools();
    for def in defs {
        if let Some(existing) = config.tools.iter_mut().find(|t| t.id == def.id) {
            existing.name = def.name;
            existing.description = def.description;
            existing.category = def.category;
            existing.download_url = def.download_url;
            existing.file_type = def.file_type;
            existing.install_subdir = def.install_subdir;
            existing.executable = def.executable;
            existing.size_estimate = def.size_estimate;
            existing.version_tag = def.version_tag;
        } else {
            config.tools.push(def);
        }
    }

    update_installed_flags(&mut config);
    let _ = save_config(&config);
    config
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let config_path = config.install_dir.join(CONFIG_FILENAME);
    let json = serde_json::to_string_pretty(config).context("Failed to serialize config")?;
    std::fs::write(&config_path, json).context("Failed to write config.json")?;
    Ok(())
}

pub fn uninstall_tool(config: &mut AppConfig, tool_id: &str) -> Result<()> {
    let tool = config
        .tools
        .iter_mut()
        .find(|t| t.id == tool_id)
        .with_context(|| format!("Tool '{}' not found in config", tool_id))?;

    let tool_dir = config.install_dir.join(&tool.install_subdir);
    if tool_dir.exists() {
        std::fs::remove_dir_all(&tool_dir)
            .with_context(|| format!("Failed to delete directory: {}", tool_dir.display()))?;
    }
    tool.installed = false;
    Ok(())
}

pub fn uninstall_all_tools(config: &mut AppConfig) -> Result<()> {
    let tools_dir = config.install_dir.join("tools");
    if tools_dir.exists() {
        std::fs::remove_dir_all(&tools_dir)
            .with_context(|| format!("Failed to delete tools directory: {}", tools_dir.display()))?;
    }
    let cache_dir = config.install_dir.join("cache");
    if cache_dir.exists() {
        let _ = std::fs::remove_dir_all(&cache_dir);
    }
    for tool in config.tools.iter_mut() {
        tool.installed = false;
    }
    Ok(())
}

fn make_default_config(install_dir: &Path) -> AppConfig {
    let deadlock_dir = crate::steam::find_deadlock_dir();
    let mut config = AppConfig {
        install_dir: install_dir.to_path_buf(),
        deadlock_game_dir: deadlock_dir,
        tools: default_tools(),
        ui_language: Some("ru".into()),
        ui_scale: Some(100),
    };
    update_installed_flags(&mut config);
    config
}
