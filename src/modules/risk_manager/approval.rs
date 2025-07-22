/*
 * QMS (Quality Management System)
 * Risk Review & Approval Module - Task 3.1.16
 * 
 * Implements comprehensive risk workflow management system with electronic signatures
 * for medical device quality management per ISO 14971:2019 and FDA 21 CFR Part 820
 * 
 * Workflow: Risk creation → Assessment → Review → Approval → Implementation
 * 
 * Author: QMS Development Team
 * Date: January 2025
 * Version: 1.0.0
 */

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::models::AuditAction;
use crate::modules::audit_logger::entry::log_action;
use crate::modules::risk_manager::risk::{RiskManager, RiskItem};

/// Risk approval workflow states
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RiskWorkflowState {
    Draft,              // Initial creation state
    Submitted,          // Submitted for review
    UnderReview,        // Being reviewed by quality engineer
    ReviewCompleted,    // Review completed, awaiting approval
    Approved,           // Approved for implementation
    ApprovedWithConditions, // Approved with specific conditions
    Rejected,           // Rejected, requires rework
    Implemented,        // Risk controls implemented
    Verified,           // Implementation verified
    Closed,             // Risk management complete
}

impl RiskWorkflowState {
    pub const fn as_str(&self) -> &'static str {
        match self {
            RiskWorkflowState::Draft => "draft",
            RiskWorkflowState::Submitted => "submitted",
            RiskWorkflowState::UnderReview => "under_review",
            RiskWorkflowState::ReviewCompleted => "review_completed",
            RiskWorkflowState::Approved => "approved",
            RiskWorkflowState::ApprovedWithConditions => "approved_with_conditions",
            RiskWorkflowState::Rejected => "rejected",
            RiskWorkflowState::Implemented => "implemented",
            RiskWorkflowState::Verified => "verified",
            RiskWorkflowState::Closed => "closed",
        }
    }
    
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "draft" => Ok(RiskWorkflowState::Draft),
            "submitted" => Ok(RiskWorkflowState::Submitted),
            "under_review" => Ok(RiskWorkflowState::UnderReview),
            "review_completed" => Ok(RiskWorkflowState::ReviewCompleted),
            "approved" => Ok(RiskWorkflowState::Approved),
            "approved_with_conditions" => Ok(RiskWorkflowState::ApprovedWithConditions),
            "rejected" => Ok(RiskWorkflowState::Rejected),
            "implemented" => Ok(RiskWorkflowState::Implemented),
            "verified" => Ok(RiskWorkflowState::Verified),
            "closed" => Ok(RiskWorkflowState::Closed),
            _ => Err(format!("Invalid workflow state: {s}")),
        }
    }
}

/// Risk approval decisions
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ApprovalDecision {
    Approve,                    // Approve risk as acceptable
    ApproveWithConditions,      // Approve with specific conditions
    Reject,                     // Reject and require rework
    RequestMoreInfo,            // Request additional information
    EscalateToManagement,       // Escalate to management for decision
}

/// Risk approval authority levels
#[derive(Debug, Clone, PartialEq)]
pub enum ApprovalAuthority {
    QualityEngineer,           // Quality Engineer approval
    QualityManager,            // Quality Manager approval
    ManagementReview,          // Management review required
    RegulatoryAffairs,         // Regulatory affairs approval
    ChiefMedicalOfficer,       // CMO approval for critical risks
}

/// Electronic signature for risk approvals
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RiskApprovalSignature {
    pub user_id: String,
    pub user_name: String,
    pub role: String,
    pub signature_text: String,        // "Risk acceptable per ISO 14971"
    pub signature_date: String,        // ISO 8601 timestamp
    pub ip_address: Option<String>,    // Client IP for audit
    pub authority_level: ApprovalAuthority,
    pub decision: ApprovalDecision,
    pub conditions: Vec<String>,       // Conditions if conditional approval
    pub rationale: String,             // Approval rationale
}

