//! Risk Communication Module
//! Task 3.1.17: Risk Communication Implementation
//! 
//! This module implements the risk communication system for notifying stakeholders
//! about risk status changes, high-priority risks, and approval requests.
//! Supports quality engineers, product managers, and regulatory affairs teams.

use crate::prelude::*;
use crate::modules::risk_manager::risk::{RiskItem, RiskManager, RiskStatus};
use crate::modules::audit_logger::functions::audit_log_action;
use crate::utils::current_timestamp;
use std::fs;
use std::path::Path;

/// Stakeholder types for risk communication
#[derive(Debug, Clone, PartialEq)]
pub enum StakeholderType {
    QualityEngineer,
    ProductManager,
    RegulatoryAffairs,
    ExecutiveManagement,
    DevelopmentTeam,
    QualityAssurance,
}

impl StakeholderType {
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "qe" | "quality" | "quality-engineer" => Some(StakeholderType::QualityEngineer),
            "pm" | "product" | "product-manager" => Some(StakeholderType::ProductManager),
            "ra" | "regulatory" | "regulatory-affairs" => Some(StakeholderType::RegulatoryAffairs),
            "exec" | "executive" | "management" => Some(StakeholderType::ExecutiveManagement),
            "dev" | "development" | "development-team" => Some(StakeholderType::DevelopmentTeam),
            "qa" | "quality-assurance" => Some(StakeholderType::QualityAssurance),
            _ => None,
        }
    }

    pub const fn to_string(&self) -> &'static str {
        match self {
            StakeholderType::QualityEngineer => "Quality Engineer",
            StakeholderType::ProductManager => "Product Manager",
            StakeholderType::RegulatoryAffairs => "Regulatory Affairs",
            StakeholderType::ExecutiveManagement => "Executive Management",
            StakeholderType::DevelopmentTeam => "Development Team",
            StakeholderType::QualityAssurance => "Quality Assurance",
        }
    }
}

/// Risk alert types
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum RiskAlertType {
    HighRiskIdentified,
    RiskStatusChanged,
    ApprovalRequired,
    MitigationOverdue,
    VerificationPending,
    ComplianceIssue,
    EscalationRequired,
}

impl RiskAlertType {
    pub const fn to_string(&self) -> &'static str {
        match self {
            RiskAlertType::HighRiskIdentified => "High Risk Identified",
            RiskAlertType::RiskStatusChanged => "Risk Status Changed",
            RiskAlertType::ApprovalRequired => "Approval Required",
            RiskAlertType::MitigationOverdue => "Mitigation Overdue",
            RiskAlertType::VerificationPending => "Verification Pending",
            RiskAlertType::ComplianceIssue => "Compliance Issue",
            RiskAlertType::EscalationRequired => "Escalation Required",
        }
    }
}

/// Risk communication message
#[derive(Debug, Clone)]
pub struct RiskCommunication {
    pub id: String,
    pub risk_id: String,
    pub alert_type: RiskAlertType,
    pub stakeholders: Vec<StakeholderType>,
    pub message: String,
    pub priority: CommunicationPriority,
    pub timestamp: u64,
    pub created_by: String,
    pub acknowledged: bool,
    pub acknowledged_by: Option<String>,
    pub acknowledged_at: Option<u64>,
}

/// Communication priority levels
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum CommunicationPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl CommunicationPriority {
    pub const fn to_string(&self) -> &'static str {
        match self {
            CommunicationPriority::Low => "Low",
            CommunicationPriority::Medium => "Medium",
            CommunicationPriority::High => "High",
            CommunicationPriority::Critical => "Critical",
        }
    }

    pub const fn emoji(&self) -> &'static str {
        match self {
            CommunicationPriority::Low => "🔵",
            CommunicationPriority::Medium => "🟡",
            CommunicationPriority::High => "🟠",
            CommunicationPriority::Critical => "🔴",
        }
    }
}

/// Risk communication manager
pub struct RiskCommunicationManager {
    project_path: String,
    communications: Vec<RiskCommunication>,
}

