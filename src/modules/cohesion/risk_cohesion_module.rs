/// GRASP Principles Enhancement: High Cohesion and Low Coupling
/// 
/// RiskCohesionModule demonstrates high cohesion by grouping all related
/// risk functionality together, and low coupling by minimizing dependencies
/// on other modules through well-defined interfaces.

use crate::prelude::*;
use crate::modules::domain_experts::RiskExpert;
use crate::modules::risk_manager::{RiskSeverity, RiskOccurrence, RiskDetectability, RiskLevel};
use std::collections::HashMap;

/// High cohesion module for risk-related operations
/// All risk functionality is grouped together with minimal external dependencies
pub struct RiskCohesionModule {
    // Internal state - high cohesion
    risk_cache: HashMap<String, RiskItem>,
    calculation_cache: HashMap<(u8, u8, u8), u32>,
    validation_rules: RiskValidationRules,
}

/// Risk validation rules - cohesive grouping of related validation logic
#[derive(Debug, Clone)]
pub struct RiskValidationRules {
    pub min_severity: u8,
    pub max_severity: u8,
    pub min_occurrence: u8,
    pub max_occurrence: u8,
    pub min_detectability: u8,
    pub max_detectability: u8,
    pub max_acceptable_rpn: u32,
}

impl Default for RiskValidationRules {
    fn default() -> Self {
        Self {
            min_severity: 1,
            max_severity: 5,
            min_occurrence: 1,
            max_occurrence: 5,
            min_detectability: 1,
            max_detectability: 5,
            max_acceptable_rpn: 49, // ISO 14971 typical threshold
        }
    }
}

impl RiskCohesionModule {
    /// Create new risk cohesion module
    pub fn new() -> Self {
        Self {
            risk_cache: HashMap::new(),
            calculation_cache: HashMap::new(),
            validation_rules: RiskValidationRules::default(),
        }
    }
    
    /// Create with custom validation rules
    pub fn with_validation_rules(rules: RiskValidationRules) -> Self {
        Self {
            risk_cache: HashMap::new(),
            calculation_cache: HashMap::new(),
            validation_rules: rules,
        }
    }
    
    /// Perform complete risk analysis - High Cohesion
    /// All risk analysis steps are grouped together in one cohesive operation
    pub fn perform_risk_analysis(
        &mut self,
        risk_id: &str,
        severity: &RiskSeverity,
        occurrence: &RiskOccurrence,
        detectability: &RiskDetectability,
    ) -> QmsResult<RiskAnalysisResult> {
        // Step 1: Validate parameters (cohesive validation)
        self.validate_risk_parameters(severity, occurrence, detectability)?;
        
        // Step 2: Calculate RPN (cohesive calculation)
        let rpn = self.calculate_rpn_cached(severity, occurrence, detectability);
        
        // Step 3: Determine risk level (cohesive assessment)
        let risk_level = self.determine_risk_level(rpn);
        
        // Step 4: Generate recommendations (cohesive analysis)
        let recommendations = self.generate_risk_recommendations(rpn, &risk_level);
        
        // Step 5: Create analysis result (cohesive result)
        let result = RiskAnalysisResult {
            risk_id: risk_id.to_string(),
            rpn,
            risk_level,
            severity: severity.clone(),
            occurrence: occurrence.clone(),
            detectability: detectability.clone(),
            is_acceptable: self.is_risk_acceptable(rpn),
            recommendations,
            analysis_timestamp: crate::utils::current_timestamp_string(),
        };
        
        // Cache the result for future reference
        self.cache_analysis_result(risk_id, &result);
        
        Ok(result)
    }
    
