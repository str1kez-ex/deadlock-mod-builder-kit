use crate::models::{AddonMod, SavedModMeta, SteamAccountInfo};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Deadlock's Steam App ID
const DEADLOCK_APP_ID: u32 = 1422450;

pub const DEADLOCK_HEROES: &[&str] = &[
    "Abrams", "Bebop", "Calico", "Dynamo", "Grey Talon", "Haze", "Holliday", "Infernus",
    "Ivy", "Kelvin", "Lady Geist", "Lash", "McGinnis", "Mirage", "Mo & Krill", "Paradox",
    "Pocket", "Rat King", "Seven", "Shiv", "Vindicta", "Viscous", "Warden", "Wraith", "Wrecker", "Yamato",
];

const METADATA_FILE: &str = "dmbk_mods_metadata.json";

pub fn load_mods_metadata(citadel: &Path) -> HashMap<String, SavedModMeta> {
    if let Ok(addons) = addons_dir(citadel) {
        let meta_file = addons.join(METADATA_FILE);
        if meta_file.exists() {
            if let Ok(raw) = std::fs::read_to_string(&meta_file) {
                if let Ok(map) = serde_json::from_str(&raw) {
                    return map;
                }
            }
        }
    }
    HashMap::new()
}

pub fn save_mod_metadata(citadel: &Path, filename: &str, meta: SavedModMeta) -> Result<()> {
    let addons = addons_dir(citadel)?;
    let meta_file = addons.join(METADATA_FILE);
    let mut map = load_mods_metadata(citadel);
    let base_name = filename.trim_end_matches(".disabled").to_string();
    map.insert(base_name, meta);
    let json = serde_json::to_string_pretty(&map)?;
    std::fs::write(&meta_file, json)?;
    Ok(())
}

pub fn get_deadlock_build_id() -> String {
    let paths = [
        PathBuf::from(r"C:\Program Files (x86)\Steam\steamapps\appmanifest_1422450.acf"),
        PathBuf::from(r"C:\Program Files\Steam\steamapps\appmanifest_1422450.acf"),
        PathBuf::from(r"D:\SteamLibrary\steamapps\appmanifest_1422450.acf"),
        PathBuf::from(r"E:\SteamLibrary\steamapps\appmanifest_1422450.acf"),
    ];

    for path in &paths {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("\"buildid\"") {
                        let parts: Vec<&str> = trimmed.split('"').collect();
                        if parts.len() >= 4 {
                            return parts[3].to_string();
                        }
                    }
                }
            }
        }
    }

    if let Ok(steam) = steamlocate::SteamDir::locate() {
        if let Ok(Some((_app, lib))) = steam.find_app(DEADLOCK_APP_ID) {
            let manifest = lib.path().join("steamapps").join(format!("appmanifest_{}.acf", DEADLOCK_APP_ID));
            if manifest.exists() {
                if let Ok(content) = std::fs::read_to_string(&manifest) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("\"buildid\"") {
                            let parts: Vec<&str> = trimmed.split('"').collect();
                            if parts.len() >= 4 {
                                return parts[3].to_string();
                            }
                        }
                    }
                }
            }
        }
    }

    "26770241".to_string()
}

pub fn detect_hero_and_category(filename: &str) -> (Option<String>, String) {
    let lower = filename.to_lowercase();
    let clean = lower.replace(['_', '-', '.'], " ");

    let mut detected_hero = None;
    for hero in DEADLOCK_HEROES {
        let h_lower = hero.to_lowercase();
        if clean.contains(&h_lower) {
            detected_hero = Some(hero.to_string());
            break;
        }
        if *hero == "Lady Geist" && (clean.contains("geist") || clean.contains("ladygeist")) {
            detected_hero = Some(hero.to_string());
            break;
        }
        if *hero == "Grey Talon" && (clean.contains("talon") || clean.contains("greytalon")) {
            detected_hero = Some(hero.to_string());
            break;
        }
        if *hero == "Mo & Krill" && (clean.contains("mokrill") || clean.contains("mo krill") || clean.contains("mo and krill")) {
            detected_hero = Some(hero.to_string());
            break;
        }
    }

    let category = if clean.contains("sound")
        || clean.contains(" vo")
        || clean.contains("voice")
        || clean.contains("audio")
        || clean.contains("music")
        || clean.contains("sfx")
    {
        "sounds".to_string()
    } else if clean.contains("map") || clean.contains("level") || clean.contains("arena") {
        "maps".to_string()
    } else if clean.contains("hud")
        || clean.contains("ui")
        || clean.contains("crosshair")
        || clean.contains("font")
        || clean.contains("icon")
    {
        "ui".to_string()
    } else if detected_hero.is_some()
        || clean.contains("skin")
        || clean.contains("model")
        || clean.contains("costume")
    {
        "skins".to_string()
    } else {
        "other".to_string()
    };

    (detected_hero, category)
}

