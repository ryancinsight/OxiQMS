//! Unified Validation and Error Handling Service
//! 
//! Consolidates validation rules and error handling across CLI, TUI, and web interfaces
//! following SOLID principles and DRY methodology.

use crate::prelude::*;
use crate::utils::common_validation::{CommonValidation, ValidationResult};
use crate::utils::error_handling::ErrorHandler;
use crate::modules::audit_logger::audit_log_action;
use std::collections::HashMap;
use std::sync::Arc;

/// Validation Service Interface
/// 
/// Provides unified validation capabilities that can be used by all interfaces,
/// eliminating validation code duplication and ensuring consistent validation rules.
pub trait ValidationServiceInterface: Send + Sync {
    /// Validate entity data
    fn validate_entity(&self, entity_type: &str, data: &EntityData) -> QmsResult<ValidationResult>;

    /// Validate field value
    fn validate_field(&self, field_name: &str, field_value: &str, field_type: FieldType) -> QmsResult<FieldValidationResult>;

    /// Validate form data
    fn validate_form(&self, form_id: &str, form_data: &FormData) -> QmsResult<FormValidationResult>;

    /// Get validation rules for entity type
    fn get_validation_rules(&self, entity_type: &str) -> Vec<ValidationRule>;

    /// Register custom validation rule
    fn register_validation_rule(&self, entity_type: &str, rule: ValidationRule) -> QmsResult<()>;

    /// Format error for interface type
    fn format_error(&self, error: &QmsError, interface_type: InterfaceType) -> ErrorResponse;

    /// Create standardized error response
    fn create_error_response(&self, error_type: ErrorType, message: &str, details: Option<&str>) -> ErrorResponse;

    /// Validate business rules
    fn validate_business_rules(&self, operation: &str, context: &BusinessRuleContext) -> QmsResult<()>;
}

/// Entity data for validation
#[derive(Debug, Clone)]
pub struct EntityData {
    pub fields: HashMap<String, String>,
    pub metadata: HashMap<String, String>,
}

/// Field type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Text,
    Email,
    Number,
    Date,
    Id,
    Username,
    Password,
    ProjectName,
    DocumentTitle,
    RiskDescription,
    Custom(String),
}

/// Field validation result
#[derive(Debug, Clone)]
pub struct FieldValidationResult {
    pub is_valid: bool,
    pub error: Option<String>,
    pub warning: Option<String>,
    pub suggestions: Vec<String>,
}

/// Form data for validation
#[derive(Debug, Clone)]
pub struct FormData {
    pub fields: HashMap<String, String>,
    pub form_type: String,
    pub context: HashMap<String, String>,
}

/// Form validation result
#[derive(Debug, Clone)]
pub struct FormValidationResult {
    pub is_valid: bool,
    pub field_errors: HashMap<String, String>,
    pub form_errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Validation rule definition
#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub name: String,
    pub field_name: String,
    pub rule_type: ValidationRuleType,
    pub error_message: String,
    pub is_required: bool,
}

/// Validation rule types
#[derive(Debug, Clone)]
pub enum ValidationRuleType {
    Required,
    MinLength(usize),
    MaxLength(usize),
    Pattern(String),
    Range(f64, f64),
    Custom(String),
}

/// Error response for different interfaces
#[derive(Debug, Clone)]
pub struct ErrorResponse {
    pub error_type: ErrorType,
    pub message: String,
    pub details: Option<String>,
    pub field_errors: HashMap<String, String>,
    pub error_code: String,
    pub timestamp: u64,
    pub interface_format: InterfaceErrorFormat,
}

/// Error type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    Validation,
    NotFound,
    Permission,
    Authentication,
    BusinessRule,
    System,
    Network,
}

/// Interface-specific error formats
#[derive(Debug, Clone)]
pub enum InterfaceErrorFormat {
    Cli(String),
    Web(String),
    Tui(String),
}

/// Business rule context
#[derive(Debug, Clone)]
pub struct BusinessRuleContext {
    pub operation: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub user_id: String,
    pub data: HashMap<String, String>,
}

/// Interface type for error formatting
#[derive(Debug, Clone, PartialEq)]
pub enum InterfaceType {
    CLI,
    Web,
    TUI,
}

/// Unified Validation Service Implementation
/// 
/// Central validation coordinator that manages all validation rules and error handling
/// using the existing validation utilities and error handling patterns.
pub struct UnifiedValidationService {
    validation_rules: Arc<std::sync::Mutex<HashMap<String, Vec<ValidationRule>>>>,
    business_rules: Arc<std::sync::Mutex<HashMap<String, Vec<BusinessRule>>>>,
}

