/// Handle risk register command
/// Task 3.1.7: Risk Register
use crate::prelude::*;
use crate::modules::risk_manager::{RiskManager, RiskRegisterFilter, RiskSeverity};

pub fn handle_risk_register(args: &[String]) -> Result<(), String> {
    let mut filter = RiskRegisterFilter::default();
    filter.sort_by = "rpn:desc".to_string(); // Default sort by RPN descending
    
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
                                return Err(format!("Unknown filter key: {}", key));
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
                         format!("{:?}", risk.verification_status)
                );
            }
            
            println!("\nUse 'qms risk view <risk-id>' for detailed information");
            println!("Use 'qms risk export-register' to export this data");
        }
        Err(e) => {
            return Err(format!("Failed to get risk register: {}", e));
        }
    }
    
    Ok(())
}

/// Handle risk statistics command
/// Task 3.1.7: Risk Register
pub fn handle_risk_stats(_args: &[String]) -> Result<(), String> {
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
                println!("  {}: {}", severity, count);
            }
            
            println!("\n🔍 Status Distribution:");
            for (status, count) in &stats.status_distribution {
                println!("  {}: {}", status, count);
            }
        }
        Err(e) => {
            return Err(format!("Failed to get risk statistics: {}", e));
        }
    }
    
    Ok(())
}

/// Handle export risk register command
/// Task 3.1.7: Risk Register
pub fn handle_export_risk_register(args: &[String]) -> Result<(), String> {
    let mut filter = RiskRegisterFilter::default();
    filter.sort_by = "rpn:desc".to_string();
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
                                return Err(format!("Unknown filter key: {}", key));
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
            return Err(format!("Failed to export risk register: {}", e));
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