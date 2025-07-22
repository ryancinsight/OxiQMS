//! Interface Segregation for Audit Logger Components
//! 
//! This module implements Interface Segregation Principle (ISP) for audit logging,
//! breaking down large interfaces into focused, cohesive contracts.
//! 
//! SOLID Principles Applied:
//! - Interface Segregation: Separate interfaces for different audit concerns
//! - Single Responsibility: Each interface handles one specific aspect
//! - Dependency Inversion: High-level modules depend on abstractions
//! - Open/Closed: New implementations can be added without modification

use crate::prelude::*;
use crate::models::AuditEntry;
use crate::json_utils::JsonSerializable;
use std::path::Path;

/// Interface Segregation Principle: Focused interface for audit entry reading
pub trait AuditReader {
    /// Read a single audit entry by ID
    fn read_entry(&self, entry_id: &str) -> QmsResult<AuditEntry>;
    
    /// Read all audit entries
    fn read_all_entries(&self) -> QmsResult<Vec<AuditEntry>>;
    
    /// Read entries within a date range
    fn read_entries_by_date_range(&self, start: &str, end: &str) -> QmsResult<Vec<AuditEntry>>;
    
    /// Read entries for a specific user
    fn read_entries_by_user(&self, user_id: &str) -> QmsResult<Vec<AuditEntry>>;
    
    /// Check if an entry exists
    fn entry_exists(&self, entry_id: &str) -> QmsResult<bool>;
}

/// Interface Segregation Principle: Focused interface for audit entry writing
pub trait AuditWriter {
    /// Write a single audit entry
    fn write_entry(&self, entry: &AuditEntry) -> QmsResult<()>;
    
    /// Write multiple audit entries in batch
    fn write_entries_batch(&self, entries: &[AuditEntry]) -> QmsResult<()>;
    
    /// Append entry to audit log with integrity chain
    fn append_with_integrity(&self, entry: &AuditEntry) -> QmsResult<()>;
    
    /// Flush any buffered entries to storage
    fn flush(&self) -> QmsResult<()>;
}

/// Interface Segregation Principle: Focused interface for audit searching
pub trait AuditSearcher {
    /// Search entries by criteria
    fn search(&self, criteria: &AuditSearchCriteria) -> QmsResult<Vec<AuditEntry>>;
    
    /// Search entries by text content
    fn search_by_text(&self, query: &str) -> QmsResult<Vec<AuditEntry>>;
    
    /// Search entries by action type
    fn search_by_action(&self, action: &crate::models::AuditAction) -> QmsResult<Vec<AuditEntry>>;
    
    /// Search entries by entity type
    fn search_by_entity_type(&self, entity_type: &str) -> QmsResult<Vec<AuditEntry>>;
    
    /// Get search statistics
    fn get_search_statistics(&self) -> QmsResult<SearchStatistics>;
}

/// Interface Segregation Principle: Focused interface for audit formatting
pub trait AuditFormatter {
    /// Format entries as JSON
    fn format_as_json(&self, entries: &[AuditEntry]) -> QmsResult<String>;
    
    /// Format entries as CSV
    fn format_as_csv(&self, entries: &[AuditEntry]) -> QmsResult<String>;
    
    /// Format entries as human-readable text
    fn format_as_text(&self, entries: &[AuditEntry]) -> QmsResult<String>;
    
    /// Format single entry with custom template
    fn format_entry_with_template(&self, entry: &AuditEntry, template: &str) -> QmsResult<String>;
}

/// Interface Segregation Principle: Focused interface for audit validation
pub trait AuditValidator {
    /// Validate audit entry structure
    fn validate_entry(&self, entry: &AuditEntry) -> QmsResult<ValidationResult>;
    
    /// Validate audit chain integrity
    fn validate_chain_integrity(&self, entries: &[AuditEntry]) -> QmsResult<IntegrityResult>;
    
    /// Validate compliance with regulations
    fn validate_compliance(&self, entries: &[AuditEntry]) -> QmsResult<ComplianceResult>;
    
    /// Validate entry checksums
    fn validate_checksums(&self, entries: &[AuditEntry]) -> QmsResult<ChecksumResult>;
}

/// Interface Segregation Principle: Focused interface for audit archiving
pub trait AuditArchiver {
    /// Archive old audit entries
    fn archive_entries(&self, before_date: &str) -> QmsResult<ArchiveResult>;
    
    /// Restore archived entries
    fn restore_entries(&self, archive_id: &str) -> QmsResult<RestoreResult>;
    