impl RiskCommunicationManager {
    pub fn new(project_path: &str) -> QmsResult<Self> {
        let comm_path = format!("{project_path}/risks/communications");
        if !Path::new(&comm_path).exists() {
            fs::create_dir_all(&comm_path)?;
        }
        
        let mut manager = Self {
            project_path: project_path.to_string(),
            communications: Vec::new(),
        };
        
        manager.load_communications()?;
        Ok(manager)
    }

    /// Send notification to stakeholders
    pub fn notify_stakeholders(
        &mut self,
        risk_id: &str,
        stakeholders: Vec<StakeholderType>,
        message: &str,
    ) -> QmsResult<()> {
        let communication = RiskCommunication {
            id: self.generate_communication_id(),
            risk_id: risk_id.to_string(),
            alert_type: RiskAlertType::RiskStatusChanged,
            stakeholders,
            message: message.to_string(),
            priority: CommunicationPriority::Medium,
            timestamp: current_timestamp(),
            created_by: "SYSTEM".to_string(), // TODO: Use actual user when auth is implemented
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
        };

        self.communications.push(communication.clone());
        self.save_communications()?;
        
        // Log to audit trail
        audit_log_action(
            "RISK_NOTIFICATION_SENT",
            "RiskCommunication",
            &communication.id,
        )?;

        println!("📢 Risk Notification Sent");
        println!("Risk ID: {risk_id}");
        println!("Message: {message}");
        println!("Stakeholders: {}", 
            communication.stakeholders.iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
        
        Ok(())
    }

    /// Generate automated risk alerts
    pub fn generate_risk_alerts(&mut self, risk_manager: &mut RiskManager) -> QmsResult<Vec<RiskCommunication>> {
        let mut alerts = Vec::new();
        
        // Check for high-risk items
        let high_risks = risk_manager.get_high_risk_items()?;
        for risk in high_risks {
            if risk.risk_priority_number >= 100 {
                let alert = self.create_high_risk_alert(&risk)?;
                alerts.push(alert);
            }
        }
        
        // Check for pending verifications
        let pending_verifications = risk_manager.get_risks_by_status(RiskStatus::Mitigated)?;
        for risk in pending_verifications {
            let alert = self.create_verification_pending_alert(&risk)?;
            alerts.push(alert);
        }
        
        // Check for overdue mitigations
        let overdue_risks = risk_manager.get_overdue_risks()?;
        for risk in overdue_risks {
            let alert = self.create_overdue_mitigation_alert(&risk)?;
            alerts.push(alert);
        }
        
        // Save all alerts
        self.communications.extend(alerts.clone());
        self.save_communications()?;
        
        Ok(alerts)
    }

    /// Create high-risk alert
    fn create_high_risk_alert(&self, risk: &RiskItem) -> QmsResult<RiskCommunication> {
        let priority = if risk.risk_priority_number >= 125 {
            CommunicationPriority::Critical
        } else if risk.risk_priority_number >= 100 {
            CommunicationPriority::High
        } else {
            CommunicationPriority::Medium
        };
        
        let stakeholders = vec![
            StakeholderType::QualityEngineer,
            StakeholderType::ProductManager,
            StakeholderType::RegulatoryAffairs,
        ];
        
        let message = format!(
            "High risk identified: {} (RPN: {}, Level: {}). Immediate attention required.",
            risk.hazard_description,
            risk.risk_priority_number,
            "ALARP" // TODO: Use actual risk level when available
        );
        
        Ok(RiskCommunication {
            id: self.generate_communication_id(),
            risk_id: risk.id.clone(),
            alert_type: RiskAlertType::HighRiskIdentified,
            stakeholders,
            message,
            priority,
            timestamp: current_timestamp(),
            created_by: "SYSTEM".to_string(),
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
        })
    }

    /// Create verification pending alert
    fn create_verification_pending_alert(&self, risk: &RiskItem) -> QmsResult<RiskCommunication> {
        let stakeholders = vec![
            StakeholderType::QualityEngineer,
            StakeholderType::QualityAssurance,
        ];
        
        let message = format!(
            "Risk verification pending: {} (RPN: {}). Verification required to complete risk management process.",
            risk.hazard_description,
            risk.risk_priority_number
        );
        
        Ok(RiskCommunication {
            id: self.generate_communication_id(),
            risk_id: risk.id.clone(),
            alert_type: RiskAlertType::VerificationPending,
            stakeholders,
            message,
            priority: CommunicationPriority::Medium,
            timestamp: current_timestamp(),
            created_by: "SYSTEM".to_string(),
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
        })
    }

    /// Create overdue mitigation alert
    fn create_overdue_mitigation_alert(&self, risk: &RiskItem) -> QmsResult<RiskCommunication> {
        let stakeholders = vec![
            StakeholderType::QualityEngineer,
            StakeholderType::ProductManager,
            StakeholderType::DevelopmentTeam,
        ];
        
        let message = format!(
            "Risk mitigation overdue: {} (RPN: {}). Mitigation measures require immediate action.",
            risk.hazard_description,
            risk.risk_priority_number
        );
        
        Ok(RiskCommunication {
            id: self.generate_communication_id(),
            risk_id: risk.id.clone(),
            alert_type: RiskAlertType::MitigationOverdue,
            stakeholders,
            message,
            priority: CommunicationPriority::High,
            timestamp: current_timestamp(),
            created_by: "SYSTEM".to_string(),
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
        })
    }

    /// Generate executive summary report
    pub fn generate_executive_summary(&self, risk_manager: &RiskManager) -> QmsResult<String> {
        let mut summary = String::new();
        
        summary.push_str("# Executive Risk Summary\n\n");
        summary.push_str(&format!("Generated: {}\n\n", crate::utils::format_timestamp(current_timestamp())));
        
        // Risk statistics
        let total_risks = risk_manager.get_total_risk_count()?;
        let high_risks = risk_manager.get_high_risk_count()?;
        let critical_risks = risk_manager.get_critical_risk_count()?;
        
        summary.push_str("## Risk Overview\n\n");
        summary.push_str(&format!("- Total Risks: {total_risks}\n"));
        summary.push_str(&format!("- High Risks (RPN ≥ 50): {high_risks}\n"));
        summary.push_str(&format!("- Critical Risks (RPN ≥ 100): {critical_risks}\n"));
        
        // Recent communications
        summary.push_str("\n## Recent Communications\n\n");
        let recent_comms: Vec<_> = self.communications.iter()
            .filter(|c| current_timestamp() - c.timestamp < 7 * 24 * 3600) // Last 7 days
            .collect();
        
        if recent_comms.is_empty() {
            summary.push_str("No recent communications.\n");
        } else {
            for comm in recent_comms {
                summary.push_str(&format!("- {} {}: {} ({})\n", 
                    comm.priority.emoji(),
                    comm.alert_type.to_string(),
                    comm.message,
                    crate::utils::format_timestamp(comm.timestamp)
                ));
            }
        }
        
        // Critical actions required
        summary.push_str("\n## Critical Actions Required\n\n");
        let critical_comms: Vec<_> = self.communications.iter()
            .filter(|c| c.priority == CommunicationPriority::Critical && !c.acknowledged)
            .collect();
        
        if critical_comms.is_empty() {
            summary.push_str("No critical actions required.\n");
        } else {
            for comm in critical_comms {
                summary.push_str(&format!("- {}: {}\n", comm.risk_id, comm.message));
            }
        }
        
        Ok(summary)
    }

    /// Generate technical risk details
    pub fn generate_technical_report(&self, risk_manager: &RiskManager) -> QmsResult<String> {
        let mut report = String::new();
        
        report.push_str("# Technical Risk Details\n\n");
        report.push_str(&format!("Generated: {}\n\n", crate::utils::format_timestamp(current_timestamp())));
        
        // Detailed risk analysis
        let risks = risk_manager.list_all_risks()?;
        
        report.push_str("## Risk Analysis\n\n");
        for risk in risks {
            report.push_str(&format!("### {} - {}\n\n", risk.id, risk.hazard_description));
            report.push_str(&format!("- **RPN**: {} (S:{} × O:{} × D:{})\n", 
                risk.risk_priority_number, risk.severity as u8, risk.occurrence as u8, risk.detectability as u8));
            report.push_str(&format!("- **Risk Level**: {}\n", "ALARP")); // TODO: Use actual risk level
            report.push_str(&format!("- **Status**: {}\n", risk.risk_status.to_string()));
            report.push_str(&format!("- **Hazardous Situation**: {}\n", risk.hazardous_situation));
            report.push_str(&format!("- **Harm**: {}\n", risk.harm));
            
            if !risk.mitigation_measures.is_empty() {
                report.push_str("- **Mitigations**:\n");
                for mitigation in &risk.mitigation_measures {
                    report.push_str(&format!("  - {}\n", mitigation.description));
                }
            }
            
            report.push('\n');
        }
        
        // Communication log
        report.push_str("## Communication Log\n\n");
        for comm in &self.communications {
            report.push_str(&format!("- **{}** [{}] {}: {}\n",
                crate::utils::format_timestamp(comm.timestamp),
                comm.priority.to_string(),
                comm.alert_type.to_string(),
                comm.message
            ));
        }
        
        Ok(report)
    }

    /// List all communications
    pub fn list_communications(&self, filter_stakeholder: Option<StakeholderType>) -> Vec<&RiskCommunication> {
        self.communications.iter()
            .filter(|comm| {
                if let Some(ref stakeholder) = filter_stakeholder {
                    comm.stakeholders.contains(stakeholder)
                } else {
                    true
                }
            })
            .collect()
    }

    /// Acknowledge communication
    pub fn acknowledge_communication(&mut self, comm_id: &str) -> QmsResult<()> {
        if let Some(comm) = self.communications.iter_mut().find(|c| c.id == comm_id) {
            comm.acknowledged = true;
            comm.acknowledged_by = Some("SYSTEM".to_string()); // TODO: Use actual user
            comm.acknowledged_at = Some(current_timestamp());
            
            self.save_communications()?;
            
            audit_log_action(
                "RISK_COMMUNICATION_ACKNOWLEDGED",
                "RiskCommunication",
                comm_id,
            )?;
            
            println!("✅ Communication {comm_id} acknowledged");
        } else {
            return Err(QmsError::not_found("Communication not found"));
        }
        
        Ok(())
    }

    /// Generate communication ID
    fn generate_communication_id(&self) -> String {
        format!("COMM-{:03}", self.communications.len() + 1)
    }

    /// Load communications from file
    fn load_communications(&mut self) -> QmsResult<()> {
        let comm_file = format!("{}/risks/communications/communications.json", self.project_path);
        if Path::new(&comm_file).exists() {
            let content = fs::read_to_string(&comm_file)?;
            if !content.trim().is_empty() {
                self.communications = self.parse_communications_json(&content)?;
            }
        }
        Ok(())
    }

    /// Save communications to file
    fn save_communications(&self) -> QmsResult<()> {
        let comm_file = format!("{}/risks/communications/communications.json", self.project_path);
        let json = self.serialize_communications_json();
        fs::write(&comm_file, json)?;
        Ok(())
    }

    /// Parse communications from JSON
    const fn parse_communications_json(&self, _json: &str) -> QmsResult<Vec<RiskCommunication>> {
        // Basic JSON parsing implementation
        // In a real implementation, this would be more robust
        Ok(Vec::new()) // Placeholder for now
    }

    /// Serialize communications to JSON
    fn serialize_communications_json(&self) -> String {
        let mut json = String::from("{\n  \"version\": \"1.0\",\n  \"communications\": [\n");
        
        for (i, comm) in self.communications.iter().enumerate() {
            if i > 0 {
                json.push_str(",\n");
            }
            json.push_str("    {\n");
            json.push_str(&format!("      \"id\": \"{}\",\n", comm.id));
            json.push_str(&format!("      \"risk_id\": \"{}\",\n", comm.risk_id));
            json.push_str(&format!("      \"alert_type\": \"{}\",\n", comm.alert_type.to_string()));
            json.push_str(&format!("      \"message\": \"{}\",\n", comm.message.replace("\"", "\\\"")));
            json.push_str(&format!("      \"priority\": \"{}\",\n", comm.priority.to_string()));
            json.push_str(&format!("      \"timestamp\": {},\n", comm.timestamp));
            json.push_str(&format!("      \"created_by\": \"{}\",\n", comm.created_by));
            json.push_str(&format!("      \"acknowledged\": {}\n", comm.acknowledged));
            json.push_str("    }");
        }
        
        json.push_str("\n  ]\n}");
        json
    }
}

// Helper trait implementations for RiskLevel
impl crate::modules::risk_manager::risk::RiskLevel {
    #[allow(dead_code)]
    pub const fn to_string(&self) -> &'static str {
        match self {
            crate::modules::risk_manager::risk::RiskLevel::Acceptable => "Acceptable",
            crate::modules::risk_manager::risk::RiskLevel::ALARP => "ALARP",
            crate::modules::risk_manager::risk::RiskLevel::Unacceptable => "Unacceptable",
        }
    }
}