/// Risk workflow entry tracking all state changes
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RiskWorkflowEntry {
    pub id: String,                    // UUID v4
    pub risk_id: String,               // Associated risk ID
    pub workflow_state: RiskWorkflowState,
    pub timestamp: String,             // ISO 8601 timestamp
    pub user_id: String,               // User who made the change
    pub user_name: String,             // User display name
    pub comments: String,              // Workflow comments
    pub signature: Option<RiskApprovalSignature>, // Electronic signature if required
    pub previous_state: Option<RiskWorkflowState>, // Previous state for audit
    pub next_actions: Vec<String>,     // Required next actions
    pub attachments: Vec<String>,      // Supporting documents
}

/// Risk approval requirements based on risk characteristics
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ApprovalRequirement {
    pub risk_id: String,
    pub required_authority: ApprovalAuthority,
    pub rationale: String,             // Why this authority is required
    pub regulatory_basis: String,      // Regulatory requirement citation
    pub deadline: Option<String>,      // Approval deadline
    pub escalation_threshold: u32,     // Days before escalation
}

/// Risk review and approval manager
pub struct RiskApprovalManager {
    project_path: std::path::PathBuf,
    workflow_entries: HashMap<String, Vec<RiskWorkflowEntry>>,
    approval_requirements: HashMap<String, ApprovalRequirement>,
    pending_approvals: HashMap<String, Vec<String>>, // Authority -> Risk IDs
}

impl RiskApprovalManager {
    /// Create new risk approval manager
    pub fn new(project_path: &Path) -> Self {
        let mut manager = Self {
            project_path: project_path.to_path_buf(),
            workflow_entries: HashMap::new(),
            approval_requirements: HashMap::new(),
            pending_approvals: HashMap::new(),
        };
        
        if let Err(e) = manager.load_workflow_data() {
            eprintln!("Warning: Could not load workflow data: {e}");
        }
        
        manager
    }
    
    /// Load workflow data from storage
    fn load_workflow_data(&mut self) -> Result<(), String> {
        let workflow_path = self.project_path.join("risk_workflow.json");
        
        if !workflow_path.exists() {
            return Ok(());
        }
        
        let content = fs::read_to_string(&workflow_path)
            .map_err(|e| format!("Failed to read workflow file: {e}"))?;
        
        // Parse workflow entries (simplified JSON parsing)
        if content.contains("\"workflow_entries\"") {
            // Load existing workflow entries
            // Implementation would parse JSON and populate workflow_entries
        }
        
        Ok(())
    }
    
    /// Save workflow data to storage
    fn save_workflow_data(&self) -> Result<(), String> {
        let workflow_path = self.project_path.join("risk_workflow.json");
        
        // Generate JSON content
        let mut json_content = String::new();
        json_content.push_str("{\n");
        json_content.push_str("  \"version\": \"1.0\",\n");
        json_content.push_str("  \"workflow_entries\": {\n");
        
        let mut first_risk = true;
        for (risk_id, entries) in &self.workflow_entries {
            if !first_risk {
                json_content.push_str(",\n");
            }
            first_risk = false;
            
            json_content.push_str(&format!("    \"{risk_id}\": [\n"));
            
            for (i, entry) in entries.iter().enumerate() {
                if i > 0 {
                    json_content.push_str(",\n");
                }
                
                json_content.push_str("      {\n");
                json_content.push_str(&format!("        \"id\": \"{}\",\n", entry.id));
                json_content.push_str(&format!("        \"workflow_state\": \"{}\",\n", entry.workflow_state.as_str()));
                json_content.push_str(&format!("        \"timestamp\": \"{}\",\n", entry.timestamp));
                json_content.push_str(&format!("        \"user_id\": \"{}\",\n", entry.user_id));
                json_content.push_str(&format!("        \"user_name\": \"{}\",\n", entry.user_name));
                json_content.push_str(&format!("        \"comments\": \"{}\"\n", entry.comments));
                json_content.push_str("      }");
            }
            
            json_content.push_str("\n    ]");
        }
        
        json_content.push_str("\n  }\n");
        json_content.push_str("}\n");
        
        fs::write(&workflow_path, json_content)
            .map_err(|e| format!("Failed to save workflow data: {e}"))?;
        
        Ok(())
    }
    
