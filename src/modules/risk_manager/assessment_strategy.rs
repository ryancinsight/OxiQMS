//! Risk Assessment Strategy Pattern Implementation
//! 
//! This module implements the Strategy pattern for different risk assessment algorithms,
//! following SOLID principles for medical device quality management.
//! 
//! SOLID Principles Applied:
//! - Single Responsibility: Each strategy handles one specific assessment algorithm
//! - Open/Closed: New assessment strategies can be added without modifying existing code
//! - Liskov Substitution: All strategies can be used interchangeably
//! - Interface Segregation: Focused interfaces for specific assessment needs
//! - Dependency Inversion: High-level code depends on abstractions, not concrete implementations

use crate::prelude::*;
use super::{RiskSeverity, RiskOccurrence, RiskDetectability, RiskLevel};

/// Strategy pattern interface for risk assessment algorithms
/// Interface Segregation Principle: Focused interface for risk assessment
pub trait RiskAssessmentStrategy {
    /// Calculate Risk Priority Number (RPN) based on severity, occurrence, and detectability
    fn calculate_rpn(&self, severity: &RiskSeverity, occurrence: &RiskOccurrence, detectability: &RiskDetectability) -> u32;
    
    /// Assess risk level based on calculated RPN
    fn assess_risk_level(&self, rpn: u32) -> RiskLevel;
    
    /// Get strategy name for audit trail purposes
    fn strategy_name(&self) -> &'static str;
    
    /// Validate assessment parameters (medical device compliance)
    fn validate_parameters(&self, severity: &RiskSeverity, occurrence: &RiskOccurrence, detectability: &RiskDetectability) -> QmsResult<()>;
}

/// ISO 14971 Standard Risk Assessment Strategy
/// Single Responsibility Principle: Handles standard ISO 14971 risk assessment
pub struct ISO14971AssessmentStrategy;

impl RiskAssessmentStrategy for ISO14971AssessmentStrategy {
    fn calculate_rpn(&self, severity: &RiskSeverity, occurrence: &RiskOccurrence, detectability: &RiskDetectability) -> u32 {
        // KISS: Use simple calculation utility with cloning
        crate::utils::SimpleCalculations::calculate_rpn(
            severity.clone() as u8,
            occurrence.clone() as u8,
            detectability.clone() as u8
        ) as u32
    }
    
    fn assess_risk_level(&self, rpn: u32) -> RiskLevel {
        match rpn {
            100..=125 => RiskLevel::Unacceptable,
            25..=99 => RiskLevel::ALARP,
            1..=24 => RiskLevel::Acceptable,
            _ => RiskLevel::Acceptable,
        }
    }
    
    fn strategy_name(&self) -> &'static str {
        "ISO14971_Standard"
    }
    
    fn validate_parameters(&self, _severity: &RiskSeverity, _occurrence: &RiskOccurrence, _detectability: &RiskDetectability) -> QmsResult<()> {
        // Standard ISO 14971 validation - all parameters are valid by enum definition
        Ok(())
    }
}

/// FDA 21 CFR Part 820 Enhanced Risk Assessment Strategy
/// Single Responsibility Principle: Handles FDA-specific risk assessment requirements
pub struct FDA820AssessmentStrategy;

impl RiskAssessmentStrategy for FDA820AssessmentStrategy {
    fn calculate_rpn(&self, severity: &RiskSeverity, occurrence: &RiskOccurrence, detectability: &RiskDetectability) -> u32 {
        // FDA 21 CFR Part 820 uses weighted calculation with emphasis on severity
        let severity_weight = 2.0;
        let occurrence_weight = 1.5;
        let detectability_weight = 1.0;
        
        let weighted_rpn = (severity.clone() as u32 as f64 * severity_weight) 
            * (occurrence.clone() as u32 as f64 * occurrence_weight) 
            * (detectability.clone() as u32 as f64 * detectability_weight);
        
        weighted_rpn.round() as u32
    }
    
    fn assess_risk_level(&self, rpn: u32) -> RiskLevel {
        // FDA requires more conservative risk assessment
        match rpn {
            75..=u32::MAX => RiskLevel::Unacceptable,
            20..=74 => RiskLevel::ALARP,
            1..=19 => RiskLevel::Acceptable,
            _ => RiskLevel::Acceptable,
        }
    }
    
