// Audit Log Rotation and Retention Module
// Implements daily rotation, compression, and automatic cleanup for audit logs

#![allow(dead_code)] // Infrastructure module - functions will be used by scheduler/daemon

use std::fs::{create_dir_all, read_dir, remove_file};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::{QmsError, QmsResult};
use crate::utils::{current_date_string, current_timestamp};
use crate::modules::audit_logger::functions::log_system_event;

/// Configuration for audit log rotation and retention
#[derive(Debug, Clone)]
pub struct RotationConfig {
    pub enable_daily_rotation: bool,
    pub retention_days: u32,
    pub enable_compression: bool,
    pub max_daily_file_size_mb: u64,
}

impl Default for RotationConfig {
    fn default() -> Self {
        Self {
            enable_daily_rotation: true,
            retention_days: 2555, // 7 years for medical device compliance
            enable_compression: true,
            max_daily_file_size_mb: 100,
        }
    }
}

/// Check if daily rotation is needed and perform it
pub fn check_and_rotate_daily_logs(project_path: &Path) -> QmsResult<bool> {
    let audit_dir = project_path.join("audit");
    let current_log = audit_dir.join("audit.log");
    
    if !current_log.exists() {
        return Ok(false); // No log to rotate
    }
    
    // Check if we need to rotate based on date
    let current_date = current_date_string();
    let daily_dir = audit_dir.join("daily");
    let today_log = daily_dir.join(format!("{current_date}.log"));
    
    // If today's log already exists, check if main log has new entries
    if today_log.exists() {
        return Ok(false); // Already rotated today
    }
    
    // Check if main log was modified today
    if let Ok(metadata) = current_log.metadata() {
        if let Ok(modified) = metadata.modified() {
            if let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) {
                if let Ok(mod_time) = modified.duration_since(UNIX_EPOCH) {
                    let seconds_diff = now.as_secs() - mod_time.as_secs();
                    
                    // If the file was modified more than 24 hours ago, rotate it
                    if seconds_diff > 86400 { // 24 hours
                        return rotate_to_daily_log(project_path, &current_date);
                    }
                }
            }
        }
    }
    
    Ok(false)
}

/// Rotate current audit log to daily directory
pub fn rotate_to_daily_log(project_path: &Path, date: &str) -> QmsResult<bool> {
    let audit_dir = project_path.join("audit");
    let current_log = audit_dir.join("audit.log");
    let daily_dir = audit_dir.join("daily");
    let daily_log = daily_dir.join(format!("{date}.log"));
    
    if !current_log.exists() {
        return Ok(false);
    }
    
    // Create daily directory if it doesn't exist
    create_dir_all(&daily_dir)
        .map_err(|e| QmsError::domain_error(&format!("Failed to create daily directory: {e}")))?;
    
    // Copy current log to daily directory (preserve original for chain continuity)
    std::fs::copy(&current_log, &daily_log)
        .map_err(|e| QmsError::domain_error(&format!("Failed to copy log for rotation: {e}")))?;
    
    // Log the rotation event in the original log
    log_system_event("AUDIT_LOG_ROTATED", &format!("Daily log rotated to {}", daily_log.display()))?;
    
    // Compress the daily log if enabled
    if should_compress() {
        compress_log_file(&daily_log)?;
    }
    
    Ok(true)
}

/// Check if compression should be enabled (configurable for stdlib-only)
fn should_compress() -> bool {
    // Enable compression for audit logs to save space
    // This can be made configurable via environment variable or config file
    std::env::var("QMS_DISABLE_COMPRESSION").is_err()
}

