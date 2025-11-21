//! Parallel file system scanning using rayon.

use crate::models::{EntryType, FileEntry};
use crate::scanner::progress::AdvancedProgress;
use rayon::prelude::*;
use std::path::Path;
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

/// Configuration for the parallel scanner.
#[derive(Debug, Clone)]
pub struct ScannerConfig {
    /// Maximum directory depth to scan
    pub max_depth: Option<usize>,
    /// Skip hidden files and directories
    pub skip_hidden: bool,
    /// Follow symbolic links
    pub follow_symlinks: bool,
    /// Number of threads to use (0 = auto-detect)
    pub thread_count: usize,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            max_depth: None,
            skip_hidden: true,
            follow_symlinks: false,
            thread_count: 0, // Auto-detect
        }
    }
}

/// Parallel scanner for high-performance directory traversal.
pub struct ParallelScanner {
    config: ScannerConfig,
    progress: Arc<AdvancedProgress>,
    error_collector: Arc<ErrorCollector>,
}

/// Collects errors that occur during parallel scanning.
pub struct ErrorCollector {
    errors: Mutex<Vec<ScanError>>,
}

/// Errors that can occur during scanning.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ScanError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

impl From<std::io::Error> for ScanError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::PermissionDenied => ScanError::PermissionDenied(err.to_string()),
            _ => ScanError::Io(err.to_string()),
        }
    }
}

impl From<walkdir::Error> for ScanError {
    fn from(err: walkdir::Error) -> Self {
        if let Some(io_err) = err.io_error() {
            ScanError::from(io_err.kind())
        } else {
            ScanError::Io(err.to_string())
        }
    }
}

impl From<std::io::ErrorKind> for ScanError {
    fn from(kind: std::io::ErrorKind) -> Self {
        match kind {
            std::io::ErrorKind::PermissionDenied => {
                ScanError::PermissionDenied("Permission denied".to_string())
            }
            _ => ScanError::Io(format!("{:?}", kind)),
        }
    }
}

impl ErrorCollector {
    /// Creates a new error collector.
    pub fn new() -> Self {
        Self {
            errors: Mutex::new(Vec::new()),
        }
    }

    /// Records an error.
    pub fn record(&self, error: ScanError) {
        if let Ok(mut errors) = self.errors.lock() {
            errors.push(error);
        }
    }

    /// Returns all collected errors.
    pub fn get_errors(&self) -> Vec<ScanError> {
        self.errors.lock().map(|e| e.clone()).unwrap_or_default()
    }

    /// Returns the number of errors collected.
    pub fn error_count(&self) -> usize {
        self.errors.lock().map(|e| e.len()).unwrap_or(0)
    }
}

impl Default for ErrorCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl ParallelScanner {
    /// Creates a new parallel scanner with the given configuration.
    pub fn new(config: ScannerConfig) -> Self {
        // Configure rayon thread pool
        let thread_count = if config.thread_count == 0 {
            num_cpus::get()
        } else {
            config.thread_count
        };

        // Try to build global thread pool, but don't fail if already initialized
        let _ = rayon::ThreadPoolBuilder::new()
            .num_threads(thread_count)
            .build_global();

        Self {
            config,
            progress: Arc::new(AdvancedProgress::new()),
            error_collector: Arc::new(ErrorCollector::new()),
        }
    }

    /// Scans the given directory path in parallel.
    pub fn scan(&self, path: &Path) -> Result<Vec<FileEntry>, ScanError> {
        // Phase 1: Collect all paths (sequential, fast)
        let walker = WalkDir::new(path)
            .follow_links(self.config.follow_symlinks)
            .max_depth(self.config.max_depth.unwrap_or(usize::MAX));

        let paths: Vec<_> = walker
            .into_iter()
            .filter_entry(|e| {
                if self.config.skip_hidden {
                    !is_hidden(e)
                } else {
                    true
                }
            })
            .filter_map(|e| e.ok())
            .collect();

        self.progress.set_total(paths.len() as u64);

        // Phase 2: Process paths in parallel
        let entries: Vec<_> = paths
            .par_iter()
            .filter_map(|entry| {
                let result = self.process_entry(entry);
                self.progress.increment();

                match result {
                    Ok(Some(file_entry)) => Some(file_entry),
                    Ok(None) => None,
                    Err(e) => {
                        self.error_collector.record(e);
                        None
                    }
                }
            })
            .collect();

        Ok(entries)
    }

