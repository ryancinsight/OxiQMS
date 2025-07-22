/// Simplified Error Handling Utilities (KISS Principle)
///
/// REFACTORED: Simplified error handling following KISS principle
/// Reduces complexity while maintaining medical device compliance requirements

use crate::prelude::*;
use std::fmt;

/// Common error handling patterns used throughout the QMS system
pub struct ErrorHandler;

impl ErrorHandler {
    /// Create validation error with context (DRY: used everywhere)
    pub fn validation_error(message: &str) -> QmsError {
        QmsError::validation_error(message)
    }
    
    /// Create validation error with field context (DRY: common pattern)
    pub fn field_validation_error(field: &str, message: &str) -> QmsError {
        QmsError::validation_error(&format!("Field '{}': {}", field, message))
    }
    
    /// Create not found error with entity context (DRY: common pattern)
    pub fn not_found_error(entity_type: &str, id: &str) -> QmsError {
        QmsError::not_found(&format!("{} with ID '{}' not found", entity_type, id))
    }
    
    /// Create already exists error with entity context (DRY: common pattern)
    pub fn already_exists_error(entity_type: &str, id: &str) -> QmsError {
        QmsError::already_exists(&format!("{} with ID '{}' already exists", entity_type, id))
    }
    
    /// Create permission error with action context (DRY: common pattern)
    pub fn permission_error(action: &str, resource: &str) -> QmsError {
        QmsError::permission_error(&format!("Permission denied: cannot {} {}", action, resource))
    }
    
    /// Create invalid operation error with context (DRY: common pattern)
    pub fn invalid_operation_error(operation: &str, reason: &str) -> QmsError {
        QmsError::invalid_operation(&format!("Invalid operation '{}': {}", operation, reason))
    }
    
    /// Wrap IO errors with context (DRY: file operations)
    pub fn io_error_with_context(error: std::io::Error, context: &str) -> QmsError {
        QmsError::io_error(&format!("IO error in {}: {}", context, error))
    }
    
    /// Create parse error with context (DRY: parsing operations)
    pub fn parse_error_with_context(message: &str, context: &str) -> QmsError {
        QmsError::parse_error(&format!("Parse error in {}: {}", context, message))
    }
    
    /// Handle result with context logging (DRY: common error handling pattern)
    pub fn handle_result_with_context<T>(
        result: QmsResult<T>,
        context: &str,
        log_errors: bool,
    ) -> QmsResult<T> {
        match result {
            Ok(value) => Ok(value),
            Err(error) => {
                if log_errors {
                    eprintln!("Error in {}: {}", context, error);
                }
                Err(error)
            }
        }
    }
    
    /// Convert validation result to QmsResult (DRY: validation to result conversion)
    pub fn validation_to_result(validation: &crate::utils::common_validation::ValidationResult) -> QmsResult<()> {
        if validation.is_valid {
            Ok(())
        } else {
            let error_message = validation.errors.join("; ");
            Err(Self::validation_error(&error_message))
        }
    }
    
    /// Add context to any error (REFACTORED: Simplified from complex error chain)
    pub fn with_context(error: QmsError, context: &str) -> QmsError {
        match error {
            QmsError::Validation(msg) => QmsError::validation_error(&format!("{}: {}", context, msg)),
            QmsError::NotFound(msg) => QmsError::not_found(&format!("{}: {}", context, msg)),
            QmsError::AlreadyExists(msg) => QmsError::already_exists(&format!("{}: {}", context, msg)),
            QmsError::Permission(msg) => QmsError::permission_error(&format!("{}: {}", context, msg)),
            QmsError::InvalidOperation(msg) => QmsError::invalid_operation(&format!("{}: {}", context, msg)),
            QmsError::Authentication(msg) => QmsError::Authentication(format!("{}: {}", context, msg)),
            QmsError::Domain(msg) => QmsError::domain_error(&format!("{}: {}", context, msg)),
            QmsError::Parse(msg) => QmsError::parse_error(&format!("{}: {}", context, msg)),
            QmsError::Lock(msg) => QmsError::Lock(format!("{}: {}", context, msg)),
            QmsError::Io(io_err) => QmsError::io_error(&format!("{}: {}", context, io_err)),
        }
    }
}

/// Simplified error context helpers (REFACTORED: KISS principle)
/// Replaces complex builder pattern with simple functions
impl ErrorHandler {
    /// Add operation context to error (medical device compliance)
    pub fn with_operation(error: QmsError, operation: &str) -> QmsError {
        Self::with_context(error, &format!("Operation: {}", operation))
    }