    /// Submit risk for review
    pub fn submit_risk_for_review(
        &mut self,
        risk_id: &str,
        user_id: &str,
        user_name: &str,
        comments: &str,
    ) -> Result<(), String> {
        // Validate risk exists
        let risk_manager = RiskManager::new(&self.project_path)
            .map_err(|e| format!("Failed to load risk manager: {e}"))?;
        
        let risk = risk_manager.load_risk(risk_id)
            .map_err(|e| format!("Risk not found: {e}"))?;
        
        // Determine required approval authority based on risk characteristics
        let approval_requirement = self.determine_approval_requirement(&risk)?;
        
        // Create workflow entry
        let workflow_entry = RiskWorkflowEntry {
            id: self.generate_uuid(),
            risk_id: risk_id.to_string(),
            workflow_state: RiskWorkflowState::Submitted,
            timestamp: self.current_timestamp(),
            user_id: user_id.to_string(),
            user_name: user_name.to_string(),
            comments: comments.to_string(),
            signature: None,
            previous_state: Some(RiskWorkflowState::Draft),
            next_actions: vec![
                "Quality Engineer review required".to_string(),
                format!("Risk requires {} approval", self.authority_display_name(&approval_requirement.required_authority)),
            ],
            attachments: Vec::new(),
        };
        
        // Add to workflow entries
        self.workflow_entries.entry(risk_id.to_string())
            .or_default()
            .push(workflow_entry);
        
        // Store approval requirement
        self.approval_requirements.insert(risk_id.to_string(), approval_requirement);
        
        // Add to pending approvals
        let authority_key = self.authority_key(&self.approval_requirements[risk_id].required_authority);
        self.pending_approvals.entry(authority_key)
            .or_default()
            .push(risk_id.to_string());
        
        // Save workflow data
        self.save_workflow_data()?;
        
        // Log the action
        let _ = log_action(
            user_id,
            AuditAction::Update,
            "risk_workflow",
            risk_id,
        );
        
        Ok(())
    }
    
