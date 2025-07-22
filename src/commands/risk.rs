//! Risk Management Commands
//! Task 3.1.1: Risk Item Schema & Storage Implementation
//! 
//! This module implements CLI commands for the comprehensive risk management
//! system required for medical device quality management per ISO 14971.

use crate::prelude::*;

// KISS: Simplified imports - grouped by functionality for better readability
// Core risk management types
use crate::modules::risk_manager::{
    RiskManager, RiskFilter, RiskRegisterFilter, RiskStatus,
    RiskSeverity, RiskOccurrence, RiskDetectability
};

// Risk assessment and scoring
use crate::modules::risk_manager::{
    RiskMatrix, RiskScoring, FMEAManager
};

// Reporting and analytics
use crate::modules::risk_manager::{
    RiskReporter, ReportType, ReportFormat, TimePeriod, RiskTrend,
    RiskMetricsManager, RiskKPIs, MetricsPeriod, RiskDashboard
};

// Compliance and validation
use crate::modules::risk_manager::{
    ISO14971Validator, RMFOptions, RMFFormat, ComplianceStatus
};

// Import/Export functionality
use crate::modules::risk_manager::{
    ImportFormat, ExportFormat, ImportOptions, ExportOptions,
    RiskImporter, RiskExporter
};

// Management and workflow
use crate::modules::risk_manager::{
    SurveillanceManager, DocumentationManager, TemplateType, OutputFormat, TemplateConfig,
    RiskCategory, ClassificationDimension, RiskCategorizationManager,
    RiskApprovalManager, RiskCommunicationManager, StakeholderType
};

// Approval workflow
use crate::modules::risk_manager::approval::ApprovalDecision;
use std::process;
use std::path::Path;

pub fn handle_risk_command(args: &[String]) -> Result<(), String> {
    if args.len() < 3 {
        print_risk_help();
        return Ok(());
    }

    match args[2].as_str() {
        "init" => handle_risk_init(&args[3..]),
        "create" => handle_risk_create(&args[3..]),
        "list" => handle_risk_list(&args[3..]),
        "view" => handle_risk_view(&args[3..]),
        "update" => handle_risk_update(&args[3..]),
        "assess" => handle_risk_assess(&args[3..]),
        "matrix" => handle_risk_matrix(&args[3..]),
        "position" => handle_risk_position(&args[3..]),
        "mitigate" => handle_risk_mitigate(&args[3..]),
        "list-mitigations" => handle_list_mitigations(&args[3..]),
        "verify-mitigation" => handle_verify_mitigation(&args[3..]),
        "verify" => handle_risk_verify(&args[3..]),              // Task 3.1.9: Risk Verification & Validation
        "residual" => handle_risk_residual(&args[3..]),
        "validate-reduction" => handle_validate_risk_reduction(&args[3..]),
        "approve-residual" => handle_approve_residual_risk(&args[3..]),
        "register" => handle_risk_register(&args[3..]),
        "stats" => handle_risk_stats(&args[3..]),
        "report" => handle_risk_report(&args[3..]),              // Task 3.1.10: Risk Reporting & Analytics
        "export-register" => handle_export_risk_register(&args[3..]),
        "status" => handle_risk_status_dashboard(&args[3..]),      // Task 3.1.8
        "overdue" => handle_risk_overdue(&args[3..]),            // Task 3.1.8
        "escalate" => handle_risk_escalate(&args[3..]),          // Task 3.1.8
        "update-status" => handle_risk_update_status(&args[3..]), // Task 3.1.8
        "fmea" => handle_risk_fmea(&args[3..]),
        "import" => handle_risk_import(&args[3..]),              // Task 3.1.12: Risk Import/Export
        "export" => handle_risk_export(&args[3..]),              // Task 3.1.12: Risk Import/Export
        "surveillance" => handle_risk_surveillance(&args[3..]),  // Task 3.1.13: Post-Market Surveillance
        "document" => handle_risk_document(&args[3..]),          // Task 3.1.14: Risk Documentation
        "trace-to-requirements" => handle_risk_traceability(&args[3..]), // Task 3.1.14: Traceability
        "categorize" => handle_risk_categorize(&args[3..]),      // Task 3.1.15: Risk Categories & Classification
        "filter-category" => handle_filter_by_category(&args[3..]), // Task 3.1.15: Filter by Category
        "category-stats" => handle_category_statistics(&args[3..]), // Task 3.1.15: Category Statistics
        "submit" => handle_risk_submit(&args[3..]),              // Task 3.1.16: Risk Review & Approval
        "approve" => handle_risk_approve(&args[3..]),            // Task 3.1.16: Risk Review & Approval
        "workflow" => handle_risk_workflow(&args[3..]),          // Task 3.1.16: Risk Review & Approval
        "pending-approvals" => handle_pending_approvals(&args[3..]), // Task 3.1.16: Risk Review & Approval
        "workflow-report" => handle_workflow_report(&args[3..]), // Task 3.1.16: Risk Review & Approval
        "iso14971-check" => handle_iso14971_check(&args[3..]),    // Task 3.1.11: ISO 14971 Compliance
        "generate-rmf" => handle_generate_rmf(&args[3..]),        // Task 3.1.11: Risk Management File
        "compliance-gaps" => handle_compliance_gaps(&args[3..]),  // Task 3.1.11: Gap Analysis
        "notify" => handle_risk_notify(&args[3..]),              // Task 3.1.17: Risk Communication
        "alerts" => handle_risk_alerts(&args[3..]),              // Task 3.1.17: Risk Communication
        "communications" => handle_risk_communications(&args[3..]), // Task 3.1.17: Risk Communication
        "acknowledge" => handle_risk_acknowledge(&args[3..]),     // Task 3.1.17: Risk Communication
        "executive-summary" => handle_executive_summary(&args[3..]), // Task 3.1.17: Risk Communication
        "technical-report" => handle_technical_report(&args[3..]), // Task 3.1.17: Risk Communication
        "metrics" => handle_risk_metrics(&args[3..]),            // Task 3.1.18: Risk Performance Metrics
        "kpis" => handle_risk_kpis(&args[3..]),                  // Task 3.1.18: Risk Performance Metrics
        "--help" | "-h" => {
            print_risk_help();
            Ok(())
        }
        _ => {
            eprintln!("Error: Unknown risk command '{}'", args[2]);
            print_risk_help();
            process::exit(1);
        }
    }
}

fn handle_risk_init(_args: &[String]) -> Result<(), String> {
    println!("🔧 Initializing risk management system...");
    
    // Get current project directory
    let project_path = get_current_project_path().map_err(|e| format!("Failed to get project path: {e}"))?;
    
    // Initialize risk manager
    let risk_manager = RiskManager::new(&project_path).map_err(|e| format!("Failed to create risk manager: {e}"))?;
    risk_manager.initialize().map_err(|e| format!("Failed to initialize risk system: {e}"))?;
    
    println!("✅ Risk management system initialized successfully!");
    println!("📁 Created directory structure:");
    println!("   - risks/");
    println!("   - risks/assessments/");
    println!("   - risks/mitigations/");
    println!("   - risks/reports/");
    println!("   - risks/evidence/");
    println!("   - risks/reviews/");
    println!("   - risks/index.json");
    
    Ok(())
}

fn handle_risk_create(args: &[String]) -> Result<(), String> {
    if args.len() < 6 {
        eprintln!("Error: Missing required arguments for risk creation");
        println!("\nUSAGE:");
        println!("    qms risk create --hazard <description> --situation <situation> --harm <harm>");
        println!("\nEXAMPLE:");
        println!("    qms risk create --hazard \"Software crash during operation\" --situation \"User enters invalid data\" --harm \"Loss of patient data\"");
        return Err("Missing arguments".to_string());
    }
    
    let mut hazard_desc = String::new();
    let mut situation = String::new(); 
    let mut harm = String::new();
    
    // Parse arguments
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--hazard" => {
                if i + 1 < args.len() {
                    hazard_desc = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --hazard".to_string());
                }
            }
            "--situation" => {
                if i + 1 < args.len() {
                    situation = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --situation".to_string());
                }
            }
            "--harm" => {
                if i + 1 < args.len() {
                    harm = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --harm".to_string());
                }
            }
            _ => i += 1,
        }
    }
    
    if hazard_desc.is_empty() || situation.is_empty() || harm.is_empty() {
        return Err("All arguments --hazard, --situation, and --harm are required".to_string());
    }
    
    println!("🔧 Creating new risk item...");
    
    // Get current project directory
    let project_path = get_current_project_path().map_err(|e| format!("Failed to get project path: {e}"))?;
    
    // Create risk manager and create risk
    let mut risk_manager = RiskManager::new(&project_path).map_err(|e| format!("Failed to create risk manager: {e}"))?;
    let risk = risk_manager.create_risk(&hazard_desc, &situation, &harm)
        .map_err(|e| format!("Failed to create risk: {e}"))?;
    
    println!("✅ Risk created successfully!");
    println!("📊 Risk Details:");
    println!("   ID: {}", risk.id);
    println!("   Hazard ID: {}", risk.hazard_id);
    println!("   Hazard: {}", risk.hazard_description);
    println!("   Situation: {}", risk.hazardous_situation);
    println!("   Harm: {}", risk.harm);
    println!("   Initial RPN: {}", risk.risk_priority_number);
    println!("   Risk Level: {:?}", risk.initial_risk_level);
    println!("   Created: {}", risk.created_at);
    
    Ok(())
}

fn handle_risk_list(args: &[String]) -> Result<(), String> {
    // Parse filtering options
    let mut filter = RiskFilter::default();
    let mut show_details = false;
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--severity" => {
                if i + 1 < args.len() {
                    filter.severity = parse_severity(&args[i + 1])?;
                    i += 2;
                } else {
                    return Err("Missing value for --severity".to_string());
                }
            }
            "--min-rpn" => {
                if i + 1 < args.len() {
                    filter.min_rpn = Some(args[i + 1].parse()
                        .map_err(|_| "Invalid RPN value".to_string())?);
                    i += 2;
                } else {
                    return Err("Missing value for --min-rpn".to_string());
                }
            }
            "--details" => {
                show_details = true;
                i += 1;
            }
            _ => i += 1,
        }
    }
    
    println!("📋 Listing project risks...");
    
    // Get current project directory
    let project_path = get_current_project_path().map_err(|e| format!("Failed to get project path: {e}"))?;
    
    // Create risk manager and list risks
    let risk_manager = RiskManager::new(&project_path).map_err(|e| format!("Failed to create risk manager: {e}"))?;
    let risks = risk_manager.list_risks(Some(&filter))
        .map_err(|e| format!("Failed to list risks: {e}"))?;
    
    if risks.is_empty() {
        println!("📭 No risks found matching the criteria.");
        return Ok(());
    }
    
    println!("📊 Found {} risk(s):\n", risks.len());
    
    for risk in &risks {
        if show_details {
            println!("🔍 Risk: {}", risk.hazard_id);
            println!("   ID: {}", risk.id);
            println!("   Description: {}", risk.description);
            println!("   Severity: {:?}", risk.severity);
            println!("   RPN: {}", risk.rpn);
            println!("   Risk Level: {:?}", risk.risk_level);
            println!("   Status: {:?}", risk.status);
            println!("   Created: {}", risk.created_at);
            println!("   Updated: {}", risk.updated_at);
            println!();
        } else {
            println!("{:<8} {:<12} {:<5} {:<15} {}", 
                risk.hazard_id, 
                format!("{:?}", risk.severity),
                risk.rpn,
                format!("{:?}", risk.risk_level),
                risk.description.chars().take(50).collect::<String>());
        }
    }
    
    if !show_details {
        println!("\n💡 Use --details flag for more information");
    }
    
    Ok(())
}

fn handle_risk_view(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Risk ID is required. Usage: qms risk view <risk-id>".to_string());
    }
    
    let risk_id = &args[0];
    
    println!("🔍 Viewing risk details...");
    
    // Get current project directory
    let project_path = get_current_project_path().map_err(|e| format!("Failed to get project path: {e}"))?;
    
    // Create risk manager and load risk
    let risk_manager = RiskManager::new(&project_path).map_err(|e| format!("Failed to create risk manager: {e}"))?;
    let risk = risk_manager.load_risk(risk_id)
        .map_err(|e| format!("Failed to load risk: {e}"))?;
    
    // Display comprehensive risk information
    println!("📊 Risk Details:");
    println!("════════════════════════════════════════");
    println!("ID: {}", risk.id);
    println!("Hazard ID: {}", risk.hazard_id);
    println!("Project: {}", risk.project_id);
    println!("\n🚨 Hazard Analysis:");
    println!("Description: {}", risk.hazard_description);
    println!("Hazardous Situation: {}", risk.hazardous_situation);
    println!("Potential Harm: {}", risk.harm);
    println!("\n📈 Initial Risk Assessment:");
    println!("Severity: {:?} ({})", risk.severity, risk.severity.clone() as u8);
    println!("Occurrence: {:?} ({})", risk.occurrence, risk.occurrence.clone() as u8);
    println!("Detectability: {:?} ({})", risk.detectability, risk.detectability.clone() as u8);
    println!("RPN: {}", risk.risk_priority_number);
    println!("Risk Level: {:?}", risk.initial_risk_level);
    println!("\n📉 Residual Risk (Post-Mitigation):");
    println!("Severity: {:?} ({})", risk.residual_severity, risk.residual_severity.clone() as u8);
    println!("Occurrence: {:?} ({})", risk.residual_occurrence, risk.residual_occurrence.clone() as u8);
    println!("Detectability: {:?} ({})", risk.residual_detectability, risk.residual_detectability.clone() as u8);
    println!("Residual RPN: {}", risk.residual_rpn);
    println!("Residual Risk Level: {:?}", risk.residual_risk_level);
    println!("\n🛡️ Mitigation Information:");
    println!("Measures: {} configured", risk.mitigation_measures.len());
    println!("Verification Method: {}", risk.verification_method);
    println!("Verification Status: {:?}", risk.verification_status);
    println!("\n📋 Metadata:");
    println!("Category: {}", risk.category);
    println!("Priority: {}", risk.priority);
    println!("Source: {}", risk.source);
    println!("Assigned To: {}", risk.assigned_to.as_deref().unwrap_or("Unassigned"));
    println!("Tags: {}", risk.tags.join(", "));
    println!("\n📅 Timeline:");
    println!("Created: {} by {}", risk.created_at, risk.created_by);
    println!("Updated: {}", risk.updated_at);
    if let Some(approved_by) = &risk.approved_by {
        println!("Approved: {} by {}", risk.approval_date.as_deref().unwrap_or("Unknown"), approved_by);
    }
    
    Ok(())
}

fn handle_risk_update(args: &[String]) -> Result<(), String> {
    println!("🔧 Risk update command - implementation pending");
    println!("Args: {args:?}");
    println!("📝 This will be implemented in task 3.1.2 (Risk Creation & Assessment)");
    Ok(())
}

fn handle_risk_assess(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        eprintln!("Error: Risk ID is required for assessment");
        println!("\nUSAGE:");
        println!("    qms risk assess <risk-id> [--severity N] [--occurrence N] [--detectability N]");
        println!("\nEXAMPLE:");
        println!("    qms risk assess HAZ-001 --severity 3 --occurrence 2 --detectability 2");
        println!("\nSEVERITY LEVELS:");
        println!("    5 = Catastrophic (Death or permanent disability)");
        println!("    4 = Critical (Serious injury requiring intervention)");
        println!("    3 = Major (Moderate injury requiring treatment)");
        println!("    2 = Minor (Minor injury, first aid)");
        println!("    1 = Negligible (No injury expected)");
        println!("\nOCCURRENCE LEVELS:");
        println!("    5 = Frequent (Very likely, >1 in 10)");
        println!("    4 = Probable (Will occur several times, 1 in 100 to 1 in 10)");
        println!("    3 = Occasional (Likely sometime, 1 in 1,000 to 1 in 100)");
        println!("    2 = Remote (Unlikely but possible, 1 in 10,000 to 1 in 1,000)");
        println!("    1 = Improbable (So unlikely, <1 in 10,000)");
        println!("\nDETECTABILITY LEVELS:");
        println!("    5 = Very Low (Cannot detect, no controls)");
        println!("    4 = Low (Poor chance of detection)");
        println!("    3 = Moderate (Moderate chance of detection)");
        println!("    2 = High (Good chance of detection)");
        println!("    1 = Very High (Almost certain detection)");
        return Err("Missing arguments".to_string());
    }
    
    let risk_id = &args[0];
    let mut severity: Option<RiskSeverity> = None;
    let mut occurrence: Option<RiskOccurrence> = None;
    let mut detectability: Option<RiskDetectability> = None;
    
    // Parse assessment parameters
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--severity" => {
                if i + 1 < args.len() {
                    severity = Some(parse_severity_value(&args[i + 1])?);
                    i += 2;
                } else {
                    return Err("Missing value for --severity".to_string());
                }
            }
            "--occurrence" => {
                if i + 1 < args.len() {
                    occurrence = Some(parse_occurrence_value(&args[i + 1])?);
                    i += 2;
                } else {
                    return Err("Missing value for --occurrence".to_string());
                }
            }
            "--detectability" => {
                if i + 1 < args.len() {
                    detectability = Some(parse_detectability_value(&args[i + 1])?);
                    i += 2;
                } else {
                    return Err("Missing value for --detectability".to_string());
                }
            }
            _ => i += 1,
        }
    }
    
    // Validate that at least one parameter is provided
    if severity.is_none() && occurrence.is_none() && detectability.is_none() {
        return Err("At least one assessment parameter (--severity, --occurrence, or --detectability) must be provided".to_string());
    }
    
    println!("📊 Assessing risk {risk_id}...");
    
    // Get current project directory
    let project_path = get_current_project_path().map_err(|e| format!("Failed to get project path: {e}"))?;
    
    // Create risk manager and assess risk
    let mut risk_manager = RiskManager::new(&project_path).map_err(|e| format!("Failed to create risk manager: {e}"))?;
    let updated_risk = risk_manager.assess_risk(risk_id, severity.clone(), occurrence.clone(), detectability.clone())
        .map_err(|e| format!("Failed to assess risk: {e}"))?;
    
    println!("✅ Risk assessment completed successfully!");
    println!("\n� Updated Risk Assessment:");
    println!("════════════════════════════════════════");
    println!("Risk ID: {}", updated_risk.hazard_id);
    println!("Description: {}", updated_risk.hazard_description);
    
    if severity.is_some() {
        println!("✓ Severity: {:?} ({})", updated_risk.severity, updated_risk.severity.clone() as u8);
    }
    if occurrence.is_some() {
        println!("✓ Occurrence: {:?} ({})", updated_risk.occurrence, updated_risk.occurrence.clone() as u8);
    }
    if detectability.is_some() {
        println!("✓ Detectability: {:?} ({})", updated_risk.detectability, updated_risk.detectability.clone() as u8);
    }
    
    println!("\n📈 Risk Calculation Results:");
    println!("RPN (Risk Priority Number): {}", updated_risk.risk_priority_number);
    println!("Risk Level: {:?}", updated_risk.initial_risk_level);
    
    // Show risk level interpretation
    match updated_risk.initial_risk_level {
        crate::modules::risk_manager::RiskLevel::Unacceptable => {
            println!("⚠️  UNACCEPTABLE RISK - Immediate action required!");
        }
        crate::modules::risk_manager::RiskLevel::ALARP => {
            println!("🔶 ALARP Risk - As Low As Reasonably Practicable, mitigation recommended");
        }
        crate::modules::risk_manager::RiskLevel::Acceptable => {
            println!("✅ Acceptable Risk - No immediate action required");
        }
    }
    
    println!("\nUpdated: {}", updated_risk.updated_at);
    
    Ok(())
}

// Helper functions to parse severity, occurrence, and detectability values
fn parse_severity_value(value: &str) -> Result<RiskSeverity, String> {
    match value {
        "5" | "catastrophic" => Ok(RiskSeverity::Catastrophic),
        "4" | "critical" => Ok(RiskSeverity::Critical),
        "3" | "major" => Ok(RiskSeverity::Major),
        "2" | "minor" => Ok(RiskSeverity::Minor),
        "1" | "negligible" => Ok(RiskSeverity::Negligible),
        _ => Err(format!("Invalid severity value: {value}. Must be 1-5 or text (catastrophic, critical, major, minor, negligible)")),
    }
}

fn parse_occurrence_value(value: &str) -> Result<RiskOccurrence, String> {
    match value {
        "5" | "frequent" => Ok(RiskOccurrence::Frequent),
        "4" | "probable" => Ok(RiskOccurrence::Probable),
        "3" | "occasional" => Ok(RiskOccurrence::Occasional),
        "2" | "remote" => Ok(RiskOccurrence::Remote),
        "1" | "improbable" => Ok(RiskOccurrence::Improbable),
        _ => Err(format!("Invalid occurrence value: {value}. Must be 1-5 or text (frequent, probable, occasional, remote, improbable)")),
    }
}

fn parse_detectability_value(value: &str) -> Result<RiskDetectability, String> {
    match value {
        "5" | "verylow" | "very-low" => Ok(RiskDetectability::VeryLow),
        "4" | "low" => Ok(RiskDetectability::Low),
        "3" | "moderate" => Ok(RiskDetectability::Moderate),
        "2" | "high" => Ok(RiskDetectability::High),
        "1" | "veryhigh" | "very-high" => Ok(RiskDetectability::VeryHigh),
        _ => Err(format!("Invalid detectability value: {value}. Must be 1-5 or text (verylow, low, moderate, high, veryhigh)")),
    }
}

