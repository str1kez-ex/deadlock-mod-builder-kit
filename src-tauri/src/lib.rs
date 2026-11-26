mod commands;
mod config;
mod installer;
pub mod logger;
mod models;
mod steam;
pub mod vpk;

use commands::AppState;
use config::load_config;
use std::sync::Mutex;
use tauri::Manager;

fn default_install_dir() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("DeadlockModKit")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let install_dir = default_install_dir();
    let _ = std::fs::create_dir_all(&install_dir);

    // Start session logger
    let session_logger = logger::SessionLogger::new(&install_dir);
    session_logger.start_session();
    session_logger.write("info", "Deadlock Mod Build Kit v1.5 started");

    let config = load_config(&install_dir);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .manage(AppState(Mutex::new(config)))
        .manage(commands::LoggerState(session_logger))
        .on_page_load(|webview, payload| {
            let state = webview.app_handle().state::<commands::LoggerState>();
            state.0.write(
                "info",
                &format!("WebView page_load {:?}: {}", payload.event(), payload.url()),
            );
        })
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
            commands::restore_vanilla_addons,
            commands::fix_mods_camera,
            commands::get_deadlock_build_version,
            commands::inspect_vpk,
            commands::merge_addon_vpks,
            commands::create_custom_mod,
            // v1.5 new commands
            commands::get_game_logs,
            commands::open_logs_folder,
            commands::set_ui_language,
            commands::set_ui_scale,
            commands::write_session_log,
            commands::save_logs_dialog,
            commands::get_disk_free_space,
        ])
        .run(tauri::generate_context!())
        .expect("Error running Deadlock Mod Kit")
}
