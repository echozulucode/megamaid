//! Progress tracking for scan operations.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Tracks progress during a file system scan.
#[derive(Debug, Default)]
pub struct ScanProgress {
    /// Number of files scanned
    pub files_scanned: AtomicUsize,

    /// Total bytes scanned
    pub bytes_scanned: AtomicU64,

    /// Number of directories visited
    pub directories_visited: AtomicUsize,
}

/// A snapshot of scan progress.
#[derive(Debug, Clone, Copy)]
pub struct ProgressReport {
    /// Number of files scanned
    pub files: usize,

    /// Total bytes scanned
    pub bytes: u64,

    /// Number of directories visited
    pub dirs: usize,
}

impl ScanProgress {
    /// Creates a new ScanProgress tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Increments the file count and adds to the byte count.
    pub fn increment_file(&self, size: u64) {
        self.files_scanned.fetch_add(1, Ordering::Relaxed);
        self.bytes_scanned.fetch_add(size, Ordering::Relaxed);
    }

    /// Increments the directory count.
    pub fn increment_directory(&self) {
        self.directories_visited
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Returns a snapshot of the current progress.
    pub fn report(&self) -> ProgressReport {
        ProgressReport {
            files: self.files_scanned.load(Ordering::Relaxed),
            bytes: self.bytes_scanned.load(Ordering::Relaxed),
            dirs: self.directories_visited.load(Ordering::Relaxed),
        }
    }

    /// Resets all counters to zero.
    pub fn reset(&self) {
        self.files_scanned.store(0, Ordering::Relaxed);
        self.bytes_scanned.store(0, Ordering::Relaxed);
        self.directories_visited.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_progress_tracking() {
        let progress = ScanProgress::new();

        progress.increment_file(100);
        progress.increment_file(200);
        progress.increment_directory();

        let report = progress.report();
        assert_eq!(report.files, 2);
        assert_eq!(report.bytes, 300);
        assert_eq!(report.dirs, 1);
    }

    #[test]
    fn test_reset() {
        let progress = ScanProgress::new();

        progress.increment_file(1000);
        progress.increment_directory();

        progress.reset();

        let report = progress.report();
        assert_eq!(report.files, 0);
        assert_eq!(report.bytes, 0);
        assert_eq!(report.dirs, 0);
    }

    #[test]
    fn test_concurrent_progress_updates() {
        let progress = Arc::new(ScanProgress::new());
        let mut handles = vec![];

        // Spawn 10 threads, each incrementing 100 times
        for _ in 0..10 {
            let p = Arc::clone(&progress);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    p.increment_file(1);
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        let report = progress.report();
        assert_eq!(report.files, 1000);
        assert_eq!(report.bytes, 1000);
    }

    #[test]
    fn test_default_values() {
        let progress = ScanProgress::default();
        let report = progress.report();

        assert_eq!(report.files, 0);
        assert_eq!(report.bytes, 0);
        assert_eq!(report.dirs, 0);
    }
}
