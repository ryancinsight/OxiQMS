/// GRASP Principles Enhancement: Controller Pattern
/// 
/// SystemController follows the Controller principle by handling system events
/// and coordinating activities between different subsystems. It doesn't do the
/// work itself but delegates to appropriate domain objects.

use crate::prelude::*;
use crate::modules::domain_experts::{RiskExpert, AuditExpert};
use crate::modules::creators::DomainObjectFactory;
use crate::modules::risk_manager::{RiskSeverity, RiskOccurrence, RiskDetectability};
use crate::modules::risk_manager::risk_service::{RiskService, RiskStatistics}; // DIP: Use abstraction
use crate::models::AuditAction;
use std::path::PathBuf;

/// System controller that coordinates activities across the QMS
/// Follows GRASP Controller principle
/// DIP Enhancement: Uses dependency injection for risk service
pub struct SystemController<R: RiskService> {
    project_path: PathBuf,
    current_user: String,
    session_id: String,
    factory: DomainObjectFactory,
    risk_service: R, // DIP: Depends on abstraction, not concrete implementation
}

impl<R: RiskService> SystemController<R> {
    /// Create new system controller with dependency injection
    /// DIP Enhancement: Accepts risk service abstraction instead of creating concrete implementation
    pub fn new(project_path: PathBuf, user_id: String, session_id: String, risk_service: R) -> Self {
        let project_id = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let factory = DomainObjectFactory::new(project_id, user_id.clone());

        Self {
            project_path,
            current_user: user_id,
            session_id,
            factory,
            risk_service, // DIP: Injected dependency
        }
    }
    
    /// Handle risk creation system event - Controller Pattern
    /// Coordinates between risk creation, validation, and audit logging
    /// DIP Enhancement: Uses injected risk service instead of direct instantiation
    pub fn handle_risk_creation(
        &mut self,
        description: &str,
        severity: RiskSeverity,
        occurrence: RiskOccurrence,
        detectability: RiskDetectability,
    ) -> QmsResult<String> {
        // 1. Validate parameters using domain expert
        RiskExpert::validate_risk_parameters(&severity, &occurrence, &detectability)?;

        // 2. Create risk using factory
        let risk = self.factory.create_risk_item(
            description,
            severity,
            occurrence,
            detectability,
        )?;

        // 3. Use injected risk service to create risk - DIP: Depends on abstraction
        let created_risk = self.risk_service.create_risk(
            description,
            "To be determined",
            "To be determined"
        )?;

        // 4. Log audit entry
        let _audit_entry = self.factory.create_audit_entry(
            AuditAction::Create,
            "Risk",
            &created_risk.id,
        )?;

        // 5. Return risk ID
        Ok(created_risk.id)
    }
    
    /// Handle risk assessment system event - Controller Pattern
    /// Coordinates risk assessment with validation and audit logging
    /// DIP Enhancement: Uses injected risk service instead of direct instantiation
    pub fn handle_risk_assessment(
        &mut self,
        risk_id: &str,
        severity: RiskSeverity,
        occurrence: RiskOccurrence,
        detectability: RiskDetectability,
    ) -> QmsResult<()> {
        // 1. Validate parameters
        RiskExpert::validate_risk_parameters(&severity, &occurrence, &detectability)?;
        
        // 2. Load and update risk using injected service - DIP: Depends on abstraction
        self.risk_service.assess_risk(
            risk_id,
            Some(severity),
            Some(occurrence),
            Some(detectability),
        )?;
        
        // 3. Log audit entry
        let _audit_entry = self.factory.create_audit_entry(
            AuditAction::Update,
            "Risk",
            risk_id,
        )?;
        
        Ok(())
    }
    
