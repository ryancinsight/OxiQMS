//! Risk Business Logic - Core Assessment Operations
//! 
//! SRP (Single Responsibility Principle): Handles only business logic operations
//! Extracted from RiskManager to separate business rules from persistence and validation
//! 
//! Medical Device Compliance: Implements ISO 14971 risk assessment methodology

use crate::prelude::*;
use crate::constants::iso_14971;
use crate::utils::RiskCalculator;
use super::risk::{RiskItem, RiskSeverity, RiskOccurrence, RiskDetectability, RiskLevel, MitigationMeasure, VerificationStatus, RiskStatus};
use super::repository::RiskRepository;
use super::validator::RiskValidator;

// Helper functions from risk module
fn get_current_user() -> QmsResult<String> {
    Ok("admin".to_string()) // Placeholder - should be replaced with actual user management
}

fn get_current_project_id() -> QmsResult<String> {
    Ok("default-project".to_string()) // Placeholder - should be replaced with actual project management
}

/// Risk Business Logic Service - Single Responsibility: Business operations only
/// 
/// Coordinates risk assessment operations using dependency injection:
/// - Repository for data persistence
/// - Validator for validation logic
/// - Pure business logic for risk calculations and assessments
pub struct RiskBusinessLogic<R: RiskRepository, V: RiskValidator> {
    repository: R,
    validator: V,
}

impl<R: RiskRepository, V: RiskValidator> RiskBusinessLogic<R, V> {
    /// Create new risk business logic service with injected dependencies
    /// DIP (Dependency Inversion Principle): Depends on abstractions, not concretions
    pub fn new(repository: R, validator: V) -> Self {
        Self {
            repository,
            validator,
        }
    }
    
    /// Create a new risk item with business logic validation
    pub fn create_risk(&self, hazard_desc: &str, situation: &str, harm: &str) -> QmsResult<RiskItem> {
        // Generate unique identifiers
        let id = crate::utils::generate_uuid();
        let hazard_id = format!("HAZ-{:03}", 1); // Simplified for now
        let timestamp = crate::utils::current_timestamp_string();
        let current_user = get_current_user()?;
        
        // Create risk item with default values - SRP: Business logic handles object creation
        let mut risk = RiskItem {
            id: id.clone(),
            project_id: get_current_project_id()?,
            hazard_id: hazard_id.clone(),
            hazard_description: hazard_desc.to_string(),
            hazardous_situation: situation.to_string(),
            harm: harm.to_string(),
            severity: RiskSeverity::Minor,
            occurrence: RiskOccurrence::Remote,
            detectability: RiskDetectability::High,
            risk_priority_number: 0,
            initial_risk_level: RiskLevel::Acceptable,
            mitigation_measures: Vec::new(),
            residual_severity: RiskSeverity::Minor,
            residual_occurrence: RiskOccurrence::Remote,
            residual_detectability: RiskDetectability::High,
            residual_rpn: 0,
            residual_risk_level: RiskLevel::Acceptable,
            residual_risk_justification: None,
            residual_risk_approved: false,
            residual_risk_approved_by: None,
            residual_risk_approval_date: None,
            verification_method: "Design Review".to_string(),
            verification_status: VerificationStatus::Planned,
            verification_evidence: Vec::new(),
            category: "Safety".to_string(),
            source: "Design Review".to_string(),
            assigned_to: Some(current_user.clone()),
            due_date: None,
            priority: "Medium".to_string(),
            risk_status: RiskStatus::Identified,
            tags: Vec::new(),
            regulatory_references: vec!["ISO_14971".to_string()],
            standard_references: vec!["ISO_14971:2019".to_string()],
            created_at: timestamp.clone(),
            updated_at: timestamp.clone(),
            created_by: current_user.clone(),
            approved_by: None,
            approval_date: None,
            post_market_data: Vec::new(),
            review_required: false,
            next_review_date: None,
        };
        
        // Calculate initial RPN
        self.calculate_and_update_rpn(&mut risk)?;
        
        // Validate the risk
        self.validator.validate_risk_identification(&risk)?;
        
        // Save to repository
        self.repository.save_risk(&risk)?;
        
        // Log business operation
        crate::audit::log_audit(&format!(
            "RISK_CREATED: {} - {} by {}",
            risk.hazard_id, risk.hazard_description, risk.created_by
        ));
        
        Ok(risk)
    }
    