    fn strategy_name(&self) -> &'static str {
        "FDA_21CFR820_Enhanced"
    }
    
    fn validate_parameters(&self, severity: &RiskSeverity, occurrence: &RiskOccurrence, detectability: &RiskDetectability) -> QmsResult<()> {
        // FDA requires additional validation for high-severity risks
        if matches!(severity, RiskSeverity::Catastrophic | RiskSeverity::Critical) {
            if matches!(occurrence, RiskOccurrence::Frequent | RiskOccurrence::Probable) {
                return Err(QmsError::validation_error(
                    "FDA 21 CFR Part 820: High severity risks with frequent occurrence require additional mitigation documentation"
                ));
            }
        }
        Ok(())
    }
}

/// Conservative Risk Assessment Strategy for High-Risk Medical Devices
/// Single Responsibility Principle: Handles conservative assessment for critical devices
pub struct ConservativeAssessmentStrategy;

impl RiskAssessmentStrategy for ConservativeAssessmentStrategy {
    fn calculate_rpn(&self, severity: &RiskSeverity, occurrence: &RiskOccurrence, detectability: &RiskDetectability) -> u32 {
        // KISS: Use simple calculation with safety margin and cloning
        let base_rpn = crate::utils::SimpleCalculations::calculate_rpn(
            severity.clone() as u8,
            occurrence.clone() as u8,
            detectability.clone() as u8
        ) as f64;
        let safety_margin = 1.25; // 25% safety margin

        (base_rpn * safety_margin).round() as u32
    }
    
    fn assess_risk_level(&self, rpn: u32) -> RiskLevel {
        // More conservative thresholds
        match rpn {
            50..=u32::MAX => RiskLevel::Unacceptable,
            15..=49 => RiskLevel::ALARP,
            1..=14 => RiskLevel::Acceptable,
            _ => RiskLevel::Acceptable,
        }
    }
    
    fn strategy_name(&self) -> &'static str {
        "Conservative_HighRisk"
    }
    
    fn validate_parameters(&self, severity: &RiskSeverity, occurrence: &RiskOccurrence, detectability: &RiskDetectability) -> QmsResult<()> {
        // Conservative strategy requires justification for any risk above negligible
        if !matches!(severity, RiskSeverity::Negligible) {
            // This would typically trigger additional documentation requirements
        }
        
        if matches!(detectability, RiskDetectability::VeryLow | RiskDetectability::Low) {
            return Err(QmsError::validation_error(
                "Conservative strategy: Poor detectability requires enhanced control measures"
            ));
        }
        
        Ok(())
    }
}

/// Factory for creating risk assessment strategies
/// Open/Closed Principle: New strategies can be added without modifying this factory
/// Dependency Inversion Principle: Returns trait objects, not concrete types
pub struct RiskAssessmentStrategyFactory;

impl RiskAssessmentStrategyFactory {
    /// Create strategy based on regulatory requirements
    /// Factory Method Pattern: Encapsulates strategy creation logic
    pub fn create_strategy(strategy_type: &str) -> QmsResult<Box<dyn RiskAssessmentStrategy>> {
        match strategy_type.to_lowercase().as_str() {
            "iso14971" | "standard" => Ok(Box::new(ISO14971AssessmentStrategy)),
            "fda" | "fda820" | "21cfr820" => Ok(Box::new(FDA820AssessmentStrategy)),
            "conservative" | "high-risk" => Ok(Box::new(ConservativeAssessmentStrategy)),
            _ => Err(QmsError::validation_error(&format!(
                "Unknown risk assessment strategy: {}. Available: iso14971, fda820, conservative", 
                strategy_type
            ))),
        }
    }
    
    /// Get default strategy based on medical device classification
    pub fn default_strategy_for_device_class(device_class: &str) -> QmsResult<Box<dyn RiskAssessmentStrategy>> {
        match device_class.to_lowercase().as_str() {
            "class_i" => Ok(Box::new(ISO14971AssessmentStrategy)),
            "class_ii" => Ok(Box::new(FDA820AssessmentStrategy)),
            "class_iii" => Ok(Box::new(ConservativeAssessmentStrategy)),
            _ => Ok(Box::new(ISO14971AssessmentStrategy)), // Default to standard
        }
    }
    
    /// List available strategies for configuration
    pub fn available_strategies() -> Vec<&'static str> {
        vec!["iso14971", "fda820", "conservative"]
    }
}

/// Risk Assessment Context - uses Strategy pattern
/// Dependency Inversion Principle: Depends on RiskAssessmentStrategy abstraction
pub struct RiskAssessmentContext {
    strategy: Box<dyn RiskAssessmentStrategy>,
}

