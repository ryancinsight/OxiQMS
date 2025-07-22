//! Centralized Risk Priority Number (RPN) Calculator
//! 
//! REFACTORED: Single source of truth for RPN calculations to eliminate DRY violations
//! Previously duplicated across RiskScoring, FMEAManager, and RiskItem modules
//! 
//! Medical Device Compliance: ISO 14971 Risk Management for Medical Devices
//! Ensures consistent risk calculation methodology across the entire QMS system

use crate::modules::risk_manager::{RiskSeverity, RiskOccurrence, RiskDetectability};

/// Risk levels based on RPN values for medical device compliance
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskLevel {
    /// RPN 100-125: Unacceptable risk requiring immediate action
    Unacceptable,
    /// RPN 25-99: ALARP (As Low As Reasonably Practicable) - requires risk reduction
    ALARP,
    /// RPN 1-24: Acceptable risk with monitoring
    Acceptable,
}

/// Centralized risk calculator following DRY principle
/// Single source of truth for all RPN calculations in the QMS system
pub struct RiskCalculator;

impl RiskCalculator {
    /// Calculate Risk Priority Number (RPN) using the standard formula
    /// RPN = Severity × Occurrence × Detectability
    /// 
    /// This is the single source of truth for RPN calculations across the entire system
    /// 
    /// # Arguments
    /// * `severity` - Risk severity level (1-5 scale)
    /// * `occurrence` - Risk occurrence probability (1-5 scale)
    /// * `detectability` - Risk detectability level (1-5 scale)
    /// 
    /// # Returns
    /// * `u32` - Risk Priority Number (1-125 range)
    /// 
    /// # Medical Device Compliance
    /// Follows ISO 14971 standard for medical device risk management
    pub fn calculate_rpn(
        severity: &RiskSeverity,
        occurrence: &RiskOccurrence,
        detectability: &RiskDetectability,
    ) -> u32 {
        (severity.clone() as u32) * (occurrence.clone() as u32) * (detectability.clone() as u32)
    }

    /// Determine risk level based on RPN value
    /// 
    /// Risk level thresholds based on medical device industry standards:
    /// - Unacceptable (100-125): Immediate action required
    /// - ALARP (25-99): Risk reduction measures needed
    /// - Acceptable (1-24): Monitor and maintain controls
    /// 
    /// # Arguments
    /// * `rpn` - Risk Priority Number
    /// 
    /// # Returns
    /// * `RiskLevel` - Categorized risk level
    pub fn calculate_risk_level(rpn: u32) -> RiskLevel {
        match rpn {
            100..=125 => RiskLevel::Unacceptable,
            25..=99 => RiskLevel::ALARP,
            1..=24 => RiskLevel::Acceptable,
            _ => RiskLevel::Acceptable, // Default to acceptable for edge cases
        }
    }

    /// Calculate RPN and determine risk level in one operation
    /// 
    /// # Arguments
    /// * `severity` - Risk severity level
    /// * `occurrence` - Risk occurrence probability
    /// * `detectability` - Risk detectability level
    /// 
    /// # Returns
    /// * `(u32, RiskLevel)` - Tuple of RPN value and risk level
    pub fn calculate_rpn_with_level(
        severity: &RiskSeverity,
        occurrence: &RiskOccurrence,
        detectability: &RiskDetectability,
    ) -> (u32, RiskLevel) {
        let rpn = Self::calculate_rpn(severity, occurrence, detectability);
        let level = Self::calculate_risk_level(rpn);
        (rpn, level)
    }

    /// Validate RPN calculation inputs
    /// 
    /// Ensures all risk factors are within valid ranges (1-5)
    /// 
    /// # Arguments
    /// * `severity` - Risk severity level
    /// * `occurrence` - Risk occurrence probability
    /// * `detectability` - Risk detectability level
    /// 
    /// # Returns
    /// * `bool` - True if all inputs are valid
    pub fn validate_inputs(
        severity: &RiskSeverity,
        occurrence: &RiskOccurrence,
        detectability: &RiskDetectability,
    ) -> bool {
        let severity_val = severity.clone() as u32;
        let occurrence_val = occurrence.clone() as u32;
        let detectability_val = detectability.clone() as u32;

        (1..=5).contains(&severity_val)
            && (1..=5).contains(&occurrence_val)
            && (1..=5).contains(&detectability_val)
    }

