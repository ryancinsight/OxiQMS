//! Unified Validation and Error Handling
//! 
//! Creates shared validation rules and error handling following DRY principles
//! and providing consistent validation across all interfaces.

use crate::prelude::*;
use crate::interfaces::{InterfaceType, InterfaceContext};
use std::collections::HashMap;
use std::sync::Arc;

/// Unified Validation Manager
/// 
/// Central coordinator for all validation operations across interfaces.
/// Implements Single Responsibility Principle by focusing solely on validation.
pub struct UnifiedValidationManager {
    /// Validation rule registry
    rule_registry: HashMap<String, Arc<dyn ValidationRule>>,
    
    /// Field validators for specific field types
    field_validators: HashMap<FieldType, Arc<dyn FieldValidator>>,
    
    /// Error formatters for different interfaces
    error_formatters: HashMap<InterfaceType, Arc<dyn ErrorFormatter>>,
    
    /// Validation context providers
    context_providers: HashMap<String, Arc<dyn ValidationContextProvider>>,
}

/// Validation rule trait
/// 
/// Defines a single validation rule that can be applied to values.
pub trait ValidationRule: Send + Sync {
    /// Validate a value against this rule
    fn validate(&self, value: &str, context: &ValidationContext) -> ValidationResult;
    
    /// Get rule name
    fn name(&self) -> &str;
    
    /// Get rule description
    fn description(&self) -> &str;
    
    /// Get rule parameters
    fn parameters(&self) -> HashMap<String, String>;
}

/// Field validator trait
/// 
/// Validates specific field types with comprehensive rules.
pub trait FieldValidator: Send + Sync {
    /// Validate field value
    fn validate_field(&self, field_name: &str, value: &str, context: &ValidationContext) -> ValidationResult;
    
    /// Get validation rules for field type
    fn get_rules(&self) -> Vec<String>;
    
    /// Get field type
    fn field_type(&self) -> FieldType;
}

/// Error formatter trait
/// 
/// Formats validation errors for specific interfaces.
pub trait ErrorFormatter: Send + Sync {
    /// Format validation result for interface
    fn format_validation_result(&self, result: &ValidationResult, context: &ValidationContext) -> FormattedError;
    
    /// Format single error message
    fn format_error_message(&self, error: &ValidationError) -> String;
    
    /// Get interface type
    fn interface_type(&self) -> InterfaceType;
}

/// Validation context provider trait
/// 
/// Provides context-specific validation information.
pub trait ValidationContextProvider: Send + Sync {
    /// Get validation context for field
    fn get_context(&self, field_name: &str, interface_type: InterfaceType) -> ValidationContext;
    
    /// Update validation context
    fn update_context(&self, context: &mut ValidationContext, updates: HashMap<String, String>);
}

/// Field type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FieldType {
    /// Username field
    Username,
    
    /// Password field
    Password,
    
    /// Email address field
    Email,
    
    /// File path field
    FilePath,
    
    /// Directory path field
    DirectoryPath,
    
    /// URL field
    Url,
    
    /// Project name field
    ProjectName,
    
    /// Document title field
    DocumentTitle,
    
    /// Risk description field
    RiskDescription,
    
    /// Configuration value field
    ConfigValue,
    
    /// Generic text field
    Text,
    
    /// Numeric field
    Number,
    
    /// Date field
    Date,
    
    /// Boolean field
    Boolean,
}

/// Validation context
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// Field being validated
    pub field_name: String,
    
    /// Field type
    pub field_type: FieldType,
    
    /// Interface type performing validation
    pub interface_type: InterfaceType,
    
    /// User context if available
    pub user_context: Option<String>,
    
    /// Project context if available
    pub project_context: Option<String>,
    
    /// Additional context data
    pub context_data: HashMap<String, String>,
    
    /// Validation strictness level
    pub strictness: ValidationStrictness,
}

/// Validation strictness level
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationStrictness {
    /// Lenient validation (warnings only)
    Lenient,
    
    /// Standard validation (normal rules)
    Standard,
    
    /// Strict validation (all rules enforced)
    Strict,
    
    /// Production validation (security-focused)
    Production,
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    
    /// Validation errors (blocking)
    pub errors: Vec<ValidationError>,
    
    /// Validation warnings (non-blocking)
    pub warnings: Vec<ValidationWarning>,
    
    /// Validation suggestions
    pub suggestions: Vec<ValidationSuggestion>,
    
    /// Validation metadata
    pub metadata: ValidationMetadata,
}

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Error code
    pub code: String,
    
    /// Error message
    pub message: String,
    
    /// Field that caused the error
    pub field: String,
    
    /// Error severity
    pub severity: ErrorSeverity,
    
    /// Suggested fix
    pub suggested_fix: Option<String>,
}

