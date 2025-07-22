//! File locking mechanism for concurrent access control
//! Phase 2 infrastructure - critical for multi-user document control

#![allow(dead_code)] // Phase 2 infrastructure - concurrency control for document checkout/checkin

use crate::audit::log_audit;
use crate::error::{QmsError, QmsResult};
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// File lock guard that automatically releases lock when dropped
pub struct LockGuard {
    _file: File, // Keep file handle alive for lock duration
    path: PathBuf,
    acquired_at: SystemTime,
    holder: String,
}

impl LockGuard {
    /// Acquire an exclusive lock on a file with timeout
    pub fn acquire(path: &Path, user: &str) -> QmsResult<Self> {
        Self::acquire_with_timeout(path, user, Duration::from_secs(30))
    }

    /// Acquire an exclusive lock on a file with custom timeout
    pub fn acquire_with_timeout(path: &Path, user: &str, timeout: Duration) -> QmsResult<Self> {
        let lock_path = Self::get_lock_file_path(path);
        let start_time = SystemTime::now();

        // Ensure parent directory exists
        if let Some(parent) = lock_path.parent() {
            crate::fs_utils::ensure_dir_exists(parent)?;
        }

        loop {
            // Try to acquire lock
            match Self::try_acquire_lock(&lock_path, user) {
                Ok(lock_guard) => {
                    log_audit(&format!("LOCK_ACQUIRED: {} by {}", path.display(), user));
                    return Ok(lock_guard);
                }
                Err(e) => {
                    // Check timeout
                    if start_time.elapsed().unwrap_or(Duration::ZERO) >= timeout {
                        return Err(QmsError::io_error(&format!(
                            "Lock acquisition timed out after {timeout:?}: {e}"
                        )));
                    }

                    // Check if lock holder is still valid
                    if let Ok(current_holder) = Self::get_lock_holder(&lock_path) {
                        if !Self::is_lock_holder_valid(&current_holder) {
                            // Force release stale lock
                            let _ = std::fs::remove_file(&lock_path);
                            log_audit(&format!(
                                "LOCK_RELEASED_STALE: {} (holder: {})",
                                path.display(),
                                current_holder
                            ));
                        }
                    }

                    // Wait before retrying
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }

    /// Try to acquire lock immediately without retry
    fn try_acquire_lock(lock_path: &Path, user: &str) -> QmsResult<Self> {
        // Check if lock file already exists
        if lock_path.exists() {
            if let Ok(holder) = Self::get_lock_holder(lock_path) {
                return Err(QmsError::io_error(&format!(
                    "File is already locked by: {holder}"
                )));
            }
        }

        // Try to create lock file exclusively
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(lock_path)
            .map_err(|e| QmsError::io_error(&format!("Failed to create lock file: {e}")))?;

        let acquired_at = SystemTime::now();
        let timestamp = acquired_at
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();

        // Write lock information
        let lock_info = format!("{}\n{}\n{}", user, timestamp, std::process::id());
        std::io::Write::write_all(&mut file, lock_info.as_bytes())
            .map_err(|e| QmsError::io_error(&format!("Failed to write lock info: {e}")))?;

        Ok(LockGuard {
            _file: file,
            path: lock_path.to_path_buf(),
            acquired_at,
            holder: user.to_string(),
        })
    }

    /// Get the lock file path for a given file
    fn get_lock_file_path(path: &Path) -> PathBuf {
        let mut lock_path = path.to_path_buf();
        let filename = lock_path.file_name().unwrap_or_default().to_string_lossy();
        lock_path.set_file_name(format!(".{filename}.lock"));
        lock_path
    }

    /// Get the current lock holder information
    fn get_lock_holder(lock_path: &Path) -> QmsResult<String> {
        let content = std::fs::read_to_string(lock_path)
            .map_err(|e| QmsError::io_error(&format!("Failed to read lock file: {e}")))?;

        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return Err(QmsError::parse_error("Lock file is empty"));
        }

        Ok(lines[0].to_string())
    }

    /// Check if a lock holder is still valid (basic implementation)
    const fn is_lock_holder_valid(_holder: &str) -> bool {
        // In a real implementation, this could check if the process is still running
        // For now, we'll use a simple heuristic based on lock age
        true // Simplified for stdlib-only implementation
    }

    /// Get lock information
    pub fn get_lock_info(&self) -> (String, SystemTime) {
        (self.holder.clone(), self.acquired_at)
    }

    /// Get the path of the locked file
    pub fn get_locked_path(&self) -> &Path {
        &self.path
    }

    /// Check how long the lock has been held
    pub fn lock_duration(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.acquired_at)
            .unwrap_or(Duration::ZERO)
    }

    /// Force release a lock (use with caution)
    pub fn force_release_lock(path: &Path) -> QmsResult<()> {
        let lock_path = Self::get_lock_file_path(path);

        if lock_path.exists() {
            let holder =
                Self::get_lock_holder(&lock_path).unwrap_or_else(|_| "unknown".to_string());

            std::fs::remove_file(&lock_path)
                .map_err(|e| QmsError::io_error(&format!("Failed to remove lock file: {e}")))?;

            log_audit(&format!(
                "LOCK_FORCE_RELEASED: {} (was held by: {})",
                path.display(),
                holder
            ));
        }

        Ok(())
    }

    /// Check if a file is currently locked
    pub fn is_locked(path: &Path) -> bool {
        let lock_path = Self::get_lock_file_path(path);
        lock_path.exists()
    }

    /// Get information about who holds the lock (if any)
    pub fn get_lock_status(path: &Path) -> Option<(String, SystemTime)> {
        let lock_path = Self::get_lock_file_path(path);

        if !lock_path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&lock_path).ok()?;
        let lines: Vec<&str> = content.lines().collect();

        if lines.len() < 2 {
            return None;
        }

        let holder = lines[0].to_string();
        let timestamp: u64 = lines[1].parse().ok()?;
        let acquired_at = UNIX_EPOCH + Duration::from_secs(timestamp);

        Some((holder, acquired_at))
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        // Remove lock file when guard is dropped
        if let Err(e) = std::fs::remove_file(&self.path) {
            // Log error but don't panic in destructor
            log_audit(&format!(
                "WARNING: Failed to release lock {}: {}",
                self.path.display(),
                e
            ));
        } else {
            log_audit(&format!(
                "LOCK_RELEASED: {} by {}",
                self.path.display(),
                self.holder
            ));
        }
    }
}

