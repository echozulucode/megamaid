use crate::AppState;
use megamaid::models::FileEntry;
use megamaid::scanner::{ParallelScanner, ScannerConfig};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc as StdArc;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub entries: Vec<FileEntry>,
    pub total_files: usize,
    pub total_size: u64,
    pub errors: Vec<String>,
}

/// Scan a directory and return file entries
#[tauri::command]
pub async fn scan_directory(
    app: AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
    path: String,
    config: ScannerConfig,
) -> Result<ScanResult, String> {
    let scan_path = PathBuf::from(&path);

    if !scan_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    // Emit start event
    let _ = app.emit("scan:started", &path);

    let scanner = ParallelScanner::new(config);
    let progress = StdArc::new(AtomicUsize::new(0));

    let entries = scanner
        .scan_with_progress(&scan_path, |count| {
            progress.store(count, Ordering::Relaxed);
            let _ = app.emit(
                "scan:progress",
                &serde_json::json!({
                    "path": path,
                    "files_scanned": count,
                }),
            );
        })
        .map_err(|e| {
            let _ = app.emit("scan:error", &e.to_string());
            e.to_string()
        })?;

    let total_files = entries.len();
    let total_size: u64 = entries.iter().map(|e| e.size).sum();

    let result = ScanResult {
        entries,
        total_files,
        total_size,
        errors: vec![],
    };

    {
        let mut guard = state.lock().map_err(|_| "Failed to lock app state")?;
        guard.current_scan_path = Some(path.clone());
        guard.scan_in_progress = false;
        guard.last_scan_result = Some(result.clone());
    }

    // Emit completion event with summary
    let _ = app.emit(
        "scan:complete",
        &serde_json::json!({
            "path": path,
            "total_files": result.total_files,
            "total_size": result.total_size,
        }),
    );

    Ok(result)
}

/// Retrieve the most recent scan result stored in state (if any)
#[tauri::command]
pub async fn get_scan_results(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Option<ScanResult>, String> {
    let guard = state.lock().map_err(|_| "Failed to lock app state")?;
    Ok(guard.last_scan_result.clone())
}
