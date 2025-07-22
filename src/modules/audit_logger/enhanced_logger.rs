//! Enhanced Audit Logger with SOLID Principles
//! 
//! This module implements an enhanced audit logger that integrates all SOLID principle
//! improvements including Observer pattern, Command pattern, Interface Segregation,
//! and Dependency Inversion.
//! 
//! SOLID Principles Applied:
//! - Single Responsibility: Each component has one clear responsibility
//! - Open/Closed: Extensible through strategy and observer implementations
//! - Liskov Substitution: All implementations can be substituted
//! - Interface Segregation: Uses focused interfaces for different operations
//! - Dependency Inversion: Depends on abstractions, not concrete implementations

use crate::prelude::*;
use crate::models::AuditEntry;
use super::{
    AuditEventSubject, AuditEvent, AuditEventType, EventSeverity,
    AuditCommandInvoker, CreateAuditEntryCommand,
    AuditReader, AuditWriter, AuditSearcher, AuditValidator,
    AuditErrorHelper, FileAuditStorage,
    EnhancedAuditSearchCriteria, ValidationResult
};
use std::sync::{Arc, Mutex};
use std::path::Path;

/// Configuration for the enhanced audit logger
#[derive(Debug, Clone)]
pub struct AuditLoggerConfig {
    pub storage_path: std::path::PathBuf,
    pub enable_observers: bool,
    pub enable_command_history: bool,
    pub max_command_history: usize,
    pub buffer_size: usize,
    pub enable_integrity_checks: bool,
    pub enable_compliance_monitoring: bool,
    pub performance_monitoring: bool,
}

impl Default for AuditLoggerConfig {
    fn default() -> Self {
        Self {
            storage_path: std::path::PathBuf::from("audit"),
            enable_observers: true,
            enable_command_history: true,
            max_command_history: 1000,
            buffer_size: 100,
            enable_integrity_checks: true,
            enable_compliance_monitoring: true,
            performance_monitoring: true,
        }
    }
}

/// Statistics for the enhanced audit logger
#[derive(Debug, Clone)]
pub struct AuditLoggerStatistics {
    pub total_entries: u64,
    pub entries_today: u64,
    pub observer_count: usize,
    pub command_history_size: usize,
    pub average_write_time_ms: f64,
    pub compliance_score: f64,
    pub integrity_status: String,
    pub last_error: Option<String>,
}

/// Enhanced Audit Logger using Dependency Inversion Principle
/// Depends on abstractions (traits) rather than concrete implementations
pub struct EnhancedAuditLogger {
    // Dependency Inversion: Inject storage dependencies
    reader: Arc<dyn AuditReader + Send + Sync>,
    writer: Arc<dyn AuditWriter + Send + Sync>,
    searcher: Arc<dyn AuditSearcher + Send + Sync>,
    validator: Option<Arc<dyn AuditValidator + Send + Sync>>,
    
    // Observer pattern for event notifications
    event_subject: Arc<AuditEventSubject>,
    
    // Command pattern for operation history
    command_invoker: Arc<Mutex<AuditCommandInvoker>>,
    
    // Configuration
    config: AuditLoggerConfig,
    
    // Statistics tracking
    statistics: Arc<Mutex<AuditLoggerStatistics>>,
}

impl EnhancedAuditLogger {
    /// Create new enhanced audit logger with dependency injection
    /// Dependency Inversion Principle: Accept abstractions as parameters
    pub fn new_with_dependencies(
        config: AuditLoggerConfig,
        reader: Arc<dyn AuditReader + Send + Sync>,
        writer: Arc<dyn AuditWriter + Send + Sync>,
        searcher: Arc<dyn AuditSearcher + Send + Sync>,
        validator: Option<Arc<dyn AuditValidator + Send + Sync>>,
    ) -> Self {
        let event_subject = Arc::new(AuditEventSubject::new());
        let command_invoker = Arc::new(Mutex::new(AuditCommandInvoker::new(config.max_command_history)));
        
        let statistics = Arc::new(Mutex::new(AuditLoggerStatistics {
            total_entries: 0,
            entries_today: 0,
            observer_count: 0,
            command_history_size: 0,
            average_write_time_ms: 0.0,
            compliance_score: 100.0,
            integrity_status: "OK".to_string(),
            last_error: None,
        }));
        
        Self {
            reader,
            writer,
            searcher,
            validator,
            event_subject,
            command_invoker,
            config,
            statistics,
        }
    }
    
