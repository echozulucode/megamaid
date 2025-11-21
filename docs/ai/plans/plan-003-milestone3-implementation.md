# Milestone 3: Parallel Operations & Advanced Features - Phased Implementation Plan

## 🎯 STATUS: READY TO START

**Target Start Date**: TBD
**Estimated Duration**: 3-4 weeks
**Prerequisites**: ✅ Milestones 1 & 2 Complete (126 tests passing)

## Overview

**Goal**: Implement parallel scanning and deletion operations, advanced progress reporting, and configuration file support to significantly improve performance and user experience.

**Success Criteria**:
- Scan performance improved by 3-5x on multi-core systems
- Deletion performance improved by 2-4x for large file sets
- Real-time ETA and throughput statistics during operations
- Configuration file support for custom rules and settings
- All components maintain >85% test coverage
- Performance: scan 1M files in <2 minutes, delete 100K files in <30 seconds

## Key Design Decisions

### Parallelization Strategy

**Rayon-based Parallel Processing**:
```rust
use rayon::prelude::*;

// Parallel scanning with work-stealing
files.par_iter()
    .map(|path| scan_file(path))
    .collect()

// Parallel deletion with bounded thread pool
plan.entries.par_iter()
    .filter(|e| e.action == Delete)
    .for_each(|entry| delete_entry(entry))
```

**Benefits**:
- Automatic work-stealing for load balancing
- Configurable thread pool size
- Easy to reason about (data parallelism)
- Integrates well with existing code

**Considerations**:
- Filesystem bottlenecks (especially on HDDs)
- Lock contention for shared state (progress tracking)
- Error handling in parallel contexts
- Deterministic ordering for predictable results

### Progress Reporting Architecture

**Real-time Progress with ETA**:
```rust
pub struct AdvancedProgress {
    total_items: AtomicU64,
    processed_items: AtomicU64,
    start_time: Instant,
    throughput: Arc<Mutex<VecDeque<(Instant, u64)>>>, // Rolling window
}

impl AdvancedProgress {
    pub fn estimate_eta(&self) -> Duration {
        // Calculate ETA based on recent throughput
    }

    pub fn current_throughput(&self) -> f64 {
        // Items per second
    }
}
```

**Features**:
- Real-time ETA based on recent throughput
- Progress percentage with visual bar
- Current throughput (files/sec, MB/sec)
- Estimated time remaining
- Smoothed updates to avoid flickering

### Configuration File Format

**YAML Configuration**:
```yaml
version: "0.1.0"

# Scanner configuration
scanner:
  max_depth: 10
  skip_hidden: true
  follow_symlinks: false
  parallel: true
  thread_count: 0  # 0 = auto-detect

# Custom detection rules
detection_rules:
  - name: "python_cache"
    pattern: "**/__pycache__/**"
    action: delete
    reason: "Python cache directory"

  - name: "large_logs"
    pattern: "**/*.log"
    min_size: 100MB
    action: review
    reason: "Large log file"

# Execution defaults
execution:
  default_mode: interactive
  always_verify: true
  backup_dir: null
  fail_fast: false

# Performance tuning
performance:
  scan_batch_size: 1000
  delete_batch_size: 100
  progress_update_interval: 100ms
```

### Thread Safety & Concurrency

**Arc + Mutex for Shared State**:
```rust
pub struct ParallelScanner {
    progress: Arc<AdvancedProgress>,
    config: Arc<ScannerConfig>,
    results: Arc<Mutex<Vec<FileEntry>>>,
}
```

**Lock-free Progress Tracking**:
```rust
pub struct AdvancedProgress {
    processed: AtomicU64,  // Lock-free atomic operations
    total: AtomicU64,
    start_time: Instant,   // Read-only after initialization
}
```

**Error Collection**:
```rust
pub struct ErrorCollector {
    errors: Arc<Mutex<Vec<ScanError>>>,
}

impl ErrorCollector {
    pub fn record(&self, error: ScanError) {
        self.errors.lock().unwrap().push(error);
    }
}
```

---

## Phase 3.1: Parallel Scanning (Week 1)

### Objectives
Implement parallel directory traversal and file scanning using rayon for significant performance improvements.

### Implementation Tasks

#### 3.1.1: Parallel Scanner Core