    /// Validate risk parameters - High Cohesion
    /// All validation logic grouped together
    fn validate_risk_parameters(
        &self,
        severity: &RiskSeverity,
        occurrence: &RiskOccurrence,
        detectability: &RiskDetectability,
    ) -> QmsResult<()> {
        let severity_val = severity.clone() as u8;
        let occurrence_val = occurrence.clone() as u8;
        let detectability_val = detectability.clone() as u8;
        
        // Validate severity
        if severity_val < self.validation_rules.min_severity || severity_val > self.validation_rules.max_severity {
            return Err(QmsError::validation_error(&format!(
                "Severity must be between {} and {}",
                self.validation_rules.min_severity,
                self.validation_rules.max_severity
            )));
        }
        
        // Validate occurrence
        if occurrence_val < self.validation_rules.min_occurrence || occurrence_val > self.validation_rules.max_occurrence {
            return Err(QmsError::validation_error(&format!(
                "Occurrence must be between {} and {}",
                self.validation_rules.min_occurrence,
                self.validation_rules.max_occurrence
            )));
        }
        
        // Validate detectability
        if detectability_val < self.validation_rules.min_detectability || detectability_val > self.validation_rules.max_detectability {
            return Err(QmsError::validation_error(&format!(
                "Detectability must be between {} and {}",
                self.validation_rules.min_detectability,
                self.validation_rules.max_detectability
            )));
        }
        
        Ok(())
    }
    
    /// Calculate RPN with caching - High Cohesion, Low Coupling
    /// Uses internal cache to reduce coupling with external calculation services
    fn calculate_rpn_cached(
        &mut self,
        severity: &RiskSeverity,
        occurrence: &RiskOccurrence,
        detectability: &RiskDetectability,
    ) -> u32 {
        let key = (severity.clone() as u8, occurrence.clone() as u8, detectability.clone() as u8);
        
        if let Some(&cached_rpn) = self.calculation_cache.get(&key) {
            return cached_rpn;
        }
        
        // Use domain expert for calculation (low coupling through interface)
        let rpn = RiskExpert::calculate_rpn(severity, occurrence, detectability);
        
        // Cache the result
        self.calculation_cache.insert(key, rpn);
        
        rpn
    }
    
    /// Determine risk level - High Cohesion
    /// Risk level determination logic grouped with other risk logic
    fn determine_risk_level(&self, rpn: u32) -> RiskLevel {
        RiskExpert::determine_risk_level(rpn)
    }
    
    /// Check if risk is acceptable - High Cohesion
    /// Acceptance logic grouped with other risk assessment logic
    fn is_risk_acceptable(&self, rpn: u32) -> bool {
        rpn <= self.validation_rules.max_acceptable_rpn
    }
    
    /// Generate risk recommendations - High Cohesion
    /// All recommendation logic grouped together
    fn generate_risk_recommendations(&self, rpn: u32, risk_level: &RiskLevel) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        match risk_level {
            RiskLevel::Unacceptable => {
                recommendations.push("IMMEDIATE ACTION REQUIRED: Risk is unacceptable".to_string());
                recommendations.push("Implement risk controls before proceeding".to_string());
                recommendations.push("Consider design changes to reduce risk".to_string());
            }
            RiskLevel::ALARP => {
                recommendations.push("Risk is ALARP (As Low As Reasonably Practicable)".to_string());
                recommendations.push("Evaluate additional risk controls".to_string());
                recommendations.push("Document risk acceptance rationale".to_string());
            }
            RiskLevel::Acceptable => {
                recommendations.push("Risk is acceptable".to_string());
                recommendations.push("Monitor risk during product lifecycle".to_string());
            }
        }
        
        // Add RPN-specific recommendations
        if rpn > 100 {
            recommendations.push("Consider multiple risk control measures".to_string());
        } else if rpn > 50 {
            recommendations.push("Implement at least one risk control measure".to_string());
        }
        
