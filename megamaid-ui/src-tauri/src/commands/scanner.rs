use megamaid::scanner::{ParallelScanner, ScannerConfig};
use megamaid::models::FileEntry;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub entries: Vec<FileEntry>,
    pub total_files: usize,
    pub total_size: u64,
    pub errors: Vec<String>,
}

/// Scan a directory and return file entries
#[tauri::command]
pub async fn scan_directory(path: String, config: ScannerConfig) -> Result<ScanResult, String> {
    let scan_path = PathBuf::from(&path);

    if !scan_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    let scanner = ParallelScanner::new(config);
    let entries = scanner.scan(&scan_path).map_err(|e| e.to_string())?;

    let total_files = entries.len();
    let total_size: u64 = entries.iter().map(|e| e.size).sum();

    Ok(ScanResult {
        entries,
        total_files,
        total_size,
        errors: vec![],
    })
}

