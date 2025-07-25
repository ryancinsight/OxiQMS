//! Utility functions for QMS operations
//! Phase 2 infrastructure - timestamp formatting, ID validation, string utilities
//!
//! DRY and KISS Improvements: Consolidated utility modules

#![allow(dead_code)] // Phase 2 infrastructure - utility functions for data processing

use std::time::{Duration, SystemTime, UNIX_EPOCH};

// DRY and KISS Improvement Modules
pub mod common_validation;
pub mod error_handling;
pub mod config_helpers;
pub mod risk_calculator;
pub mod simple_calculations;
pub mod test_helpers;

// Re-export commonly used utilities for convenience
pub use common_validation::{CommonValidation, ValidationResult};
pub use error_handling::{ErrorHandler, ErrorRecovery};
pub use config_helpers::{ConfigHelper, ConfigPaths, DefaultConfigBuilder};
pub use risk_calculator::{RiskCalculator, RiskLevel as CalculatorRiskLevel};
pub use simple_calculations::{SimpleCalculations, SimpleMetrics, SimpleUtils, RiskLevel, TrendDirection};

#[cfg(test)]
pub use test_helpers::{TestHelper, TestValidation};

/// Format a Unix timestamp into a human-readable string
pub fn format_timestamp(timestamp: u64) -> String {
    let system_time = UNIX_EPOCH + Duration::from_secs(timestamp);
    match system_time.elapsed() {
        Ok(elapsed) => {
            let days = elapsed.as_secs() / 86400;
            if days == 0 {
                "today".to_string()
            } else if days == 1 {
                "yesterday".to_string()
            } else {
                format!("{days} days ago")
            }
        }
        Err(_) => {
            // Future timestamp or error, show raw timestamp
            format!("timestamp: {timestamp}")
        }
    }
}

/// Get current Unix timestamp
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Get current ISO 8601 timestamp string (for audit entries)
pub fn current_iso8601_timestamp() -> String {
    let timestamp = current_timestamp();
    
    // Convert to a simplified ISO 8601 format
    // This is a basic implementation without timezone support
    let days_since_epoch = timestamp / 86400;
    let year = 1970 + (days_since_epoch / 365);
    let day_of_year = days_since_epoch % 365;
    let month = (day_of_year / 31) + 1; // Simplified month calculation
    let day = (day_of_year % 31) + 1;
    
    let seconds_in_day = timestamp % 86400;
    let hour = seconds_in_day / 3600;
    let minute = (seconds_in_day % 3600) / 60;
    let second = seconds_in_day % 60;
    
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", 
            year, 
            month.min(12), 
            day.min(31), 
            hour, 
            minute, 
            second)
}

/// Get current date string in YYYY-MM-DD format
pub fn current_date_string() -> String {
    let timestamp = current_timestamp();
    let days_since_epoch = timestamp / 86400;
    let year = 1970 + (days_since_epoch / 365);
    let day_of_year = days_since_epoch % 365;
    let month = (day_of_year / 31) + 1; // Simplified month calculation
    let day = (day_of_year % 31) + 1;
    
    format!("{:04}-{:02}-{:02}", year, month.min(12), day.min(31))
}

/// Get current timestamp as ISO 8601 string
pub fn current_timestamp_string() -> String {
    let timestamp = current_timestamp();
    // Simple ISO-8601-like format using timestamp
    let system_time = UNIX_EPOCH + Duration::from_secs(timestamp);
    format!("{system_time:?}").replace("SystemTime { ", "").replace(" }", "")
}

/// Format a timestamp string back to a readable format
pub fn format_timestamp_from_string(timestamp_str: &str) -> String {
    // Try to parse the timestamp string back to a number
    if let Ok(timestamp) = timestamp_str.parse::<u64>() {
        format_timestamp(timestamp)
    } else {
        // If parsing fails, try to extract timestamp from SystemTime format
        if timestamp_str.contains("SystemTime") {
            timestamp_str.to_string()
        } else {
            // Fallback to original string
            timestamp_str.to_string()
        }
    }
}