```rust
// src/scanner/parallel.rs
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use walkdir::WalkDir;

pub struct ParallelScanner {
    config: ScannerConfig,
    progress: Arc<AdvancedProgress>,
    error_collector: Arc<ErrorCollector>,
}

pub struct ScannerConfig {
    pub max_depth: Option<usize>,
    pub skip_hidden: bool,
    pub follow_symlinks: bool,
    pub thread_count: usize,  // 0 = auto
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            max_depth: None,
            skip_hidden: true,
            follow_symlinks: false,
            thread_count: 0,  // Auto-detect
        }
    }
}

impl ParallelScanner {
    pub fn new(config: ScannerConfig) -> Self {
        // Configure rayon thread pool
        let thread_count = if config.thread_count == 0 {
            num_cpus::get()
        } else {
            config.thread_count
        };

        rayon::ThreadPoolBuilder::new()
            .num_threads(thread_count)
            .build_global()
            .unwrap_or_else(|_| {
                // Already initialized, that's fine
            });

        Self {
            config,
            progress: Arc::new(AdvancedProgress::new()),
            error_collector: Arc::new(ErrorCollector::new()),
        }
    }

    pub fn scan(&self, path: &Path) -> Result<Vec<FileEntry>, ScanError> {
        // Phase 1: Collect all paths (sequential, fast)
        let paths: Vec<_> = WalkDir::new(path)
            .follow_links(self.config.follow_symlinks)
            .max_depth(self.config.max_depth.unwrap_or(usize::MAX))
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

        // Calculate size
        let size = if metadata.is_dir() {
            self.calculate_dir_size(path)?
        } else {
            metadata.len()
        };

        // Get modification time
        let modified = metadata.modified()?;
        let modified_rfc3339 = chrono::DateTime::<chrono::Utc>::from(modified)
            .to_rfc3339();

        Ok(Some(FileEntry {
            path: path.to_path_buf(),
            size,
            modified: modified_rfc3339,
            is_dir: metadata.is_dir(),
        }))
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
                    self.calculate_dir_size(&entry.path()).ok()
                } else {
                    None
                }
            })
            .flatten()
            .sum();

        Ok(total)
    }

    pub fn progress(&self) -> &AdvancedProgress {
        &self.progress
    }

    pub fn errors(&self) -> Vec<ScanError> {
        self.error_collector.get_errors()
    }
}

pub struct ErrorCollector {
    errors: Mutex<Vec<ScanError>>,
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self {
            errors: Mutex::new(Vec::new()),
        }
    }

    pub fn record(&self, error: ScanError) {
        if let Ok(mut errors) = self.errors.lock() {
            errors.push(error);
        }
    }

    pub fn get_errors(&self) -> Vec<ScanError> {
        self.errors.lock()
            .map(|e| e.clone())
            .unwrap_or_default()
    }
}

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
        ScanError::Io(err.to_string())
    }
}
```

#### 3.1.2: Advanced Progress Tracking

```rust
// src/scanner/progress.rs
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

pub struct AdvancedProgress {
    total: AtomicU64,
    processed: AtomicU64,
    start_time: Instant,
    throughput_history: Arc<Mutex<VecDeque<ThroughputSample>>>,
}

struct ThroughputSample {
    timestamp: Instant,
    items_processed: u64,
}

impl AdvancedProgress {
    pub fn new() -> Self {
        Self {
            total: AtomicU64::new(0),
            processed: AtomicU64::new(0),
            start_time: Instant::now(),
            throughput_history: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
        }
    }

    pub fn set_total(&self, total: u64) {
        self.total.store(total, Ordering::Relaxed);
    }

    pub fn increment(&self) {
        let new_value = self.processed.fetch_add(1, Ordering::Relaxed) + 1;

        // Record throughput sample every 100 items
        if new_value % 100 == 0 {
            self.record_throughput_sample(new_value);
        }
    }

    pub fn increment_by(&self, amount: u64) {
        let new_value = self.processed.fetch_add(amount, Ordering::Relaxed) + amount;
        self.record_throughput_sample(new_value);
    }

    fn record_throughput_sample(&self, items_processed: u64) {
        if let Ok(mut history) = self.throughput_history.lock() {
            let sample = ThroughputSample {
                timestamp: Instant::now(),
                items_processed,
            };

            history.push_back(sample);

            // Keep only last 10 seconds of samples
            let cutoff = Instant::now() - Duration::from_secs(10);
            while history.front().map_or(false, |s| s.timestamp < cutoff) {
                history.pop_front();
            }
        }
    }

    pub fn get_processed(&self) -> u64 {
        self.processed.load(Ordering::Relaxed)
    }

    pub fn get_total(&self) -> u64 {
        self.total.load(Ordering::Relaxed)
    }

    pub fn percentage(&self) -> f64 {
        let total = self.get_total();
        if total == 0 {
            return 0.0;
        }
        (self.get_processed() as f64 / total as f64) * 100.0
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn current_throughput(&self) -> Option<f64> {
        let history = self.throughput_history.lock().ok()?;

        if history.len() < 2 {
            return None;
        }

        let first = history.front()?;
        let last = history.back()?;

        let duration = last.timestamp.duration_since(first.timestamp);
        let items = last.items_processed - first.items_processed;

        if duration.as_secs_f64() < 0.1 {
            return None;
        }

        Some(items as f64 / duration.as_secs_f64())
    }

    pub fn estimate_eta(&self) -> Option<Duration> {
        let throughput = self.current_throughput()?;
        if throughput < 0.1 {
            return None;
        }

        let remaining = self.get_total().saturating_sub(self.get_processed());
        let seconds_remaining = remaining as f64 / throughput;

        Some(Duration::from_secs_f64(seconds_remaining))
    }

    pub fn format_eta(&self) -> String {
        match self.estimate_eta() {
            Some(eta) => {
                let seconds = eta.as_secs();
                if seconds < 60 {
                    format!("{}s", seconds)
                } else if seconds < 3600 {
                    format!("{}m {}s", seconds / 60, seconds % 60)
                } else {
                    format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
                }
            }
            None => "calculating...".to_string(),
        }
    }
}
```

#### 3.1.3: CLI Integration with Advanced Progress

