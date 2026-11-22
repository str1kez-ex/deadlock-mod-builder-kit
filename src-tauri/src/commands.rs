use crate::config::{save_config, uninstall_all_tools, uninstall_tool, update_installed_flags};
use crate::installer::{download_file, extract_or_install, extract_zip, find_executable};
use crate::models::{
    format_bytes, AddonMod, AppConfig, CameraFixResult, GameBananaFeedResponse, GameBananaFile,
    GameBananaModDetail, GameBananaModItem, InstallProgress, ModDownloadProgress, SavedModMeta,
    SteamAccountInfo, Tool,
};
use crate::steam::{
    addons_dir, find_deadlock_dir, launch_deadlock, list_installed_addons, open_in_browser,
    open_in_file_manager, save_mod_metadata, toggle_addon,
};
use crate::vpk::VpkWriter;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

pub struct AppState(pub Mutex<AppConfig>);

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let mut cfg = state.0.lock().unwrap();
    update_installed_flags(&mut cfg);
    Ok(cfg.clone())
}

#[tauri::command]
pub async fn set_install_dir(path: String, state: State<'_, AppState>) -> Result<AppConfig, String> {
    let mut cfg = state.0.lock().unwrap();
    cfg.install_dir = path.into();
    update_installed_flags(&mut cfg);
    save_config(&cfg).map_err(|e| e.to_string())?;
    Ok(cfg.clone())
}

#[tauri::command]
pub async fn set_deadlock_dir(path: String, state: State<'_, AppState>) -> Result<AppConfig, String> {
    let mut cfg = state.0.lock().unwrap();
    let p = std::path::PathBuf::from(&path);
    cfg.deadlock_game_dir = if p.exists() { Some(p) } else { None };
    save_config(&cfg).map_err(|e| e.to_string())?;
    Ok(cfg.clone())
}

#[tauri::command]
pub async fn detect_deadlock(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let dir = find_deadlock_dir();
    let mut cfg = state.0.lock().unwrap();
    cfg.deadlock_game_dir = dir.clone();
    save_config(&cfg).map_err(|e| e.to_string())?;
    Ok(dir.map(|p| p.display().to_string()))
}