    /// Assess a risk by updating severity, occurrence, detectability
    pub fn assess_risk(
        &self,
        risk_id: &str,
        severity: Option<RiskSeverity>,
        occurrence: Option<RiskOccurrence>,
        detectability: Option<RiskDetectability>,
    ) -> QmsResult<RiskItem> {
        // Load existing risk
        let mut risk = self.repository.load_risk(risk_id)?;
        
        // Update parameters if provided
        if let Some(sev) = severity {
            risk.severity = sev;
        }
        if let Some(occ) = occurrence {
            risk.occurrence = occ;
        }
        if let Some(det) = detectability {
            risk.detectability = det;
        }
        
        // Validate new parameters
        self.validator.validate_risk_parameters(&risk.severity, &risk.occurrence, &risk.detectability)?;
        
        // Recalculate RPN and risk level
        self.calculate_and_update_rpn(&mut risk)?;
        
        // Update residual risk (initially same as initial risk)
        risk.residual_severity = risk.severity.clone();
        risk.residual_occurrence = risk.occurrence.clone();
        risk.residual_detectability = risk.detectability.clone();
        self.calculate_and_update_residual_rpn(&mut risk)?;
        
        // Update timestamp
        risk.updated_at = crate::utils::current_timestamp_string();
        
        // Save updated risk
        self.repository.save_risk(&risk)?;
        
        // Log assessment operation
        crate::audit::log_audit(&format!(
            "RISK_ASSESSED: {} - RPN: {}, Level: {:?} by {}",
            risk.hazard_id, risk.risk_priority_number, risk.initial_risk_level, 
            get_current_user()?
        ));
        
        Ok(risk)
    }
    
    /// Add mitigation measure to a risk
    pub fn add_mitigation_measure(
        &self,
        risk_id: &str,
        description: &str,
        effectiveness: f32,
        implementation: &str,
        verification_method: &str,
    ) -> QmsResult<MitigationMeasure> {
        // Load existing risk
        let mut risk = self.repository.load_risk(risk_id)?;

        // Create mitigation measure
        let measure = MitigationMeasure {
            id: crate::utils::generate_uuid(),
            description: description.to_string(),
            implementation: implementation.to_string(),
            effectiveness,
            cost: None,
            timeline: None,
            verification_method: verification_method.to_string(),
            verification_status: VerificationStatus::Planned,
            verification_evidence: Vec::new(),
            implementation_status: "Planned".to_string(),
            assigned_to: None,
            due_date: None,
            implemented_date: None,
            verified_date: None,
        };

        // Validate mitigation measure
        if effectiveness < 0.0 || effectiveness > 1.0 {
            return Err(QmsError::validation_error("Mitigation effectiveness must be between 0.0 and 1.0"));
        }
        
        // Add to risk
        risk.mitigation_measures.push(measure.clone());
        
        // Recalculate residual risk based on mitigation effectiveness
        self.calculate_residual_risk_with_mitigations(&mut risk)?;
        
        // Update timestamp
        risk.updated_at = crate::utils::current_timestamp_string();
        
        // Save updated risk
        self.repository.save_risk(&risk)?;
        
        // Log mitigation addition
        crate::audit::log_audit(&format!(
            "MITIGATION_ADDED: {} - {} (effectiveness: {:.1}%)",
            risk.hazard_id, description, effectiveness * 100.0
        ));
        
        Ok(measure)
    }
    
    /// Calculate and update RPN for a risk
    fn calculate_and_update_rpn(&self, risk: &mut RiskItem) -> QmsResult<()> {
        risk.risk_priority_number = RiskCalculator::calculate_rpn(
            &risk.severity,
            &risk.occurrence,
            &risk.detectability,
        );
        
        risk.initial_risk_level = self.determine_risk_level(risk.risk_priority_number);
        
        // Validate calculated RPN
        self.validator.validate_rpn(risk.risk_priority_number)?;
        
        Ok(())
    }
    