fn handle_risk_mitigate(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        print_mitigate_help();
        return Ok(());
    }
    
    let mut risk_id: Option<String> = None;
    let mut measure: Option<String> = None;
    let mut effectiveness: Option<f32> = None;
    let mut implementation: Option<String> = None;
    let mut timeline: Option<String> = None;
    let mut verification_method: Option<String> = None;
    
    // Parse arguments
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--id" => {
                if i + 1 < args.len() {
                    risk_id = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --id".to_string());
                }
            }
            "--measure" => {
                if i + 1 < args.len() {
                    measure = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --measure".to_string());
                }
            }
            "--effectiveness" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<f32>() {
                        Ok(eff) => {
                            if (0.0..=1.0).contains(&eff) {
                                effectiveness = Some(eff);
                            } else {
                                return Err("Effectiveness must be between 0.0 and 1.0".to_string());
                            }
                        }
                        Err(_) => return Err("Invalid effectiveness value. Must be a number between 0.0 and 1.0".to_string()),
                    }
                    i += 2;
                } else {
                    return Err("Missing value for --effectiveness".to_string());
                }
            }
            "--implementation" => {
                if i + 1 < args.len() {
                    implementation = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --implementation".to_string());
                }
            }
            "--timeline" => {
                if i + 1 < args.len() {
                    timeline = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --timeline".to_string());
                }
            }
            "--verification" => {
                if i + 1 < args.len() {
                    verification_method = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --verification".to_string());
                }
            }
            "--help" | "-h" => {
                print_mitigate_help();
                return Ok(());
            }
            _ => {
                // First positional argument is risk ID
                if risk_id.is_none() {
                    risk_id = Some(args[i].clone());
                }
                i += 1;
            }
        }
    }
    
    // Validate required parameters
    let risk_id = risk_id.ok_or("Risk ID is required. Use --id <risk-id> or provide as first argument")?;
    let measure = measure.ok_or("Mitigation measure description is required. Use --measure <description>")?;
    let effectiveness = effectiveness.ok_or("Effectiveness is required. Use --effectiveness <0.0-1.0>")?;
    
    // Get current project path
    let project_path = crate::utils::get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;
    
    // Create risk manager and add mitigation
    let mut risk_manager = crate::modules::risk_manager::RiskManager::new(&project_path)
        .map_err(|e| format!("Failed to create risk manager: {e}"))?;
    
    match risk_manager.add_mitigation_measure(
        &risk_id,
        &measure,
        effectiveness,
        implementation.as_deref(),
        timeline.as_deref(),
        verification_method.as_deref()
    ) {
        Ok(mitigation) => {
            println!("✅ Mitigation measure added successfully");
            println!("🆔 Mitigation ID: {}", mitigation.id);
            println!("📝 Description: {}", mitigation.description);
            println!("� Effectiveness: {:.1}%", mitigation.effectiveness * 100.0);
            if let Some(impl_desc) = implementation {
                println!("🔧 Implementation: {impl_desc}");
            }
            if let Some(timeline_desc) = timeline {
                println!("📅 Timeline: {timeline_desc}");
            }
            if let Some(verify_method) = verification_method {
                println!("✓ Verification Method: {verify_method}");
            }
            println!("📋 Status: {}", mitigation.implementation_status);
            
            // Load updated risk to show residual risk
            match risk_manager.load_risk(&risk_id) {
                Ok(updated_risk) => {
                    println!("\n🎯 Updated Risk Assessment:");
                    println!("   Initial RPN: {} ({})", updated_risk.risk_priority_number, format_risk_level(&updated_risk.initial_risk_level));
                    println!("   Residual RPN: {} ({})", updated_risk.residual_rpn, format_risk_level(&updated_risk.residual_risk_level));
                    println!("   Risk Reduction: {:.1}%", 
                        (1.0 - (updated_risk.residual_rpn as f32 / updated_risk.risk_priority_number as f32)) * 100.0);
                }
                Err(e) => {
                    println!("⚠️ Warning: Could not load updated risk details: {e}");
                }
            }
        }
        Err(e) => {
            return Err(format!("Failed to add mitigation measure: {e}"));
        }
    }
    
    Ok(())
}

// ===============================================================
// Task 3.1.8: Risk Monitoring & Tracking Command Handlers
// ===============================================================

fn handle_risk_status_dashboard(args: &[String]) -> Result<(), String> {
    if !args.is_empty() && (args[0] == "--help" || args[0] == "-h") {
        print_risk_status_help();
        return Ok(());
    }

    // Get current project directory
    let project_path = get_current_project_path().map_err(|e| format!("Failed to get project path: {e}"))?;
    
    let risk_manager = RiskManager::new(&project_path).map_err(|e| format!("Failed to create risk manager: {e}"))?;
    risk_manager.display_risk_status_dashboard()
        .map_err(|e| format!("Failed to display risk status dashboard: {e}"))?;

    Ok(())
}

fn handle_risk_overdue(args: &[String]) -> Result<(), String> {
    if !args.is_empty() && (args[0] == "--help" || args[0] == "-h") {
        print_risk_overdue_help();
        return Ok(());
    }

    // Get current project directory
    let project_path = get_current_project_path().map_err(|e| format!("Failed to get project path: {e}"))?;
    
    let risk_manager = RiskManager::new(&project_path).map_err(|e| format!("Failed to create risk manager: {e}"))?;
    risk_manager.display_overdue_risks()
        .map_err(|e| format!("Failed to display overdue risks: {e}"))?;

    Ok(())
}

fn handle_risk_escalate(args: &[String]) -> Result<(), String> {
    if !args.is_empty() && (args[0] == "--help" || args[0] == "-h") {
        print_risk_escalate_help();
        return Ok(());
    }

    // Default RPN threshold of 50 for escalation
    let mut rpn_threshold = 50;

    // Parse arguments
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--threshold" => {
                if i + 1 < args.len() {
                    rpn_threshold = args[i + 1].parse()
                        .map_err(|_| "Invalid RPN threshold value".to_string())?;
                    i += 2;
                } else {
                    return Err("--threshold requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }

    // Get current project directory
    let project_path = get_current_project_path().map_err(|e| format!("Failed to get project path: {e}"))?;
    
    let risk_manager = RiskManager::new(&project_path).map_err(|e| format!("Failed to create risk manager: {e}"))?;
    risk_manager.display_escalation_alerts(rpn_threshold)
        .map_err(|e| format!("Failed to display escalation alerts: {e}"))?;

    Ok(())
}

fn handle_risk_update_status(args: &[String]) -> Result<(), String> {
    if args.len() < 2 || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_risk_update_status_help();
        return Ok(());
    }

    let risk_id = &args[0];
    let status_str = &args[1];

    let new_status = match status_str.to_lowercase().as_str() {
        "identified" => RiskStatus::Identified,
        "assessed" => RiskStatus::Assessed,
        "mitigated" => RiskStatus::Mitigated,
        "verified" => RiskStatus::Verified,
        "closed" => RiskStatus::Closed,
        _ => {
            return Err(format!("Invalid status '{status_str}'. Valid statuses: identified, assessed, mitigated, verified, closed"));
        }
    };

    // Get current project directory
    let project_path = get_current_project_path().map_err(|e| format!("Failed to get project path: {e}"))?;
    
    let mut risk_manager = RiskManager::new(&project_path).map_err(|e| format!("Failed to create risk manager: {e}"))?;
    risk_manager.update_risk_status(risk_id, new_status)
        .map_err(|e| format!("Failed to update risk status: {e}"))?;

    Ok(())
}

fn print_mitigate_help() {
    println!("🛡️ QMS Risk Mitigation Management");
    println!();
    println!("USAGE:");
    println!("    qms risk mitigate <risk-id> --measure <description> --effectiveness <0.0-1.0> [OPTIONS]");
    println!();
    println!("REQUIRED ARGUMENTS:");
    println!("    <risk-id>                    Risk ID to add mitigation to");
    println!("    --measure <description>      Description of the mitigation measure");
    println!("    --effectiveness <value>      Expected effectiveness (0.0-1.0, where 1.0 = 100% effective)");
    println!();
    println!("OPTIONAL ARGUMENTS:");
    println!("    --implementation <desc>      How the mitigation will be implemented");
    println!("    --timeline <timeline>        Implementation timeline (e.g., '2 weeks', 'Q1 2025')");
    println!("    --verification <method>      How effectiveness will be verified");
    println!("    --help, -h                   Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    # Add basic mitigation");
    println!("    qms risk mitigate HAZ-001 --measure \"Add input validation\" --effectiveness 0.8");
    println!();
    println!("    # Add mitigation with full details");
    println!("    qms risk mitigate HAZ-001 \\");
    println!("        --measure \"Implement user authentication\" \\");
    println!("        --effectiveness 0.9 \\");
    println!("        --implementation \"Add OAuth 2.0 with JWT tokens\" \\");
    println!("        --timeline \"3 weeks\" \\");
    println!("        --verification \"Penetration testing\"");
    println!();
    println!("EFFECTIVENESS GUIDELINES:");
    println!("    0.9-1.0    Very High (eliminates 90-100% of risk)");
    println!("    0.7-0.8    High (eliminates 70-80% of risk)");
    println!("    0.5-0.6    Moderate (eliminates 50-60% of risk)");
    println!("    0.2-0.4    Low (eliminates 20-40% of risk)");
    println!("    0.0-0.1    Minimal (eliminates 0-10% of risk)");
    println!();
    println!("NOTE: Mitigation effectiveness primarily reduces occurrence probability.");
    println!("      Residual risk will be automatically calculated and displayed.");
}

const fn format_risk_level(level: &crate::modules::risk_manager::RiskLevel) -> &'static str {
    match level {
        crate::modules::risk_manager::RiskLevel::Unacceptable => "Unacceptable",
        crate::modules::risk_manager::RiskLevel::ALARP => "ALARP", 
        crate::modules::risk_manager::RiskLevel::Acceptable => "Acceptable",
    }
}

fn handle_list_mitigations(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Risk ID is required. Usage: qms risk list-mitigations <risk-id>".to_string());
    }
    
    let risk_id = &args[0];
    
    // Get current project path
    let project_path = crate::utils::get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;
    
    // Create risk manager
    let risk_manager = crate::modules::risk_manager::RiskManager::new(&project_path)
        .map_err(|e| format!("Failed to create risk manager: {e}"))?;
    
    match risk_manager.list_mitigations(risk_id) {
        Ok(mitigations) => {
            if mitigations.is_empty() {
                println!("📋 No mitigation measures found for risk {risk_id}");
                println!("💡 Add mitigation with: qms risk mitigate {risk_id} --measure \"description\" --effectiveness 0.8");
            } else {
                println!("🛡️ Mitigation Measures for Risk {risk_id}");
                println!("═══════════════════════════════════════════════════════════════");
                
                for (i, mitigation) in mitigations.iter().enumerate() {
                    println!("{}. {} (ID: {})", i + 1, mitigation.description, &mitigation.id[..8]);
                    println!("   📊 Effectiveness: {:.1}%", mitigation.effectiveness * 100.0);
                    println!("   📋 Status: {}", mitigation.implementation_status);
                    
                    if !mitigation.implementation.is_empty() {
                        println!("   🔧 Implementation: {}", mitigation.implementation);
                    }
                    
                    if let Some(timeline) = &mitigation.timeline {
                        println!("   📅 Timeline: {timeline}");
                    }
                    
                    if !mitigation.verification_method.is_empty() {
                        println!("   ✓ Verification: {}", mitigation.verification_method);
                    }
                    
                    println!("   🔍 Verification Status: {:?}", mitigation.verification_status);
                    
                    if let Some(verified_date) = &mitigation.verified_date {
                        println!("   ✅ Verified: {verified_date}");
                    }
                    
                    if i < mitigations.len() - 1 {
                        println!();
                    }
                }
                
                println!("═══════════════════════════════════════════════════════════════");
                println!("Total mitigation measures: {}", mitigations.len());
            }
        }
        Err(e) => {
            return Err(format!("Failed to list mitigations: {e}"));
        }
    }
    
    Ok(())
}

fn handle_verify_mitigation(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        print_verify_mitigation_help();
        return Ok(());
    }
    
    let mut risk_id: Option<String> = None;
    let mut mitigation_id: Option<String> = None;
    let mut verification_method: Option<String> = None;
    let mut evidence: Option<String> = None;
    
    // Parse arguments
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--risk-id" => {
                if i + 1 < args.len() {
                    risk_id = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --risk-id".to_string());
                }
            }
            "--mitigation-id" => {
                if i + 1 < args.len() {
                    mitigation_id = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --mitigation-id".to_string());
                }
            }
            "--method" => {
                if i + 1 < args.len() {
                    verification_method = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --method".to_string());
                }
            }
            "--evidence" => {
                if i + 1 < args.len() {
                    evidence = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --evidence".to_string());
                }
            }
            "--help" | "-h" => {
                print_verify_mitigation_help();
                return Ok(());
            }
            _ => {
                // Positional arguments: risk_id mitigation_id
                if risk_id.is_none() {
                    risk_id = Some(args[i].clone());
                } else if mitigation_id.is_none() {
                    mitigation_id = Some(args[i].clone());
                }
                i += 1;
            }
        }
    }
    
    // Validate required parameters
    let risk_id = risk_id.ok_or("Risk ID is required")?;
    let mitigation_id = mitigation_id.ok_or("Mitigation ID is required")?;
    let verification_method = verification_method.unwrap_or_else(|| "Manual Verification".to_string());
    let evidence = evidence.unwrap_or_else(|| "Verification completed".to_string());
    
    // Get current project path
    let project_path = crate::utils::get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;
    
    // Create risk manager and verify mitigation
    let mut risk_manager = crate::modules::risk_manager::RiskManager::new(&project_path)
        .map_err(|e| format!("Failed to create risk manager: {e}"))?;
    
    match risk_manager.verify_mitigation(&risk_id, &mitigation_id, &verification_method, &evidence) {
        Ok(_) => {
            println!("✅ Mitigation verified successfully");
            println!("🆔 Risk ID: {risk_id}");
            println!("🆔 Mitigation ID: {mitigation_id}");
            println!("🔍 Verification Method: {verification_method}");
            println!("📋 Evidence: {evidence}");
            println!("📅 Verified Date: {}", crate::utils::current_timestamp_string());
        }
        Err(e) => {
            return Err(format!("Failed to verify mitigation: {e}"));
        }
    }
    
    Ok(())
}

fn print_verify_mitigation_help() {
    println!("✅ QMS Mitigation Verification");
    println!();
    println!("USAGE:");
    println!("    qms risk verify-mitigation <risk-id> <mitigation-id> [OPTIONS]");
    println!();
    println!("REQUIRED ARGUMENTS:");
    println!("    <risk-id>        Risk ID containing the mitigation");
    println!("    <mitigation-id>  Mitigation measure ID to verify");
    println!();
    println!("OPTIONAL ARGUMENTS:");
    println!("    --method <method>    Verification method used");
    println!("    --evidence <text>    Evidence of verification");
    println!("    --help, -h           Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    # Basic verification");
    println!("    qms risk verify-mitigation HAZ-001 abc12345");
    println!();
    println!("    # Verification with details");
    println!("    qms risk verify-mitigation HAZ-001 abc12345 \\");
    println!("        --method \"Penetration Testing\" \\");
    println!("        --evidence \"Security scan passed with 0 vulnerabilities\"");
}

/// Task 3.1.9: Risk Verification & Validation
/// Handle comprehensive risk verification command
fn handle_risk_verify(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        print_risk_verify_help();
        return Ok(());
    }
    
    let mut risk_id: Option<String> = None;
    let mut verification_method: Option<String> = None;
    let mut evidence: Option<String> = None;
    let mut effectiveness: Option<f32> = None;
    let mut validation_notes: Option<String> = None;
    let mut evidence_type: Option<String> = None;
    let mut evidence_reference: Option<String> = None;
    let mut description: Option<String> = None;
    
    // Parse arguments
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--method" => {
                if i + 1 < args.len() {
                    verification_method = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --method".to_string());
                }
            }
            "--evidence" => {
                if i + 1 < args.len() {
                    evidence = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --evidence".to_string());
                }
            }
            "--effectiveness" => {
                if i + 1 < args.len() {
                    effectiveness = Some(args[i + 1].parse::<f32>()
                        .map_err(|_| "Invalid effectiveness value. Must be between 0.0 and 1.0".to_string())?);
                    i += 2;
                } else {
                    return Err("Missing value for --effectiveness".to_string());
                }
            }
            "--notes" => {
                if i + 1 < args.len() {
                    validation_notes = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --notes".to_string());
                }
            }
            "--evidence-type" => {
                if i + 1 < args.len() {
                    evidence_type = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --evidence-type".to_string());
                }
            }
            "--evidence-ref" => {
                if i + 1 < args.len() {
                    evidence_reference = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --evidence-ref".to_string());
                }
            }
            "--description" => {
                if i + 1 < args.len() {
                    description = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --description".to_string());
                }
            }
            "--help" | "-h" => {
                print_risk_verify_help();
                return Ok(());
            }
            _ => {
                // Positional argument: risk_id
                if risk_id.is_none() {
                    risk_id = Some(args[i].clone());
                }
                i += 1;
            }
        }
    }
    
    // Validate required parameters
    let risk_id = risk_id.ok_or("Risk ID is required")?;
    
    // Get current project path
    let project_path = crate::utils::get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;
    
    // Create risk manager
    let mut risk_manager = crate::modules::risk_manager::RiskManager::new(&project_path)
        .map_err(|e| format!("Failed to create risk manager: {e}"))?;
    
    println!("🔍 Risk Verification & Validation");
    println!("Risk ID: {risk_id}");
    
    // Perform different verification actions based on provided parameters
    if let (Some(method), Some(evidence_text)) = (verification_method.clone(), evidence.clone()) {
        // Basic risk control verification
        risk_manager.verify_risk_control(&risk_id, &method, &evidence_text)
            .map_err(|e| format!("Failed to verify risk control: {e}"))?;
        
        println!("✅ Risk control verified successfully");
        println!("🔍 Method: {method}");
        println!("📋 Evidence: {evidence_text}");
    }
    
    if let (Some(eff), Some(notes)) = (effectiveness, validation_notes) {
        // Mitigation effectiveness validation
        if !(0.0..=1.0).contains(&eff) {
            return Err("Effectiveness must be between 0.0 and 1.0".to_string());
        }
        
        risk_manager.validate_mitigation_effectiveness(&risk_id, eff, &notes)
            .map_err(|e| format!("Failed to validate mitigation effectiveness: {e}"))?;
        
        println!("✅ Mitigation effectiveness validated");
        println!("📊 Effectiveness: {:.1}%", eff * 100.0);
        println!("📝 Notes: {notes}");
    }
    
    if let (Some(ev_type), Some(ev_ref), Some(desc)) = (evidence_type.clone(), evidence_reference, description) {
        // Track verification evidence
        risk_manager.track_verification_evidence(&risk_id, &ev_type, &ev_ref, &desc)
            .map_err(|e| format!("Failed to track verification evidence: {e}"))?;
        
        println!("✅ Verification evidence tracked");
        println!("📂 Type: {ev_type}");
        println!("🔗 Reference: {ev_ref}");
        println!("📝 Description: {desc}");
    }
    
    if verification_method.is_none() && evidence.is_none() && effectiveness.is_none() && evidence_type.is_none() {
        // No specific action requested, show verification status
        let risk = risk_manager.load_risk(&risk_id)
            .map_err(|e| format!("Failed to load risk: {e}"))?;
        
        println!("📊 Current Verification Status:");
        println!("   Method: {}", risk.verification_method);
        println!("   Status: {:?}", risk.verification_status);
        println!("   Evidence ({} items):", risk.verification_evidence.len());
        for (i, evidence) in risk.verification_evidence.iter().enumerate() {
            println!("     {}. {}", i + 1, evidence);
        }
        
        println!("\n💡 Available verification actions:");
        println!("   qms risk verify {risk_id} --method \"Testing\" --evidence \"TC-001 passed\"");
        println!("   qms risk verify {risk_id} --effectiveness 0.85 --notes \"85% effective\"");
        println!("   qms risk verify {risk_id} --evidence-type \"TestCase\" --evidence-ref \"TC-001\" --description \"Unit test passed\"");
    }
    
    Ok(())
}

fn print_risk_verify_help() {
    println!("🔍 QMS Risk Verification & Validation");
    println!();
    println!("USAGE:");
    println!("    qms risk verify <risk-id> [OPTIONS]");
    println!();
    println!("REQUIRED ARGUMENTS:");
    println!("    <risk-id>    Risk ID to verify");
    println!();
    println!("VERIFICATION OPTIONS:");
    println!("    --method <method>        Verification method (test, analysis, inspection, demonstration)");
    println!("    --evidence <text>        Evidence of verification");
    println!("    --effectiveness <0.0-1.0> Mitigation effectiveness (0.0-1.0)");
    println!("    --notes <text>           Validation notes");
    println!("    --evidence-type <type>   Evidence type (TestCase, Document, Analysis, etc.)");
    println!("    --evidence-ref <ref>     Evidence reference (TC-001, DOC-002, etc.)");
    println!("    --description <text>     Evidence description");
    println!("    --help, -h               Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    # Verify risk control with test evidence");
    println!("    qms risk verify HAZ-001 --method test --evidence TC-001");
    println!();
    println!("    # Validate mitigation effectiveness");
    println!("    qms risk verify HAZ-001 --effectiveness 0.85 --notes \"85% effective based on testing\"");
    println!();
    println!("    # Track verification evidence");
    println!("    qms risk verify HAZ-001 --evidence-type TestCase --evidence-ref TC-001 --description \"Unit test passed\"");
    println!();
    println!("    # Show current verification status");
    println!("    qms risk verify HAZ-001");
    println!();
    println!("VERIFICATION METHODS:");
    println!("    test           Execute test cases to verify controls");
    println!("    analysis       Mathematical or analytical verification");
    println!("    inspection     Visual or manual inspection");
    println!("    demonstration  Demonstrate functionality");
}

fn handle_risk_matrix(args: &[String]) -> Result<(), String> {
    let mut show_report = false;
    let mut export_format: Option<String> = None;
    let mut output_file: Option<String> = None;
    let mut show_stats = false;
    
    // Parse arguments
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--report" => {
                show_report = true;
                i += 1;
            }
            "--stats" => {
                show_stats = true;
                i += 1;
            }
            "--export" => {
                if i + 1 < args.len() {
                    export_format = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --export".to_string());
                }
            }
            "--output" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --output".to_string());
                }
            }
            "--help" | "-h" => {
                print_matrix_help();
                return Ok(());
            }
            _ => {
                eprintln!("Warning: Unknown matrix option '{}'", args[i]);
                i += 1;
            }
        }
    }
    
    println!("📊 Generating ISO 14971 Risk Assessment Matrix...");
    
    // Create risk matrix
    let matrix = RiskMatrix::new();
    
    // Handle export request
    if let Some(format) = export_format {
        if format.to_lowercase() == "csv" {
            let csv_content = matrix.generate_csv_export();
            
            if let Some(output_path) = output_file {
                std::fs::write(&output_path, &csv_content)
                    .map_err(|e| format!("Failed to write CSV file: {e}"))?;
                println!("✅ Risk matrix exported to: {output_path}");
            } else {
                println!("📋 Risk Matrix CSV Export:");
                println!("{csv_content}");
            }
            return Ok(());
        } else {
            return Err(format!("Unsupported export format: {format}. Supported: csv"));
        }
    }
    
    // Show detailed report or basic matrix
    if show_report {
        println!("{}", matrix.generate_detailed_report());
    } else {
        println!("{}", matrix.generate_ascii_matrix());
    }
    
    // Show statistics if requested
    if show_stats {
        let stats = matrix.calculate_statistics();
        println!("\n{stats}");
    }
    
    println!("\n💡 Matrix Commands:");
    println!("    qms risk matrix --report     # Detailed matrix report");
    println!("    qms risk matrix --stats      # Matrix statistics");
    println!("    qms risk matrix --export csv --output matrix.csv");
    println!("    qms risk position HAZ-001    # Show risk position on matrix");
    
    Ok(())
}