/// Generate a simple UUID-like identifier
pub fn generate_uuid() -> String {
    let timestamp = current_timestamp();
    
    // Get additional entropy from system time and process info
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    
    // Use thread ID and process ID for more entropy
    let thread_id = std::thread::current().id();
    let process_id = std::process::id();
    
    // Create a more random UUID by combining multiple entropy sources
    let entropy1 = (timestamp ^ nanos as u64) % 0xFFFFFFFF;
    let entropy2 = (timestamp.wrapping_mul(31) ^ process_id as u64) % 0xFFFF;
    let entropy3 = (timestamp.wrapping_mul(37) ^ format!("{thread_id:?}").len() as u64) % 0xFFFF;
    let entropy4 = (nanos as u64).wrapping_mul(41) % 0xFFFF;
    let entropy5 = timestamp.wrapping_mul(43) % 0xFFFFFFFFFFFF;
    
    format!("{entropy1:08x}-{entropy2:04x}-{entropy3:04x}-{entropy4:04x}-{entropy5:012x}")
}

/// Validate string length
pub const fn validate_string_length(s: &str, max_len: usize) -> bool {
    !s.is_empty() && s.len() <= max_len
}

/// Validate range for numeric values
pub const fn validate_range(value: u8, min: u8, max: u8) -> bool {
    value >= min && value <= max
}

/// Validate ID format (prefix + specific pattern)
pub fn validate_id_format(id: &str, prefix: &str) -> bool {
    if !id.starts_with(prefix) {
        return false;
    }

    let suffix = &id[prefix.len()..];
    if suffix.len() < 2 {
        return false;
    }

    // Check if suffix contains only alphanumeric characters and hyphens
    suffix.chars().all(|c| c.is_alphanumeric() || c == '-')
}

