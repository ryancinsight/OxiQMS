//! Risk Scoring and Matrix Implementation
//! Task 3.1.4: Risk Assessment Calculations
//! 
//! This module implements the comprehensive risk assessment calculations and
//! risk matrix visualization per ISO 14971 requirements for medical devices.

#![allow(dead_code)] // Allow dead code during development - functions will be used in future tasks

use crate::modules::risk_manager::{RiskSeverity, RiskOccurrence, RiskDetectability, RiskLevel};
use crate::utils::RiskCalculator; // REFACTORED: Use centralized risk calculator
use std::fmt::{self, Display};

/// Risk Matrix Position for 5x5 grid visualization
#[derive(Debug, Clone, PartialEq)]
pub struct RiskMatrixPosition {
    pub severity: RiskSeverity,
    pub occurrence: RiskOccurrence,
    pub risk_level: RiskLevel,
    pub rpn: u32,
    pub color_code: MatrixColor,
}

/// Color coding for risk matrix cells
#[derive(Debug, Clone, PartialEq)]
pub enum MatrixColor {
    Green,      // Acceptable risk
    Yellow,     // ALARP (As Low As Reasonably Practicable)
    Red,        // Unacceptable risk
}

/// Risk Matrix Generator per ISO 14971
pub struct RiskMatrix {
    matrix: Vec<Vec<RiskMatrixPosition>>,
    title: String,
    version: String,
}

impl Default for RiskMatrix {
    fn default() -> Self {
        Self::new()
    }
}

impl RiskMatrix {
    /// Create a new 5x5 risk matrix per ISO 14971 guidelines
    pub fn new() -> Self {
        let mut matrix = Vec::new();
        
        // Build 5x5 matrix (Severity × Occurrence)
        for severity_val in 1..=5 {
            let mut row = Vec::new();
            let severity = Self::value_to_severity(severity_val);
            
            for occurrence_val in 1..=5 {
                let occurrence = Self::value_to_occurrence(occurrence_val);
                // Use moderate detectability for matrix display (most common scenario)
                let detectability = RiskDetectability::Moderate;
                
                // REFACTORED: Use centralized RiskCalculator instead of inline calculation
                let rpn = RiskCalculator::calculate_rpn(&severity, &occurrence, &detectability);
                let risk_level = Self::calculate_risk_level(rpn);
                let color_code = Self::risk_level_to_color(&risk_level);
                
                let position = RiskMatrixPosition {
                    severity: severity.clone(),
                    occurrence: occurrence.clone(),
                    risk_level,
                    rpn,
                    color_code,
                };
                
                row.push(position);
            }
            matrix.push(row);
        }
        
        Self {
            matrix,
            title: "ISO 14971 Risk Assessment Matrix".to_string(),
            version: "1.0".to_string(),
        }
    }
    
    /// Generate ASCII representation of the risk matrix
    pub fn generate_ascii_matrix(&self) -> String {
        let mut output = String::new();
        
        // Header
        output.push_str(&format!("┌─ {} (v{}) ─┐\n", self.title, self.version));
        output.push_str("│ Medical Device Risk Matrix (Severity × Occurrence)\n");
        output.push_str("│ 🟢 = Acceptable  🟡 = ALARP  🔴 = Unacceptable\n");
        output.push_str("└─────────────────────────────────────────────────┘\n\n");
        
        // Column headers (Occurrence)
        output.push_str("         │");
        for occ_val in 1..=5 {
            let occurrence = Self::value_to_occurrence(occ_val);
            output.push_str(&format!(" {:^12} │", Self::occurrence_short_name(&occurrence)));
        }
        output.push('\n');
        
        // Separator
        output.push_str("─────────┼");
        for _ in 1..=5 {
            output.push_str("──────────────┼");
        }
        output.push('\n');
        
        // Matrix rows (Severity - highest to lowest for proper risk matrix display)
        for severity_val in (1..=5).rev() {
            let severity = Self::value_to_severity(severity_val);
            let severity_name = Self::severity_short_name(&severity);
            
            output.push_str(&format!("{severity_name:^9}│"));
            
            for occurrence_val in 1..=5 {
                let position = &self.matrix[(severity_val - 1) as usize][(occurrence_val - 1) as usize];
                let symbol = Self::color_to_symbol(&position.color_code);
                let rpn_display = if position.rpn >= 100 { 
                    format!("{}*", position.rpn) 
                } else { 
                    position.rpn.to_string() 
                };
                
                output.push_str(&format!(" {symbol} {rpn_display:^8} │"));
            }
            output.push('\n');
        }
        
        // Footer with legend
        output.push_str("\nRisk Priority Numbers (RPN = Severity × Occurrence × Detectability)\n");
        output.push_str("Detectability factor: 3 (Moderate) used for matrix display\n");
        output.push_str("* High-priority risks requiring immediate attention\n\n");
        
        // Risk level thresholds
        output.push_str("Risk Level Thresholds:\n");
        output.push_str("• 🟢 Acceptable:   RPN 1-24   (No immediate action required)\n");
        output.push_str("• 🟡 ALARP:       RPN 25-99  (Mitigation recommended)\n");
        output.push_str("• 🔴 Unacceptable: RPN 100-125 (Immediate action required)\n");
        
        output
    }
    
