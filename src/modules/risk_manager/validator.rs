//! Risk Validator - Validation Logic Layer
//! 
//! SRP (Single Responsibility Principle): Handles only validation operations
//! Extracted from RiskManager to separate validation concerns
//! 
//! Medical Device Compliance: Ensures ISO 14971 compliance in risk assessments

use crate::prelude::*;
use crate::constants::{iso_14971, validation};
use super::risk::{RiskItem, RiskSeverity, RiskOccurrence, RiskDetectability};

/// Risk Validator Trait - Single Responsibility: Validation only
/// 
/// Abstracts validation logic to enable:
/// - Different validation strategies (ISO 14971, FDA, etc.)
/// - Dependency injection for testing
/// - Clear separation from business logic and persistence
pub trait RiskValidator {
    /// Validate a complete risk item
    fn validate_risk(&self, risk: &RiskItem) -> QmsResult<ValidationResult>;
    
    /// Validate risk parameters (severity, occurrence, detectability)
    fn validate_risk_parameters(
        &self, 
        severity: &RiskSeverity, 
        occurrence: &RiskOccurrence, 
        detectability: &RiskDetectability
    ) -> QmsResult<()>;
    
    /// Validate risk identification fields
    fn validate_risk_identification(&self, risk: &RiskItem) -> QmsResult<()>;
    
    /// Validate mitigation measures
    fn validate_mitigation_measures(&self, risk: &RiskItem) -> QmsResult<()>;
    
    /// Validate RPN calculation
    fn validate_rpn(&self, rpn: u32) -> QmsResult<()>;
}

/// Validation Result Structure
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    pub fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }
    
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// ISO 14971 Risk Validator Implementation
/// 
/// Validates risks according to ISO 14971:2019 standard
/// Ensures medical device risk management compliance
pub struct Iso14971RiskValidator;

impl Iso14971RiskValidator {
    pub fn new() -> Self {
        Self
    }
    
    /// Validate hazard ID format per ISO 14971
    fn validate_hazard_id(&self, hazard_id: &str) -> QmsResult<()> {
        if hazard_id.is_empty() {
            return Err(QmsError::validation_error("Hazard ID cannot be empty"));
        }
        
        if hazard_id.len() < validation::MIN_ID_LENGTH || hazard_id.len() > validation::MAX_ID_LENGTH {
            return Err(QmsError::validation_error(&format!(
                "Hazard ID length must be between {} and {} characters",
                validation::MIN_ID_LENGTH,
                validation::MAX_ID_LENGTH
            )));
        }
        
        // Check if follows HAZ-XXX pattern
        if !hazard_id.starts_with("HAZ-") {
            return Err(QmsError::validation_error("Hazard ID must start with 'HAZ-'"));
        }
        
        Ok(())
    }
    
    /// Validate description fields per ISO 14971 requirements
    fn validate_descriptions(&self, risk: &RiskItem) -> QmsResult<()> {
        // Hazard description validation
        if risk.hazard_description.trim().is_empty() {
            return Err(QmsError::validation_error("Hazard description is required"));
        }
        
        if risk.hazard_description.len() > validation::MAX_DOCUMENT_DESCRIPTION_LENGTH {
            return Err(QmsError::validation_error(&format!(
                "Hazard description exceeds maximum length of {} characters",
                validation::MAX_DOCUMENT_DESCRIPTION_LENGTH
            )));
        }
        
        // Hazardous situation validation
        if risk.hazardous_situation.trim().is_empty() {
            return Err(QmsError::validation_error("Hazardous situation description is required"));
        }
        
        // Harm description validation
        if risk.harm.trim().is_empty() {
            return Err(QmsError::validation_error("Harm description is required"));
        }
        
        Ok(())
    }
    
    /// Validate risk scale values per ISO 14971
    fn validate_risk_scale_values(&self, severity: &RiskSeverity, occurrence: &RiskOccurrence, detectability: &RiskDetectability) -> QmsResult<()> {
        let severity_val = severity.clone() as u8;
        let occurrence_val = occurrence.clone() as u8;
        let detectability_val = detectability.clone() as u8;
        
        if !(iso_14971::MIN_RISK_SCALE..=iso_14971::MAX_RISK_SCALE).contains(&severity_val) {
            return Err(QmsError::validation_error(&format!(
                "Severity value {} is outside valid range {}-{}",
                severity_val, iso_14971::MIN_RISK_SCALE, iso_14971::MAX_RISK_SCALE
            )));
        }
        
        if !(iso_14971::MIN_RISK_SCALE..=iso_14971::MAX_RISK_SCALE).contains(&occurrence_val) {
            return Err(QmsError::validation_error(&format!(
                "Occurrence value {} is outside valid range {}-{}",
                occurrence_val, iso_14971::MIN_RISK_SCALE, iso_14971::MAX_RISK_SCALE
            )));
        }
        
        if !(iso_14971::MIN_RISK_SCALE..=iso_14971::MAX_RISK_SCALE).contains(&detectability_val) {
            return Err(QmsError::validation_error(&format!(
                "Detectability value {} is outside valid range {}-{}",
                detectability_val, iso_14971::MIN_RISK_SCALE, iso_14971::MAX_RISK_SCALE
            )));
        }
        
        Ok(())
    }
}