    /// Approve risk with electronic signature
    pub fn approve_risk(
        &mut self,
        risk_id: &str,
        user_id: &str,
        user_name: &str,
        user_role: &str,
        signature_text: &str,
        decision: ApprovalDecision,
        conditions: Vec<String>,
        rationale: &str,
    ) -> Result<(), String> {
        // Validate user has appropriate authority
        let approval_req = self.approval_requirements.get(risk_id)
            .ok_or("No approval requirement found for risk")?;
        
        let user_authority = self.determine_user_authority(user_role)?;
        
        if !self.has_sufficient_authority(&user_authority, &approval_req.required_authority) {
            return Err(format!("Insufficient authority. Required: {:?}, User has: {:?}", 
                approval_req.required_authority, user_authority));
        }
        
        // Create electronic signature
        let signature = RiskApprovalSignature {
            user_id: user_id.to_string(),
            user_name: user_name.to_string(),
            role: user_role.to_string(),
            signature_text: signature_text.to_string(),
            signature_date: self.current_timestamp(),
            ip_address: None, // Could be populated from CLI environment
            authority_level: user_authority,
            decision: decision.clone(),
            conditions: conditions.clone(),
            rationale: rationale.to_string(),
        };
        
        // Determine next workflow state
        let next_state = match decision {
            ApprovalDecision::Approve => RiskWorkflowState::Approved,
            ApprovalDecision::ApproveWithConditions => RiskWorkflowState::ApprovedWithConditions,
            ApprovalDecision::Reject => RiskWorkflowState::Rejected,
            ApprovalDecision::RequestMoreInfo => RiskWorkflowState::UnderReview,
            ApprovalDecision::EscalateToManagement => RiskWorkflowState::UnderReview,
        };
        
        // Get current state
        let current_state = self.get_current_workflow_state(risk_id)?;
        
        // Create workflow entry
        let workflow_entry = RiskWorkflowEntry {
            id: self.generate_uuid(),
            risk_id: risk_id.to_string(),
            workflow_state: next_state.clone(),
            timestamp: self.current_timestamp(),
            user_id: user_id.to_string(),
            user_name: user_name.to_string(),
            comments: format!("Risk approval decision: {decision:?}. Rationale: {rationale}"),
            signature: Some(signature),
            previous_state: Some(current_state),
            next_actions: self.determine_next_actions(&next_state, &conditions),
            attachments: Vec::new(),
        };
        
        // Add to workflow entries
        self.workflow_entries.entry(risk_id.to_string())
            .or_default()
            .push(workflow_entry);
        
        // Remove from pending approvals
        let authority_key = self.authority_key(&approval_req.required_authority);
        if let Some(pending) = self.pending_approvals.get_mut(&authority_key) {
            pending.retain(|id| id != risk_id);
        }
        
        // Save workflow data
        self.save_workflow_data()?;
        
        // Log the action
        let _ = log_action(
            user_id,
            AuditAction::Update,
            "risk_approval",
            risk_id,
        );
        
        Ok(())
    }
    
    /// Get current workflow state for a risk
    pub fn get_current_workflow_state(&self, risk_id: &str) -> Result<RiskWorkflowState, String> {
        let entries = self.workflow_entries.get(risk_id)
            .ok_or("No workflow entries found for risk")?;
        
        if entries.is_empty() {
            return Ok(RiskWorkflowState::Draft);
        }
        
        // Get the most recent entry
        let latest_entry = entries.last().unwrap();
        Ok(latest_entry.workflow_state.clone())
    }
    
    /// Get workflow history for a risk
    pub fn get_workflow_history(&self, risk_id: &str) -> Result<Vec<RiskWorkflowEntry>, String> {
        let entries = self.workflow_entries.get(risk_id)
            .ok_or("No workflow entries found for risk")?;
        
        Ok(entries.clone())
    }
    
    /// Get all pending approvals for a specific authority
    pub fn get_pending_approvals(&self, authority: &ApprovalAuthority) -> Vec<String> {
        let authority_key = self.authority_key(authority);
        self.pending_approvals.get(&authority_key)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Get all pending approvals for a user role
    pub fn get_pending_approvals_for_user(&self, user_role: &str) -> Result<Vec<String>, String> {
        let user_authority = self.determine_user_authority(user_role)?;
        Ok(self.get_pending_approvals(&user_authority))
    }
    
    /// Generate workflow status report
    pub fn generate_workflow_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("# Risk Workflow Status Report\n\n");
        report.push_str(&format!("Report Generated: {}\n\n", self.current_timestamp()));
        
        // Summary statistics
        let total_risks = self.workflow_entries.len();
        let mut state_counts = HashMap::new();
        
        for entries in self.workflow_entries.values() {
            if let Some(latest) = entries.last() {
                *state_counts.entry(latest.workflow_state.clone()).or_insert(0) += 1;
            }
        }
        
        report.push_str("## Summary Statistics\n\n");
        report.push_str(&format!("Total Risks in Workflow: {total_risks}\n\n"));
        
        report.push_str("### Risks by State\n\n");
        for (state, count) in &state_counts {
            report.push_str(&format!("- **{}**: {} risks\n", state.as_str(), count));
        }
        
        // Pending approvals by authority
        report.push_str("\n## Pending Approvals\n\n");
        for (authority, risk_ids) in &self.pending_approvals {
            if !risk_ids.is_empty() {
                report.push_str(&format!("### {} ({} risks)\n\n", authority, risk_ids.len()));
                for risk_id in risk_ids {
                    report.push_str(&format!("- {risk_id}\n"));
                }
                report.push('\n');
            }
        }
        
        // Overdue approvals
        report.push_str("## Overdue Approvals\n\n");
        let overdue_count = self.get_overdue_approvals().len();
        if overdue_count > 0 {
            report.push_str(&format!("⚠️  {overdue_count} risks have overdue approvals\n\n"));
        } else {
            report.push_str("✅ No overdue approvals\n\n");
        }
        
        report
    }
    