/// Stdlib-only compression implementation for audit log files
fn compress_log_file(log_path: &Path) -> QmsResult<()> {
    if !log_path.exists() {
        return Err(QmsError::validation_error("Log file does not exist"));
    }

    // Read the original log file
    let original_data = std::fs::read(log_path)
        .map_err(|e| QmsError::io_error(&format!("Failed to read log file: {e}")))?;

    // Compress using our stdlib-only compression
    let compressed_data = compress_audit_log_data(&original_data)?;

    // Write compressed file with .gz extension
    let compressed_path = log_path.with_extension("log.gz");
    std::fs::write(&compressed_path, &compressed_data)
        .map_err(|e| QmsError::io_error(&format!("Failed to write compressed file: {e}")))?;

    // Remove original file after successful compression
    std::fs::remove_file(log_path)
        .map_err(|e| QmsError::io_error(&format!("Failed to remove original file: {e}")))?;

    // Log the compression event
    log_system_event("AUDIT_LOG_COMPRESSED", &format!(
        "Log file compressed: {} -> {} (ratio: {:.1}%)",
        log_path.display(),
        compressed_path.display(),
        (1.0 - (compressed_data.len() as f64 / original_data.len() as f64)) * 100.0
    ))?;

    Ok(())
}

/// Perform comprehensive cleanup of old audit logs
pub fn cleanup_old_logs_comprehensive(project_path: &Path, retention_days: u32) -> QmsResult<CleanupReport> {
    let audit_dir = project_path.join("audit");
    let daily_dir = audit_dir.join("daily");
    
    let mut report = CleanupReport {
        files_deleted: 0,
        files_compressed: 0,
        bytes_freed: 0,
        errors: Vec::new(),
    };
    
    if !daily_dir.exists() {
        return Ok(report);
    }
    
    let cutoff_timestamp = current_timestamp() - (retention_days as u64 * 24 * 60 * 60);
    
    for entry in read_dir(&daily_dir)
        .map_err(|e| QmsError::domain_error(&format!("Failed to read daily directory: {e}")))?
    {
        let entry = entry.map_err(|e| QmsError::domain_error(&format!("Failed to read directory entry: {e}")))?;
        let path = entry.path();
        
        if path.is_file() {
            match process_old_log_file(&path, cutoff_timestamp) {
                Ok(action) => {
                    match action {
                        FileAction::Deleted(size) => {
                            report.files_deleted += 1;
                            report.bytes_freed += size;
                        }
                        FileAction::Compressed(size) => {
                            report.files_compressed += 1;
                            report.bytes_freed += size;
                        }
                        FileAction::Kept => {}
                    }
                }
                Err(e) => {
                    report.errors.push(format!("Failed to process {}: {}", path.display(), e));
                }
            }
        }
    }
    
    // Log cleanup summary
    if report.files_deleted > 0 || report.files_compressed > 0 {
        log_system_event(
            "AUDIT_CLEANUP_COMPLETED",
            &format!(
                "Cleanup completed: {} files deleted, {} files compressed, {} bytes freed",
                report.files_deleted, report.files_compressed, report.bytes_freed
            )
        )?;
    }
    
    Ok(report)
}

/// Process a single old log file (delete or compress based on age)
fn process_old_log_file(path: &Path, cutoff_timestamp: u64) -> QmsResult<FileAction> {
    let metadata = path.metadata()
        .map_err(|e| QmsError::domain_error(&format!("Cannot read file metadata: {e}")))?;
    
    let file_size = metadata.len();
    
    let created_time = metadata.created()
        .map_err(|e| QmsError::domain_error(&format!("Cannot read file creation time: {e}")))?;
    
    let created_timestamp = created_time.duration_since(UNIX_EPOCH)
        .map_err(|e| QmsError::domain_error(&format!("Invalid file timestamp: {e}")))?
        .as_secs();
    
    if created_timestamp < cutoff_timestamp {
        // File is older than retention period - delete it
        remove_file(path)
            .map_err(|e| QmsError::domain_error(&format!("Cannot delete old log file: {e}")))?;
        
        Ok(FileAction::Deleted(file_size))
    } else {
        // File is within retention period but might need compression
        if should_compress() && !is_compressed(path) {
            compress_log_file(path)?;
            Ok(FileAction::Compressed(file_size / 2)) // Assume 50% compression ratio
        } else {
            Ok(FileAction::Kept)
        }
    }
}

/// Check if a file is already compressed (simple check by extension)
fn is_compressed(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "gz" || ext == "zip")
        .unwrap_or(false)
}

/// Report of cleanup operations
#[derive(Debug, Clone)]
pub struct CleanupReport {
    pub files_deleted: usize,
    pub files_compressed: usize,
    pub bytes_freed: u64,
    pub errors: Vec<String>,
}