    /// List available archives
    fn list_archives(&self) -> QmsResult<Vec<ArchiveInfo>>;
    
    /// Delete old archives
    fn cleanup_archives(&self, retention_days: u32) -> QmsResult<CleanupResult>;
}

/// Interface Segregation Principle: Focused interface for audit metrics
pub trait AuditMetricsCollector {
    /// Collect performance metrics
    fn collect_performance_metrics(&self) -> QmsResult<PerformanceMetrics>;
    
    /// Collect usage statistics
    fn collect_usage_statistics(&self) -> QmsResult<UsageStatistics>;
    
    /// Collect compliance metrics
    fn collect_compliance_metrics(&self) -> QmsResult<ComplianceMetrics>;
    
    /// Reset metrics collection
    fn reset_metrics(&self) -> QmsResult<()>;
}

/// Search criteria for audit entries
#[derive(Debug, Clone)]
pub struct AuditSearchCriteria {
    pub user_id: Option<String>,
    pub action: Option<crate::models::AuditAction>,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub text_query: Option<String>,
    pub limit: Option<usize>,
}

/// Search statistics
#[derive(Debug, Clone)]
pub struct SearchStatistics {
    pub total_searches: u64,
    pub average_search_time_ms: f64,
    pub most_common_queries: Vec<String>,
    pub search_success_rate: f64,
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Integrity check result
#[derive(Debug, Clone)]
pub struct IntegrityResult {
    pub is_intact: bool,
    pub broken_chains: Vec<String>,
    pub missing_entries: Vec<String>,
    pub checksum_failures: Vec<String>,
}

/// Compliance check result
#[derive(Debug, Clone)]
pub struct ComplianceResult {
    pub is_compliant: bool,
    pub violations: Vec<ComplianceViolation>,
    pub compliance_score: f64,
}

#[derive(Debug, Clone)]
pub struct ComplianceViolation {
    pub rule: String,
    pub description: String,
    pub severity: String,
    pub entry_id: String,
}

/// Checksum validation result
#[derive(Debug, Clone)]
pub struct ChecksumResult {
    pub all_valid: bool,
    pub invalid_entries: Vec<String>,
    pub total_checked: usize,
}

/// Archive operation result
#[derive(Debug, Clone)]
pub struct ArchiveResult {
    pub archive_id: String,
    pub entries_archived: usize,
    pub archive_size_bytes: u64,
    pub archive_path: String,
}

/// Restore operation result
#[derive(Debug, Clone)]
pub struct RestoreResult {
    pub entries_restored: usize,
    pub restore_path: String,
    pub conflicts_resolved: usize,
}

/// Archive information
#[derive(Debug, Clone)]
pub struct ArchiveInfo {
    pub archive_id: String,
    pub created_date: String,
    pub entry_count: usize,
    pub size_bytes: u64,
    pub description: String,
}

/// Cleanup operation result
#[derive(Debug, Clone)]
pub struct CleanupResult {
    pub archives_deleted: usize,
    pub space_freed_bytes: u64,
    pub cleanup_date: String,
}

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub entries_per_second: f64,
    pub average_write_time_ms: f64,
    pub average_read_time_ms: f64,
    pub memory_usage_mb: f64,
    pub disk_usage_mb: f64,
}

/// Usage statistics
#[derive(Debug, Clone)]
pub struct UsageStatistics {
    pub total_entries: u64,
    pub entries_today: u64,
    pub active_users: u64,
    pub most_active_user: String,
    pub peak_usage_hour: u8,
}

/// Compliance metrics
#[derive(Debug, Clone)]
pub struct ComplianceMetrics {
    pub compliance_score: f64,
    pub total_violations: u64,
    pub critical_violations: u64,
    pub last_audit_date: String,
    pub next_audit_due: String,
}

/// File-based implementation of audit interfaces
/// Single Responsibility Principle: Handles file-based audit operations
pub struct FileAuditStorage {
    storage_path: std::path::PathBuf,
    buffer_size: usize,
}

impl FileAuditStorage {
    pub fn new(storage_path: &Path) -> Self {
        Self {
            storage_path: storage_path.to_path_buf(),
            buffer_size: 1000,
        }
    }
    