    /// Generate detailed risk matrix report
    pub fn generate_detailed_report(&self) -> String {
        let mut output = String::new();
        
        output.push_str("═══════════════════════════════════════════════════════════════\n");
        output.push_str(&format!("         {} (v{})\n", self.title, self.version));
        output.push_str("         Medical Device Risk Assessment Matrix Report\n");
        output.push_str("═══════════════════════════════════════════════════════════════\n\n");
        
        output.push_str("ISO 14971 Risk Management for Medical Devices\n");
        output.push_str("This matrix assists in systematic risk assessment and control.\n\n");
        
        // Risk assessment scales
        output.push_str("SEVERITY SCALE (Medical Device Context):\n");
        output.push_str("  5 - Catastrophic: Death or permanent disability\n");
        output.push_str("  4 - Critical:     Serious injury requiring medical intervention\n");
        output.push_str("  3 - Major:        Moderate injury requiring treatment\n");
        output.push_str("  2 - Minor:        Minor injury, first aid required\n");
        output.push_str("  1 - Negligible:   No injury expected\n\n");
        
        output.push_str("OCCURRENCE SCALE (Probability of Occurrence):\n");
        output.push_str("  5 - Frequent:     Very likely (>1 in 10 uses)\n");
        output.push_str("  4 - Probable:     Likely (1 in 100 to 1 in 10)\n");
        output.push_str("  3 - Occasional:   Possible (1 in 1,000 to 1 in 100)\n");
        output.push_str("  2 - Remote:       Unlikely (1 in 10,000 to 1 in 1,000)\n");
        output.push_str("  1 - Improbable:   Very unlikely (<1 in 10,000)\n\n");
        
        output.push_str("DETECTABILITY SCALE (Ability to Detect Before Harm):\n");
        output.push_str("  1 - Very High:    Almost certain detection (>99%)\n");
        output.push_str("  2 - High:         Good chance of detection (90-99%)\n");
        output.push_str("  3 - Moderate:     Moderate chance (50-89%)\n");
        output.push_str("  4 - Low:          Poor chance (10-49%)\n");
        output.push_str("  5 - Very Low:     Cannot detect (<10%)\n\n");
        
        // Add the ASCII matrix
        output.push_str(&self.generate_ascii_matrix());
        
        // Risk management guidance
        output.push_str("\nRISK MANAGEMENT GUIDANCE:\n");
        output.push_str("═══════════════════════════════════════════════════════════════\n");
        output.push_str("🔴 UNACCEPTABLE RISKS (RPN 100-125):\n");
        output.push_str("   • Immediate risk reduction required\n");
        output.push_str("   • Product cannot be released until risks are mitigated\n");
        output.push_str("   • Consider design changes, safety features, warnings\n");
        output.push_str("   • Document all mitigation measures\n\n");
        
        output.push_str("🟡 ALARP RISKS (RPN 25-99):\n");
        output.push_str("   • Risk reduction efforts should be applied\n");
        output.push_str("   • Cost-benefit analysis of mitigation measures\n");
        output.push_str("   • Document justification if risk is accepted\n");
        output.push_str("   • Consider additional controls where practicable\n\n");
        
        output.push_str("🟢 ACCEPTABLE RISKS (RPN 1-24):\n");
        output.push_str("   • No immediate action required\n");
        output.push_str("   • Monitor during post-market surveillance\n");
        output.push_str("   • Document risk acceptance rationale\n");
        output.push_str("   • Periodic review recommended\n\n");
        
        output.push_str("Generated by QMS Risk Management System\n");
        output.push_str("Compliant with ISO 14971:2019 and FDA guidance\n");
        
        output
    }
    
