/*
 * QMS Risk Approval Command Handlers - Task 3.1.16
 * 
 * Command handlers for risk review and approval workflow system
 * Implements electronic signatures and approval processes per ISO 14971
 * 
 * Author: QMS Development Team
 * Date: January 2025
 */

/// Handle risk submission for approval - Task 3.1.16
pub fn handle_risk_submit(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        println!("Usage: qms risk submit <risk_id> [--comment \"submission comment\"]");
        return Ok(());
    }
    
    let risk_id = &args[0];
    let mut comment = "Risk submitted for approval review".to_string();
    
    // Parse comment if provided
    if args.len() >= 3 && args[1] == "--comment" {
        comment = args[2].clone();
    }
    
    // Initialize approval manager
    let mut approval_manager = RiskApprovalManager::new(Path::new("."));
    
    // Get current user (simplified for CLI)
    let user_id = "cli_user";
    let user_name = "CLI User";
    
    // Submit risk for review
    match approval_manager.submit_risk_for_review(risk_id, user_id, user_name, &comment) {
        Ok(()) => {
            println!("✅ Risk {risk_id} submitted for approval");
            println!("   Comment: {comment}");
            println!("   Status: Submitted for review");
            println!();
            
            // Show next steps
            println!("Next Steps:");
            println!("  1. Quality Engineer review required");
            println!("  2. Use 'qms risk workflow {risk_id}' to check status");
            println!("  3. Approver can use 'qms risk approve {risk_id}' when ready");
        }
        Err(e) => {
            eprintln!("❌ Error submitting risk for approval: {e}");
            return Err(e);
        }
    }
    
    Ok(())
}

/// Handle risk approval with electronic signature - Task 3.1.16
pub fn handle_risk_approve(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        println!("Usage: qms risk approve <risk_id> --signature \"Risk acceptable per ISO 14971\" [--role quality_engineer] [--reject] [--conditions \"condition1,condition2\"]");
        return Ok(());
    }
    
    let risk_id = &args[0];
    let mut signature_text = "Risk acceptable per ISO 14971".to_string();
    let mut user_role = "quality_engineer".to_string();
    let mut decision = ApprovalDecision::Approve;
    let mut conditions = Vec::new();
    let mut rationale = "Risk assessment reviewed and approved".to_string();
    
    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--signature" => {
                if i + 1 < args.len() {
                    signature_text = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--signature requires a value".to_string());
                }
            }
            "--role" => {
                if i + 1 < args.len() {
                    user_role = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--role requires a value".to_string());
                }
            }
            "--reject" => {
                decision = ApprovalDecision::Reject;
                rationale = "Risk assessment rejected, requires rework".to_string();
                i += 1;
            }
            "--conditions" => {
                if i + 1 < args.len() {
                    conditions = args[i + 1].split(',').map(|s| s.trim().to_string()).collect();
                    decision = ApprovalDecision::ApproveWithConditions;
                    i += 2;
                } else {
                    return Err("--conditions requires a value".to_string());
                }
            }
            "--rationale" => {
                if i + 1 < args.len() {
                    rationale = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--rationale requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }
    
    // Initialize approval manager
    let mut approval_manager = RiskApprovalManager::new(Path::new("."));
    
    // Get current user (simplified for CLI)
    let user_id = "cli_user";
    let user_name = "CLI User";
    
    // Approve risk
    match approval_manager.approve_risk(
        risk_id,
        user_id,
        user_name,
        &user_role,
        &signature_text,
        decision.clone(),
        conditions.clone(),
        &rationale,
    ) {
        Ok(()) => {
            println!("✅ Risk {risk_id} approval recorded");
            println!("   Decision: {decision:?}");
            println!("   Signature: {signature_text}");
            println!("   Role: {user_role}");
            println!("   Rationale: {rationale}");
            
            if !conditions.is_empty() {
                println!("   Conditions:");
                for condition in &conditions {
                    println!("     - {condition}");
                }
            }
            
            println!();
            
            // Show next steps based on decision
            match decision {
                ApprovalDecision::Approve => {
                    println!("Next Steps:");
                    println!("  1. Implement risk controls");
                    println!("  2. Schedule verification activities");
                    println!("  3. Use 'qms risk workflow {risk_id}' to track progress");
                }
                ApprovalDecision::ApproveWithConditions => {
                    println!("Next Steps:");
                    println!("  1. Address approval conditions");
                    println!("  2. Implement risk controls");
                    println!("  3. Schedule verification activities");
                }
                ApprovalDecision::Reject => {
                    println!("Next Steps:");
                    println!("  1. Review rejection rationale");
                    println!("  2. Revise risk analysis");
                    println!("  3. Resubmit for approval");
                }
                _ => {}
            }
        }
        Err(e) => {
            eprintln!("❌ Error approving risk: {e}");
            return Err(e);
        }
    }
    
    Ok(())
}