    pub const fn with_buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = buffer_size;
        self
    }
    
    /// Initialize storage directory structure
    pub fn initialize(&self) -> QmsResult<()> {
        std::fs::create_dir_all(&self.storage_path)?;
        std::fs::create_dir_all(self.storage_path.join("archives"))?;
        std::fs::create_dir_all(self.storage_path.join("backups"))?;
        Ok(())
    }
    
    /// Get audit log file path
    fn get_log_path(&self) -> std::path::PathBuf {
        self.storage_path.join("audit.log")
    }
    
    /// Parse audit entry from JSON line
    fn parse_entry_from_line(&self, line: &str) -> QmsResult<AuditEntry> {
        AuditEntry::from_json(line)
            .map_err(|e| QmsError::domain_error(&format!("Failed to parse audit entry: {e}")))
    }
}

impl AuditReader for FileAuditStorage {
    fn read_entry(&self, entry_id: &str) -> QmsResult<AuditEntry> {
        let entries = self.read_all_entries()?;
        entries.into_iter()
            .find(|entry| entry.id == entry_id)
            .ok_or_else(|| QmsError::NotFound(format!("Audit entry not found: {entry_id}")))
    }
    
    fn read_all_entries(&self) -> QmsResult<Vec<AuditEntry>> {
        let log_path = self.get_log_path();
        if !log_path.exists() {
            return Ok(Vec::new());
        }
        
        let content = std::fs::read_to_string(log_path)?;
        let mut entries = Vec::new();
        
        for line in content.lines() {
            if !line.trim().is_empty() {
                match self.parse_entry_from_line(line) {
                    Ok(entry) => entries.push(entry),
                    Err(e) => {
                        eprintln!("Warning: Failed to parse audit entry: {e}");
                    }
                }
            }
        }
        
        Ok(entries)
    }
    
    fn read_entries_by_date_range(&self, start: &str, end: &str) -> QmsResult<Vec<AuditEntry>> {
        let all_entries = self.read_all_entries()?;
        
        let filtered_entries = all_entries.into_iter()
            .filter(|entry| {
                entry.timestamp.as_str() >= start && entry.timestamp.as_str() <= end
            })
            .collect();
        
        Ok(filtered_entries)
    }
    
    fn read_entries_by_user(&self, user_id: &str) -> QmsResult<Vec<AuditEntry>> {
        let all_entries = self.read_all_entries()?;
        
        let user_entries = all_entries.into_iter()
            .filter(|entry| entry.user_id == user_id)
            .collect();
        
        Ok(user_entries)
    }
    