    /// Generate CSV export of risk matrix
    pub fn generate_csv_export(&self) -> String {
        let mut csv = String::new();
        
        // Header
        csv.push_str("Severity,Occurrence,RPN,Risk_Level,Color_Code,Severity_Description,Occurrence_Description\n");
        
        // Data rows
        for (severity_idx, row) in self.matrix.iter().enumerate() {
            for (occurrence_idx, position) in row.iter().enumerate() {
                csv.push_str(&format!(
                    "{},{},{},{:?},{:?},{},{}\n",
                    severity_idx + 1,
                    occurrence_idx + 1,
                    position.rpn,
                    position.risk_level,
                    position.color_code,
                    Self::severity_description(&position.severity),
                    Self::occurrence_description(&position.occurrence)
                ));
            }
        }
        
        csv
    }
    
    /// Get position in matrix for given severity and occurrence
    pub fn get_position(&self, severity: &RiskSeverity, occurrence: &RiskOccurrence) -> Option<&RiskMatrixPosition> {
        let severity_idx = (severity.clone() as u32 - 1) as usize;
        let occurrence_idx = (occurrence.clone() as u32 - 1) as usize;
        
        if severity_idx < 5 && occurrence_idx < 5 {
            Some(&self.matrix[severity_idx][occurrence_idx])
        } else {
            None
        }
    }
    
    /// Calculate statistics for the risk matrix
    pub fn calculate_statistics(&self) -> RiskMatrixStatistics {
        let mut stats = RiskMatrixStatistics {
            total_positions: 25,
            acceptable_count: 0,
            alarp_count: 0,
            unacceptable_count: 0,
            min_rpn: u32::MAX,
            max_rpn: 0,
            avg_rpn: 0.0,
        };
        
        let mut total_rpn = 0u32;
        
        for row in &self.matrix {
            for position in row {
                match position.risk_level {
                    RiskLevel::Acceptable => stats.acceptable_count += 1,
                    RiskLevel::ALARP => stats.alarp_count += 1,
                    RiskLevel::Unacceptable => stats.unacceptable_count += 1,
                }
                
                stats.min_rpn = stats.min_rpn.min(position.rpn);
                stats.max_rpn = stats.max_rpn.max(position.rpn);
                total_rpn += position.rpn;
            }
        }
        
        stats.avg_rpn = total_rpn as f64 / 25.0;
        stats
    }
    
    // Helper functions for conversions and formatting
    
    const fn value_to_severity(value: u32) -> RiskSeverity {
        match value {
            5 => RiskSeverity::Catastrophic,
            4 => RiskSeverity::Critical,
            3 => RiskSeverity::Major,
            2 => RiskSeverity::Minor,
            _ => RiskSeverity::Negligible,
        }
    }
    
    const fn value_to_occurrence(value: u32) -> RiskOccurrence {
        match value {
            5 => RiskOccurrence::Frequent,
            4 => RiskOccurrence::Probable,
            3 => RiskOccurrence::Occasional,
            2 => RiskOccurrence::Remote,
            _ => RiskOccurrence::Improbable,
        }
    }
    
    const fn calculate_risk_level(rpn: u32) -> RiskLevel {
        match rpn {
            100..=125 => RiskLevel::Unacceptable,
            25..=99 => RiskLevel::ALARP,
            _ => RiskLevel::Acceptable,
        }
    }
    
    const fn risk_level_to_color(risk_level: &RiskLevel) -> MatrixColor {
        match risk_level {
            RiskLevel::Acceptable => MatrixColor::Green,
            RiskLevel::ALARP => MatrixColor::Yellow,
            RiskLevel::Unacceptable => MatrixColor::Red,
        }
    }
    
