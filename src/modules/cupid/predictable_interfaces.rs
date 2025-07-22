/// CUPID Principles Enhancement: Predictable and Idiomatic Interfaces
/// 
/// This module demonstrates:
/// - Predictable: Consistent behavior and naming conventions
/// - Idiomatic: Following Rust best practices and conventions
/// - Domain-focused: Clear business domain alignment

use crate::prelude::*;
use std::fmt;

/// Predictable interface for all QMS domain operations
/// Consistent naming and behavior across all implementations
pub trait PredictableOperation<Input, Output> {
    /// Execute the operation - predictable naming
    fn execute(&self, input: Input) -> QmsResult<Output>;
    
    /// Validate input before execution - predictable validation
    fn validate_input(&self, input: &Input) -> QmsResult<()>;
    
    /// Get operation metadata - predictable introspection
    fn operation_info(&self) -> OperationInfo;
    
    /// Check if operation can be executed - predictable preconditions
    fn can_execute(&self, input: &Input) -> bool {
        self.validate_input(input).is_ok()
    }
}

/// Operation metadata - predictable structure
#[derive(Debug, Clone)]
pub struct OperationInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    pub domain: String,
    pub input_type: String,
    pub output_type: String,
}

/// Idiomatic Rust builder pattern for operation configuration
#[derive(Debug, Clone)]
pub struct OperationConfig {
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub enable_logging: bool,
    pub enable_caching: bool,
    pub validation_level: ValidationLevel,
}

impl Default for OperationConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            retry_attempts: 3,
            enable_logging: true,
            enable_caching: true,
            validation_level: ValidationLevel::Standard,
        }
    }
}

impl OperationConfig {
    /// Idiomatic builder pattern
    pub fn builder() -> OperationConfigBuilder {
        OperationConfigBuilder::default()
    }
}

/// Idiomatic builder for operation configuration
#[derive(Debug, Default)]
pub struct OperationConfigBuilder {
    timeout_seconds: Option<u64>,
    retry_attempts: Option<u32>,
    enable_logging: Option<bool>,
    enable_caching: Option<bool>,
    validation_level: Option<ValidationLevel>,
}

impl OperationConfigBuilder {
    pub fn timeout_seconds(mut self, seconds: u64) -> Self {
        self.timeout_seconds = Some(seconds);
        self
    }
    
    pub fn retry_attempts(mut self, attempts: u32) -> Self {
        self.retry_attempts = Some(attempts);
        self
    }
    
    pub fn enable_logging(mut self, enable: bool) -> Self {
        self.enable_logging = Some(enable);
        self
    }
    
    pub fn enable_caching(mut self, enable: bool) -> Self {
        self.enable_caching = Some(enable);
        self
    }
    
    pub fn validation_level(mut self, level: ValidationLevel) -> Self {
        self.validation_level = Some(level);
        self
    }
    
    pub fn build(self) -> OperationConfig {
        OperationConfig {
            timeout_seconds: self.timeout_seconds.unwrap_or(30),
            retry_attempts: self.retry_attempts.unwrap_or(3),
            enable_logging: self.enable_logging.unwrap_or(true),
            enable_caching: self.enable_caching.unwrap_or(true),
            validation_level: self.validation_level.unwrap_or(ValidationLevel::Standard),
        }
    }
}

/// Validation levels - predictable enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidationLevel {
    None,
    Basic,
    Standard,
    Strict,
    Regulatory,
}

impl fmt::Display for ValidationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationLevel::None => write!(f, "None"),
            ValidationLevel::Basic => write!(f, "Basic"),
            ValidationLevel::Standard => write!(f, "Standard"),
            ValidationLevel::Strict => write!(f, "Strict"),
            ValidationLevel::Regulatory => write!(f, "Regulatory"),
        }
    }
}

/// Predictable risk assessment operation
pub struct PredictableRiskAssessment {
    config: OperationConfig,
}

impl PredictableRiskAssessment {
    /// Idiomatic constructor
    pub fn new() -> Self {
        Self {
            config: OperationConfig::default(),
        }
    }
    
    /// Idiomatic constructor with configuration
    pub fn with_config(config: OperationConfig) -> Self {
        Self { config }
    }
    
    /// Idiomatic method for updating configuration
    pub fn configure(&mut self, config: OperationConfig) {
        self.config = config;
    }
}