    /// Add entity context to error (medical device compliance)
    pub fn with_entity(error: QmsError, entity_type: &str, entity_id: &str) -> QmsError {
        Self::with_context(error, &format!("Entity: {} ({})", entity_type, entity_id))
    }

    /// Add user context to error (medical device compliance)
    pub fn with_user(error: QmsError, user_id: &str) -> QmsError {
        Self::with_context(error, &format!("User: {}", user_id))
    }

    /// Add multiple context pieces to error (medical device compliance)
    pub fn with_full_context(
        error: QmsError,
        operation: Option<&str>,
        entity: Option<(&str, &str)>,
        user: Option<&str>
    ) -> QmsError {
        let mut context_parts = Vec::new();

        if let Some(op) = operation {
            context_parts.push(format!("Operation: {}", op));
        }

        if let Some((entity_type, entity_id)) = entity {
            context_parts.push(format!("Entity: {} ({})", entity_type, entity_id));
        }

        if let Some(user_id) = user {
            context_parts.push(format!("User: {}", user_id));
        }

        if context_parts.is_empty() {
            error
        } else {
            Self::with_context(error, &context_parts.join(", "))
        }
    }
}

/// Simplified helper macros (REFACTORED: KISS principle)
#[macro_export]
macro_rules! validate_field {
    ($condition:expr, $field:expr, $message:expr) => {
        if !$condition {
            return Err(crate::utils::error_handling::ErrorHandler::field_validation_error($field, $message));
        }
    };
}

#[macro_export]
macro_rules! ensure_exists {
    ($option:expr, $entity_type:expr, $id:expr) => {
        match $option {
            Some(value) => value,
            None => return Err(crate::utils::error_handling::ErrorHandler::not_found_error($entity_type, $id)),
        }
    };
}

/// Simplified error recovery (REFACTORED: KISS principle)
/// Removed complex retry and fallback patterns - use standard Rust patterns instead
pub struct ErrorRecovery;

impl ErrorRecovery {
    /// Simple retry for critical operations (medical device compliance)
    pub fn retry_simple<T, F>(mut operation: F, max_attempts: u32) -> QmsResult<T>
    where
        F: FnMut() -> QmsResult<T>,
    {
        let mut last_error = None;

        for attempt in 1..=max_attempts {
            match operation() {
                Ok(result) => return Ok(result),
                Err(error) => {
                    last_error = Some(error);
                    if attempt < max_attempts {
                        // Simple fixed delay instead of exponential backoff
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| QmsError::domain_error("Retry failed with no error")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_creation() {
        let error = ErrorHandler::field_validation_error("username", "too short");
        assert!(matches!(error, QmsError::Validation(_)));
        
        let error = ErrorHandler::not_found_error("User", "123");
        assert!(matches!(error, QmsError::NotFound(_)));
    }
    
    #[test]
    fn test_simplified_error_context() {
        // Test individual context helpers
        let operation_error = ErrorHandler::with_operation(
            QmsError::validation_error("Invalid data"),
            "create_user"
        );
        assert!(operation_error.to_string().contains("create_user"));

        let entity_error = ErrorHandler::with_entity(
            QmsError::validation_error("Invalid data"),
            "User",
            "123"
        );
        assert!(entity_error.to_string().contains("User"));
        assert!(entity_error.to_string().contains("123"));

        let user_error = ErrorHandler::with_user(
            QmsError::validation_error("Invalid data"),
            "admin"
        );
        assert!(user_error.to_string().contains("admin"));

        // Test full context helper
        let full_context_error = ErrorHandler::with_full_context(
            QmsError::validation_error("Invalid data"),
            Some("create_user"),
            Some(("User", "123")),
            Some("admin")
        );
        let error_msg = full_context_error.to_string();
        assert!(error_msg.contains("create_user"));
        assert!(error_msg.contains("User"));
        assert!(error_msg.contains("123"));
        assert!(error_msg.contains("admin"));
    }
    
    #[test]
    fn test_validation_macros() {
        fn test_validation() -> QmsResult<()> {
            let username = "ab"; // Too short
            validate_field!(username.len() >= 3, "username", "must be at least 3 characters");
            Ok(())
        }
        
        assert!(test_validation().is_err());
    }
}