    const fn color_to_symbol(color: &MatrixColor) -> &'static str {
        match color {
            MatrixColor::Green => "🟢",
            MatrixColor::Yellow => "🟡",
            MatrixColor::Red => "🔴",
        }
    }
    
    const fn severity_short_name(severity: &RiskSeverity) -> &'static str {
        match severity {
            RiskSeverity::Catastrophic => "CATAS",
            RiskSeverity::Critical => "CRIT",
            RiskSeverity::Major => "MAJOR",
            RiskSeverity::Minor => "MINOR",
            RiskSeverity::Negligible => "NEGL",
        }
    }
    
    const fn occurrence_short_name(occurrence: &RiskOccurrence) -> &'static str {
        match occurrence {
            RiskOccurrence::Frequent => "FREQUENT",
            RiskOccurrence::Probable => "PROBABLE",
            RiskOccurrence::Occasional => "OCCASIONAL",
            RiskOccurrence::Remote => "REMOTE",
            RiskOccurrence::Improbable => "IMPROBABLE",
        }
    }
    
    const fn severity_description(severity: &RiskSeverity) -> &'static str {
        match severity {
            RiskSeverity::Catastrophic => "Death or permanent disability",
            RiskSeverity::Critical => "Serious injury requiring medical intervention",
            RiskSeverity::Major => "Moderate injury requiring treatment",
            RiskSeverity::Minor => "Minor injury, first aid required",
            RiskSeverity::Negligible => "No injury expected",
        }
    }
    
    const fn occurrence_description(occurrence: &RiskOccurrence) -> &'static str {
        match occurrence {
            RiskOccurrence::Frequent => "Very likely (>1 in 10 uses)",
            RiskOccurrence::Probable => "Likely (1 in 100 to 1 in 10)",
            RiskOccurrence::Occasional => "Possible (1 in 1,000 to 1 in 100)",
            RiskOccurrence::Remote => "Unlikely (1 in 10,000 to 1 in 1,000)",
            RiskOccurrence::Improbable => "Very unlikely (<1 in 10,000)",
        }
    }
}

/// Risk matrix statistics for analysis
#[derive(Debug, Clone)]
pub struct RiskMatrixStatistics {
    pub total_positions: u32,
    pub acceptable_count: u32,
    pub alarp_count: u32,
    pub unacceptable_count: u32,
    pub min_rpn: u32,
    pub max_rpn: u32,
    pub avg_rpn: f64,
}

impl Display for RiskMatrixStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, 
            "Risk Matrix Statistics:\n\
             • Total Positions: {}\n\
             • 🟢 Acceptable: {} ({:.1}%)\n\
             • 🟡 ALARP: {} ({:.1}%)\n\
             • 🔴 Unacceptable: {} ({:.1}%)\n\
             • RPN Range: {} - {}\n\
             • Average RPN: {:.1}",
            self.total_positions,
            self.acceptable_count, 
            (self.acceptable_count as f64 / self.total_positions as f64) * 100.0,
            self.alarp_count,
            (self.alarp_count as f64 / self.total_positions as f64) * 100.0,
            self.unacceptable_count,
            (self.unacceptable_count as f64 / self.total_positions as f64) * 100.0,
            self.min_rpn,
            self.max_rpn,
            self.avg_rpn
        )
    }
}

/// Risk scoring utilities for assessment calculations
pub struct RiskScoring;

impl RiskScoring {
    /// Calculate RPN (Risk Priority Number)
    /// REFACTORED: Delegates to centralized RiskCalculator to eliminate DRY violation
    pub fn calculate_rpn(severity: &RiskSeverity, occurrence: &RiskOccurrence, detectability: &RiskDetectability) -> u32 {
        RiskCalculator::calculate_rpn(severity, occurrence, detectability)
    }
    