    /// Get risk level description for reporting
    /// 
    /// # Arguments
    /// * `level` - Risk level
    /// 
    /// # Returns
    /// * `&'static str` - Human-readable description
    pub fn get_risk_level_description(level: &RiskLevel) -> &'static str {
        match level {
            RiskLevel::Unacceptable => "Unacceptable - Immediate action required",
            RiskLevel::ALARP => "ALARP - Risk reduction measures needed",
            RiskLevel::Acceptable => "Acceptable - Monitor and maintain controls",
        }
    }

    /// Get recommended actions based on risk level
    /// 
    /// # Arguments
    /// * `level` - Risk level
    /// 
    /// # Returns
    /// * `Vec<&'static str>` - List of recommended actions
    pub fn get_recommended_actions(level: &RiskLevel) -> Vec<&'static str> {
        match level {
            RiskLevel::Unacceptable => vec![
                "Stop activity immediately",
                "Implement immediate risk controls",
                "Escalate to management",
                "Document corrective actions",
                "Re-evaluate after controls implemented",
            ],
            RiskLevel::ALARP => vec![
                "Implement additional risk controls",
                "Consider design changes",
                "Enhance monitoring procedures",
                "Document risk reduction measures",
                "Schedule periodic review",
            ],
            RiskLevel::Acceptable => vec![
                "Monitor existing controls",
                "Maintain current procedures",
                "Periodic risk assessment review",
                "Document risk acceptance rationale",
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_rpn() {
        // Test standard RPN calculation
        let rpn = RiskCalculator::calculate_rpn(
            &RiskSeverity::Major,      // 3
            &RiskOccurrence::Probable, // 4
            &RiskDetectability::Low,   // 4
        );
        assert_eq!(rpn, 48); // 3 * 4 * 4 = 48
    }

    #[test]
    fn test_calculate_risk_level() {
        assert_eq!(RiskCalculator::calculate_risk_level(125), RiskLevel::Unacceptable);
        assert_eq!(RiskCalculator::calculate_risk_level(100), RiskLevel::Unacceptable);
        assert_eq!(RiskCalculator::calculate_risk_level(99), RiskLevel::ALARP);
        assert_eq!(RiskCalculator::calculate_risk_level(25), RiskLevel::ALARP);
        assert_eq!(RiskCalculator::calculate_risk_level(24), RiskLevel::Acceptable);
        assert_eq!(RiskCalculator::calculate_risk_level(1), RiskLevel::Acceptable);
    }

    #[test]
    fn test_calculate_rpn_with_level() {
        let (rpn, level) = RiskCalculator::calculate_rpn_with_level(
            &RiskSeverity::Catastrophic, // 5
            &RiskOccurrence::Frequent,   // 5
            &RiskDetectability::VeryLow, // 5
        );
        assert_eq!(rpn, 125); // 5 * 5 * 5 = 125
        assert_eq!(level, RiskLevel::Unacceptable);
    }

    #[test]
    fn test_validate_inputs() {
        assert!(RiskCalculator::validate_inputs(
            &RiskSeverity::Minor,
            &RiskOccurrence::Remote,
            &RiskDetectability::High
        ));
    }

    #[test]
    fn test_risk_level_descriptions() {
        assert_eq!(
            RiskCalculator::get_risk_level_description(&RiskLevel::Unacceptable),
            "Unacceptable - Immediate action required"
        );
        assert_eq!(
            RiskCalculator::get_risk_level_description(&RiskLevel::ALARP),
            "ALARP - Risk reduction measures needed"
        );
        assert_eq!(
            RiskCalculator::get_risk_level_description(&RiskLevel::Acceptable),
            "Acceptable - Monitor and maintain controls"
        );
    }

    #[test]
    fn test_recommended_actions() {
        let unacceptable_actions = RiskCalculator::get_recommended_actions(&RiskLevel::Unacceptable);
        assert!(unacceptable_actions.contains(&"Stop activity immediately"));
        
        let alarp_actions = RiskCalculator::get_recommended_actions(&RiskLevel::ALARP);
        assert!(alarp_actions.contains(&"Implement additional risk controls"));
        
        let acceptable_actions = RiskCalculator::get_recommended_actions(&RiskLevel::Acceptable);
        assert!(acceptable_actions.contains(&"Monitor existing controls"));
    }
}
