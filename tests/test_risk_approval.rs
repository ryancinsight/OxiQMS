/*
 * QMS Risk Approval System Tests - Task 3.1.16
 * 
 * Tests for risk review and approval workflow system
 * Verifies electronic signatures and approval processes per ISO 14971
 * 
 * Author: QMS Development Team
 * Date: January 2025
 */

#[cfg(test)]
mod tests {
    use qms::modules::risk_manager::approval::{RiskApprovalManager, RiskWorkflowState, ApprovalDecision};
    use qms::modules::risk_manager::risk::{RiskManager, RiskSeverity, RiskOccurrence, RiskDetectability};
    use std::path::PathBuf;
    use std::fs;

    #[test]
    fn test_risk_approval_workflow() {
        // Set up test environment with unique directory
        let test_dir = std::env::temp_dir().join(format!("qms_test_approval_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = fs::create_dir_all(&test_dir);

        // Create and initialize risk manager
        let mut risk_manager = RiskManager::new(&test_dir).unwrap();
        risk_manager.initialize().unwrap(); // Initialize directory structure

        // Create test risk
        let risk = risk_manager.create_risk(
            "Test software crash",
            "During data processing",
            "Data loss for patient"
        ).unwrap();
        
        // Assess the risk
        let _ = risk_manager.assess_risk(
            &risk.id,
            Some(RiskSeverity::Major),
            Some(RiskOccurrence::Occasional),
            Some(RiskDetectability::Low)
        );
        
        // Create approval manager
        let mut approval_manager = RiskApprovalManager::new(&test_dir);
        
        // Test risk submission
        let result = approval_manager.submit_risk_for_review(
            &risk.id,
            "test_user",
            "Test User",
            "Risk ready for approval"
        );
        
        assert!(result.is_ok(), "Risk submission should succeed");
        
        // Verify workflow state
        let state = approval_manager.get_current_workflow_state(&risk.id).unwrap();
        assert_eq!(state, RiskWorkflowState::Submitted);
        
        // Test risk approval
        let approval_result = approval_manager.approve_risk(
            &risk.id,
            "approver_user",
            "Approver User",
            "quality_manager",
            "Risk acceptable per ISO 14971",
            ApprovalDecision::Approve,
            vec![],
            "Risk assessment reviewed and approved"
        );
        
        assert!(approval_result.is_ok(), "Risk approval should succeed");
        
        // Verify new workflow state
        let new_state = approval_manager.get_current_workflow_state(&risk.id).unwrap();
        assert_eq!(new_state, RiskWorkflowState::Approved);
        
        // Test workflow history
        let history = approval_manager.get_workflow_history(&risk.id).unwrap();
        assert_eq!(history.len(), 2, "Should have 2 workflow entries");
        
        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }
    
    #[test]
    fn test_conditional_approval() {
        // Set up test environment with unique directory
        let test_dir = std::env::temp_dir().join(format!("qms_test_conditional_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = fs::create_dir_all(&test_dir);

        // Create and initialize risk manager
        let mut risk_manager = RiskManager::new(&test_dir).unwrap();
        risk_manager.initialize().unwrap(); // Initialize directory structure

        // Create test risk
        let risk = risk_manager.create_risk(
            "Test hazard",
            "Test situation",
            "Test harm"
        ).unwrap();
        
        let mut approval_manager = RiskApprovalManager::new(&test_dir);
        
        // Submit risk
        let _ = approval_manager.submit_risk_for_review(
            &risk.id,
            "test_user",
            "Test User",
            "Risk ready for conditional approval"
        );
        
        // Approve with conditions
        let result = approval_manager.approve_risk(
            &risk.id,
            "approver_user",
            "Approver User",
            "quality_manager",
            "Risk acceptable with conditions",
            ApprovalDecision::ApproveWithConditions,
            vec!["Additional testing required".to_string()],
            "Risk approved with specific conditions"
        );
        
        assert!(result.is_ok(), "Conditional approval should succeed");
        
        // Verify state
        let state = approval_manager.get_current_workflow_state(&risk.id).unwrap();
        assert_eq!(state, RiskWorkflowState::ApprovedWithConditions);
        
        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }
    
    #[test]
    fn test_workflow_report_generation() {
        let test_dir = PathBuf::from("test_workflow_report");
        let approval_manager = RiskApprovalManager::new(&test_dir);
        
        // Generate report
        let report = approval_manager.generate_workflow_report();
        
        assert!(report.contains("Risk Workflow Status Report"));
        assert!(report.contains("Summary Statistics"));
        assert!(report.contains("Total Risks in Workflow"));
        assert!(report.contains("Pending Approvals"));
        assert!(report.contains("Overdue Approvals"));
    }
}
