//! File system utilities for QMS data management
//! Phase 2 infrastructure - atomic operations, permissions, backup utilities

#![allow(dead_code)] // Phase 2 infrastructure - file system utilities for document control

use crate::audit::log_audit;
use crate::error::{QmsError, QmsResult};
use crate::utils::current_timestamp;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Atomic file write operation for data integrity
pub fn atomic_write(path: &Path, content: &str) -> QmsResult<()> {
    // Create temporary file path with simple numeric suffix for Windows compatibility
    let mut temp_path = path.with_extension("tmp");
    let mut counter = 0;
    
    // Find an available temp filename
    while temp_path.exists() && counter < 1000 {
        temp_path = path.with_extension(format!("tmp{counter}"));
        counter += 1;
    }
    
    if counter >= 1000 {
        return Err(QmsError::io_error("Cannot create unique temporary file"));
    }

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        ensure_dir_exists(parent)?;
    }

    // Write content to temporary file
    {
        let mut temp_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&temp_path)
            .map_err(|e| QmsError::io_error(&format!("Failed to create temp file: {e}")))?;

        temp_file
            .write_all(content.as_bytes())
            .map_err(|e| QmsError::io_error(&format!("Failed to write to temp file: {e}")))?;

        temp_file
            .flush()
            .map_err(|e| QmsError::io_error(&format!("Failed to flush temp file: {e}")))?;
    }

    // Calculate checksum of written content
    let checksum = calculate_checksum(&temp_path)?;

    // Atomically rename temp file to final path
    fs::rename(&temp_path, path)
        .map_err(|e| QmsError::io_error(&format!("Failed to rename temp file: {e}")))?;

    // Log the file operation for audit trail
    log_audit(&format!(
        "FILE_WRITE: {} (checksum: {})",
        path.display(),
        checksum
    ));

    Ok(())
}

/// Ensure directory exists, creating it and all parent directories if necessary
pub fn ensure_dir_exists(path: &Path) -> QmsResult<()> {
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| {
            QmsError::io_error(&format!(
                "Failed to create directory {}: {}",
                path.display(),
                e
            ))
        })?;

        // Set appropriate permissions on Unix systems
        #[cfg(unix)]
        set_unix_directory_permissions(path)?;

        log_audit(&format!("DIRECTORY_CREATED: {}", path.display()));
    }
    Ok(())
}

/// Calculate checksum of a file for integrity verification
pub fn calculate_checksum(path: &Path) -> QmsResult<String> {
    let content = fs::read_to_string(path)
        .map_err(|e| QmsError::io_error(&format!("Failed to read file for checksum: {e}")))?;

    // Use simple hash-based checksum (stdlib-only implementation)
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    let checksum = format!("{:016x}", hasher.finish());

    Ok(checksum)
}

/// Create backup of a file before modification
pub fn backup_file(path: &Path) -> QmsResult<PathBuf> {
    if !path.exists() {
        return Err(QmsError::not_found(&format!(
            "File to backup does not exist: {}",
            path.display()
        )));
    }

    let timestamp = current_timestamp();
    let backup_name = format!(
        "{}.backup.{}",
        path.file_name().unwrap_or_default().to_string_lossy(),
        timestamp
    );

    let backup_path = path.with_file_name(backup_name);

    fs::copy(path, &backup_path)
        .map_err(|e| QmsError::io_error(&format!("Failed to create backup: {e}")))?;

    log_audit(&format!(
        "FILE_BACKUP: {} -> {}",
        path.display(),
        backup_path.display()
    ));

    Ok(backup_path)
}

/// Set file permissions for Unix systems (medical device compliance)
#[cfg(unix)]
pub fn set_unix_file_permissions(path: &Path) -> QmsResult<()> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = fs::metadata(path)
        .map_err(|e| QmsError::io_error(&format!("Failed to read file metadata: {}", e)))?;

    let mut permissions = metadata.permissions();

    // Set read/write for owner, read for group, no access for others (0640)
    permissions.set_mode(0o640);

    fs::set_permissions(path, permissions)
        .map_err(|e| QmsError::io_error(&format!("Failed to set file permissions: {}", e)))?;

    Ok(())
}

/// Set directory permissions for Unix systems (medical device compliance)
#[cfg(unix)]
pub fn set_unix_directory_permissions(path: &Path) -> QmsResult<()> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = fs::metadata(path)
        .map_err(|e| QmsError::io_error(&format!("Failed to read directory metadata: {}", e)))?;

    let mut permissions = metadata.permissions();

    // Set read/write/execute for owner, read/execute for group, no access for others (0750)
    permissions.set_mode(0o750);

    fs::set_permissions(path, permissions)
        .map_err(|e| QmsError::io_error(&format!("Failed to set directory permissions: {}", e)))?;

    Ok(())
}

/// Set file permissions for Windows systems (basic implementation)
#[cfg(windows)]
pub fn set_windows_file_permissions(path: &Path) -> QmsResult<()> {
    // Windows permission handling is more complex and typically requires
    // Windows API calls. For a basic implementation, we'll ensure the file
    // is not read-only to allow modifications.
    let metadata = fs::metadata(path)
        .map_err(|e| QmsError::io_error(&format!("Failed to read file metadata: {e}")))?;

    let mut permissions = metadata.permissions();

    // Ensure file is not read-only
    // Note: Windows permission model is different from Unix
    // Using readonly=false is acceptable for medical device file access
    #[allow(clippy::permissions_set_readonly_false)]
    permissions.set_readonly(false);

    fs::set_permissions(path, permissions)
        .map_err(|e| QmsError::io_error(&format!("Failed to set file permissions: {e}")))?;

    Ok(())
}