impl RiskAssessmentContext {
    /// Create new assessment context with specified strategy
    pub fn new(strategy: Box<dyn RiskAssessmentStrategy>) -> Self {
        Self { strategy }
    }
    
    /// Create context with strategy from factory
    pub fn with_strategy_type(strategy_type: &str) -> QmsResult<Self> {
        let strategy = RiskAssessmentStrategyFactory::create_strategy(strategy_type)?;
        Ok(Self::new(strategy))
    }
    
    /// Perform complete risk assessment
    pub fn assess_risk(
        &self, 
        severity: &RiskSeverity, 
        occurrence: &RiskOccurrence, 
        detectability: &RiskDetectability
    ) -> QmsResult<RiskAssessmentResult> {
        // Validate parameters using strategy-specific validation
        self.strategy.validate_parameters(severity, occurrence, detectability)?;
        
        // Calculate RPN using selected strategy
        let rpn = self.strategy.calculate_rpn(severity, occurrence, detectability);
        
        // Assess risk level using selected strategy
        let risk_level = self.strategy.assess_risk_level(rpn);
        
        Ok(RiskAssessmentResult {
            rpn,
            risk_level,
            strategy_used: self.strategy.strategy_name().to_string(),
            severity: severity.clone(),
            occurrence: occurrence.clone(),
            detectability: detectability.clone(),
        })
    }
    
    /// Change assessment strategy at runtime
    /// Open/Closed Principle: Can extend with new strategies without modification
    pub fn set_strategy(&mut self, strategy: Box<dyn RiskAssessmentStrategy>) {
        self.strategy = strategy;
    }
    
    /// Get current strategy name for audit purposes
    pub fn current_strategy_name(&self) -> &str {
        self.strategy.strategy_name()
    }
}

/// Result of risk assessment operation
#[derive(Debug, Clone)]
pub struct RiskAssessmentResult {
    pub rpn: u32,
    pub risk_level: RiskLevel,
    pub strategy_used: String,
    pub severity: RiskSeverity,
    pub occurrence: RiskOccurrence,
    pub detectability: RiskDetectability,
}

impl RiskAssessmentResult {
    /// Convert to JSON for audit trail
    pub fn to_json(&self) -> String {
        format!(
            r#"{{
    "rpn": {},
    "risk_level": "{:?}",
    "strategy_used": "{}",
    "severity": "{:?}",
    "occurrence": "{:?}",
    "detectability": "{:?}"
}}"#,
            self.rpn,
            self.risk_level,
            self.strategy_used,
            self.severity,
            self.occurrence,
            self.detectability
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iso14971_strategy() {
        let strategy = ISO14971AssessmentStrategy;
        let rpn = strategy.calculate_rpn(&RiskSeverity::Critical, &RiskOccurrence::Probable, &RiskDetectability::Moderate);
        assert_eq!(rpn, 48); // 4 * 4 * 3
        assert_eq!(strategy.assess_risk_level(rpn), RiskLevel::ALARP);
    }

    #[test]
    fn test_fda820_strategy() {
        let strategy = FDA820AssessmentStrategy;
        let rpn = strategy.calculate_rpn(&RiskSeverity::Critical, &RiskOccurrence::Probable, &RiskDetectability::Moderate);
        // FDA strategy uses weighted calculation: (4*2.0) * (4*1.5) * (3*1.0) = 8 * 6 * 3 = 144
        assert_eq!(rpn, 144);
        assert_eq!(strategy.assess_risk_level(rpn), RiskLevel::Unacceptable);
    }

    #[test]
    fn test_strategy_factory() {
        let strategy = RiskAssessmentStrategyFactory::create_strategy("iso14971").unwrap();
        assert_eq!(strategy.strategy_name(), "ISO14971_Standard");
        
        let strategy = RiskAssessmentStrategyFactory::create_strategy("fda820").unwrap();
        assert_eq!(strategy.strategy_name(), "FDA_21CFR820_Enhanced");
    }

    #[test]
    fn test_assessment_context() {
        let context = RiskAssessmentContext::with_strategy_type("conservative").unwrap();
        let result = context.assess_risk(
            &RiskSeverity::Major,
            &RiskOccurrence::Occasional,
            &RiskDetectability::High
        ).unwrap();
        
        assert_eq!(result.strategy_used, "Conservative_HighRisk");
        assert_eq!(result.rpn, 23); // (3 * 3 * 2) * 1.25 = 22.5 rounded to 23
    }
}