```rust
// src/cli/orchestrator.rs (additions)
fn run_scan_parallel(
    path: &Path,
    output: PathBuf,
    config: ScannerConfig,
) -> Result<()> {
    println!("🔍 Scanning directory (parallel): {}", path.display());
    println!();

    let scanner = ParallelScanner::new(config);
    let progress = scanner.progress().clone();

    // Spawn progress reporter thread
    let progress_handle = std::thread::spawn(move || {
        let pb = ProgressBar::new(100);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {percent}% | {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );

        loop {
            let processed = progress.get_processed();
            let total = progress.get_total();
            let percentage = progress.percentage();
            let throughput = progress.current_throughput().unwrap_or(0.0);
            let eta = progress.format_eta();

            pb.set_position(percentage as u64);
            pb.set_message(format!(
                "{}/{} files | {:.0} files/sec | ETA: {}",
                processed, total, throughput, eta
            ));

            if processed >= total && total > 0 {
                pb.finish_with_message("✓ Scan complete");
                break;
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    });

    // Perform scan
    let entries = scanner.scan(path)?;

    // Wait for progress reporter
    progress_handle.join().unwrap();

    // Report errors
    let errors = scanner.errors();
    if !errors.is_empty() {
        println!("\n⚠️  {} errors during scan:", errors.len());
        for error in errors.iter().take(10) {
            println!("  • {}", error);
        }
        if errors.len() > 10 {
            println!("  ... and {} more", errors.len() - 10);
        }
    }

    // Continue with detection and plan generation...
    Ok(())
}
```

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parallel_scan_empty_directory() {
        let temp = TempDir::new().unwrap();
        let scanner = ParallelScanner::new(ScannerConfig::default());

        let result = scanner.scan(temp.path()).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_parallel_scan_with_files() {
        let temp = TempDir::new().unwrap();

        // Create 100 files
        for i in 0..100 {
            std::fs::write(temp.path().join(format!("file{}.txt", i)), "content").unwrap();
        }

        let scanner = ParallelScanner::new(ScannerConfig::default());
        let result = scanner.scan(temp.path()).unwrap();

        assert_eq!(result.len(), 100);
    }

    #[test]
    fn test_parallel_scan_performance() {
        let temp = TempDir::new().unwrap();

        // Create 1000 files
        for i in 0..1000 {
            std::fs::write(temp.path().join(format!("file{}.txt", i)), "x".repeat(1000)).unwrap();
        }

        let start = Instant::now();
        let scanner = ParallelScanner::new(ScannerConfig::default());
        let result = scanner.scan(temp.path()).unwrap();
        let duration = start.elapsed();

        assert_eq!(result.len(), 1000);
        println!("Scanned 1000 files in {:?}", duration);
        assert!(duration.as_secs() < 5, "Should scan 1000 files in <5s");
    }

    #[test]
    fn test_progress_tracking() {
        let progress = AdvancedProgress::new();
        progress.set_total(100);

        for _ in 0..50 {
            progress.increment();
        }

        assert_eq!(progress.get_processed(), 50);
        assert_eq!(progress.percentage(), 50.0);
    }

    #[test]
    fn test_eta_calculation() {
        let progress = AdvancedProgress::new();
        progress.set_total(1000);

        // Simulate processing
        for i in 0..100 {
            progress.increment();
            if i % 10 == 0 {
                std::thread::sleep(Duration::from_millis(10));
            }
        }

        let eta = progress.estimate_eta();
        assert!(eta.is_some());
    }

    #[test]
    fn test_throughput_calculation() {
        let progress = AdvancedProgress::new();
        progress.set_total(1000);

        // Simulate processing at known rate
        for _ in 0..200 {
            progress.increment();
            std::thread::sleep(Duration::from_millis(5));
        }

        let throughput = progress.current_throughput();
        assert!(throughput.is_some());

        let tps = throughput.unwrap();
        assert!(tps > 0.0 && tps < 1000.0);
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
    fn test_configurable_thread_count() {
        let config = ScannerConfig {
            thread_count: 4,
            ..Default::default()
        };

        let scanner = ParallelScanner::new(config);
        // Scanner should use 4 threads
    }
}
```

### Acceptance Criteria
- [ ] Parallel scanner uses rayon for multi-threaded traversal
- [ ] Configurable thread pool size (auto-detect or manual)
- [ ] Advanced progress tracking with ETA and throughput
- [ ] Error collection from parallel operations
- [ ] 3-5x performance improvement over sequential scanning
- [ ] All unit tests pass with >85% coverage
- [ ] Scan 1M files in <2 minutes on SSD

---

## Phase 3.2: Parallel Deletion (Week 2)

### Objectives
Implement parallel deletion operations with proper synchronization and progress tracking.

### Implementation Tasks

#### 3.2.1: Parallel Execution Engine

```rust
// src/executor/parallel.rs
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct ParallelExecutor {
    config: ExecutionConfig,
    progress: Arc<AdvancedProgress>,
    results: Arc<Mutex<Vec<OperationResult>>>,
}

pub struct ExecutionConfig {
    pub mode: ExecutionMode,
    pub batch_size: usize,
    pub thread_count: usize,
    pub backup_dir: Option<PathBuf>,
    pub use_recycle_bin: bool,
    pub fail_fast: bool,
}

impl ParallelExecutor {
    pub fn new(config: ExecutionConfig) -> Self {
        // Configure thread pool
        let thread_count = if config.thread_count == 0 {
            num_cpus::get()
        } else {
            config.thread_count
        };

        rayon::ThreadPoolBuilder::new()
            .num_threads(thread_count)
            .build_global()
            .ok();

        Self {
            config,
            progress: Arc::new(AdvancedProgress::new()),
            results: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn execute(&self, plan: &CleanupPlan) -> Result<ExecutionResult, ExecutionError> {
        let start_time = Instant::now();

        // Filter entries to process
        let entries_to_process: Vec<_> = plan.entries.iter()
            .filter(|e| e.action == CleanupAction::Delete)
            .collect();

        self.progress.set_total(entries_to_process.len() as u64);

        // Process in batches for better error handling and progress updates
        let batches: Vec<_> = entries_to_process
            .chunks(self.config.batch_size)
            .collect();

        for batch in batches {
            // Check for abort signal
            if self.should_abort() {
                break;
            }

            // Process batch in parallel
            let batch_results: Vec<_> = batch
                .par_iter()
                .map(|entry| {
                    let result = self.execute_single(plan, entry);
                    self.progress.increment();
                    result
                })
                .collect();

            // Collect results
            {
                let mut results = self.results.lock().unwrap();
                results.extend(batch_results);
            }

            // Check for fail-fast
            if self.config.fail_fast {
                let results = self.results.lock().unwrap();
                if results.iter().any(|r| r.status == OperationStatus::Failed) {
                    break;
                }
            }
        }

        let duration = start_time.elapsed();
        let operations = self.results.lock().unwrap().clone();
        let summary = self.compute_summary(&operations, duration);

        Ok(ExecutionResult { operations, summary })
    }

    fn execute_single(&self, plan: &CleanupPlan, entry: &CleanupEntry) -> OperationResult {
        let full_path = plan.base_path.join(&entry.path);
        let timestamp = SystemTime::now();

        // Dry-run mode
        if self.config.mode == ExecutionMode::DryRun {
            return OperationResult {
                path: full_path,
                action: OperationAction::Delete,
                status: OperationStatus::DryRun,
                size_freed: Some(entry.size),
                error: None,
                timestamp,
            };
        }

        // Determine action
        let action = if self.config.use_recycle_bin {
            OperationAction::MoveToRecycleBin
        } else if let Some(ref backup_dir) = self.config.backup_dir {
            OperationAction::MoveToBackup
        } else {
            OperationAction::Delete
        };

        // Execute
        let result = match action {
            OperationAction::Delete => self.delete_path(&full_path),
            OperationAction::MoveToBackup => self.move_to_backup(&full_path, entry),
            OperationAction::MoveToRecycleBin => self.move_to_recycle_bin(&full_path),
            _ => Ok(()),
        };

        match result {
            Ok(()) => OperationResult {
                path: full_path,
                action,
                status: OperationStatus::Success,
                size_freed: Some(entry.size),
                error: None,
                timestamp,
            },
            Err(e) => OperationResult {
                path: full_path,
                action,
                status: OperationStatus::Failed,
                size_freed: None,
                error: Some(e.to_string()),
                timestamp,
            },
        }
    }

    fn delete_path(&self, path: &Path) -> Result<(), std::io::Error> {
        if path.is_dir() {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        }
    }

    fn move_to_backup(&self, path: &Path, entry: &CleanupEntry) -> Result<(), std::io::Error> {
        let backup_dir = self.config.backup_dir.as_ref().unwrap();
        let dest = backup_dir.join(&entry.path);

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::rename(path, &dest)?;
        Ok(())
    }

    fn move_to_recycle_bin(&self, path: &Path) -> Result<(), std::io::Error> {
        trash::delete(path).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
        })
    }

    fn should_abort(&self) -> bool {
        // Check for user abort signal (could be implemented with ctrl-c handler)
        false
    }

    fn compute_summary(&self, operations: &[OperationResult], duration: Duration) -> ExecutionSummary {
        let total_operations = operations.len();
        let successful = operations.iter().filter(|o| o.status == OperationStatus::Success || o.status == OperationStatus::DryRun).count();
        let failed = operations.iter().filter(|o| o.status == OperationStatus::Failed).count();
        let skipped = operations.iter().filter(|o| o.status == OperationStatus::Skipped).count();
        let space_freed = operations.iter()
            .filter_map(|o| o.size_freed)
            .sum();

        ExecutionSummary {
            total_operations,
            successful,
            failed,
            skipped,
            space_freed,
            duration,
        }
    }

    pub fn progress(&self) -> &AdvancedProgress {
        &self.progress
    }
}
```

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_execution_dry_run() {
        let temp = TempDir::new().unwrap();

        // Create 100 files
        for i in 0..100 {
            std::fs::write(temp.path().join(format!("file{}.txt", i)), "content").unwrap();
        }

        let plan = create_deletion_plan(&temp, 100);
        let config = ExecutionConfig {
            mode: ExecutionMode::DryRun,
            batch_size: 10,
            thread_count: 4,
            backup_dir: None,
            use_recycle_bin: false,
            fail_fast: false,
        };

        let executor = ParallelExecutor::new(config);
        let result = executor.execute(&plan).unwrap();

        assert_eq!(result.summary.total_operations, 100);
        assert!(temp.path().join("file0.txt").exists(), "Files should not be deleted in dry-run");
    }

    #[test]
    fn test_parallel_execution_batch_delete() {
        let temp = TempDir::new().unwrap();

        // Create 100 files
        for i in 0..100 {
            std::fs::write(temp.path().join(format!("file{}.txt", i)), "content").unwrap();
        }

        let plan = create_deletion_plan(&temp, 100);
        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            batch_size: 10,
            thread_count: 4,
            backup_dir: None,
            use_recycle_bin: false,
            fail_fast: false,
        };

        let executor = ParallelExecutor::new(config);
        let result = executor.execute(&plan).unwrap();

        assert_eq!(result.summary.successful, 100);
        assert!(!temp.path().join("file0.txt").exists(), "Files should be deleted");
    }

    #[test]
    fn test_parallel_execution_performance() {
        let temp = TempDir::new().unwrap();

        // Create 1000 files
        for i in 0..1000 {
            std::fs::write(temp.path().join(format!("file{}.txt", i)), "content").unwrap();
        }

        let plan = create_deletion_plan(&temp, 1000);
        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            batch_size: 50,
            thread_count: 4,
            backup_dir: None,
            use_recycle_bin: false,
            fail_fast: false,
        };

        let start = Instant::now();
        let executor = ParallelExecutor::new(config);
        let result = executor.execute(&plan).unwrap();
        let duration = start.elapsed();

        assert_eq!(result.summary.successful, 1000);
        println!("Deleted 1000 files in {:?}", duration);
        assert!(duration.as_secs() < 10, "Should delete 1000 files in <10s");
    }

    #[test]
    fn test_parallel_execution_with_backup() {
        let temp = TempDir::new().unwrap();
        let backup_dir = temp.path().join("backups");
        std::fs::create_dir(&backup_dir).unwrap();

        // Create nested structure
        std::fs::create_dir_all(temp.path().join("a/b/c")).unwrap();
        for i in 0..10 {
            std::fs::write(temp.path().join(format!("a/b/c/file{}.txt", i)), "content").unwrap();
        }

        let plan = create_deletion_plan(&temp, 10);
        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            batch_size: 5,
            thread_count: 2,
            backup_dir: Some(backup_dir.clone()),
            use_recycle_bin: false,
            fail_fast: false,
        };

        let executor = ParallelExecutor::new(config);
        let result = executor.execute(&plan).unwrap();

        assert_eq!(result.summary.successful, 10);
        assert!(backup_dir.join("a/b/c/file0.txt").exists(), "Files should be in backup");
    }

    #[test]
    fn test_parallel_execution_fail_fast() {
        let temp = TempDir::new().unwrap();

        // Create some files and one protected directory
        for i in 0..10 {
            std::fs::write(temp.path().join(format!("file{}.txt", i)), "content").unwrap();
        }

        let plan = create_mixed_plan(&temp); // Includes some invalid entries
        let config = ExecutionConfig {
            mode: ExecutionMode::Batch,
            batch_size: 5,
            thread_count: 2,
            backup_dir: None,
            use_recycle_bin: false,
            fail_fast: true,
        };

        let executor = ParallelExecutor::new(config);
        let result = executor.execute(&plan).unwrap();

        assert!(result.summary.failed > 0);
        assert!(result.operations.len() < plan.entries.len(), "Should stop early with fail-fast");
    }
}
```

### Acceptance Criteria
- [ ] Parallel executor uses rayon for multi-threaded deletion
- [ ] Batch processing for better error handling
- [ ] Thread-safe result collection
- [ ] Fail-fast mode works correctly in parallel context
- [ ] 2-4x performance improvement over sequential deletion
- [ ] All unit tests pass with >85% coverage
- [ ] Delete 100K files in <30 seconds

---

## Phase 3.3: Configuration File Support (Week 3)

### Objectives
Implement YAML-based configuration files for custom rules, default settings, and performance tuning.

### Implementation Tasks

#### 3.3.1: Configuration Schema

```rust
// src/config/mod.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MegamaidConfig {
    pub version: String,

    #[serde(default)]
    pub scanner: ScannerConfigSection,

    #[serde(default)]
    pub detection_rules: Vec<CustomDetectionRule>,

    #[serde(default)]
    pub execution: ExecutionConfigSection,

    #[serde(default)]
    pub performance: PerformanceConfigSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerConfigSection {
    #[serde(default = "default_max_depth")]
    pub max_depth: Option<usize>,

    #[serde(default = "default_skip_hidden")]
    pub skip_hidden: bool,

    #[serde(default)]
    pub follow_symlinks: bool,

    #[serde(default = "default_parallel")]
    pub parallel: bool,

    #[serde(default)]
    pub thread_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomDetectionRule {
    pub name: String,
    pub pattern: String,
    pub action: CleanupAction,
    pub reason: String,

    #[serde(default)]
    pub min_size: Option<String>,  // e.g., "100MB"

    #[serde(default)]
    pub max_age: Option<String>,  // e.g., "30d"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfigSection {
    #[serde(default = "default_mode")]
    pub default_mode: ExecutionMode,

    #[serde(default = "default_always_verify")]
    pub always_verify: bool,

    #[serde(default)]
    pub backup_dir: Option<PathBuf>,

    #[serde(default)]
    pub fail_fast: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfigSection {
    #[serde(default = "default_scan_batch_size")]
    pub scan_batch_size: usize,

    #[serde(default = "default_delete_batch_size")]
    pub delete_batch_size: usize,

    #[serde(default = "default_progress_update_interval")]
    pub progress_update_interval: u64,  // milliseconds
}

// Defaults
fn default_max_depth() -> Option<usize> { None }
fn default_skip_hidden() -> bool { true }
fn default_parallel() -> bool { true }
fn default_mode() -> ExecutionMode { ExecutionMode::Interactive }
fn default_always_verify() -> bool { true }
fn default_scan_batch_size() -> usize { 1000 }
fn default_delete_batch_size() -> usize { 100 }
fn default_progress_update_interval() -> u64 { 100 }

impl Default for MegamaidConfig {
    fn default() -> Self {
        Self {
            version: "0.1.0".to_string(),
            scanner: ScannerConfigSection::default(),
            detection_rules: Vec::new(),
            execution: ExecutionConfigSection::default(),
            performance: PerformanceConfigSection::default(),
        }
    }
}

impl MegamaidConfig {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(path, yaml)?;
        Ok(())
    }

    pub fn get_default_path() -> PathBuf {
        // ~/.config/megamaid/config.yaml on Linux
        // %APPDATA%\megamaid\config.yaml on Windows
        #[cfg(unix)]
        {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("megamaid")
                .join("config.yaml")
        }

        #[cfg(windows)]
        {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("megamaid")
                .join("config.yaml")
        }
    }

    pub fn parse_size(size_str: &str) -> Result<u64, ConfigError> {
        let size_str = size_str.trim().to_uppercase();

        let (num_str, multiplier) = if size_str.ends_with("KB") {
            (&size_str[..size_str.len()-2], 1024u64)
        } else if size_str.ends_with("MB") {
            (&size_str[..size_str.len()-2], 1024 * 1024)
        } else if size_str.ends_with("GB") {
            (&size_str[..size_str.len()-2], 1024 * 1024 * 1024)
        } else {
            (size_str.as_str(), 1)
        };

        let num: u64 = num_str.trim().parse()
            .map_err(|_| ConfigError::InvalidSize(size_str.to_string()))?;

        Ok(num * multiplier)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("Invalid size format: {0}")]
    InvalidSize(String),
}
```

#### 3.3.2: Custom Rule Implementation

```rust
// src/detector/custom_rules.rs
use glob::Pattern;

pub struct CustomRule {
    name: String,
    pattern: Pattern,
    action: CleanupAction,
    reason: String,
    min_size: Option<u64>,
}

impl CustomRule {
    pub fn from_config(rule: &CustomDetectionRule, base_path: &Path) -> Result<Self, DetectionError> {
        let pattern = Pattern::new(&rule.pattern)?;

        let min_size = rule.min_size.as_ref()
            .map(|s| MegamaidConfig::parse_size(s))
            .transpose()?;

        Ok(Self {
            name: rule.name.clone(),
            pattern,
            action: rule.action,
            reason: rule.reason.clone(),
            min_size,
        })
    }
}

impl DetectionRule for CustomRule {
    fn should_flag(&self, entry: &FileEntry) -> bool {
        // Check pattern match
        let matches_pattern = self.pattern.matches_path(&entry.path);

        if !matches_pattern {
            return false;
        }

        // Check size constraint if present
        if let Some(min_size) = self.min_size {
            if entry.size < min_size {
                return false;
            }
        }

        true
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn reason(&self) -> String {
        self.reason.clone()
    }

    fn suggested_action(&self) -> CleanupAction {
        self.action
    }
}
```

#### 3.3.3: CLI Integration

```rust
// src/cli/commands.rs (additions)

#[derive(Parser)]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long, value_name = "FILE", global = true)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands ...

    /// Initialize a configuration file
    InitConfig {
        /// Output path for config file
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// Show current configuration
    ShowConfig,
}

// src/cli/orchestrator.rs
fn run_init_config(output: Option<PathBuf>) -> Result<()> {
    let config_path = output.unwrap_or_else(|| MegamaidConfig::get_default_path());

    // Create parent directory if needed
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Generate default config
    let config = MegamaidConfig::default();
    config.save(&config_path)?;

    println!("✓ Configuration file created: {}", config_path.display());
    println!();
    println!("Edit this file to customize:");
    println!("  - Scanner settings");
    println!("  - Custom detection rules");
    println!("  - Execution defaults");
    println!("  - Performance tuning");

    Ok(())
}
```

### Example Configuration File

```yaml
version: "0.1.0"

# Scanner configuration
scanner:
  max_depth: 10
  skip_hidden: true
  follow_symlinks: false
  parallel: true
  thread_count: 0  # 0 = auto-detect

# Custom detection rules
detection_rules:
  # Python cache directories
  - name: "python_cache"
    pattern: "**/__pycache__/**"
    action: delete
    reason: "Python bytecode cache"

  # Python virtual environments
  - name: "python_venv"
    pattern: "**/venv/**"
    action: review
    reason: "Python virtual environment"

  # Large log files
  - name: "large_logs"
    pattern: "**/*.log"
    min_size: "100MB"
    action: review
    reason: "Large log file exceeding 100MB"

  # Temporary files
  - name: "temp_files"
    pattern: "**/.tmp/**"
    action: delete
    reason: "Temporary directory"

  # Build artifacts for various languages
  - name: "go_build"
    pattern: "**/go-build/**"
    action: delete
    reason: "Go build cache"

  - name: "maven_target"
    pattern: "**/target/**"
    action: delete
    reason: "Maven build artifacts"

# Execution defaults
execution:
  default_mode: interactive
  always_verify: true
  backup_dir: null
  fail_fast: false

# Performance tuning
performance:
  scan_batch_size: 1000
  delete_batch_size: 100
  progress_update_interval: 100
```

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_load_save() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("config.yaml");

        let config = MegamaidConfig::default();
        config.save(&config_path).unwrap();

        let loaded = MegamaidConfig::load(&config_path).unwrap();
        assert_eq!(loaded.version, config.version);
    }

    #[test]
    fn test_parse_size() {
        assert_eq!(MegamaidConfig::parse_size("100").unwrap(), 100);
        assert_eq!(MegamaidConfig::parse_size("100KB").unwrap(), 100 * 1024);
        assert_eq!(MegamaidConfig::parse_size("100MB").unwrap(), 100 * 1024 * 1024);
        assert_eq!(MegamaidConfig::parse_size("1GB").unwrap(), 1024 * 1024 * 1024);
    }

    #[test]
    fn test_custom_rule_pattern_match() {
        let rule_config = CustomDetectionRule {
            name: "test".to_string(),
            pattern: "**/__pycache__/**".to_string(),
            action: CleanupAction::Delete,
            reason: "test".to_string(),
            min_size: None,
            max_age: None,
        };

        let rule = CustomRule::from_config(&rule_config, Path::new("/")).unwrap();

        let entry1 = FileEntry {
            path: PathBuf::from("/project/__pycache__/module.pyc"),
            size: 1024,
            modified: "2024-01-01T00:00:00Z".to_string(),
            is_dir: false,
        };

        let entry2 = FileEntry {
            path: PathBuf::from("/project/src/main.py"),
            size: 1024,
            modified: "2024-01-01T00:00:00Z".to_string(),
            is_dir: false,
        };

        assert!(rule.should_flag(&entry1));
        assert!(!rule.should_flag(&entry2));
    }

    #[test]
    fn test_custom_rule_with_size_constraint() {
        let rule_config = CustomDetectionRule {
            name: "large_logs".to_string(),
            pattern: "**/*.log".to_string(),
            action: CleanupAction::Review,
            reason: "Large log file".to_string(),
            min_size: Some("100MB".to_string()),
            max_age: None,
        };

        let rule = CustomRule::from_config(&rule_config, Path::new("/")).unwrap();

        let small_log = FileEntry {
            path: PathBuf::from("/logs/app.log"),
            size: 10 * 1024 * 1024,  // 10MB
            modified: "2024-01-01T00:00:00Z".to_string(),
            is_dir: false,
        };

        let large_log = FileEntry {
            path: PathBuf::from("/logs/huge.log"),
            size: 200 * 1024 * 1024,  // 200MB
            modified: "2024-01-01T00:00:00Z".to_string(),
            is_dir: false,
        };

        assert!(!rule.should_flag(&small_log));
        assert!(rule.should_flag(&large_log));
    }

    #[test]
    fn test_config_with_custom_rules() {
        let yaml = r#"
version: "0.1.0"

detection_rules:
  - name: "test_rule"
    pattern: "**/*.tmp"
    action: delete
    reason: "Temporary file"
"#;

        let config: MegamaidConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.detection_rules.len(), 1);
        assert_eq!(config.detection_rules[0].name, "test_rule");
    }
}
```

### Acceptance Criteria
- [ ] YAML-based configuration file format
- [ ] Custom detection rules with glob patterns
- [ ] Size constraints for rules (min_size)
- [ ] Scanner, execution, and performance settings
- [ ] `init-config` command generates template
- [ ] `show-config` displays current settings
- [ ] Configuration loaded from default location or --config flag
- [ ] All unit tests pass with >85% coverage

---

## Phase 3.4: Integration & Testing (Week 4)

### Objectives
Comprehensive integration testing, performance validation, and documentation.

### Implementation Tasks

#### 3.4.1: Integration Tests

```rust
// tests/integration/milestone3_integration_tests.rs