/// Cross-platform file permission helper
pub fn set_secure_file_permissions(path: &Path) -> QmsResult<()> {
    #[cfg(unix)]
    return set_unix_file_permissions(path);

    #[cfg(windows)]
    return set_windows_file_permissions(path);

    #[cfg(not(any(unix, windows)))]
    {
        log_audit(&format!(
            "WARNING: Secure file permissions not implemented for this platform: {}",
            path.display()
        ));
        Ok(())
    }
}

/// Cross-platform directory permission helper
pub const fn set_secure_directory_permissions(_path: &Path) -> QmsResult<()> {
    #[cfg(unix)]
    return set_unix_directory_permissions(_path);

    #[cfg(windows)]
    {
        // Windows directories don't need special permission handling in basic implementation
        Ok(())
    }

    #[cfg(not(any(unix, windows)))]
    {
        log_audit(&format!(
            "WARNING: Secure directory permissions not implemented for this platform: {}",
            path.display()
        ));
        Ok(())
    }
}

/// Check if a file is locked (basic check by attempting to open for writing)
pub fn is_file_locked(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }

    // Try to open file for writing to check if it's locked
    OpenOptions::new().write(true).open(path).is_err()
}

/// Get file size in bytes
pub fn get_file_size(path: &Path) -> QmsResult<u64> {
    let metadata = fs::metadata(path)
        .map_err(|e| QmsError::io_error(&format!("Failed to read file metadata: {e}")))?;

    Ok(metadata.len())
}

/// Check if path is a valid filename (no path traversal, invalid characters)
pub fn validate_filename(filename: &str) -> bool {
    if filename.is_empty() || filename.len() > 255 {
        return false;
    }

    // Check for path traversal attempts
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return false;
    }

    // Check for invalid characters (basic set)
    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '\0'];
    if filename
        .chars()
        .any(|c| invalid_chars.contains(&c) || c.is_control())
    {
        return false;
    }

    // Check for reserved names on Windows
    let reserved_names = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];

    let upper_filename = filename.to_uppercase();
    if reserved_names
        .iter()
        .any(|&name| upper_filename == name || upper_filename.starts_with(&format!("{name}.")))
    {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_ensure_dir_exists() {
        let temp_dir = env::temp_dir();
        let test_dir = temp_dir.join(format!("qms_test_dir_{}", current_timestamp()));

        // Directory should not exist initially
        assert!(!test_dir.exists());

        // Create directory
        assert!(ensure_dir_exists(&test_dir).is_ok());
        assert!(test_dir.exists());

        // Should not error if directory already exists
        assert!(ensure_dir_exists(&test_dir).is_ok());

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_atomic_write() {
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join(format!("qms_test_atomic_{}.txt", current_timestamp()));

        let content = "Test content for atomic write";

        // Atomic write should succeed
        assert!(atomic_write(&test_file, content).is_ok());
        assert!(test_file.exists());

        // Content should match
        let read_content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(read_content, content);

        // Clean up
        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_calculate_checksum() {
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join(format!("qms_test_checksum_{}.txt", current_timestamp()));

        let content = "Test content for checksum";
        fs::write(&test_file, content).unwrap();

        // Calculate checksum
        let checksum = calculate_checksum(&test_file).unwrap();
        assert!(!checksum.is_empty());
        assert_eq!(checksum.len(), 16); // 64-bit hash as hex string

        // Same content should produce same checksum
        let checksum2 = calculate_checksum(&test_file).unwrap();
        assert_eq!(checksum, checksum2);

        // Clean up
        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_backup_file() {
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join(format!("qms_test_backup_{}.txt", current_timestamp()));

        let content = "Test content for backup";
        fs::write(&test_file, content).unwrap();

        // Create backup
        let backup_path = backup_file(&test_file).unwrap();
        assert!(backup_path.exists());
        assert!(backup_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains("backup"));

        // Backup should have same content
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backup_content, content);

        // Clean up
        let _ = fs::remove_file(&test_file);
        let _ = fs::remove_file(&backup_path);
    }

    #[test]
    fn test_validate_filename() {
        // Valid filenames
        assert!(validate_filename("test.txt"));
        assert!(validate_filename("document_v1.2.pdf"));
        assert!(validate_filename("file-name_123.json"));

        // Invalid filenames
        assert!(!validate_filename(""));
        assert!(!validate_filename("../test.txt"));
        assert!(!validate_filename("test/file.txt"));
        assert!(!validate_filename("test\\file.txt"));
        assert!(!validate_filename("test<file>.txt"));
        assert!(!validate_filename("CON.txt"));
        assert!(!validate_filename("NUL"));
        assert!(!validate_filename(&"x".repeat(256))); // Too long
    }

    #[test]
    fn test_get_file_size() {
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join(format!("qms_test_size_{}.txt", current_timestamp()));

        let content = "Test content";
        fs::write(&test_file, content).unwrap();

        let size = get_file_size(&test_file).unwrap();
        assert_eq!(size, content.len() as u64);

        // Clean up
        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_is_file_locked() {
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join(format!("qms_test_lock_{}.txt", current_timestamp()));

        // Non-existent file should not be locked
        assert!(!is_file_locked(&test_file));

        // Create file
        fs::write(&test_file, "test").unwrap();

        // File should not be locked initially
        assert!(!is_file_locked(&test_file));

        // Clean up
        let _ = fs::remove_file(&test_file);
    }
}