/// Validation warning
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Warning code
    pub code: String,
    
    /// Warning message
    pub message: String,
    
    /// Field that caused the warning
    pub field: String,
    
    /// Suggested improvement
    pub suggestion: Option<String>,
}

/// Validation suggestion
#[derive(Debug, Clone)]
pub struct ValidationSuggestion {
    /// Suggestion type
    pub suggestion_type: SuggestionType,
    
    /// Suggestion message
    pub message: String,
    
    /// Suggested value
    pub suggested_value: Option<String>,
}

/// Validation metadata
#[derive(Debug, Clone)]
pub struct ValidationMetadata {
    /// Validation timestamp
    pub timestamp: u64,
    
    /// Rules applied
    pub rules_applied: Vec<String>,
    
    /// Validation duration
    pub duration_ms: u64,
    
    /// Validation context
    pub context: String,
}

/// Error severity
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    /// Critical error (blocks operation)
    Critical,
    
    /// High severity error
    High,
    
    /// Medium severity error
    Medium,
    
    /// Low severity error
    Low,
    
    /// Informational
    Info,
}

/// Suggestion type
#[derive(Debug, Clone, PartialEq)]
pub enum SuggestionType {
    /// Correction suggestion
    Correction,
    
    /// Improvement suggestion
    Improvement,
    
    /// Alternative suggestion
    Alternative,
    
    /// Best practice suggestion
    BestPractice,
}

/// Formatted error for interface display
#[derive(Debug, Clone)]
pub struct FormattedError {
    /// Formatted error message
    pub message: String,
    
    /// Interface-specific formatting
    pub formatted_content: String,
    
    /// Error metadata for interface
    pub metadata: HashMap<String, String>,
}

impl UnifiedValidationManager {
    /// Create new unified validation manager
    pub fn new() -> Self {
        let mut manager = Self {
            rule_registry: HashMap::new(),
            field_validators: HashMap::new(),
            error_formatters: HashMap::new(),
            context_providers: HashMap::new(),
        };
        
        // Initialize default validation rules
        manager.initialize_default_rules();
        
        // Initialize field validators
        manager.initialize_field_validators();
        
        // Initialize error formatters
        manager.initialize_error_formatters();
        
        // Initialize context providers
        manager.initialize_context_providers();
        
        manager
    }
    
    /// Validate field value
    pub fn validate_field(&self, field_name: &str, value: &str, field_type: FieldType, interface_type: InterfaceType) -> ValidationResult {
        let start_time = std::time::Instant::now();
        
        // Get validation context
        let context = self.get_validation_context(field_name, field_type.clone(), interface_type);
        
        // Get field validator
        let validator = self.field_validators.get(&field_type);
        
        let result = if let Some(validator) = validator {
            validator.validate_field(field_name, value, &context)
        } else {
            // Fallback to basic validation
            self.basic_field_validation(field_name, value, &context)
        };
        
        // Add metadata
        let duration = start_time.elapsed();
        let mut final_result = result;
        final_result.metadata.duration_ms = duration.as_millis() as u64;
        final_result.metadata.timestamp = crate::utils::current_timestamp();
        
        final_result
    }
    
    /// Validate multiple fields
    pub fn validate_fields(&self, fields: HashMap<String, (String, FieldType)>, interface_type: InterfaceType) -> HashMap<String, ValidationResult> {
        let mut results = HashMap::new();
        
        for (field_name, (value, field_type)) in fields {
            let result = self.validate_field(&field_name, &value, field_type, interface_type.clone());
            results.insert(field_name, result);
        }
        
        results
    }
    
