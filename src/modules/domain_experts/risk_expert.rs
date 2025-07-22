/// GRASP Principles Enhancement: Information Expert Pattern
/// 
/// RiskExpert encapsulates all risk-related knowledge and calculations.
/// This follows the Information Expert principle by placing responsibility
/// with the class that has the information needed to fulfill it.

use crate::prelude::*;
use crate::modules::risk_manager::{RiskSeverity, RiskOccurrence, RiskDetectability, RiskLevel};

/// Risk domain expert that encapsulates all risk calculation logic
/// Follows GRASP Information Expert principle
pub struct RiskExpert;

impl RiskExpert {
    /// Calculate Risk Priority Number (RPN) - Information Expert
    /// This class has the knowledge of how RPN should be calculated
    pub fn calculate_rpn(
        severity: &RiskSeverity,
        occurrence: &RiskOccurrence, 
        detectability: &RiskDetectability
    ) -> u32 {
        (severity.clone() as u32) * (occurrence.clone() as u32) * (detectability.clone() as u32)
    }
    
    /// Determine risk level from RPN - Information Expert
    /// This class knows the business rules for risk level classification
    pub fn determine_risk_level(rpn: u32) -> RiskLevel {
        match rpn {
            100..=u32::MAX => RiskLevel::Unacceptable,
            50..=99 => RiskLevel::ALARP,
            1..=49 => RiskLevel::Acceptable,
            _ => RiskLevel::Acceptable,
        }
    }
    
    /// Validate risk parameters - Information Expert
    /// This class knows what constitutes valid risk parameters
    pub fn validate_risk_parameters(
        severity: &RiskSeverity,
        occurrence: &RiskOccurrence,
        detectability: &RiskDetectability
    ) -> QmsResult<()> {
        // Validate severity range
        let severity_value = severity.clone() as u8;
        if !(1..=5).contains(&severity_value) {
            return Err(QmsError::validation_error("Severity must be between 1 and 5"));
        }
        
        // Validate occurrence range
        let occurrence_value = occurrence.clone() as u8;
        if !(1..=5).contains(&occurrence_value) {
            return Err(QmsError::validation_error("Occurrence must be between 1 and 5"));
        }
        
        // Validate detectability range
        let detectability_value = detectability.clone() as u8;
        if !(1..=5).contains(&detectability_value) {
            return Err(QmsError::validation_error("Detectability must be between 1 and 5"));
        }
        
        Ok(())
    }
    
    /// Calculate risk reduction percentage - Information Expert
    /// This class knows how to calculate risk reduction metrics
    pub fn calculate_risk_reduction(initial_rpn: u32, residual_rpn: u32) -> f64 {
        if initial_rpn == 0 {
            return 0.0;
        }
        
        let reduction = initial_rpn.saturating_sub(residual_rpn) as f64;
        (reduction / initial_rpn as f64) * 100.0
    }
    
    /// Determine if risk is acceptable - Information Expert
    /// This class knows the business rules for risk acceptance
    pub fn is_risk_acceptable(risk_level: &RiskLevel) -> bool {
        matches!(risk_level, RiskLevel::Acceptable)
    }
    
    /// Get risk severity description - Information Expert
    /// This class knows how to describe risk severity levels
    pub fn get_severity_description(severity: &RiskSeverity) -> &'static str {
        match severity {
            RiskSeverity::Negligible => "No injury expected",
            RiskSeverity::Minor => "Minor injury, first aid required",
            RiskSeverity::Major => "Moderate injury requiring treatment",
            RiskSeverity::Critical => "Serious injury requiring medical intervention",
            RiskSeverity::Catastrophic => "Death or permanent disability",
        }
    }
    
    /// Get occurrence probability description - Information Expert
    /// This class knows how to describe occurrence probabilities
    pub fn get_occurrence_description(occurrence: &RiskOccurrence) -> &'static str {
        match occurrence {
            RiskOccurrence::Improbable => "So unlikely, assumed not to occur (<1 in 10,000)",
            RiskOccurrence::Remote => "Unlikely but possible (1 in 10,000 to 1 in 1,000)",
            RiskOccurrence::Occasional => "Likely to occur sometime (1 in 1,000 to 1 in 100)",
            RiskOccurrence::Probable => "Will occur several times (1 in 100 to 1 in 10)",
            RiskOccurrence::Frequent => "Very likely to occur repeatedly (>1 in 10)",
        }
    }
    
    /// Get detectability description - Information Expert
    /// This class knows how to describe detectability levels
    pub fn get_detectability_description(detectability: &RiskDetectability) -> &'static str {
        match detectability {
            RiskDetectability::VeryHigh => "Almost certain to detect",
            RiskDetectability::High => "High chance of detection",
            RiskDetectability::Moderate => "Moderate chance of detection",
            RiskDetectability::Low => "Low chance of detection",
            RiskDetectability::VeryLow => "Very unlikely to detect",
        }
    }
    
    /// Calculate risk matrix position - Information Expert
    /// This class knows how to position risks in a risk matrix
    pub fn calculate_matrix_position(severity: &RiskSeverity, occurrence: &RiskOccurrence) -> (u8, u8) {
        (severity.clone() as u8, occurrence.clone() as u8)
    }
    
    /// Recommend mitigation priority - Information Expert
    /// This class knows how to prioritize risk mitigation
    pub fn recommend_mitigation_priority(rpn: u32) -> MitigationPriority {
        match rpn {
            100..=u32::MAX => MitigationPriority::Immediate,
            50..=99 => MitigationPriority::High,
            25..=49 => MitigationPriority::Medium,
            10..=24 => MitigationPriority::Low,
            _ => MitigationPriority::Monitor,
        }
    }
}

/// Mitigation priority levels
#[derive(Debug, Clone, PartialEq)]
pub enum MitigationPriority {
    Immediate,
    High,
    Medium,
    Low,
    Monitor,
}

impl MitigationPriority {
    pub fn description(&self) -> &'static str {
        match self {
            MitigationPriority::Immediate => "Immediate action required - stop production if necessary",
            MitigationPriority::High => "High priority - address within 30 days",
            MitigationPriority::Medium => "Medium priority - address within 90 days",
            MitigationPriority::Low => "Low priority - address within 180 days",
            MitigationPriority::Monitor => "Monitor and review periodically",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rpn_calculation() {
        let rpn = RiskExpert::calculate_rpn(
            &RiskSeverity::Critical,
            &RiskOccurrence::Probable,
            &RiskDetectability::Low
        );
        assert_eq!(rpn, 4 * 4 * 4); // 64
    }
    
    #[test]
    fn test_risk_level_determination() {
        assert_eq!(RiskExpert::determine_risk_level(125), RiskLevel::Unacceptable);
        assert_eq!(RiskExpert::determine_risk_level(75), RiskLevel::ALARP);
        assert_eq!(RiskExpert::determine_risk_level(25), RiskLevel::Acceptable);
    }
    
    #[test]
    fn test_risk_reduction_calculation() {
        let reduction = RiskExpert::calculate_risk_reduction(100, 25);
        assert_eq!(reduction, 75.0);
    }
    
    #[test]
    fn test_mitigation_priority() {
        assert_eq!(RiskExpert::recommend_mitigation_priority(125), MitigationPriority::Immediate);
        assert_eq!(RiskExpert::recommend_mitigation_priority(75), MitigationPriority::High);
        assert_eq!(RiskExpert::recommend_mitigation_priority(35), MitigationPriority::Medium);
    }
}
