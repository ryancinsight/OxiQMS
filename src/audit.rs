//! Comprehensive audit logging and tracing system for QMS compliance
//! Phase 2 infrastructure - FDA 21 CFR Part 820 and ISO 13485 compliant audit trails

#![allow(dead_code)] // Phase 2 infrastructure - audit system for compliance tracking

use crate::config::LoggingConfig;
use crate::error::{QmsError, QmsResult};
use crate::utils::current_timestamp;
use std::fs;
use std::io::Write;
use std::sync::{Mutex, OnceLock};
use tracing::{debug, error, info, warn};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// Global static for the non-blocking writer guard
static GUARD: OnceLock<Mutex<Option<tracing_appender::non_blocking::WorkerGuard>>> = OnceLock::new();

/// Initialize comprehensive tracing system with file rotation and audit compliance
pub fn init_tracing(config: &LoggingConfig) -> QmsResult<tracing_appender::non_blocking::WorkerGuard> {
    // Create log directory if it doesn't exist
    let log_dir = config.log_dir();
    fs::create_dir_all(&log_dir)
        .map_err(|e| QmsError::io_error(&format!("Failed to create log directory: {e}")))?;

    // Set up the logging filter based on configuration
    let filter = EnvFilter::new(&config.level);

    // Create file appender with rotation for compliance
    let file_appender = rolling::Builder::new()
        .rotation(rolling::Rotation::HOURLY) // Hourly rotation for detailed audit trails
        .filename_prefix("qms-audit")
        .filename_suffix("log")
        .max_log_files(config.max_files)
        .build(&log_dir)
        .map_err(|e| QmsError::io_error(&format!("Failed to create file appender: {e}")))?;

    let (non_blocking, guard) = non_blocking(file_appender);

    // Create the file layer
    let file_layer = if config.json_format {
        // JSON format for structured audit trails (FDA compliance)
        fmt::layer()
            .json()
            .with_writer(non_blocking.clone())
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .boxed()
    } else {
        // Human-readable format
        fmt::layer()
            .with_writer(non_blocking.clone())
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_file(true)
            .with_line_number(true)
            .boxed()
    };

    // Create console layer if enabled
    let console_layer = if config.console_logging {
        Some(
            fmt::layer()
                .with_writer(std::io::stderr)
                .with_target(false)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_file(false)
                .with_line_number(false)
                .boxed()
        )
    } else {
        None
    };

    // Initialize the subscriber
    match console_layer {
        Some(console) => {
            tracing_subscriber::registry()
                .with(filter)
                .with(file_layer)
                .with(console)
                .try_init()
                .map_err(|e| QmsError::domain_error(&format!("Failed to initialize tracing: {e}")))?;
        }
        None => {
            tracing_subscriber::registry()
                .with(filter)
                .with(file_layer)
                .try_init()
                .map_err(|e| QmsError::domain_error(&format!("Failed to initialize tracing: {e}")))?;
        }
    }

    // Store the guard globally to prevent it from being dropped
    GUARD
        .set(Mutex::new(Some(guard)))
        .map_err(|_| QmsError::domain_error("Failed to store tracing guard"))?;

    info!(
        config = ?config,
        "Tracing initialized with FDA-compliant audit logging"
    );

    // Return a guard that must be kept alive
    Ok(create_dummy_guard())
}

/// Create a dummy guard that keeps the logging active
pub fn create_dummy_guard() -> tracing_appender::non_blocking::WorkerGuard {
    // Create a temporary file appender to generate a guard
    let temp_appender = rolling::Builder::new()
        .rotation(rolling::Rotation::NEVER)
        .filename_prefix("temp")
        .filename_suffix("log")
        .build(std::env::temp_dir())
        .expect("Failed to create temporary appender");
    
    let (_writer, guard) = non_blocking(temp_appender);
    guard
}

/// Legacy audit logging function for backward compatibility
pub fn log_audit(entry: &str) {
    info!(event = "AUDIT", message = entry, "Legacy audit log entry");
}

/// Log a command execution with structured data
pub fn log_command_execution(command: &str) {
    info!(
        event = "COMMAND_EXECUTED",
        command = command,
        timestamp = current_timestamp(),
        "Command executed"
    );
}

/// Log an error event with structured data
pub fn log_error(error_msg: &str) {
    error!(
        event = "ERROR",
        error = error_msg,
        timestamp = current_timestamp(),
        "Error occurred"
    );
}

/// Log a project event with structured data
pub fn log_project_event(event: &str, project_id: &str) {
    info!(
        event = "PROJECT_EVENT",
        project_event = event,
        project_id = project_id,
        timestamp = current_timestamp(),
        "Project event"
    );
}

/// Setup legacy audit logger for backward compatibility
pub fn setup_audit_logger() -> QmsResult<()> {
    // Use default logging configuration for legacy compatibility
    let config = LoggingConfig::default();
    let _guard = init_tracing(&config)?;
    
    info!("Legacy audit logger setup completed");
    Ok(())
}

/// Log user action for audit trail
pub fn log_user_action(user_id: &str, action: &str, resource: &str, outcome: &str) {
    info!(
        event = "USER_ACTION",
        user_id = user_id,
        action = action,
        resource = resource,
        outcome = outcome,
        timestamp = current_timestamp(),
        "User action audit log"
    );
}

/// Log system event for compliance monitoring
pub fn log_system_event(event_type: &str, component: &str, details: &str) {
    info!(
        event = "SYSTEM_EVENT",
        event_type = event_type,
        component = component,
        details = details,
        timestamp = current_timestamp(),
        "System event audit log"
    );
}

/// Log data integrity check for FDA compliance
pub fn log_integrity_check(entity_type: &str, entity_id: &str, checksum: &str, status: &str) {
    info!(
        event = "INTEGRITY_CHECK",
        entity_type = entity_type,
        entity_id = entity_id,
        checksum = checksum,
        status = status,
        timestamp = current_timestamp(),
        "Data integrity verification"
    );
}

/// Log regulatory compliance event
pub fn log_compliance_event(regulation: &str, requirement: &str, status: &str, details: &str) {
    info!(
        event = "COMPLIANCE_EVENT",
        regulation = regulation,
        requirement = requirement,
        status = status,
        details = details,
        timestamp = current_timestamp(),
        "Regulatory compliance audit log"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_logging_config_validation() {
        let mut config = LoggingConfig::default();
        assert!(config.validate().is_ok());

        config.level = "INVALID".to_string();
        assert!(config.validate().is_err());

        config.level = "INFO".to_string();
        config.max_file_size = 0;
        assert!(config.validate().is_err());

        config.max_file_size = 1024;
        config.max_files = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_legacy_audit_logging() {
        // Test legacy functions don't panic
        log_audit("TEST_AUDIT");
        log_command_execution("test_command");
        log_error("test_error");
        log_project_event("TEST", "proj_123");
    }

    #[test]
    fn test_structured_logging_functions() {
        // Test structured logging functions
        log_user_action("user123", "CREATE", "document", "SUCCESS");
        log_system_event("STARTUP", "audit_system", "System initialized");
        log_integrity_check("document", "doc123", "abc123", "VALID");
        log_compliance_event("FDA_21_CFR_820", "Part_820.40", "COMPLIANT", "Document control verified");
    }

    #[test]
    fn test_fda_compliant_config() {
        let config = LoggingConfig::new_fda_compliant();
        assert!(!config.console_logging); // File-only for compliance
        assert!(config.json_format); // Structured format required
        assert!(config.audit_logging);
        assert_eq!(config.max_files, 50); // Keep more files for compliance
    }
}
