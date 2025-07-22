/// GRASP Principles Enhancement: Information Expert Pattern for Audit Domain
///
/// AuditExpert encapsulates all audit-related knowledge and business rules.
/// This follows the Information Expert principle by placing audit responsibilities
/// with the class that has the audit domain knowledge.

use crate::prelude::*;
use crate::models::{AuditEntry, AuditAction};
use std::collections::HashMap;

/// Audit level enumeration for demonstration
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum AuditLevel {
    Info,
    Warning,
    Error,
    Critical,
}

/// Audit domain expert that encapsulates all audit logic
/// Follows GRASP Information Expert principle
pub struct AuditExpert;

impl AuditExpert {
    /// Determine audit level based on action - Information Expert
    /// This class knows which actions require which audit levels
    pub fn determine_audit_level(action: &AuditAction) -> AuditLevel {
        match action {
            AuditAction::Create => AuditLevel::Info,
            AuditAction::Update => AuditLevel::Info,
            AuditAction::Delete => AuditLevel::Warning,
            AuditAction::Export => AuditLevel::Warning,
            _ => AuditLevel::Info,
        }
    }
    
    /// Validate audit entry completeness - Information Expert
    /// This class knows what makes a complete audit entry
    pub fn validate_audit_entry(entry: &AuditEntry) -> QmsResult<()> {
        if entry.user_id.is_empty() {
            return Err(QmsError::validation_error("Audit entry must have user ID"));
        }

        // Additional validation logic would go here
        // Working with existing AuditEntry structure

        Ok(())
    }
    
    /// Calculate audit retention period - Information Expert
    /// This class knows the business rules for audit retention
    pub fn calculate_retention_period(action: &AuditAction) -> u32 {
        match action {
            AuditAction::Delete => 2555, // 7 years for deletions
            AuditAction::Export => 2555, // 7 years for data exports
            AuditAction::Create | AuditAction::Update => 1825, // 5 years for modifications
            _ => 1095, // 3 years default
        }
    }
    
    /// Determine if audit entry requires immediate notification - Information Expert
    /// This class knows which events need immediate attention
    pub fn requires_immediate_notification(entry: &AuditEntry) -> bool {
        // Simplified logic for demonstration
        matches!(entry.action, AuditAction::Delete)
    }
    
    /// Generate audit summary statistics - Information Expert
    /// This class knows how to analyze audit data
    pub fn generate_audit_summary(entries: &[AuditEntry]) -> AuditSummary {
        let mut summary = AuditSummary::default();
        let mut user_activity = HashMap::new();

        for entry in entries {
            summary.total_entries += 1;

            // Count by user
            *user_activity.entry(entry.user_id.clone()).or_insert(0) += 1;

            // Track delete events as critical
            if matches!(entry.action, AuditAction::Delete) {
                summary.critical_events += 1;
            }
        }

        summary.user_activity = user_activity;
        summary
    }
    
    /// Assess audit compliance status - Information Expert
    /// This class knows the compliance requirements
    pub fn assess_compliance_status(entries: &[AuditEntry], days: u32) -> ComplianceStatus {
        let required_daily_entries = 10; // Minimum expected daily audit entries
        let expected_total = (required_daily_entries * days) as usize;

        let compliance_percentage = if expected_total > 0 {
            (entries.len() as f64 / expected_total as f64) * 100.0
        } else {
            100.0
        };

        let critical_events = entries.iter()
            .filter(|e| matches!(e.action, AuditAction::Delete))
            .count();

        ComplianceStatus {
            is_compliant: compliance_percentage >= 80.0 && critical_events == 0,
            compliance_percentage,
            total_entries: entries.len(),
            expected_entries: expected_total,
            critical_events,
            recommendations: Self::generate_compliance_recommendations(compliance_percentage, critical_events),
        }
    }
    
    /// Generate compliance recommendations - Information Expert
    /// This class knows what recommendations to make based on audit data
    fn generate_compliance_recommendations(compliance_percentage: f64, critical_events: usize) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if compliance_percentage < 80.0 {
            recommendations.push("Increase audit logging frequency to meet compliance requirements".to_string());
        }
        
        if critical_events > 0 {
            recommendations.push("Address all critical audit events immediately".to_string());
        }
        
        if compliance_percentage < 50.0 {
            recommendations.push("Review audit system configuration - logging may be insufficient".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("Audit compliance is satisfactory".to_string());
        }
        
        recommendations
    }
    
    /// Format audit entry for display - Information Expert
    /// This class knows how to format audit information
    pub fn format_audit_entry(entry: &AuditEntry) -> String {
        format!(
            "[{}] {:?} (User: {})",
            entry.timestamp,
            entry.action,
            entry.user_id
        )
    }
}

/// Audit summary statistics
#[derive(Debug, Default)]
pub struct AuditSummary {
    pub total_entries: usize,
    pub critical_events: usize,
    pub user_activity: HashMap<String, usize>,
}

/// Compliance assessment result
#[derive(Debug)]
pub struct ComplianceStatus {
    pub is_compliant: bool,
    pub compliance_percentage: f64,
    pub total_entries: usize,
    pub expected_entries: usize,
    pub critical_events: usize,
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_audit_level_determination() {
        assert_eq!(AuditExpert::determine_audit_level(&AuditAction::Delete), AuditLevel::Warning);
        assert_eq!(AuditExpert::determine_audit_level(&AuditAction::Create), AuditLevel::Info);
        assert_eq!(AuditExpert::determine_audit_level(&AuditAction::Update), AuditLevel::Info);
    }

    #[test]
    fn test_retention_period_calculation() {
        assert_eq!(AuditExpert::calculate_retention_period(&AuditAction::Delete), 2555);
        assert_eq!(AuditExpert::calculate_retention_period(&AuditAction::Create), 1825);
    }

    #[test]
    fn test_immediate_notification_requirement() {
        // Create a minimal audit entry for testing
        let test_entry = AuditEntry {
            id: "test".to_string(),
            timestamp: "1234567890".to_string(),
            user_id: "test_user".to_string(),
            session_id: Some("test_session".to_string()),
            action: AuditAction::Delete,
            entity_type: "test".to_string(),
            entity_id: "test".to_string(),
            old_value: None,
            new_value: None,
            details: None,
            signature: None,
            checksum: "".to_string(),
            previous_hash: None,
            ip_address: None,
        };

        assert!(AuditExpert::requires_immediate_notification(&test_entry));
    }
}