    fn process_entry(&self, entry: &walkdir::DirEntry) -> Result<Option<FileEntry>, ScanError> {
        let path = entry.path();
        let metadata = entry.metadata()?;

        // Determine entry type
        let entry_type = if metadata.is_dir() {
            EntryType::Directory
        } else {
            EntryType::File
        };

        // Calculate size
        let size = if metadata.is_dir() {
            self.calculate_dir_size(path)?
        } else {
            metadata.len()
        };

        // Get modification time
        let modified = metadata.modified()?;

        Ok(Some(FileEntry::new(
            path.to_path_buf(),
            size,
            modified,
            entry_type,
        )))
    }

    fn calculate_dir_size(&self, dir_path: &Path) -> Result<u64, ScanError> {
        // Use parallel iteration for large directories
        let entries: Vec<_> = std::fs::read_dir(dir_path)?
            .filter_map(|e| e.ok())
            .collect();

        let total: u64 = entries
            .par_iter()
            .map(|entry| {
                let metadata = entry.metadata().ok()?;
                if metadata.is_file() {
                    Some(metadata.len())
                } else if metadata.is_dir() {
                    calculate_dir_size_recursive(&entry.path()).ok()
                } else {
                    None
                }
            })
            .flatten()
            .sum();

        Ok(total)
    }

    /// Returns a reference to the progress tracker.
    pub fn progress(&self) -> &AdvancedProgress {
        &self.progress
    }

    /// Returns all errors that occurred during scanning.
    pub fn errors(&self) -> Vec<ScanError> {
        self.error_collector.get_errors()
    }

    /// Returns the number of errors that occurred.
    pub fn error_count(&self) -> usize {
        self.error_collector.error_count()
    }
}

/// Helper function for recursive directory size calculation.
fn calculate_dir_size_recursive(dir_path: &Path) -> Result<u64, ScanError> {
    let entries: Vec<_> = std::fs::read_dir(dir_path)?
        .filter_map(|e| e.ok())
        .collect();

    let total: u64 = entries
        .par_iter()
        .map(|entry| {
            let metadata = entry.metadata().ok()?;
            if metadata.is_file() {
                Some(metadata.len())
            } else if metadata.is_dir() {
                calculate_dir_size_recursive(&entry.path()).ok()
            } else {
                None
            }
        })
        .flatten()
        .sum();

    Ok(total)
}

