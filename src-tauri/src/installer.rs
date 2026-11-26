use anyhow::{Context, Result};
use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub fn build_http_client() -> Result<reqwest::Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"),
    );
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("*/*"),
    );

    reqwest::Client::builder()
        .default_headers(headers)
        .redirect(reqwest::redirect::Policy::limited(20))
        .build()
        .context("Failed to build HTTP client")
}

/// Downloads a file from `url` into `dest_path`.
/// Calls `on_progress(downloaded_bytes, total_bytes, speed_mbps)`.
pub async fn download_file(
    url: &str,
    dest_path: &Path,
    on_progress: impl Fn(u64, u64, f32),
) -> Result<()> {
    let client = build_http_client()?;
    let resp = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("Failed to connect to download URL: {}", url))?
        .error_for_status()
        .with_context(|| format!("Server returned error status for URL: {}", url))?;

    let total = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();

    if let Some(parent) = dest_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .context("Failed to create download cache folder")?;
    }

    let mut file = File::create(dest_path)
        .await
        .context("Failed to create destination file on disk")?;

    let start_time = Instant::now();
    let mut last_sample_time = Instant::now();
    let mut last_sample_bytes: u64 = 0;
    let mut speed_mbps: f32 = 0.0;

    // Report immediate initial progress so UI updates without delay
    on_progress(0, total, 0.0);

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Error receiving data chunk from download stream")?;
        file.write_all(&chunk)
            .await
            .context("Failed to write data chunk to disk")?;
        downloaded += chunk.len() as u64;

        let now = Instant::now();
        let elapsed_since_sample = now.duration_since(last_sample_time).as_secs_f32();
        if elapsed_since_sample >= 0.25 {
            let bytes_diff = downloaded.saturating_sub(last_sample_bytes);
            speed_mbps = (bytes_diff as f32 / 1_048_576.0) / elapsed_since_sample;
            last_sample_time = now;
            last_sample_bytes = downloaded;
            on_progress(downloaded, total, speed_mbps);
        }
    }

    let total_elapsed = start_time.elapsed().as_secs_f32();
    let final_speed = if total_elapsed > 0.0 {
        (downloaded as f32 / 1_048_576.0) / total_elapsed
    } else {
        speed_mbps
    };
    on_progress(downloaded, total, final_speed);

    Ok(())
}

/// Extracts or places archive/file into `dest_dir`
pub fn extract_or_install(archive_path: &Path, dest_dir: &Path, file_type: &str, expected_exe: &str) -> Result<()> {
    std::fs::create_dir_all(dest_dir)
        .with_context(|| format!("Failed to create destination directory: {}", dest_dir.display()))?;

    match file_type {
        "7z" => {
            extract_7z(archive_path, dest_dir)?;
        }
        "zip" => {
            extract_zip(archive_path, dest_dir)?;
        }
        "tar.gz" | "tgz" => {
            extract_tar_gz(archive_path, dest_dir)?;
        }
        "tar.xz" => {
            extract_tar_xz(archive_path, dest_dir)?;
        }
        "exe" => {
            let safe_exe = crate::steam::sanitize_incoming_vpk_filename(expected_exe);
            let target_exe = dest_dir.join(safe_exe);
            std::fs::copy(archive_path, &target_exe)
                .with_context(|| format!("Failed to copy executable to {}", target_exe.display()))?;
        }
        "msi" => {
            let fname = archive_path
                .file_name()
                .and_then(|n| n.to_str())
                .map(crate::steam::sanitize_incoming_vpk_filename)
                .unwrap_or_else(|| "installer.msi".to_string());
            let target = dest_dir.join(fname);
            std::fs::copy(archive_path, &target)
                .with_context(|| format!("Failed to copy MSI to {}", target.display()))?;
        }
        _ => {
            let name = archive_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.ends_with(".7z") {
                extract_7z(archive_path, dest_dir)?;
            } else if name.ends_with(".zip") {
                extract_zip(archive_path, dest_dir)?;
            } else {
                anyhow::bail!("Unsupported archive format: {}", name);
            }
        }
    }

    Ok(())
}