/// Handle workflow status inquiry - Task 3.1.16
pub fn handle_risk_workflow(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        println!("Usage: qms risk workflow <risk_id>");
        return Ok(());
    }
    
    let risk_id = &args[0];
    
    // Initialize approval manager
    let approval_manager = RiskApprovalManager::new(Path::new("."));
    
    // Get current workflow state
    match approval_manager.get_current_workflow_state(risk_id) {
        Ok(state) => {
            println!("Risk {risk_id} Workflow Status");
            println!("═══════════════════════════════════════");
            println!("Current State: {state:?}");
            println!();
            
            // Get workflow history
            match approval_manager.get_workflow_history(risk_id) {
                Ok(history) => {
                    if history.is_empty() {
                        println!("No workflow history found.");
                        println!("Use 'qms risk submit {risk_id}' to start the approval process.");
                    } else {
                        println!("Workflow History:");
                        println!("─────────────────");
                        
                        for (i, entry) in history.iter().enumerate() {
                            println!("{}. {} → {:?}", i + 1, entry.timestamp, entry.workflow_state);
                            println!("   User: {} ({})", entry.user_name, entry.user_id);
                            println!("   Comments: {}", entry.comments);
                            
                            if let Some(signature) = &entry.signature {
                                println!("   ✍️  Signature: {}", signature.signature_text);
                                println!("   📋 Decision: {:?}", signature.decision);
                                println!("   🔐 Authority: {:?}", signature.authority_level);
                                if !signature.conditions.is_empty() {
                                    println!("   ⚠️  Conditions:");
                                    for condition in &signature.conditions {
                                        println!("       - {condition}");
                                    }
                                }
                            }
                            
                            if !entry.next_actions.is_empty() {
                                println!("   📝 Next Actions:");
                                for action in &entry.next_actions {
                                    println!("       - {action}");
                                }
                            }
                            
                            println!();
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Could not retrieve workflow history: {e}");
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Error retrieving workflow status: {e}");
            return Err(e);
        }
    }
    
    Ok(())
}

/// Handle pending approvals for user - Task 3.1.16
pub fn handle_pending_approvals(args: &[String]) -> Result<(), String> {
    let mut user_role = "quality_engineer".to_string();
    
    // Parse role if provided
    if args.len() >= 2 && args[0] == "--role" {
        user_role = args[1].clone();
    }
    
    // Initialize approval manager
    let approval_manager = RiskApprovalManager::new(Path::new("."));
    
    // Get pending approvals for user
    match approval_manager.get_pending_approvals_for_user(&user_role) {
        Ok(pending_risks) => {
            println!("Pending Approvals for {user_role} Role");
            println!("═══════════════════════════════════════════");
            
            if pending_risks.is_empty() {
                println!("✅ No pending approvals found for your role.");
                println!();
                println!("Available roles: quality_engineer, quality_manager, management, regulatory_affairs, cmo");
            } else {
                println!("Found {} risks pending approval:", pending_risks.len());
                println!();
                
                for (i, risk_id) in pending_risks.iter().enumerate() {
                    println!("{}. Risk ID: {}", i + 1, risk_id);
                    
                    // Get current state
                    if let Ok(state) = approval_manager.get_current_workflow_state(risk_id) {
                        println!("   Current State: {state:?}");
                    }
                    
                    println!("   Action: qms risk approve {risk_id} --signature \"Risk acceptable per ISO 14971\"");
                    println!();
                }
                
                println!("💡 Tips:");
                println!("  - Use 'qms risk workflow <risk_id>' to view detailed workflow");
                println!("  - Use 'qms risk approve <risk_id> --reject' to reject");
                println!("  - Use 'qms risk approve <risk_id> --conditions \"condition1,condition2\"' for conditional approval");
            }
        }
        Err(e) => {
            eprintln!("❌ Error retrieving pending approvals: {e}");
            return Err(e);
        }
    }
    
    Ok(())
}

/// Handle workflow report generation - Task 3.1.16
pub fn handle_workflow_report(args: &[String]) -> Result<(), String> {
    let mut save_to_file = false;
    
    // Parse arguments
    for arg in args {
        if arg == "--save" {
            save_to_file = true;
        }
    }
    
    // Initialize approval manager
    let approval_manager = RiskApprovalManager::new(Path::new("."));
    
    // Generate workflow report
    let report = approval_manager.generate_workflow_report();
    
    println!("{report}");
    
    if save_to_file {
        let filename = format!("risk_workflow_report_{}.md", 
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs());
        
        if let Err(e) = std::fs::write(&filename, &report) {
            eprintln!("Error saving report: {e}");
        } else {
            println!("📄 Report saved to: {filename}");
        }
    } else {
        println!("💡 Tip: Use --save flag to save report to file");
    }
    
    Ok(())
}