    /// Format validation result for interface
    pub fn format_validation_result(&self, result: &ValidationResult, interface_type: InterfaceType) -> FormattedError {
        if let Some(formatter) = self.error_formatters.get(&interface_type) {
            formatter.format_validation_result(result, &ValidationContext::default())
        } else {
            // Fallback formatting
            FormattedError {
                message: if result.is_valid {
                    "Validation passed".to_string()
                } else {
                    format!("Validation failed: {}", 
                        result.errors.iter().map(|e| &e.message).cloned().collect::<Vec<_>>().join(", ")
                    )
                },
                formatted_content: format!("{:?}", result),
                metadata: HashMap::new(),
            }
        }
    }
    
    /// Register custom validation rule
    pub fn register_rule(&mut self, name: String, rule: Arc<dyn ValidationRule>) {
        self.rule_registry.insert(name, rule);
    }
    
    /// Register custom field validator
    pub fn register_field_validator(&mut self, field_type: FieldType, validator: Arc<dyn FieldValidator>) {
        self.field_validators.insert(field_type, validator);
    }
    
    /// Register custom error formatter
    pub fn register_error_formatter(&mut self, interface_type: InterfaceType, formatter: Arc<dyn ErrorFormatter>) {
        self.error_formatters.insert(interface_type, formatter);
    }
    
    // Private helper methods
    
    /// Initialize default validation rules
    fn initialize_default_rules(&mut self) {
        self.register_rule("required".to_string(), Arc::new(RequiredRule::new()));
        self.register_rule("min_length".to_string(), Arc::new(MinLengthRule::new(1)));
        self.register_rule("max_length".to_string(), Arc::new(MaxLengthRule::new(255)));
        self.register_rule("email_format".to_string(), Arc::new(EmailFormatRule::new()));
        self.register_rule("path_format".to_string(), Arc::new(PathFormatRule::new()));
        self.register_rule("alphanumeric".to_string(), Arc::new(AlphanumericRule::new()));
    }
    
    /// Initialize field validators
    fn initialize_field_validators(&mut self) {
        self.register_field_validator(FieldType::Username, Arc::new(UsernameValidator::new()));
        self.register_field_validator(FieldType::Password, Arc::new(PasswordValidator::new()));
        self.register_field_validator(FieldType::Email, Arc::new(EmailValidator::new()));
        self.register_field_validator(FieldType::FilePath, Arc::new(FilePathValidator::new()));
        self.register_field_validator(FieldType::ProjectName, Arc::new(ProjectNameValidator::new()));
    }
    
    /// Initialize error formatters
    fn initialize_error_formatters(&mut self) {
        self.register_error_formatter(InterfaceType::CLI, Arc::new(CliErrorFormatter::new()));
        self.register_error_formatter(InterfaceType::Web, Arc::new(WebErrorFormatter::new()));
        self.register_error_formatter(InterfaceType::TUI, Arc::new(TuiErrorFormatter::new()));
    }
    
    /// Initialize context providers
    fn initialize_context_providers(&mut self) {
        self.context_providers.insert("default".to_string(), Arc::new(DefaultContextProvider::new()));
    }
    
    /// Get validation context
    fn get_validation_context(&self, field_name: &str, field_type: FieldType, interface_type: InterfaceType) -> ValidationContext {
        let provider = self.context_providers.get("default").unwrap();
        provider.get_context(field_name, interface_type)
    }
    
    /// Basic field validation fallback
    fn basic_field_validation(&self, field_name: &str, value: &str, context: &ValidationContext) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Basic required check
        if value.trim().is_empty() {
            errors.push(ValidationError {
                code: "REQUIRED".to_string(),
                message: format!("Field '{}' is required", field_name),
                field: field_name.to_string(),
                severity: ErrorSeverity::High,
                suggested_fix: Some("Please provide a value".to_string()),
            });
        }
        
        // Basic length check
        if value.len() > 1000 {
            warnings.push(ValidationWarning {
                code: "LONG_VALUE".to_string(),
                message: format!("Field '{}' is very long", field_name),
                field: field_name.to_string(),
                suggestion: Some("Consider shortening the value".to_string()),
            });
        }
        
        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            suggestions: Vec::new(),
            metadata: ValidationMetadata {
                timestamp: 0, // Will be set by caller
                rules_applied: vec!["basic".to_string()],
                duration_ms: 0, // Will be set by caller
                context: format!("{:?}", context.interface_type),
            },
        }
    }
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self {
            field_name: "unknown".to_string(),
            field_type: FieldType::Text,
            interface_type: InterfaceType::CLI,
            user_context: None,
            project_context: None,
            context_data: HashMap::new(),
            strictness: ValidationStrictness::Standard,
        }
    }
}