/// Validates a relative path inside an archive before any directory or file is created.
/// Rejects `..`, root dirs, drive prefixes, backslashes-as-traversal, and empty paths (§5.3).
pub fn validate_archive_entry_path(raw_name: &str) -> Result<PathBuf> {
    let normalized = raw_name.replace('\\', "/");
    let mut clean = PathBuf::new();
    for component in Path::new(&normalized).components() {
        match component {
            std::path::Component::Normal(part) => {
                let s = part.to_string_lossy();
                if s == ".." || s.contains('\0') {
                    anyhow::bail!("Zip-slip detected: invalid entry component {:?}", raw_name);
                }
                clean.push(part);
            }
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir
            | std::path::Component::RootDir
            | std::path::Component::Prefix(_) => {
                anyhow::bail!("Zip-slip detected: entry {:?} escapes destination directory", raw_name);
            }
        }
    }
    if clean.as_os_str().is_empty() {
        anyhow::bail!("Zip-slip detected: empty entry path");
    }
    Ok(clean)
}

/// Safe 7z extraction with strict Zip-Slip prevention (§5.3).
pub fn extract_7z(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dest_dir)
        .with_context(|| format!("Failed to create destination directory: {}", dest_dir.display()))?;
    let canonical_dest = dest_dir
        .canonicalize()
        .with_context(|| format!("Cannot canonicalize destination: {}", dest_dir.display()))?;

    sevenz_rust::decompress_file_with_extract_fn(archive_path, &canonical_dest, |entry, reader, _| {
        // Validate entry path BEFORE creating any directory or file
        let rel = validate_archive_entry_path(entry.name())
            .map_err(|e| sevenz_rust::Error::other(e.to_string()))?;
        let target = canonical_dest.join(&rel);

        if entry.is_directory() {
            std::fs::create_dir_all(&target).map_err(sevenz_rust::Error::io)?;
            let canon = target.canonicalize().map_err(sevenz_rust::Error::io)?;
            if !canon.starts_with(&canonical_dest) {
                return Err(sevenz_rust::Error::other("Zip-slip detected in 7z directory"));
            }
            return Ok(true);
        }

        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(sevenz_rust::Error::io)?;
            let canon_parent = parent.canonicalize().map_err(sevenz_rust::Error::io)?;
            if !canon_parent.starts_with(&canonical_dest) {
                return Err(sevenz_rust::Error::other("Zip-slip detected in 7z entry parent"));
            }
            let safe_target = canon_parent.join(
                target
                    .file_name()
                    .ok_or_else(|| sevenz_rust::Error::other("Missing filename in 7z entry"))?,
            );
            sevenz_rust::default_entry_extract_fn(entry, reader, &safe_target)
        } else {
            Err(sevenz_rust::Error::other("Invalid 7z entry path without parent"))
        }
    })
    .map_err(|e| anyhow::anyhow!("7-Zip extraction failed: {:?}", e))?;

    Ok(())
}

