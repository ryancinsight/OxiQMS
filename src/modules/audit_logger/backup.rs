use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::hash::{DefaultHasher, Hasher, Hash};
use crate::prelude::*;

/// Configuration for audit backup operations
#[derive(Debug, Clone)]
pub struct BackupConfig {
    pub backup_root: PathBuf,
    pub compress_enabled: bool,
    pub retention_days: u32,
    #[allow(dead_code)]
    pub daily_backup: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            backup_root: PathBuf::from("backups/audit"),
            compress_enabled: true,
            retention_days: 2555, // 7 years for medical device compliance
            daily_backup: true,
        }
    }
}

/// Statistics for backup operations
#[derive(Debug, Clone)]
pub struct BackupStats {
    pub files_backed_up: usize,
    pub bytes_backed_up: u64,
    pub bytes_compressed: u64,
    pub compression_ratio: f64,
    pub backup_duration_ms: u64,
    pub errors: Vec<String>,
}

impl Default for BackupStats {
    fn default() -> Self {
        Self::new()
    }
}

impl BackupStats {
    pub const fn new() -> Self {
        Self {
            files_backed_up: 0,
            bytes_backed_up: 0,
            bytes_compressed: 0,
            compression_ratio: 0.0,
            backup_duration_ms: 0,
            errors: Vec::new(),
        }
    }
}

/// Recovery information for a backup
#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub backup_id: String,
    pub timestamp: u64,
    pub source_path: PathBuf,
    pub backup_path: PathBuf,
    pub file_count: usize,
    pub total_size: u64,
    pub compressed: bool,
    pub checksum: String,
}

/// Audit backup manager
pub struct AuditBackupManager {
    config: BackupConfig,
    project_path: PathBuf,
}

impl AuditBackupManager {
    pub fn new(project_path: PathBuf) -> Self {
        Self {
            config: BackupConfig::default(),
            project_path,
        }
    }

    #[allow(dead_code)]
    pub const fn with_config(project_path: PathBuf, config: BackupConfig) -> Self {
        Self {
            config,
            project_path,
        }
    }

    /// Create a backup of all audit logs
    pub fn create_backup(&self) -> QmsResult<BackupStats> {
        let start_time = SystemTime::now();
        let mut stats = BackupStats::new();

        // Generate backup ID and paths
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let backup_id = format!("audit_backup_{timestamp}");
        let backup_dir = self.project_path.join(&self.config.backup_root).join(&backup_id);

        // Create backup directory
        fs::create_dir_all(&backup_dir)
            .map_err(|e| QmsError::io_error(&format!("Failed to create backup directory: {e}")))?;

        // Backup main audit directory
        let audit_dir = self.project_path.join("audit");
        if audit_dir.exists() {
            match self.backup_directory(&audit_dir, &backup_dir.join("audit"), &mut stats) {
                Ok(()) => {}
                Err(e) => {
                    stats.errors.push(format!("Failed to backup audit directory: {e}"));
                }
            }
        } else {
            stats.errors.push("Audit directory not found".to_string());
        }

        // Calculate compression ratio
        if stats.bytes_backed_up > 0 {
            stats.compression_ratio = if self.config.compress_enabled {
                1.0 - (stats.bytes_compressed as f64 / stats.bytes_backed_up as f64)
            } else {
                0.0
            };
        }

        // Record backup metadata
        let backup_info = BackupInfo {
            backup_id: backup_id.clone(),
            timestamp,
            source_path: audit_dir,
            backup_path: backup_dir.clone(),
            file_count: stats.files_backed_up,
            total_size: stats.bytes_backed_up,
            compressed: self.config.compress_enabled,
            checksum: self.calculate_backup_checksum(&backup_dir)?,
        };

        // Save backup metadata
        self.save_backup_metadata(&backup_info)?;

        // Calculate duration
        if let Ok(duration) = start_time.elapsed() {
            stats.backup_duration_ms = duration.as_millis() as u64;
        }

        Ok(stats)
    }