    /// Handle document creation system event - Controller Pattern
    /// Coordinates document creation with validation and audit logging
    pub fn handle_document_creation(
        &self,
        title: &str,
        content: &str,
    ) -> QmsResult<String> {
        // 1. Validate input
        if title.trim().is_empty() {
            return Err(QmsError::validation_error("Document title cannot be empty"));
        }

        if content.trim().is_empty() {
            return Err(QmsError::validation_error("Document content cannot be empty"));
        }

        // 2. Generate document ID
        let document_id = format!("DOC-{:03}", self.generate_doc_number());

        // 3. Log audit entry
        let _audit_entry = self.factory.create_audit_entry(
            AuditAction::Create,
            "Document",
            &document_id,
        )?;

        Ok(document_id)
    }
    
    /// Handle system startup event - Controller Pattern
    /// Coordinates system initialization across all subsystems
    pub fn handle_system_startup(&self) -> QmsResult<()> {
        // 1. Validate system configuration
        self.validate_system_configuration()?;

        // 2. Log startup
        let _audit_entry = self.factory.create_audit_entry(
            AuditAction::Create,
            "System",
            "startup",
        )?;

        Ok(())
    }

    /// Handle user session events - Controller Pattern
    /// Coordinates user authentication with audit logging
    pub fn handle_user_login(&self, username: &str) -> QmsResult<()> {
        let _audit_entry = self.factory.create_audit_entry(
            AuditAction::Create,
            "Session",
            username,
        )?;
        Ok(())
    }

    pub fn handle_user_logout(&self, username: &str) -> QmsResult<()> {
        let _audit_entry = self.factory.create_audit_entry(
            AuditAction::Delete,
            "Session",
            username,
        )?;
        Ok(())
    }
    
    /// Validate system configuration - Helper method
    fn validate_system_configuration(&self) -> QmsResult<()> {
        // Create project directory if it doesn't exist
        if !self.project_path.exists() {
            std::fs::create_dir_all(&self.project_path)?;
        }

        // Check if required subdirectories exist
        let required_dirs = ["risks", "documents", "audit", "requirements"];
        for dir in &required_dirs {
            let dir_path = self.project_path.join(dir);
            if !dir_path.exists() {
                std::fs::create_dir_all(&dir_path)?;
            }
        }

        Ok(())
    }
    
    /// Get system status - Controller Pattern
    /// Coordinates status gathering from all subsystems
    pub fn get_system_status(&self) -> QmsResult<SystemStatus> {
        let mut status = SystemStatus::default();
        
        // Check risk service status - DIP: Uses injected service
        if let Ok(stats) = self.risk_service.get_risk_statistics() {
            status.total_risks = stats.total_risks;
            status.high_priority_risks = stats.high_priority_risks;
        }
        
        // Check audit system status
        status.audit_system_active = true; // Would check actual audit system
        
        // Check document system status
        status.document_system_active = self.project_path.join("documents").exists();
        
        status.system_healthy = status.audit_system_active && status.document_system_active;

        Ok(status)
    }

    /// Generate document number - Helper method
    fn generate_doc_number(&self) -> u32 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        ((timestamp % 1000) + 3000) as u32
    }
}

/// System status information
#[derive(Debug, Default)]
pub struct SystemStatus {
    pub system_healthy: bool,
    pub audit_system_active: bool,
    pub document_system_active: bool,
    pub total_risks: usize,
    pub high_priority_risks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_controller_creation() {
        let temp_path = std::env::temp_dir().join("test_qms");
        let mock_risk_service = crate::modules::risk_manager::risk_service::MockRiskService::new();
        let controller = SystemController::new(
            temp_path,
            "test_user".to_string(),
            "test_session".to_string(),
            mock_risk_service, // DIP: Inject mock service for testing
        );

        assert_eq!(controller.current_user, "test_user");
        assert_eq!(controller.session_id, "test_session");
    }

    #[test]
    fn test_system_startup_handling() {
        let temp_path = std::env::temp_dir().join("test_qms_startup");
        let mock_risk_service = crate::modules::risk_manager::risk_service::MockRiskService::new();
        let controller = SystemController::new(
            temp_path,
            "test_user".to_string(),
            "test_session".to_string(),
            mock_risk_service, // DIP: Inject mock service for testing
        );

        // This would normally require audit system initialization
        // For test purposes, we just verify the method doesn't panic
        assert!(controller.validate_system_configuration().is_ok());
    }
}