/// Get the current project path from working directory or environment variable
/// Follows DIP: depends on abstraction (environment/filesystem) not concrete implementation
pub fn get_current_project_path() -> Result<std::path::PathBuf, crate::error::QmsError> {
    use std::env;

    // First check if QMS_PROJECT_PATH environment variable is set (DIP: dependency injection)
    if let Ok(project_path) = env::var("QMS_PROJECT_PATH") {
        let path = std::path::PathBuf::from(project_path);
        if path.exists() && path.join("project.json").exists() {
            return Ok(path);
        }
    }

    let current_dir = env::current_dir()
        .map_err(|_| crate::error::QmsError::io_error("Failed to get current directory"))?;

    // Look for project.json in current directory and parent directories
    let mut dir = current_dir.as_path();
    loop {
        let project_file = dir.join("project.json");
        if project_file.exists() {
            return Ok(dir.to_path_buf());
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }

    Err(crate::error::QmsError::not_found("No QMS project found. Use 'qms init' to create a project."))
}

/// Check if a QMS project exists without throwing an error
/// Used by web server to determine whether to show setup page or main application
pub fn qms_project_exists() -> bool {
    use std::env;

    // First check if QMS_PROJECT_PATH environment variable is set
    if let Ok(project_path) = env::var("QMS_PROJECT_PATH") {
        let path = std::path::PathBuf::from(project_path);
        if path.exists() && path.join("project.json").exists() {
            return true;
        }
    }

    // Check current directory and parent directories for project.json
    if let Ok(current_dir) = env::current_dir() {
        let mut dir = current_dir.as_path();
        loop {
            let project_file = dir.join("project.json");
            if project_file.exists() {
                return true;
            }

            match dir.parent() {
                Some(parent) => dir = parent,
                None => break,
            }
        }
    }

    false
}

/// Get the current project path if it exists, otherwise return None
/// Safe version that doesn't throw errors - used for conditional initialization
pub fn get_current_project_path_safe() -> Option<std::path::PathBuf> {
    if qms_project_exists() {
        get_current_project_path().ok()
    } else {
        None
    }
}

/// Calculate SHA-256 hash of a string for integrity verification
pub fn calculate_sha256(content: &str) -> String {
    use sha2::{Sha256, Digest};
    format!("{:x}", Sha256::digest(content.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_string_length() {
        assert!(validate_string_length("test", 10));
        assert!(!validate_string_length("", 10));
        assert!(!validate_string_length("too long string", 5));
    }

    #[test]
    fn test_validate_range() {
        assert!(validate_range(5, 1, 10));
        assert!(!validate_range(15, 1, 10));
        assert!(!validate_range(0, 1, 10));
    }

    #[test]
    fn test_validate_id_format() {
        assert!(validate_id_format("DOC-123", "DOC-"));
        assert!(validate_id_format("RISK-20250715-001", "RISK-"));
        assert!(!validate_id_format("INVALID", "DOC-"));
        assert!(!validate_id_format("DOC-", "DOC-"));
    }

    #[test]
    fn test_format_timestamp() {
        let now = current_timestamp();
        let formatted = format_timestamp(now);
        assert!(formatted.contains("today") || formatted.contains("timestamp"));
    }
}

/// User Context Management System
/// Provides centralized user context for CLI operations and modules
/// Replaces hardcoded user placeholders with proper authentication integration
pub mod user_context {
    use crate::prelude::*;
    use crate::modules::user_manager::{FileAuthManager};
    use crate::modules::user_manager::auth::UserSession;
    use std::path::PathBuf;
    use std::sync::{Mutex, Arc};
    use std::collections::HashMap;

    /// Global user context manager for CLI operations
    static USER_CONTEXT_MANAGER: std::sync::OnceLock<Arc<Mutex<UserContextManager>>> = std::sync::OnceLock::new();

    /// User context information for operations
    #[derive(Debug, Clone)]
    pub struct UserContext {
        pub user_id: String,
        pub username: String,
        pub session_id: Option<String>,
        pub roles: Vec<String>,
        pub permissions: Vec<String>,
        pub project_path: PathBuf,
    }

    /// User context manager for CLI and module operations
    pub struct UserContextManager {
        current_context: Option<UserContext>,
        session_cache: HashMap<String, UserSession>,
        project_path: PathBuf,
    }

    impl UserContextManager {
        /// Create new user context manager
        pub fn new(project_path: PathBuf) -> Self {
            Self {
                current_context: None,
                session_cache: HashMap::new(),
                project_path,
            }
        }

        /// Initialize global user context manager
        pub fn initialize(project_path: PathBuf) -> QmsResult<()> {
            let manager = Arc::new(Mutex::new(Self::new(project_path)));
            USER_CONTEXT_MANAGER.set(manager)
                .map_err(|_| QmsError::domain_error("User context manager already initialized"))?;
            Ok(())
        }

        /// Get global user context manager
        pub fn global() -> QmsResult<Arc<Mutex<UserContextManager>>> {
            USER_CONTEXT_MANAGER.get()
                .ok_or_else(|| QmsError::domain_error("User context manager not initialized"))
                .map(Arc::clone)
        }

        /// Set current user context from authentication
        pub fn set_user_context(&mut self, user_id: String, session: Option<UserSession>) -> QmsResult<()> {
            let username = if let Some(ref session) = session {
                session.username.clone()
            } else {
                user_id.clone()
            };

            let roles = if let Some(ref session) = session {
                session.roles.iter().map(|r| r.name.clone()).collect()
            } else {
                vec!["User".to_string()]
            };

            let permissions = if let Some(ref session) = session {
                session.roles.iter()
                    .flat_map(|role| &role.permissions)
                    .map(|perm| format!("{:?}", perm))
                    .collect()
            } else {
                vec![]
            };

            let context = UserContext {
                user_id: user_id.clone(),
                username,
                session_id: session.as_ref().map(|s| s.session_id.clone()),
                roles,
                permissions,
                project_path: self.project_path.clone(),
            };

            if let Some(session) = session {
                self.session_cache.insert(session.session_id.clone(), session);
            }

            self.current_context = Some(context);
            Ok(())
        }

        /// Get current user context
        pub fn get_current_context(&self) -> Option<&UserContext> {
            self.current_context.as_ref()
        }

        /// Clear current user context (logout)
        pub fn clear_context(&mut self) {
            if let Some(context) = &self.current_context {
                if let Some(session_id) = &context.session_id {
                    self.session_cache.remove(session_id);
                }
            }
            self.current_context = None;
        }

        /// Get current user ID or default
        pub fn get_current_user_id(&self) -> String {
            self.current_context
                .as_ref()
                .map(|ctx| ctx.user_id.clone())
                .unwrap_or_else(|| "system".to_string())
        }

        /// Get current username or default
        pub fn get_current_username(&self) -> String {
            self.current_context
                .as_ref()
                .map(|ctx| ctx.username.clone())
                .unwrap_or_else(|| "System User".to_string())
        }

        /// Check if user has permission
        pub fn has_permission(&self, permission: &str) -> bool {
            self.current_context
                .as_ref()
                .map(|ctx| ctx.permissions.contains(&permission.to_string()))
                .unwrap_or(false)
        }
    }

    /// Convenience functions for getting current user context

    /// Get current user ID for operations
    pub fn get_current_user_id() -> String {
        if let Ok(manager) = UserContextManager::global() {
            if let Ok(manager) = manager.lock() {
                return manager.get_current_user_id();
            }
        }
        "system".to_string()
    }

    /// Get current username for operations
    pub fn get_current_username() -> String {
        if let Ok(manager) = UserContextManager::global() {
            if let Ok(manager) = manager.lock() {
                return manager.get_current_username();
            }
        }
        "System User".to_string()
    }

    /// Get current user context
    pub fn get_current_context() -> Option<UserContext> {
        if let Ok(manager) = UserContextManager::global() {
            if let Ok(manager) = manager.lock() {
                return manager.get_current_context().cloned();
            }
        }
        None
    }

    /// Initialize user context from CLI login
    pub fn initialize_cli_context(project_path: PathBuf, username: Option<String>) -> QmsResult<()> {
        UserContextManager::initialize(project_path.clone())?;

        if let Some(username) = username {
            // Try to authenticate user
            let auth_manager = FileAuthManager::from_project_path(&project_path)?;

            // For CLI, we'll create a basic context without full authentication
            // In a production system, this would require password authentication
            let manager = UserContextManager::global()?;
            let mut manager = manager.lock().unwrap();
            manager.set_user_context(username, None)?;
        }

        Ok(())
    }

    /// Check if user has permission
    pub fn has_permission(permission: &str) -> bool {
        if let Ok(manager) = UserContextManager::global() {
            if let Ok(manager) = manager.lock() {
                return manager.has_permission(permission);
            }
        }
        false
    }

    /// Get current project ID from project path
    pub fn get_current_project_id() -> String {
        if let Ok(project_path) = crate::utils::get_current_project_path() {
            // Extract project ID from path (last directory component)
            if let Some(project_id) = project_path.file_name() {
                if let Some(id_str) = project_id.to_str() {
                    return id_str.to_string();
                }
            }

            // Fallback: try to read from project.json
            let project_file = project_path.join("project.json");
            if let Ok(content) = std::fs::read_to_string(project_file) {
                if let Ok(json) = crate::json_utils::JsonValue::parse(&content) {
                    if let crate::json_utils::JsonValue::Object(obj) = json {
                        if let Some(data) = obj.get("data") {
                            if let crate::json_utils::JsonValue::Object(data_obj) = data {
                                if let Some(crate::json_utils::JsonValue::String(id)) = data_obj.get("id") {
                                    return id.clone();
                                }
                            }
                        }
                    }
                }
            }
        }

        // Ultimate fallback
        "unknown_project".to_string()
    }

    /// Get current project name from project metadata
    pub fn get_current_project_name() -> String {
        if let Ok(project_path) = crate::utils::get_current_project_path() {
            let project_file = project_path.join("project.json");
            if let Ok(content) = std::fs::read_to_string(project_file) {
                if let Ok(json) = crate::json_utils::JsonValue::parse(&content) {
                    if let crate::json_utils::JsonValue::Object(obj) = json {
                        if let Some(data) = obj.get("data") {
                            if let crate::json_utils::JsonValue::Object(data_obj) = data {
                                if let Some(crate::json_utils::JsonValue::String(name)) = data_obj.get("name") {
                                    return name.clone();
                                }
                            }
                        }
                    }
                }
            }
        }

        "Unknown Project".to_string()
    }
}