    /// Create new enhanced audit logger with default file-based storage
    /// Factory method for common use case
    pub fn new_with_file_storage(config: AuditLoggerConfig) -> QmsResult<Self> {
        let storage = Arc::new(FileAuditStorage::new(&config.storage_path));
        
        // Initialize storage (we know it's FileAuditStorage since we created it)
        // Create a temporary FileAuditStorage to initialize
        let temp_storage = FileAuditStorage::new(&config.storage_path);
        temp_storage.initialize()?;
        
        Ok(Self::new_with_dependencies(
            config,
            storage.clone(),
            storage.clone(),
            storage.clone(),
            None, // No validator by default
        ))
    }
    
    /// Log audit entry with enhanced features
    /// Single Responsibility Principle: Focused on audit entry logging with enhancements
    pub fn log_entry(&self, entry: &AuditEntry) -> QmsResult<()> {
        let start_time = crate::utils::current_timestamp();
        
        // Create audit event for observers
        let audit_event = AuditEvent::new(AuditEventType::EntryCreated, entry.clone())
            .with_severity(EventSeverity::Medium);
        
        // Use command pattern if enabled
        if self.config.enable_command_history {
            let command = Box::new(CreateAuditEntryCommand::new(
                entry.clone(),
                self.config.storage_path.join("audit.log")
            ));
            
            let command_invoker = self.command_invoker.lock()
                .map_err(|_| QmsError::domain_error("Failed to acquire command invoker lock"))?;
            
            let result = command_invoker.execute_command(command)?;
            
            if !result.success {
                return Err(QmsError::domain_error(&result.message));
            }
        } else {
            // Direct write if command pattern is disabled
            self.writer.write_entry(entry)?;
        }
        
        // Notify observers if enabled
        if self.config.enable_observers {
            if let Err(e) = self.event_subject.notify_observers(&audit_event) {
                // Log observer errors but don't fail the main operation
                eprintln!("Observer notification failed: {}", e);
            }
        }
        
        // Update statistics
        let end_time = crate::utils::current_timestamp();
        let execution_time = end_time - start_time;
        
        if let Ok(mut stats) = self.statistics.lock() {
            stats.total_entries += 1;
            
            // Update average write time (simple moving average)
            if stats.average_write_time_ms == 0.0 {
                stats.average_write_time_ms = execution_time as f64;
            } else {
                stats.average_write_time_ms = (stats.average_write_time_ms * 0.9) + (execution_time as f64 * 0.1);
            }
        }
        
        Ok(())
    }
    
    /// Log multiple entries in batch
    /// Single Responsibility Principle: Focused on batch logging
    pub fn log_entries_batch(&self, entries: &[AuditEntry]) -> QmsResult<()> {
        if entries.is_empty() {
            return Ok(());
        }
        
        // Use batch write for efficiency
        self.writer.write_entries_batch(entries)?;
        
        // Notify observers for each entry if enabled
        if self.config.enable_observers {
            for entry in entries {
                let audit_event = AuditEvent::new(AuditEventType::EntryCreated, entry.clone())
                    .with_severity(EventSeverity::Low); // Lower severity for batch operations
                
                if let Err(e) = self.event_subject.notify_observers(&audit_event) {
                    eprintln!("Observer notification failed for batch entry {}: {}", entry.id, e);
                }
            }
        }
        
        // Update statistics
        if let Ok(mut stats) = self.statistics.lock() {
            stats.total_entries += entries.len() as u64;
        }
        
        Ok(())
    }
    
    /// Search audit entries using injected searcher
    /// Interface Segregation Principle: Uses focused searcher interface
    pub fn search_entries(&self, criteria: &EnhancedAuditSearchCriteria) -> QmsResult<Vec<AuditEntry>> {
        self.searcher.search(criteria)
    }
    
