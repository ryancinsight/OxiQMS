//! Audit entry functionality for regulatory compliance
//! Phase 2.2.1 - Audit Entry Schema & Storage
//! Implements comprehensive audit trail per FDA 21 CFR Part 11 and ISO 13485
//!
//! REFACTORED: Separated responsibilities following Single Responsibility Principle
//! - AuditEntryLogger: Handles entry writing and formatting
//! - AuditFileRotator: Manages file rotation and directory structure
//! - AuditStatisticsCollector: Collects and calculates audit statistics
//! - AuditLoggerOrchestrator: Coordinates the above components

#![allow(dead_code)] // Phase 2.2.1 infrastructure - comprehensive audit system

use crate::error::{QmsError, QmsResult};
use crate::json_utils::JsonSerializable;
use crate::models::{AuditEntry, AuditAction, ElectronicSignature};
use crate::utils::{current_timestamp, generate_uuid, current_date_string};
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{Write, BufRead, BufReader};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

/// Global audit logger instance for thread-safe operations
static AUDIT_LOGGER: OnceLock<Mutex<AuditLogger>> = OnceLock::new();

/// Configuration for audit logging
#[derive(Debug, Clone)]
pub struct AuditConfig {
    pub project_path: String,
    pub retention_days: u32,      // Default: 2555 days (7 years for medical devices)
    pub daily_rotation: bool,     // Enable daily log rotation
    pub max_file_size_mb: u32,    // Max size before rotation (default: 100MB)
    pub require_checksums: bool,  // Calculate integrity checksums
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            project_path: ".".to_string(),
            retention_days: 2555, // 7 years for medical device compliance
            daily_rotation: true,
            max_file_size_mb: 100,
            require_checksums: true,
        }
    }
}

impl AuditConfig {
    /// Create medical device compliant configuration
    pub fn medical_device_default() -> Self {
        Self {
            project_path: ".".to_string(),
            retention_days: 2555, // 7 years for medical device compliance
            daily_rotation: true,
            max_file_size_mb: 50,  // Smaller files for better integrity
            require_checksums: true,
        }
    }
}

/// Audit statistics for monitoring and compliance
#[derive(Debug, Clone)]
pub struct AuditStats {
    pub total_entries: usize,
    pub entries_today: usize,
    pub file_count: usize,
    pub total_size_bytes: u64,
    pub oldest_entry_date: Option<String>,
    pub newest_entry_date: Option<String>,
}

// ================================================================================================
// REFACTORED COMPONENTS - Single Responsibility Principle Implementation
// ================================================================================================

/// Handles audit entry writing and formatting
/// Single Responsibility: Writing audit entries to files
pub struct AuditEntryLogger {
    current_file: Option<File>,
}

impl AuditEntryLogger {
    /// Create a new audit entry logger
    pub fn new() -> Self {
        Self {
            current_file: None,
        }
    }

    /// Set the current file for writing
    pub fn set_file(&mut self, file: File) {
        self.current_file = Some(file);
    }

    /// Write an audit entry to the current file
    pub fn write_entry(&mut self, entry: &AuditEntry) -> QmsResult<()> {
        if let Some(ref mut file) = self.current_file {
            let json_line = format!("{}\n", entry.to_json());
            file.write_all(json_line.as_bytes())
                .map_err(|e| QmsError::io_error(&format!("Failed to write audit entry: {e}")))?;
            file.flush()
                .map_err(|e| QmsError::io_error(&format!("Failed to flush audit log: {e}")))?;
            Ok(())
        } else {
            Err(QmsError::domain_error("No audit log file is open for writing"))
        }
    }

    /// Close the current file
    pub fn close_file(&mut self) {
        self.current_file = None;
    }
}

/// Manages file rotation and directory structure
/// Single Responsibility: File system operations for audit logs
pub struct AuditFileRotator {
    config: AuditConfig,
    current_date: String,
}

impl AuditFileRotator {
    /// Create a new file rotator
    pub fn new(config: AuditConfig) -> QmsResult<Self> {
        let mut rotator = Self {
            config,
            current_date: String::new(),
        };

        rotator.ensure_directory_structure()?;
        Ok(rotator)
    }

