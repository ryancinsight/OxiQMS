/// CUPID Principles Enhancement: Composable and Unix Philosophy
/// 
/// This module demonstrates CUPID principles:
/// - Composable: Services can be combined and reused
/// - Unix philosophy: Each service does one thing well
/// - Predictable: Consistent interfaces and behavior
/// - Idiomatic: Follows Rust best practices
/// - Domain-focused: Aligned with business domain concepts

use crate::prelude::*;
use std::collections::HashMap;

/// Composable service trait - CUPID Composable principle
/// Services can be combined and chained together
pub trait ComposableService<Input, Output> {
    fn process(&self, input: Input) -> QmsResult<Output>;
    fn service_name(&self) -> &'static str;
    fn can_compose_with<T>(&self, _other: &T) -> bool where T: ComposableService<Output, Input> {
        true // Default implementation allows composition
    }
}

/// Unix Philosophy: Risk validation service does one thing well
pub struct RiskValidationService;

impl ComposableService<RiskValidationInput, RiskValidationOutput> for RiskValidationService {
    fn process(&self, input: RiskValidationInput) -> QmsResult<RiskValidationOutput> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Validate severity
        if input.severity < 1 || input.severity > 5 {
            errors.push("Severity must be between 1 and 5".to_string());
        }
        
        // Validate occurrence
        if input.occurrence < 1 || input.occurrence > 5 {
            errors.push("Occurrence must be between 1 and 5".to_string());
        }
        
        // Validate detectability
        if input.detectability < 1 || input.detectability > 5 {
            errors.push("Detectability must be between 1 and 5".to_string());
        }
        
        // Generate warnings for edge cases
        if input.severity == 5 && input.occurrence >= 4 {
            warnings.push("High severity with high occurrence requires immediate attention".to_string());
        }
        
        let is_valid = errors.is_empty();
        Ok(RiskValidationOutput {
            is_valid,
            errors,
            warnings,
            validated_input: if is_valid { Some(input) } else { None },
        })
    }
    
    fn service_name(&self) -> &'static str {
        "RiskValidationService"
    }
}

/// Unix Philosophy: RPN calculation service does one thing well
pub struct RpnCalculationService;

impl ComposableService<RiskValidationOutput, RpnCalculationOutput> for RpnCalculationService {
    fn process(&self, input: RiskValidationOutput) -> QmsResult<RpnCalculationOutput> {
        if !input.is_valid {
            return Err(QmsError::validation_error("Cannot calculate RPN for invalid risk parameters"));
        }
        
        let validated = input.validated_input.ok_or_else(|| {
            QmsError::validation_error("No validated input available")
        })?;
        
        let rpn = validated.severity * validated.occurrence * validated.detectability;
        
        Ok(RpnCalculationOutput {
            rpn,
            severity: validated.severity,
            occurrence: validated.occurrence,
            detectability: validated.detectability,
            calculation_method: "Standard ISO 14971".to_string(),
        })
    }
    
    fn service_name(&self) -> &'static str {
        "RpnCalculationService"
    }
}

/// Unix Philosophy: Risk level assessment service does one thing well
pub struct RiskLevelAssessmentService;

impl ComposableService<RpnCalculationOutput, RiskLevelOutput> for RiskLevelAssessmentService {
    fn process(&self, input: RpnCalculationOutput) -> QmsResult<RiskLevelOutput> {
        let risk_level = match input.rpn {
            100..=u32::MAX => "Unacceptable",
            50..=99 => "ALARP",
            1..=49 => "Acceptable",
            _ => "Acceptable",
        };
        
        let is_acceptable = input.rpn <= 49;
        let requires_action = input.rpn >= 50;
        
        let recommendations = match input.rpn {
            100..=u32::MAX => vec![
                "Immediate action required".to_string(),
                "Consider design changes".to_string(),
                "Implement multiple risk controls".to_string(),
            ],
            50..=99 => vec![
                "Implement risk controls".to_string(),
                "Document risk acceptance rationale".to_string(),
            ],
            _ => vec![
                "Monitor during product lifecycle".to_string(),
            ],
        };
        
        Ok(RiskLevelOutput {
            rpn: input.rpn,
            risk_level: risk_level.to_string(),
            is_acceptable,
            requires_action,
            recommendations,
        })
    }
    
    fn service_name(&self) -> &'static str {
        "RiskLevelAssessmentService"
    }
}

/// Composable service pipeline - CUPID Composable principle
/// Allows chaining services together in a predictable way
pub struct ServicePipeline<T> {
    services: Vec<Box<dyn Fn(T) -> QmsResult<T>>>,
    pipeline_name: String,
}

impl<T> ServicePipeline<T> {
    pub fn new(name: &str) -> Self {
        Self {
            services: Vec::new(),
            pipeline_name: name.to_string(),
        }
    }
    