impl crate::modules::risk_manager::risk::RiskStatus {
    pub const fn to_string(&self) -> &'static str {
        match self {
            crate::modules::risk_manager::risk::RiskStatus::Identified => "Identified",
            crate::modules::risk_manager::risk::RiskStatus::Assessed => "Assessed",
            crate::modules::risk_manager::risk::RiskStatus::Mitigated => "Mitigated",
            crate::modules::risk_manager::risk::RiskStatus::Verified => "Verified",
            crate::modules::risk_manager::risk::RiskStatus::Closed => "Closed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_stakeholder_type_parsing() {
        assert_eq!(StakeholderType::from_string("qe"), Some(StakeholderType::QualityEngineer));
        assert_eq!(StakeholderType::from_string("pm"), Some(StakeholderType::ProductManager));
        assert_eq!(StakeholderType::from_string("ra"), Some(StakeholderType::RegulatoryAffairs));
        assert_eq!(StakeholderType::from_string("invalid"), None);
    }

    #[test]
    fn test_communication_priority_formatting() {
        assert_eq!(CommunicationPriority::Critical.to_string(), "Critical");
        assert_eq!(CommunicationPriority::Critical.emoji(), "🔴");
        assert_eq!(CommunicationPriority::Low.emoji(), "🔵");
    }

    #[test]
    fn test_risk_alert_type_formatting() {
        assert_eq!(RiskAlertType::HighRiskIdentified.to_string(), "High Risk Identified");
        assert_eq!(RiskAlertType::ApprovalRequired.to_string(), "Approval Required");
    }

    #[test]
    fn test_communication_manager_initialization() {
        // Apply SOLID principles: Single Responsibility - test only initialization logic
        // Apply CUPID principles: Predictable - use proper isolation for consistent results
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        // Test the core business logic, not file system details (Dependency Inversion)
        let result = RiskCommunicationManager::new(project_path);
        assert!(result.is_ok(), "Communication manager initialization should succeed");

        // tempdir automatically cleans up when dropped (RAII pattern)
    }

    #[test]
    fn test_communication_id_generation() {
        // Apply SOLID principles: Single Responsibility - test only ID generation logic
        // Apply GRASP principles: Low Coupling - isolate test from file system details
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path().to_str().unwrap();

        // Test the core business logic (Information Expert principle)
        let manager = RiskCommunicationManager::new(project_path).unwrap();
        let id = manager.generate_communication_id();
        assert_eq!(id, "COMM-001", "First communication ID should be COMM-001");

        // tempdir automatically cleans up when dropped (RAII pattern)
    }
}