pub fn extract_zip(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    let file = std::fs::File::open(archive_path)
        .with_context(|| format!("Cannot open zip: {}", archive_path.display()))?;
    let mut archive = zip::ZipArchive::new(file).context("Failed to read zip archive")?;

    std::fs::create_dir_all(dest_dir)
        .with_context(|| format!("Failed to create destination directory: {}", dest_dir.display()))?;
    // FIX §5.3 — Zip-Slip prevention: resolve dest_dir to its canonical form once
    let canonical_dest = dest_dir
        .canonicalize()
        .with_context(|| format!("Cannot canonicalize destination: {}", dest_dir.display()))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        // Skip symlinks to prevent symlink-based traversal
        if file.is_symlink() {
            continue;
        }

        // Validate entry path BEFORE creating any directory on disk
        let raw_str = file.name().to_string();
        if raw_str.trim().is_empty() || raw_str == "/" || raw_str == "\\" {
            continue;
        }
        let trimmed = raw_str.trim_end_matches(['/', '\\']);
        if trimmed.is_empty() {
            continue;
        }
        let rel_path = validate_archive_entry_path(trimmed)?;
        let outpath = canonical_dest.join(&rel_path);

        if file.is_dir() || raw_str.ends_with('/') || raw_str.ends_with('\\') {
            std::fs::create_dir_all(&outpath)?;
            let canon = outpath.canonicalize()?;
            if !canon.starts_with(&canonical_dest) {
                anyhow::bail!("Zip-slip detected: directory {:?} escapes destination", raw_str);
            }
        } else {
            let parent = outpath
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Zip entry has no parent: {:?}", raw_str))?;
            std::fs::create_dir_all(parent)?;
            let canonical_parent = parent
                .canonicalize()
                .with_context(|| format!("Cannot canonicalize zip entry parent: {}", parent.display()))?;
            if !canonical_parent.starts_with(&canonical_dest) {
                anyhow::bail!("Zip-slip detected: entry {:?} escapes destination directory", raw_str);
            }

            let fname = outpath
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("Zip entry has no file name: {:?}", raw_str))?;
            let safe_outpath = canonical_parent.join(fname);
            let mut outfile = std::fs::File::create(&safe_outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

fn unpack_tar_safe<R: std::io::Read>(mut tar: tar::Archive<R>, dest_dir: &Path) -> Result<()> {
    let canonical_dest = dest_dir
        .canonicalize()
        .with_context(|| format!("Cannot canonicalize destination: {}", dest_dir.display()))?;
    tar.set_preserve_permissions(false);
    for entry_res in tar.entries()? {
        let mut entry = entry_res?;
        let etype = entry.header().entry_type();
        if etype.is_symlink() || etype.is_hard_link() {
            continue;
        }
        let raw_path = entry.path()?.to_string_lossy().to_string();
        let trimmed = raw_path.trim_end_matches(['/', '\\']);
        if trimmed.is_empty() || trimmed == "." {
            continue;
        }
        let rel = validate_archive_entry_path(trimmed)?;
        let outpath = canonical_dest.join(&rel);
        if etype.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
                let canon_parent = parent.canonicalize()?;
                if !canon_parent.starts_with(&canonical_dest) {
                    anyhow::bail!("Tar Zip-slip detected: {}", raw_path);
                }
            }
            entry.unpack_in(&canonical_dest)?;
        }
    }
    Ok(())
}

fn extract_tar_gz(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    let file = std::fs::File::open(archive_path)?;
    let gz = flate2::read::GzDecoder::new(file);
    let tar = tar::Archive::new(gz);
    unpack_tar_safe(tar, dest_dir).context("Failed to extract tar.gz")
}

fn extract_tar_xz(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    let file = std::fs::File::open(archive_path)?;
    let xz = xz2::read::XzDecoder::new(file);
    let tar = tar::Archive::new(xz);
    unpack_tar_safe(tar, dest_dir).context("Failed to extract tar.xz")
}

/// Recursively looks for an executable with `expected_name` in `base_dir` up to 3 levels deep
pub fn find_executable(base_dir: &Path, expected_name: &str) -> Option<PathBuf> {
    if !base_dir.exists() {
        return None;
    }

    let direct = base_dir.join(expected_name);
    if direct.is_file() {
        return Some(direct);
    }

    search_dir(base_dir, expected_name, 0, 3)
}

fn search_dir(dir: &Path, target: &str, depth: usize, max_depth: usize) -> Option<PathBuf> {
    if depth > max_depth {
        return None;
    }

    let entries = std::fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(fname) = path.file_name().and_then(|n| n.to_str()) {
                if fname.eq_ignore_ascii_case(target) {
                    return Some(path);
                }
            }
        } else if path.is_dir() {
            if let Some(found) = search_dir(&path, target, depth + 1, max_depth) {
                return Some(found);
            }
        }
    }
    None
}
