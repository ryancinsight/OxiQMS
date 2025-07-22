/// KISS Improvement: Simplified Calculation Utilities
/// 
/// This module provides simple, easy-to-understand calculation utilities
/// that replace complex implementations, following the KISS principle.

use crate::prelude::*;

/// Simple calculation utilities for QMS operations
pub struct SimpleCalculations;

impl SimpleCalculations {
    /// Simple RPN calculation (KISS: replaces complex calculation logic)
    pub const fn calculate_rpn(severity: u8, occurrence: u8, detectability: u8) -> u16 {
        (severity as u16) * (occurrence as u16) * (detectability as u16)
    }
    
    /// Simple risk level determination (KISS: clear thresholds)
    pub const fn determine_risk_level(rpn: u16) -> RiskLevel {
        match rpn {
            100..=u16::MAX => RiskLevel::Unacceptable,
            50..=99 => RiskLevel::ALARP,
            1..=49 => RiskLevel::Acceptable,
            0 => RiskLevel::Acceptable,
        }
    }
    
    /// Simple percentage calculation (KISS: basic math)
    pub fn calculate_percentage(part: f64, total: f64) -> f64 {
        if total == 0.0 {
            0.0
        } else {
            (part / total) * 100.0
        }
    }
    
    /// Simple average calculation (KISS: basic statistics)
    pub fn calculate_average(values: &[f64]) -> f64 {
        if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<f64>() / values.len() as f64
        }
    }
    
    /// Simple compliance score (KISS: straightforward scoring)
    pub fn calculate_compliance_score(passed: usize, total: usize) -> f64 {
        if total == 0 {
            100.0 // No tests means 100% compliance
        } else {
            Self::calculate_percentage(passed as f64, total as f64)
        }
    }
    
    /// Simple trend calculation (KISS: basic trend analysis)
    pub fn calculate_trend(current: f64, previous: f64) -> TrendDirection {
        if current > previous {
            TrendDirection::Increasing
        } else if current < previous {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        }
    }
    
    /// Simple priority score (KISS: basic prioritization)
    pub fn calculate_priority_score(urgency: u8, importance: u8) -> u8 {
        // Simple multiplication with cap at 100
        let score = urgency * importance;
        if score > 100 {
            100
        } else {
            score
        }
    }
    
    /// Simple time estimation (KISS: basic time calculations)
    pub fn estimate_completion_time(
        total_items: usize,
        completed_items: usize,
        elapsed_hours: f64,
    ) -> f64 {
        if completed_items == 0 || elapsed_hours == 0.0 {
            return 0.0;
        }
        
        let rate = completed_items as f64 / elapsed_hours;
        let remaining_items = total_items.saturating_sub(completed_items) as f64;
        
        remaining_items / rate
    }
    
    /// Simple quality score (KISS: basic quality metrics)
    pub fn calculate_quality_score(
        defects: usize,
        total_items: usize,
        severity_weights: &[f64],
    ) -> f64 {
        if total_items == 0 {
            return 100.0;
        }
        
        let weighted_defects = if severity_weights.is_empty() {
            defects as f64
        } else {
            severity_weights.iter().sum::<f64>()
        };
        
        let defect_rate = weighted_defects / total_items as f64;
        let quality_score = (1.0 - defect_rate) * 100.0;
        
        quality_score.max(0.0).min(100.0)
    }
}

/// Simple risk level enumeration (KISS: clear categories)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Acceptable,
    ALARP,      // As Low As Reasonably Practicable
    Unacceptable,
}

impl RiskLevel {
    /// Simple risk level description (KISS: clear descriptions)
    pub const fn description(&self) -> &'static str {
        match self {
            RiskLevel::Acceptable => "Risk is acceptable and can proceed",
            RiskLevel::ALARP => "Risk is tolerable but should be reduced if reasonably practicable",
            RiskLevel::Unacceptable => "Risk is unacceptable and must be reduced",
        }
    }
    
    /// Simple action required (KISS: clear actions)
    pub const fn action_required(&self) -> &'static str {
        match self {
            RiskLevel::Acceptable => "Monitor during use",
            RiskLevel::ALARP => "Implement additional controls if reasonably practicable",
            RiskLevel::Unacceptable => "Implement risk controls before proceeding",
        }
    }
}

/// Simple trend direction (KISS: basic trend analysis)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

impl TrendDirection {
    pub const fn description(&self) -> &'static str {
        match self {
            TrendDirection::Increasing => "Trending upward",
            TrendDirection::Decreasing => "Trending downward",
            TrendDirection::Stable => "Stable",
        }
    }
}

/// Simple metrics calculator (KISS: basic metrics)
pub struct SimpleMetrics {
    pub total_count: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub values: Vec<f64>,
}