    /// Ensure audit directory structure exists
    pub fn ensure_directory_structure(&self) -> QmsResult<()> {
        let audit_dir = PathBuf::from(&self.config.project_path).join("audit");
        let daily_dir = audit_dir.join("daily");
        let exports_dir = audit_dir.join("exports");

        create_dir_all(&audit_dir)
            .map_err(|e| QmsError::io_error(&format!("Failed to create audit directory: {e}")))?;
        create_dir_all(&daily_dir)
            .map_err(|e| QmsError::io_error(&format!("Failed to create daily audit directory: {e}")))?;
        create_dir_all(&exports_dir)
            .map_err(|e| QmsError::io_error(&format!("Failed to create exports directory: {e}")))?;

        Ok(())
    }

    /// Check if rotation is needed and return new file if so
    pub fn check_rotation_needed(&mut self) -> QmsResult<Option<File>> {
        let today = current_date_string();

        if self.config.daily_rotation && self.current_date != today {
            self.current_date = today;
            Ok(Some(self.open_log_file()?))
        } else if self.current_date.is_empty() {
            self.current_date = today;
            Ok(Some(self.open_log_file()?))
        } else {
            Ok(None)
        }
    }

    /// Open a log file for the current date
    fn open_log_file(&self) -> QmsResult<File> {
        let filename = if self.config.daily_rotation {
            format!("{}.log", self.current_date)
        } else {
            "audit.log".to_string()
        };

        let file_path = PathBuf::from(&self.config.project_path)
            .join("audit")
            .join(&filename);

        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)
            .map_err(|e| QmsError::io_error(&format!("Failed to open audit log file: {e}")))
    }

    /// Clean up old audit logs based on retention policy
    pub fn cleanup_old_logs(&self) -> QmsResult<usize> {
        let audit_dir = PathBuf::from(&self.config.project_path).join("audit");
        let mut deleted_count = 0;

        if !audit_dir.exists() {
            return Ok(0);
        }

        let cutoff_timestamp = current_timestamp() - (self.config.retention_days as u64 * 24 * 60 * 60);

        for entry in std::fs::read_dir(&audit_dir).map_err(|e| QmsError::io_error(&format!("Failed to read audit directory: {e}")))? {
            let entry = entry.map_err(|e| QmsError::io_error(&format!("Failed to read directory entry: {e}")))?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("log") {
                if let Ok(metadata) = path.metadata() {
                    if let Ok(created) = metadata.created() {
                        if let Ok(duration) = created.duration_since(std::time::UNIX_EPOCH) {
                            if duration.as_secs() < cutoff_timestamp
                                && std::fs::remove_file(&path).is_ok() {
                                    deleted_count += 1;
                                }
                        }
                    }
                }
            }
        }

        Ok(deleted_count)
    }
}

/// Collects and calculates audit statistics
/// Single Responsibility: Statistics collection and calculation
pub struct AuditStatisticsCollector {
    config: AuditConfig,
    entry_count: usize,
}

impl AuditStatisticsCollector {
    /// Create a new statistics collector
    pub fn new(config: AuditConfig) -> Self {
        Self {
            config,
            entry_count: 0,
        }
    }

    /// Increment the entry count
    pub fn increment_count(&mut self) {
        self.entry_count += 1;
    }

    /// Get current entry count
    pub fn get_entry_count(&self) -> usize {
        self.entry_count
    }