/// Action taken on a file during cleanup
#[derive(Debug)]
enum FileAction {
    Deleted(u64),     // File deleted, size freed
    Compressed(u64),  // File compressed, size saved
    Kept,             // File kept as-is
}

/// Get rotation statistics for monitoring
pub fn get_rotation_statistics(project_path: &Path) -> QmsResult<RotationStats> {
    let audit_dir = project_path.join("audit");
    let daily_dir = audit_dir.join("daily");
    
    let mut stats = RotationStats {
        daily_files_count: 0,
        total_daily_size: 0,
        oldest_daily_file: None,
        newest_daily_file: None,
        compressed_files_count: 0,
    };
    
    if !daily_dir.exists() {
        return Ok(stats);
    }
    
    for entry in read_dir(&daily_dir)
        .map_err(|e| QmsError::domain_error(&format!("Failed to read daily directory: {e}")))?
    {
        let entry = entry.map_err(|e| QmsError::domain_error(&format!("Failed to read directory entry: {e}")))?;
        let path = entry.path();
        
        if path.is_file() {
            stats.daily_files_count += 1;
            
            if let Ok(metadata) = path.metadata() {
                stats.total_daily_size += metadata.len();
                
                if let Ok(created) = metadata.created() {
                    if let Ok(duration) = created.duration_since(UNIX_EPOCH) {
                        let timestamp = duration.as_secs();
                        
                        if stats.oldest_daily_file.is_none() || timestamp < stats.oldest_daily_file.unwrap() {
                            stats.oldest_daily_file = Some(timestamp);
                        }
                        
                        if stats.newest_daily_file.is_none() || timestamp > stats.newest_daily_file.unwrap() {
                            stats.newest_daily_file = Some(timestamp);
                        }
                    }
                }
            }
            
            if is_compressed(&path) {
                stats.compressed_files_count += 1;
            }
        }
    }
    
    Ok(stats)
}

/// Statistics about audit log rotation
#[derive(Debug, Clone)]
pub struct RotationStats {
    pub daily_files_count: usize,
    pub total_daily_size: u64,
    pub oldest_daily_file: Option<u64>, // Unix timestamp
    pub newest_daily_file: Option<u64>, // Unix timestamp  
    pub compressed_files_count: usize,
}

