// Base template for user operations
// Defines the common workflow: Validation → Action → Audit → Response

use crate::prelude::*;
use crate::modules::user_manager::interfaces::{UserValidator, UserAuditor};

/// Result of a user operation
#[derive(Debug, Clone)]
pub struct UserOperationResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
    pub audit_logged: bool,
}

impl<T> UserOperationResult<T> {
    /// Create successful result
    pub fn success(data: T, message: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            message,
            audit_logged: false,
        }
    }
    
    /// Create error result
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message,
            audit_logged: false,
        }
    }
    
    /// Mark as audit logged
    pub fn with_audit_logged(mut self) -> Self {
        self.audit_logged = true;
        self
    }
}

/// Base template for user operations
/// Implements Template Method pattern with hooks for customization
pub trait BaseUserOperation<TInput, TOutput> {
    /// Template method - defines the workflow
    fn execute(&self, input: TInput) -> QmsResult<UserOperationResult<TOutput>> {
        // Step 1: Pre-validation
        self.pre_validate(&input)?;
        
        // Step 2: Main validation
        self.validate(&input)?;
        
        // Step 3: Pre-action hook
        self.pre_action(&input)?;
        
        // Step 4: Execute main action
        let result = self.perform_action(input)?;
        
        // Step 5: Post-action hook
        self.post_action(&result)?;
        
        // Step 6: Audit logging
        let audit_result = self.log_audit(&result);
        
        // Step 7: Post-processing
        let final_result = self.post_process(result)?;
        
        Ok(match audit_result {
            Ok(_) => final_result.with_audit_logged(),
            Err(_) => final_result, // Continue even if audit fails
        })
    }
    
    /// Pre-validation hook (can be overridden)
    fn pre_validate(&self, _input: &TInput) -> QmsResult<()> {
        Ok(())
    }
    
    /// Main validation (must be implemented)
    fn validate(&self, input: &TInput) -> QmsResult<()>;
    
    /// Pre-action hook (can be overridden)
    fn pre_action(&self, _input: &TInput) -> QmsResult<()> {
        Ok(())
    }
    
    /// Main action (must be implemented)
    fn perform_action(&self, input: TInput) -> QmsResult<UserOperationResult<TOutput>>;
    
    /// Post-action hook (can be overridden)
    fn post_action(&self, _result: &UserOperationResult<TOutput>) -> QmsResult<()> {
        Ok(())
    }
    
    /// Audit logging (can be overridden)
    fn log_audit(&self, result: &UserOperationResult<TOutput>) -> QmsResult<()> {
        // Default implementation - subclasses should override for specific audit messages
        if result.success {
            self.log_success_audit(result)
        } else {
            self.log_failure_audit(result)
        }
    }
    
    /// Log successful operation (can be overridden)
    fn log_success_audit(&self, _result: &UserOperationResult<TOutput>) -> QmsResult<()> {
        // Default: no logging
        Ok(())
    }
    
    /// Log failed operation (can be overridden)
    fn log_failure_audit(&self, _result: &UserOperationResult<TOutput>) -> QmsResult<()> {
        // Default: no logging
        Ok(())
    }
    
    /// Post-processing (can be overridden)
    fn post_process(&self, result: UserOperationResult<TOutput>) -> QmsResult<UserOperationResult<TOutput>> {
        Ok(result)
    }
    
    /// Get operation name for audit logging
    fn get_operation_name(&self) -> &'static str;
    
    /// Get operation category for audit logging
    fn get_operation_category(&self) -> &'static str {
        "User"
    }
}

/// Default validator implementation
pub struct DefaultUserValidator;

impl UserValidator for DefaultUserValidator {
    fn validate_username(&self, username: &str) -> QmsResult<()> {
        if username.len() < 3 || username.len() > 50 {
            return Err(QmsError::validation_error("Username must be 3-50 characters"));
        }
        
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(QmsError::validation_error("Username can only contain alphanumeric characters and underscores"));
        }
        
        Ok(())
    }
    
    fn validate_user(&self, user: &crate::models::User) -> QmsResult<()> {
        self.validate_username(&user.username)?;
        
        if user.password_hash.is_empty() {
            return Err(QmsError::validation_error("Password hash cannot be empty"));
        }
        
        if user.roles.is_empty() {
            return Err(QmsError::validation_error("User must have at least one role"));
        }
        
        Ok(())
    }
    
    fn validate_role_assignment(&self, user: &crate::models::User, role: &crate::models::Role) -> QmsResult<()> {
        self.validate_user(user)?;
        
        if role.name.is_empty() {
            return Err(QmsError::validation_error("Role name cannot be empty"));
        }
        
        if role.permissions.is_empty() {
            return Err(QmsError::validation_error("Role must have at least one permission"));
        }
        
        // Check if user already has this role
        if user.roles.iter().any(|r| r.name == role.name) {
            return Err(QmsError::validation_error("User already has this role"));
        }
        
        Ok(())
    }
}

/// Default auditor implementation
pub struct DefaultUserAuditor;

impl UserAuditor for DefaultUserAuditor {
    fn log_user_operation(&self, operation: &str, username: &str, details: &str) -> QmsResult<()> {
        crate::modules::audit_logger::audit_log_action(
            operation,
            "User",
            &format!("{username}: {details}")
        )
    }
    
