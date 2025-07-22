pub mod entry;
pub mod export;
pub mod functions;
pub mod integrity;
pub mod regulatory;
pub mod rotation;
pub mod search;
pub mod dashboard;
pub mod signatures;
pub mod backup;
pub mod performance;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod test_log_writing_export;

// Re-export main functionality for when it's used
#[allow(unused_imports)]
pub use entry::{
    AuditConfig, AuditStats, AuditEntryBuilder,
    initialize_audit_logger, log_audit_entry, log_action, 
    log_update, log_with_signature, get_audit_statistics
};

// Re-export core audit functions
#[allow(unused_imports)]
pub use functions::{
    AuditSession,
    initialize_audit_system, set_current_session, clear_current_session, get_current_session,
    log_user_action, log_create_operation, log_read_operation, log_update_operation,
    log_delete_operation, log_approval_operation, log_system_event, log_bulk_operation,
    buffer_audit_entry, flush_audit_buffer, search_audit_logs,
    rotate_audit_logs, cleanup_old_audit_logs,
    // Convenience wrapper functions for CRUD operations (task 2.2.2)
    audit_log_create, audit_log_read, audit_log_update, audit_log_delete, audit_log_action
};

// Re-export integrity functions
#[allow(unused_imports)]
pub use integrity::{
    ChainVerificationResult, ChainBreak,
    get_last_entry_hash, append_audit_entry_with_chain, verify_audit_chain,
    verify_audit_file, initialize_audit_chain, export_chain_verification_report
};

// Re-export rotation functions
#[allow(unused_imports)]
pub use rotation::{
    RotationConfig, CleanupReport, RotationStats,
    check_and_rotate_daily_logs, rotate_to_daily_log, cleanup_old_logs_comprehensive,
    get_rotation_statistics, schedule_daily_rotation
};

// Re-export search functions
#[allow(unused_imports)]
pub use search::{
    AuditSearchCriteria, AuditSearchResults, AuditOutputFormat, AuditSearchEngine,
    AuditStatistics, format_search_results, parse_date_to_timestamp
};

// Re-export regulatory compliance functions
#[allow(unused_imports)]
pub use regulatory::{
    RegulatoryCompliance, ComplianceReport, ComplianceValidation, ComplianceIssue,
    ComplianceIssueType, IssueSeverity, ComplianceSummary, AuditTrailSummary,
    format_compliance_report
};

// Re-export dashboard functions
#[allow(unused_imports)]
pub use dashboard::{
    AuditDashboard, AuditDashboardEngine, GeneralMetrics, UserActivityMetrics,
    ActionMetrics, TimeAnalysis, SecurityAlert, TrendAnalysis, format_dashboard
};

// Re-export electronic signature functions
#[allow(unused_imports)]
pub use signatures::{
    ElectronicSignature, ElectronicSignatureManager, SignatureMethod, SignaturePolicy,
    SignatureVerification, SignatureRequirements, format_signature_verification
};

// Re-export export functions
#[allow(unused_imports)]
pub use export::{
    ExportFormat, ExportOptions, ExportStats, AuditExportEngine, format_export_stats
};

// Re-export backup functions
#[allow(unused_imports)]
pub use backup::{
    BackupConfig, BackupStats, BackupInfo, AuditBackupManager, format_backup_stats, format_backup_info
};

// Re-export performance functions
#[allow(unused_imports)]
pub use performance::{
    PerformanceConfig, PerformanceMetrics, PerformanceAuditLogger, format_performance_metrics
};

// SOLID Principle Enhancement Modules
pub mod observer_pattern;
pub mod command_pattern;
pub mod segregated_interfaces;
pub mod error_hierarchy;
pub mod enhanced_logger;

// SOLID Principle Enhancement Exports
pub use observer_pattern::{
    AuditEventObserver, AuditEventSubject, AuditEvent, AuditEventType, EventSeverity,
    SecurityAlertObserver, ComplianceMonitorObserver, PerformanceMonitorObserver
};

pub use command_pattern::{
    AuditCommand, AuditCommandType, AuditCommandResult, AuditCommandInvoker,
    CreateAuditEntryCommand, BatchAuditCommand, CommandStatistics
};

pub use segregated_interfaces::{
    AuditReader, AuditWriter, AuditSearcher, AuditFormatter, AuditValidator,
    AuditArchiver, AuditMetricsCollector, FileAuditStorage,
    AuditSearchCriteria as EnhancedAuditSearchCriteria, ValidationResult
};

pub use error_hierarchy::{
    AuditErrorHelper
};

pub use enhanced_logger::{
    EnhancedAuditLogger, AuditLoggerConfig, AuditLoggerStatistics
};