    fn entry_exists(&self, entry_id: &str) -> QmsResult<bool> {
        match self.read_entry(entry_id) {
            Ok(_) => Ok(true),
            Err(QmsError::NotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

impl AuditWriter for FileAuditStorage {
    fn write_entry(&self, entry: &AuditEntry) -> QmsResult<()> {
        use std::fs::OpenOptions;
        use std::io::Write;
        
        let log_path = self.get_log_path();
        
        // Ensure directory exists
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;
        
        let json_line = format!("{}\n", entry.to_json());
        file.write_all(json_line.as_bytes())?;
        file.flush()?;
        
        Ok(())
    }
    
    fn write_entries_batch(&self, entries: &[AuditEntry]) -> QmsResult<()> {
        use std::fs::OpenOptions;
        use std::io::Write;
        
        let log_path = self.get_log_path();
        
        // Ensure directory exists
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;
        
        for entry in entries {
            let json_line = format!("{}\n", entry.to_json());
            file.write_all(json_line.as_bytes())?;
        }
        
        file.flush()?;
        Ok(())
    }
    
    fn append_with_integrity(&self, entry: &AuditEntry) -> QmsResult<()> {
        // For now, just use regular write
        // In a full implementation, this would include integrity chain verification
        self.write_entry(entry)
    }
    
    fn flush(&self) -> QmsResult<()> {
        // File operations are already flushed in write methods
        Ok(())
    }
}

impl AuditSearcher for FileAuditStorage {
    fn search(&self, criteria: &AuditSearchCriteria) -> QmsResult<Vec<AuditEntry>> {
        let mut entries = self.read_all_entries()?;
        
        // Apply filters
        if let Some(ref user_id) = criteria.user_id {
            entries.retain(|e| e.user_id == *user_id);
        }
        
        if let Some(ref action) = criteria.action {
            entries.retain(|e| e.action == *action);
        }
        
        if let Some(ref entity_type) = criteria.entity_type {
            entries.retain(|e| e.entity_type == *entity_type);
        }
        
        if let Some(ref entity_id) = criteria.entity_id {
            entries.retain(|e| e.entity_id == *entity_id);
        }
        
        if let Some(ref start_date) = criteria.start_date {
            entries.retain(|e| e.timestamp >= *start_date);
        }
        
        if let Some(ref end_date) = criteria.end_date {
            entries.retain(|e| e.timestamp <= *end_date);
        }
        
        if let Some(ref text_query) = criteria.text_query {
            let query_lower = text_query.to_lowercase();
            entries.retain(|e| {
                e.details.as_ref().is_some_and(|d| d.to_lowercase().contains(&query_lower)) ||
                e.entity_type.to_lowercase().contains(&query_lower) ||
                e.entity_id.to_lowercase().contains(&query_lower)
            });
        }
        
        // Apply limit
        if let Some(limit) = criteria.limit {
            entries.truncate(limit);
        }
        
        Ok(entries)
    }
    
    fn search_by_text(&self, query: &str) -> QmsResult<Vec<AuditEntry>> {
        let criteria = AuditSearchCriteria {
            user_id: None,
            action: None,
            entity_type: None,
            entity_id: None,
            start_date: None,
            end_date: None,
            text_query: Some(query.to_string()),
            limit: None,
        };
        
        self.search(&criteria)
    }
    
    fn search_by_action(&self, action: &crate::models::AuditAction) -> QmsResult<Vec<AuditEntry>> {
        let criteria = AuditSearchCriteria {
            user_id: None,
            action: Some(action.clone()),
            entity_type: None,
            entity_id: None,
            start_date: None,
            end_date: None,
            text_query: None,
            limit: None,
        };
        
        self.search(&criteria)
    }
    
    fn search_by_entity_type(&self, entity_type: &str) -> QmsResult<Vec<AuditEntry>> {
        let criteria = AuditSearchCriteria {
            user_id: None,
            action: None,
            entity_type: Some(entity_type.to_string()),
            entity_id: None,
            start_date: None,
            end_date: None,
            text_query: None,
            limit: None,
        };
        
        self.search(&criteria)
    }
    
    fn get_search_statistics(&self) -> QmsResult<SearchStatistics> {
        // Placeholder implementation
        Ok(SearchStatistics {
            total_searches: 0,
            average_search_time_ms: 0.0,
            most_common_queries: Vec::new(),
            search_success_rate: 100.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::models::AuditAction;

    #[test]
    fn test_file_audit_storage_read_write() {
        let temp_dir = tempdir().unwrap();
        let storage = FileAuditStorage::new(temp_dir.path());
        storage.initialize().unwrap();
        
        let entry = AuditEntry {
            id: "test-id".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            user_id: "test-user".to_string(),
            session_id: None,
            action: AuditAction::Create,
            entity_type: "Document".to_string(),
            entity_id: "DOC-001".to_string(),
            old_value: None,
            new_value: None,
            details: Some("Test entry".to_string()),
            ip_address: None,
            signature: None,
            checksum: "test-checksum".to_string(),
            previous_hash: None,
        };
        
        // Test write
        storage.write_entry(&entry).unwrap();
        
        // Test read
        let read_entry = storage.read_entry("test-id").unwrap();
        assert_eq!(read_entry.id, entry.id);
        assert_eq!(read_entry.user_id, entry.user_id);
        
        // Test search
        let criteria = AuditSearchCriteria {
            user_id: Some("test-user".to_string()),
            action: None,
            entity_type: None,
            entity_id: None,
            start_date: None,
            end_date: None,
            text_query: None,
            limit: None,
        };
        
        let search_results = storage.search(&criteria).unwrap();
        assert_eq!(search_results.len(), 1);
        assert_eq!(search_results[0].id, entry.id);
    }

    #[test]
    fn test_batch_write() {
        let temp_dir = tempdir().unwrap();
        let storage = FileAuditStorage::new(temp_dir.path());
        storage.initialize().unwrap();
        
        let mut entries = Vec::new();
        for i in 0..5 {
            entries.push(AuditEntry {
                id: format!("test-id-{}", i),
                timestamp: "2024-01-01T00:00:00Z".to_string(),
                user_id: "test-user".to_string(),
                session_id: None,
                action: AuditAction::Create,
                entity_type: "Document".to_string(),
                entity_id: format!("DOC-{:03}", i),
                old_value: None,
                new_value: None,
                details: Some(format!("Test entry {}", i)),
                ip_address: None,
                signature: None,
                checksum: "test-checksum".to_string(),
                previous_hash: None,
            });
        }
        
        storage.write_entries_batch(&entries).unwrap();
        
        let all_entries = storage.read_all_entries().unwrap();
        assert_eq!(all_entries.len(), 5);
    }
}