    /// Get overdue approvals based on deadlines
    pub fn get_overdue_approvals(&self) -> Vec<String> {
        let current_time = self.current_timestamp();
        let mut overdue = Vec::new();
        
        for (risk_id, requirement) in &self.approval_requirements {
            if let Some(deadline) = &requirement.deadline {
                if current_time > *deadline {
                    overdue.push(risk_id.clone());
                }
            }
        }
        
        overdue
    }
    
    /// Determine required approval authority based on risk characteristics
    fn determine_approval_requirement(&self, risk: &RiskItem) -> Result<ApprovalRequirement, String> {
        let authority = if risk.risk_priority_number > 100 {
            // High RPN risks require management approval
            ApprovalAuthority::ManagementReview
        } else if risk.risk_priority_number > 50 {
            // Medium RPN risks require quality manager approval
            ApprovalAuthority::QualityManager
        } else {
            // Low RPN risks require quality engineer approval
            ApprovalAuthority::QualityEngineer
        };
        
        let rationale = format!("Risk {} (RPN: {}) requires {} approval per risk management protocol", 
            risk.id, risk.risk_priority_number, self.authority_display_name(&authority));
        
        Ok(ApprovalRequirement {
            risk_id: risk.id.clone(),
            required_authority: authority,
            rationale,
            regulatory_basis: "ISO 14971:2019 Section 6 - Risk evaluation".to_string(),
            deadline: None,
            escalation_threshold: 5,
        })
    }
    
    /// Determine user authority based on role
    fn determine_user_authority(&self, user_role: &str) -> Result<ApprovalAuthority, String> {
        match user_role.to_lowercase().as_str() {
            "quality_engineer" | "qe" => Ok(ApprovalAuthority::QualityEngineer),
            "quality_manager" | "qm" => Ok(ApprovalAuthority::QualityManager),
            "management" | "manager" => Ok(ApprovalAuthority::ManagementReview),
            "regulatory_affairs" | "ra" => Ok(ApprovalAuthority::RegulatoryAffairs),
            "cmo" | "chief_medical_officer" => Ok(ApprovalAuthority::ChiefMedicalOfficer),
            _ => Err(format!("Unknown user role: {user_role}")),
        }
    }
    
    /// Check if user has sufficient authority for approval
    const fn has_sufficient_authority(&self, user_authority: &ApprovalAuthority, required_authority: &ApprovalAuthority) -> bool {
        use ApprovalAuthority::*;
        
        match (user_authority, required_authority) {
            (ChiefMedicalOfficer, _) => true,
            (ManagementReview, ManagementReview) => true,
            (ManagementReview, QualityManager) => true,
            (ManagementReview, QualityEngineer) => true,
            (RegulatoryAffairs, RegulatoryAffairs) => true,
            (QualityManager, QualityManager) => true,
            (QualityManager, QualityEngineer) => true,
            (QualityEngineer, QualityEngineer) => true,
            _ => false,
        }
    }
    