#[test]
fn test_full_parallel_workflow() {
    let temp = TempDir::new().unwrap();
    create_large_test_structure(&temp, 10_000);

    // Parallel scan
    let scanner = ParallelScanner::new(ScannerConfig::default());
    let entries = scanner.scan(temp.path()).unwrap();
    assert!(entries.len() >= 10_000);

    // Detection
    let detector = DetectionEngine::new();
    let detections = detector.detect(&entries).unwrap();

    // Plan generation
    let plan = PlanGenerator::generate(temp.path(), &detections).unwrap();

    // Parallel execution
    let executor = ParallelExecutor::new(ExecutionConfig {
        mode: ExecutionMode::Batch,
        batch_size: 100,
        thread_count: 4,
        backup_dir: None,
        use_recycle_bin: false,
        fail_fast: false,
    });

    let result = executor.execute(&plan).unwrap();
    assert!(result.summary.successful > 0);
}

#[test]
fn test_config_driven_workflow() {
    let temp = TempDir::new().unwrap();

    // Create config with custom rules
    let config = create_test_config_with_custom_rules();
    let config_path = temp.path().join("config.yaml");
    config.save(&config_path).unwrap();

    // Load and use config
    let loaded_config = MegamaidConfig::load(&config_path).unwrap();

    // Apply custom rules
    let custom_rules = load_custom_rules(&loaded_config);
    assert!(custom_rules.len() > 0);
}