impl ValidationResult {
    /// Create successful validation result
    pub fn success() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
            metadata: ValidationMetadata {
                timestamp: crate::utils::current_timestamp(),
                rules_applied: Vec::new(),
                duration_ms: 0,
                context: "success".to_string(),
            },
        }
    }
    
    /// Create failed validation result
    pub fn failure(errors: Vec<ValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
            suggestions: Vec::new(),
            metadata: ValidationMetadata {
                timestamp: crate::utils::current_timestamp(),
                rules_applied: Vec::new(),
                duration_ms: 0,
                context: "failure".to_string(),
            },
        }
    }
    
    /// Add warning to result
    pub fn with_warning(mut self, warning: ValidationWarning) -> Self {
        self.warnings.push(warning);
        self
    }
    
    /// Add suggestion to result
    pub fn with_suggestion(mut self, suggestion: ValidationSuggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }
}

// Validation Rule Implementations

/// Required field validation rule
pub struct RequiredRule;

impl RequiredRule {
    pub fn new() -> Self {
        Self
    }
}

impl ValidationRule for RequiredRule {
    fn validate(&self, value: &str, context: &ValidationContext) -> ValidationResult {
        if value.trim().is_empty() {
            ValidationResult::failure(vec![ValidationError {
                code: "REQUIRED".to_string(),
                message: format!("Field '{}' is required", context.field_name),
                field: context.field_name.clone(),
                severity: ErrorSeverity::High,
                suggested_fix: Some("Please provide a value".to_string()),
            }])
        } else {
            ValidationResult::success()
        }
    }

    fn name(&self) -> &str {
        "required"
    }

    fn description(&self) -> &str {
        "Validates that a field has a non-empty value"
    }

    fn parameters(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

/// Minimum length validation rule
pub struct MinLengthRule {
    min_length: usize,
}

impl MinLengthRule {
    pub fn new(min_length: usize) -> Self {
        Self { min_length }
    }
}

impl ValidationRule for MinLengthRule {
    fn validate(&self, value: &str, context: &ValidationContext) -> ValidationResult {
        if value.len() < self.min_length {
            ValidationResult::failure(vec![ValidationError {
                code: "MIN_LENGTH".to_string(),
                message: format!("Field '{}' must be at least {} characters long", context.field_name, self.min_length),
                field: context.field_name.clone(),
                severity: ErrorSeverity::Medium,
                suggested_fix: Some(format!("Add {} more characters", self.min_length - value.len())),
            }])
        } else {
            ValidationResult::success()
        }
    }

    fn name(&self) -> &str {
        "min_length"
    }

    fn description(&self) -> &str {
        "Validates minimum string length"
    }

    fn parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("min_length".to_string(), self.min_length.to_string());
        params
    }
}

/// Maximum length validation rule
pub struct MaxLengthRule {
    max_length: usize,
}

impl MaxLengthRule {
    pub fn new(max_length: usize) -> Self {
        Self { max_length }
    }
}

impl ValidationRule for MaxLengthRule {
    fn validate(&self, value: &str, context: &ValidationContext) -> ValidationResult {
        if value.len() > self.max_length {
            ValidationResult::failure(vec![ValidationError {
                code: "MAX_LENGTH".to_string(),
                message: format!("Field '{}' must be no more than {} characters long", context.field_name, self.max_length),
                field: context.field_name.clone(),
                severity: ErrorSeverity::Medium,
                suggested_fix: Some(format!("Remove {} characters", value.len() - self.max_length)),
            }])
        } else {
            ValidationResult::success()
        }
    }

    fn name(&self) -> &str {
        "max_length"
    }

    fn description(&self) -> &str {
        "Validates maximum string length"
    }

    fn parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("max_length".to_string(), self.max_length.to_string());
        params
    }
}

/// Email format validation rule
pub struct EmailFormatRule;

impl EmailFormatRule {
    pub fn new() -> Self {
        Self
    }
}