/// Attempts to auto-detect the Deadlock installation directory using steamlocate.
/// Returns the path to `deadlock/game/citadel` if found.
pub fn find_deadlock_dir() -> Option<PathBuf> {
    let steam = steamlocate::SteamDir::locate().ok()?;
    let (app, library) = steam.find_app(DEADLOCK_APP_ID).ok()??;
    let install_path = library.resolve_app_dir(&app);
    let citadel = install_path.join("game").join("citadel");
    if citadel.exists() {
        Some(citadel)
    } else {
        None
    }
}

/// Returns the addons directory where VPK mods are placed.
pub fn addons_dir(citadel: &Path) -> Result<PathBuf> {
    let addons = citadel.join("addons");
    std::fs::create_dir_all(&addons)
        .with_context(|| format!("Failed to create addons dir: {}", addons.display()))?;
    Ok(addons)
}

/// Lists all .vpk and .vpk.disabled files in addons folder
pub fn list_installed_addons(citadel: &Path) -> Vec<AddonMod> {
    let mut mods = Vec::new();
    let Ok(addons) = addons_dir(citadel) else {
        return mods;
    };

    let metadata_map = load_mods_metadata(citadel);

    let Ok(entries) = std::fs::read_dir(addons) else {
        return mods;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        let is_vpk = fname.ends_with(".vpk");
        let is_disabled = fname.ends_with(".vpk.disabled");

        if is_vpk || is_disabled {
            let metadata = entry.metadata().ok();
            let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
            let time_str = metadata
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    let dur = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
                    format!("{}s ago", dur.as_secs())
                })
                .unwrap_or_else(|| "Unknown".to_string());

            let (mut detected_hero, mut category) = detect_hero_and_category(&fname);
            let mut mod_title = None;
            let mut thumbnail_url = None;
            let mut gamebanana_id = None;
            let mut author = None;
            let mut description = None;

            let base_name = fname.trim_end_matches(".disabled");
            if let Some(meta) = metadata_map.get(base_name) {
                if meta.hero_name.is_some() {
                    detected_hero = meta.hero_name.clone();
                }
                if let Some(cat) = &meta.category {
                    category = cat.clone();
                }
                mod_title = meta.mod_title.clone();
                thumbnail_url = meta.thumbnail_url.clone();
                gamebanana_id = meta.mod_id;
                author = meta.author.clone();
                description = meta.description.clone();
            }

            mods.push(AddonMod {
                filename: fname,
                size_bytes: size,
                modified_time: time_str,
                enabled: is_vpk,
                detected_hero,
                category,
                mod_title,
                thumbnail_url,
                gamebanana_id,
                author,
                description,
            });
        }
    }

    mods.sort_by(|a, b| a.filename.cmp(&b.filename));
    mods
}

/// Toggle enable/disable of an addon by renaming extension
pub fn toggle_addon(citadel: &Path, filename: &str, enable: bool) -> Result<()> {
    let addons = addons_dir(citadel)?;
    let current_path = addons.join(filename);
    if !current_path.exists() {
        anyhow::bail!("Addon file not found: {}", filename);
    }

    let target_name = if enable {
        if filename.ends_with(".disabled") {
            filename.trim_end_matches(".disabled").to_string()
        } else {
            filename.to_string()
        }
    } else {
        if !filename.ends_with(".disabled") {
            format!("{}.disabled", filename)
        } else {
            filename.to_string()
        }
    };

    let target_path = addons.join(target_name);
    std::fs::rename(current_path, target_path)?;
    Ok(())
}

/// Sets active mod for a hero by enabling the specified mod and disabling all other mods for that hero.
/// If `active_filename` is None, all mods for that hero are disabled.
pub fn set_active_hero_mod(citadel: &Path, hero_name: &str, active_filename: Option<&str>) -> Result<()> {
    let addons = list_installed_addons(citadel);
    for addon in addons {
        if let Some(ref hero) = addon.detected_hero {
            if hero.eq_ignore_ascii_case(hero_name) {
                let should_enable = match active_filename {
                    Some(target) => addon.filename.eq_ignore_ascii_case(target) || addon.filename.trim_end_matches(".disabled").eq_ignore_ascii_case(target.trim_end_matches(".disabled")),
                    None => false,
                };
                let _ = toggle_addon(citadel, &addon.filename, should_enable);
            }
        }
    }
    Ok(())
}