#[test]
fn test_parallel_performance_comparison() {
    let temp = TempDir::new().unwrap();
    create_large_test_structure(&temp, 5_000);

    // Sequential scan
    let seq_start = Instant::now();
    let seq_scanner = FileScanner::new(ScannerOptions::default());
    let seq_result = seq_scanner.scan(temp.path()).unwrap();
    let seq_duration = seq_start.elapsed();

    // Parallel scan
    let par_start = Instant::now();
    let par_scanner = ParallelScanner::new(ScannerConfig::default());
    let par_result = par_scanner.scan(temp.path()).unwrap();
    let par_duration = par_start.elapsed();

    println!("Sequential: {:?}, Parallel: {:?}", seq_duration, par_duration);
    println!("Speedup: {:.2}x", seq_duration.as_secs_f64() / par_duration.as_secs_f64());

    assert_eq!(seq_result.len(), par_result.len());
    assert!(par_duration < seq_duration, "Parallel should be faster");
}
```

#### 3.4.2: Performance Benchmarks

```rust
// tests/benchmarks/milestone3_benchmarks.rs

#[test]
#[ignore]
fn bench_parallel_scan_1m_files() {
    let temp = TempDir::new().unwrap();
    create_large_test_structure(&temp, 1_000_000);

    let start = Instant::now();
    let scanner = ParallelScanner::new(ScannerConfig {
        thread_count: num_cpus::get(),
        ..Default::default()
    });
    let result = scanner.scan(temp.path()).unwrap();
    let duration = start.elapsed();

    println!("Scanned {} files in {:?}", result.len(), duration);
    assert!(duration.as_secs() < 120, "Should scan 1M files in <2 minutes");
}

