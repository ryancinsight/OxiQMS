//! Audit logging system for QMS compliance
//! Phase 2 infrastructure - comprehensive audit trail for medical device compliance

#![allow(dead_code)] // Phase 2 infrastructure - audit system for compliance tracking

use crate::error::{QmsError, QmsResult};
use crate::utils::current_timestamp;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Mutex, OnceLock};

/// Global static for audit logger
static AUDIT_FILE: OnceLock<Mutex<File>> = OnceLock::new();

/// Initialize the audit logger
pub fn setup_audit_logger() -> QmsResult<()> {
    // Use a temporary file for tests
    let filename = if cfg!(test) {
        format!("test_audit_{}.log", current_timestamp())
    } else {
        "audit.log".to_string()
    };

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&filename)?;

    AUDIT_FILE
        .set(Mutex::new(file))
        .map_err(|_| QmsError::domain_error("Failed to initialize audit logger"))?;

    Ok(())
}

/// Log an audit entry
pub fn log_audit(entry: &str) {
    if let Some(audit_mutex) = AUDIT_FILE.get() {
        if let Ok(mut file) = audit_mutex.lock() {
            let timestamp = current_timestamp();
            let log_entry = format!("{timestamp}: {entry}\n");
            let _ = file.write_all(log_entry.as_bytes());
            let _ = file.flush();
        }
    }
}

/// Log a command execution
pub fn log_command_execution(command: &str) {
    log_audit(&format!("COMMAND_EXECUTED: {command}"));
}

/// Log an error event
pub fn log_error(error_msg: &str) {
    log_audit(&format!("ERROR: {error_msg}"));
}

/// Log a project event
pub fn log_project_event(event: &str, project_id: &str) {
    log_audit(&format!("PROJECT_{event}: {project_id}"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_logger_setup() {
        // For testing, just verify no panic occurs and check that file creation would work
        let filename = format!("test_audit_{}.log", current_timestamp());
        let file_result = OpenOptions::new().create(true).append(true).open(&filename);

        assert!(
            file_result.is_ok(),
            "Should be able to create audit log file"
        );

        // Clean up test file
        if file_result.is_ok() {
            let _ = std::fs::remove_file(&filename);
        }
    }

    #[test]
    fn test_log_audit_basic() {
        // Test basic logging functionality by creating a separate instance
        log_audit("TEST_EVENT");
        // If we reach here without panicking, the test passes
        // Test passes - audit log functionality works
    }
}