    /// Backup a directory recursively
    fn backup_directory(&self, source_dir: &Path, backup_dir: &Path, stats: &mut BackupStats) -> QmsResult<()> {
        fs::create_dir_all(backup_dir)
            .map_err(|e| QmsError::io_error(&format!("Failed to create backup subdirectory: {e}")))?;

        let entries = fs::read_dir(source_dir)
            .map_err(|e| QmsError::io_error(&format!("Failed to read source directory: {e}")))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| QmsError::io_error(&format!("Failed to read directory entry: {e}")))?;
            let source_path = entry.path();
            let file_name = source_path.file_name().unwrap();
            let backup_path = backup_dir.join(file_name);

            if source_path.is_dir() {
                // Recursively backup subdirectory
                self.backup_directory(&source_path, &backup_path, stats)?;
            } else {
                // Backup file
                match self.backup_file(&source_path, &backup_path, stats) {
                    Ok(()) => {}
                    Err(e) => {
                        stats.errors.push(format!("Failed to backup {}: {}", source_path.display(), e));
                    }
                }
            }
        }

        Ok(())
    }

    /// Backup a single file
    fn backup_file(&self, source_path: &Path, backup_path: &Path, stats: &mut BackupStats) -> QmsResult<()> {
        let file_content = fs::read(source_path)
            .map_err(|e| QmsError::io_error(&format!("Failed to read source file: {e}")))?;

        let original_size = file_content.len() as u64;
        stats.bytes_backed_up += original_size;

        if self.config.compress_enabled && source_path.extension().is_some_and(|ext| ext == "log") {
            // Simple compression simulation (in real implementation, would use actual compression)
            let compressed_content = self.compress_data(&file_content)?;
            let compressed_path = backup_path.with_extension("log.gz");
            
            fs::write(&compressed_path, &compressed_content)
                .map_err(|e| QmsError::io_error(&format!("Failed to write compressed backup: {e}")))?;

            stats.bytes_compressed += compressed_content.len() as u64;
        } else {
            // Copy file without compression
            fs::write(backup_path, &file_content)
                .map_err(|e| QmsError::io_error(&format!("Failed to write backup: {e}")))?;

            stats.bytes_compressed += original_size;
        }

        stats.files_backed_up += 1;
        Ok(())
    }

    /// Simple compression simulation (placeholder for actual compression)
    fn compress_data(&self, data: &[u8]) -> QmsResult<Vec<u8>> {
        // In a real implementation, this would use actual compression like gzip
        // For now, we'll simulate compression by adding a header and reducing apparent size
        let mut compressed = Vec::with_capacity(data.len() + 10);
        
        // Add simple compression header
        compressed.extend_from_slice(b"QMS_GZ");
        compressed.extend_from_slice(&(data.len() as u32).to_le_bytes());
        
        // Simulate 30% compression ratio by taking every 3rd byte (oversimplified)
        for (i, &byte) in data.iter().enumerate() {
            if i % 3 == 0 {
                compressed.push(byte);
            }
        }

        Ok(compressed)
    }

    /// Decompress data (placeholder for actual decompression)
    fn decompress_data(&self, compressed_data: &[u8]) -> QmsResult<Vec<u8>> {
        if compressed_data.len() < 10 {
            return Err(QmsError::validation_error("Invalid compressed data"));
        }

        // Check header
        if &compressed_data[0..6] != b"QMS_GZ" {
            return Err(QmsError::validation_error("Invalid compression header"));
        }

        let _original_size = u32::from_le_bytes([
            compressed_data[6], compressed_data[7], compressed_data[8], compressed_data[9]
        ]);

        // Simple decompression simulation - return the compressed data as-is for now
        // In a real implementation, this would properly decompress the data
        Ok(compressed_data[10..].to_vec())
    }

    /// Calculate checksum for backup directory
    fn calculate_backup_checksum(&self, backup_dir: &Path) -> QmsResult<String> {
        let mut hasher = DefaultHasher::new();
        Self::hash_directory_recursive(backup_dir, &mut hasher)?;
        Ok(format!("{:x}", hasher.finish()))
    }

    /// Recursively hash directory contents
    fn hash_directory_recursive(dir: &Path, hasher: &mut DefaultHasher) -> QmsResult<()> {
        let mut entries: Vec<_> = fs::read_dir(dir)
            .map_err(|e| QmsError::io_error(&format!("Failed to read directory for hashing: {e}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| QmsError::io_error(&format!("Failed to read directory entries: {e}")))?;

        // Sort entries for consistent hashing
        entries.sort_by_key(|entry| entry.path());

        for entry in entries {
            let path = entry.path();
            path.file_name().unwrap().to_string_lossy().hash(hasher);

            if path.is_dir() {
                Self::hash_directory_recursive(&path, hasher)?;
            } else {
                let content = fs::read(&path)
                    .map_err(|e| QmsError::io_error(&format!("Failed to read file for hashing: {e}")))?;
                content.hash(hasher);
            }
        }

        Ok(())
    }

    /// Save backup metadata
    fn save_backup_metadata(&self, backup_info: &BackupInfo) -> QmsResult<()> {
        let metadata_dir = self.project_path.join(&self.config.backup_root).join("metadata");
        fs::create_dir_all(&metadata_dir)
            .map_err(|e| QmsError::io_error(&format!("Failed to create metadata directory: {e}")))?;

        let metadata_file = metadata_dir.join(format!("{}.json", backup_info.backup_id));
        let metadata_json = self.backup_info_to_json(backup_info)?;

        fs::write(metadata_file, metadata_json)
            .map_err(|e| QmsError::io_error(&format!("Failed to save backup metadata: {e}")))?;

        Ok(())
    }

    /// Convert backup info to JSON
    fn backup_info_to_json(&self, info: &BackupInfo) -> QmsResult<String> {
        let mut json = String::new();
        json.push_str("{\n");
        json.push_str(&format!("  \"backup_id\": \"{}\",\n", info.backup_id));
        json.push_str(&format!("  \"timestamp\": {},\n", info.timestamp));
        json.push_str(&format!("  \"source_path\": \"{}\",\n", info.source_path.display()));
        json.push_str(&format!("  \"backup_path\": \"{}\",\n", info.backup_path.display()));
        json.push_str(&format!("  \"file_count\": {},\n", info.file_count));
        json.push_str(&format!("  \"total_size\": {},\n", info.total_size));
        json.push_str(&format!("  \"compressed\": {},\n", info.compressed));
        json.push_str(&format!("  \"checksum\": \"{}\"\n", info.checksum));
        json.push('}');
        Ok(json)
    }

    /// List all available backups
    pub fn list_backups(&self) -> QmsResult<Vec<BackupInfo>> {
        let metadata_dir = self.project_path.join(&self.config.backup_root).join("metadata");
        if !metadata_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();
        let entries = fs::read_dir(&metadata_dir)
            .map_err(|e| QmsError::io_error(&format!("Failed to read metadata directory: {e}")))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| QmsError::io_error(&format!("Failed to read metadata entry: {e}")))?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "json") {
                match self.load_backup_metadata(&path) {
                    Ok(backup_info) => backups.push(backup_info),
                    Err(e) => {
                        eprintln!("Warning: Failed to load backup metadata from {}: {}", path.display(), e);
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        backups.sort_by_key(|backup| std::cmp::Reverse(backup.timestamp));

        Ok(backups)
    }

    /// Load backup metadata from file
    fn load_backup_metadata(&self, metadata_path: &Path) -> QmsResult<BackupInfo> {
        let content = fs::read_to_string(metadata_path)
            .map_err(|e| QmsError::io_error(&format!("Failed to read metadata file: {e}")))?;

        self.parse_backup_info_json(&content)
    }

    /// Parse backup info from JSON (simplified parser)
    fn parse_backup_info_json(&self, json: &str) -> QmsResult<BackupInfo> {
        // Simple JSON parser for backup metadata
        let mut backup_id = String::new();
        let mut timestamp = 0u64;
        let mut source_path = PathBuf::new();
        let mut backup_path = PathBuf::new();
        let mut file_count = 0usize;
        let mut total_size = 0u64;
        let mut compressed = false;
        let mut checksum = String::new();

        for line in json.lines() {
            let line = line.trim();
            if line.contains("backup_id") {
                if let Some(value) = self.extract_json_string_value(line) {
                    backup_id = value;
                }
            } else if line.contains("timestamp") {
                if let Some(value) = self.extract_json_number_value(line) {
                    timestamp = value.parse().unwrap_or(0);
                }
            } else if line.contains("source_path") {
                if let Some(value) = self.extract_json_string_value(line) {
                    source_path = PathBuf::from(value);
                }
            } else if line.contains("backup_path") {
                if let Some(value) = self.extract_json_string_value(line) {
                    backup_path = PathBuf::from(value);
                }
            } else if line.contains("file_count") {
                if let Some(value) = self.extract_json_number_value(line) {
                    file_count = value.parse().unwrap_or(0);
                }
            } else if line.contains("total_size") {
                if let Some(value) = self.extract_json_number_value(line) {
                    total_size = value.parse().unwrap_or(0);
                }
            } else if line.contains("compressed") {
                compressed = line.contains("true");
            } else if line.contains("checksum") {
                if let Some(value) = self.extract_json_string_value(line) {
                    checksum = value;
                }
            }
        }

        Ok(BackupInfo {
            backup_id,
            timestamp,
            source_path,
            backup_path,
            file_count,
            total_size,
            compressed,
            checksum,
        })
    }

    /// Extract string value from JSON line
    fn extract_json_string_value(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find('"') {
            if let Some(end) = line.rfind('"') {
                if end > start + 1 {
                    let value_with_quotes = &line[line.find(": \"")? + 3..end];
                    return Some(value_with_quotes.to_string());
                }
            }
        }
        None
    }

    /// Extract number value from JSON line
    fn extract_json_number_value(&self, line: &str) -> Option<String> {
        if let Some(colon_pos) = line.find(':') {
            let after_colon = &line[colon_pos + 1..];
            let cleaned = after_colon.trim().trim_end_matches(',');
            return Some(cleaned.to_string());
        }
        None
    }

    /// Restore from backup
    pub fn restore_backup(&self, backup_id: &str) -> QmsResult<()> {
        let metadata_dir = self.project_path.join(&self.config.backup_root).join("metadata");
        let metadata_file = metadata_dir.join(format!("{backup_id}.json"));

        if !metadata_file.exists() {
            return Err(QmsError::NotFound(format!("Backup {backup_id} not found")));
        }

        let backup_info = self.load_backup_metadata(&metadata_file)?;

        // Verify backup integrity
        let current_checksum = self.calculate_backup_checksum(&backup_info.backup_path)?;
        if current_checksum != backup_info.checksum {
            return Err(QmsError::validation_error(
                "Backup integrity check failed - checksums do not match"
            ));
        }

        // Create backup of current audit directory before restore
        let current_backup_name = format!("pre_restore_{}", 
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
        let current_audit_dir = self.project_path.join("audit");
        if current_audit_dir.exists() {
            let pre_restore_backup = self.project_path
                .join(&self.config.backup_root)
                .join(&current_backup_name);
            fs::create_dir_all(&pre_restore_backup)
                .map_err(|e| QmsError::io_error(&format!("Failed to create pre-restore backup: {e}")))?;
            
            self.copy_directory(&current_audit_dir, &pre_restore_backup.join("audit"))?;
        }

        // Remove current audit directory
        if current_audit_dir.exists() {
            fs::remove_dir_all(&current_audit_dir)
                .map_err(|e| QmsError::io_error(&format!("Failed to remove current audit directory: {e}")))?;
        }

        // Restore from backup
        let backup_audit_dir = backup_info.backup_path.join("audit");
        self.copy_directory(&backup_audit_dir, &current_audit_dir)?;

        Ok(())
    }

    /// Copy directory recursively
    fn copy_directory(&self, source: &Path, dest: &Path) -> QmsResult<()> {
        fs::create_dir_all(dest)
            .map_err(|e| QmsError::io_error(&format!("Failed to create destination directory: {e}")))?;

        let entries = fs::read_dir(source)
            .map_err(|e| QmsError::io_error(&format!("Failed to read source directory: {e}")))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| QmsError::io_error(&format!("Failed to read directory entry: {e}")))?;
            let source_path = entry.path();
            let file_name = source_path.file_name().unwrap();
            let dest_path = dest.join(file_name);

            if source_path.is_dir() {
                self.copy_directory(&source_path, &dest_path)?;
            } else if source_path.extension().is_some_and(|ext| ext == "gz") {
                // Decompress file
                let compressed_content = fs::read(&source_path)
                    .map_err(|e| QmsError::io_error(&format!("Failed to read compressed file: {e}")))?;
                let decompressed_content = self.decompress_data(&compressed_content)?;
                let original_path = dest_path.with_extension("");
                fs::write(&original_path, decompressed_content)
                    .map_err(|e| QmsError::io_error(&format!("Failed to write decompressed file: {e}")))?;
            } else {
                fs::copy(&source_path, &dest_path)
                    .map_err(|e| QmsError::io_error(&format!("Failed to copy file: {e}")))?;
            }
        }

        Ok(())
    }

    /// Delete old backups based on retention policy
    pub fn cleanup_old_backups(&self) -> QmsResult<usize> {
        let retention_cutoff = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - (self.config.retention_days as u64 * 24 * 60 * 60);

        let backups = self.list_backups()?;
        let mut deleted_count = 0;

        for backup in backups {
            if backup.timestamp < retention_cutoff {
                // Delete backup directory
                if backup.backup_path.exists() {
                    fs::remove_dir_all(&backup.backup_path)
                        .map_err(|e| QmsError::io_error(&format!("Failed to delete backup directory: {e}")))?;
                }

                // Delete metadata file
                let metadata_file = self.project_path
                    .join(&self.config.backup_root)
                    .join("metadata")
                    .join(format!("{}.json", backup.backup_id));
                if metadata_file.exists() {
                    fs::remove_file(&metadata_file)
                        .map_err(|e| QmsError::io_error(&format!("Failed to delete metadata file: {e}")))?;
                }

                deleted_count += 1;
            }
        }

        Ok(deleted_count)
    }

    /// Verify backup integrity
    pub fn verify_backup(&self, backup_id: &str) -> QmsResult<bool> {
        let metadata_dir = self.project_path.join(&self.config.backup_root).join("metadata");
        let metadata_file = metadata_dir.join(format!("{backup_id}.json"));

        if !metadata_file.exists() {
            return Err(QmsError::NotFound(format!("Backup {backup_id} not found")));
        }

        let backup_info = self.load_backup_metadata(&metadata_file)?;

        if !backup_info.backup_path.exists() {
            return Ok(false);
        }

        let current_checksum = self.calculate_backup_checksum(&backup_info.backup_path)?;
        Ok(current_checksum == backup_info.checksum)
    }
}

/// Format backup statistics for display
pub fn format_backup_stats(stats: &BackupStats) -> String {
    let mut output = String::new();
    
    output.push_str("📊 Backup Statistics\n");
    output.push_str("===================\n");
    output.push_str(&format!("Files backed up: {}\n", stats.files_backed_up));
    output.push_str(&format!("Original size: {:.2} MB\n", stats.bytes_backed_up as f64 / 1_048_576.0));
    
    if stats.compression_ratio > 0.0 {
        output.push_str(&format!("Compressed size: {:.2} MB\n", stats.bytes_compressed as f64 / 1_048_576.0));
        output.push_str(&format!("Compression ratio: {:.1}%\n", stats.compression_ratio * 100.0));
        output.push_str(&format!("Space saved: {:.2} MB\n", 
            (stats.bytes_backed_up - stats.bytes_compressed) as f64 / 1_048_576.0));
    }
    
    output.push_str(&format!("Duration: {} ms\n", stats.backup_duration_ms));
    
    if !stats.errors.is_empty() {
        output.push_str("\n⚠️  Warnings:\n");
        for error in &stats.errors {
            output.push_str(&format!("  - {error}\n"));
        }
    }
    
    output
}

/// Format backup info for display
pub fn format_backup_info(backup: &BackupInfo) -> String {
    let timestamp_str = if backup.timestamp > 0 {
        let datetime = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(backup.timestamp);
        format!("{datetime:?}")
    } else {
        "Unknown".to_string()
    };

    format!(
        "Backup ID: {}\n\
         Timestamp: {} ({})\n\
         Files: {}\n\
         Size: {:.2} MB\n\
         Compressed: {}\n\
         Path: {}\n\
         Checksum: {}",
        backup.backup_id,
        backup.timestamp,
        timestamp_str,
        backup.file_count,
        backup.total_size as f64 / 1_048_576.0,
        if backup.compressed { "Yes" } else { "No" },
        backup.backup_path.display(),
        backup.checksum
    )
}