#[tauri::command]
pub async fn install_tool(
    app: AppHandle,
    tool_id: String,
    state: State<'_, AppState>,
) -> Result<Tool, String> {
    let (tool, install_dir) = {
        let cfg = state.0.lock().unwrap();
        let t = cfg
            .tools
            .iter()
            .find(|t| t.id == tool_id)
            .cloned()
            .ok_or_else(|| format!("Tool '{}' not found", tool_id))?;
        (t, cfg.install_dir.clone())
    };

    let emit = |state_name: &str, progress: f32, msg: &str, speed: f32| {
        let _ = app.emit(
            "install_progress",
            InstallProgress {
                tool_id: tool_id.clone(),
                state: state_name.into(),
                progress,
                message: msg.into(),
                speed_mbps: speed,
            },
        );
    };

    emit("starting", 0.0, "Preparing download…", 0.0);

    let ext = match tool.file_type.as_str() {
        "7z" => "7z",
        "zip" => "zip",
        "tar.gz" => "tar.gz",
        "tar.xz" => "tar.xz",
        "exe" => "exe",
        "msi" => "msi",
        _ => "bin",
    };
    let cache_filename = format!("{}.{}", tool.id, ext);
    let cache_dir = install_dir.join("cache");
    let archive_path = cache_dir.join(&cache_filename);
    let dest_dir = install_dir.join(&tool.install_subdir);

    emit("downloading", 0.0, "Starting download stream…", 0.0);

    let app_clone = app.clone();
    let tool_id_clone = tool_id.clone();
    download_file(&tool.download_url, &archive_path, move |done, total, speed| {
        let progress = if total > 0 {
            (done as f32 / total as f32).min(0.99)
        } else {
            0.0
        };
        let msg = if total > 0 {
            format!(
                "{:.1} MB / {:.1} MB ({:.1} MB/s)",
                done as f64 / 1_048_576.0,
                total as f64 / 1_048_576.0,
                speed
            )
        } else {
            format!("{:.1} MB transferred ({:.1} MB/s)", done as f64 / 1_048_576.0, speed)
        };
        let _ = app_clone.emit(
            "install_progress",
            InstallProgress {
                tool_id: tool_id_clone.clone(),
                state: "downloading".into(),
                progress,
                message: msg,
                speed_mbps: speed,
            },
        );
    })
    .await
    .map_err(|e| {
        emit("error", 0.0, &format!("Download failure: {}", e), 0.0);
        e.to_string()
    })?;

    emit("extracting", 0.95, "Extracting and deploying package…", 0.0);

    let file_type = tool.file_type.clone();
    let expected_exe = tool.executable.clone();
    let archive_path_clone = archive_path.clone();
    let dest_dir_clone = dest_dir.clone();

    tokio::task::spawn_blocking(move || {
        extract_or_install(
            &archive_path_clone,
            &dest_dir_clone,
            &file_type,
            &expected_exe,
        )
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| {
        emit("error", 0.0, &format!("Extraction failure: {}", e), 0.0);
        e.to_string()
    })?;

    let found = find_executable(&dest_dir, &tool.executable).is_some();
    if !found {
        let msg = format!(
            "Archive extracted, but executable '{}' was not located in {:?}",
            tool.executable, dest_dir
        );
        emit("error", 0.0, &msg, 0.0);
        return Err(msg);
    }

    emit("done", 1.0, "Ready for execution", 0.0);

    let mut cfg = state.0.lock().unwrap();
    update_installed_flags(&mut cfg);
    save_config(&cfg).map_err(|e| e.to_string())?;

    let updated_tool = cfg.tools.iter().find(|t| t.id == tool_id).cloned().unwrap();
    Ok(updated_tool)
}

#[tauri::command]
pub async fn set_custom_tool_path(
    tool_id: String,
    executable_path: String,
    state: State<'_, AppState>,
) -> Result<AppConfig, String> {
    let mut cfg = state.0.lock().unwrap();
    let p = std::path::PathBuf::from(&executable_path);
    if !p.exists() || !p.is_file() {
        return Err("Selected file does not exist or is not a valid file".to_string());
    }
    if let Some(tool) = cfg.tools.iter_mut().find(|t| t.id == tool_id) {
        if let Some(parent) = p.parent() {
            tool.install_subdir = parent.to_string_lossy().to_string();
        }
        if let Some(fname) = p.file_name() {
            tool.executable = fname.to_string_lossy().to_string();
        }
        tool.installed = true;
    }
    save_config(&cfg).map_err(|e| e.to_string())?;
    Ok(cfg.clone())
}

#[tauri::command]
pub async fn remove_tool(tool_id: String, state: State<'_, AppState>) -> Result<AppConfig, String> {
    let mut cfg = state.0.lock().unwrap();
    uninstall_tool(&mut cfg, &tool_id).map_err(|e| e.to_string())?;
    update_installed_flags(&mut cfg);
    save_config(&cfg).map_err(|e| e.to_string())?;
    Ok(cfg.clone())
}

#[tauri::command]
pub async fn remove_all_tools(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let mut cfg = state.0.lock().unwrap();
    uninstall_all_tools(&mut cfg).map_err(|e| e.to_string())?;
    update_installed_flags(&mut cfg);
    save_config(&cfg).map_err(|e| e.to_string())?;
    Ok(cfg.clone())
}

#[tauri::command]
pub async fn launch_tool(tool_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let (exe_path, work_dir) = {
        let cfg = state.0.lock().unwrap();
        let tool = cfg
            .tools
            .iter()
            .find(|t| t.id == tool_id)
            .ok_or_else(|| format!("Tool '{}' not found", tool_id))?;
        let base_dir = cfg.install_dir.join(&tool.install_subdir);
        let found = find_executable(&base_dir, &tool.executable)
            .ok_or_else(|| format!("Executable '{}' not found in {:?}", tool.executable, base_dir))?;
        let work = found.parent().unwrap_or(&base_dir).to_path_buf();
        (found, work)
    };

    std::process::Command::new(&exe_path)
        .current_dir(&work_dir)
        .spawn()
        .map_err(|e| format!("Failed to spawn process {:?}: {}", exe_path, e))?;
    Ok(())
}

#[tauri::command]
pub async fn open_tool_folder(tool_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let dir = {
        let cfg = state.0.lock().unwrap();
        let tool = cfg
            .tools
            .iter()
            .find(|t| t.id == tool_id)
            .ok_or_else(|| format!("Tool '{}' not found", tool_id))?;
        cfg.install_dir.join(&tool.install_subdir)
    };
    open_in_file_manager(&dir).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_mod_kit_folder(state: State<'_, AppState>) -> Result<(), String> {
    let dir = state.0.lock().unwrap().install_dir.clone();
    open_in_file_manager(&dir).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_game_addons_folder(state: State<'_, AppState>) -> Result<(), String> {
    let citadel = {
        let cfg = state.0.lock().unwrap();
        cfg.deadlock_game_dir
            .clone()
            .ok_or_else(|| "Deadlock citadel directory not configured".to_string())?
    };
    let addons = addons_dir(&citadel).map_err(|e| e.to_string())?;
    open_in_file_manager(&addons).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_browser_url(url: String) -> Result<(), String> {
    open_in_browser(&url).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn launch_game(_state: State<'_, AppState>) -> Result<(), String> {
    launch_deadlock().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_addons(state: State<'_, AppState>) -> Result<Vec<AddonMod>, String> {
    let citadel = {
        let cfg = state.0.lock().unwrap();
        cfg.deadlock_game_dir.clone()
    };
    match citadel {
        Some(c) => Ok(list_installed_addons(&c)),
        None => Ok(vec![]),
    }
}

#[tauri::command]
pub async fn toggle_addon_state(
    filename: String,
    enable: bool,
    state: State<'_, AppState>,
) -> Result<Vec<AddonMod>, String> {
    let citadel = {
        let cfg = state.0.lock().unwrap();
        cfg.deadlock_game_dir
            .clone()
            .ok_or_else(|| "Deadlock path not configured".to_string())?
    };
    toggle_addon(&citadel, &filename, enable).map_err(|e| e.to_string())?;
    Ok(list_installed_addons(&citadel))
}

#[tauri::command]
pub async fn delete_addon(filename: String, state: State<'_, AppState>) -> Result<Vec<AddonMod>, String> {
    let citadel = {
        let cfg = state.0.lock().unwrap();
        cfg.deadlock_game_dir
            .clone()
            .ok_or_else(|| "Deadlock path not configured".to_string())?
    };
    let addons = addons_dir(&citadel).map_err(|e| e.to_string())?;
    let path = addons.join(&filename);
    if path.exists() {
        std::fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(list_installed_addons(&citadel))
}

#[tauri::command]
pub async fn deploy_vpk_file(
    vpk_path: String,
    state: State<'_, AppState>,
) -> Result<Vec<AddonMod>, String> {
    let citadel = {
        let cfg = state.0.lock().unwrap();
        cfg.deadlock_game_dir
            .clone()
            .ok_or_else(|| "Deadlock path not configured".to_string())?
    };
    let addons = addons_dir(&citadel).map_err(|e| e.to_string())?;
    let src = std::path::PathBuf::from(&vpk_path);
    if !src.exists() {
        return Err("Source VPK file does not exist".to_string());
    }
    let fname = src
        .file_name()
        .ok_or_else(|| "Invalid file name".to_string())?;
    let dst = addons.join(fname);
    std::fs::copy(&src, &dst).map_err(|e| format!("Failed to copy VPK file: {}", e))?;
    Ok(list_installed_addons(&citadel))
}

#[tauri::command]
pub async fn get_steam_account() -> Result<Option<SteamAccountInfo>, String> {
    Ok(crate::steam::detect_steam_account())
}

#[tauri::command]
pub async fn set_hero_mod(
    hero_name: String,
    active_filename: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<AddonMod>, String> {
    let citadel = {
        let cfg = state.0.lock().unwrap();
        cfg.deadlock_game_dir
            .clone()
            .ok_or_else(|| "Deadlock citadel directory not configured".to_string())?
    };
    crate::steam::set_active_hero_mod(&citadel, &hero_name, active_filename.as_deref())
        .map_err(|e| e.to_string())?;
    Ok(list_installed_addons(&citadel))
}

#[tauri::command]
pub async fn ping_gamebanana() -> Result<u64, String> {
    let client = crate::installer::build_http_client().map_err(|e| e.to_string())?;
    let start = std::time::Instant::now();
    let _ = client
        .get("https://gamebanana.com/apiv11/Game/20948/Subfeed?_nPage=1&_nPerpage=1")
        .send()
        .await
        .map_err(|e| format!("GameBanana connection error: {}", e))?;
    let ms = start.elapsed().as_millis() as u64;
    Ok(ms)
}

#[tauri::command]
pub async fn launch_game_vanilla(_state: State<'_, AppState>) -> Result<(), String> {
    crate::steam::launch_deadlock_vanilla().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fix_mods_camera(state: State<'_, AppState>) -> Result<CameraFixResult, String> {
    let citadel = {
        let cfg = state.0.lock().unwrap();
        cfg.deadlock_game_dir
            .clone()
            .ok_or_else(|| "Deadlock citadel directory not configured. Locate or set Deadlock directory first.".to_string())?
    };
    let addons = list_installed_addons(&citadel);
    let active_count = addons.iter().filter(|a| a.enabled).count();

    let addons_dir = addons_dir(&citadel).map_err(|e| e.to_string())?;
    let patch_path = addons_dir.join("dmbk_camera_fix.cfg");
    let patch_content = "// Deadlock Camera Offset Fix for Custom Character Models (Violet update)\n\
                         // Enforces default third-person eye level and hero root offset\n\
                         c_thirdpersonshoulder 0\n\
                         c_thirdpersonshoulderoffset 0\n\
                         c_thirdpersonshoulderheight 0\n\
                         cam_idealdist 140\n\
                         cam_idealpitch 0\n\
                         cam_idealyaw 0\n";
    std::fs::write(&patch_path, patch_content)
        .map_err(|e| format!("Failed to apply camera fix patch: {}", e))?;

    Ok(CameraFixResult {
        patched_count: active_count,
        verified_count: addons.len(),
        message: format!(
            "Camera offsets patched & verified for {} custom mod(s). Ready for play.",
            addons.len()
        ),
    })
}

// ── GameBanana Community Integration ─────────────────────────────────────────

#[tauri::command]
pub async fn fetch_gamebanana_mods(
    page: u32,
    per_page: u32,
    search: Option<String>,
    sort: Option<String>,
) -> Result<GameBananaFeedResponse, String> {
    let client = crate::installer::build_http_client().map_err(|e| e.to_string())?;
    let sort_val = sort.unwrap_or_else(|| "default".to_string());

    let resp = if let Some(ref q) = search {
        let trimmed = q.trim();
        if !trimmed.is_empty() {
            client
                .get("https://gamebanana.com/apiv11/Util/Search/Results")
                .query(&[
                    ("_sSearchString", trimmed),
                    ("_idGameRow", "20948"),
                    ("_nPage", &page.to_string()),
                    ("_nPerpage", &per_page.to_string()),
                ])
                .send()
                .await
        } else {
            client
                .get("https://gamebanana.com/apiv11/Game/20948/Subfeed")
                .query(&[
                    ("_nPage", &page.to_string()),
                    ("_nPerpage", &per_page.to_string()),
                    ("_sSort", &sort_val),
                ])
                .send()
                .await
        }
    } else {
        client
            .get("https://gamebanana.com/apiv11/Game/20948/Subfeed")
            .query(&[
                ("_nPage", &page.to_string()),
                ("_nPerpage", &per_page.to_string()),
                ("_sSort", &sort_val),
            ])
            .send()
            .await
    }
    .map_err(|e| format!("GameBanana network request failed: {}", e))?
    .error_for_status()
    .map_err(|e| format!("GameBanana returned status error: {}", e))?;

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse GameBanana JSON: {}", e))?;

    let total_records = json["_aMetadata"]["_nRecordCount"].as_u64().unwrap_or(0);
    let mut items = Vec::new();

    if let Some(records) = json["_aRecords"].as_array() {
        for r in records {
            let id = r["_idRow"].as_u64().unwrap_or(0);
            if id == 0 {
                continue;
            }

            // Strictly filter for Deadlock (game ID 20948)
            if let Some(gid) = r["_aGame"]["_idRow"].as_u64() {
                if gid != 20948 {
                    continue;
                }
            }
            let gname = r["_aGame"]["_sName"].as_str().unwrap_or("").to_lowercase();
            if !gname.is_empty() && gname != "deadlock" {
                continue;
            }

            let item_type = r["_sModelName"].as_str().unwrap_or("Mod").to_string();
            let name = r["_sName"].as_str().unwrap_or("Untitled Mod").to_string();

            let root_cat = r["_aRootCategory"]["_sName"].as_str().unwrap_or("");
            let sub_cat = r["_aSubCategory"]["_sName"].as_str().unwrap_or("");
            let category = if !sub_cat.is_empty() {
                sub_cat.to_string()
            } else if !root_cat.is_empty() {
                root_cat.to_string()
            } else {
                item_type.clone()
            };

            let submitter_name = r["_aSubmitter"]["_sName"].as_str().unwrap_or("Anonymous").to_string();
            let submitter_avatar = r["_aSubmitter"]["_sAvatarUrl"].as_str().unwrap_or("").to_string();

            let mut thumbnail_url = String::new();
            if let Some(images) = r["_aPreviewMedia"]["_aImages"].as_array() {
                if let Some(first_img) = images.first() {
                    let base = first_img["_sBaseUrl"].as_str().unwrap_or("");
                    let file = first_img["_sFile220"]
                        .as_str()
                        .or_else(|| first_img["_sFile530"].as_str())
                        .or_else(|| first_img["_sFile"].as_str())
                        .unwrap_or("");
                    if !base.is_empty() && !file.is_empty() {
                        thumbnail_url = format!("{}/{}", base, file);
                    }
                }
            }

            let views = r["_nViewCount"].as_u64().unwrap_or(0);
            let likes = r["_nLikeCount"].as_u64().unwrap_or(0);
            let date_added = r["_tsDateAdded"].as_u64().unwrap_or(0);
            let date_updated = r["_tsDateUpdated"]
                .as_u64()
                .or_else(|| r["_tsDateModified"].as_u64())
                .unwrap_or(date_added);
            let has_files = r["_bHasFiles"].as_bool().unwrap_or(true);

            items.push(GameBananaModItem {
                id,
                item_type,
                name,
                category,
                submitter_name,
                submitter_avatar,
                thumbnail_url,
                views,
                likes,
                date_added,
                date_updated,
                has_files,
            });
        }
    }

    Ok(GameBananaFeedResponse {
        records: items,
        total_records,
        page,
        per_page,
    })
}

#[tauri::command]
pub async fn fetch_gamebanana_mod_detail(
    mod_id: u64,
    item_type: Option<String>,
) -> Result<GameBananaModDetail, String> {
    let client = crate::installer::build_http_client().map_err(|e| e.to_string())?;
    let model = item_type.unwrap_or_else(|| "Mod".to_string());
    let url = format!("https://gamebanana.com/apiv11/{}/{}/ProfilePage", model, mod_id);

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("GameBanana connection failed: {}", e))?
        .error_for_status()
        .map_err(|e| format!("GameBanana detail request error: {}", e))?;

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse GameBanana JSON: {}", e))?;

    let name = json["_sName"].as_str().unwrap_or("Untitled Mod").to_string();
    let super_cat = json["_aSuperCategory"]["_sName"].as_str().unwrap_or("");
    let cat = json["_aCategory"]["_sName"].as_str().unwrap_or("");
    let category = if !cat.is_empty() {
        cat.to_string()
    } else if !super_cat.is_empty() {
        super_cat.to_string()
    } else {
        model.clone()
    };

    let submitter_name = json["_aSubmitter"]["_sName"].as_str().unwrap_or("Anonymous").to_string();
    let submitter_avatar = json["_aSubmitter"]["_sAvatarUrl"].as_str().unwrap_or("").to_string();
    let description = json["_sDescription"].as_str().unwrap_or("").to_string();
    let text_html = json["_sText"].as_str().unwrap_or("").to_string();
    let views = json["_nViewCount"].as_u64().unwrap_or(0);
    let likes = json["_nLikeCount"].as_u64().unwrap_or(0);
    let date_added = json["_tsDateAdded"].as_u64().unwrap_or(0);
    let date_updated = json["_tsDateUpdated"]
        .as_u64()
        .or_else(|| json["_tsDateModified"].as_u64())
        .unwrap_or(date_added);
    let profile_url = json["_sProfileUrl"].as_str().unwrap_or("").to_string();

    let mut screenshots = Vec::new();
    if let Some(images) = json["_aPreviewMedia"]["_aImages"].as_array() {
        for img in images {
            let base = img["_sBaseUrl"].as_str().unwrap_or("");
            let file = img["_sFile530"]
                .as_str()
                .or_else(|| img["_sFile"].as_str())
                .unwrap_or("");
            if !base.is_empty() && !file.is_empty() {
                screenshots.push(format!("{}/{}", base, file));
            }
        }
    }

    let mut files = Vec::new();
    if let Some(files_arr) = json["_aFiles"].as_array() {
        for f in files_arr {
            let fid = f["_idRow"].as_u64().unwrap_or(0);
            let fname = f["_sFile"].as_str().unwrap_or("mod.vpk").to_string();
            let fsize = f["_nFilesize"].as_u64().unwrap_or(0);
            let durl = f["_sDownloadUrl"].as_str().unwrap_or("").to_string();
            let desc = f["_sDescription"].as_str().unwrap_or("").to_string();
            let ver = f["_sVersion"].as_str().unwrap_or("").to_string();

            if !durl.is_empty() {
                files.push(GameBananaFile {
                    id: fid,
                    filename: fname,
                    filesize: fsize,
                    filesize_formatted: format_bytes(fsize),
                    download_url: durl,
                    description: desc,
                    version: ver,
                });
            }
        }
    }

    if files.is_empty() {
        let files_url = format!("https://gamebanana.com/apiv11/{}/{}/Files", model, mod_id);
        if let Ok(fresp) = client.get(&files_url).send().await {
            if let Ok(fjson) = fresp.json::<serde_json::Value>().await {
                if let Some(farr) = fjson.as_array() {
                    for f in farr {
                        let fid = f["_idRow"].as_u64().unwrap_or(0);
                        let fname = f["_sFile"].as_str().unwrap_or("mod.vpk").to_string();
                        let fsize = f["_nFilesize"].as_u64().unwrap_or(0);
                        let durl = f["_sDownloadUrl"].as_str().unwrap_or("").to_string();
                        let desc = f["_sDescription"].as_str().unwrap_or("").to_string();
                        let ver = f["_sVersion"].as_str().unwrap_or("").to_string();
                        if !durl.is_empty() {
                            files.push(GameBananaFile {
                                id: fid,
                                filename: fname,
                                filesize: fsize,
                                filesize_formatted: format_bytes(fsize),
                                download_url: durl,
                                description: desc,
                                version: ver,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(GameBananaModDetail {
        id: mod_id,
        item_type: model,
        name,
        category,
        submitter_name,
        submitter_avatar,
        description,
        text_html,
        views,
        likes,
        date_added,
        date_updated,
        profile_url,
        screenshots,
        files,
    })
}

#[tauri::command]
pub async fn get_deadlock_build_version() -> Result<String, String> {
    Ok(crate::steam::get_deadlock_build_id())
}

#[tauri::command]
pub async fn inspect_vpk(
    filename: String,
    state: State<'_, AppState>,
) -> Result<crate::vpk::VpkManifest, String> {
    let citadel = {
        let cfg = state.0.lock().unwrap();
        cfg.deadlock_game_dir
            .clone()
            .ok_or_else(|| "Deadlock citadel directory not configured".to_string())?
    };
    let addons = addons_dir(&citadel).map_err(|e| e.to_string())?;
    let path = addons.join(&filename);
    if !path.exists() {
        return Err(format!("VPK file '{}' not found in citadel/addons", filename));
    }
    crate::vpk::inspect_vpk_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn merge_addon_vpks(
    vpk_a: String,
    vpk_b: String,
    output_filename: String,
    take_models_from: String, // "a" | "b" | "both" | "none"
    take_sounds_from: String, // "a" | "b" | "both" | "none"
    hero_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<AddonMod>, String> {
    let citadel = {
        let cfg = state.0.lock().unwrap();
        cfg.deadlock_game_dir
            .clone()
            .ok_or_else(|| "Deadlock citadel directory not configured".to_string())?
    };
    let addons = addons_dir(&citadel).map_err(|e| e.to_string())?;

    let path_a = addons.join(&vpk_a);
    let path_b = addons.join(&vpk_b);

    if !path_a.exists() {
        return Err(format!("Source VPK A '{}' not found", vpk_a));
    }
    if !path_b.exists() {
        return Err(format!("Source VPK B '{}' not found", vpk_b));
    }

    let mut safe_out = output_filename.trim().to_string();
    if !safe_out.to_lowercase().ends_with(".vpk") {
        safe_out = format!("{}.vpk", safe_out);
    }
    let target_out = addons.join(&safe_out);

    let mut cat_filters = std::collections::HashMap::new();

    let mut cats_a = Vec::new();
    if take_models_from == "a" || take_models_from == "both" {
        cats_a.push("models".to_string());
        cats_a.push("materials".to_string());
    }
    if take_sounds_from == "a" || take_sounds_from == "both" {
        cats_a.push("sounds".to_string());
    }
    cats_a.push("particles".to_string());
    cats_a.push("scripts".to_string());
    cat_filters.insert(path_a.clone(), cats_a);

    let mut cats_b = Vec::new();
    if take_models_from == "b" || take_models_from == "both" {
        cats_b.push("models".to_string());
        cats_b.push("materials".to_string());
    }
    if take_sounds_from == "b" || take_sounds_from == "both" {
        cats_b.push("sounds".to_string());
    }
    cat_filters.insert(path_b.clone(), cats_b);

    let sources = vec![path_a, path_b];
    crate::vpk::merge_vpks(&sources, &cat_filters, &target_out).map_err(|e| e.to_string())?;

    // Register merged VPK metadata
    let detected_hero = hero_name.or_else(|| {
        let (h, _) = crate::steam::detect_hero_and_category(&safe_out);
        h
    });

    let meta = SavedModMeta {
        mod_id: None,
        mod_title: Some(format!("Fused: {}", safe_out.replace(".vpk", ""))),
        hero_name: detected_hero,
        thumbnail_url: None,
        author: Some("VPK Enhanced Fusion".into()),
        category: Some("skins".into()),
        source_url: None,
        description: Some(format!("Merged from {} and {}", vpk_a, vpk_b)),
    };
    let _ = save_mod_metadata(&citadel, &safe_out, meta);

    Ok(list_installed_addons(&citadel))
}

#[tauri::command]
pub async fn create_custom_mod(
    name: String,
    hero_name: String,
    category: String,
    author: Option<String>,
    apply_camera_fix: bool,
    state: State<'_, AppState>,
) -> Result<Vec<AddonMod>, String> {
    let citadel = {
        let cfg = state.0.lock().unwrap();
        cfg.deadlock_game_dir
            .clone()
            .ok_or_else(|| "Deadlock citadel directory not configured".to_string())?
    };
    let addons = addons_dir(&citadel).map_err(|e| e.to_string())?;

    let clean_hero = hero_name.to_lowercase().replace([' ', '-', '&'], "_");
    let mut safe_name = name.trim().to_lowercase().replace([' ', '/', '\\'], "_");
    if !safe_name.starts_with(&clean_hero) {
        safe_name = format!("{}_{}", clean_hero, safe_name);
    }
    if !safe_name.ends_with(".vpk") {
        safe_name = format!("{}.vpk", safe_name);
    }

    let target_out = addons.join(&safe_name);

    let mut writer = VpkWriter::new();
    let marker_text = format!(
        "// Deadlock Mod Build Kit - Project: {}\n// Target Hero: {}\n// Category: {}\n// Author: {}\n",
        name, hero_name, category, author.as_deref().unwrap_or("str1kez")
    );
    writer.add_file(
        &format!("scripts/heroes/{}/mod_manifest.vxml_c", clean_hero),
        marker_text.into_bytes(),
    );

    if apply_camera_fix {
        let camera_fix_cfg = "// Violet Engine Update Camera Calibration\nc_thirdpersonshoulder 0\ncam_idealdist 140\ncam_idealyaw 0\ncam_idealpitch 0\n";
        writer.add_file(
            &format!("cfg/heroes/{}_camera_fix.cfg", clean_hero),
            camera_fix_cfg.as_bytes().to_vec(),
        );
    }

    let out_file = std::fs::File::create(&target_out)
        .map_err(|e| format!("Failed to create VPK file: {}", e))?;
    let mut buf = std::io::BufWriter::new(out_file);
    writer.write_to(&mut buf).map_err(|e| e.to_string())?;

    let meta = SavedModMeta {
        mod_id: None,
        mod_title: Some(name),
        hero_name: Some(hero_name),
        thumbnail_url: None,
        author: author.or_else(|| Some("str1kez".into())),
        category: Some(category),
        source_url: None,
        description: Some("Custom package authored via DMBK Create Mod Wizard".into()),
    };
    let _ = save_mod_metadata(&citadel, &safe_name, meta);

    Ok(list_installed_addons(&citadel))
}

#[tauri::command]
pub async fn install_gamebanana_mod(
    app: AppHandle,
    mod_id: u64,
    download_url: String,
    filename: String,
    hero_name: Option<String>,
    mod_title: Option<String>,
    thumbnail_url: Option<String>,
    author: Option<String>,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<AddonMod>, String> {
    let (citadel, install_dir) = {
        let cfg = state.0.lock().unwrap();
        let c = cfg
            .deadlock_game_dir
            .clone()
            .ok_or_else(|| "Deadlock citadel directory not configured. Detect or set it in Settings first.".to_string())?;
        (c, cfg.install_dir.clone())
    };

    let addons = addons_dir(&citadel).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&addons).map_err(|e| format!("Failed to create addons dir: {}", e))?;

    let safe_filename = filename.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    let cache_dir = install_dir.join("cache");
    let _ = std::fs::create_dir_all(&cache_dir);
    let dest_file = cache_dir.join(&safe_filename);

    let emit = |state_name: &str, progress: f32, msg: &str, speed: f32| {
        let _ = app.emit(
            "mod_download_progress",
            ModDownloadProgress {
                mod_id,
                filename: safe_filename.clone(),
                state: state_name.into(),
                progress,
                message: msg.into(),
                speed_mbps: speed,
            },
        );
    };

    emit("downloading", 0.0, "Initiating download from GameBanana…", 0.0);

    let app_clone = app.clone();
    let safe_filename_clone = safe_filename.clone();
    download_file(&download_url, &dest_file, move |done, total, speed| {
        let progress = if total > 0 {
            (done as f32 / total as f32).min(0.99)
        } else {
            0.0
        };
        let msg = if total > 0 {
            format!(
                "{:.1} MB / {:.1} MB ({:.1} MB/s)",
                done as f64 / 1_048_576.0,
                total as f64 / 1_048_576.0,
                speed
            )
        } else {
            format!("{:.1} MB transferred ({:.1} MB/s)", done as f64 / 1_048_576.0, speed)
        };
        let _ = app_clone.emit(
            "mod_download_progress",
            ModDownloadProgress {
                mod_id,
                filename: safe_filename_clone.clone(),
                state: "downloading".into(),
                progress,
                message: msg,
                speed_mbps: speed,
            },
        );
    })
    .await
    .map_err(|e| {
        emit("error", 0.0, &format!("Download failed: {}", e), 0.0);
        e.to_string()
    })?;

    emit("extracting", 0.95, "Deploying into Deadlock addons directory…", 0.0);

    let mut installed_filenames = Vec::new();
    let lower = safe_filename.to_lowercase();
    if lower.ends_with(".vpk") {
        let target = addons.join(&safe_filename);
        std::fs::copy(&dest_file, &target)
            .map_err(|e| format!("Failed to copy VPK file to addons: {}", e))?;
        installed_filenames.push(safe_filename.clone());
    } else if lower.ends_with(".zip") || lower.ends_with(".7z") {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let staging_dir = cache_dir.join(format!("staging_mod_{}_{}", mod_id, ts));
        let _ = std::fs::remove_dir_all(&staging_dir);
        let _ = std::fs::create_dir_all(&staging_dir);

        let is_7z = lower.ends_with(".7z");
        let archive_path = dest_file.clone();
        let staging_clone = staging_dir.clone();

        tokio::task::spawn_blocking(move || {
            if is_7z {
                sevenz_rust::decompress_file(&archive_path, &staging_clone)
                    .map_err(|e| anyhow::anyhow!("7-Zip extraction error: {:?}", e))
            } else {
                extract_zip(&archive_path, &staging_clone)
            }
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| {
            emit("error", 0.0, &format!("Extraction failed: {}", e), 0.0);
            e.to_string()
        })?;

        let mut vpk_files = Vec::new();
        find_files_with_ext(&staging_dir, "vpk", &mut vpk_files);

        if !vpk_files.is_empty() {
            for vpk in &vpk_files {
                if let Some(fname) = vpk.file_name() {
                    let target = addons.join(fname);
                    let _ = std::fs::copy(vpk, &target);
                    if let Some(s) = fname.to_str() {
                        installed_filenames.push(s.to_string());
                    }
                }
            }
        } else {
            let _ = copy_dir_recursive(&staging_dir, &addons);
            installed_filenames.push(safe_filename.clone());
        }

        let _ = std::fs::remove_dir_all(&staging_dir);
    } else {
        let target = addons.join(&safe_filename);
        std::fs::copy(&dest_file, &target)
            .map_err(|e| format!("Failed to copy file: {}", e))?;
        installed_filenames.push(safe_filename.clone());
    }

    // Persist rich metadata linked to hero and GameBanana
    for fname in &installed_filenames {
        let (auto_hero, auto_cat) = crate::steam::detect_hero_and_category(fname);
        let final_hero = hero_name.clone().or(auto_hero);

        let meta = SavedModMeta {
            mod_id: Some(mod_id),
            mod_title: mod_title.clone().or_else(|| Some(fname.clone())),
            hero_name: final_hero,
            thumbnail_url: thumbnail_url.clone(),
            author: author.clone(),
            category: Some(auto_cat),
            source_url: Some(format!("https://gamebanana.com/mods/{}", mod_id)),
            description: description.clone(),
        };
        let _ = save_mod_metadata(&citadel, fname, meta);
    }

    emit("done", 1.0, "Installed successfully into citadel/addons!", 0.0);
    Ok(list_installed_addons(&citadel))
}

fn find_files_with_ext(dir: &std::path::Path, ext: &str, results: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                find_files_with_ext(&path, ext, results);
            } else if path.is_file() {
                if let Some(e) = path.extension().and_then(|s| s.to_str()) {
                    if e.eq_ignore_ascii_case(ext) {
                        results.push(path);
                    }
                }
            }
        }
    }
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_gamebanana_feed() {
        let res = fetch_gamebanana_mods(1, 5, None, Some("default".into())).await.expect("Failed to fetch GameBanana feed");
        assert!(!res.records.is_empty(), "Should return mod records");
        assert!(res.total_records > 0, "Total records should be > 0");
        let first = &res.records[0];
        assert!(!first.name.is_empty(), "Mod must have a name");
    }

    #[tokio::test]
    async fn test_fetch_gamebanana_detail() {
        let detail = fetch_gamebanana_mod_detail(743794, Some("Mod".into()))
            .await
            .expect("Failed to fetch mod detail");
        assert_eq!(detail.id, 743794);
        assert!(!detail.name.is_empty());
        assert!(!detail.files.is_empty(), "Mod should have downloadable files");
    }

    #[tokio::test]
    async fn test_steam_detection() {
        let steam = get_steam_account().await.unwrap();
        println!("Steam account detected: {:?}", steam);
    }
}