#[test]
#[ignore]
fn bench_parallel_delete_100k_files() {
    let temp = TempDir::new().unwrap();
    create_large_test_structure(&temp, 100_000);

    let plan = create_full_deletion_plan(&temp);

    let start = Instant::now();
    let executor = ParallelExecutor::new(ExecutionConfig {
        mode: ExecutionMode::Batch,
        batch_size: 100,
        thread_count: num_cpus::get(),
        backup_dir: None,
        use_recycle_bin: false,
        fail_fast: false,
    });
    let result = executor.execute(&plan).unwrap();
    let duration = start.elapsed();

    println!("Deleted {} files in {:?}", result.summary.successful, duration);
    assert!(duration.as_secs() < 30, "Should delete 100K files in <30 seconds");
}
```

### Documentation Updates

Update all documentation with Milestone 3 features:
- README.md: Add parallel operations, config file support
- QUICKSTART.md: Add config file examples
- CLAUDE.md: Update architecture with parallel processing
- Create CONFIGURATION.md guide

### Acceptance Criteria
- [ ] All integration tests pass
- [ ] Performance benchmarks meet targets (1M scan <2min, 100K delete <30s)
- [ ] Documentation updated
- [ ] All 150+ tests passing
- [ ] Code coverage >85% for all modules
- [ ] Zero clippy warnings

---

## Success Metrics

### Performance
- [ ] Parallel scanning 3-5x faster than sequential
- [ ] Parallel deletion 2-4x faster than sequential
- [ ] Scan 1M files in <2 minutes (SSD)
- [ ] Delete 100K files in <30 seconds
- [ ] Memory usage <200MB for 1M file scan

### Functional
- [ ] Parallel operations maintain correctness
- [ ] Progress tracking accurate in parallel context
- [ ] Configuration files load and apply correctly
- [ ] Custom rules work as expected
- [ ] Error handling robust in parallel context

### Quality
- [ ] All 150+ tests passing
- [ ] Code coverage >85% for all new modules
- [ ] Zero clippy warnings
- [ ] Documentation complete and accurate

---

## Dependencies

Add to Cargo.toml:
```toml
[dependencies]
# Parallelism
rayon = "1.8"
num_cpus = "1.16"