fn handle_risk_position(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        eprintln!("Error: Risk ID is required");
        println!("\nUSAGE:");
        println!("    qms risk position <risk-id>");
        println!("\nEXAMPLE:");
        println!("    qms risk position HAZ-001");
        return Err("Missing risk ID".to_string());
    }
    
    let risk_id = &args[0];
    
    println!("🎯 Finding risk position on matrix...");
    
    // Get current project directory
    let project_path = get_current_project_path().map_err(|e| format!("Failed to get project path: {e}"))?;
    
    // Load the risk
    let risk_manager = RiskManager::new(&project_path).map_err(|e| format!("Failed to create risk manager: {e}"))?;
    let risk = risk_manager.load_risk(risk_id)
        .map_err(|e| format!("Failed to load risk: {e}"))?;
    
    // Create risk matrix and find position
    let matrix = RiskMatrix::new();
    
    println!("📊 Risk: {} ({})", risk.hazard_id, risk.hazard_description);
    println!("────────────────────────────────────────────────────────");
    
    // Current risk position
    if let Some(position) = matrix.get_position(&risk.severity, &risk.occurrence) {
        println!("📍 Current Position on Matrix:");
        println!("   Severity: {:?} ({})", risk.severity, risk.severity.clone() as u8);
        println!("   Occurrence: {:?} ({})", risk.occurrence, risk.occurrence.clone() as u8);
        println!("   Detectability: {:?} ({})", risk.detectability, risk.detectability.clone() as u8);
        println!("   RPN: {}", risk.risk_priority_number);
        println!("   Risk Level: {:?} {}", risk.initial_risk_level, 
            match position.color_code {
                crate::modules::risk_manager::MatrixColor::Green => "🟢",
                crate::modules::risk_manager::MatrixColor::Yellow => "🟡",
                crate::modules::risk_manager::MatrixColor::Red => "🔴",
            });
        
        // Generate assessment summary
        let summary = RiskScoring::generate_assessment_summary(
            &risk.severity, 
            &risk.occurrence, 
            &risk.detectability
        );
        println!("\n{summary}");
        
        // Residual risk position (if different)
        if risk.residual_rpn != risk.risk_priority_number {
            println!("\n📉 Residual Risk Position (Post-Mitigation):");
            if let Some(residual_position) = matrix.get_position(&risk.residual_severity, &risk.residual_occurrence) {
                println!("   Residual Severity: {:?} ({})", risk.residual_severity, risk.residual_severity.clone() as u8);
                println!("   Residual Occurrence: {:?} ({})", risk.residual_occurrence, risk.residual_occurrence.clone() as u8);
                println!("   Residual Detectability: {:?} ({})", risk.residual_detectability, risk.residual_detectability.clone() as u8);
                println!("   Residual RPN: {}", risk.residual_rpn);
                println!("   Residual Risk Level: {:?} {}", risk.residual_risk_level,
                    match residual_position.color_code {
                        crate::modules::risk_manager::MatrixColor::Green => "🟢",
                        crate::modules::risk_manager::MatrixColor::Yellow => "🟡", 
                        crate::modules::risk_manager::MatrixColor::Red => "🔴",
                    });
                
                // Risk reduction calculation
                let reduction = RiskScoring::calculate_risk_reduction(
                    risk.risk_priority_number, 
                    risk.residual_rpn
                );
                println!("   Risk Reduction: {reduction:.1}%");
            }
        }
        
    } else {
        println!("⚠️  Could not determine matrix position for this risk");
    }
    
    println!("\n💡 Matrix Navigation:");
    println!("    qms risk matrix              # View full risk matrix");
    println!("    qms risk assess {risk_id}          # Update risk assessment");
    
    Ok(())
}

fn print_matrix_help() {
    println!("📊 Risk Matrix Commands - ISO 14971 Compliant\n");
    println!("USAGE:");
    println!("    qms risk matrix [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    (none)        Display ASCII risk matrix");
    println!("    --report      Generate detailed matrix report");
    println!("    --stats       Show matrix statistics");
    println!("    --export csv  Export matrix to CSV format");
    println!("    --output FILE Specify output file for export");
    println!("    --help        Show this help message\n");
    println!("EXAMPLES:");
    println!("    qms risk matrix");
    println!("    qms risk matrix --report");
    println!("    qms risk matrix --stats");
    println!("    qms risk matrix --export csv --output risk_matrix.csv\n");
    println!("MATRIX FEATURES:");
    println!("• 5×5 Severity × Occurrence grid");
    println!("• Color-coded risk levels (🟢🟡🔴)");
    println!("• ISO 14971 compliant thresholds");
    println!("• RPN calculations with detectability factor");
    println!("• Medical device context explanations");
}

fn handle_risk_fmea(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        print_fmea_help();
        return Ok(());
    }
    
    match args[0].as_str() {
        "create" => handle_fmea_create(&args[1..]),
        "list" => handle_fmea_list(&args[1..]),
        "view" => handle_fmea_view(&args[1..]),
        "add-failure" => handle_fmea_add_failure(&args[1..]),
        "export" => handle_fmea_export(&args[1..]),
        "table" => handle_fmea_table(&args[1..]),
        "--help" | "-h" => {
            print_fmea_help();
            Ok(())
        }
        _ => {
            eprintln!("Error: Unknown FMEA command '{}'", args[0]);
            print_fmea_help();
            Err("Unknown FMEA command".to_string())
        }
    }
}

fn handle_fmea_create(args: &[String]) -> Result<(), String> {
    if args.len() < 6 {
        eprintln!("Error: Missing required arguments for FMEA creation");
        println!("\nUSAGE:");
        println!("    qms risk fmea create --component <component> --function <function> --name <name>");
        println!("\nEXAMPLE:");
        println!("    qms risk fmea create --component \"User Interface\" --function \"Data Entry\" --name \"UI Data Entry FMEA\"");
        return Err("Missing arguments".to_string());
    }
    
    let mut component = String::new();
    let mut function = String::new();
    let mut name = String::new();
    
    // Parse arguments
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--component" => {
                if i + 1 < args.len() {
                    component = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --component".to_string());
                }
            }
            "--function" => {
                if i + 1 < args.len() {
                    function = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --function".to_string());
                }
            }
            "--name" => {
                if i + 1 < args.len() {
                    name = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --name".to_string());
                }
            }
            _ => i += 1,
        }
    }
    
    if component.is_empty() || function.is_empty() || name.is_empty() {
        return Err("All arguments --component, --function, and --name are required".to_string());
    }
    
    println!("🔬 Creating FMEA analysis...");
    
    // Get current project directory
    let project_path = get_current_project_path().map_err(|e| format!("Failed to get project path: {e}"))?;
    
    // Create FMEA manager and create analysis
    let fmea_manager = FMEAManager::new(&project_path).map_err(|e| format!("Failed to create FMEA manager: {e}"))?;
    
    // Initialize FMEA system if needed
    fmea_manager.initialize().map_err(|e| format!("Failed to initialize FMEA system: {e}"))?;
    
    let analysis = fmea_manager.create_fmea_analysis(&component, &function, &name)
        .map_err(|e| format!("Failed to create FMEA analysis: {e}"))?;
    
    println!("✅ FMEA analysis created successfully!");
    println!("📊 FMEA Details:");
    println!("   ID: {}", analysis.id);
    println!("   Name: {}", analysis.name);
    println!("   Component: {}", analysis.component);
    println!("   Function: {}", analysis.function);
    println!("   Status: {:?}", analysis.status);
    println!("   Created: {}", analysis.created_at);
    println!("   Scope: {}", analysis.scope);
    println!("\n💡 Next steps:");
    println!("   1. Add failure modes: qms risk fmea add-failure {} --mode \"description\"", analysis.id);
    println!("   2. View analysis: qms risk fmea view {}", analysis.id);
    println!("   3. Generate table: qms risk fmea table {}", analysis.id);
    
    Ok(())
}

fn handle_fmea_list(_args: &[String]) -> Result<(), String> {
    println!("📋 Listing FMEA analyses...");
    
    // Get current project directory
    let project_path = get_current_project_path().map_err(|e| format!("Failed to get project path: {e}"))?;
    
    // Create FMEA manager and list analyses
    let fmea_manager = FMEAManager::new(&project_path).map_err(|e| format!("Failed to create FMEA manager: {e}"))?;
    let analyses = fmea_manager.list_fmea_analyses()
        .map_err(|e| format!("Failed to list FMEA analyses: {e}"))?;
    
    if analyses.is_empty() {
        println!("📭 No FMEA analyses found.");
        println!("\n💡 Create your first FMEA analysis:");
        println!("    qms risk fmea create --component \"System\" --function \"Primary Function\" --name \"Analysis Name\"");
        return Ok(());
    }
    
    println!("📊 Found {} FMEA analysis(es):\n", analyses.len());
    
    println!("{:<8} {:<25} {:<20} {:<15} {:<10} {:<12}", 
        "ID", "Name", "Component", "Function", "Status", "Failure Modes");
    println!("{}", "─".repeat(100));
    
    for analysis in &analyses {
        println!("{:<8} {:<25} {:<20} {:<15} {:<10} {:<12}", 
            analysis.id.chars().take(8).collect::<String>(),
            analysis.name.chars().take(25).collect::<String>(),
            analysis.component.chars().take(20).collect::<String>(),
            analysis.function.chars().take(15).collect::<String>(),
            format!("{:?}", analysis.status),
            analysis.failure_modes.len());
    }
    
    println!("\n💡 Use 'qms risk fmea view <id>' for detailed information");
    
    Ok(())
}

fn handle_fmea_view(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("FMEA ID is required. Usage: qms risk fmea view <fmea-id>".to_string());
    }
    
    let fmea_id = &args[0];
    
    println!("🔍 Viewing FMEA analysis details...");
    
    // Get current project directory
    let project_path = get_current_project_path().map_err(|e| format!("Failed to get project path: {e}"))?;
    
    // Create FMEA manager and load analysis
    let fmea_manager = FMEAManager::new(&project_path).map_err(|e| format!("Failed to create FMEA manager: {e}"))?;
    let analysis = fmea_manager.load_fmea_analysis(fmea_id)
        .map_err(|e| format!("Failed to load FMEA analysis: {e}"))?;
    
    // Display comprehensive FMEA information
    println!("📊 FMEA Analysis Details:");
    println!("════════════════════════════════════════");
    println!("ID: {}", analysis.id);
    println!("Name: {}", analysis.name);
    println!("Component: {}", analysis.component);
    println!("Function: {}", analysis.function);
    println!("Status: {:?}", analysis.status);
    println!("Description: {}", analysis.description);
    println!("Scope: {}", analysis.scope);
    println!("Version: {}", analysis.version);
    println!("Created: {} by {}", analysis.created_at, analysis.created_by);
    println!("Updated: {}", analysis.updated_at);
    
    if analysis.failure_modes.is_empty() {
        println!("\n🔍 Failure Modes: None");
        println!("\n💡 Add failure modes:");
        println!("    qms risk fmea add-failure {} --mode \"Failure description\" --function \"Function that fails\"", analysis.id);
    } else {
        println!("\n🔍 Failure Modes ({}):", analysis.failure_modes.len());
        println!("{}", "─".repeat(80));
        
        for (idx, fm) in analysis.failure_modes.iter().enumerate() {
            println!("{}. {} ({})", idx + 1, fm.mode_id, fm.description);
            println!("   Function: {}", fm.function);
            println!("   RPN: {} (S:{} × O:{} × D:{})", 
                fm.rpn, 
                fm.severity.clone() as u8,
                fm.occurrence.clone() as u8,
                fm.detectability.clone() as u8);
            println!("   Criticality: {}", fm.criticality);
            println!("   Status: {:?}", fm.status);
            println!();
        }
    }
    
    println!("📋 Team Members: {}", 
        if analysis.team_members.is_empty() { 
            "None assigned".to_string() 
        } else { 
            analysis.team_members.join(", ") 
        });
    
    println!("📝 Assumptions: {}", 
        if analysis.assumptions.is_empty() { 
            "None documented".to_string() 
        } else { 
            analysis.assumptions.join("; ") 
        });
    
    Ok(())
}

fn handle_fmea_add_failure(args: &[String]) -> Result<(), String> {
    if args.len() < 5 {
        eprintln!("Error: Missing required arguments for failure mode addition");
        println!("\nUSAGE:");
        println!("    qms risk fmea add-failure <fmea-id> --mode <description> --function <function>");
        println!("\nEXAMPLE:");
        println!("    qms risk fmea add-failure abc123 --mode \"Input validation failure\" --function \"Data validation\"");
        return Err("Missing arguments".to_string());
    }
    
    let fmea_id = &args[0];
    let mut mode_description = String::new();
    let mut function = String::new();
    
    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--mode" => {
                if i + 1 < args.len() {
                    mode_description = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --mode".to_string());
                }
            }
            "--function" => {
                if i + 1 < args.len() {
                    function = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --function".to_string());
                }
            }
            _ => i += 1,
        }
    }
    
    if mode_description.is_empty() || function.is_empty() {
        return Err("Both --mode and --function arguments are required".to_string());
    }
    
    println!("⚠️ Adding failure mode to FMEA analysis...");
    
    // Get current project directory
    let project_path = get_current_project_path().map_err(|e| format!("Failed to get project path: {e}"))?;
    
    // Create FMEA manager and add failure mode
    let fmea_manager = FMEAManager::new(&project_path).map_err(|e| format!("Failed to create FMEA manager: {e}"))?;
    let failure_mode = fmea_manager.add_failure_mode(fmea_id, &mode_description, &function)
        .map_err(|e| format!("Failed to add failure mode: {e}"))?;
    
    println!("✅ Failure mode added successfully!");
    println!("📊 Failure Mode Details:");
    println!("   ID: {}", failure_mode.id);
    println!("   Mode ID: {}", failure_mode.mode_id);
    println!("   Description: {}", failure_mode.description);
    println!("   Function: {}", failure_mode.function);
    println!("   Initial RPN: {}", failure_mode.rpn);
    println!("   Criticality: {}", failure_mode.criticality);
    println!("   Status: {:?}", failure_mode.status);
    println!("   Created: {}", failure_mode.created_at);
    
    println!("\n💡 Next steps:");
    println!("   1. Assess risk parameters for this failure mode");
    println!("   2. Add effects and causes");
    println!("   3. Define current controls");
    println!("   4. Generate FMEA table: qms risk fmea table {fmea_id}");
    
    Ok(())
}

fn handle_fmea_table(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("FMEA ID is required. Usage: qms risk fmea table <fmea-id> [--output file.csv]".to_string());
    }
    
    let fmea_id = &args[0];
    let mut output_file: Option<String> = None;
    
    // Parse optional output file
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --output".to_string());
                }
            }
            _ => i += 1,
        }
    }
    
    println!("📊 Generating FMEA table...");
    
    // Get current project directory
    let project_path = get_current_project_path().map_err(|e| format!("Failed to get project path: {e}"))?;
    
    // Create FMEA manager and generate table
    let fmea_manager = FMEAManager::new(&project_path).map_err(|e| format!("Failed to create FMEA manager: {e}"))?;
    let csv_table = fmea_manager.generate_fmea_table(fmea_id)
        .map_err(|e| format!("Failed to generate FMEA table: {e}"))?;
    
    if let Some(output_path) = output_file {
        // Save to file
        std::fs::write(&output_path, &csv_table)
            .map_err(|e| format!("Failed to write table to file: {e}"))?;
        
        println!("✅ FMEA table exported to: {output_path}");
    } else {
        // Display in console
        println!("📋 FMEA Table (CSV format):");
        println!("{}", "─".repeat(100));
        println!("{csv_table}");
    }
    
    Ok(())
}

fn handle_fmea_export(args: &[String]) -> Result<(), String> {
    if args.len() < 4 {
        eprintln!("Error: Missing required arguments for FMEA export");
        println!("\nUSAGE:");
        println!("    qms risk fmea export <fmea-id> --format <csv|json> --output <filename>");
        println!("\nEXAMPLE:");
        println!("    qms risk fmea export abc123 --format csv --output fmea_analysis.csv");
        return Err("Missing arguments".to_string());
    }
    
    let fmea_id = &args[0];
    let mut format = String::new();
    let mut output_file = String::new();
    
    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--format" => {
                if i + 1 < args.len() {
                    format = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --format".to_string());
                }
            }
            "--output" => {
                if i + 1 < args.len() {
                    output_file = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --output".to_string());
                }
            }
            _ => i += 1,
        }
    }
    
    if format.is_empty() || output_file.is_empty() {
        return Err("Both --format and --output arguments are required".to_string());
    }
    
    if !matches!(format.to_lowercase().as_str(), "csv" | "json") {
        return Err("Format must be 'csv' or 'json'".to_string());
    }
    
    println!("📤 Exporting FMEA analysis in {format} format...");
    
    // Get current project directory
    let project_path = get_current_project_path().map_err(|e| format!("Failed to get project path: {e}"))?;
    
    // Create FMEA manager and export
    let fmea_manager = FMEAManager::new(&project_path).map_err(|e| format!("Failed to create FMEA manager: {e}"))?;
    let output_path = Path::new(&output_file);
    
    fmea_manager.export_fmea_table(fmea_id, &format, output_path)
        .map_err(|e| format!("Failed to export FMEA: {e}"))?;
    
    println!("✅ FMEA analysis exported successfully!");
    println!("📁 File: {output_file}");
    println!("📊 Format: {}", format.to_uppercase());
    
    Ok(())
}

fn print_fmea_help() {
    println!("🔬 FMEA (Failure Mode & Effects Analysis) Commands\n");
    println!("USAGE:");
    println!("    qms risk fmea <COMMAND>\n");
    println!("COMMANDS:");
    println!("    create        Create new FMEA analysis");
    println!("    list          List all FMEA analyses");
    println!("    view          View detailed FMEA analysis");
    println!("    add-failure   Add failure mode to analysis");
    println!("    table         Generate FMEA table (CSV)");
    println!("    export        Export FMEA to file");
    println!("    help          Show this help message\n");
    println!("EXAMPLES:");
    println!("    qms risk fmea create --component \"User Interface\" --function \"Data Entry\" --name \"UI FMEA\"");
    println!("    qms risk fmea list");
    println!("    qms risk fmea view abc123");
    println!("    qms risk fmea add-failure abc123 --mode \"Input validation failure\" --function \"Data validation\"");
    println!("    qms risk fmea table abc123 --output fmea_table.csv");
    println!("    qms risk fmea export abc123 --format csv --output analysis.csv");
    println!("\nFor more information on a specific command, use:");
    println!("    qms risk fmea <COMMAND> --help");
}

fn parse_severity(value: &str) -> Result<Option<RiskSeverity>, String> {
    match value.to_lowercase().as_str() {
        "catastrophic" | "5" => Ok(Some(RiskSeverity::Catastrophic)),
        "critical" | "4" => Ok(Some(RiskSeverity::Critical)),
        "major" | "3" => Ok(Some(RiskSeverity::Major)),
        "minor" | "2" => Ok(Some(RiskSeverity::Minor)),
        "negligible" | "1" => Ok(Some(RiskSeverity::Negligible)),
        _ => Err(format!("Invalid severity value: {value}")),
    }
}

// Helper function to get current project path
fn get_current_project_path() -> QmsResult<std::path::PathBuf> {
    // Try to find project.json in current directory or parent directories
    let current_dir = std::env::current_dir()?;
    let mut path = current_dir.as_path();
    
    loop {
        let project_file = path.join("project.json");
        if project_file.exists() {
            return Ok(path.to_path_buf());
        }
        
        match path.parent() {
            Some(parent) => path = parent,
            None => break,
        }
    }
    
    // Fallback to default project directory
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| QmsError::io_error("Cannot determine home directory"))?;
    
    Ok(Path::new(&home).join(".qms").join("projects").join("default"))
}

/// Handle residual risk analysis command
/// Task 3.1.6: Residual Risk Analysis
fn handle_risk_residual(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        println!("Error: Risk ID required");
        println!("Usage: qms risk residual <RISK_ID> [--justify \"justification text\"]");
        return Err("Missing risk ID".to_string());
    }
    
    let risk_id = &args[0];
    
    // Check for justification flag
    if args.len() >= 2 && args[1] == "--justify" {
        if args.len() < 3 {
            println!("Error: Justification text required after --justify flag");
            return Err("Missing justification".to_string());
        }
        
        let justification = &args[2];
        
        // Get current project path and update with justification
        let project_path = get_current_project_path().map_err(|e| e.to_string())?;
        let mut risk_manager = RiskManager::new(&project_path).map_err(|e| e.to_string())?;
        
        match risk_manager.justify_residual_risk(risk_id, justification) {
            Ok(risk) => {
                println!("✅ Residual risk justification added for {risk_id}");
                println!("Justification: {justification}");
                
                if !risk.residual_risk_approved {
                    println!("\n⚠️  Note: This residual risk requires approval from a Quality Engineer");
                    println!("Use: qms risk approve-residual {risk_id} --approver \"Quality Engineer Name\"");
                }
            },
            Err(e) => {
                println!("❌ Failed to justify residual risk: {e}");
                return Err(e.to_string());
            }
        }
    } else {
        // Show residual risk analysis without justification
        let project_path = get_current_project_path().map_err(|e| e.to_string())?;
        let risk_manager = RiskManager::new(&project_path).map_err(|e| e.to_string())?;
        
        match risk_manager.get_residual_risk_analysis(risk_id) {
            Ok(analysis) => println!("{analysis}"),
            Err(e) => {
                println!("❌ Failed to get residual risk analysis: {e}");
                return Err(e.to_string());
            }
        }
    }
    
    Ok(())
}