/// Business rule definition
#[derive(Debug, Clone)]
pub struct BusinessRule {
    pub name: String,
    pub operation: String,
    pub condition: String,
    pub error_message: String,
}

impl UnifiedValidationService {
    /// Create new unified validation service
    pub fn new() -> Self {
        let mut validation_rules = HashMap::new();
        let mut business_rules = HashMap::new();

        // Initialize default validation rules
        Self::initialize_default_validation_rules(&mut validation_rules);
        Self::initialize_default_business_rules(&mut business_rules);

        Self {
            validation_rules: Arc::new(std::sync::Mutex::new(validation_rules)),
            business_rules: Arc::new(std::sync::Mutex::new(business_rules)),
        }
    }

    /// Initialize default validation rules for all entity types
    fn initialize_default_validation_rules(rules: &mut HashMap<String, Vec<ValidationRule>>) {
        // Document validation rules
        let document_rules = vec![
            ValidationRule {
                name: "title_required".to_string(),
                field_name: "title".to_string(),
                rule_type: ValidationRuleType::Required,
                error_message: "Document title is required".to_string(),
                is_required: true,
            },
            ValidationRule {
                name: "title_length".to_string(),
                field_name: "title".to_string(),
                rule_type: ValidationRuleType::MaxLength(200),
                error_message: "Document title must not exceed 200 characters".to_string(),
                is_required: false,
            },
        ];
        rules.insert("document".to_string(), document_rules);

        // Risk validation rules
        let risk_rules = vec![
            ValidationRule {
                name: "description_required".to_string(),
                field_name: "description".to_string(),
                rule_type: ValidationRuleType::Required,
                error_message: "Risk description is required".to_string(),
                is_required: true,
            },
            ValidationRule {
                name: "severity_range".to_string(),
                field_name: "severity".to_string(),
                rule_type: ValidationRuleType::Range(1.0, 5.0),
                error_message: "Risk severity must be between 1 and 5".to_string(),
                is_required: false,
            },
        ];
        rules.insert("risk".to_string(), risk_rules);

        // User validation rules
        let user_rules = vec![
            ValidationRule {
                name: "username_required".to_string(),
                field_name: "username".to_string(),
                rule_type: ValidationRuleType::Required,
                error_message: "Username is required".to_string(),
                is_required: true,
            },
            ValidationRule {
                name: "username_length".to_string(),
                field_name: "username".to_string(),
                rule_type: ValidationRuleType::MinLength(3),
                error_message: "Username must be at least 3 characters".to_string(),
                is_required: false,
            },
        ];
        rules.insert("user".to_string(), user_rules);

        // Project validation rules
        let project_rules = vec![
            ValidationRule {
                name: "name_required".to_string(),
                field_name: "name".to_string(),
                rule_type: ValidationRuleType::Required,
                error_message: "Project name is required".to_string(),
                is_required: true,
            },
            ValidationRule {
                name: "name_length".to_string(),
                field_name: "name".to_string(),
                rule_type: ValidationRuleType::MaxLength(100),
                error_message: "Project name must not exceed 100 characters".to_string(),
                is_required: false,
            },
        ];
        rules.insert("project".to_string(), project_rules);
    }

    /// Initialize default business rules
    fn initialize_default_business_rules(rules: &mut HashMap<String, Vec<BusinessRule>>) {
        // Document business rules
        let document_business_rules = vec![
            BusinessRule {
                name: "document_unique_title".to_string(),
                operation: "create".to_string(),
                condition: "title_must_be_unique".to_string(),
                error_message: "A document with this title already exists".to_string(),
            },
        ];
        rules.insert("document".to_string(), document_business_rules);

        // Risk business rules
        let risk_business_rules = vec![
            BusinessRule {
                name: "risk_assessment_complete".to_string(),
                operation: "approve".to_string(),
                condition: "all_assessments_complete".to_string(),
                error_message: "All risk assessments must be complete before approval".to_string(),
            },
        ];
        rules.insert("risk".to_string(), risk_business_rules);
    }