impl Default for Iso14971RiskValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl RiskValidator for Iso14971RiskValidator {
    fn validate_risk(&self, risk: &RiskItem) -> QmsResult<ValidationResult> {
        let mut result = ValidationResult::new();
        
        // Validate hazard ID
        if let Err(e) = self.validate_hazard_id(&risk.hazard_id) {
            result.add_error(e.to_string());
        }
        
        // Validate descriptions
        if let Err(e) = self.validate_descriptions(risk) {
            result.add_error(e.to_string());
        }
        
        // Validate risk parameters
        if let Err(e) = self.validate_risk_parameters(&risk.severity, &risk.occurrence, &risk.detectability) {
            result.add_error(e.to_string());
        }
        
        // Validate RPN
        if let Err(e) = self.validate_rpn(risk.risk_priority_number) {
            result.add_error(e.to_string());
        }
        
        // Add warnings for high-risk scenarios
        if risk.risk_priority_number >= iso_14971::RPN_UNACCEPTABLE_THRESHOLD {
            result.add_warning("High RPN value requires immediate risk reduction measures".to_string());
        }
        
        if risk.mitigation_measures.is_empty() && risk.risk_priority_number >= iso_14971::RPN_ALARP_THRESHOLD {
            result.add_warning("Risk in ALARP region should have mitigation measures defined".to_string());
        }
        
        Ok(result)
    }
    
    fn validate_risk_parameters(
        &self, 
        severity: &RiskSeverity, 
        occurrence: &RiskOccurrence, 
        detectability: &RiskDetectability
    ) -> QmsResult<()> {
        self.validate_risk_scale_values(severity, occurrence, detectability)
    }
    
    fn validate_risk_identification(&self, risk: &RiskItem) -> QmsResult<()> {
        self.validate_hazard_id(&risk.hazard_id)?;
        self.validate_descriptions(risk)?;
        Ok(())
    }
    
    fn validate_mitigation_measures(&self, risk: &RiskItem) -> QmsResult<()> {
        // Check if high-risk items have mitigation measures
        if risk.risk_priority_number >= iso_14971::RPN_ALARP_THRESHOLD && risk.mitigation_measures.is_empty() {
            return Err(QmsError::validation_error(
                "Risks with RPN >= 25 must have mitigation measures defined"
            ));
        }
        
        // Validate each mitigation measure
        for measure in &risk.mitigation_measures {
            if measure.description.trim().is_empty() {
                return Err(QmsError::validation_error("Mitigation measure description cannot be empty"));
            }
            
            if measure.effectiveness < 0.0 || measure.effectiveness > 1.0 {
                return Err(QmsError::validation_error("Mitigation effectiveness must be between 0.0 and 1.0"));
            }
        }
        
        Ok(())
    }
    
    fn validate_rpn(&self, rpn: u32) -> QmsResult<()> {
        if rpn == 0 {
            return Err(QmsError::validation_error("RPN cannot be zero"));
        }
        
        if rpn > iso_14971::MAX_RPN_VALUE {
            return Err(QmsError::validation_error(&format!(
                "RPN {} exceeds maximum value of {}",
                rpn, iso_14971::MAX_RPN_VALUE
            )));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::risk_manager::risk::MitigationMeasure;
    
    #[test]
    fn test_validator_creation() {
        let validator = Iso14971RiskValidator::new();
        assert!(true); // Just test creation
    }
    
    #[test]
    fn test_risk_parameter_validation() {
        let validator = Iso14971RiskValidator::new();
        
        let result = validator.validate_risk_parameters(
            &RiskSeverity::Major,
            &RiskOccurrence::Probable,
            &RiskDetectability::Moderate
        );
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_rpn_validation() {
        let validator = Iso14971RiskValidator::new();
        
        // Valid RPN
        assert!(validator.validate_rpn(50).is_ok());
        
        // Invalid RPN (zero)
        assert!(validator.validate_rpn(0).is_err());
        
        // Invalid RPN (too high)
        assert!(validator.validate_rpn(200).is_err());
    }
    
    #[test]
    fn test_hazard_id_validation() {
        let validator = Iso14971RiskValidator::new();
        
        // Valid hazard ID
        assert!(validator.validate_hazard_id("HAZ-001").is_ok());
        
        // Invalid hazard ID (empty)
        assert!(validator.validate_hazard_id("").is_err());
        
        // Invalid hazard ID (wrong format)
        assert!(validator.validate_hazard_id("RISK-001").is_err());
    }
}