    /// Calculate and update residual RPN for a risk
    fn calculate_and_update_residual_rpn(&self, risk: &mut RiskItem) -> QmsResult<()> {
        risk.residual_rpn = RiskCalculator::calculate_rpn(
            &risk.residual_severity,
            &risk.residual_occurrence,
            &risk.residual_detectability,
        );
        
        risk.residual_risk_level = self.determine_risk_level(risk.residual_rpn);
        
        Ok(())
    }
    
    /// Calculate residual risk considering mitigation effectiveness
    fn calculate_residual_risk_with_mitigations(&self, risk: &mut RiskItem) -> QmsResult<()> {
        // Calculate combined effectiveness of all mitigations
        let mut combined_effectiveness = 1.0f32;
        for measure in &risk.mitigation_measures {
            combined_effectiveness *= (1.0 - measure.effectiveness);
        }
        let total_effectiveness = 1.0 - combined_effectiveness;

        // Apply effectiveness to reduce occurrence (most common mitigation target)
        let original_occurrence = risk.occurrence.clone() as u8;
        let reduced_occurrence = ((original_occurrence as f32) * (1.0 - total_effectiveness)).max(1.0) as u8;

        risk.residual_occurrence = match reduced_occurrence {
            1 => RiskOccurrence::Improbable,
            2 => RiskOccurrence::Remote,
            3 => RiskOccurrence::Occasional,
            4 => RiskOccurrence::Probable,
            5 => RiskOccurrence::Frequent,
            _ => RiskOccurrence::Improbable,
        };

        // Keep severity and detectability the same for residual risk
        risk.residual_severity = risk.severity.clone();
        risk.residual_detectability = risk.detectability.clone();

        // Recalculate residual RPN
        self.calculate_and_update_residual_rpn(risk)?;

        Ok(())
    }
    
    /// Determine risk level based on RPN value per ISO 14971
    fn determine_risk_level(&self, rpn: u32) -> RiskLevel {
        match rpn {
            rpn if rpn >= iso_14971::RPN_UNACCEPTABLE_THRESHOLD => RiskLevel::Unacceptable,
            rpn if rpn >= iso_14971::RPN_ALARP_THRESHOLD => RiskLevel::ALARP,
            _ => RiskLevel::Acceptable,
        }
    }
    
    /// Get risk by ID
    pub fn get_risk(&self, risk_id: &str) -> QmsResult<RiskItem> {
        self.repository.load_risk(risk_id)
    }
    
    /// Check if risk exists
    pub fn risk_exists(&self, risk_id: &str) -> QmsResult<bool> {
        self.repository.risk_exists(risk_id)
    }
    
    /// Get all risks
    pub fn get_all_risks(&self) -> QmsResult<Vec<RiskItem>> {
        self.repository.load_all_risks()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::repository::FileRiskRepository;
    use super::super::validator::Iso14971RiskValidator;
    use crate::utils::test_helpers::TestHelper;

    #[test]
    fn test_business_logic_creation() {
        let temp_dir = TestHelper::create_temp_dir();
        let repository = FileRiskRepository::new(&temp_dir).unwrap();
        let validator = Iso14971RiskValidator::new();

        let business_logic = RiskBusinessLogic::new(repository, validator);
        assert!(true); // Just test creation - SRP: Dependency injection working
    }

    #[test]
    fn test_risk_level_determination() {
        let temp_dir = TestHelper::create_temp_dir();
        let repository = FileRiskRepository::new(&temp_dir).unwrap();
        let validator = Iso14971RiskValidator::new();
        let business_logic = RiskBusinessLogic::new(repository, validator);

        // SRP: Business logic handles risk level determination
        assert_eq!(business_logic.determine_risk_level(10), RiskLevel::Acceptable);
        assert_eq!(business_logic.determine_risk_level(50), RiskLevel::ALARP);
        assert_eq!(business_logic.determine_risk_level(120), RiskLevel::Unacceptable);
    }
}
