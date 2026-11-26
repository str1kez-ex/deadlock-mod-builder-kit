// logger.rs — Session-based file logging + game console monitoring
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct SessionLogger {
    log_dir: PathBuf,
    log_file: Arc<Mutex<Option<std::fs::File>>>,
    session_path: Arc<Mutex<Option<PathBuf>>>,
}

impl SessionLogger {
    pub fn new(install_dir: &std::path::Path) -> Self {
        let log_dir = install_dir.join("logs");
        let _ = std::fs::create_dir_all(&log_dir);
        Self {
            log_dir,
            log_file: Arc::new(Mutex::new(None)),
            session_path: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start_session(&self) {
        let now = chrono::Local::now();
        let filename = format!("session_{}.log", now.format("%Y-%m-%d_%H-%M-%S"));
        let path = self.log_dir.join(&filename);
        match std::fs::File::create(&path) {
            Ok(file) => {
                let header = format!(
                    "[{}] ===== Deadlock Mod Build Kit session started =====\n",
                    now.format("%Y-%m-%d %H:%M:%S")
                );
                use std::io::Write;
                let mut f = &file;
                let _ = f.write_all(header.as_bytes());
                *self.log_file.lock().unwrap() = Some(file);
                *self.session_path.lock().unwrap() = Some(path);
            }
            Err(e) => {
                eprintln!("[DMBK Logger] Failed to create session log: {}", e);
            }
        }
    }

    pub fn write(&self, level: &str, message: &str) {
        let now = chrono::Local::now();
        let line = format!("[{}] [{}] {}\n", now.format("%H:%M:%S"), level.to_uppercase(), message);
        if let Ok(mut guard) = self.log_file.lock() {
            if let Some(ref mut file) = *guard {
                use std::io::Write;
                let _ = file.write_all(line.as_bytes());
            }
        }
    }

    pub fn get_log_dir(&self) -> PathBuf {
        self.log_dir.clone()
    }

    pub fn get_session_path(&self) -> Option<PathBuf> {
        self.session_path.lock().unwrap().clone()
    }
}

/// Read last N lines from game console log as formatted strings
pub fn read_game_console_log(citadel: &std::path::Path, max_lines: usize) -> Vec<String> {
    read_game_console_entries(citadel, max_lines)
        .into_iter()
        .map(|e| format!("[GAME] {}", e.message))
        .collect()
}

/// Read last N lines from game console log as structured `LogEntry` items
pub fn read_game_console_entries(citadel: &std::path::Path, max_lines: usize) -> Vec<crate::models::LogEntry> {
    // Deadlock may write to game/citadel/console.log or game/citadel/log.txt
    let candidates = [
        citadel.join("console.log"),
        citadel.join("log.txt"),
        citadel.join("citadel").join("console.log"),
    ];

    let now_time = chrono::Local::now().format("%H:%M:%S").to_string();

    for path in &candidates {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                let entries: Vec<crate::models::LogEntry> = content
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .rev()
                    .take(max_lines)
                    .map(|l| crate::models::LogEntry {
                        time: now_time.clone(),
                        level: if l.to_lowercase().contains("error") || l.to_lowercase().contains("fail") {
                            "error".to_string()
                        } else if l.to_lowercase().contains("warn") {
                            "warn".to_string()
                        } else {
                            "game".to_string()
                        },
                        message: l.to_string(),
                        source: "game".to_string(),
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect();
                return entries;
            }
        }
    }
    vec![]
}