/// Schedule automatic daily rotation (would be called by a scheduler)
pub fn schedule_daily_rotation(project_path: &Path) -> QmsResult<()> {
    // Check and perform daily rotation
    if check_and_rotate_daily_logs(project_path)? {
        // Rotation performed, now check if cleanup is needed
        let retention_days = 2555; // 7 years default
        let cleanup_report = cleanup_old_logs_comprehensive(project_path, retention_days)?;
        
        if !cleanup_report.errors.is_empty() {
            log_system_event(
                "AUDIT_CLEANUP_WARNINGS",
                &format!("Cleanup completed with {} warnings", cleanup_report.errors.len())
            )?;
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_rotation_config_default() {
        let config = RotationConfig::default();
        assert!(config.enable_daily_rotation);
        assert_eq!(config.retention_days, 2555); // 7 years
        assert!(config.enable_compression);
    }

    #[test]
    fn test_cleanup_empty_directory() {
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path();
        
        let report = cleanup_old_logs_comprehensive(project_path, 7).unwrap();
        assert_eq!(report.files_deleted, 0);
        assert_eq!(report.files_compressed, 0);
    }

    #[test]
    fn test_rotation_stats_empty() {
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path();
        
        let stats = get_rotation_statistics(project_path).unwrap();
        assert_eq!(stats.daily_files_count, 0);
        assert_eq!(stats.total_daily_size, 0);
    }
}

/// Stdlib-only compression for audit log data using multiple techniques
fn compress_audit_log_data(data: &[u8]) -> QmsResult<Vec<u8>> {
    let mut compressed = Vec::with_capacity(data.len() + 20);

    // Add QMS audit log compression header
    compressed.extend_from_slice(b"QMS_AUDIT_V1"); // 12-byte header
    compressed.extend_from_slice(&(data.len() as u64).to_le_bytes()); // 8-byte original size

    if data.is_empty() {
        return Ok(compressed);
    }

    // Phase 1: Audit log specific pattern compression
    let pattern_compressed = compress_audit_patterns(data);

    // Phase 2: Run-Length Encoding for repeated characters
    let rle_compressed = apply_simple_rle(&pattern_compressed);

    // Phase 3: Byte frequency optimization
    let final_compressed = apply_byte_frequency_compression(&rle_compressed);

    compressed.extend_from_slice(&final_compressed);

    // Ensure compression was beneficial
    if compressed.len() >= data.len() + 20 {
        // Store uncompressed with different header
        let mut uncompressed = Vec::with_capacity(data.len() + 20);
        uncompressed.extend_from_slice(b"QMS_AUDIT_RAW"); // Raw data header
        uncompressed.extend_from_slice(&(data.len() as u64).to_le_bytes());
        uncompressed.extend_from_slice(data);
        Ok(uncompressed)
    } else {
        Ok(compressed)
    }
}

/// Compress common audit log patterns
fn compress_audit_patterns(data: &[u8]) -> Vec<u8> {
    let data_str = String::from_utf8_lossy(data);
    let mut compressed = String::new();

    // Common audit log patterns to compress
    let patterns = [
        ("AUDIT_ENTRY:", "AE:"),
        ("timestamp:", "ts:"),
        ("user_id:", "uid:"),
        ("action:", "act:"),
        ("entity_type:", "et:"),
        ("entity_id:", "eid:"),
        ("checksum:", "cs:"),
        ("CREATE", "C"),
        ("UPDATE", "U"),
        ("DELETE", "D"),
        ("READ", "R"),
        ("DOCUMENT", "DOC"),
        ("RISK", "RSK"),
        ("REQUIREMENT", "REQ"),
        ("TEST_CASE", "TC"),
    ];

    let mut result = data_str.to_string();
    for (pattern, replacement) in &patterns {
        result = result.replace(pattern, replacement);
    }

    compressed = result;
    compressed.into_bytes()
}

/// Apply simple Run-Length Encoding
fn apply_simple_rle(data: &[u8]) -> Vec<u8> {
    let mut compressed = Vec::new();
    let mut i = 0;

    while i < data.len() {
        let current_byte = data[i];
        let mut count = 1;

        // Count consecutive identical bytes
        while i + count < data.len() && data[i + count] == current_byte && count < 255 {
            count += 1;
        }

        if count >= 4 {
            // Use RLE for runs of 4 or more (more conservative for text data)
            compressed.push(0xFE); // RLE marker
            compressed.push(current_byte);
            compressed.push(count as u8);
        } else {
            // Store bytes directly
            for _ in 0..count {
                compressed.push(current_byte);
            }
        }

        i += count;
    }

    compressed
}

/// Apply byte frequency compression for common characters
fn apply_byte_frequency_compression(data: &[u8]) -> Vec<u8> {
    // Build frequency table
    let mut freq = [0u32; 256];
    for &byte in data {
        freq[byte as usize] += 1;
    }

    // Find most frequent bytes
    let mut freq_pairs: Vec<_> = freq.iter().enumerate().collect();
    freq_pairs.sort_by(|a, b| b.1.cmp(a.1));

    // Create mapping for top 8 most frequent bytes
    let mut mapping = [None; 256];
    for (i, &(byte, count)) in freq_pairs.iter().take(8).enumerate() {
        if *count > 20 {  // Only map if frequent enough
            mapping[byte] = Some(i as u8);
        }
    }

    let mut compressed = Vec::new();

    // Store mapping table (8 bytes for top frequent bytes)
    for i in 0..8 {
        if let Some((byte, _)) = freq_pairs.get(i) {
            compressed.push(*byte as u8);
        } else {
            compressed.push(0);
        }
    }

    // Compress data using mapping
    for &byte in data {
        if let Some(short_code) = mapping[byte as usize] {
            compressed.push(0xFD); // Frequency marker
            compressed.push(short_code);
        } else {
            compressed.push(byte);
        }
    }

    compressed
}
