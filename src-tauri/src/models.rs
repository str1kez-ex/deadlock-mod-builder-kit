use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub download_url: String,
    pub file_type: String, // "7z" | "zip" | "exe" | "msi"
    pub install_subdir: String,
    pub executable: String,
    pub installed: bool,
    pub size_estimate: String,
    pub version_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgress {
    pub tool_id: String,
    pub state: String, // "starting" | "downloading" | "extracting" | "done" | "error"
    pub progress: f32, // 0.0 - 1.0
    pub message: String,
    pub speed_mbps: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddonMod {
    pub filename: String,
    pub size_bytes: u64,
    pub modified_time: String,
    pub enabled: bool,
    pub detected_hero: Option<String>,
    pub category: String, // "skins" | "sounds" | "maps" | "ui" | "other"
    pub mod_title: Option<String>,
    pub thumbnail_url: Option<String>,
    pub gamebanana_id: Option<u64>,
    pub author: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SavedModMeta {
    pub mod_id: Option<u64>,
    pub mod_title: Option<String>,
    pub hero_name: Option<String>,
    pub thumbnail_url: Option<String>,
    pub author: Option<String>,
    pub category: Option<String>,
    pub source_url: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamAccountInfo {
    pub steam_id_64: String,
    pub account_id: u32,
    pub account_name: String,
    pub persona_name: String,
    pub statlocker_url: String,
    pub is_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub install_dir: PathBuf,
    pub deadlock_game_dir: Option<PathBuf>,
    pub tools: Vec<Tool>,
}

// ── GameBanana Community Models ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBananaFile {
    pub id: u64,
    pub filename: String,
    pub filesize: u64,
    pub filesize_formatted: String,
    pub download_url: String,
    pub description: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBananaModItem {
    pub id: u64,
    pub item_type: String, // "Mod" | "Sound"
    pub name: String,
    pub category: String,
    pub submitter_name: String,
    pub submitter_avatar: String,
    pub thumbnail_url: String,
    pub views: u64,
    pub likes: u64,
    pub date_added: u64,
    pub date_updated: u64,
    pub has_files: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBananaFeedResponse {
    pub records: Vec<GameBananaModItem>,
    pub total_records: u64,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBananaModDetail {
    pub id: u64,
    pub item_type: String,
    pub name: String,
    pub category: String,
    pub submitter_name: String,
    pub submitter_avatar: String,
    pub description: String,
    pub text_html: String,
    pub views: u64,
    pub likes: u64,
    pub date_added: u64,
    pub date_updated: u64,
    pub profile_url: String,
    pub screenshots: Vec<String>,
    pub files: Vec<GameBananaFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraFixResult {
    pub patched_count: usize,
    pub verified_count: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDownloadProgress {
    pub mod_id: u64,
    pub filename: String,
    pub state: String, // "downloading" | "extracting" | "done" | "error"
    pub progress: f32, // 0.0 - 1.0
    pub message: String,
    pub speed_mbps: f32,
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