    /// Apply validation rule to field value
    fn apply_validation_rule(&self, rule: &ValidationRule, value: &str) -> FieldValidationResult {
        let mut result = FieldValidationResult {
            is_valid: true,
            error: None,
            warning: None,
            suggestions: Vec::new(),
        };

        match &rule.rule_type {
            ValidationRuleType::Required => {
                if value.trim().is_empty() {
                    result.is_valid = false;
                    result.error = Some(rule.error_message.clone());
                }
            }
            ValidationRuleType::MinLength(min_len) => {
                if value.len() < *min_len {
                    result.is_valid = false;
                    result.error = Some(rule.error_message.clone());
                }
            }
            ValidationRuleType::MaxLength(max_len) => {
                if value.len() > *max_len {
                    result.is_valid = false;
                    result.error = Some(rule.error_message.clone());
                }
            }
            ValidationRuleType::Pattern(pattern) => {
                // Simplified pattern matching - in real implementation would use regex
                if !value.contains(pattern) {
                    result.is_valid = false;
                    result.error = Some(rule.error_message.clone());
                }
            }
            ValidationRuleType::Range(min, max) => {
                if let Ok(num_value) = value.parse::<f64>() {
                    if num_value < *min || num_value > *max {
                        result.is_valid = false;
                        result.error = Some(rule.error_message.clone());
                    }
                } else {
                    result.is_valid = false;
                    result.error = Some("Value must be a number".to_string());
                }
            }
            ValidationRuleType::Custom(_custom_rule) => {
                // Custom validation logic would be implemented here
                // For now, just pass through
            }
        }

        result
    }

    /// Validate field using existing validation utilities
    fn validate_field_with_utilities(&self, field_name: &str, field_value: &str, field_type: FieldType) -> FieldValidationResult {
        let mut result = FieldValidationResult {
            is_valid: true,
            error: None,
            warning: None,
            suggestions: Vec::new(),
        };

        match field_type {
            FieldType::Email => {
                if !CommonValidation::validate_email(field_value) {
                    result.is_valid = false;
                    result.error = Some("Invalid email format".to_string());
                }
            }
            FieldType::Username => {
                if !CommonValidation::validate_username(field_value) {
                    result.is_valid = false;
                    result.error = Some("Invalid username format".to_string());
                }
            }
            FieldType::ProjectName => {
                if !CommonValidation::validate_project_name(field_value) {
                    result.is_valid = false;
                    result.error = Some("Invalid project name format".to_string());
                }
            }
            FieldType::Date => {
                if !CommonValidation::validate_date_string(field_value) {
                    result.is_valid = false;
                    result.error = Some("Invalid date format (expected YYYY-MM-DD)".to_string());
                }
            }
            FieldType::Id => {
                // Generic ID validation - would be more specific in real implementation
                if field_value.len() < 3 {
                    result.is_valid = false;
                    result.error = Some("ID must be at least 3 characters".to_string());
                }
            }
            FieldType::Number => {
                if field_value.parse::<f64>().is_err() {
                    result.is_valid = false;
                    result.error = Some("Must be a valid number".to_string());
                }
            }
            FieldType::Text => {
                if !CommonValidation::validate_text_content(field_value) {
                    result.is_valid = false;
                    result.error = Some("Invalid text content".to_string());
                }
            }
            FieldType::Password => {
                // Basic password validation - would be more comprehensive in real implementation
                if field_value.len() < 8 {
                    result.is_valid = false;
                    result.error = Some("Password must be at least 8 characters".to_string());
                }
            }
            FieldType::DocumentTitle => {
                if !CommonValidation::validate_string_length(field_value, 1, 200) {
                    result.is_valid = false;
                    result.error = Some("Document title must be 1-200 characters".to_string());
                }
            }
            FieldType::RiskDescription => {
                if !CommonValidation::validate_text_content(field_value) {
                    result.is_valid = false;
                    result.error = Some("Invalid risk description".to_string());
                }
            }
            FieldType::Custom(_) => {
                // Custom field validation would be implemented here
            }
        }

        result
    }

    /// Log validation event for audit
    fn log_validation(&self, operation: &str, entity_type: &str, success: bool) {
        let event_type = if success {
            "VALIDATION_SUCCESS"
        } else {
            "VALIDATION_FAILED"
        };

        let _ = audit_log_action(
            event_type,
            "Validation",
            &format!("{}:{}", operation, entity_type),
        );
    }
}

impl ValidationServiceInterface for UnifiedValidationService {
    fn validate_entity(&self, entity_type: &str, data: &EntityData) -> QmsResult<ValidationResult> {
        let mut validation_result = ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Get validation rules for entity type
        let rules = self.get_validation_rules(entity_type);

        // Apply each rule to the corresponding field
        for rule in rules {
            if let Some(field_value) = data.fields.get(&rule.field_name) {
                let field_result = self.apply_validation_rule(&rule, field_value);
                if !field_result.is_valid {
                    validation_result.is_valid = false;
                    if let Some(error) = field_result.error {
                        validation_result.errors.push(error);
                    }
                }
                if let Some(warning) = field_result.warning {
                    validation_result.warnings.push(warning);
                }
            } else if rule.is_required {
                validation_result.is_valid = false;
                validation_result.errors.push(format!("Required field '{}' is missing", rule.field_name));
            }
        }

        self.log_validation("validate_entity", entity_type, validation_result.is_valid);

        Ok(validation_result)
    }