    /// Read audit entry by ID using injected reader
    pub fn read_entry(&self, entry_id: &str) -> QmsResult<AuditEntry> {
        self.reader.read_entry(entry_id)
    }
    
    /// Read all audit entries using injected reader
    pub fn read_all_entries(&self) -> QmsResult<Vec<AuditEntry>> {
        self.reader.read_all_entries()
    }
    
    /// Register an observer for audit events
    /// Open/Closed Principle: Can add new observers without modification
    pub fn register_observer(&self, observer: Arc<dyn super::AuditEventObserver>) -> QmsResult<()> {
        self.event_subject.register_observer(observer)?;
        
        // Update statistics
        if let Ok(mut stats) = self.statistics.lock() {
            stats.observer_count = self.event_subject.observer_count().unwrap_or(0);
        }
        
        Ok(())
    }
    
    /// Unregister an observer
    pub fn unregister_observer(&self, observer_name: &str) -> QmsResult<bool> {
        let result = self.event_subject.unregister_observer(observer_name)?;
        
        // Update statistics
        if let Ok(mut stats) = self.statistics.lock() {
            stats.observer_count = self.event_subject.observer_count().unwrap_or(0);
        }
        
        Ok(result)
    }
    
    /// Undo last audit operation (if command pattern is enabled)
    pub fn undo_last_operation(&self) -> QmsResult<String> {
        if !self.config.enable_command_history {
            return Err(QmsError::domain_error("Command history is disabled"));
        }
        
        let command_invoker = self.command_invoker.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire command invoker lock"))?;
        
        let result = command_invoker.undo_last_command()?;
        
        // Update statistics
        if let Ok(mut stats) = self.statistics.lock() {
            stats.command_history_size = command_invoker.get_history_summary().unwrap_or_default().len();
        }
        
        Ok(result.message)
    }
    
    /// Redo last undone operation
    pub fn redo_last_operation(&self) -> QmsResult<String> {
        if !self.config.enable_command_history {
            return Err(QmsError::domain_error("Command history is disabled"));
        }
        
        let command_invoker = self.command_invoker.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire command invoker lock"))?;
        
        let result = command_invoker.redo_last_command()?;
        
        // Update statistics
        if let Ok(mut stats) = self.statistics.lock() {
            stats.command_history_size = command_invoker.get_history_summary().unwrap_or_default().len();
        }
        
        Ok(result.message)
    }
    
    /// Get command history summary
    pub fn get_command_history(&self) -> QmsResult<Vec<String>> {
        if !self.config.enable_command_history {
            return Ok(Vec::new());
        }
        
        let command_invoker = self.command_invoker.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire command invoker lock"))?;
        
        command_invoker.get_history_summary()
    }
    
    /// Validate audit entries using injected validator
    pub fn validate_entries(&self, entries: &[AuditEntry]) -> QmsResult<ValidationResult> {
        if let Some(ref validator) = self.validator {
            // Validate each entry and aggregate results
            let mut all_valid = true;
            let mut all_errors = Vec::new();
            let mut all_warnings = Vec::new();
            
            for entry in entries {
                let result = validator.validate_entry(entry)?;
                if !result.is_valid {
                    all_valid = false;
                }
                all_errors.extend(result.errors);
                all_warnings.extend(result.warnings);
            }
            
            Ok(ValidationResult {
                is_valid: all_valid,
                errors: all_errors,
                warnings: all_warnings,
            })
        } else {
            // No validator configured - assume valid
            Ok(ValidationResult {
                is_valid: true,
                errors: Vec::new(),
                warnings: vec!["No validator configured".to_string()],
            })
        }
    }
    
    /// Get audit logger statistics
    pub fn get_statistics(&self) -> QmsResult<AuditLoggerStatistics> {
        let stats = self.statistics.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire statistics lock"))?;
        
        Ok(stats.clone())
    }
    