        recommendations
    }
    
    /// Cache analysis result - High Cohesion
    /// Caching logic grouped with other internal operations
    fn cache_analysis_result(&mut self, risk_id: &str, result: &RiskAnalysisResult) {
        // In a real implementation, this might cache to a more sophisticated storage
        // For now, we just track that analysis was performed
        self.calculation_cache.insert(
            (result.severity.clone() as u8, result.occurrence.clone() as u8, result.detectability.clone() as u8),
            result.rpn
        );
    }
    
    /// Get cached analysis - Low Coupling
    /// Provides interface to cached data without exposing internal structure
    pub fn get_cached_rpn(&self, severity: &RiskSeverity, occurrence: &RiskOccurrence, detectability: &RiskDetectability) -> Option<u32> {
        let key = (severity.clone() as u8, occurrence.clone() as u8, detectability.clone() as u8);
        self.calculation_cache.get(&key).copied()
    }
    
    /// Clear cache - High Cohesion
    /// Cache management grouped with other internal operations
    pub fn clear_cache(&mut self) {
        self.calculation_cache.clear();
        self.risk_cache.clear();
    }
    
    /// Get validation rules - Low Coupling
    /// Provides read-only access to internal rules without exposing structure
    pub fn get_validation_rules(&self) -> &RiskValidationRules {
        &self.validation_rules
    }
    
    /// Update validation rules - Low Coupling
    /// Controlled interface for updating internal rules
    pub fn update_validation_rules(&mut self, rules: RiskValidationRules) {
        self.validation_rules = rules;
        // Clear cache since rules changed
        self.clear_cache();
    }
}

/// Risk analysis result - High Cohesion
/// All related analysis data grouped together
#[derive(Debug, Clone)]
pub struct RiskAnalysisResult {
    pub risk_id: String,
    pub rpn: u32,
    pub risk_level: RiskLevel,
    pub severity: RiskSeverity,
    pub occurrence: RiskOccurrence,
    pub detectability: RiskDetectability,
    pub is_acceptable: bool,
    pub recommendations: Vec<String>,
    pub analysis_timestamp: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_risk_analysis_high_cohesion() {
        let mut module = RiskCohesionModule::new();
        
        let result = module.perform_risk_analysis(
            "TEST-001",
            &RiskSeverity::Critical,
            &RiskOccurrence::Probable,
            &RiskDetectability::Low,
        ).unwrap();
        
        assert_eq!(result.risk_id, "TEST-001");
        assert_eq!(result.rpn, 64); // 4 * 4 * 4
        assert!(!result.is_acceptable); // RPN > 49
        assert!(!result.recommendations.is_empty());
    }
    
    #[test]
    fn test_caching_low_coupling() {
        let mut module = RiskCohesionModule::new();
        
        // First calculation
        let rpn1 = module.calculate_rpn_cached(
            &RiskSeverity::Minor,
            &RiskOccurrence::Remote,
            &RiskDetectability::High,
        );
        
        // Second calculation should use cache
        let rpn2 = module.calculate_rpn_cached(
            &RiskSeverity::Minor,
            &RiskOccurrence::Remote,
            &RiskDetectability::High,
        );
        
        assert_eq!(rpn1, rpn2);
        
        // Verify cache is working
        let cached = module.get_cached_rpn(
            &RiskSeverity::Minor,
            &RiskOccurrence::Remote,
            &RiskDetectability::High,
        );
        assert_eq!(cached, Some(rpn1));
    }
    
    #[test]
    fn test_validation_rules_cohesion() {
        let custom_rules = RiskValidationRules {
            min_severity: 1,
            max_severity: 3,
            min_occurrence: 1,
            max_occurrence: 3,
            min_detectability: 1,
            max_detectability: 3,
            max_acceptable_rpn: 20,
        };
        
        let mut module = RiskCohesionModule::with_validation_rules(custom_rules);
        
        // This should fail with custom rules
        let result = module.perform_risk_analysis(
            "TEST-002",
            &RiskSeverity::Catastrophic, // Value 5, but max is 3
            &RiskOccurrence::Remote,
            &RiskDetectability::High,
        );
        
        assert!(result.is_err());
    }
}
