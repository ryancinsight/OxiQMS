/// GRASP Principles Enhancement: Creator Pattern
///
/// DomainObjectFactory follows the Creator principle by being responsible
/// for creating domain objects. It has the information needed to create
/// objects and aggregates the created objects.

use crate::prelude::*;
use crate::models::{RiskItem, AuditEntry, AuditAction};
use crate::modules::risk_manager::{RiskSeverity, RiskOccurrence, RiskDetectability};
use crate::utils::current_timestamp;

/// Factory for creating domain objects following GRASP Creator principle
/// This class is responsible for creating objects because it:
/// 1. Has the initializing data for the objects
/// 2. Records the created objects
/// 3. Uses the created objects closely
pub struct DomainObjectFactory {
    project_id: String,
    created_by: String,
}

impl DomainObjectFactory {
    /// Create new factory instance
    pub fn new(project_id: String, created_by: String) -> Self {
        Self {
            project_id,
            created_by,
        }
    }

    /// Create a new risk item - Creator Pattern
    /// This factory knows how to properly initialize a RiskItem
    pub fn create_risk_item(
        &self,
        description: &str,
        severity: RiskSeverity,
        occurrence: RiskOccurrence,
        detectability: RiskDetectability,
    ) -> QmsResult<RiskItem> {
        let risk_id = format!("RISK-{:03}", self.generate_risk_number());
        let timestamp = current_timestamp();

        // Calculate RPN
        let rpn = (severity.clone() as u8 as u32) * (occurrence.clone() as u8 as u32) * (detectability.clone() as u8 as u32);

        Ok(RiskItem {
            id: risk_id,
            description: description.to_string(),
            severity: severity as u8,
            occurrence: occurrence as u8,
            detectability: detectability as u8,
            rpn: rpn as u16,
            mitigation: Some("To be determined".to_string()),
            created_at: timestamp,
            updated_at: timestamp,
        })
    }
    
    /// Create a new audit entry - Creator Pattern
    /// This factory knows how to properly initialize an AuditEntry
    pub fn create_audit_entry(
        &self,
        action: AuditAction,
        entity_type: &str,
        entity_id: &str,
    ) -> QmsResult<AuditEntry> {
        Ok(AuditEntry {
            id: format!("AUDIT-{:03}", self.generate_audit_number()),
            timestamp: current_timestamp().to_string(),
            user_id: self.created_by.clone(),
            session_id: Some("demo_session".to_string()),
            action,
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            old_value: None,
            new_value: None,
            details: None,
            signature: None,
            checksum: "".to_string(),
            previous_hash: None,
            ip_address: None,
        })
    }
    
    // Helper methods for object creation

    fn generate_risk_number(&self) -> u32 {
        // In a real implementation, this would query the database for the next number
        // For demo purposes, using a simple timestamp-based approach
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        (timestamp % 1000) as u32
    }

    fn generate_audit_number(&self) -> u32 {
        // Similar to risk number generation
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        ((timestamp % 1000) + 2000) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_risk_item() {
        let factory = DomainObjectFactory::new("TEST_PROJECT".to_string(), "test_user".to_string());

        let risk = factory.create_risk_item(
            "Electrical shock hazard",
            RiskSeverity::Minor,
            RiskOccurrence::Remote,
            RiskDetectability::High,
        ).unwrap();

        assert!(risk.id.starts_with("RISK-"));
        assert_eq!(risk.description, "Electrical shock hazard");
        assert_eq!(risk.severity, RiskSeverity::Minor as u8);
    }

    #[test]
    fn test_create_audit_entry() {
        let factory = DomainObjectFactory::new("TEST_PROJECT".to_string(), "test_user".to_string());

        let entry = factory.create_audit_entry(
            AuditAction::Create,
            "Risk",
            "RISK-001",
        ).unwrap();

        assert_eq!(entry.user_id, "test_user");
        assert_eq!(entry.action, AuditAction::Create);
        assert_eq!(entry.entity_type, "Risk");
        assert_eq!(entry.entity_id, "RISK-001");
    }
}