impl Default for PredictableRiskAssessment {
    fn default() -> Self {
        Self::new()
    }
}

/// Input for risk assessment - predictable structure
#[derive(Debug, Clone)]
pub struct RiskAssessmentInput {
    pub risk_id: String,
    pub severity: u8,
    pub occurrence: u8,
    pub detectability: u8,
    pub description: String,
}

/// Output for risk assessment - predictable structure
#[derive(Debug, Clone)]
pub struct RiskAssessmentOutput {
    pub risk_id: String,
    pub rpn: u32,
    pub risk_level: String,
    pub is_acceptable: bool,
    pub recommendations: Vec<String>,
    pub assessment_timestamp: String,
    pub assessor: String,
}

impl PredictableOperation<RiskAssessmentInput, RiskAssessmentOutput> for PredictableRiskAssessment {
    fn execute(&self, input: RiskAssessmentInput) -> QmsResult<RiskAssessmentOutput> {
        // Predictable validation step
        self.validate_input(&input)?;
        
        // Predictable calculation
        let rpn = (input.severity as u32) * (input.occurrence as u32) * (input.detectability as u32);
        
        // Predictable risk level determination
        let risk_level = match rpn {
            100..=u32::MAX => "Unacceptable",
            50..=99 => "ALARP",
            1..=49 => "Acceptable",
            _ => "Acceptable",
        };
        
        let is_acceptable = rpn <= 49;
        
        // Predictable recommendations
        let recommendations = self.generate_recommendations(rpn, &input);
        
        Ok(RiskAssessmentOutput {
            risk_id: input.risk_id,
            rpn,
            risk_level: risk_level.to_string(),
            is_acceptable,
            recommendations,
            assessment_timestamp: crate::utils::current_timestamp_string(),
            assessor: "System".to_string(),
        })
    }
    
    fn validate_input(&self, input: &RiskAssessmentInput) -> QmsResult<()> {
        // Predictable validation based on configuration
        match self.config.validation_level {
            ValidationLevel::None => Ok(()),
            ValidationLevel::Basic => self.basic_validation(input),
            ValidationLevel::Standard => self.standard_validation(input),
            ValidationLevel::Strict => self.strict_validation(input),
            ValidationLevel::Regulatory => self.regulatory_validation(input),
        }
    }
    
    fn operation_info(&self) -> OperationInfo {
        OperationInfo {
            name: "RiskAssessment".to_string(),
            description: "Assesses risk based on severity, occurrence, and detectability".to_string(),
            version: "1.0.0".to_string(),
            domain: "RiskManagement".to_string(),
            input_type: "RiskAssessmentInput".to_string(),
            output_type: "RiskAssessmentOutput".to_string(),
        }
    }
}

impl PredictableRiskAssessment {
    /// Predictable validation methods
    fn basic_validation(&self, input: &RiskAssessmentInput) -> QmsResult<()> {
        if input.risk_id.is_empty() {
            return Err(QmsError::validation_error("Risk ID cannot be empty"));
        }
        Ok(())
    }
    
    fn standard_validation(&self, input: &RiskAssessmentInput) -> QmsResult<()> {
        self.basic_validation(input)?;
        
        if !(1..=5).contains(&input.severity) {
            return Err(QmsError::validation_error("Severity must be between 1 and 5"));
        }
        
        if !(1..=5).contains(&input.occurrence) {
            return Err(QmsError::validation_error("Occurrence must be between 1 and 5"));
        }
        
        if !(1..=5).contains(&input.detectability) {
            return Err(QmsError::validation_error("Detectability must be between 1 and 5"));
        }
        
        Ok(())
    }
    
    fn strict_validation(&self, input: &RiskAssessmentInput) -> QmsResult<()> {
        self.standard_validation(input)?;
        
        if input.description.len() < 10 {
            return Err(QmsError::validation_error("Description must be at least 10 characters"));
        }
        
        if input.description.len() > 1000 {
            return Err(QmsError::validation_error("Description must be less than 1000 characters"));
        }
        
        Ok(())
    }
    
