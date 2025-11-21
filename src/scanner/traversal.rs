//! Directory traversal implementation.

use crate::models::{EntryType, FileEntry};
use std::path::Path;
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};

/// Error types for scanning operations.
#[derive(Debug, Error)]
pub enum ScanError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Walk error: {0}")]
    Walk(#[from] walkdir::Error),

    #[error("Path does not exist: {0}")]
    PathNotFound(String),
}

/// Configuration for file system scanning.
#[derive(Debug, Clone, Default)]
pub struct ScanConfig {
    /// Whether to follow symbolic links
    pub follow_links: bool,

    /// Maximum depth to traverse (None = unlimited)
    pub max_depth: Option<usize>,

    /// Whether to skip hidden files/directories
    pub skip_hidden: bool,
}

/// Scans directories and collects file metadata.
pub struct FileScanner {
    config: ScanConfig,
}

impl FileScanner {
    /// Creates a new FileScanner with the given configuration.
    pub fn new(config: ScanConfig) -> Self {
        Self { config }
    }

    /// Scans the given root directory and returns all entries.
    pub fn scan(&self, root: &Path) -> Result<Vec<FileEntry>, ScanError> {
        if !root.exists() {
            return Err(ScanError::PathNotFound(root.display().to_string()));
        }

        let mut entries = Vec::new();
        let max_depth = self.config.max_depth.unwrap_or(usize::MAX);

        for entry in WalkDir::new(root)
            .follow_links(self.config.follow_links)
            .max_depth(max_depth)
        {
            let entry = entry?;

            if self.should_skip(&entry) {
                continue;
            }

            entries.push(self.to_file_entry(entry)?);
        }

        Ok(entries)
    }

    /// Determines if an entry should be skipped.
    fn should_skip(&self, entry: &DirEntry) -> bool {
        if self.config.skip_hidden {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with('.') && name != "." {
                    return true;
                }
            }
        }
        false
    }

    /// Converts a DirEntry to a FileEntry.
    fn to_file_entry(&self, entry: DirEntry) -> Result<FileEntry, ScanError> {
        let metadata = entry.metadata()?;

        let entry_type = if metadata.is_dir() {
            EntryType::Directory
        } else {
            EntryType::File
        };

        // For directories, calculate recursive size
        let size = if metadata.is_dir() {
            self.calculate_dir_size(entry.path())?
        } else {
            metadata.len()
        };

        Ok(FileEntry::new(
            entry.path().to_path_buf(),
            size,
            metadata.modified()?,
            entry_type,
        ))
    }

    /// Calculates the total size of all files in a directory recursively.
    fn calculate_dir_size(&self, dir_path: &Path) -> Result<u64, ScanError> {
        let mut total_size = 0u64;

        for entry in WalkDir::new(dir_path).follow_links(false) {
            let entry = entry?;
            let metadata = entry.metadata()?;

            // Only count files, not directories themselves
            if metadata.is_file() {
                total_size = total_size.saturating_add(metadata.len());
            }
        }

        Ok(total_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scan_empty_directory() {
        let temp = TempDir::new().unwrap();
        let scanner = FileScanner::new(ScanConfig::default());
        let results = scanner.scan(temp.path()).unwrap();

        // Should contain only the root directory itself
        assert_eq!(results.len(), 1);
        assert!(results[0].is_directory());
    }

    #[test]
    fn test_scan_single_file() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("test.txt"), "content").unwrap();

        let scanner = FileScanner::new(ScanConfig::default());
        let results = scanner.scan(temp.path()).unwrap();

        // Should find: root directory + test.txt
        assert_eq!(results.len(), 2);

        let file = results
            .iter()
            .find(|e| e.path.ends_with("test.txt"))
            .unwrap();
        assert!(file.is_file());
        assert_eq!(file.size, 7); // "content" = 7 bytes
    }

    #[test]
    fn test_scan_nested_directories() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("a/b/c")).unwrap();
        fs::write(temp.path().join("a/b/c/file.txt"), "test").unwrap();

        let scanner = FileScanner::new(ScanConfig::default());
        let results = scanner.scan(temp.path()).unwrap();

        // Should find: root + a + a/b + a/b/c + a/b/c/file.txt = 5 entries
        assert!(results.len() >= 5);

        let file = results
            .iter()
            .find(|e| e.path.ends_with("file.txt"))
            .unwrap();
        assert!(file.is_file());
        assert_eq!(file.size, 4); // "test" = 4 bytes
    }

    #[test]
    fn test_skip_hidden_files() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join(".hidden"), "secret").unwrap();
        fs::write(temp.path().join("visible.txt"), "public").unwrap();

        let config = ScanConfig {
            skip_hidden: true,
            ..Default::default()
        };
        let scanner = FileScanner::new(config);
        let results = scanner.scan(temp.path()).unwrap();

        // Should not find .hidden file
        assert!(!results.iter().any(|e| {
            e.path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n == ".hidden")
                .unwrap_or(false)
        }));

        // Should find visible.txt
        assert!(results.iter().any(|e| e.path.ends_with("visible.txt")));
    }

    #[test]
    fn test_max_depth_limiting() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("a/b/c/d")).unwrap();

        let config = ScanConfig {
            max_depth: Some(2),
            ..Default::default()
        };
        let scanner = FileScanner::new(config);
        let results = scanner.scan(temp.path()).unwrap();

        // Should not find "d" directory at depth 3
        assert!(!results.iter().any(|e| {
            e.path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n == "d")
                .unwrap_or(false)
        }));
    }

    #[test]
    fn test_metadata_accuracy() {
        let temp = TempDir::new().unwrap();
        let content = "x".repeat(1024); // 1KB
        fs::write(temp.path().join("sized.txt"), &content).unwrap();

        let scanner = FileScanner::new(ScanConfig::default());
        let results = scanner.scan(temp.path()).unwrap();

        let file = results
            .iter()
            .find(|e| e.path.ends_with("sized.txt"))
            .unwrap();
        assert_eq!(file.size, 1024);
        assert!(file.modified.elapsed().unwrap().as_secs() < 5);
    }

    #[test]
    fn test_nonexistent_path() {
        let scanner = FileScanner::new(ScanConfig::default());
        let result = scanner.scan(Path::new("/nonexistent/path"));

        assert!(result.is_err());
        match result {
            Err(ScanError::PathNotFound(_)) => (),
            _ => panic!("Expected PathNotFound error"),
        }
    }

    #[test]
    fn test_default_config() {
        let config = ScanConfig::default();

        assert!(!config.follow_links);
        assert_eq!(config.max_depth, None);
        assert!(!config.skip_hidden);
    }
}