/// Handle validate risk reduction command  
/// Task 3.1.6: Residual Risk Analysis
fn handle_validate_risk_reduction(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        println!("Error: Risk ID required");
        println!("Usage: qms risk validate-reduction <RISK_ID>");
        return Err("Missing risk ID".to_string());
    }
    
    let risk_id = &args[0];
    let project_path = get_current_project_path().map_err(|e| e.to_string())?;
    let risk_manager = RiskManager::new(&project_path).map_err(|e| e.to_string())?;
    
    match risk_manager.validate_risk_reduction(risk_id) {
        Ok(is_valid) => {
            if is_valid {
                println!("✅ Risk reduction validation PASSED for {risk_id}");
                println!("   Residual risk ≤ initial risk (ISO 14971 compliant)");
            } else {
                println!("❌ Risk reduction validation FAILED for {risk_id}");
                println!("   Residual risk > initial risk - requires additional mitigation or justification");
            }
        },
        Err(e) => {
            println!("❌ Failed to validate risk reduction: {e}");
            return Err(e.to_string());
        }
    }
    
    Ok(())
}

/// Handle approve residual risk command
/// Task 3.1.6: Residual Risk Analysis  
fn handle_approve_residual_risk(args: &[String]) -> Result<(), String> {
    if args.len() < 3 || args[1] != "--approver" {
        println!("Error: Risk ID and approver required");
        println!("Usage: qms risk approve-residual <RISK_ID> --approver \"Quality Engineer Name\"");
        return Err("Missing risk ID or approver".to_string());
    }
    
    let risk_id = &args[0];
    let approver = &args[2];
    
    let project_path = get_current_project_path().map_err(|e| e.to_string())?;
    let mut risk_manager = RiskManager::new(&project_path).map_err(|e| e.to_string())?;
    
    match risk_manager.approve_residual_risk(risk_id, approver) {
        Ok(_) => {
            println!("✅ Residual risk approved for {risk_id} by {approver}");
            println!("   Risk can now proceed to final verification");
        },
        Err(e) => {
            println!("❌ Failed to approve residual risk: {e}");
            return Err(e.to_string());
        }
    }
    
    Ok(())
}

fn print_risk_help() {
    println!("🚨 Manage QMS risk analysis (ISO 14971 compliant)\n");
    println!("USAGE:");
    println!("    qms risk <COMMAND>\n");
    println!("COMMANDS:");
    println!("    init              Initialize risk management system");
    println!("    create            Create a new risk item");
    println!("    list              List all project risks");
    println!("    view              View detailed risk information");
    println!("    update            Update risk information");
    println!("    assess            Perform risk assessment");
    println!("    matrix            Display ISO 14971 risk matrix");
    println!("    position          Show risk position on matrix");
    println!("    mitigate          Add risk mitigation measures");
    println!("    list-mitigations  List mitigation measures for a risk");
    println!("    verify-mitigation Verify mitigation effectiveness");
    println!("    verify            Comprehensive risk verification & validation");
    println!("    residual          View/justify residual risk analysis");
    println!("    validate-reduction Validate that residual risk ≤ initial risk");
    println!("    approve-residual  Approve residual risk (Quality Engineer)");
    println!("    register          View comprehensive risk register");
    println!("    stats             Show risk register statistics");
    println!("    export-register   Export risk register to file");
    println!("    report            Generate risk reports and analytics");
    println!("    status            Show risk status dashboard");
    println!("    overdue           Show overdue risks");
    println!("    escalate          Show risks requiring escalation");
    println!("    update-status     Update risk lifecycle status");
    println!("    fmea              FMEA (Failure Mode & Effects Analysis)");
    println!("    import            Import risk data from files");
    println!("    export            Export risk data to files");
    println!("    iso14971-check    Validate ISO 14971 compliance");
    println!("    generate-rmf      Generate Risk Management File (RMF)");
    println!("    compliance-gaps   Analyze compliance gaps");
    println!("    categorize        Categorize risks by safety, security, performance, etc.");
    println!("    filter-category   Filter risks by category");
    println!("    category-stats    Show risk category statistics");
    println!("    submit            Submit risk for approval review");
    println!("    approve           Approve risk with electronic signature");
    println!("    workflow          Show risk workflow status and history");
    println!("    pending-approvals Show pending approvals for user role");
    println!("    workflow-report   Generate risk workflow status report");
    println!("    notify            Send risk notifications to stakeholders");
    println!("    alerts            Generate automated risk alerts");
    println!("    communications    List risk communications");
    println!("    acknowledge       Acknowledge risk communication");
    println!("    executive-summary Generate executive risk summary");
    println!("    technical-report  Generate technical risk details");
    println!("    metrics           Generate risk performance metrics");
    println!("    kpis              Display key performance indicators");
    println!("    help              Show this help message\n");
    println!("EXAMPLES:");
    println!("    qms risk init");
    println!("    qms risk create --hazard \"Software crash\" --situation \"Invalid input\" --harm \"Data loss\"");
    println!("    qms risk list --severity critical --min-rpn 50");
    println!("    qms risk view HAZ-001");
    println!("    qms risk assess HAZ-001 --severity 4 --occurrence 3 --detectability 2");
    println!("    qms risk matrix --report");
    println!("    qms risk position HAZ-001");
    println!("    qms risk mitigate HAZ-001 --measure \"Add input validation\" --effectiveness 0.8");
    println!("    qms risk list-mitigations HAZ-001");
    println!("    qms risk verify-mitigation HAZ-001 abc12345 --method \"Testing\"");
    println!("    qms risk verify HAZ-001 --method test --evidence TC-001");
    println!("    qms risk residual HAZ-001");
    println!("    qms risk residual HAZ-001 --justify \"Risk acceptable for device class II\"");
    println!("    qms risk validate-reduction HAZ-001");
    println!("    qms risk approve-residual HAZ-001 --approver \"Jane Smith, QE\"");
    println!("    qms risk fmea create --component \"UI\" --function \"Data Entry\" --name \"UI FMEA\"");
    println!("    qms risk import --file risks.csv --format csv");
    println!("    qms risk export --format pdf --output risk_report.pdf --summary-only");
    println!("    qms risk register --filter status:open --sort rpn:desc");
    println!("    qms risk stats");
    println!("    qms risk export-register --format csv --output risks.csv");
    println!("    qms risk report --type summary --format pdf --output summary.pdf");
    println!("    qms risk status");
    println!("    qms risk overdue");
    println!("    qms risk escalate --threshold 75");
    println!("    qms risk update-status HAZ-001 assessed");
    println!("    qms risk iso14971-check --detailed");
    println!("    qms risk generate-rmf --format pdf --output device_rmf.pdf");
    println!("    qms risk compliance-gaps --output gaps.md");
    println!("    qms risk categorize HAZ-001 --category safety --subcategory electrical");
    println!("    qms risk filter-category --category safety --classification high");
    println!("    qms risk category-stats --report");
    println!("    qms risk submit HAZ-001 --comment \"Risk assessment complete\"");
    println!("    qms risk approve HAZ-001 --signature \"Risk acceptable per ISO 14971\" --role quality_engineer");
    println!("    qms risk approve HAZ-001 --reject --rationale \"Insufficient mitigation\"");
    println!("    qms risk approve HAZ-001 --conditions \"Additional testing required\"");
    println!("    qms risk workflow HAZ-001");
    println!("    qms risk pending-approvals --role quality_manager");
    println!("    qms risk workflow-report --save");
    println!("    qms risk notify HAZ-001 --stakeholders qe,pm --message \"High risk identified\"");
    println!("    qms risk alerts");
    println!("    qms risk communications --stakeholder qe");
    println!("    qms risk acknowledge COMM-001");
    println!("    qms risk executive-summary --output executive_summary.md");
    println!("    qms risk technical-report --output technical_report.md");
    println!("    qms risk metrics --period 30d --format csv --output metrics.csv");
    println!("    qms risk kpis --detailed --period 3m");
    println!("\nFor more information on a specific command, use:");
    println!("    qms risk <COMMAND> --help");
}

/// Handle risk register command
/// Task 3.1.7: Risk Register
fn handle_risk_register(args: &[String]) -> Result<(), String> {
    let mut filter = RiskRegisterFilter {
        sort_by: "rpn:desc".to_string(), // Default sort by RPN descending
        ..Default::default()
    };
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--filter" => {
                if i + 1 < args.len() {
                    let filter_value = &args[i + 1];
                    if let Some((key, value)) = filter_value.split_once(':') {
                        match key {
                            "status" => filter.status = Some(value.to_string()),
                            "severity" => {
                                filter.severity = parse_severity_arg(value);
                            },
                            "assignee" => filter.assignee = Some(value.to_string()),
                            "mitigation-status" => filter.mitigation_status = Some(value.to_string()),
                            _ => {
                                return Err(format!("Unknown filter key: {key}"));
                            }
                        }
                    }
                    i += 2;
                } else {
                    return Err("Missing value for --filter".to_string());
                }
            }
            "--sort" => {
                if i + 1 < args.len() {
                    filter.sort_by = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --sort".to_string());
                }
            }
            "--help" | "-h" => {
                print_register_help();
                return Ok(());
            }
            _ => {
                i += 1;
            }
        }
    }
    
    let project_path = crate::utils::get_current_project_path().map_err(|e| e.to_string())?;
    let risk_manager = RiskManager::new(&project_path).map_err(|e| e.to_string())?;
    
    match risk_manager.get_risk_register(&filter) {
        Ok(risks) => {
            println!("📊 Risk Register");
            println!("════════════════");
            
            if risks.is_empty() {
                println!("No risks found matching the specified criteria.");
                return Ok(());
            }
            
            println!("Total risks: {}\n", risks.len());
            
            // Display table header
            println!("{:<12} {:<30} {:<10} {:<5} {:<8} {:<12} {:<10}", 
                     "Risk ID", "Hazard Description", "Severity", "RPN", "Level", "Mitigations", "Status");
            println!("{}", "-".repeat(95));
            
            // Display risks
            for risk in &risks {
                let hazard_desc = if risk.hazard_description.len() > 28 {
                    format!("{}...", &risk.hazard_description[..25])
                } else {
                    risk.hazard_description.clone()
                };
                
                println!("{:<12} {:<30} {:<10} {:<5} {:<8} {:<12} {:<10}", 
                         risk.hazard_id,
                         hazard_desc,
                         format!("{:?}", risk.severity),
                         risk.risk_priority_number,
                         format!("{:?}", risk.initial_risk_level),
                         risk.mitigation_measures.len(),
                         format!("{:?}", risk.risk_status)
                );
            }
            
            println!("\nUse 'qms risk view <risk-id>' for detailed information");
            println!("Use 'qms risk export-register' to export this data");
        }
        Err(e) => {
            return Err(format!("Failed to get risk register: {e}"));
        }
    }
    
    Ok(())
}

/// Handle risk statistics command
/// Task 3.1.7: Risk Register
fn handle_risk_stats(_args: &[String]) -> Result<(), String> {
    let project_path = crate::utils::get_current_project_path().map_err(|e| e.to_string())?;
    let risk_manager = RiskManager::new(&project_path).map_err(|e| e.to_string())?;
    
    match risk_manager.get_risk_register_stats() {
        Ok(stats) => {
            println!("📈 Risk Register Statistics");
            println!("═══════════════════════════");
            println!("Total Risks: {}", stats.total_risks);
            println!("Open Risks: {}", stats.open_risks);
            println!("High Priority Risks: {}", stats.high_priority_risks);
            println!("Overdue Mitigations: {}", stats.overdue_mitigations);
            println!("Average RPN: {:.1}", stats.avg_rpn);
            
            println!("\n📊 Severity Distribution:");
            for (severity, count) in &stats.severity_distribution {
                println!("  {severity}: {count}");
            }
            
            println!("\n🔍 Status Distribution:");
            for (status, count) in &stats.status_distribution {
                println!("  {status}: {count}");
            }
        }
        Err(e) => {
            return Err(format!("Failed to get risk statistics: {e}"));
        }
    }
    
    Ok(())
}

/// Handle export risk register command
/// Task 3.1.7: Risk Register
fn handle_export_risk_register(args: &[String]) -> Result<(), String> {
    let mut filter = RiskRegisterFilter {
        sort_by: "rpn:desc".to_string(),
        ..Default::default()
    };
    let mut format = "csv".to_string();
    let mut output_path = "risk_register.csv".to_string();
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--format" => {
                if i + 1 < args.len() {
                    format = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --format".to_string());
                }
            }
            "--output" => {
                if i + 1 < args.len() {
                    output_path = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --output".to_string());
                }
            }
            "--filter" => {
                if i + 1 < args.len() {
                    let filter_value = &args[i + 1];
                    if let Some((key, value)) = filter_value.split_once(':') {
                        match key {
                            "status" => filter.status = Some(value.to_string()),
                            "severity" => {
                                filter.severity = parse_severity_arg(value);
                            },
                            "assignee" => filter.assignee = Some(value.to_string()),
                            "mitigation-status" => filter.mitigation_status = Some(value.to_string()),
                            _ => {
                                return Err(format!("Unknown filter key: {key}"));
                            }
                        }
                    }
                    i += 2;
                } else {
                    return Err("Missing value for --filter".to_string());
                }
            }
            "--sort" => {
                if i + 1 < args.len() {
                    filter.sort_by = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --sort".to_string());
                }
            }
            "--help" | "-h" => {
                print_export_register_help();
                return Ok(());
            }
            _ => {
                i += 1;
            }
        }
    }
    
    let project_path = crate::utils::get_current_project_path().map_err(|e| e.to_string())?;
    let risk_manager = RiskManager::new(&project_path).map_err(|e| e.to_string())?;
    
    match risk_manager.export_risk_register(&filter, &format, &output_path) {
        Ok(()) => {
            println!("✅ Risk register exported successfully!");
        }
        Err(e) => {
            return Err(format!("Failed to export risk register: {e}"));
        }
    }
    
    Ok(())
}

fn parse_severity_arg(arg: &str) -> Option<RiskSeverity> {
    match arg.to_lowercase().as_str() {
        "catastrophic" | "5" => Some(RiskSeverity::Catastrophic),
        "critical" | "4" => Some(RiskSeverity::Critical),
        "major" | "3" => Some(RiskSeverity::Major),
        "minor" | "2" => Some(RiskSeverity::Minor),
        "negligible" | "1" => Some(RiskSeverity::Negligible),
        _ => None,
    }
}

fn print_register_help() {
    println!("📊 Risk Register - Comprehensive risk database with filtering/sorting");
    println!();
    println!("USAGE:");
    println!("    qms risk register [--filter <key:value>] [--sort <field:direction>]");
    println!();
    println!("FILTER OPTIONS:");
    println!("    --filter status:open            Show only open risks");
    println!("    --filter status:closed          Show only closed risks");
    println!("    --filter status:high-priority   Show only high-priority risks");
    println!("    --filter status:overdue         Show only overdue mitigations");
    println!("    --filter severity:critical      Filter by severity level");
    println!("    --filter assignee:username      Filter by assigned user");
    println!("    --filter mitigation-status:pending  Filter by mitigation status");
    println!();
    println!("SORT OPTIONS:");
    println!("    --sort rpn:desc         Sort by RPN (descending)");
    println!("    --sort rpn:asc          Sort by RPN (ascending)");
    println!("    --sort severity:desc    Sort by severity (descending)");
    println!("    --sort created:desc     Sort by creation date (newest first)");
    println!("    --sort updated:desc     Sort by update date (newest first)");
    println!();
    println!("EXAMPLES:");
    println!("    qms risk register");
    println!("    qms risk register --filter status:open --sort rpn:desc");
    println!("    qms risk register --filter severity:critical");
    println!("    qms risk register --filter assignee:john.doe");
}

fn print_export_register_help() {
    println!("📤 Export Risk Register - Export risk data to various formats");
    println!();
    println!("USAGE:");
    println!("    qms risk export-register [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --format <format>       Export format (csv, pdf, json)");
    println!("    --output <file>         Output file path");
    println!("    --filter <key:value>    Apply filters (same as register command)");
    println!("    --sort <field:dir>      Sort order (same as register command)");
    println!();
    println!("EXAMPLES:");
    println!("    qms risk export-register --format csv --output risks.csv");
    println!("    qms risk export-register --format pdf --output risk_summary.pdf");
    println!("    qms risk export-register --format json --filter status:open");
}

// ===============================================================
// Task 3.1.8: Risk Monitoring & Tracking Help Functions
// ===============================================================

fn print_risk_status_help() {
    println!("📊 Risk Status Dashboard");
    println!();
    println!("Display comprehensive risk status dashboard with lifecycle tracking.");
    println!();
    println!("USAGE:");
    println!("    qms risk status");
    println!();
    println!("DISPLAYS:");
    println!("    • Total risks count");
    println!("    • Risks by status (Identified, Assessed, Mitigated, Verified, Closed)");
    println!("    • Overdue risks count");
    println!("    • High RPN risks (≥50)");
    println!();
    println!("EXAMPLES:");
    println!("    qms risk status");
}

fn print_risk_overdue_help() {
    println!("⚠️ Overdue Risks Report");
    println!();
    println!("Display all risks that are past their due dates and not closed.");
    println!();
    println!("USAGE:");
    println!("    qms risk overdue");
    println!();
    println!("DISPLAYS:");
    println!("    • Risk ID and description");
    println!("    • Current status");
    println!("    • Due date");
    println!("    • RPN value");
    println!("    • Assigned user (if any)");
    println!();
    println!("EXAMPLES:");
    println!("    qms risk overdue");
}

fn print_risk_escalate_help() {
    println!("🚨 Risk Escalation Alerts");
    println!();
    println!("Display risks requiring management escalation and attention.");
    println!();
    println!("USAGE:");
    println!("    qms risk escalate [--threshold <rpn>]");
    println!();
    println!("OPTIONS:");
    println!("    --threshold <rpn>       RPN threshold for high-risk alerts (default: 50)");
    println!();
    println!("ALERT TYPES:");
    println!("    • HighRPN: Risks with RPN above threshold");
    println!("    • OverdueAssessment: Identified risks needing assessment");
    println!("    • PendingVerification: Mitigated risks needing verification");
    println!("    • OverdueMitigation: Risks past due date");
    println!("    • EscalationRequired: Critical/Catastrophic risks not closed");
    println!();
    println!("EXAMPLES:");
    println!("    qms risk escalate");
    println!("    qms risk escalate --threshold 75");
}

fn print_risk_update_status_help() {
    println!("🔄 Update Risk Status");
    println!();
    println!("Update risk through lifecycle: Identified → Assessed → Mitigated → Verified → Closed");
    println!();
    println!("USAGE:");
    println!("    qms risk update-status <risk-id> <status>");
    println!();
    println!("ARGUMENTS:");
    println!("    <risk-id>              Risk ID or hazard ID (e.g., HAZ-001)");
    println!("    <status>               New status (identified, assessed, mitigated, verified, closed)");
    println!();
    println!("STATUS TRANSITIONS:");
    println!("    Identified → Assessed  (risk assessment completed)");
    println!("    Assessed → Mitigated   (mitigation measures implemented)");
    println!("    Mitigated → Verified   (mitigation effectiveness verified)");
    println!("    Verified → Closed      (risk fully addressed)");
    println!("    Note: Reverse transitions allowed for corrections");
    println!();
    println!("EXAMPLES:");
    println!("    qms risk update-status HAZ-001 assessed");
    println!("    qms risk update-status abc12345 mitigated");
    println!("    qms risk update-status HAZ-003 closed");
}

/// Handle risk report command
/// Task 3.1.10: Risk Reporting & Analytics
fn handle_risk_report(args: &[String]) -> Result<(), String> {
    let mut report_type = ReportType::Summary;
    let mut period: Option<TimePeriod> = None;
    let mut output_file: Option<String> = None;
    let mut format = ReportFormat::Markdown;
    
    // Parse arguments
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--type" => {
                if i + 1 < args.len() {
                    report_type = match args[i + 1].as_str() {
                        "summary" => ReportType::Summary,
                        "trend" => ReportType::TrendAnalysis,
                        "fmea" => ReportType::FMEA,
                        "compliance" => ReportType::Compliance,
                        "mitigation" => ReportType::MitigationEffectiveness,
                        "distribution" => ReportType::RiskDistribution,
                        _ => {
                            return Err(format!("Invalid report type: {}. Supported: summary, trend, fmea, compliance, mitigation, distribution", args[i + 1]));
                        }
                    };
                    i += 2;
                } else {
                    return Err("Missing value for --type".to_string());
                }
            }
            "--period" => {
                if i + 1 < args.len() {
                    period = Some(parse_time_period(&args[i + 1])?);
                    i += 2;
                } else {
                    return Err("Missing value for --period".to_string());
                }
            }
            "--output" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --output".to_string());
                }
            }
            "--format" => {
                if i + 1 < args.len() {
                    format = match args[i + 1].as_str() {
                        "markdown" | "md" => ReportFormat::Markdown,
                        "csv" => ReportFormat::CSV,
                        "json" => ReportFormat::JSON,
                        "html" => ReportFormat::HTML,
                        "pdf" => ReportFormat::PDF,
                        _ => {
                            return Err(format!("Invalid format: {}. Supported: markdown, csv, json, html, pdf", args[i + 1]));
                        }
                    };
                    i += 2;
                } else {
                    return Err("Missing value for --format".to_string());
                }
            }
            "--help" | "-h" => {
                print_risk_report_help();
                return Ok(());
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }
    
    // Get current project path
    let project_path = get_current_project_path().map_err(|e| e.to_string())?;
    
    // Initialize risk manager and reporter
    let risk_manager = RiskManager::new(&project_path).map_err(|e| e.to_string())?;
    let reporter = RiskReporter::new(&project_path).map_err(|e| e.to_string())?;
    
    // Load all risks
    let risks = match risk_manager.load_all_risks() {
        Ok(risks) => risks,
        Err(_) => {
            println!("⚠️  No risks found in current project");
            return Ok(());
        }
    };
    
    if risks.is_empty() {
        println!("⚠️  No risks found in current project");
        println!("Use 'qms risk create' to add risks first");
        return Ok(());
    }
    
    // Generate report based on type
    let report_content = match report_type {
        ReportType::Summary => {
            println!("📊 Generating risk summary report...");
            reporter.generate_summary_report(&risks, format.clone()).map_err(|e| e.to_string())?
        }
        ReportType::TrendAnalysis => {
            println!("📈 Generating trend analysis report...");
            let period_to_use = period.unwrap_or(TimePeriod::Months(6));
            let trends = reporter.generate_trend_analysis(period_to_use).map_err(|e| e.to_string())?;
            format_trend_report(&trends)
        }
        _ => {
            return Err(format!("Report type {report_type:?} not yet implemented"));
        }
    };
    
    // Output or save report
    if let Some(file_path) = output_file {
        reporter.save_report_to_file(&report_content, &file_path).map_err(|e| e.to_string())?;
        println!("✅ Report saved to: risks/reports/{file_path}");
        
        // Also show a preview for user
        println!("\n📄 Report Preview (first 500 characters):");
        println!("{}", &report_content[..std::cmp::min(500, report_content.len())]);
        if report_content.len() > 500 {
            println!("...\n[Content truncated. Full report saved to file]");
        }
    } else {
        // Display full report to console
        println!("📄 Risk Report:");
        println!("{report_content}");
    }
    
    Ok(())
}