    fn regulatory_validation(&self, input: &RiskAssessmentInput) -> QmsResult<()> {
        self.strict_validation(input)?;
        
        // Additional regulatory checks
        if !input.risk_id.starts_with("RISK-") {
            return Err(QmsError::validation_error("Risk ID must start with 'RISK-' for regulatory compliance"));
        }
        
        // Check for required keywords in description for medical devices
        let required_keywords = ["hazard", "harm", "patient", "user"];
        let description_lower = input.description.to_lowercase();
        
        if !required_keywords.iter().any(|&keyword| description_lower.contains(keyword)) {
            return Err(QmsError::validation_error(
                "Description must contain at least one of: hazard, harm, patient, user"
            ));
        }
        
        Ok(())
    }
    
    /// Predictable recommendation generation
    fn generate_recommendations(&self, rpn: u32, input: &RiskAssessmentInput) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Standard recommendations based on RPN
        match rpn {
            100..=u32::MAX => {
                recommendations.push("IMMEDIATE ACTION REQUIRED: Risk is unacceptable".to_string());
                recommendations.push("Stop production until risk is mitigated".to_string());
                recommendations.push("Implement multiple risk control measures".to_string());
            }
            50..=99 => {
                recommendations.push("Risk requires mitigation before release".to_string());
                recommendations.push("Implement at least one risk control measure".to_string());
                recommendations.push("Document risk acceptance rationale".to_string());
            }
            25..=49 => {
                recommendations.push("Consider additional risk controls".to_string());
                recommendations.push("Monitor risk during product lifecycle".to_string());
            }
            _ => {
                recommendations.push("Risk is acceptable".to_string());
                recommendations.push("Continue monitoring during post-market surveillance".to_string());
            }
        }
        
        // Additional recommendations based on individual parameters
        if input.severity >= 4 {
            recommendations.push("High severity requires additional design review".to_string());
        }
        
        if input.occurrence >= 4 {
            recommendations.push("High occurrence probability requires process improvements".to_string());
        }
        
        if input.detectability >= 4 {
            recommendations.push("Poor detectability requires improved testing procedures".to_string());
        }
        
        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_predictable_operation_interface() {
        let operation = PredictableRiskAssessment::new();
        
        let input = RiskAssessmentInput {
            risk_id: "RISK-001".to_string(),
            severity: 3,
            occurrence: 2,
            detectability: 4,
            description: "Test hazard that could cause harm to patient".to_string(),
        };
        
        // Predictable interface methods
        assert!(operation.can_execute(&input));
        assert!(operation.validate_input(&input).is_ok());
        
        let info = operation.operation_info();
        assert_eq!(info.name, "RiskAssessment");
        assert_eq!(info.domain, "RiskManagement");
        
        let result = operation.execute(input).unwrap();
        assert_eq!(result.rpn, 24); // 3 * 2 * 4
        assert_eq!(result.risk_level, "Acceptable");
        assert!(result.is_acceptable);
    }
    
    #[test]
    fn test_idiomatic_builder_pattern() {
        let config = OperationConfig::builder()
            .timeout_seconds(60)
            .retry_attempts(5)
            .enable_logging(false)
            .validation_level(ValidationLevel::Strict)
            .build();
        
        assert_eq!(config.timeout_seconds, 60);
        assert_eq!(config.retry_attempts, 5);
        assert!(!config.enable_logging);
        assert_eq!(config.validation_level, ValidationLevel::Strict);
    }
    
    #[test]
    fn test_validation_levels_predictable() {
        let mut operation = PredictableRiskAssessment::new();
        
        let input = RiskAssessmentInput {
            risk_id: "INVALID".to_string(), // Invalid for regulatory
            severity: 3,
            occurrence: 2,
            detectability: 4,
            description: "Short".to_string(), // Too short for strict
        };
        
        // Basic validation should pass
        let config = OperationConfig::builder()
            .validation_level(ValidationLevel::Basic)
            .build();
        operation.configure(config);
        assert!(operation.validate_input(&input).is_ok());
        
        // Strict validation should fail
        let config = OperationConfig::builder()
            .validation_level(ValidationLevel::Strict)
            .build();
        operation.configure(config);
        assert!(operation.validate_input(&input).is_err());
        
        // Regulatory validation should fail
        let config = OperationConfig::builder()
            .validation_level(ValidationLevel::Regulatory)
            .build();
        operation.configure(config);
        assert!(operation.validate_input(&input).is_err());
    }
}