    /// Handle audit error with context (REFACTORED: KISS principle)
    /// Simplified error handling without complex error hierarchies
    pub fn handle_error(&self, error: QmsError, operation: &str) -> QmsError {
        let error_with_context = crate::utils::ErrorHandler::with_operation(error, operation);

        // Update statistics with error information
        if let Ok(mut stats) = self.statistics.lock() {
            stats.last_error = Some(error_with_context.to_string());

            // Simplified compliance score adjustment based on error type
            match error_with_context {
                QmsError::Permission(_) | QmsError::Authentication(_) => {
                    stats.compliance_score = (stats.compliance_score - 10.0).max(0.0);
                }
                QmsError::Validation(_) | QmsError::Domain(_) => {
                    stats.compliance_score = (stats.compliance_score - 5.0).max(0.0);
                }
                _ => {
                    stats.compliance_score = (stats.compliance_score - 2.0).max(0.0);
                }
            }
        }

        // Create security alert event for critical errors (simplified)
        if matches!(error_with_context, QmsError::Permission(_) | QmsError::Authentication(_)) {
            let alert_entry = AuditEntry {
                id: crate::utils::generate_uuid(),
                timestamp: crate::utils::current_iso8601_timestamp(),
                user_id: "system".to_string(),
                session_id: None,
                action: crate::models::AuditAction::Update,
                entity_type: "AuditSystem".to_string(),
                entity_id: "error-handler".to_string(),
                old_value: None,
                new_value: None,
                details: Some(error_with_context.to_string()),
                ip_address: None,
                signature: None,
                checksum: String::new(),
                previous_hash: None,
            };
            
            let alert_event = AuditEvent::new(AuditEventType::SecurityAlert, alert_entry)
                .with_severity(EventSeverity::Critical);
            
            if let Err(e) = self.event_subject.notify_observers(&alert_event) {
                eprintln!("Failed to notify observers about critical error: {}", e);
            }
        }

        error_with_context
    }
    
    /// Flush any buffered operations
    pub fn flush(&self) -> QmsResult<()> {
        self.writer.flush()
    }
    
    /// Get configuration
    pub fn get_config(&self) -> &AuditLoggerConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::models::AuditAction;

    #[test]
    fn test_enhanced_audit_logger_creation() {
        let temp_dir = tempdir().unwrap();
        let config = AuditLoggerConfig {
            storage_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let logger = EnhancedAuditLogger::new_with_file_storage(config).unwrap();
        let stats = logger.get_statistics().unwrap();
        
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.observer_count, 0);
    }

    #[test]
    fn test_audit_entry_logging() {
        let temp_dir = tempdir().unwrap();
        let config = AuditLoggerConfig {
            storage_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let logger = EnhancedAuditLogger::new_with_file_storage(config).unwrap();
        
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
        
        logger.log_entry(&entry).unwrap();
        
        let stats = logger.get_statistics().unwrap();
        assert_eq!(stats.total_entries, 1);
        
        let read_entry = logger.read_entry("test-id").unwrap();
        assert_eq!(read_entry.id, entry.id);
    }

    #[test]
    fn test_observer_registration() {
        let temp_dir = tempdir().unwrap();
        let config = AuditLoggerConfig {
            storage_path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let logger = EnhancedAuditLogger::new_with_file_storage(config).unwrap();
        let observer = Arc::new(crate::modules::audit_logger::PerformanceMonitorObserver::new());
        
        logger.register_observer(observer).unwrap();
        
        let stats = logger.get_statistics().unwrap();
        assert_eq!(stats.observer_count, 1);
    }

    #[test]
    fn test_command_history() {
        let temp_dir = tempdir().unwrap();
        let config = AuditLoggerConfig {
            storage_path: temp_dir.path().to_path_buf(),
            enable_command_history: true,
            ..Default::default()
        };
        
        let logger = EnhancedAuditLogger::new_with_file_storage(config).unwrap();
        
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
        
        logger.log_entry(&entry).unwrap();
        
        let history = logger.get_command_history().unwrap();
        assert_eq!(history.len(), 1);
        
        // Test undo
        let undo_result = logger.undo_last_operation().unwrap();
        // The undo operation should succeed and return a message about the compensating entry
        assert!(undo_result.contains("Compensating") || undo_result.contains("undone") || undo_result.contains("Created"));
    }
}