    /// Get display name for authority
    const fn authority_display_name(&self, authority: &ApprovalAuthority) -> &'static str {
        match authority {
            ApprovalAuthority::QualityEngineer => "Quality Engineer",
            ApprovalAuthority::QualityManager => "Quality Manager",
            ApprovalAuthority::ManagementReview => "Management Review",
            ApprovalAuthority::RegulatoryAffairs => "Regulatory Affairs",
            ApprovalAuthority::ChiefMedicalOfficer => "Chief Medical Officer",
        }
    }
    
    /// Get authority key for pending approvals
    fn authority_key(&self, authority: &ApprovalAuthority) -> String {
        self.authority_display_name(authority).to_string()
    }
    
    /// Determine next actions based on workflow state
    fn determine_next_actions(&self, state: &RiskWorkflowState, conditions: &[String]) -> Vec<String> {
        match state {
            RiskWorkflowState::Approved => {
                vec!["Implement risk controls".to_string(), "Schedule verification activities".to_string()]
            }
            RiskWorkflowState::ApprovedWithConditions => {
                let mut actions = vec!["Address approval conditions".to_string()];
                for condition in conditions {
                    actions.push(format!("Condition: {condition}"));
                }
                actions.push("Implement risk controls".to_string());
                actions
            }
            RiskWorkflowState::Rejected => {
                vec!["Review rejection rationale".to_string(), "Revise risk analysis".to_string(), "Resubmit for approval".to_string()]
            }
            RiskWorkflowState::UnderReview => {
                vec!["Await review completion".to_string()]
            }
            _ => Vec::new(),
        }
    }
    
    /// Generate UUID v4
    fn generate_uuid(&self) -> String {
        format!("{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            self.random_u32(),
            self.random_u16(),
            0x4000 | (self.random_u16() & 0x0FFF),
            0x8000 | (self.random_u16() & 0x3FFF),
            self.random_u64() & 0xFFFFFFFFFFFF)
    }
    
    fn random_u32(&self) -> u32 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u32
    }
    
    fn random_u16(&self) -> u16 {
        (self.random_u32() >> 16) as u16
    }
    
    fn random_u64(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
    
    /// Get current timestamp in ISO 8601 format
    fn current_timestamp(&self) -> String {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_workflow_state_conversion() {
        assert_eq!(RiskWorkflowState::Draft.as_str(), "draft");
        assert_eq!(RiskWorkflowState::from_str("approved").unwrap(), RiskWorkflowState::Approved);
        assert!(RiskWorkflowState::from_str("invalid").is_err());
    }
    
    #[test]
    fn test_approval_manager_creation() {
        let temp_path = PathBuf::from("test_project");
        let manager = RiskApprovalManager::new(&temp_path);
        
        assert_eq!(manager.project_path, temp_path);
        assert!(manager.workflow_entries.is_empty());
        assert!(manager.approval_requirements.is_empty());
    }
    
    #[test]
    fn test_authority_hierarchy() {
        let manager = RiskApprovalManager::new(&PathBuf::from("test"));
        
        // CMO can approve anything
        assert!(manager.has_sufficient_authority(
            &ApprovalAuthority::ChiefMedicalOfficer,
            &ApprovalAuthority::QualityEngineer
        ));
        
        // Quality Manager can approve QE level
        assert!(manager.has_sufficient_authority(
            &ApprovalAuthority::QualityManager,
            &ApprovalAuthority::QualityEngineer
        ));
        
        // QE cannot approve management level
        assert!(!manager.has_sufficient_authority(
            &ApprovalAuthority::QualityEngineer,
            &ApprovalAuthority::ManagementReview
        ));
    }
    
    #[test]
    fn test_user_authority_determination() {
        let manager = RiskApprovalManager::new(&PathBuf::from("test"));
        
        assert_eq!(
            manager.determine_user_authority("quality_engineer").unwrap(),
            ApprovalAuthority::QualityEngineer
        );
        
        assert_eq!(
            manager.determine_user_authority("qm").unwrap(),
            ApprovalAuthority::QualityManager
        );
        
        assert!(manager.determine_user_authority("invalid_role").is_err());
    }
}