/// Parse time period string into TimePeriod enum
fn parse_time_period(period_str: &str) -> Result<TimePeriod, String> {
    if period_str.ends_with("days") || period_str.ends_with("day") {
        let num_str = period_str.trim_end_matches("days").trim_end_matches("day");
        let num: u32 = num_str.parse().map_err(|_| format!("Invalid number in period: {period_str}"))?;
        Ok(TimePeriod::Days(num))
    } else if period_str.ends_with("weeks") || period_str.ends_with("week") {
        let num_str = period_str.trim_end_matches("weeks").trim_end_matches("week");
        let num: u32 = num_str.parse().map_err(|_| format!("Invalid number in period: {period_str}"))?;
        Ok(TimePeriod::Weeks(num))
    } else if period_str.ends_with("months") || period_str.ends_with("month") {
        let num_str = period_str.trim_end_matches("months").trim_end_matches("month");
        let num: u32 = num_str.parse().map_err(|_| format!("Invalid number in period: {period_str}"))?;
        Ok(TimePeriod::Months(num))
    } else if period_str.ends_with("years") || period_str.ends_with("year") {
        let num_str = period_str.trim_end_matches("years").trim_end_matches("year");
        let num: u32 = num_str.parse().map_err(|_| format!("Invalid number in period: {period_str}"))?;
        Ok(TimePeriod::Years(num))
    } else {
        // Try common shortcuts
        match period_str {
            "1week" => Ok(TimePeriod::Weeks(1)),
            "2weeks" => Ok(TimePeriod::Weeks(2)),
            "1month" => Ok(TimePeriod::Months(1)),
            "3months" => Ok(TimePeriod::Months(3)),
            "6months" => Ok(TimePeriod::Months(6)),
            "1year" => Ok(TimePeriod::Years(1)),
            _ => Err(format!("Invalid period format: {period_str}. Use format like '6months', '2weeks', '1year'"))
        }
    }
}

/// Format trend analysis report
fn format_trend_report(trends: &[RiskTrend]) -> String {
    let mut report = String::new();
    report.push_str("# Risk Trend Analysis Report\n\n");
    
    if trends.is_empty() {
        report.push_str("No trend data available.\n");
        return report;
    }
    
    report.push_str("| Period | Total Risks | New | Closed | Avg RPN | High Priority |\n");
    report.push_str("|--------|-------------|-----|--------|---------|---------------|\n");
    
    for trend in trends {
        report.push_str(&format!(
            "| {} | {} | {} | {} | {:.1} | {} |\n",
            trend.period,
            trend.total_risks,
            trend.new_risks,
            trend.closed_risks,
            trend.average_rpn,
            trend.high_priority_risks
        ));
    }
    
    report
}

/// Print help for risk report command
fn print_risk_report_help() {
    println!("📊 Risk Reporting & Analytics");
    println!();
    println!("Generate comprehensive risk management reports and analytics");
    println!();
    println!("USAGE:");
    println!("    qms risk report [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --type <TYPE>        Report type [default: summary]");
    println!("                         Options: summary, trend, fmea, compliance, mitigation, distribution");
    println!("    --period <PERIOD>    Time period for trend analysis");
    println!("                         Examples: 6months, 2weeks, 1year, 30days");
    println!("    --format <FORMAT>    Output format [default: markdown]");
    println!("                         Options: markdown, csv, json, html, pdf");
    println!("    --output <FILE>      Save report to file (relative to risks/reports/)");
    println!("    -h, --help           Show this help message");
    println!();
    println!("REPORT TYPES:");
    println!("    summary              Executive summary with key metrics and top risks");
    println!("    trend                Risk trends over time (requires --period)");
    println!("    fmea                 FMEA-specific analysis and reporting");
    println!("    compliance           Regulatory compliance status (ISO 14971, FDA)");
    println!("    mitigation           Mitigation effectiveness analysis");
    println!("    distribution         Risk distribution by RPN, level, category");
    println!();
    println!("EXAMPLES:");
    println!("    qms risk report");
    println!("    qms risk report --type summary --format pdf --output summary.pdf");
    println!("    qms risk report --type trend --period 6months");
    println!("    qms risk report --type compliance --format json");
    println!("    qms risk report --type mitigation --output mitigation_analysis.md");
}

// =====================================================
// Task 3.1.11: ISO 14971 Compliance Implementation
// =====================================================

/// Handle ISO 14971 compliance validation
fn handle_iso14971_check(args: &[String]) -> Result<(), String> {
    let project_path = crate::utils::get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;
    
    let mut detailed = false;
    let mut output_file: Option<String> = None;
    let mut format = "table".to_string();
    
    // Parse arguments
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--detailed" | "-d" => {
                detailed = true;
            }
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    return Err("Error: --output requires a file path".to_string());
                }
            }
            "--format" | "-f" => {
                if i + 1 < args.len() {
                    format = args[i + 1].clone();
                    i += 1;
                } else {
                    return Err("Error: --format requires a format type".to_string());
                }
            }
            "--help" | "-h" => {
                print_iso14971_check_help();
                return Ok(());
            }
            _ => {
                return Err(format!("Error: Unknown option '{}'", args[i]));
            }
        }
        i += 1;
    }
    
    println!("🔍 Performing ISO 14971 compliance validation...");
    
    let validator = ISO14971Validator::new(&project_path)
        .map_err(|e| format!("Failed to create validator: {e}"))?;
    
    let report = validator.validate_iso14971_process()
        .map_err(|e| format!("Compliance validation failed: {e}"))?;
    
    // Display results
    match format.as_str() {
        "table" => print_compliance_report_table(&report, detailed),
        "json" => print_compliance_report_json(&report),
        "summary" => print_compliance_report_summary(&report),
        _ => {
            return Err(format!("Error: Unsupported format '{format}'"));
        }
    }
    
    // Save to file if requested
    if let Some(output_path) = output_file {
        let content = match format.as_str() {
            "json" => format_compliance_report_json(&report),
            _ => format_compliance_report_markdown(&report, detailed),
        };
        
        let full_path = project_path.join("risks").join("reports").join(&output_path);
        if let Some(parent) = full_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&full_path, content)
            .map_err(|e| format!("Failed to write report: {e}"))?;
        
        println!("📄 Report saved to: {}", full_path.display());
    }
    
    Ok(())
}

/// Handle Risk Management File (RMF) generation
fn handle_generate_rmf(args: &[String]) -> Result<(), String> {
    let project_path = crate::utils::get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;
    
    let mut output_file = "rmf.md".to_string();
    let mut format = RMFFormat::Markdown;
    let mut include_detailed = true;
    let mut include_fmea = true;
    let mut include_verification = true;
    let mut jurisdiction = "FDA".to_string();
    
    // Parse arguments
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_file = args[i + 1].clone();
                    i += 1;
                } else {
                    return Err("Error: --output requires a file path".to_string());
                }
            }
            "--format" | "-f" => {
                if i + 1 < args.len() {
                    format = match args[i + 1].as_str() {
                        "markdown" | "md" => RMFFormat::Markdown,
                        "pdf" => RMFFormat::PDF,
                        "html" => RMFFormat::HTML,
                        "word" => RMFFormat::Word,
                        _ => return Err(format!("Error: Unsupported format '{}'", args[i + 1])),
                    };
                    i += 1;
                } else {
                    return Err("Error: --format requires a format type".to_string());
                }
            }
            "--no-detailed" => {
                include_detailed = false;
            }
            "--no-fmea" => {
                include_fmea = false;
            }
            "--no-verification" => {
                include_verification = false;
            }
            "--jurisdiction" | "-j" => {
                if i + 1 < args.len() {
                    jurisdiction = args[i + 1].clone();
                    i += 1;
                } else {
                    return Err("Error: --jurisdiction requires a jurisdiction".to_string());
                }
            }
            "--help" | "-h" => {
                print_generate_rmf_help();
                return Ok(());
            }
            _ => {
                return Err(format!("Error: Unknown option '{}'", args[i]));
            }
        }
        i += 1;
    }
    
    let options = RMFOptions {
        output_format: format,
        include_detailed_analysis: include_detailed,
        include_fmea_data: include_fmea,
        include_verification_evidence: include_verification,
        regulatory_jurisdiction: jurisdiction,
    };
    
    let validator = ISO14971Validator::new(&project_path)
        .map_err(|e| format!("Failed to create validator: {e}"))?;
    
    let output_path = project_path.join("risks").join("reports").join(&output_file);
    if let Some(parent) = output_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    
    validator.generate_rmf(&output_path, &options)
        .map_err(|e| format!("RMF generation failed: {e}"))?;
    
    println!("✅ Risk Management File generated successfully");
    println!("📄 Output: {}", output_path.display());
    
    Ok(())
}

/// Handle compliance gap analysis
fn handle_compliance_gaps(args: &[String]) -> Result<(), String> {
    let project_path = crate::utils::get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;
    
    let mut output_file: Option<String> = None;
    
    // Parse arguments
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    return Err("Error: --output requires a file path".to_string());
                }
            }
            "--help" | "-h" => {
                print_compliance_gaps_help();
                return Ok(());
            }
            _ => {
                return Err(format!("Error: Unknown option '{}'", args[i]));
            }
        }
        i += 1;
    }
    
    println!("🔍 Analyzing ISO 14971 compliance gaps...");
    
    let validator = ISO14971Validator::new(&project_path)
        .map_err(|e| format!("Failed to create validator: {e}"))?;
    
    let gaps = validator.check_compliance_gaps()
        .map_err(|e| format!("Gap analysis failed: {e}"))?;
    
    if gaps.is_empty() {
        println!("✅ No significant compliance gaps identified!");
        println!("   Your risk management process appears to be well-aligned with ISO 14971 requirements.");
    } else {
        println!("⚠️  Compliance gaps identified ({} issues):", gaps.len());
        println!();
        
        for (index, gap) in gaps.iter().enumerate() {
            println!("{}. {}", index + 1, gap);
        }
        
        println!();
        println!("💡 Recommendations:");
        println!("   - Address critical gaps before regulatory submission");
        println!("   - Use 'qms risk iso14971-check --detailed' for specific recommendations");
        println!("   - Generate RMF with 'qms risk generate-rmf' after addressing gaps");
    }
    
    // Save gaps to file if requested
    if let Some(output_path) = output_file {
        let content = format_compliance_gaps(&gaps);
        let full_path = project_path.join("risks").join("reports").join(&output_path);
        if let Some(parent) = full_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&full_path, content)
            .map_err(|e| format!("Failed to write gaps report: {e}"))?;
        
        println!("📄 Gaps analysis saved to: {}", full_path.display());
    }
    
    Ok(())
}

// Compliance report formatting functions

fn print_compliance_report_table(report: &crate::modules::risk_manager::iso14971::ComplianceReport, detailed: bool) {
    println!();
    println!("📋 ISO 14971 Compliance Report");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Project: {} ({})", report.project_name, report.project_id);
    println!("Assessment Date: {}", report.assessment_date);
    println!("Overall Compliance: {:.1}% ({})", 
             report.overall_compliance_percentage, report.overall_status);
    println!();
    
    println!("┌─────────┬──────────────────────────────────────┬────────────┬─────────────┐");
    println!("│ Section │ Title                                │ Compliance │ Status      │");
    println!("├─────────┼──────────────────────────────────────┼────────────┼─────────────┤");
    
    for section in &report.sections {
        println!("│ {:7} │ {:<36} │ {:8.1}% │ {} │", 
                 section.section,
                 if section.title.len() > 36 { &section.title[..33] } else { &section.title },
                 section.compliance_percentage,
                 format_status_short(&section.status));
    }
    
    println!("└─────────┴──────────────────────────────────────┴────────────┴─────────────┘");
    
    if detailed {
        println!();
        println!("📋 Detailed Section Analysis:");
        for section in &report.sections {
            println!();
            println!("Section {}: {}", section.section, section.title);
            println!("  Compliance: {:.1}% ({})", section.compliance_percentage, section.status);
            
            if !section.satisfied_elements.is_empty() {
                println!("  ✅ Satisfied ({}):", section.satisfied_elements.len());
                for element in &section.satisfied_elements {
                    println!("     - {element}");
                }
            }
            
            if !section.missing_elements.is_empty() {
                println!("  ❌ Missing ({}):", section.missing_elements.len());
                for element in &section.missing_elements {
                    println!("     - {element}");
                }
            }
            
            if !section.recommendations.is_empty() {
                println!("  💡 Recommendations:");
                for rec in &section.recommendations {
                    println!("     - {rec}");
                }
            }
        }
    }
    
    if !report.critical_gaps.is_empty() {
        println!();
        println!("🚨 Critical Gaps:");
        for gap in &report.critical_gaps {
            println!("   - {gap}");
        }
    }
    
    if !report.recommendations.is_empty() {
        println!();
        println!("💡 Overall Recommendations:");
        for rec in &report.recommendations {
            println!("   - {rec}");
        }
    }
    
    println!();
    println!("Next Review: {}", report.next_review_date);
}

fn print_compliance_report_summary(report: &crate::modules::risk_manager::iso14971::ComplianceReport) {
    println!();
    println!("📊 ISO 14971 Compliance Summary");
    println!("═══════════════════════════════════");
    println!("Project: {}", report.project_name);
    println!("Overall Compliance: {:.1}% ({})", 
             report.overall_compliance_percentage, report.overall_status);
    
    let fully_compliant = report.sections.iter().filter(|s| s.compliance_percentage >= 100.0).count();
    let substantially_compliant = report.sections.iter().filter(|s| s.compliance_percentage >= 80.0 && s.compliance_percentage < 100.0).count();
    let partially_compliant = report.sections.iter().filter(|s| s.compliance_percentage >= 50.0 && s.compliance_percentage < 80.0).count();
    let non_compliant = report.sections.iter().filter(|s| s.compliance_percentage < 50.0).count();
    
    println!();
    println!("Section Status:");
    println!("  ✅ Fully Compliant: {fully_compliant}");
    println!("  🟡 Substantially Compliant: {substantially_compliant}");
    println!("  ⚠️  Partially Compliant: {partially_compliant}");
    println!("  ❌ Non-Compliant: {non_compliant}");
    
    if report.overall_compliance_percentage >= 80.0 {
        println!();
        println!("🎯 Ready for regulatory review with minor improvements needed.");
    } else if report.overall_compliance_percentage >= 50.0 {
        println!();
        println!("⚠️  Significant improvements needed before regulatory submission.");
    } else {
        println!();
        println!("🚨 Major compliance work required. Consider regulatory consulting.");
    }
}

fn print_compliance_report_json(report: &crate::modules::risk_manager::iso14971::ComplianceReport) {
    let json = format_compliance_report_json(report);
    println!("{json}");
}

fn format_compliance_report_json(report: &crate::modules::risk_manager::iso14971::ComplianceReport) -> String {
    // Basic JSON formatting for compliance report
    format!(
        r#"{{
  "compliance_report": {{
    "project_id": "{}",
    "project_name": "{}",
    "assessment_date": "{}",
    "overall_compliance_percentage": {},
    "overall_status": "{}",
    "sections_count": {},
    "critical_gaps_count": {},
    "recommendations_count": {},
    "next_review_date": "{}"
  }}
}}"#,
        report.project_id,
        report.project_name,
        report.assessment_date,
        report.overall_compliance_percentage,
        report.overall_status,
        report.sections.len(),
        report.critical_gaps.len(),
        report.recommendations.len(),
        report.next_review_date
    )
}

fn format_compliance_report_markdown(report: &crate::modules::risk_manager::iso14971::ComplianceReport, detailed: bool) -> String {
    let mut content = String::new();
    content.push_str("# ISO 14971 Compliance Report\n\n");
    content.push_str(&format!("**Project**: {} ({})\n", report.project_name, report.project_id));
    content.push_str(&format!("**Assessment Date**: {}\n", report.assessment_date));
    content.push_str(&format!("**Overall Compliance**: {:.1}% ({})\n\n", 
                             report.overall_compliance_percentage, report.overall_status));
    
    content.push_str("## Section Summary\n\n");
    content.push_str("| Section | Title | Compliance | Status |\n");
    content.push_str("|---------|-------|------------|---------|\n");
    
    for section in &report.sections {
        content.push_str(&format!("| {} | {} | {:.1}% | {:?} |\n",
                                section.section, section.title,
                                section.compliance_percentage,
                                section.status));
    }
    
    if detailed {
        content.push_str("\n## Detailed Analysis\n\n");
        for section in &report.sections {
            content.push_str(&format!("### Section {}: {}\n\n", section.section, section.title));
            content.push_str(&format!("**Compliance**: {:.1}%\n\n", section.compliance_percentage));
            
            if !section.satisfied_elements.is_empty() {
                content.push_str("**Satisfied Elements**:\n");
                for element in &section.satisfied_elements {
                    content.push_str(&format!("- ✅ {element}\n"));
                }
                content.push('\n');
            }
            
            if !section.missing_elements.is_empty() {
                content.push_str("**Missing Elements**:\n");
                for element in &section.missing_elements {
                    content.push_str(&format!("- ❌ {element}\n"));
                }
                content.push('\n');
            }
        }
    }
    
    content
}

fn format_compliance_gaps(gaps: &[String]) -> String {
    let mut content = String::new();
    content.push_str("# ISO 14971 Compliance Gap Analysis\n\n");
    content.push_str(&format!("**Assessment Date**: {}\n", crate::utils::current_date_string()));
    content.push_str(&format!("**Total Gaps Identified**: {}\n\n", gaps.len()));
    
    if gaps.is_empty() {
        content.push_str("✅ No significant compliance gaps identified.\n");
    } else {
        content.push_str("## Identified Gaps\n\n");
        for (index, gap) in gaps.iter().enumerate() {
            content.push_str(&format!("{}. {}\n", index + 1, gap));
        }
    }
    
    content
}

fn format_status_short(status: &ComplianceStatus) -> String {
    match status {
        ComplianceStatus::FullyCompliant => "✅ Full".to_string(),
        ComplianceStatus::SubstantiallyCompliant => "🟡 Subst".to_string(),
        ComplianceStatus::PartiallyCompliant => "⚠️  Part".to_string(),
        ComplianceStatus::NonCompliant => "❌ None".to_string(),
    }
}

// Help functions for ISO 14971 commands

fn print_iso14971_check_help() {
    println!("🔍 ISO 14971 Compliance Validation");
    println!();
    println!("Perform comprehensive validation of risk management process compliance");
    println!("with ISO 14971 standard for medical devices.");
    println!();
    println!("USAGE:");
    println!("    qms risk iso14971-check [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -d, --detailed       Show detailed section-by-section analysis");
    println!("    -f, --format <TYPE>  Output format [default: table]");
    println!("                         Options: table, json, summary");
    println!("    -o, --output <FILE>  Save report to file (relative to risks/reports/)");
    println!("    -h, --help           Show this help message");
    println!();
    println!("ISO 14971 SECTIONS VALIDATED:");
    println!("    3. Risk management process");
    println!("    4. General requirements for risk management");
    println!("    5. Risk analysis");
    println!("    6. Risk evaluation");
    println!("    7. Risk control");
    println!("    8. Overall residual risk evaluation");
    println!("    9. Risk management report");
    println!("    10. Production and post-production information");
    println!();
    println!("COMPLIANCE LEVELS:");
    println!("    ✅ Fully Compliant (100%)");
    println!("    🟡 Substantially Compliant (80-99%)");
    println!("    ⚠️  Partially Compliant (50-79%)");
    println!("    ❌ Non-Compliant (<50%)");
    println!();
    println!("EXAMPLES:");
    println!("    qms risk iso14971-check");
    println!("    qms risk iso14971-check --detailed");
    println!("    qms risk iso14971-check --format json --output compliance_report.json");
    println!("    qms risk iso14971-check --format summary");
}

fn print_generate_rmf_help() {
    println!("📋 Risk Management File (RMF) Generation");
    println!();
    println!("Generate comprehensive Risk Management File in accordance with ISO 14971");
    println!("for regulatory submissions and compliance documentation.");
    println!();
    println!("USAGE:");
    println!("    qms risk generate-rmf [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -o, --output <FILE>     Output file name [default: rmf.md]");
    println!("    -f, --format <FORMAT>   Output format [default: markdown]");
    println!("                            Options: markdown, pdf, html, word");
    println!("    -j, --jurisdiction <J>  Regulatory jurisdiction [default: FDA]");
    println!("                            Options: FDA, CE, Health_Canada, TGA");
    println!("    --no-detailed           Exclude detailed risk analysis");
    println!("    --no-fmea              Exclude FMEA data");
    println!("    --no-verification      Exclude verification evidence");
    println!("    -h, --help             Show this help message");
    println!();
    println!("RMF CONTENTS:");
    println!("    • Risk management policy and procedures");
    println!("    • Risk analysis results and documentation");
    println!("    • Risk evaluation and acceptability criteria");
    println!("    • Risk control measures and effectiveness");
    println!("    • Residual risk analysis and justification");
    println!("    • Overall risk management conclusions");
    println!("    • Post-market surveillance plan");
    println!();
    println!("OUTPUT FORMATS:");
    println!("    markdown    Markdown format for version control and editing");
    println!("    pdf         PDF format for regulatory submissions");
    println!("    html        HTML format for web-based reviews");
    println!("    word        Word format for collaborative editing");
    println!();
    println!("EXAMPLES:");
    println!("    qms risk generate-rmf");
    println!("    qms risk generate-rmf --format pdf --output medical_device_rmf.pdf");
    println!("    qms risk generate-rmf --jurisdiction CE --no-fmea");
    println!("    qms risk generate-rmf --format html --output rmf_review.html");
}