    fn log_authentication(&self, username: &str, success: bool, ip: Option<&str>) -> QmsResult<()> {
        let operation = if success { "LOGIN_SUCCESS" } else { "LOGIN_FAILED" };
        let details = match ip {
            Some(ip) => format!("from {ip}"),
            None => "from unknown IP".to_string(),
        };
        
        self.log_user_operation(operation, username, &details)
    }
    
    fn log_session_operation(&self, operation: &str, session_id: &str, username: &str) -> QmsResult<()> {
        crate::modules::audit_logger::audit_log_action(
            operation,
            "Session",
            &format!("session:{session_id} user:{username}")
        )
    }
    
    fn log_role_operation(&self, operation: &str, username: &str, role: &str) -> QmsResult<()> {
        self.log_user_operation(operation, username, &format!("role:{role}"))
    }
}

/// Validation helper functions
pub struct ValidationHelpers;

impl ValidationHelpers {
    /// Validate password strength
    pub fn validate_password_strength(password: &str, min_length: usize, require_special: bool) -> QmsResult<()> {
        if password.len() < min_length {
            return Err(QmsError::validation_error(&format!(
                "Password must be at least {min_length} characters long"
            )));
        }
        
        if require_special && !password.chars().any(|c| !c.is_alphanumeric()) {
            return Err(QmsError::validation_error(
                "Password must contain at least one special character"
            ));
        }
        
        Ok(())
    }
    
    /// Validate email format (basic validation)
    pub fn validate_email(email: &str) -> QmsResult<()> {
        if !email.contains('@') || !email.contains('.') {
            return Err(QmsError::validation_error("Invalid email format"));
        }
        
        Ok(())
    }
    
    /// Validate role name
    pub fn validate_role_name(role_name: &str) -> QmsResult<()> {
        if role_name.is_empty() || role_name.len() > 50 {
            return Err(QmsError::validation_error("Role name must be 1-50 characters"));
        }
        
        if !role_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == ' ') {
            return Err(QmsError::validation_error(
                "Role name can only contain alphanumeric characters, underscores, and spaces"
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{User, Role, Permission};
    
    // Test implementation of BaseUserOperation
    struct TestUserOperation {
        should_fail_validation: bool,
        should_fail_action: bool,
    }
    
    impl TestUserOperation {
        fn new() -> Self {
            Self {
                should_fail_validation: false,
                should_fail_action: false,
            }
        }
        
        fn with_validation_failure() -> Self {
            Self {
                should_fail_validation: true,
                should_fail_action: false,
            }
        }
        
        fn with_action_failure() -> Self {
            Self {
                should_fail_validation: false,
                should_fail_action: true,
            }
        }
    }
    
    impl BaseUserOperation<String, String> for TestUserOperation {
        fn validate(&self, _input: &String) -> QmsResult<()> {
            if self.should_fail_validation {
                Err(QmsError::validation_error("Validation failed"))
            } else {
                Ok(())
            }
        }
        
        fn perform_action(&self, input: String) -> QmsResult<UserOperationResult<String>> {
            if self.should_fail_action {
                Ok(UserOperationResult::error("Action failed".to_string()))
            } else {
                Ok(UserOperationResult::success(
                    format!("Processed: {input}"),
                    "Operation successful".to_string()
                ))
            }
        }
        
        fn get_operation_name(&self) -> &'static str {
            "TEST_OPERATION"
        }
    }
    
    #[test]
    fn test_successful_operation() {
        let operation = TestUserOperation::new();
        let result = operation.execute("test_input".to_string()).unwrap();
        
        assert!(result.success);
        assert_eq!(result.data.unwrap(), "Processed: test_input");
        assert_eq!(result.message, "Operation successful");
    }
    
    #[test]
    fn test_validation_failure() {
        let operation = TestUserOperation::with_validation_failure();
        let result = operation.execute("test_input".to_string());
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_action_failure() {
        let operation = TestUserOperation::with_action_failure();
        let result = operation.execute("test_input".to_string()).unwrap();
        
        assert!(!result.success);
        assert!(result.data.is_none());
        assert_eq!(result.message, "Action failed");
    }
    
    #[test]
    fn test_username_validation() {
        let validator = DefaultUserValidator;
        
        // Valid usernames
        assert!(validator.validate_username("valid_user").is_ok());
        assert!(validator.validate_username("user123").is_ok());
        
        // Invalid usernames
        assert!(validator.validate_username("ab").is_err()); // Too short
        assert!(validator.validate_username("user@domain").is_err()); // Invalid chars
        assert!(validator.validate_username(&"a".repeat(51)).is_err()); // Too long
    }
    
    #[test]
    fn test_password_strength_validation() {
        // Valid passwords
        assert!(ValidationHelpers::validate_password_strength("password123", 8, false).is_ok());
        assert!(ValidationHelpers::validate_password_strength("password123!", 8, true).is_ok());
        
        // Invalid passwords
        assert!(ValidationHelpers::validate_password_strength("short", 8, false).is_err());
        assert!(ValidationHelpers::validate_password_strength("password123", 8, true).is_err());
    }
}