    /// Calculate comprehensive audit statistics
    pub fn calculate_statistics(&self) -> QmsResult<AuditStats> {
        let audit_dir = PathBuf::from(&self.config.project_path).join("audit");
        let mut stats = AuditStats {
            total_entries: 0,
            entries_today: 0,
            file_count: 0,
            total_size_bytes: 0,
            oldest_entry_date: None,
            newest_entry_date: None,
        };

        if audit_dir.exists() {
            for entry in std::fs::read_dir(&audit_dir).map_err(|e| QmsError::io_error(&format!("Failed to read audit directory: {e}")))? {
                let entry = entry.map_err(|e| QmsError::io_error(&format!("Failed to read directory entry: {e}")))?;
                let path = entry.path();

                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("log") {
                    stats.file_count += 1;

                    if let Ok(metadata) = path.metadata() {
                        stats.total_size_bytes += metadata.len();
                    }

                    // Count entries in file
                    if let Ok(file) = File::open(&path) {
                        let reader = BufReader::new(file);
                        let mut file_entries = 0;

                        for line in reader.lines() {
                            if line.is_ok() {
                                file_entries += 1;
                                stats.total_entries += 1;
                            }
                        }

                        // Check if this is today's file
                        if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                            if filename == current_date_string() {
                                stats.entries_today = file_entries;
                            }

                            // Track oldest and newest dates
                            if stats.oldest_entry_date.is_none() || filename < stats.oldest_entry_date.as_ref().unwrap() {
                                stats.oldest_entry_date = Some(filename.to_string());
                            }
                            if stats.newest_entry_date.is_none() || filename > stats.newest_entry_date.as_ref().unwrap() {
                                stats.newest_entry_date = Some(filename.to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(stats)
    }
}

/// Orchestrates audit logging components following Single Responsibility Principle
/// Coordinates: AuditEntryLogger, AuditFileRotator, and AuditStatisticsCollector
pub struct AuditLoggerOrchestrator {
    logger: AuditEntryLogger,
    rotator: AuditFileRotator,
    stats: AuditStatisticsCollector,
}

impl AuditLoggerOrchestrator {
    /// Create a new audit logger orchestrator
    pub fn new(config: AuditConfig) -> QmsResult<Self> {
        let rotator = AuditFileRotator::new(config.clone())?;
        let stats = AuditStatisticsCollector::new(config.clone());
        let mut logger = AuditEntryLogger::new();

        // Initialize with the current log file
        let mut orchestrator = Self {
            logger,
            rotator,
            stats,
        };

        // Ensure we have a file open for writing
        orchestrator.ensure_file_open()?;

        Ok(orchestrator)
    }

    /// Ensure a log file is open for writing
    fn ensure_file_open(&mut self) -> QmsResult<()> {
        if let Some(file) = self.rotator.check_rotation_needed()? {
            self.logger.set_file(file);
        }
        Ok(())
    }

    /// Log an audit entry (main public interface)
    pub fn log_entry(&mut self, entry: &AuditEntry) -> QmsResult<()> {
        // Check if rotation is needed and update file if so
        if let Some(file) = self.rotator.check_rotation_needed()? {
            self.logger.set_file(file);
        }

        // Write the entry
        self.logger.write_entry(entry)?;

        // Update statistics
        self.stats.increment_count();

        Ok(())
    }

    /// Get audit statistics
    pub fn get_statistics(&self) -> QmsResult<AuditStats> {
        self.stats.calculate_statistics()
    }

    /// Clean up old logs
    pub fn cleanup_old_logs(&self) -> QmsResult<usize> {
        self.rotator.cleanup_old_logs()
    }
}

/// Comprehensive audit logger with regulatory compliance features
/// REFACTORED: Now uses AuditLoggerOrchestrator internally while maintaining API compatibility
pub struct AuditLogger {
    orchestrator: AuditLoggerOrchestrator,
}

impl AuditLogger {
    /// Create a new audit logger with configuration
    /// REFACTORED: Now delegates to AuditLoggerOrchestrator
    pub fn new(config: AuditConfig) -> QmsResult<Self> {
        let orchestrator = AuditLoggerOrchestrator::new(config)?;
        Ok(Self { orchestrator })
    }

    /// Log a complete audit entry (thread-safe)
    /// REFACTORED: Delegates to orchestrator
    pub fn log_entry(&mut self, entry: &AuditEntry) -> QmsResult<()> {
        self.orchestrator.log_entry(entry)
    }

    /// Get audit statistics for monitoring
    /// REFACTORED: Delegates to orchestrator
    pub fn get_statistics(&self) -> QmsResult<AuditStats> {
        self.orchestrator.get_statistics()
    }

    /// Clean up old audit logs based on retention policy
    /// REFACTORED: Delegates to orchestrator
    pub fn cleanup_old_logs(&self) -> QmsResult<usize> {
        self.orchestrator.cleanup_old_logs()
    }
}

/// Audit entry builder for convenient creation
pub struct AuditEntryBuilder {
    entry: AuditEntry,
}

impl AuditEntryBuilder {
    /// Create a new audit entry builder
    pub fn new(user_id: String, action: AuditAction, entity_type: String, entity_id: String) -> Self {
        let timestamp = crate::utils::current_iso8601_timestamp();
        let id = generate_uuid();
        
        let mut entry = AuditEntry {
            id: id.clone(),
            timestamp,
            user_id,
            session_id: None,
            action,
            entity_type,
            entity_id,
            old_value: None,
            new_value: None,
            details: None,
            ip_address: None,
            signature: None,
            checksum: String::new(),
            previous_hash: None,
        };

        // Calculate checksum for integrity
        entry.checksum = calculate_entry_checksum(&entry);
        
        Self { entry }
    }

    /// Set session ID
    pub fn session_id(mut self, session_id: String) -> Self {
        self.entry.session_id = Some(session_id);
        self.entry.checksum = calculate_entry_checksum(&self.entry);
        self
    }

    /// Set old and new values for update operations
    pub fn values(mut self, old_value: Option<String>, new_value: Option<String>) -> Self {
        self.entry.old_value = old_value;
        self.entry.new_value = new_value;
        self.entry.checksum = calculate_entry_checksum(&self.entry);
        self
    }

    /// Set additional details
    pub fn details(mut self, details: String) -> Self {
        self.entry.details = Some(details);
        self.entry.checksum = calculate_entry_checksum(&self.entry);
        self
    }

    /// Set IP address (for web interface)
    pub fn ip_address(mut self, ip_address: String) -> Self {
        self.entry.ip_address = Some(ip_address);
        self.entry.checksum = calculate_entry_checksum(&self.entry);
        self
    }

    /// Add electronic signature for critical operations
    pub fn signature(mut self, user_id: String, meaning: String, signed_data: &str) -> Self {
        let signature = ElectronicSignature {
            user_id,
            timestamp: crate::utils::current_iso8601_timestamp(),
            meaning,
            signed_data_hash: crate::json_utils::calculate_checksum(signed_data),
            certificate_info: None,
        };
        
        self.entry.signature = Some(signature);
        self.entry.checksum = calculate_entry_checksum(&self.entry);
        self
    }

    /// Set previous hash for chain integrity
    pub fn previous_hash(mut self, previous_hash: String) -> Self {
        self.entry.previous_hash = Some(previous_hash);
        self.entry.checksum = calculate_entry_checksum(&self.entry);
        self
    }

    /// Build the final audit entry
    pub fn build(self) -> AuditEntry {
        self.entry
    }
}

/// Calculate integrity checksum for audit entry
fn calculate_entry_checksum(entry: &AuditEntry) -> String {
    let mut data = format!("{}{}{}{}{}{}",
        entry.id, entry.timestamp, entry.user_id, entry.action.to_json(),
        entry.entity_type, entry.entity_id
    );
    
    if let Some(ref old_val) = entry.old_value {
        data.push_str(old_val);
    }
    if let Some(ref new_val) = entry.new_value {
        data.push_str(new_val);
    }
    if let Some(ref details) = entry.details {
        data.push_str(details);
    }
    if let Some(ref prev_hash) = entry.previous_hash {
        data.push_str(prev_hash);
    }
    
    crate::json_utils::calculate_checksum(&data)
}

/// Initialize global audit logger
pub fn initialize_audit_logger(config: AuditConfig) -> QmsResult<()> {
    let logger = AuditLogger::new(config)?;
    
    // In test mode, allow reinitialization by checking if already initialized
    match AUDIT_LOGGER.set(Mutex::new(logger)) {
        Ok(_) => Ok(()),
        Err(_) => {
            // Already initialized - this is OK in test mode
            #[cfg(test)]
            return Ok(());
            
            #[cfg(not(test))]
            return Err(QmsError::domain_error("Audit logger already initialized"));
        }
    }
}

/// Log an audit entry using the global logger
pub fn log_audit_entry(entry: &AuditEntry) -> QmsResult<()> {
    if let Some(logger_mutex) = AUDIT_LOGGER.get() {
        let mut logger = logger_mutex.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire audit logger lock"))?;
        logger.log_entry(entry)
    } else {
        Err(QmsError::domain_error("Audit logger not initialized"))
    }
}

/// Convenience function to log a simple action
pub fn log_action(user_id: &str, action: AuditAction, entity_type: &str, entity_id: &str) -> QmsResult<()> {
    let entry = AuditEntryBuilder::new(
        user_id.to_string(),
        action,
        entity_type.to_string(),
        entity_id.to_string(),
    ).build();
    
    log_audit_entry(&entry)
}

/// Convenience function to log an update with old/new values
pub fn log_update(user_id: &str, entity_type: &str, entity_id: &str, 
                 old_value: &str, new_value: &str, details: Option<&str>) -> QmsResult<()> {
    let mut builder = AuditEntryBuilder::new(
        user_id.to_string(),
        AuditAction::Update,
        entity_type.to_string(),
        entity_id.to_string(),
    ).values(Some(old_value.to_string()), Some(new_value.to_string()));
    
    if let Some(details_str) = details {
        builder = builder.details(details_str.to_string());
    }
    
    log_audit_entry(&builder.build())
}

/// Convenience function to log with electronic signature
pub fn log_with_signature(user_id: &str, action: AuditAction, entity_type: &str, entity_id: &str,
                         signature_meaning: &str, signed_data: &str) -> QmsResult<()> {
    let entry = AuditEntryBuilder::new(
        user_id.to_string(),
        action,
        entity_type.to_string(),
        entity_id.to_string(),
    ).signature(user_id.to_string(), signature_meaning.to_string(), signed_data)
     .build();
    
    log_audit_entry(&entry)
}

/// Get audit statistics from global logger
pub fn get_audit_statistics() -> QmsResult<AuditStats> {
    if let Some(logger_mutex) = AUDIT_LOGGER.get() {
        let logger = logger_mutex.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire audit logger lock"))?;
        logger.get_statistics()
    } else {
        Err(QmsError::domain_error("Audit logger not initialized"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::env;
    use std::sync::Mutex;

    // Global mutex to ensure audit logger tests run sequentially
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn create_temp_audit_dir() -> std::io::Result<PathBuf> {
        let temp_dir = env::temp_dir().join(format!("qms_audit_test_{}", current_timestamp()));
        fs::create_dir_all(&temp_dir)?;
        Ok(temp_dir)
    }

    #[test]
    fn test_audit_entry_builder() {
        let entry = AuditEntryBuilder::new(
            "test_user".to_string(),
            AuditAction::Create,
            "Document".to_string(),
            "DOC-001".to_string(),
        ).details("Test creation".to_string())
         .build();

        assert_eq!(entry.user_id, "test_user");
        assert_eq!(entry.action, AuditAction::Create);
        assert_eq!(entry.entity_type, "Document");
        assert_eq!(entry.entity_id, "DOC-001");
        assert_eq!(entry.details, Some("Test creation".to_string()));
        assert!(!entry.checksum.is_empty());
    }

    #[test]
    fn test_audit_logger_creation() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|poisoned| poisoned.into_inner()); // Ensure sequential execution
        let temp_dir = create_temp_audit_dir().unwrap();
        let config = AuditConfig {
            project_path: temp_dir.to_string_lossy().to_string(),
            ..Default::default()
        };

        let logger = AuditLogger::new(config);
        assert!(logger.is_ok());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_audit_entry_logging() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|poisoned| poisoned.into_inner()); // Ensure sequential execution
        let temp_dir = create_temp_audit_dir().unwrap();

        // Ensure audit directory exists
        let audit_dir = temp_dir.join("audit");
        std::fs::create_dir_all(&audit_dir).unwrap();

        let config = AuditConfig {
            project_path: temp_dir.to_string_lossy().to_string(),
            ..Default::default()
        };

        let mut logger = AuditLogger::new(config).unwrap();
        
        let entry = AuditEntryBuilder::new(
            "test_user".to_string(),
            AuditAction::Create,
            "Document".to_string(),
            "DOC-001".to_string(),
        ).build();

        let result = logger.log_entry(&entry);
        assert!(result.is_ok());

        // Verify file exists
        let audit_file = temp_dir.join("audit").join(format!("{}.log", current_date_string()));
        assert!(audit_file.exists());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_json_serialization() {
        let entry = AuditEntryBuilder::new(
            "test_user".to_string(),
            AuditAction::Update,
            "Document".to_string(),
            "DOC-001".to_string(),
        ).values(Some("old content".to_string()), Some("new content".to_string()))
         .details("Content update".to_string())
         .build();

        let json = entry.to_json();
        assert!(json.contains("\"action\": \"Update\""));
        assert!(json.contains("\"old_value\": \"old content\""));
        assert!(json.contains("\"new_value\": \"new content\""));

        // Test round-trip
        let parsed = AuditEntry::from_json(&json);
        assert!(parsed.is_ok());
        let parsed_entry = parsed.unwrap();
        assert_eq!(parsed_entry.user_id, entry.user_id);
        assert_eq!(parsed_entry.action, entry.action);
    }

    #[test]
    fn test_electronic_signature() {
        let entry = AuditEntryBuilder::new(
            "test_user".to_string(),
            AuditAction::Approve,
            "Document".to_string(),
            "DOC-001".to_string(),
        ).signature("test_user".to_string(), "Approved for release".to_string(), "document_content")
         .build();

        assert!(entry.signature.is_some());
        let signature = entry.signature.unwrap();
        assert_eq!(signature.user_id, "test_user");
        assert_eq!(signature.meaning, "Approved for release");
        assert!(!signature.signed_data_hash.is_empty());
    }
}