# Configuration
glob = "0.3"
dirs = "5.0"

# Existing dependencies...
```

---

## Risk Mitigation

### Identified Risks

1. **Filesystem Bottlenecks**
   - Risk: Parallel operations may not improve performance on slow drives
   - Mitigation: Make parallel execution optional, benchmark on different storage types
   - Fallback: Auto-detect drive type and adjust parallelism

2. **Lock Contention**
   - Risk: Shared state (progress, results) may become bottleneck
   - Mitigation: Use lock-free atomics where possible, minimize critical sections
   - Monitoring: Profile lock contention during performance tests

3. **Error Handling Complexity**
   - Risk: Parallel errors harder to track and report
   - Mitigation: Comprehensive error collection, detailed logging
   - Testing: Extensive error injection tests

4. **Configuration Complexity**
   - Risk: Users may create invalid or dangerous configurations
   - Mitigation: Strict validation, safe defaults, clear error messages
   - Documentation: Comprehensive config documentation with examples

---

## Deliverables Checklist

- [ ] Parallel scanner with rayon integration
- [ ] Advanced progress tracking with ETA
- [ ] Parallel executor with batch processing
- [ ] Configuration file support (YAML)
- [ ] Custom detection rules
- [ ] CLI commands: init-config, show-config
- [ ] 30+ new unit tests
- [ ] 5+ integration tests
- [ ] 2+ performance benchmarks
- [ ] Updated documentation (README, QUICKSTART, CLAUDE, new CONFIGURATION.md)

**Expected Test Count**: 150+ total (126 from M1&2 + 25+ new for M3)

---

## Next Steps (Milestone 4 Preview)

After Milestone 3 completion:
- Tauri GUI for visual plan review
- Interactive file browser
- Real-time disk usage visualization
- Drag-and-drop plan editing
- Visual progress indicators

---

## Timeline

**Week 1**: Parallel scanning with advanced progress tracking
**Week 2**: Parallel deletion operations
**Week 3**: Configuration file support and custom rules
**Week 4**: Integration testing, benchmarking, documentation

**Total Duration**: 3-4 weeks for complete Milestone 3 implementation