fn print_compliance_gaps_help() {
    println!("🔍 ISO 14971 Compliance Gap Analysis");
    println!();
    println!("Identify specific compliance gaps and areas requiring attention");
    println!("before regulatory submission.");
    println!();
    println!("USAGE:");
    println!("    qms risk compliance-gaps [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -o, --output <FILE>  Save gap analysis to file (relative to risks/reports/)");
    println!("    -h, --help           Show this help message");
    println!();
    println!("GAP ANALYSIS INCLUDES:");
    println!("    • Missing risk management documentation");
    println!("    • Incomplete risk assessments");
    println!("    • Unaddressed unacceptable risks");
    println!("    • Missing verification evidence");
    println!("    • Inadequate post-market surveillance planning");
    println!();
    println!("EXAMPLES:");
    println!("    qms risk compliance-gaps");
    println!("    qms risk compliance-gaps --output gap_analysis.md");
}

/// Handle risk import command
/// Task 3.1.12: Risk Import/Export
fn handle_risk_import(args: &[String]) -> Result<(), String> {
    let mut file_path = None;
    let mut format = None;
    let mut validate_only = false;
    let mut skip_duplicates = true;
    let mut component_filter = None;
    
    // Parse arguments
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--file" | "-f" => {
                if i + 1 < args.len() {
                    file_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --file requires a file path".to_string());
                }
            }
            "--format" => {
                if i + 1 < args.len() {
                    format = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --format requires a format (csv, json, fmea-template)".to_string());
                }
            }
            "--validate-only" => {
                validate_only = true;
                i += 1;
            }
            "--allow-duplicates" => {
                skip_duplicates = false;
                i += 1;
            }
            "--component" => {
                if i + 1 < args.len() {
                    component_filter = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --component requires a component name".to_string());
                }
            }
            "--help" | "-h" => {
                print_risk_import_help();
                return Ok(());
            }
            _ => {
                return Err(format!("Error: Unknown option '{}'", args[i]));
            }
        }
    }
    
    // Validate required arguments
    let file_path = file_path.ok_or("Error: --file is required")?;
    let format_str = format.ok_or("Error: --format is required")?;
    
    // Parse format
    let import_format = ImportFormat::from_str(&format_str)
        .map_err(|e| format!("Error: {e}"))?;
    
    // Get current project path
    let project_path = crate::utils::get_current_project_path()
        .map_err(|e| format!("Error: {e}"))?;
    
    // Create importer
    let mut importer = RiskImporter::new(&project_path)
        .map_err(|e| format!("Error: Failed to create importer: {e}"))?;
    
    // Set up import options
    let options = ImportOptions {
        validate_only,
        skip_duplicates,
        backup_existing: true,
        component_filter,
    };
    
    // Perform import
    let file_path = Path::new(&file_path);
    match importer.import_from_file(file_path, import_format, options) {
        Ok(result) => {
            println!("✅ Risk Import Completed");
            println!();
            println!("📊 IMPORT SUMMARY:");
            println!("   Total Records:    {}", result.total_records);
            println!("   Imported:         {}", result.imported_records);
            println!("   Skipped:          {}", result.skipped_records);
            
            if !result.created_risks.is_empty() {
                println!();
                println!("🆕 Created Risks: {}", result.created_risks.len());
                for risk_id in &result.created_risks {
                    println!("   • {risk_id}");
                }
            }
            
            if !result.updated_risks.is_empty() {
                println!();
                println!("🔄 Updated Risks: {}", result.updated_risks.len());
                for risk_id in &result.updated_risks {
                    println!("   • {risk_id}");
                }
            }
            
            if !result.validation_errors.is_empty() {
                println!();
                println!("⚠️  Validation Warnings: {}", result.validation_errors.len());
                for (i, error) in result.validation_errors.iter().enumerate() {
                    if i < 5 { // Show first 5 warnings
                        println!("   • {error}");
                    }
                }
                if result.validation_errors.len() > 5 {
                    println!("   ... and {} more", result.validation_errors.len() - 5);
                }
            }
            
            if validate_only {
                println!();
                println!("🔍 Validation-only mode: No data was imported");
            }
            
            Ok(())
        }
        Err(e) => Err(format!("Error: Import failed: {e}"))
    }
}

/// Handle risk export command
/// Task 3.1.12: Risk Import/Export
fn handle_risk_export(args: &[String]) -> Result<(), String> {
    let mut format = None;
    let mut output = None;
    let mut include_history = false;
    let mut summary_only = false;
    let mut severity_filter = None;
    let mut component_filter = None;
    
    // Parse arguments
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--format" | "-f" => {
                if i + 1 < args.len() {
                    format = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --format requires a format (csv, json, pdf, fmea-template)".to_string());
                }
            }
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --output requires a file path".to_string());
                }
            }
            "--include-history" => {
                include_history = true;
                i += 1;
            }
            "--summary-only" => {
                summary_only = true;
                i += 1;
            }
            "--min-severity" => {
                if i + 1 < args.len() {
                    severity_filter = Some(args[i + 1].parse::<u8>()
                        .map_err(|_| "Error: --min-severity must be a number (1-5)")?);
                    i += 2;
                } else {
                    return Err("Error: --min-severity requires a number (1-5)".to_string());
                }
            }
            "--component" => {
                if i + 1 < args.len() {
                    component_filter = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --component requires a component name".to_string());
                }
            }
            "--help" | "-h" => {
                print_risk_export_help();
                return Ok(());
            }
            _ => {
                return Err(format!("Error: Unknown option '{}'", args[i]));
            }
        }
    }
    
    // Validate required arguments
    let format_str = format.ok_or("Error: --format is required")?;
    let output_path = output.ok_or("Error: --output is required")?;
    
    // Parse format
    let export_format = ExportFormat::from_str(&format_str)
        .map_err(|e| format!("Error: {e}"))?;
    
    // Get current project path
    let project_path = crate::utils::get_current_project_path()
        .map_err(|e| format!("Error: {e}"))?;
    
    // Create exporter
    let exporter = RiskExporter::new(&project_path)
        .map_err(|e| format!("Error: Failed to create exporter: {e}"))?;
    
    // Set up export options
    let options = ExportOptions {
        include_history,
        summary_only,
        severity_filter,
        status_filter: None,
        component_filter,
    };
    
    // Perform export
    let output_path = Path::new(&output_path);
    match exporter.export_to_file(output_path, export_format, options) {
        Ok(result) => {
            println!("✅ Risk Export Completed");
            println!();
            println!("📊 EXPORT SUMMARY:");
            println!("   Exported Risks:   {}", result.exported_count);
            println!("   Output File:      {}", result.output_file.display());
            println!("   Format:           {:?}", result.format);
            
            // Display file size if available
            if let Ok(metadata) = std::fs::metadata(&result.output_file) {
                let file_size = metadata.len();
                if file_size < 1024 {
                    println!("   File Size:        {file_size} bytes");
                } else if file_size < 1024 * 1024 {
                    println!("   File Size:        {:.1} KB", file_size as f64 / 1024.0);
                } else {
                    println!("   File Size:        {:.1} MB", file_size as f64 / (1024.0 * 1024.0));
                }
            }
            
            match result.format {
                ExportFormat::Csv => {
                    println!();
                    println!("📄 CSV file ready for spreadsheet applications");
                    println!("   • Compatible with Excel, Google Sheets, LibreOffice");
                    println!("   • Includes all risk parameters and calculations");
                }
                ExportFormat::Json => {
                    println!();
                    println!("🔧 JSON file ready for data processing");
                    println!("   • Structured data format for automation");
                    println!("   • Suitable for backup and data exchange");
                }
                ExportFormat::Pdf => {
                    println!();
                    println!("📑 PDF report ready for review");
                    println!("   • Professional format for regulatory submissions");
                    println!("   • Suitable for sharing and archival");
                }
                ExportFormat::FmeaTemplate => {
                    println!();
                    println!("📋 FMEA template ready for analysis");
                    println!("   • Structured for failure mode analysis");
                    println!("   • Compatible with FMEA tools and spreadsheets");
                }
            }
            
            Ok(())
        }
        Err(e) => Err(format!("Error: Export failed: {e}"))
    }
}

/// Print help for risk import command
fn print_risk_import_help() {
    println!("📥 Risk Import - Import risk data from external files");
    println!();
    println!("Import risk data from CSV, JSON, or FMEA template files with");
    println!("comprehensive validation and duplicate detection.");
    println!();
    println!("USAGE:");
    println!("    qms risk import --file <FILE> --format <FORMAT> [OPTIONS]");
    println!();
    println!("REQUIRED OPTIONS:");
    println!("    -f, --file <FILE>        Input file path");
    println!("    --format <FORMAT>        Import format (csv, json, fmea-template)");
    println!();
    println!("OPTIONS:");
    println!("    --validate-only          Only validate, don't import data");
    println!("    --allow-duplicates       Allow importing duplicate hazard IDs");
    println!("    --component <NAME>       Filter FMEA data by component");
    println!("    -h, --help              Show this help message");
    println!();
    println!("IMPORT FORMATS:");
    println!("    csv              Standard CSV with risk data columns");
    println!("                     Required: hazard_id, description, hazardous_situation,");
    println!("                              harm, severity, occurrence, detectability");
    println!();
    println!("    json             JSON export format for backup/restore");
    println!("                     Full risk data with metadata and history");
    println!();
    println!("    fmea-template    FMEA analysis CSV template");
    println!("                     Required: component, function, failure_mode,");
    println!("                              failure_effect, severity, occurrence, detectability");
    println!();
    println!("CSV FORMAT EXAMPLE:");
    println!("    hazard_id,description,hazardous_situation,harm,severity,occurrence,detectability");
    println!("    HAZ-001,\"Battery overheating\",\"Temperature >60°C\",\"Burns\",4,3,2");
    println!();
    println!("VALIDATION:");
    println!("    • File format and structure validation");
    println!("    • Risk parameter validation (1-5 scale)");
    println!("    • Duplicate hazard_id detection");
    println!("    • Required field completeness check");
    println!("    • RPN calculation verification");
    println!();
    println!("EXAMPLES:");
    println!("    qms risk import --file risks.csv --format csv");
    println!("    qms risk import --file backup.json --format json --validate-only");
    println!("    qms risk import --file fmea.csv --format fmea-template --component Battery");
}

/// Print help for risk export command
fn print_risk_export_help() {
    println!("📤 Risk Export - Export risk data to external files");
    println!();
    println!("Export risk data to CSV, JSON, PDF, or FMEA template formats");
    println!("with filtering and customization options.");
    println!();
    println!("USAGE:");
    println!("    qms risk export --format <FORMAT> --output <FILE> [OPTIONS]");
    println!();
    println!("REQUIRED OPTIONS:");
    println!("    -f, --format <FORMAT>    Export format (csv, json, pdf, fmea-template)");
    println!("    -o, --output <FILE>      Output file path");
    println!();
    println!("FILTERING OPTIONS:");
    println!("    --min-severity <N>       Export only risks with severity >= N (1-5)");
    println!("    --component <NAME>       Filter by component name");
    println!();
    println!("FORMAT OPTIONS:");
    println!("    --include-history        Include version history (JSON only)");
    println!("    --summary-only          Generate summary report (PDF only)");
    println!("    -h, --help              Show this help message");
    println!();
    println!("EXPORT FORMATS:");
    println!("    csv              Comma-separated values for spreadsheets");
    println!("                     • Compatible with Excel, Google Sheets");
    println!("                     • Includes all risk parameters and RPN");
    println!();
    println!("    json             Complete JSON export with metadata");
    println!("                     • Structured data for automation");
    println!("                     • Suitable for backup and data exchange");
    println!();
    println!("    pdf              Professional PDF report");
    println!("                     • Summary or detailed risk listings");
    println!("                     • Suitable for regulatory submissions");
    println!();
    println!("    fmea-template    FMEA-compatible CSV template");
    println!("                     • Structured for failure mode analysis");
    println!("                     • Compatible with FMEA tools");
    println!();
    println!("EXAMPLES:");
    println!("    qms risk export --format csv --output all_risks.csv");
    println!("    qms risk export --format pdf --output risk_summary.pdf --summary-only");
    println!("    qms risk export --format json --output backup.json --include-history");
    println!("    qms risk export --format fmea-template --output fmea.csv --component Battery");
    println!("    qms risk export --format csv --output high_risks.csv --min-severity 4");
}

/// Handle post-market surveillance commands
/// Task 3.1.13: Post-Market Surveillance Implementation
fn handle_risk_surveillance(args: &[String]) -> Result<(), String> {
    use crate::modules::risk_manager::SurveillanceManager;
    
    if args.is_empty() {
        print_surveillance_help();
        return Ok(());
    }

    let project_path = crate::utils::get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;

    let mut surveillance_manager = SurveillanceManager::new(&project_path)
        .map_err(|e| format!("Failed to create surveillance manager: {e}"))?;

    match args[0].as_str() {
        "init" => {
            surveillance_manager.initialize()
                .map_err(|e| format!("Failed to initialize surveillance system: {e}"))?;
            println!("✅ Post-market surveillance system initialized");
            Ok(())
        }
        "add-data" => handle_add_surveillance_data(&mut surveillance_manager, &args[1..]),
        "update-frequency" => handle_update_frequency(&mut surveillance_manager, &args[1..]),
        "trigger-review" => handle_trigger_review(&mut surveillance_manager, &args[1..]),
        "add-action" => handle_add_corrective_action(&mut surveillance_manager, &args[1..]),
        "list" => handle_list_surveillance(&surveillance_manager, &args[1..]),
        "report" => handle_surveillance_report(&surveillance_manager, &args[1..]),
        "--help" | "-h" => {
            print_surveillance_help();
            Ok(())
        }
        _ => {
            eprintln!("❌ Unknown surveillance command: {}", args[0]);
            print_surveillance_help();
            Err(format!("Unknown surveillance command: {}", args[0]))
        }
    }
}

/// Add surveillance data
fn handle_add_surveillance_data(manager: &mut SurveillanceManager, args: &[String]) -> Result<(), String> {
    use crate::modules::risk_manager::surveillance::{SurveillanceType, DeviceInfo};
    
    if args.len() < 6 {
        eprintln!("❌ Missing required arguments for add-data");
        println!();
        println!("USAGE:");
        println!("    qms risk surveillance add-data --risk-id <id> --type <type> --source <source> --description <desc> --device-model <model>");
        println!();
        println!("EXAMPLE:");
        println!("    qms risk surveillance add-data --risk-id RISK-001 --type complaint --source \"Customer Support\" --description \"Device alarm malfunction\" --device-model \"Model X1\"");
        return Err("Insufficient arguments".to_string());
    }

    let mut risk_id = None;
    let mut data_type = None;
    let mut source = None;
    let mut description = None;
    let mut device_model = None;
    let mut serial_number = None;
    let mut software_version = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--risk-id" => {
                if i + 1 < args.len() {
                    risk_id = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--risk-id requires a value".to_string());
                }
            }
            "--type" => {
                if i + 1 < args.len() {
                    data_type = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--type requires a value".to_string());
                }
            }
            "--source" => {
                if i + 1 < args.len() {
                    source = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--source requires a value".to_string());
                }
            }
            "--description" => {
                if i + 1 < args.len() {
                    description = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--description requires a value".to_string());
                }
            }
            "--device-model" => {
                if i + 1 < args.len() {
                    device_model = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--device-model requires a value".to_string());
                }
            }
            "--serial-number" => {
                if i + 1 < args.len() {
                    serial_number = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--serial-number requires a value".to_string());
                }
            }
            "--software-version" => {
                if i + 1 < args.len() {
                    software_version = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--software-version requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }

    let risk_id = risk_id.ok_or("--risk-id is required")?;
    let data_type_str = data_type.ok_or("--type is required")?;
    let source = source.ok_or("--source is required")?;
    let description = description.ok_or("--description is required")?;
    let device_model = device_model.ok_or("--device-model is required")?;

    let surveillance_type = SurveillanceType::from_str(&data_type_str)
        .map_err(|e| format!("Invalid surveillance type: {e}"))?;

    let device_info = DeviceInfo {
        device_model,
        serial_number,
        software_version,
        manufacturing_date: None,
        lot_number: None,
        installation_date: None,
        location: None,
    };

    let surveillance_data = manager.add_surveillance_data(
        &risk_id,
        surveillance_type,
        &source,
        &description,
        device_info,
    ).map_err(|e| format!("Failed to add surveillance data: {e}"))?;

    println!("✅ Surveillance data added successfully");
    println!("   📊 ID: {}", surveillance_data.id);
    println!("   🎯 Risk ID: {}", surveillance_data.risk_id);
    println!("   📋 Type: {}", surveillance_data.data_type.display_name());
    println!("   📍 Source: {}", surveillance_data.source);
    println!("   📅 Date: {}", surveillance_data.date_reported);

    Ok(())
}

/// Update frequency data for risk estimates
fn handle_update_frequency(manager: &mut SurveillanceManager, args: &[String]) -> Result<(), String> {
    use crate::modules::risk_manager::surveillance::FrequencyData;
    
    if args.len() < 8 {
        eprintln!("❌ Missing required arguments for update-frequency");
        println!();
        println!("USAGE:");
        println!("    qms risk surveillance update-frequency --id <surveillance-id> --numerator <n> --denominator <d> --period <period>");
        println!();
        println!("EXAMPLE:");
        println!("    qms risk surveillance update-frequency --id SUR-001 --numerator 3 --denominator 1000 --period \"6 months\"");
        return Err("Insufficient arguments".to_string());
    }

    let mut surveillance_id = None;
    let mut numerator = None;
    let mut denominator = None;
    let mut time_period = None;
    let mut data_source = "Field data".to_string();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--id" => {
                if i + 1 < args.len() {
                    surveillance_id = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--id requires a value".to_string());
                }
            }
            "--numerator" => {
                if i + 1 < args.len() {
                    numerator = Some(args[i + 1].parse::<u32>().map_err(|_| "Invalid numerator")?);
                    i += 2;
                } else {
                    return Err("--numerator requires a value".to_string());
                }
            }
            "--denominator" => {
                if i + 1 < args.len() {
                    denominator = Some(args[i + 1].parse::<u32>().map_err(|_| "Invalid denominator")?);
                    i += 2;
                } else {
                    return Err("--denominator requires a value".to_string());
                }
            }
            "--period" => {
                if i + 1 < args.len() {
                    time_period = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--period requires a value".to_string());
                }
            }
            "--source" => {
                if i + 1 < args.len() {
                    data_source = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--source requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }

    let surveillance_id = surveillance_id.ok_or("--id is required")?;
    let numerator = numerator.ok_or("--numerator is required")?;
    let denominator = denominator.ok_or("--denominator is required")?;
    let time_period = time_period.ok_or("--period is required")?;

    let frequency_data = FrequencyData {
        numerator,
        denominator,
        time_period: time_period.clone(),
        confidence_level: Some(0.95),
        data_source,
    };

    manager.update_risk_estimates(&surveillance_id, frequency_data)
        .map_err(|e| format!("Failed to update risk estimates: {e}"))?;

    let rate = numerator as f64 / denominator as f64 * 100.0;
    println!("✅ Risk estimates updated from surveillance data");
    println!("   📊 Surveillance ID: {surveillance_id}");
    println!("   📈 Frequency Rate: {rate:.2}% ({numerator}/{denominator})");
    println!("   ⏱️  Time Period: {time_period}");
    println!("   ⚠️  Risk occurrence level updated based on new frequency data");

    Ok(())
}

/// Trigger risk review
fn handle_trigger_review(manager: &mut SurveillanceManager, args: &[String]) -> Result<(), String> {
    if args.len() < 4 {
        eprintln!("❌ Missing required arguments for trigger-review");
        println!();
        println!("USAGE:");
        println!("    qms risk surveillance trigger-review --id <surveillance-id> --reason <reason>");
        println!();
        println!("EXAMPLE:");
        println!("    qms risk surveillance trigger-review --id SUR-001 --reason \"Multiple similar incidents reported\"");
        return Err("Insufficient arguments".to_string());
    }

    let mut surveillance_id = None;
    let mut reason = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--id" => {
                if i + 1 < args.len() {
                    surveillance_id = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--id requires a value".to_string());
                }
            }
            "--reason" => {
                if i + 1 < args.len() {
                    reason = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--reason requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }

    let surveillance_id = surveillance_id.ok_or("--id is required")?;
    let reason = reason.ok_or("--reason is required")?;

    manager.trigger_risk_review(&surveillance_id, &reason)
        .map_err(|e| format!("Failed to trigger risk review: {e}"))?;

    println!("✅ Risk review triggered successfully");
    println!("   📊 Surveillance ID: {surveillance_id}");
    println!("   📝 Reason: {reason}");
    println!("   🚨 Risk marked for regulatory review and escalation");

    Ok(())
}

/// Add corrective action
fn handle_add_corrective_action(manager: &mut SurveillanceManager, args: &[String]) -> Result<(), String> {
    use crate::modules::risk_manager::surveillance::ActionType;
    
    if args.len() < 8 {
        eprintln!("❌ Missing required arguments for add-action");
        println!();
        println!("USAGE:");
        println!("    qms risk surveillance add-action --id <surveillance-id> --type <type> --description <desc> --responsible <person>");
        println!();
        println!("EXAMPLE:");
        println!("    qms risk surveillance add-action --id SUR-001 --type software-update --description \"Deploy alarm fix\" --responsible \"Engineering Team\"");
        return Err("Insufficient arguments".to_string());
    }

    let mut surveillance_id = None;
    let mut action_type = None;
    let mut description = None;
    let mut responsible_party = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--id" => {
                if i + 1 < args.len() {
                    surveillance_id = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--id requires a value".to_string());
                }
            }
            "--type" => {
                if i + 1 < args.len() {
                    action_type = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--type requires a value".to_string());
                }
            }
            "--description" => {
                if i + 1 < args.len() {
                    description = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--description requires a value".to_string());
                }
            }
            "--responsible" => {
                if i + 1 < args.len() {
                    responsible_party = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--responsible requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }

    let surveillance_id = surveillance_id.ok_or("--id is required")?;
    let action_type_str = action_type.ok_or("--type is required")?;
    let description = description.ok_or("--description is required")?;
    let responsible_party = responsible_party.ok_or("--responsible is required")?;

    let action_type = match action_type_str.as_str() {
        "field-correction" => ActionType::FieldCorrection,
        "software-update" => ActionType::SoftwareUpdate,
        "labeling-change" => ActionType::LabelingChange,
        "training-update" => ActionType::TrainingUpdate,
        "design-change" => ActionType::DesignChange,
        "process-improvement" => ActionType::ProcessImprovement,
        "procedure-update" => ActionType::ProcedureUpdate,
        "communication" => ActionType::Communication,
        "investigation" => ActionType::Investigation,
        other => ActionType::Other(other.to_string()),
    };

    let action = manager.add_corrective_action(
        &surveillance_id,
        action_type,
        &description,
        &responsible_party,
    ).map_err(|e| format!("Failed to add corrective action: {e}"))?;

    println!("✅ Corrective action added successfully");
    println!("   🆔 Action ID: {}", action.id);
    println!("   📊 Surveillance ID: {surveillance_id}");
    println!("   📋 Type: {:?}", action.action_type);
    println!("   📝 Description: {}", action.description);
    println!("   👤 Responsible: {}", action.responsible_party);
    println!("   📅 Initiated: {}", action.date_initiated);

    Ok(())
}