impl SimpleMetrics {
    pub fn new() -> Self {
        Self {
            total_count: 0,
            success_count: 0,
            failure_count: 0,
            values: Vec::new(),
        }
    }
    
    pub fn add_success(&mut self) {
        self.total_count += 1;
        self.success_count += 1;
    }
    
    pub fn add_failure(&mut self) {
        self.total_count += 1;
        self.failure_count += 1;
    }
    
    pub fn add_value(&mut self, value: f64) {
        self.values.push(value);
    }
    
    /// Simple success rate (KISS: basic calculation)
    pub fn success_rate(&self) -> f64 {
        SimpleCalculations::calculate_compliance_score(self.success_count, self.total_count)
    }
    
    /// Simple failure rate (KISS: basic calculation)
    pub fn failure_rate(&self) -> f64 {
        SimpleCalculations::calculate_compliance_score(self.failure_count, self.total_count)
    }
    
    /// Simple average value (KISS: basic statistics)
    pub fn average_value(&self) -> f64 {
        SimpleCalculations::calculate_average(&self.values)
    }
    
    /// Simple summary (KISS: basic reporting)
    pub fn summary(&self) -> MetricsSummary {
        MetricsSummary {
            total_count: self.total_count,
            success_rate: self.success_rate(),
            failure_rate: self.failure_rate(),
            average_value: self.average_value(),
        }
    }
}

impl Default for SimpleMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple metrics summary (KISS: basic summary structure)
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub total_count: usize,
    pub success_rate: f64,
    pub failure_rate: f64,
    pub average_value: f64,
}

impl MetricsSummary {
    /// Simple status assessment (KISS: basic status)
    pub fn status(&self) -> &'static str {
        if self.success_rate >= 95.0 {
            "Excellent"
        } else if self.success_rate >= 80.0 {
            "Good"
        } else if self.success_rate >= 60.0 {
            "Fair"
        } else {
            "Poor"
        }
    }
}

/// Simple utility functions (KISS: basic utilities)
pub struct SimpleUtils;

impl SimpleUtils {
    /// Simple ID generation (KISS: basic ID format)
    pub fn generate_simple_id(prefix: &str, counter: u32) -> String {
        format!("{}-{:03}", prefix, counter)
    }
    
    /// Simple timestamp (KISS: basic timestamp)
    pub fn simple_timestamp() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        timestamp.to_string()
    }
    
    /// Simple date string (KISS: basic date format)
    pub fn simple_date() -> String {
        // This is a simplified implementation
        // In a real system, you'd use a proper date library
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Simple conversion to YYYY-MM-DD format
        // This is approximate and for demonstration only
        let days_since_epoch = timestamp / 86400;
        let year = 1970 + (days_since_epoch / 365);
        let day_of_year = days_since_epoch % 365;
        let month = (day_of_year / 30) + 1;
        let day = (day_of_year % 30) + 1;
        
        format!("{:04}-{:02}-{:02}", year, month, day)
    }
    
    /// Simple hash (KISS: basic hash for checksums)
    pub fn simple_hash(input: &str) -> u64 {
        // Simple hash function for basic checksums
        let mut hash = 0u64;
        for byte in input.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
        hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rpn_calculation() {
        let rpn = SimpleCalculations::calculate_rpn(4, 3, 2);
        assert_eq!(rpn, 24);
        
        let level = SimpleCalculations::determine_risk_level(rpn);
        assert_eq!(level, RiskLevel::Acceptable);
    }
    
    #[test]
    fn test_percentage_calculation() {
        let percentage = SimpleCalculations::calculate_percentage(25.0, 100.0);
        assert_eq!(percentage, 25.0);
        
        let percentage_zero = SimpleCalculations::calculate_percentage(10.0, 0.0);
        assert_eq!(percentage_zero, 0.0);
    }
    
    #[test]
    fn test_simple_metrics() {
        let mut metrics = SimpleMetrics::new();
        metrics.add_success();
        metrics.add_success();
        metrics.add_failure();
        
        assert!((metrics.success_rate() - 66.67).abs() < 0.1); // Approximately 66.67%
        assert_eq!(metrics.total_count, 3);
    }
    
    #[test]
    fn test_trend_calculation() {
        let trend = SimpleCalculations::calculate_trend(10.0, 5.0);
        assert_eq!(trend, TrendDirection::Increasing);
        
        let trend_stable = SimpleCalculations::calculate_trend(5.0, 5.0);
        assert_eq!(trend_stable, TrendDirection::Stable);
    }
    
    #[test]
    fn test_simple_utils() {
        let id = SimpleUtils::generate_simple_id("TEST", 42);
        assert_eq!(id, "TEST-042");
        
        let hash1 = SimpleUtils::simple_hash("test");
        let hash2 = SimpleUtils::simple_hash("test");
        assert_eq!(hash1, hash2); // Same input should produce same hash
    }
}