/// Launches Deadlock via Steam URI.
pub fn launch_deadlock() -> Result<()> {
    let uri = format!("steam://rungameid/{}", DEADLOCK_APP_ID);
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &uri])
            .spawn()
            .context("Failed to launch Deadlock via Steam URI")?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&uri)
            .spawn()
            .context("Failed to launch Deadlock via xdg-open")?;
    }
    Ok(())
}

/// Launches Deadlock via Steam with addons disabled (-disableaddons)
pub fn launch_deadlock_vanilla() -> Result<()> {
    let uri = format!("steam://rungameid/{}//-disableaddons", DEADLOCK_APP_ID);
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &uri])
            .spawn()
            .context("Failed to launch Deadlock via Steam URI")?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&uri)
            .spawn()
            .context("Failed to launch Deadlock via xdg-open")?;
    }
    Ok(())
}

/// Opens a path in the system file manager (Explorer on Windows, xdg-open on Linux)
pub fn open_in_file_manager(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path.as_os_str())
            .spawn()
            .context("Failed to open path in Windows Explorer")?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path.as_os_str())
            .spawn()
            .context("Failed to open path in file manager")?;
    }
    Ok(())
}

/// Opens a web URL in the user's default browser
pub fn open_in_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", url])
            .spawn()
            .context("Failed to open URL in browser")?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .context("Failed to open URL in browser")?;
    }
    Ok(())
}

/// Detects local active Steam account from loginusers.vdf
pub fn detect_steam_account() -> Option<SteamAccountInfo> {
    let steam = steamlocate::SteamDir::locate().ok()?;
    let config_vdf = steam.path().join("config").join("loginusers.vdf");
    if !config_vdf.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&config_vdf).ok()?;
    parse_loginusers_vdf(&content)
}

fn parse_loginusers_vdf(content: &str) -> Option<SteamAccountInfo> {
    let mut best_steam_id = String::new();
    let mut best_account_name = String::new();
    let mut best_persona_name = String::new();
    let mut best_timestamp: u64 = 0;
    let mut best_is_autologin = false;

    let mut current_steam_id = String::new();
    let mut current_account = String::new();
    let mut current_persona = String::new();
    let mut current_timestamp: u64 = 0;
    let mut current_autologin = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('"') && trimmed.ends_with('"') && !trimmed.contains('\t') && !trimmed.contains("  ") {
            let id = trimmed.trim_matches('"');
            if id.starts_with("7656") && id.len() >= 16 {
                if !current_steam_id.is_empty() {
                    if current_autologin || current_timestamp > best_timestamp {
                        best_steam_id = current_steam_id.clone();
                        best_account_name = current_account.clone();
                        best_persona_name = current_persona.clone();
                        best_timestamp = current_timestamp;
                        best_is_autologin = current_autologin;
                    }
                }
                current_steam_id = id.to_string();
                current_account.clear();
                current_persona.clear();
                current_timestamp = 0;
                current_autologin = false;
            }
        } else if trimmed.contains('"') {
            let parts: Vec<&str> = trimmed.split('"').filter(|s| !s.trim().is_empty()).collect();
            if parts.len() >= 2 {
                let key = parts[0].trim();
                let val = parts[1].trim();
                if key.eq_ignore_ascii_case("AccountName") {
                    current_account = val.to_string();
                } else if key.eq_ignore_ascii_case("PersonaName") {
                    current_persona = val.to_string();
                } else if key.eq_ignore_ascii_case("AutoLogin") {
                    current_autologin = val == "1";
                } else if key.eq_ignore_ascii_case("timestamp") {
                    current_timestamp = val.parse().unwrap_or(0);
                }
            }
        }
    }

    if !current_steam_id.is_empty() {
        if best_steam_id.is_empty() || current_autologin || (!best_is_autologin && current_timestamp > best_timestamp) {
            best_steam_id = current_steam_id;
            best_account_name = current_account;
            best_persona_name = current_persona;
        }
    }

    if best_steam_id.is_empty() {
        return None;
    }

    let steam64: u64 = best_steam_id.parse().unwrap_or(0);
    let account_id = if steam64 > 76561197960265728 {
        (steam64 - 76561197960265728) as u32
    } else {
        steam64 as u32
    };

    Some(SteamAccountInfo {
        steam_id_64: best_steam_id,
        account_id,
        account_name: best_account_name,
        persona_name: if best_persona_name.is_empty() {
            "Deadlock Player".to_string()
        } else {
            best_persona_name
        },
        statlocker_url: format!("https://statlocker.net/players/{}", account_id),
        is_detected: true,
    })
}