/// List surveillance data
fn handle_list_surveillance(manager: &SurveillanceManager, args: &[String]) -> Result<(), String> {
    use crate::modules::risk_manager::surveillance::{SurveillanceType, SurveillanceStatus};
    
    let mut type_filter = None;
    let mut status_filter = None;
    let mut risk_id_filter = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--type" => {
                if i + 1 < args.len() {
                    let type_str = &args[i + 1];
                    type_filter = Some(SurveillanceType::from_str(type_str)
                        .map_err(|e| format!("Invalid surveillance type: {e}"))?);
                    i += 2;
                } else {
                    return Err("--type requires a value".to_string());
                }
            }
            "--status" => {
                if i + 1 < args.len() {
                    let status_str = &args[i + 1];
                    status_filter = Some(match status_str.as_str() {
                        "reported" => SurveillanceStatus::Reported,
                        "investigating" => SurveillanceStatus::UnderInvestigation,
                        "analyzed" => SurveillanceStatus::Analyzed,
                        "risk-updated" => SurveillanceStatus::RiskUpdated,
                        "closed" => SurveillanceStatus::Closed,
                        "escalated" => SurveillanceStatus::EscalatedToRegulatory,
                        _ => return Err(format!("Invalid status: {status_str}")),
                    });
                    i += 2;
                } else {
                    return Err("--status requires a value".to_string());
                }
            }
            "--risk-id" => {
                if i + 1 < args.len() {
                    risk_id_filter = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--risk-id requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }

    let surveillance_list = manager.list_surveillance_data(type_filter, status_filter)
        .map_err(|e| format!("Failed to list surveillance data: {e}"))?;

    // Apply risk ID filter if specified
    let filtered_list: Vec<_> = if let Some(ref risk_id) = risk_id_filter {
        surveillance_list.into_iter()
            .filter(|s| s.risk_id == *risk_id)
            .collect()
    } else {
        surveillance_list
    };

    if filtered_list.is_empty() {
        println!("📊 No surveillance data found matching the specified criteria");
        return Ok(());
    }

    println!("📊 POST-MARKET SURVEILLANCE DATA");
    println!("=================================");
    println!("Total entries: {}", filtered_list.len());
    println!();

    for data in &filtered_list {
        println!("🆔 ID: {}", data.id);
        println!("🎯 Risk ID: {}", data.risk_id);
        println!("📋 Type: {}", data.data_type.display_name());
        println!("📍 Source: {}", data.source);
        println!("📅 Date: {}", data.date_reported);
        println!("🔄 Status: {:?}", data.status);
        println!("📝 Description: {}", data.description);
        println!("🔧 Device: {}", data.device_info.device_model);
        if data.risk_reassessment_required {
            println!("⚠️  RISK REASSESSMENT REQUIRED");
        }
        if !data.corrective_actions.is_empty() {
            println!("🛠️  Corrective Actions: {}", data.corrective_actions.len());
        }
        println!("---");
    }

    Ok(())
}

/// Generate surveillance report
fn handle_surveillance_report(manager: &SurveillanceManager, args: &[String]) -> Result<(), String> {
    let mut risk_id = None;
    let mut output_file = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--risk-id" => {
                if i + 1 < args.len() {
                    risk_id = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--risk-id requires a value".to_string());
                }
            }
            "--output" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--output requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }

    let report = manager.generate_surveillance_report(risk_id.as_deref())
        .map_err(|e| format!("Failed to generate surveillance report: {e}"))?;

    if let Some(output_file) = output_file {
        std::fs::write(&output_file, &report)
            .map_err(|e| format!("Failed to write report to file: {e}"))?;
        println!("✅ Surveillance report saved to: {output_file}");
    } else {
        println!("{report}");
    }

    Ok(())
}

/// Print surveillance help
fn print_surveillance_help() {
    println!("📊 QMS Risk Post-Market Surveillance Commands");
    println!("==============================================");
    println!();
    println!("Track and analyze real-world device performance data to update risk assessments");
    println!("and ensure continued safety and effectiveness per ISO 14971 requirements.");
    println!();
    println!("COMMANDS:");
    println!("    init                     Initialize surveillance system");
    println!("    add-data                 Add new surveillance data entry");
    println!("    update-frequency         Update risk frequency from field data");
    println!("    trigger-review           Trigger risk review and escalation");
    println!("    add-action               Add corrective action");
    println!("    list                     List surveillance data with filtering");
    println!("    report                   Generate surveillance summary report");
    println!();
    println!("ADD SURVEILLANCE DATA:");
    println!("    qms risk surveillance add-data --risk-id <id> --type <type> --source <source>");
    println!("                                   --description <desc> --device-model <model>");
    println!("                                   [--serial-number <sn>] [--software-version <ver>]");
    println!();
    println!("    Surveillance Types:");
    println!("      complaint              Customer complaint report");
    println!("      field-report           Field service report");
    println!("      adverse-event          Adverse event report");
    println!("      device-failure         Device failure analysis");
    println!("      usability              User interface/usability issue");
    println!("      software-bug           Software defect report");
    println!("      performance            Performance monitoring data");
    println!("      clinical               Clinical user feedback");
    println!("      regulatory             Regulatory authority report");
    println!("      study                  Post-market clinical study data");
    println!();
    println!("UPDATE FREQUENCY:");
    println!("    qms risk surveillance update-frequency --id <surveillance-id> --numerator <n>");
    println!("                                          --denominator <d> --period <period>");
    println!("                                          [--source <source>]");
    println!();
    println!("TRIGGER REVIEW:");
    println!("    qms risk surveillance trigger-review --id <surveillance-id> --reason <reason>");
    println!();
    println!("ADD CORRECTIVE ACTION:");
    println!("    qms risk surveillance add-action --id <surveillance-id> --type <type>");
    println!("                                     --description <desc> --responsible <person>");
    println!();
    println!("    Action Types:");
    println!("      field-correction       Field correction/recall");
    println!("      software-update        Software patch/update");
    println!("      labeling-change        Warning label update");
    println!("      training-update        User training enhancement");
    println!("      design-change          Device design modification");
    println!("      process-improvement    Manufacturing process change");
    println!("      procedure-update       Operating procedure update");
    println!("      communication          Safety communication");
    println!("      investigation          Root cause analysis");
    println!();
    println!("LIST SURVEILLANCE:");
    println!("    qms risk surveillance list [--type <type>] [--status <status>] [--risk-id <id>]");
    println!();
    println!("    Status Values:");
    println!("      reported               Data reported, not yet analyzed");
    println!("      investigating          Being investigated");
    println!("      analyzed               Analysis complete");
    println!("      risk-updated           Risk assessment updated");
    println!("      closed                 Investigation closed");
    println!("      escalated              Escalated to regulatory authorities");
    println!();
    println!("GENERATE REPORT:");
    println!("    qms risk surveillance report [--risk-id <id>] [--output <file>]");
    println!();
    println!("EXAMPLES:");
    println!("    # Initialize surveillance system");
    println!("    qms risk surveillance init");
    println!();
    println!("    # Add customer complaint");
    println!("    qms risk surveillance add-data --risk-id RISK-001 --type complaint");
    println!("                                   --source \"Customer Support\"");
    println!("                                   --description \"Device alarm malfunction\"");
    println!("                                   --device-model \"Model X1\"");
    println!();
    println!("    # Update risk frequency from field data");
    println!("    qms risk surveillance update-frequency --id SUR-001 --numerator 3");
    println!("                                          --denominator 1000 --period \"6 months\"");
    println!();
    println!("    # Trigger regulatory review");
    println!("    qms risk surveillance trigger-review --id SUR-001");
    println!("                                        --reason \"Multiple similar incidents\"");
    println!();
    println!("    # Add corrective action");
    println!("    qms risk surveillance add-action --id SUR-001 --type software-update");
    println!("                                     --description \"Deploy alarm fix\"");
    println!("                                     --responsible \"Engineering Team\"");
    println!();
    println!("    # List all complaints");
    println!("    qms risk surveillance list --type complaint");
    println!();
    println!("    # Generate surveillance report");
    println!("    qms risk surveillance report --output surveillance_summary.txt");
}

/// Handle risk documentation commands
/// Task 3.1.14: Risk Documentation Implementation
fn handle_risk_document(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        print_documentation_help();
        return Ok(());
    }

    let project_path = crate::utils::get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;

    let documentation_manager = DocumentationManager::new(&project_path)
        .map_err(|e| format!("Failed to create documentation manager: {e}"))?;

    match args[0].as_str() {
        "--template" => handle_generate_template(&documentation_manager, &args[1..]),
        "--summary" => handle_generate_summary(&documentation_manager, &args[1..]),
        "--fmea-table" => handle_generate_fmea_table(&documentation_manager, &args[1..]),
        "--compliance" => handle_generate_compliance_report(&documentation_manager, &args[1..]),
        "--help" | "-h" => {
            print_documentation_help();
            Ok(())
        }
        _ => {
            eprintln!("❌ Unknown documentation command: {}", args[0]);
            print_documentation_help();
            Err(format!("Unknown documentation command: {}", args[0]))
        }
    }
}

