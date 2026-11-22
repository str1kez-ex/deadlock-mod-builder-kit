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
            sevenz_rust::decompress_file(archive_path, dest_dir)
                .map_err(|e| anyhow::anyhow!("7-Zip extraction failed: {:?}", e))?;
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
            let target_exe = dest_dir.join(expected_exe);
            std::fs::copy(archive_path, &target_exe)
                .with_context(|| format!("Failed to copy executable to {}", target_exe.display()))?;
        }
        "msi" => {
            let target = dest_dir.join(archive_path.file_name().unwrap_or_default());
            std::fs::copy(archive_path, &target)
                .with_context(|| format!("Failed to copy MSI to {}", target.display()))?;
        }
        _ => {
            let name = archive_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.ends_with(".7z") {
                sevenz_rust::decompress_file(archive_path, dest_dir)
                    .map_err(|e| anyhow::anyhow!("7-Zip extraction failed: {:?}", e))?;
            } else if name.ends_with(".zip") {
                extract_zip(archive_path, dest_dir)?;
            } else {
                anyhow::bail!("Unsupported archive format: {}", name);
            }
        }
    }

    Ok(())
}

pub fn extract_zip(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    let file = std::fs::File::open(archive_path)
        .with_context(|| format!("Cannot open zip: {}", archive_path.display()))?;
    let mut archive = zip::ZipArchive::new(file).context("Failed to read zip archive")?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let raw_name = file.mangled_name();
        let outpath = dest_dir.join(raw_name);

        if file.name().ends_with('/') || file.name().ends_with('\\') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

fn extract_tar_gz(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    let file = std::fs::File::open(archive_path)?;
    let gz = flate2::read::GzDecoder::new(file);
    let mut tar = tar::Archive::new(gz);
    tar.unpack(dest_dir).context("Failed to extract tar.gz")?;
    Ok(())
}

fn extract_tar_xz(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    let file = std::fs::File::open(archive_path)?;
    let xz = xz2::read::XzDecoder::new(file);
    let mut tar = tar::Archive::new(xz);
    tar.unpack(dest_dir).context("Failed to extract tar.xz")?;
    Ok(())
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