impl ValidationRule for EmailFormatRule {
    fn validate(&self, value: &str, context: &ValidationContext) -> ValidationResult {
        // Simple email validation (in real implementation would use proper regex)
        if value.contains('@') && value.contains('.') && value.len() > 5 {
            ValidationResult::success()
        } else {
            ValidationResult::failure(vec![ValidationError {
                code: "EMAIL_FORMAT".to_string(),
                message: format!("Field '{}' must be a valid email address", context.field_name),
                field: context.field_name.clone(),
                severity: ErrorSeverity::High,
                suggested_fix: Some("Use format: user@domain.com".to_string()),
            }])
        }
    }

    fn name(&self) -> &str {
        "email_format"
    }

    fn description(&self) -> &str {
        "Validates email address format"
    }

    fn parameters(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

/// Path format validation rule
pub struct PathFormatRule;

impl PathFormatRule {
    pub fn new() -> Self {
        Self
    }
}

impl ValidationRule for PathFormatRule {
    fn validate(&self, value: &str, context: &ValidationContext) -> ValidationResult {
        let path = std::path::Path::new(value);

        // Check for invalid characters (simplified)
        if value.contains('\0') || value.contains('\x01') {
            return ValidationResult::failure(vec![ValidationError {
                code: "PATH_INVALID_CHARS".to_string(),
                message: format!("Field '{}' contains invalid path characters", context.field_name),
                field: context.field_name.clone(),
                severity: ErrorSeverity::High,
                suggested_fix: Some("Remove invalid characters".to_string()),
            }]);
        }

        // Check path length
        if value.len() > 260 {
            return ValidationResult::failure(vec![ValidationError {
                code: "PATH_TOO_LONG".to_string(),
                message: format!("Field '{}' path is too long", context.field_name),
                field: context.field_name.clone(),
                severity: ErrorSeverity::Medium,
                suggested_fix: Some("Use a shorter path".to_string()),
            }]);
        }

        ValidationResult::success()
    }

    fn name(&self) -> &str {
        "path_format"
    }

    fn description(&self) -> &str {
        "Validates file system path format"
    }

    fn parameters(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

/// Alphanumeric validation rule
pub struct AlphanumericRule;

impl AlphanumericRule {
    pub fn new() -> Self {
        Self
    }
}

impl ValidationRule for AlphanumericRule {
    fn validate(&self, value: &str, context: &ValidationContext) -> ValidationResult {
        if value.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            ValidationResult::success()
        } else {
            ValidationResult::failure(vec![ValidationError {
                code: "ALPHANUMERIC".to_string(),
                message: format!("Field '{}' must contain only letters, numbers, underscores, and hyphens", context.field_name),
                field: context.field_name.clone(),
                severity: ErrorSeverity::Medium,
                suggested_fix: Some("Remove special characters".to_string()),
            }])
        }
    }

    fn name(&self) -> &str {
        "alphanumeric"
    }

    fn description(&self) -> &str {
        "Validates alphanumeric characters only"
    }

    fn parameters(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

// Field Validator Implementations

/// Username field validator
pub struct UsernameValidator;

impl UsernameValidator {
    pub fn new() -> Self {
        Self
    }
}

impl FieldValidator for UsernameValidator {
    fn validate_field(&self, field_name: &str, value: &str, context: &ValidationContext) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Required check
        if value.trim().is_empty() {
            result.errors.push(ValidationError {
                code: "USERNAME_REQUIRED".to_string(),
                message: "Username is required".to_string(),
                field: field_name.to_string(),
                severity: ErrorSeverity::High,
                suggested_fix: Some("Please enter a username".to_string()),
            });
            result.is_valid = false;
        }

        // Length check
        if value.len() < 3 {
            result.errors.push(ValidationError {
                code: "USERNAME_TOO_SHORT".to_string(),
                message: "Username must be at least 3 characters long".to_string(),
                field: field_name.to_string(),
                severity: ErrorSeverity::Medium,
                suggested_fix: Some("Use a longer username".to_string()),
            });
            result.is_valid = false;
        }

        if value.len() > 50 {
            result.errors.push(ValidationError {
                code: "USERNAME_TOO_LONG".to_string(),
                message: "Username must be no more than 50 characters long".to_string(),
                field: field_name.to_string(),
                severity: ErrorSeverity::Medium,
                suggested_fix: Some("Use a shorter username".to_string()),
            });
            result.is_valid = false;
        }

        // Character check
        if !value.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            result.errors.push(ValidationError {
                code: "USERNAME_INVALID_CHARS".to_string(),
                message: "Username can only contain letters, numbers, underscores, and hyphens".to_string(),
                field: field_name.to_string(),
                severity: ErrorSeverity::Medium,
                suggested_fix: Some("Remove special characters".to_string()),
            });
            result.is_valid = false;
        }

        result
    }

    fn get_rules(&self) -> Vec<String> {
        vec!["required".to_string(), "min_length".to_string(), "max_length".to_string(), "alphanumeric".to_string()]
    }

    fn field_type(&self) -> FieldType {
        FieldType::Username
    }
}

/// Password field validator
pub struct PasswordValidator;

impl PasswordValidator {
    pub fn new() -> Self {
        Self
    }
}

impl FieldValidator for PasswordValidator {
    fn validate_field(&self, field_name: &str, value: &str, context: &ValidationContext) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Required check
        if value.is_empty() {
            result.errors.push(ValidationError {
                code: "PASSWORD_REQUIRED".to_string(),
                message: "Password is required".to_string(),
                field: field_name.to_string(),
                severity: ErrorSeverity::High,
                suggested_fix: Some("Please enter a password".to_string()),
            });
            result.is_valid = false;
            return result;
        }

        // Length check
        if value.len() < 8 {
            result.errors.push(ValidationError {
                code: "PASSWORD_TOO_SHORT".to_string(),
                message: "Password must be at least 8 characters long".to_string(),
                field: field_name.to_string(),
                severity: ErrorSeverity::High,
                suggested_fix: Some("Use a longer password".to_string()),
            });
            result.is_valid = false;
        }

        // Strength checks (warnings for better UX)
        if !value.chars().any(|c| c.is_uppercase()) {
            result.warnings.push(ValidationWarning {
                code: "PASSWORD_NO_UPPERCASE".to_string(),
                message: "Password should contain at least one uppercase letter".to_string(),
                field: field_name.to_string(),
                suggestion: Some("Add an uppercase letter for better security".to_string()),
            });
        }

        if !value.chars().any(|c| c.is_lowercase()) {
            result.warnings.push(ValidationWarning {
                code: "PASSWORD_NO_LOWERCASE".to_string(),
                message: "Password should contain at least one lowercase letter".to_string(),
                field: field_name.to_string(),
                suggestion: Some("Add a lowercase letter for better security".to_string()),
            });
        }

        if !value.chars().any(|c| c.is_numeric()) {
            result.warnings.push(ValidationWarning {
                code: "PASSWORD_NO_NUMBER".to_string(),
                message: "Password should contain at least one number".to_string(),
                field: field_name.to_string(),
                suggestion: Some("Add a number for better security".to_string()),
            });
        }

        if !value.chars().any(|c| !c.is_alphanumeric()) {
            result.warnings.push(ValidationWarning {
                code: "PASSWORD_NO_SPECIAL".to_string(),
                message: "Password should contain at least one special character".to_string(),
                field: field_name.to_string(),
                suggestion: Some("Add a special character for better security".to_string()),
            });
        }

        result
    }

    fn get_rules(&self) -> Vec<String> {
        vec!["required".to_string(), "min_length".to_string()]
    }

    fn field_type(&self) -> FieldType {
        FieldType::Password
    }
}

/// Email field validator
pub struct EmailValidator;

impl EmailValidator {
    pub fn new() -> Self {
        Self
    }
}

impl FieldValidator for EmailValidator {
    fn validate_field(&self, field_name: &str, value: &str, context: &ValidationContext) -> ValidationResult {
        let email_rule = EmailFormatRule::new();
        email_rule.validate(value, context)
    }

    fn get_rules(&self) -> Vec<String> {
        vec!["email_format".to_string()]
    }

    fn field_type(&self) -> FieldType {
        FieldType::Email
    }
}

/// File path field validator
pub struct FilePathValidator;

impl FilePathValidator {
    pub fn new() -> Self {
        Self
    }
}

impl FieldValidator for FilePathValidator {
    fn validate_field(&self, field_name: &str, value: &str, context: &ValidationContext) -> ValidationResult {
        let path_rule = PathFormatRule::new();
        path_rule.validate(value, context)
    }

    fn get_rules(&self) -> Vec<String> {
        vec!["path_format".to_string()]
    }

    fn field_type(&self) -> FieldType {
        FieldType::FilePath
    }
}

/// Project name field validator
pub struct ProjectNameValidator;

impl ProjectNameValidator {
    pub fn new() -> Self {
        Self
    }
}

impl FieldValidator for ProjectNameValidator {
    fn validate_field(&self, field_name: &str, value: &str, context: &ValidationContext) -> ValidationResult {
        let mut result = ValidationResult::success();

        // Required check
        if value.trim().is_empty() {
            result.errors.push(ValidationError {
                code: "PROJECT_NAME_REQUIRED".to_string(),
                message: "Project name is required".to_string(),
                field: field_name.to_string(),
                severity: ErrorSeverity::High,
                suggested_fix: Some("Please enter a project name".to_string()),
            });
            result.is_valid = false;
        }

        // Length check
        if value.len() < 2 {
            result.errors.push(ValidationError {
                code: "PROJECT_NAME_TOO_SHORT".to_string(),
                message: "Project name must be at least 2 characters long".to_string(),
                field: field_name.to_string(),
                severity: ErrorSeverity::Medium,
                suggested_fix: Some("Use a longer project name".to_string()),
            });
            result.is_valid = false;
        }

        if value.len() > 100 {
            result.errors.push(ValidationError {
                code: "PROJECT_NAME_TOO_LONG".to_string(),
                message: "Project name must be no more than 100 characters long".to_string(),
                field: field_name.to_string(),
                severity: ErrorSeverity::Medium,
                suggested_fix: Some("Use a shorter project name".to_string()),
            });
            result.is_valid = false;
        }

        // Character check (more lenient than username)
        if value.chars().any(|c| c.is_control() || c == '/' || c == '\\' || c == ':' || c == '*' || c == '?' || c == '"' || c == '<' || c == '>' || c == '|') {
            result.errors.push(ValidationError {
                code: "PROJECT_NAME_INVALID_CHARS".to_string(),
                message: "Project name contains invalid characters".to_string(),
                field: field_name.to_string(),
                severity: ErrorSeverity::Medium,
                suggested_fix: Some("Remove invalid characters (/ \\ : * ? \" < > |)".to_string()),
            });
            result.is_valid = false;
        }

        result
    }

    fn get_rules(&self) -> Vec<String> {
        vec!["required".to_string(), "min_length".to_string(), "max_length".to_string()]
    }

    fn field_type(&self) -> FieldType {
        FieldType::ProjectName
    }
}

// Error Formatter Implementations

/// CLI error formatter
pub struct CliErrorFormatter;

impl CliErrorFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl ErrorFormatter for CliErrorFormatter {
    fn format_validation_result(&self, result: &ValidationResult, _context: &ValidationContext) -> FormattedError {
        let mut formatted_content = String::new();

        if result.is_valid {
            formatted_content.push_str("✅ Validation passed\n");
        } else {
            formatted_content.push_str("❌ Validation failed:\n");
            for error in &result.errors {
                formatted_content.push_str(&format!("  • {}\n", error.message));
                if let Some(ref fix) = error.suggested_fix {
                    formatted_content.push_str(&format!("    💡 {}\n", fix));
                }
            }
        }

        if !result.warnings.is_empty() {
            formatted_content.push_str("\n⚠️  Warnings:\n");
            for warning in &result.warnings {
                formatted_content.push_str(&format!("  • {}\n", warning.message));
                if let Some(ref suggestion) = warning.suggestion {
                    formatted_content.push_str(&format!("    💡 {}\n", suggestion));
                }
            }
        }

        FormattedError {
            message: if result.is_valid { "Validation passed".to_string() } else { "Validation failed".to_string() },
            formatted_content,
            metadata: HashMap::new(),
        }
    }

    fn format_error_message(&self, error: &ValidationError) -> String {
        format!("❌ {}", error.message)
    }

    fn interface_type(&self) -> InterfaceType {
        InterfaceType::CLI
    }
}

/// Web error formatter
pub struct WebErrorFormatter;

impl WebErrorFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl ErrorFormatter for WebErrorFormatter {
    fn format_validation_result(&self, result: &ValidationResult, _context: &ValidationContext) -> FormattedError {
        let formatted_content = format!(r#"{{
  "valid": {},
  "errors": [{}],
  "warnings": [{}],
  "suggestions": [{}]
}}"#,
            result.is_valid,
            result.errors.iter().map(|e| format!(r#"{{"code": "{}", "message": "{}"}}"#, e.code, e.message)).collect::<Vec<_>>().join(", "),
            result.warnings.iter().map(|w| format!(r#"{{"code": "{}", "message": "{}"}}"#, w.code, w.message)).collect::<Vec<_>>().join(", "),
            result.suggestions.iter().map(|s| format!(r#"{{"type": "{:?}", "message": "{}"}}"#, s.suggestion_type, s.message)).collect::<Vec<_>>().join(", ")
        );

        FormattedError {
            message: if result.is_valid { "Validation passed".to_string() } else { "Validation failed".to_string() },
            formatted_content,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("content_type".to_string(), "application/json".to_string());
                meta
            },
        }
    }

    fn format_error_message(&self, error: &ValidationError) -> String {
        format!(r#"{{"error": "{}", "code": "{}"}}"#, error.message, error.code)
    }

    fn interface_type(&self) -> InterfaceType {
        InterfaceType::Web
    }
}

/// TUI error formatter
pub struct TuiErrorFormatter;

impl TuiErrorFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl ErrorFormatter for TuiErrorFormatter {
    fn format_validation_result(&self, result: &ValidationResult, _context: &ValidationContext) -> FormattedError {
        let mut formatted_content = String::new();

        if result.is_valid {
            formatted_content.push_str("╔══════════════════════════════════════╗\n");
            formatted_content.push_str("║            ✅ Valid Input            ║\n");
            formatted_content.push_str("╚══════════════════════════════════════╝\n");
        } else {
            formatted_content.push_str("╔══════════════════════════════════════╗\n");
            formatted_content.push_str("║          ❌ Validation Failed        ║\n");
            formatted_content.push_str("╠══════════════════════════════════════╣\n");

            for error in &result.errors {
                let message = if error.message.len() > 34 {
                    format!("{}...", &error.message[..31])
                } else {
                    error.message.clone()
                };
                formatted_content.push_str(&format!("║ • {:<34} ║\n", message));

                if let Some(ref fix) = error.suggested_fix {
                    let fix_msg = if fix.len() > 32 {
                        format!("{}...", &fix[..29])
                    } else {
                        fix.clone()
                    };
                    formatted_content.push_str(&format!("║   💡 {:<32} ║\n", fix_msg));
                }
            }

            formatted_content.push_str("╚══════════════════════════════════════╝\n");
        }

        if !result.warnings.is_empty() {
            formatted_content.push_str("\n╔══════════════════════════════════════╗\n");
            formatted_content.push_str("║            ⚠️  Warnings              ║\n");
            formatted_content.push_str("╠══════════════════════════════════════╣\n");

            for warning in &result.warnings {
                let message = if warning.message.len() > 34 {
                    format!("{}...", &warning.message[..31])
                } else {
                    warning.message.clone()
                };
                formatted_content.push_str(&format!("║ • {:<34} ║\n", message));
            }

            formatted_content.push_str("╚══════════════════════════════════════╝\n");
        }

        FormattedError {
            message: if result.is_valid { "Validation passed".to_string() } else { "Validation failed".to_string() },
            formatted_content,
            metadata: HashMap::new(),
        }
    }

    fn format_error_message(&self, error: &ValidationError) -> String {
        format!("❌ {}", error.message)
    }

    fn interface_type(&self) -> InterfaceType {
        InterfaceType::TUI
    }
}

// Context Provider Implementation

/// Default validation context provider
pub struct DefaultContextProvider;

impl DefaultContextProvider {
    pub fn new() -> Self {
        Self
    }
}

impl ValidationContextProvider for DefaultContextProvider {
    fn get_context(&self, field_name: &str, interface_type: InterfaceType) -> ValidationContext {
        let field_type = match field_name {
            "username" => FieldType::Username,
            "password" => FieldType::Password,
            "email" => FieldType::Email,
            "project_name" => FieldType::ProjectName,
            name if name.contains("path") => FieldType::FilePath,
            name if name.contains("url") => FieldType::Url,
            _ => FieldType::Text,
        };

        ValidationContext {
            field_name: field_name.to_string(),
            field_type,
            interface_type,
            user_context: None,
            project_context: None,
            context_data: HashMap::new(),
            strictness: ValidationStrictness::Standard,
        }
    }

    fn update_context(&self, context: &mut ValidationContext, updates: HashMap<String, String>) {
        context.context_data.extend(updates);
    }
}