    pub fn add_service<F>(mut self, service: F) -> Self 
    where 
        F: Fn(T) -> QmsResult<T> + 'static,
    {
        self.services.push(Box::new(service));
        self
    }
    
    pub fn execute(&self, input: T) -> QmsResult<T> {
        let mut current = input;
        
        for (index, service) in self.services.iter().enumerate() {
            current = service(current).map_err(|e| {
                QmsError::validation_error(&format!(
                    "Pipeline '{}' failed at step {}: {}",
                    self.pipeline_name, index + 1, e
                ))
            })?;
        }
        
        Ok(current)
    }
}

/// Complete risk assessment pipeline - Composable services
pub struct RiskAssessmentPipeline {
    validation_service: RiskValidationService,
    calculation_service: RpnCalculationService,
    assessment_service: RiskLevelAssessmentService,
}

impl RiskAssessmentPipeline {
    pub fn new() -> Self {
        Self {
            validation_service: RiskValidationService,
            calculation_service: RpnCalculationService,
            assessment_service: RiskLevelAssessmentService,
        }
    }
    
    /// Execute complete risk assessment - Composable pattern
    pub fn assess_risk(&self, severity: u32, occurrence: u32, detectability: u32) -> QmsResult<RiskLevelOutput> {
        // Step 1: Validate input
        let validation_input = RiskValidationInput {
            severity,
            occurrence,
            detectability,
        };
        
        let validation_output = self.validation_service.process(validation_input)?;
        
        // Step 2: Calculate RPN
        let calculation_output = self.calculation_service.process(validation_output)?;
        
        // Step 3: Assess risk level
        let assessment_output = self.assessment_service.process(calculation_output)?;
        
        Ok(assessment_output)
    }
}

/// Input/Output types for composable services

#[derive(Debug, Clone)]
pub struct RiskValidationInput {
    pub severity: u32,
    pub occurrence: u32,
    pub detectability: u32,
}

#[derive(Debug, Clone)]
pub struct RiskValidationOutput {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub validated_input: Option<RiskValidationInput>,
}

#[derive(Debug, Clone)]
pub struct RpnCalculationOutput {
    pub rpn: u32,
    pub severity: u32,
    pub occurrence: u32,
    pub detectability: u32,
    pub calculation_method: String,
}

#[derive(Debug, Clone)]
pub struct RiskLevelOutput {
    pub rpn: u32,
    pub risk_level: String,
    pub is_acceptable: bool,
    pub requires_action: bool,
    pub recommendations: Vec<String>,
}

/// Service registry for managing composable services - CUPID Predictable
pub struct ServiceRegistry {
    services: HashMap<String, String>, // service_name -> description
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }
    
    pub fn register_service(&mut self, name: &str, description: &str) {
        self.services.insert(name.to_string(), description.to_string());
    }
    
    pub fn list_services(&self) -> Vec<(String, String)> {
        self.services.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
    
    pub fn get_service_description(&self, name: &str) -> Option<&String> {
        self.services.get(name)
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register_service("RiskValidationService", "Validates risk assessment parameters");
        registry.register_service("RpnCalculationService", "Calculates Risk Priority Number");
        registry.register_service("RiskLevelAssessmentService", "Assesses risk acceptability level");
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_composable_services_unix_philosophy() {
        let validation_service = RiskValidationService;
        let calculation_service = RpnCalculationService;
        let assessment_service = RiskLevelAssessmentService;
        
        // Each service does one thing well
        assert_eq!(validation_service.service_name(), "RiskValidationService");
        assert_eq!(calculation_service.service_name(), "RpnCalculationService");
        assert_eq!(assessment_service.service_name(), "RiskLevelAssessmentService");
    }
    
    #[test]
    fn test_risk_assessment_pipeline_composable() {
        let pipeline = RiskAssessmentPipeline::new();
        
        let result = pipeline.assess_risk(4, 4, 4).unwrap();
        
        assert_eq!(result.rpn, 64);
        assert_eq!(result.risk_level, "ALARP");
        assert!(!result.is_acceptable);
        assert!(result.requires_action);
        assert!(!result.recommendations.is_empty());
    }
    
    #[test]
    fn test_service_registry_predictable() {
        let registry = ServiceRegistry::default();
        
        let services = registry.list_services();
        assert_eq!(services.len(), 3);
        
        let description = registry.get_service_description("RiskValidationService");
        assert!(description.is_some());
        assert_eq!(description.unwrap(), "Validates risk assessment parameters");
    }
    
    #[test]
    fn test_validation_service_idiomatic() {
        let service = RiskValidationService;
        
        // Valid input
        let valid_input = RiskValidationInput {
            severity: 3,
            occurrence: 2,
            detectability: 4,
        };
        
        let result = service.process(valid_input).unwrap();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
        
        // Invalid input
        let invalid_input = RiskValidationInput {
            severity: 6, // Invalid
            occurrence: 2,
            detectability: 4,
        };
        
        let result = service.process(invalid_input).unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }
}
