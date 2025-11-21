//! File system entry representation with metadata for drift detection.

use std::path::PathBuf;
use std::time::SystemTime;

/// Represents a file or directory entry with metadata for cleanup analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct FileEntry {
    /// Absolute path to the file or directory
    pub path: PathBuf,

    /// Size in bytes (0 for directories)
    pub size: u64,

    /// Last modification time
    pub modified: SystemTime,

    /// Type of entry (file or directory)
    pub entry_type: EntryType,

    /// Optional NTFS MFT record number for rename detection (Windows-specific)
    pub file_id: Option<u64>,
}

/// Type of file system entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    /// Regular file
    File,
    /// Directory
    Directory,
}

impl FileEntry {
    /// Creates a new FileEntry.
    pub fn new(path: PathBuf, size: u64, modified: SystemTime, entry_type: EntryType) -> Self {
        Self {
            path,
            size,
            modified,
            entry_type,
            file_id: None,
        }
    }

    /// Creates a new FileEntry with an optional file ID.
    pub fn with_file_id(
        path: PathBuf,
        size: u64,
        modified: SystemTime,
        entry_type: EntryType,
        file_id: Option<u64>,
    ) -> Self {
        Self {
            path,
            size,
            modified,
            entry_type,
            file_id,
        }
    }

    /// Returns true if this entry is a file.
    pub fn is_file(&self) -> bool {
        matches!(self.entry_type, EntryType::File)
    }

    /// Returns true if this entry is a directory.
    pub fn is_directory(&self) -> bool {
        matches!(self.entry_type, EntryType::Directory)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_entry_creation() {
        let path = PathBuf::from("/test/file.txt");
        let size = 1024;
        let modified = SystemTime::now();

        let entry = FileEntry::new(path.clone(), size, modified, EntryType::File);

        assert_eq!(entry.path, path);
        assert_eq!(entry.size, size);
        assert_eq!(entry.modified, modified);
        assert_eq!(entry.entry_type, EntryType::File);
        assert_eq!(entry.file_id, None);
    }

    #[test]
    fn test_file_entry_with_file_id() {
        let path = PathBuf::from("/test/file.txt");
        let file_id = Some(123456);

        let entry = FileEntry::with_file_id(
            path.clone(),
            1024,
            SystemTime::now(),
            EntryType::File,
            file_id,
        );

        assert_eq!(entry.file_id, file_id);
    }

    #[test]
    fn test_file_id_optional() {
        // Verify file_id can be None (for non-NTFS)
        let entry = FileEntry::new(
            PathBuf::from("/test"),
            0,
            SystemTime::now(),
            EntryType::Directory,
        );

        assert_eq!(entry.file_id, None);
    }

    #[test]
    fn test_is_file() {
        let file_entry = FileEntry::new(
            PathBuf::from("/test/file.txt"),
            100,
            SystemTime::now(),
            EntryType::File,
        );

        assert!(file_entry.is_file());
        assert!(!file_entry.is_directory());
    }

    #[test]
    fn test_is_directory() {
        let dir_entry = FileEntry::new(
            PathBuf::from("/test/dir"),
            0,
            SystemTime::now(),
            EntryType::Directory,
        );

        assert!(dir_entry.is_directory());
        assert!(!dir_entry.is_file());
    }

    #[test]
    #[allow(clippy::useless_vec)]
    fn test_file_entry_ordering_by_size() {
        let small = FileEntry::new(
            PathBuf::from("/small.txt"),
            100,
            SystemTime::now(),
            EntryType::File,
        );

        let large = FileEntry::new(
            PathBuf::from("/large.txt"),
            1000,
            SystemTime::now(),
            EntryType::File,
        );

        let mut entries = vec![large.clone(), small.clone()];
        entries.sort_by(|a, b| b.size.cmp(&a.size));

        assert_eq!(entries[0].size, 1000);
        assert_eq!(entries[1].size, 100);
    }
}