/// Handle risk traceability report generation
fn handle_risk_traceability(args: &[String]) -> Result<(), String> {
    let project_path = crate::utils::get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;

    let documentation_manager = DocumentationManager::new(&project_path)
        .map_err(|e| format!("Failed to create documentation manager: {e}"))?;

    let mut output_file = "risk_traceability.html".to_string();
    let mut format = OutputFormat::HTML;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--output" => {
                if i + 1 < args.len() {
                    output_file = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--output requires a value".to_string());
                }
            }
            "--format" => {
                if i + 1 < args.len() {
                    format = OutputFormat::from_str(&args[i + 1])
                        .map_err(|e| format!("Invalid format: {e}"))?;
                    i += 2;
                } else {
                    return Err("--format requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }

    let config = create_default_template_config()?;
    let output_path = std::path::Path::new(&output_file);

    documentation_manager.generate_traceability_report(output_path, config, format)
        .map_err(|e| format!("Failed to generate traceability report: {e}"))?;

    println!("✅ Risk traceability report generated: {output_file}");
    Ok(())
}

/// Generate document template
fn handle_generate_template(manager: &DocumentationManager, args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        eprintln!("❌ Missing template type");
        println!();
        println!("USAGE:");
        println!("    qms risk document --template <type> [--output <file>] [--format <format>]");
        println!();
        println!("TEMPLATE TYPES:");
        println!("    risk-assessment    Risk assessment template");
        println!("    fmea              FMEA template");
        println!("    risk-plan         Risk management plan template");
        println!("    effectiveness     Control effectiveness template");
        println!("    surveillance      Post-market surveillance plan template");
        return Err("Missing template type".to_string());
    }

    let template_type = TemplateType::from_str(&args[0])
        .map_err(|e| format!("Invalid template type: {e}"))?;

    let mut output_file = format!("{}.html", template_type.display_name().to_lowercase().replace(' ', "_"));
    let mut format = OutputFormat::HTML;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" => {
                if i + 1 < args.len() {
                    output_file = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--output requires a value".to_string());
                }
            }
            "--format" => {
                if i + 1 < args.len() {
                    format = OutputFormat::from_str(&args[i + 1])
                        .map_err(|e| format!("Invalid format: {e}"))?;
                    i += 2;
                } else {
                    return Err("--format requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }

    let config = create_default_template_config()?;
    let output_path = std::path::Path::new(&output_file);
    
    let template_name = template_type.display_name().to_string(); // Get name before move

    manager.generate_template(template_type, output_path, config, format)
        .map_err(|e| format!("Failed to generate template: {e}"))?;

    println!("✅ {template_name} template generated: {output_file}");
    Ok(())
}

/// Generate risk summary report
fn handle_generate_summary(manager: &DocumentationManager, args: &[String]) -> Result<(), String> {
    let mut output_file = "risk_summary.html".to_string();
    let mut format = OutputFormat::HTML;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--output" => {
                if i + 1 < args.len() {
                    output_file = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--output requires a value".to_string());
                }
            }
            "--format" => {
                if i + 1 < args.len() {
                    format = OutputFormat::from_str(&args[i + 1])
                        .map_err(|e| format!("Invalid format: {e}"))?;
                    i += 2;
                } else {
                    return Err("--format requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }

    let config = create_default_template_config()?;
    let output_path = std::path::Path::new(&output_file);

    manager.generate_risk_summary_report(output_path, config, format)
        .map_err(|e| format!("Failed to generate risk summary: {e}"))?;

    println!("✅ Risk summary report generated: {output_file}");
    Ok(())
}

/// Generate FMEA table report
fn handle_generate_fmea_table(manager: &DocumentationManager, args: &[String]) -> Result<(), String> {
    let mut output_file = "fmea_table.html".to_string();
    let mut format = OutputFormat::HTML;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--output" => {
                if i + 1 < args.len() {
                    output_file = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--output requires a value".to_string());
                }
            }
            "--format" => {
                if i + 1 < args.len() {
                    format = OutputFormat::from_str(&args[i + 1])
                        .map_err(|e| format!("Invalid format: {e}"))?;
                    i += 2;
                } else {
                    return Err("--format requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }

    let config = create_default_template_config()?;
    let output_path = std::path::Path::new(&output_file);

    manager.generate_fmea_table(output_path, config, format)
        .map_err(|e| format!("Failed to generate FMEA table: {e}"))?;

    println!("✅ FMEA table generated: {output_file}");
    Ok(())
}

/// Generate compliance report
fn handle_generate_compliance_report(manager: &DocumentationManager, args: &[String]) -> Result<(), String> {
    let mut output_file = "compliance_report.html".to_string();
    let mut format = OutputFormat::HTML;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--output" => {
                if i + 1 < args.len() {
                    output_file = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--output requires a value".to_string());
                }
            }
            "--format" => {
                if i + 1 < args.len() {
                    format = OutputFormat::from_str(&args[i + 1])
                        .map_err(|e| format!("Invalid format: {e}"))?;
                    i += 2;
                } else {
                    return Err("--format requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }

    let config = create_default_template_config()?;
    let output_path = std::path::Path::new(&output_file);

    manager.generate_compliance_report(output_path, config, format)
        .map_err(|e| format!("Failed to generate compliance report: {e}"))?;

    println!("✅ Compliance report generated: {output_file}");
    Ok(())
}

/// Create default template configuration
fn create_default_template_config() -> Result<TemplateConfig, String> {
    Ok(TemplateConfig {
        title: "Risk Management Documentation".to_string(),
        version: "1.0".to_string(),
        author: "QMS Risk Management System".to_string(),
        organization: "Medical Device Company".to_string(),
        device_name: "Medical Device".to_string(),
        device_version: "1.0".to_string(),
        date_generated: crate::utils::current_iso8601_timestamp(),
        regulatory_basis: vec![
            "ISO 14971:2019".to_string(),
            "ISO 13485:2016".to_string(),
            "FDA 21 CFR Part 820".to_string(),
        ],
    })
}

/// Print documentation help
fn print_documentation_help() {
    println!("📄 QMS Risk Documentation Commands");
    println!("===================================");
    println!();
    println!("Generate standardized risk management documentation templates and reports");
    println!("required for regulatory compliance per ISO 14971 requirements.");
    println!();
    println!("COMMANDS:");
    println!("    --template <type>        Generate document template");
    println!("    --summary               Generate risk summary report");
    println!("    --fmea-table            Generate FMEA analysis table");
    println!("    --compliance            Generate ISO 14971 compliance report");
    println!();
    println!("TEMPLATE GENERATION:");
    println!("    qms risk document --template <type> [--output <file>] [--format <format>]");
    println!();
    println!("    Template Types:");
    println!("      risk-assessment       Risk assessment template per ISO 14971");
    println!("      fmea                  Failure Mode and Effects Analysis template");
    println!("      risk-plan             Risk management plan template");
    println!("      effectiveness         Risk control effectiveness assessment");
    println!("      surveillance          Post-market surveillance plan template");
    println!();
    println!("REPORT GENERATION:");
    println!("    qms risk document --summary [--output <file>] [--format <format>]");
    println!("    qms risk document --fmea-table [--output <file>] [--format <format>]");
    println!("    qms risk document --compliance [--output <file>] [--format <format>]");
    println!();
    println!("TRACEABILITY:");
    println!("    qms risk trace-to-requirements [--output <file>] [--format <format>]");
    println!();
    println!("OUTPUT FORMATS:");
    println!("    html                  HTML format (default) - web viewing and conversion");
    println!("    markdown              Markdown format - documentation systems");
    println!("    csv                   CSV format - data analysis (reports only)");
    println!("    json                  JSON format - data exchange (reports only)");
    println!();
    println!("EXAMPLES:");
    println!("    # Generate risk assessment template");
    println!("    qms risk document --template risk-assessment --output risk_assessment.html");
    println!();
    println!("    # Generate FMEA template");
    println!("    qms risk document --template fmea --output fmea_template.html");
    println!();
    println!("    # Generate risk management plan");
    println!("    qms risk document --template risk-plan --format markdown --output plan.md");
    println!();
    println!("    # Generate risk summary report");
    println!("    qms risk document --summary --output summary.html");
    println!();
    println!("    # Generate FMEA table from current risks");
    println!("    qms risk document --fmea-table --format csv --output fmea_analysis.csv");
    println!();
    println!("    # Generate compliance checklist");
    println!("    qms risk document --compliance --output compliance.html");
    println!();
    println!("    # Generate traceability matrix");
    println!("    qms risk trace-to-requirements --format csv --output traceability.csv");
    println!();
    println!("REGULATORY USE:");
    println!("    Generated templates and reports are designed for:");
    println!("    • FDA 510(k) submissions");
    println!("    • ISO 13485 quality management system documentation");
    println!("    • EU MDR technical documentation");
    println!("    • Risk management file compilation");
    println!("    • Design control documentation");
    println!();
    println!("    NOTE: Templates should be reviewed and completed by qualified");
    println!("    personnel before regulatory submission.");
}

/// Handle risk categorization command - Task 3.1.15
fn handle_risk_categorize(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        println!("Error: Risk ID required");
        println!("Usage: qms risk categorize <risk_id> --category <category> [--subcategory <subcategory>] [--component <component>] [--use-case <use_case>] [--user-type <user_type>] [--environment <environment>] [--rationale <rationale>]");
        return Ok(());
    }
    
    let risk_id = &args[0];
    let mut category: Option<RiskCategory> = None;
    let mut subcategory: Option<String> = None;
    let mut classifications = Vec::new();
    let mut rationale = "Risk categorization per ISO 14971".to_string();
    
    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--category" => {
                if i + 1 < args.len() {
                    category = Some(RiskCategory::from_str(&args[i + 1])?);
                    i += 2;
                } else {
                    return Err("--category requires a value".to_string());
                }
            }
            "--subcategory" => {
                if i + 1 < args.len() {
                    subcategory = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--subcategory requires a value".to_string());
                }
            }
            "--component" => {
                if i + 1 < args.len() {
                    classifications.push(ClassificationDimension::Component(args[i + 1].clone()));
                    i += 2;
                } else {
                    return Err("--component requires a value".to_string());
                }
            }
            "--use-case" => {
                if i + 1 < args.len() {
                    classifications.push(ClassificationDimension::UseCase(args[i + 1].clone()));
                    i += 2;
                } else {
                    return Err("--use-case requires a value".to_string());
                }
            }
            "--user-type" => {
                if i + 1 < args.len() {
                    classifications.push(ClassificationDimension::UserType(args[i + 1].clone()));
                    i += 2;
                } else {
                    return Err("--user-type requires a value".to_string());
                }
            }
            "--environment" => {
                if i + 1 < args.len() {
                    classifications.push(ClassificationDimension::Environment(args[i + 1].clone()));
                    i += 2;
                } else {
                    return Err("--environment requires a value".to_string());
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
    
    let category = category.ok_or("--category is required")?;
    
    // Initialize categorization manager
    let mut categorization_manager = RiskCategorizationManager::new();
    
    // Categorize the risk
    categorization_manager.categorize_risk(
        risk_id,
        category.clone(),
        subcategory.as_deref(),
        classifications,
        "cli_user", // TODO: Get actual user from session
        &rationale,
    )?;
    
    println!("✅ Risk {} successfully categorized as {}", risk_id, category.as_str());
    
    if let Some(sub) = subcategory {
        println!("   Subcategory: {sub}");
    }
    
    if let Some(categorization) = categorization_manager.get_categorization(risk_id) {
        if !categorization.classifications.is_empty() {
            println!("   Classifications:");
            for classification in &categorization.classifications {
                println!("     • {}: {}", classification.dimension_type(), classification.value());
            }
        }
    }
    
    println!("   Rationale: {rationale}");
    
    Ok(())
}

/// Handle filter by category command - Task 3.1.15  
fn handle_filter_by_category(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        println!("Error: Category required");
        println!("Usage: qms risk filter-category <category>");
        println!("Categories: safety, security, performance, usability, environmental");
        return Ok(());
    }
    
    let category = RiskCategory::from_str(&args[0])?;
    let categorization_manager = RiskCategorizationManager::new();
    
    let risk_ids = categorization_manager.filter_by_category(&category);
    
    if risk_ids.is_empty() {
        println!("No risks found in category: {}", category.as_str());
        return Ok(());
    }
    
    println!("Risks in category '{}':", category.as_str());
    println!("════════════════════════════════════════");
    
    let risk_manager = match RiskManager::new(std::path::Path::new(".")) {
        Ok(rm) => rm,
        Err(e) => {
            eprintln!("Error initializing risk manager: {e}");
            return Err(format!("Risk manager initialization failed: {e}"));
        }
    };
    
    for risk_id in risk_ids {
        if let Ok(risk) = risk_manager.load_risk(&risk_id) {
            println!("🔴 {} - {}", risk_id, risk.hazard_description);
            println!("   RPN: {} | Status: {:?}", risk.risk_priority_number, risk.risk_status);
            
            if let Some(categorization) = categorization_manager.get_categorization(&risk_id) {
                if let Some(subcategory) = &categorization.subcategory {
                    println!("   Subcategory: {}", subcategory.name);
                }
                if !categorization.classifications.is_empty() {
                    print!("   Classifications: ");
                    let class_strs: Vec<String> = categorization.classifications.iter()
                        .map(|c| format!("{}:{}", c.dimension_type(), c.value()))
                        .collect();
                    println!("{}", class_strs.join(", "));
                }
            }
            println!();
        }
    }
    
    Ok(())
}

/// Handle category statistics command - Task 3.1.15
fn handle_category_statistics(args: &[String]) -> Result<(), String> {
    let categorization_manager = RiskCategorizationManager::new();
    let risk_manager = match RiskManager::new(std::path::Path::new(".")) {
        Ok(rm) => rm,
        Err(e) => {
            eprintln!("Error initializing risk manager: {e}");
            return Err(format!("Risk manager initialization failed: {e}"));
        }
    };
    
    let stats = categorization_manager.category_statistics(&risk_manager);
    
    println!("Risk Category Statistics");
    println!("════════════════════════════════════════");
    println!("Total Categorized Risks: {}", stats.total_risks);
    println!();
    
    if stats.total_risks == 0 {
        println!("No categorized risks found.");
        println!("Use 'qms risk categorize' to categorize risks.");
        return Ok(());
    }
    
    // Category distribution
    println!("📊 Category Distribution:");
    println!("────────────────────────");
    for (category, count) in &stats.category_counts {
        let percentage = stats.coverage_analysis.get(category).unwrap_or(&0.0);
        println!("  {category:13} │ {count:3} risks ({percentage:5.1}%)");
    }
    println!();
    
    // High-risk categories
    if !stats.high_risk_categories.is_empty() {
        println!("⚠️  High-Risk Categories (RPN > 125):");
        println!("─────────────────────────────────────");
        for (category, count) in &stats.high_risk_categories {
            println!("  {category:13} │ {count:3} high-risk items");
        }
        println!();
    }
    
    // Classification breakdown
    println!("🔍 Classification Breakdown:");
    println!("────────────────────────────");
    for (dimension, values) in &stats.classification_counts {
        println!("  {}:", dimension.replace('_', " ").to_uppercase());
        for (value, count) in values {
            println!("    {value:15} │ {count:3} risks");
        }
        println!();
    }
    
    // Generate detailed report option
    if args.iter().any(|arg| arg == "--report") {
        let report = categorization_manager.generate_categorization_report(&risk_manager);
        let filename = format!("risk_categorization_report_{}.md", 
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs());
        
        if let Err(e) = std::fs::write(&filename, report) {
            eprintln!("Error writing report: {e}");
        } else {
            println!("📄 Detailed report saved to: {filename}");
        }
    } else {
        println!("💡 Tip: Use --report flag to generate a detailed markdown report");
    }
    
    Ok(())
}

// Task 3.1.17: Risk Communication Implementation
fn handle_risk_notify(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        eprintln!("Error: Missing required arguments for risk notify");
        print_risk_notify_help();
        return Err("Missing arguments".to_string());
    }

    let project_path = crate::utils::get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;
    
    let mut communication_manager = RiskCommunicationManager::new(&project_path.to_string_lossy())
        .map_err(|e| format!("Failed to initialize communication manager: {e}"))?;

    // Parse arguments
    let mut stakeholders = Vec::new();
    let mut risk_id = None;
    let mut message = None;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--stakeholders" => {
                if i + 1 < args.len() {
                    let stakeholder_list = &args[i + 1];
                    for stakeholder_str in stakeholder_list.split(',') {
                        if let Some(stakeholder) = StakeholderType::from_string(stakeholder_str.trim()) {
                            stakeholders.push(stakeholder);
                        } else {
                            eprintln!("Warning: Unknown stakeholder type: {stakeholder_str}");
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("Error: --stakeholders requires a value");
                    return Err("Missing stakeholders value".to_string());
                }
            }
            "--risk-id" => {
                if i + 1 < args.len() {
                    risk_id = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --risk-id requires a value");
                    return Err("Missing risk-id value".to_string());
                }
            }
            "--message" => {
                if i + 1 < args.len() {
                    message = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --message requires a value");
                    return Err("Missing message value".to_string());
                }
            }
            _ => {
                // If no flag, assume it's the risk ID
                if risk_id.is_none() {
                    risk_id = Some(args[i].clone());
                }
                i += 1;
            }
        }
    }

    // Validate required arguments
    let risk_id = risk_id.ok_or("Missing risk ID")?;
    let message = message.ok_or("Missing message")?;
    
    if stakeholders.is_empty() {
        // Default to quality engineer if no stakeholders specified
        stakeholders.push(StakeholderType::QualityEngineer);
    }

    // Send notification
    communication_manager.notify_stakeholders(&risk_id, stakeholders, &message)
        .map_err(|e| format!("Failed to send notification: {e}"))?;

    Ok(())
}

fn handle_risk_alerts(_args: &[String]) -> Result<(), String> {
    let project_path = crate::utils::get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;
    
    let mut communication_manager = RiskCommunicationManager::new(&project_path.to_string_lossy())
        .map_err(|e| format!("Failed to initialize communication manager: {e}"))?;
    
    let mut risk_manager = RiskManager::new(&project_path)
        .map_err(|e| format!("Failed to initialize risk manager: {e}"))?;

    // Generate automated alerts
    let alerts = communication_manager.generate_risk_alerts(&mut risk_manager)
        .map_err(|e| format!("Failed to generate alerts: {e}"))?;

    println!("🚨 Risk Alerts Generated");
    println!("═══════════════════════");
    
    if alerts.is_empty() {
        println!("✅ No active risk alerts");
    } else {
        for alert in alerts {
            println!("{} {} - {}", 
                alert.priority.emoji(),
                alert.alert_type.to_string(),
                alert.message
            );
            println!("   Risk ID: {}", alert.risk_id);
            println!("   Stakeholders: {}", 
                alert.stakeholders.iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            println!();
        }
    }

    Ok(())
}

fn handle_risk_communications(args: &[String]) -> Result<(), String> {
    let project_path = crate::utils::get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;
    
    let communication_manager = RiskCommunicationManager::new(&project_path.to_string_lossy())
        .map_err(|e| format!("Failed to initialize communication manager: {e}"))?;

    // Parse filter options
    let mut filter_stakeholder = None;
    let mut show_acknowledged = false;
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--stakeholder" => {
                if i + 1 < args.len() {
                    filter_stakeholder = StakeholderType::from_string(&args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("Error: --stakeholder requires a value");
                    return Err("Missing stakeholder value".to_string());
                }
            }
            "--show-acknowledged" => {
                show_acknowledged = true;
                i += 1;
            }
            _ => {
                eprintln!("Warning: Unknown argument: {}", args[i]);
                i += 1;
            }
        }
    }

    // List communications
    let communications = communication_manager.list_communications(filter_stakeholder);
    
    println!("📢 Risk Communications");
    println!("══════════════════════");
    
    if communications.is_empty() {
        println!("No communications found");
    } else {
        for comm in communications {
            if !show_acknowledged && comm.acknowledged {
                continue;
            }
            
            println!("{} {} [{}] - {}", 
                comm.priority.emoji(),
                comm.alert_type.to_string(),
                comm.priority.to_string(),
                comm.message
            );
            println!("   ID: {} | Risk: {} | Time: {}", 
                comm.id,
                comm.risk_id,
                crate::utils::format_timestamp(comm.timestamp)
            );
            println!("   Stakeholders: {}", 
                comm.stakeholders.iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            if comm.acknowledged {
                println!("   ✅ Acknowledged");
            }
            println!();
        }
    }

    Ok(())
}

fn handle_risk_acknowledge(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        eprintln!("Error: Missing communication ID");
        eprintln!("Usage: qms risk acknowledge <communication-id>");
        return Err("Missing communication ID".to_string());
    }

    let project_path = crate::utils::get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;
    
    let mut communication_manager = RiskCommunicationManager::new(&project_path.to_string_lossy())
        .map_err(|e| format!("Failed to initialize communication manager: {e}"))?;

    let comm_id = &args[0];
    
    communication_manager.acknowledge_communication(comm_id)
        .map_err(|e| format!("Failed to acknowledge communication: {e}"))?;

    Ok(())
}

fn handle_executive_summary(args: &[String]) -> Result<(), String> {
    let project_path = crate::utils::get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;
    
    let communication_manager = RiskCommunicationManager::new(&project_path.to_string_lossy())
        .map_err(|e| format!("Failed to initialize communication manager: {e}"))?;
    
    let risk_manager = RiskManager::new(&project_path)
        .map_err(|e| format!("Failed to initialize risk manager: {e}"))?;

    let summary = communication_manager.generate_executive_summary(&risk_manager)
        .map_err(|e| format!("Failed to generate executive summary: {e}"))?;

    // Check for output file option
    let mut output_file = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--output" => {
                if i + 1 < args.len() {
                    output_file = Some(&args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("Error: --output requires a filename");
                    return Err("Missing output filename".to_string());
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    if let Some(filename) = output_file {
        std::fs::write(filename, &summary)
            .map_err(|e| format!("Failed to write summary to file: {e}"))?;
        println!("📄 Executive summary saved to: {filename}");
    } else {
        println!("{summary}");
    }

    Ok(())
}

fn handle_technical_report(args: &[String]) -> Result<(), String> {
    let project_path = crate::utils::get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;
    
    let communication_manager = RiskCommunicationManager::new(&project_path.to_string_lossy())
        .map_err(|e| format!("Failed to initialize communication manager: {e}"))?;
    
    let risk_manager = RiskManager::new(&project_path)
        .map_err(|e| format!("Failed to initialize risk manager: {e}"))?;

    let report = communication_manager.generate_technical_report(&risk_manager)
        .map_err(|e| format!("Failed to generate technical report: {e}"))?;

    // Check for output file option
    let mut output_file = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--output" => {
                if i + 1 < args.len() {
                    output_file = Some(&args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("Error: --output requires a filename");
                    return Err("Missing output filename".to_string());
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    if let Some(filename) = output_file {
        std::fs::write(filename, &report)
            .map_err(|e| format!("Failed to write report to file: {e}"))?;
        println!("📄 Technical report saved to: {filename}");
    } else {
        println!("{report}");
    }

    Ok(())
}

fn print_risk_notify_help() {
    println!("📢 Risk Notification Help");
    println!("═════════════════════════");
    println!("Send risk notifications to stakeholders");
    println!();
    println!("USAGE:");
    println!("  qms risk notify <risk-id> --stakeholders <list> --message <message>");
    println!("  qms risk notify --risk-id <id> --stakeholders <list> --message <message>");
    println!();
    println!("OPTIONS:");
    println!("  --risk-id <id>           Risk ID to notify about");
    println!("  --stakeholders <list>    Comma-separated list of stakeholders");
    println!("  --message <message>      Notification message");
    println!();
    println!("STAKEHOLDER TYPES:");
    println!("  qe, quality, quality-engineer    Quality Engineer");
    println!("  pm, product, product-manager     Product Manager");
    println!("  ra, regulatory, regulatory-affairs   Regulatory Affairs");
    println!("  exec, executive, management      Executive Management");
    println!("  dev, development, development-team   Development Team");
    println!("  qa, quality-assurance            Quality Assurance");
    println!();
    println!("EXAMPLES:");
    println!("  qms risk notify HAZ-001 --stakeholders qe,pm --message \"High risk identified\"");
    println!("  qms risk notify --risk-id HAZ-002 --stakeholders ra --message \"Approval required\"");
}

// Include approval command handlers from separate file

// Task 3.1.18: Risk Performance Metrics Implementation

fn handle_risk_metrics(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        print_risk_metrics_help();
        return Ok(());
    }

    let current_path = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?;
    
    let mut period = MetricsPeriod::All;
    let mut output_file: Option<String> = None;
    let mut report_format = "text";
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "--period" => {
                if i + 1 < args.len() {
                    period = parse_metrics_period(&args[i + 1])?;
                    i += 2;
                } else {
                    return Err("Missing period value".to_string());
                }
            }
            "--output" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing output filename".to_string());
                }
            }
            "--format" => {
                if i + 1 < args.len() {
                    report_format = &args[i + 1];
                    i += 2;
                } else {
                    return Err("Missing format value".to_string());
                }
            }
            "--help" | "-h" => {
                print_risk_metrics_help();
                return Ok(());
            }
            _ => {
                i += 1;
            }
        }
    }

    let metrics_manager = RiskMetricsManager::new(&current_path)
        .map_err(|e| format!("Failed to create metrics manager: {e}"))?;

    match report_format {
        "csv" => {
            let output_path = output_file.as_ref()
                .map(|f| current_path.join(f))
                .unwrap_or_else(|| current_path.join("risk_metrics.csv"));
            
            metrics_manager.export_metrics_csv(&period, &output_path)
                .map_err(|e| format!("Failed to export CSV: {e}"))?;
            
            println!("📊 Risk metrics exported to: {}", output_path.display());
        }
        "text" | "markdown" => {
            let report = metrics_manager.generate_metrics_report(&period)
                .map_err(|e| format!("Failed to generate report: {e}"))?;
            
            if let Some(filename) = output_file {
                std::fs::write(&filename, &report)
                    .map_err(|e| format!("Failed to write report: {e}"))?;
                println!("📄 Risk metrics report saved to: {filename}");
            } else {
                println!("{report}");
            }
        }
        _ => {
            return Err(format!("Unsupported format: {report_format}"));
        }
    }

    Ok(())
}

fn handle_risk_kpis(args: &[String]) -> Result<(), String> {
    if !args.is_empty() && (args[0] == "--help" || args[0] == "-h") {
        print_risk_kpis_help();
        return Ok(());
    }

    let current_path = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?;
    
    let mut period = MetricsPeriod::All;
    let mut detailed = false;
    let mut dashboard = false;
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "--period" => {
                if i + 1 < args.len() {
                    period = parse_metrics_period(&args[i + 1])?;
                    i += 2;
                } else {
                    return Err("Missing period value".to_string());
                }
            }
            "--detailed" => {
                detailed = true;
                i += 1;
            }
            "--dashboard" => {
                dashboard = true;
                i += 1;
            }
            "--help" | "-h" => {
                print_risk_kpis_help();
                return Ok(());
            }
            _ => {
                i += 1;
            }
        }
    }

    let metrics_manager = RiskMetricsManager::new(&current_path)
        .map_err(|e| format!("Failed to create metrics manager: {e}"))?;

    if dashboard {
        let dashboard_data = metrics_manager.get_dashboard_data(&period)
            .map_err(|e| format!("Failed to get dashboard data: {e}"))?;
        
        display_risk_dashboard(&dashboard_data);
    } else {
        let kpis = metrics_manager.calculate_kpis(&period)
            .map_err(|e| format!("Failed to calculate KPIs: {e}"))?;
        
        display_risk_kpis(&kpis, detailed);
    }

    Ok(())
}

fn parse_metrics_period(period_str: &str) -> Result<MetricsPeriod, String> {
    if period_str.ends_with("d") || period_str.ends_with("days") {
        let num_str = period_str.trim_end_matches("d").trim_end_matches("ays");
        let days = num_str.parse::<u32>()
            .map_err(|_| format!("Invalid days value: {num_str}"))?;
        Ok(MetricsPeriod::Days(days))
    } else if period_str.ends_with("w") || period_str.ends_with("weeks") {
        let num_str = period_str.trim_end_matches("w").trim_end_matches("eeks");
        let weeks = num_str.parse::<u32>()
            .map_err(|_| format!("Invalid weeks value: {num_str}"))?;
        Ok(MetricsPeriod::Weeks(weeks))
    } else if period_str.ends_with("m") || period_str.ends_with("months") {
        let num_str = period_str.trim_end_matches("m").trim_end_matches("onths");
        let months = num_str.parse::<u32>()
            .map_err(|_| format!("Invalid months value: {num_str}"))?;
        Ok(MetricsPeriod::Months(months))
    } else if period_str == "all" {
        Ok(MetricsPeriod::All)
    } else {
        Err(format!("Invalid period format: {period_str}. Use format like '30d', '4w', '3m', or 'all'"))
    }
}

fn display_risk_kpis(kpis: &RiskKPIs, detailed: bool) {
    println!("📊 Risk Management KPIs");
    println!("═══════════════════════");
    println!();
    
    // Core KPIs
    println!("🎯 Core Performance Indicators:");
    println!("  • Total Risks: {}", kpis.total_risks);
    println!("  • Risk Closure Rate: {:.1}%", kpis.closure_rate);
    println!("  • Average RPN: {:.1}", kpis.average_rpn);
    println!("  • Average Closure Time: {:.1} days", kpis.average_closure_time);
    println!("  • Verified Mitigations: {:.1}%", kpis.verified_mitigations_percentage);
    println!("  • Mitigation Effectiveness: {:.1}%", kpis.mitigation_effectiveness);
    println!();
    
    // Risk Indicators
    println!("🚨 Risk Indicators:");
    println!("  • High Risk Count (RPN > 50): {}", kpis.high_risk_count);
    println!("  • Overdue Risks: {}", kpis.overdue_risks_count);
    println!("  • Health Score: {:.1}/100", kpis.trend_indicators.health_score);
    println!();
    
    if detailed {
        // Status Distribution
        println!("📋 Status Distribution:");
        for (status, count) in &kpis.status_distribution {
            let percentage = (*count as f64 / kpis.total_risks as f64) * 100.0;
            println!("  • {status:?}: {count} ({percentage:.1}%)");
        }
        println!();
        
        // Severity Distribution
        println!("⚠️ Severity Distribution:");
        for (severity, count) in &kpis.severity_distribution {
            let percentage = (*count as f64 / kpis.total_risks as f64) * 100.0;
            println!("  • {severity:?}: {count} ({percentage:.1}%)");
        }
        println!();
        
        // Trend Analysis
        println!("📈 Trend Analysis:");
        println!("  • Risk Creation Trend: {:.1} risks/month", kpis.trend_indicators.risk_creation_trend);
        println!("  • Risk Closure Trend: {:.1} risks/month", kpis.trend_indicators.risk_closure_trend);
        println!("  • RPN Trend: {:.1} (average change)", kpis.trend_indicators.rpn_trend);
        println!("  • Mitigation Trend: {:.1}%", kpis.trend_indicators.mitigation_trend);
        println!();
    }
    
    // Health Score Interpretation
    let health_emoji = if kpis.trend_indicators.health_score >= 80.0 {
        "💚"
    } else if kpis.trend_indicators.health_score >= 60.0 {
        "🟡"
    } else {
        "🔴"
    };
    
    println!("{} Overall Health: {:.1}/100", health_emoji, kpis.trend_indicators.health_score);
}

fn display_risk_dashboard(dashboard: &RiskDashboard) {
    println!("🎛️ Risk Management Dashboard");
    println!("═══════════════════════════");
    println!();
    
    // Key Metrics
    display_risk_kpis(&dashboard.kpis, false);
    
    // Recent Risks
    if !dashboard.recent_risks.is_empty() {
        println!("🕐 Recent Risks:");
        for (i, risk) in dashboard.recent_risks.iter().enumerate() {
            if i < 5 {
                println!("  • {}: {} (RPN: {})", risk.hazard_id, risk.hazard_description, risk.risk_priority_number);
            }
        }
        println!();
    }
    
    // Critical Alerts
    if !dashboard.critical_alerts.is_empty() {
        println!("🚨 Critical Alerts:");
        for alert in &dashboard.critical_alerts {
            println!("  {alert}");
        }
        println!();
    }
    
    // Activity Summary
    println!("📊 Activity Summary:");
    println!("  {}", dashboard.activity_summary);
}

fn print_risk_metrics_help() {
    println!("📊 Risk Performance Metrics Help");
    println!("═══════════════════════════════");
    println!("Generate comprehensive risk performance metrics and reports");
    println!();
    println!("USAGE:");
    println!("  qms risk metrics [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("  --period <period>        Time period for metrics (e.g., 30d, 4w, 3m, all)");
    println!("  --format <format>        Output format (text, markdown, csv)");
    println!("  --output <file>          Output file path");
    println!("  --help, -h               Show this help message");
    println!();
    println!("PERIOD FORMATS:");
    println!("  30d, 30days              Last 30 days");
    println!("  4w, 4weeks               Last 4 weeks");
    println!("  3m, 3months              Last 3 months");
    println!("  all                      All time");
    println!();
    println!("EXAMPLES:");
    println!("  qms risk metrics                     # All-time metrics");
    println!("  qms risk metrics --period 30d        # Last 30 days");
    println!("  qms risk metrics --format csv --output metrics.csv");
    println!("  qms risk metrics --period 3m --format markdown --output report.md");
}

fn print_risk_kpis_help() {
    println!("🎯 Risk Key Performance Indicators Help");
    println!("═══════════════════════════════════════");
    println!("Display key performance indicators for risk management");
    println!();
    println!("USAGE:");
    println!("  qms risk kpis [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("  --period <period>        Time period for KPIs (e.g., 30d, 4w, 3m, all)");
    println!("  --detailed               Show detailed breakdown");
    println!("  --dashboard              Show dashboard view");
    println!("  --help, -h               Show this help message");
    println!();
    println!("KPI CATEGORIES:");
    println!("  • Core Performance: Closure rate, average RPN, closure time");
    println!("  • Risk Indicators: High-risk count, overdue risks, health score");
    println!("  • Distributions: Status and severity breakdowns");
    println!("  • Trends: Creation, closure, and effectiveness trends");
    println!();
    println!("EXAMPLES:");
    println!("  qms risk kpis                        # Basic KPIs");
    println!("  qms risk kpis --detailed             # Detailed breakdown");
    println!("  qms risk kpis --dashboard            # Dashboard view");
    println!("  qms risk kpis --period 30d --detailed");
}
include!("risk_approval_handlers.rs");