    /// Determine risk level from RPN
    /// REFACTORED: Delegates to centralized RiskCalculator to eliminate DRY violation
    pub fn rpn_to_risk_level(rpn: u32) -> RiskLevel {
        // Convert from centralized RiskLevel to local RiskLevel
        match RiskCalculator::calculate_risk_level(rpn) {
            crate::utils::CalculatorRiskLevel::Unacceptable => RiskLevel::Unacceptable,
            crate::utils::CalculatorRiskLevel::ALARP => RiskLevel::ALARP,
            crate::utils::CalculatorRiskLevel::Acceptable => RiskLevel::Acceptable,
        }
    }
    
    /// Generate risk level explanation
    pub const fn explain_risk_level(risk_level: &RiskLevel) -> &'static str {
        match risk_level {
            RiskLevel::Unacceptable => 
                "UNACCEPTABLE: Immediate risk reduction required. Product cannot be released until risks are mitigated.",
            RiskLevel::ALARP => 
                "ALARP: As Low As Reasonably Practicable. Risk reduction efforts should be applied where practicable.",
            RiskLevel::Acceptable => 
                "ACCEPTABLE: No immediate action required. Monitor during post-market surveillance.",
        }
    }
    
    /// Calculate risk reduction effectiveness
    pub fn calculate_risk_reduction(initial_rpn: u32, residual_rpn: u32) -> f64 {
        if initial_rpn == 0 {
            return 0.0;
        }
        ((initial_rpn as f64 - residual_rpn as f64) / initial_rpn as f64) * 100.0
    }
    
    /// Suggest mitigation priorities based on RPN
    pub const fn suggest_mitigation_priority(rpn: u32) -> &'static str {
        match rpn {
            100..=125 => "CRITICAL: Immediate action required - halting development may be necessary",
            75..=99 => "HIGH: Schedule immediate risk reduction activities",
            50..=74 => "MEDIUM: Plan risk reduction within current development cycle",
            25..=49 => "LOW: Consider risk reduction where reasonably practicable",
            _ => "MONITOR: No immediate action required, periodic review recommended",
        }
    }
    
    /// Validate risk assessment parameters
    pub fn validate_assessment(severity: &RiskSeverity, occurrence: &RiskOccurrence, detectability: &RiskDetectability) -> Result<(), String> {
        let severity_val = severity.clone() as u32;
        let occurrence_val = occurrence.clone() as u32;
        let detectability_val = detectability.clone() as u32;
        
        if !(1..=5).contains(&severity_val) {
            return Err("Severity must be between 1 and 5".to_string());
        }
        
        if !(1..=5).contains(&occurrence_val) {
            return Err("Occurrence must be between 1 and 5".to_string());
        }
        
        if !(1..=5).contains(&detectability_val) {
            return Err("Detectability must be between 1 and 5".to_string());
        }
        
        Ok(())
    }
    
    /// Generate risk assessment summary
    pub fn generate_assessment_summary(
        severity: &RiskSeverity, 
        occurrence: &RiskOccurrence, 
        detectability: &RiskDetectability
    ) -> String {
        let rpn = Self::calculate_rpn(severity, occurrence, detectability);
        let risk_level = Self::rpn_to_risk_level(rpn);
        let explanation = Self::explain_risk_level(&risk_level);
        let priority = Self::suggest_mitigation_priority(rpn);
        
        format!(
            "Risk Assessment Summary:\n\
             • Severity: {:?} ({})\n\
             • Occurrence: {:?} ({})\n\
             • Detectability: {:?} ({})\n\
             • RPN: {}\n\
             • Risk Level: {:?}\n\
             • Explanation: {}\n\
             • Mitigation Priority: {}",
            severity, severity.clone() as u32,
            occurrence, occurrence.clone() as u32,
            detectability, detectability.clone() as u32,
            rpn, risk_level, explanation, priority
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_risk_matrix_creation() {
        let matrix = RiskMatrix::new();
        assert_eq!(matrix.matrix.len(), 5); // 5 severity levels
        assert_eq!(matrix.matrix[0].len(), 5); // 5 occurrence levels
    }
    
    #[test]
    fn test_risk_matrix_position_lookup() {
        let matrix = RiskMatrix::new();
        let position = matrix.get_position(&RiskSeverity::Critical, &RiskOccurrence::Probable);
        assert!(position.is_some());
        
        let pos = position.unwrap();
        assert_eq!(pos.severity, RiskSeverity::Critical);
        assert_eq!(pos.occurrence, RiskOccurrence::Probable);
        assert_eq!(pos.rpn, 48); // 4 * 4 * 3 = 48 (moderate detectability)
        assert_eq!(pos.risk_level, RiskLevel::ALARP);
        assert_eq!(pos.color_code, MatrixColor::Yellow);
    }
    
    #[test]
    fn test_rpn_calculation() {
        let rpn = RiskScoring::calculate_rpn(
            &RiskSeverity::Catastrophic,
            &RiskOccurrence::Frequent,
            &RiskDetectability::VeryLow
        );
        assert_eq!(rpn, 125); // 5 * 5 * 5 = 125 (maximum RPN)
        
        let risk_level = RiskScoring::rpn_to_risk_level(rpn);
        assert_eq!(risk_level, RiskLevel::Unacceptable);
    }
    
    #[test]
    fn test_risk_level_boundaries() {
        assert_eq!(RiskScoring::rpn_to_risk_level(1), RiskLevel::Acceptable);
        assert_eq!(RiskScoring::rpn_to_risk_level(24), RiskLevel::Acceptable);
        assert_eq!(RiskScoring::rpn_to_risk_level(25), RiskLevel::ALARP);
        assert_eq!(RiskScoring::rpn_to_risk_level(99), RiskLevel::ALARP);
        assert_eq!(RiskScoring::rpn_to_risk_level(100), RiskLevel::Unacceptable);
        assert_eq!(RiskScoring::rpn_to_risk_level(125), RiskLevel::Unacceptable);
    }
    
    #[test]
    fn test_risk_reduction_calculation() {
        let initial_rpn = 100;
        let residual_rpn = 25;
        let reduction = RiskScoring::calculate_risk_reduction(initial_rpn, residual_rpn);
        assert_eq!(reduction, 75.0); // 75% reduction
    }
    
    #[test]
    fn test_assessment_validation() {
        let result = RiskScoring::validate_assessment(
            &RiskSeverity::Major,
            &RiskOccurrence::Remote,
            &RiskDetectability::High
        );
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_matrix_statistics() {
        let matrix = RiskMatrix::new();
        let stats = matrix.calculate_statistics();
        
        assert_eq!(stats.total_positions, 25);
        assert!(stats.acceptable_count > 0);
        assert!(stats.alarp_count > 0);
        // With moderate detectability (3), max RPN is 75, so no unacceptable risks (which require RPN >= 100)
        assert_eq!(stats.unacceptable_count, 0);
        assert_eq!(stats.acceptable_count + stats.alarp_count + stats.unacceptable_count, 25);
        assert!(stats.min_rpn >= 3); // Minimum with moderate detectability
        assert!(stats.max_rpn <= 75); // Maximum with moderate detectability
    }
    
    #[test]
    fn test_ascii_matrix_generation() {
        let matrix = RiskMatrix::new();
        let ascii = matrix.generate_ascii_matrix();
        
        // Verify contains key elements
        assert!(ascii.contains("ISO 14971 Risk Assessment Matrix"));
        assert!(ascii.contains("🟢"));
        assert!(ascii.contains("🟡"));
        assert!(ascii.contains("🔴"));
        assert!(ascii.contains("CATAS"));
        assert!(ascii.contains("FREQUENT"));
        assert!(ascii.contains("RPN"));
    }
    
    #[test]
    fn test_csv_export() {
        let matrix = RiskMatrix::new();
        let csv = matrix.generate_csv_export();
        
        assert!(csv.contains("Severity,Occurrence,RPN,Risk_Level,Color_Code"));
        assert!(csv.contains("Death or permanent disability"));
        assert!(csv.contains("Very likely"));
    }
    
    #[test]
    fn test_assessment_summary() {
        let summary = RiskScoring::generate_assessment_summary(
            &RiskSeverity::Critical,
            &RiskOccurrence::Probable,
            &RiskDetectability::Low
        );
        
        assert!(summary.contains("Risk Assessment Summary"));
        assert!(summary.contains("Severity: Critical"));
        assert!(summary.contains("RPN: 64"));
        assert!(summary.contains("ALARP"));
    }
}