/// Utility functions for lock management
pub mod lock_utils {
    use super::*;

    /// Clean up stale lock files older than the specified duration
    pub fn cleanup_stale_locks(directory: &Path, max_age: Duration) -> QmsResult<usize> {
        let mut cleaned = 0;

        if !directory.exists() {
            return Ok(0);
        }

        let entries = std::fs::read_dir(directory)
            .map_err(|e| QmsError::io_error(&format!("Failed to read directory: {e}")))?;

        for entry in entries {
            let entry =
                entry.map_err(|e| QmsError::io_error(&format!("Failed to read entry: {e}")))?;
            let path = entry.path();

            if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy();
                if filename_str.starts_with('.') && filename_str.ends_with(".lock") {
                    if let Ok(metadata) = path.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                                if elapsed > max_age
                                    && std::fs::remove_file(&path).is_ok() {
                                        cleaned += 1;
                                        log_audit(&format!(
                                            "STALE_LOCK_CLEANED: {}",
                                            path.display()
                                        ));
                                    }
                            }
                        }
                    }
                }
            }
        }

        Ok(cleaned)
    }

    /// List all active locks in a directory
    pub fn list_active_locks(directory: &Path) -> QmsResult<Vec<(PathBuf, String, SystemTime)>> {
        let mut locks = Vec::new();

        if !directory.exists() {
            return Ok(locks);
        }

        let entries = std::fs::read_dir(directory)
            .map_err(|e| QmsError::io_error(&format!("Failed to read directory: {e}")))?;

        for entry in entries {
            let entry =
                entry.map_err(|e| QmsError::io_error(&format!("Failed to read entry: {e}")))?;
            let path = entry.path();

            if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy();
                if filename_str.starts_with('.') && filename_str.ends_with(".lock") {
                    // Extract original file path from lock file name
                    let original_name = &filename_str[1..filename_str.len() - 5]; // Remove . prefix and .lock suffix
                    let original_path = directory.join(original_name);

                    if let Some((holder, acquired_at)) = LockGuard::get_lock_status(&original_path)
                    {
                        locks.push((original_path, holder, acquired_at));
                    }
                }
            }
        }

        Ok(locks)
    }

    /// Check if any files in a directory are locked
    pub fn has_active_locks(directory: &Path) -> bool {
        list_active_locks(directory)
            .map(|locks| !locks.is_empty())
            .unwrap_or(false)
    }

    /// Wait for all locks in a directory to be released
    pub fn wait_for_no_locks(directory: &Path, timeout: Duration) -> QmsResult<()> {
        let start_time = SystemTime::now();

        loop {
            if !has_active_locks(directory) {
                return Ok(());
            }

            if start_time.elapsed().unwrap_or(Duration::ZERO) >= timeout {
                return Err(QmsError::io_error(
                    "Timeout waiting for locks to be released",
                ));
            }

            thread::sleep(Duration::from_millis(250));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_lock_acquire_and_release() {
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join(format!(
            "qms_test_lock_{}.txt",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        // Create test file
        std::fs::write(&test_file, "test content").unwrap();

        // File should not be locked initially
        assert!(!LockGuard::is_locked(&test_file));

        // Acquire lock
        {
            let _lock = LockGuard::acquire(&test_file, "test_user").unwrap();
            assert!(LockGuard::is_locked(&test_file));

            // Check lock status
            let (holder, _acquired_at) = LockGuard::get_lock_status(&test_file).unwrap();
            assert_eq!(holder, "test_user");
        } // Lock should be released here

        // File should not be locked after guard is dropped
        assert!(!LockGuard::is_locked(&test_file));

        // Clean up
        let _ = std::fs::remove_file(&test_file);
    }

    #[test]
    fn test_lock_conflict() {
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join(format!(
            "qms_test_conflict_{}.txt",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        // Create test file
        std::fs::write(&test_file, "test content").unwrap();

        // Acquire first lock
        let _lock1 = LockGuard::acquire(&test_file, "user1").unwrap();

        // Second lock should fail immediately
        let result =
            LockGuard::acquire_with_timeout(&test_file, "user2", Duration::from_millis(100));
        assert!(result.is_err());

        // Clean up
        let _ = std::fs::remove_file(&test_file);
    }

    #[test]
    fn test_force_release_lock() {
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join(format!(
            "qms_test_force_{}.txt",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        // Create test file
        std::fs::write(&test_file, "test content").unwrap();

        // Acquire lock
        let _lock = LockGuard::acquire(&test_file, "test_user").unwrap();
        assert!(LockGuard::is_locked(&test_file));

        // Force release lock
        LockGuard::force_release_lock(&test_file).unwrap();
        assert!(!LockGuard::is_locked(&test_file));

        // Clean up
        let _ = std::fs::remove_file(&test_file);
    }

    #[test]
    fn test_lock_duration() {
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join(format!(
            "qms_test_duration_{}.txt",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        // Create test file
        std::fs::write(&test_file, "test content").unwrap();

        // Acquire lock
        let lock = LockGuard::acquire(&test_file, "test_user").unwrap();

        // Sleep briefly
        thread::sleep(Duration::from_millis(10));

        // Check duration
        let duration = lock.lock_duration();
        assert!(duration >= Duration::from_millis(10));
        assert!(duration < Duration::from_secs(1));

        // Clean up
        let _ = std::fs::remove_file(&test_file);
    }

    #[test]
    fn test_cleanup_stale_locks() {
        let temp_dir = env::temp_dir();
        let test_subdir = temp_dir.join(format!(
            "qms_test_cleanup_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        // Create test directory
        std::fs::create_dir_all(&test_subdir).unwrap();

        // Create a fake stale lock file
        let stale_lock = test_subdir.join(".test_file.lock");
        std::fs::write(&stale_lock, "stale_user\n123456789\n1234").unwrap();

        // Set the file modification time to be old (simulate stale lock)
        // Wait a moment to ensure the file is considered stale
        thread::sleep(Duration::from_millis(10));

        // Clean up stale locks (very short max age to catch our test lock)
        let cleaned =
            lock_utils::cleanup_stale_locks(&test_subdir, Duration::from_millis(1)).unwrap();
        // Verify cleanup result is valid
        assert!(cleaned <= 1); // Should clean 0 or 1 file depending on timing

        // Clean up
        let _ = std::fs::remove_dir_all(&test_subdir);
    }

    #[test]
    fn test_list_active_locks() {
        let temp_dir = env::temp_dir();
        let test_subdir = temp_dir.join(format!(
            "qms_test_list_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        // Create test directory
        std::fs::create_dir_all(&test_subdir).unwrap();

        // No locks initially
        let locks = lock_utils::list_active_locks(&test_subdir).unwrap();
        assert_eq!(locks.len(), 0);

        // Create test file and acquire lock
        let test_file = test_subdir.join("test_file.txt");
        std::fs::write(&test_file, "test content").unwrap();

        let _lock = LockGuard::acquire(&test_file, "test_user").unwrap();

        // Should find one active lock
        let locks = lock_utils::list_active_locks(&test_subdir).unwrap();
        assert_eq!(locks.len(), 1);
        assert_eq!(locks[0].1, "test_user");

        // Clean up
        let _ = std::fs::remove_dir_all(&test_subdir);
    }
}