    fn validate_field(&self, field_name: &str, field_value: &str, field_type: FieldType) -> QmsResult<FieldValidationResult> {
        let result = self.validate_field_with_utilities(field_name, field_value, field_type);
        
        self.log_validation("validate_field", field_name, result.is_valid);
        
        Ok(result)
    }

    fn validate_form(&self, form_id: &str, form_data: &FormData) -> QmsResult<FormValidationResult> {
        let mut result = FormValidationResult {
            is_valid: true,
            field_errors: HashMap::new(),
            form_errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Validate each field in the form
        for (field_name, field_value) in &form_data.fields {
            // Determine field type based on field name (simplified)
            let field_type = match field_name.as_str() {
                name if name.contains("email") => FieldType::Email,
                name if name.contains("username") => FieldType::Username,
                name if name.contains("password") => FieldType::Password,
                name if name.contains("date") => FieldType::Date,
                name if name.contains("number") || name.contains("count") => FieldType::Number,
                _ => FieldType::Text,
            };

            let field_result = self.validate_field_with_utilities(field_name, field_value, field_type);
            if !field_result.is_valid {
                result.is_valid = false;
                if let Some(error) = field_result.error {
                    result.field_errors.insert(field_name.clone(), error);
                }
            }
        }

        self.log_validation("validate_form", form_id, result.is_valid);

        Ok(result)
    }

    fn get_validation_rules(&self, entity_type: &str) -> Vec<ValidationRule> {
        if let Ok(rules) = self.validation_rules.lock() {
            rules.get(entity_type).cloned().unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    fn register_validation_rule(&self, entity_type: &str, rule: ValidationRule) -> QmsResult<()> {
        if let Ok(mut rules) = self.validation_rules.lock() {
            rules.entry(entity_type.to_string())
                .or_insert_with(Vec::new)
                .push(rule);
            Ok(())
        } else {
            Err(QmsError::domain_error("Failed to acquire validation rules lock"))
        }
    }

    fn format_error(&self, error: &QmsError, interface_type: InterfaceType) -> ErrorResponse {
        let error_type = match error {
            QmsError::Validation(_) => ErrorType::Validation,
            QmsError::NotFound(_) => ErrorType::NotFound,
            QmsError::Permission(_) => ErrorType::Permission,
            QmsError::Authentication(_) => ErrorType::Authentication,
            _ => ErrorType::System,
        };

        let message = error.to_string();
        let error_code = format!("{:?}", error_type).to_uppercase();

        let interface_format = match interface_type {
            InterfaceType::CLI => InterfaceErrorFormat::Cli(format!("Error: {}", message)),
            InterfaceType::Web => InterfaceErrorFormat::Web(format!(
                r#"{{"success": false, "error": "{}", "code": "{}"}}"#,
                message, error_code
            )),
            InterfaceType::TUI => InterfaceErrorFormat::Tui(format!("❌ {}", message)),
        };

        ErrorResponse {
            error_type,
            message,
            details: None,
            field_errors: HashMap::new(),
            error_code,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            interface_format,
        }
    }

    fn create_error_response(&self, error_type: ErrorType, message: &str, details: Option<&str>) -> ErrorResponse {
        let error_code = format!("{:?}", error_type).to_uppercase();

        ErrorResponse {
            error_type,
            message: message.to_string(),
            details: details.map(|s| s.to_string()),
            field_errors: HashMap::new(),
            error_code,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            interface_format: InterfaceErrorFormat::Cli("".to_string()), // Default format
        }
    }

    fn validate_business_rules(&self, operation: &str, context: &BusinessRuleContext) -> QmsResult<()> {
        if let Ok(rules) = self.business_rules.lock() {
            if let Some(entity_rules) = rules.get(&context.entity_type) {
                for rule in entity_rules {
                    if rule.operation == operation {
                        // Simplified business rule validation
                        // In a real implementation, this would evaluate the condition
                        // against the context data
                        
                        // For demonstration, just log the validation
                        self.log_validation("validate_business_rules", &context.entity_type, true);
                    }
                }
            }
        }

        Ok(())
    }
}