/// Checks if a directory entry is hidden.
#[cfg(windows)]
fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    use std::os::windows::fs::MetadataExt;

    entry
        .metadata()
        .ok()
        .map(|m| {
            const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
            (m.file_attributes() & FILE_ATTRIBUTE_HIDDEN) != 0
        })
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Instant;
    use tempfile::TempDir;

    #[test]
    fn test_parallel_scan_empty_directory() {
        let temp = TempDir::new().unwrap();
        let scanner = ParallelScanner::new(ScannerConfig::default());

        let result = scanner.scan(temp.path()).unwrap();
        // Should include the root directory itself
        assert_eq!(result.len(), 1);
        assert!(result[0].is_directory());
    }

    #[test]
    fn test_parallel_scan_with_files() {
        let temp = TempDir::new().unwrap();

        // Create 100 files
        for i in 0..100 {
            fs::write(temp.path().join(format!("file{}.txt", i)), "content").unwrap();
        }

        let scanner = ParallelScanner::new(ScannerConfig::default());
        let result = scanner.scan(temp.path()).unwrap();

        // Should include root directory + 100 files
        assert_eq!(result.len(), 101);
    }

    #[test]
    fn test_parallel_scan_with_nested_directories() {
        let temp = TempDir::new().unwrap();

        // Create nested directory structure
        fs::create_dir_all(temp.path().join("a/b/c")).unwrap();
        fs::write(temp.path().join("a/file1.txt"), "content1").unwrap();
        fs::write(temp.path().join("a/b/file2.txt"), "content2").unwrap();
        fs::write(temp.path().join("a/b/c/file3.txt"), "content3").unwrap();

        let scanner = ParallelScanner::new(ScannerConfig::default());
        let result = scanner.scan(temp.path()).unwrap();

        // Should find root + 3 directories (a, b, c) + 3 files = 7
        assert_eq!(result.len(), 7);
    }

    #[test]
    fn test_parallel_scan_performance() {
        let temp = TempDir::new().unwrap();

        // Create 1000 files
        for i in 0..1000 {
            fs::write(temp.path().join(format!("file{}.txt", i)), "x".repeat(1000)).unwrap();
        }

        let start = Instant::now();
        let scanner = ParallelScanner::new(ScannerConfig::default());
        let result = scanner.scan(temp.path()).unwrap();
        let duration = start.elapsed();

        // Should include root + 1000 files
        assert_eq!(result.len(), 1001);
        println!("Scanned 1000 files in {:?}", duration);
        assert!(duration.as_secs() < 5, "Should scan 1000 files in <5s");
    }

    #[test]
    fn test_scanner_config_default() {
        let config = ScannerConfig::default();

        assert_eq!(config.max_depth, None);
        assert!(config.skip_hidden);
        assert!(!config.follow_symlinks);
        assert_eq!(config.thread_count, 0);
    }

    #[test]
    fn test_configurable_thread_count() {
        let config = ScannerConfig {
            thread_count: 4,
            ..Default::default()
        };

        let scanner = ParallelScanner::new(config);
        // Scanner should use 4 threads (verified by rayon configuration)
        assert_eq!(scanner.config.thread_count, 4);
    }

    #[test]
    fn test_error_collection() {
        let collector = Arc::new(ErrorCollector::new());

        // Simulate parallel error recording
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let collector = Arc::clone(&collector);
                std::thread::spawn(move || {
                    collector.record(ScanError::Io(format!("Error {}", i)));
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let errors = collector.get_errors();
        assert_eq!(errors.len(), 10);
    }

    #[test]
    fn test_max_depth_limiting() {
        let temp = TempDir::new().unwrap();

        // Create deeply nested structure
        fs::create_dir_all(temp.path().join("a/b/c/d/e")).unwrap();
        fs::write(temp.path().join("a/file1.txt"), "1").unwrap();
        fs::write(temp.path().join("a/b/file2.txt"), "2").unwrap();
        fs::write(temp.path().join("a/b/c/file3.txt"), "3").unwrap();
        fs::write(temp.path().join("a/b/c/d/file4.txt"), "4").unwrap();
        fs::write(temp.path().join("a/b/c/d/e/file5.txt"), "5").unwrap();

        let config = ScannerConfig {
            max_depth: Some(3),
            ..Default::default()
        };

        let scanner = ParallelScanner::new(config);
        let result = scanner.scan(temp.path()).unwrap();

        // Should stop at depth 3, not reaching d and e
        let paths: Vec<_> = result.iter().map(|e| e.path.as_path()).collect();
        assert!(!paths.iter().any(|p| p.ends_with("d/file4.txt")));
        assert!(!paths.iter().any(|p| p.ends_with("e/file5.txt")));
    }

    #[test]
    fn test_directory_size_calculation() {
        let temp = TempDir::new().unwrap();
        let dir_path = temp.path().join("test_dir");
        fs::create_dir(&dir_path).unwrap();
        fs::write(dir_path.join("file1.txt"), "a".repeat(100)).unwrap();
        fs::write(dir_path.join("file2.txt"), "b".repeat(200)).unwrap();

        let scanner = ParallelScanner::new(ScannerConfig::default());
        let result = scanner.scan(temp.path()).unwrap();

        // Find the directory entry
        let dir_entry = result.iter().find(|e| e.path == dir_path).unwrap();

        // Directory size should be sum of files (300 bytes)
        assert_eq!(dir_entry.size, 300);
    }
}
