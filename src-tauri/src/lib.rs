mod commands;
mod config;
mod installer;
mod models;
mod steam;
pub mod vpk;

use commands::AppState;
use config::load_config;
use std::sync::Mutex;

fn default_install_dir() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("DeadlockModKit")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let install_dir = default_install_dir();
    let _ = std::fs::create_dir_all(&install_dir);

    let config = load_config(&install_dir);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .manage(AppState(Mutex::new(config)))
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::set_install_dir,
            commands::set_deadlock_dir,
            commands::detect_deadlock,
            commands::set_custom_tool_path,
            commands::install_tool,
            commands::remove_tool,
            commands::remove_all_tools,
            commands::launch_tool,
            commands::open_tool_folder,
            commands::open_mod_kit_folder,
            commands::open_game_addons_folder,
            commands::open_browser_url,
            commands::launch_game,
            commands::get_addons,
            commands::toggle_addon_state,
            commands::delete_addon,
            commands::deploy_vpk_file,
            commands::fetch_gamebanana_mods,
            commands::fetch_gamebanana_mod_detail,
            commands::install_gamebanana_mod,
            commands::get_steam_account,
            commands::set_hero_mod,
            commands::ping_gamebanana,
            commands::launch_game_vanilla,
            commands::fix_mods_camera,
            commands::get_deadlock_build_version,
            commands::inspect_vpk,
            commands::merge_addon_vpks,
            commands::create_custom_mod,
        ])
        .run(tauri::generate_context!())
        .expect("Error running Deadlock Mod Kit");
}
