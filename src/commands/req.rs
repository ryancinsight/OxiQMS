/*
 * QMS (Quality Management System)
 * Requirements Command Handler - Task 3.2.1
 * 
 * CLI command handlers for requirement management
 * 
 * Author: QMS Development Team
 * Date: January 2025
 * Version: 1.0.0
 */

use crate::modules::traceability::requirement::{RequirementManager, RequirementCategory, RequirementPriority, RequirementStatus, VerificationMethod, RequirementUpdate};

pub fn handle_req_command(args: &[String]) -> Result<(), String> {
    if args.len() < 3 {
        print_req_help();
        return Ok(());
    }

    match args[2].as_str() {
        "init" => handle_req_init(&args[3..]),
        "create" => handle_req_create(&args[3..]),
        "list" => handle_req_list(&args[3..]),
        "show" => handle_req_show(&args[3..]),
        "update" => handle_req_update(&args[3..]),
        "delete" => handle_req_delete(&args[3..]),
        "verify" => handle_req_verify(&args[3..]),
        "verification-report" => handle_req_verification_report(&args[3..]),
        "--help" | "-h" => {
            print_req_help();
            Ok(())
        }
        _ => {
            eprintln!("Error: Unknown requirements command '{}'", args[2]);
            print_req_help();
            Err(format!("Unknown requirements command '{}'", args[2]))
        }
    }
}

fn handle_req_init(args: &[String]) -> Result<(), String> {
    if args.len() < 3 {
        print_req_help();
        return Ok(());
    }

    match args[2].as_str() {
        "init" => handle_req_init(&args[3..]),
        "create" => handle_req_create(&args[3..]),
        "list" => handle_req_list(&args[3..]),
        "show" => handle_req_show(&args[3..]),
        "update" => handle_req_update(&args[3..]),
        "delete" => handle_req_delete(&args[3..]),
        "--help" | "-h" => {
            print_req_help();
            Ok(())
        }
        _ => {
            eprintln!("Error: Unknown requirements command '{}'", args[2]);
            print_req_help();
            Err(format!("Unknown requirements command '{}'", args[2]))
        }
    }
}

fn handle_req_create(args: &[String]) -> Result<(), String> {
    let mut req_id = String::new();
    let mut title = String::new();
    let mut description = String::new();
    let mut category = RequirementCategory::Functional;
    let mut priority = RequirementPriority::Medium;
    let mut source = String::new();
    let mut auto_id = false;
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--id" => {
                if i + 1 < args.len() {
                    req_id = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--id requires a value".to_string());
                }
            }
            "--title" => {
                if i + 1 < args.len() {
                    title = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--title requires a value".to_string());
                }
            }
            "--desc" | "--description" => {
                if i + 1 < args.len() {
                    description = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--desc requires a value".to_string());
                }
            }
            "--category" => {
                if i + 1 < args.len() {
                    category = RequirementCategory::from_str(&args[i + 1]);
                    i += 2;
                } else {
                    return Err("--category requires a value".to_string());
                }
            }
            "--priority" => {
                if i + 1 < args.len() {
                    priority = RequirementPriority::from_str(&args[i + 1]);
                    i += 2;
                } else {
                    return Err("--priority requires a value".to_string());
                }
            }
            "--source" => {
                if i + 1 < args.len() {
                    source = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--source requires a value".to_string());
                }
            }
            "--auto-id" => {
                auto_id = true;
                i += 1;
            }
            "--help" | "-h" => {
                print_req_create_help();
                return Ok(());
            }
            _ => {
                return Err(format!("Unknown option: {}", args[i]));
            }
        }
    }
    
    // Validate required fields
    if title.is_empty() {
        return Err("Title is required (--title)".to_string());
    }
    if description.is_empty() {
        return Err("Description is required (--desc)".to_string());
    }
    
    let project_path = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?;
    
    let mut manager = RequirementManager::new(&project_path)
        .map_err(|e| format!("Failed to initialize requirements system: {e}"))?;
    
    // Auto-generate ID if requested
    if auto_id || req_id.is_empty() {
        req_id = manager.generate_next_req_id();
    }
    
    // Create requirement
    let id = manager.create_requirement(
        crate::utils::user_context::get_current_project_id(), // Get actual project ID from project context
        req_id.clone(),
        title.clone(),
        description.clone(),
        category.clone(),
        crate::utils::user_context::get_current_user_id(), // Get actual user ID from context
    ).map_err(|e| format!("Failed to create requirement: {e}"))?;
    
    println!("✅ Requirement created successfully");
    println!("🆔 ID: {id}");
    println!("📝 Requirement ID: {req_id}");
    println!("📄 Title: {title}");
    println!("📊 Category: {}", category.as_str());
    println!("⭐ Priority: {}", priority.as_str());
    if !source.is_empty() {
        println!("📚 Source: {source}");
    }
    
    Ok(())
}

fn handle_req_list(args: &[String]) -> Result<(), String> {
    let mut filter_category = None;
    let mut filter_status = None;
    let mut filter_priority = None;
    let mut show_details = false;
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--category" => {
                if i + 1 < args.len() {
                    filter_category = Some(RequirementCategory::from_str(&args[i + 1]));
                    i += 2;
                } else {
                    return Err("--category requires a value".to_string());
                }
            }
            "--status" => {
                if i + 1 < args.len() {
                    filter_status = Some(RequirementStatus::from_str(&args[i + 1]));
                    i += 2;
                } else {
                    return Err("--status requires a value".to_string());
                }
            }
            "--priority" => {
                if i + 1 < args.len() {
                    filter_priority = Some(RequirementPriority::from_str(&args[i + 1]));
                    i += 2;
                } else {
                    return Err("--priority requires a value".to_string());
                }
            }
            "--details" => {
                show_details = true;
                i += 1;
            }
            "--help" | "-h" => {
                print_req_list_help();
                return Ok(());
            }
            _ => {
                return Err(format!("Unknown option: {}", args[i]));
            }
        }
    }
    
    let project_path = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?;
    
    let manager = RequirementManager::new(&project_path)
        .map_err(|e| format!("Failed to initialize requirements system: {e}"))?;
    
    let mut requirements = manager.list_requirements();
    
    // Apply filters
    if let Some(category) = &filter_category {
        requirements.retain(|r| &r.category == category);
    }
    if let Some(status) = &filter_status {
        requirements.retain(|r| &r.status == status);
    }
    if let Some(priority) = &filter_priority {
        requirements.retain(|r| &r.priority == priority);
    }
    
    // Sort by req_id
    requirements.sort_by(|a, b| a.req_id.cmp(&b.req_id));
    
    if requirements.is_empty() {
        println!("No requirements found matching the specified criteria.");
        return Ok(());
    }
    
    println!("📋 Requirements List ({} items)", requirements.len());
    println!();
    
    if show_details {
        for req in requirements {
            println!("┌─ {} - {} [{}] ({})", req.req_id, req.title, req.category.as_str(), req.status.as_str());
            println!("│  📄 Description: {}", req.description);
            println!("│  ⭐ Priority: {}", req.priority.as_str());
            println!("│  🔬 Verification: {}", req.verification_method.as_str());
            println!("│  👤 Created by: {} on {}", req.created_by, req.created_at);
            if let Some(assigned_to) = &req.assigned_to {
                println!("│  👥 Assigned to: {assigned_to}");
            }
            if !req.tags.is_empty() {
                println!("│  🏷️  Tags: {}", req.tags.join(", "));
            }
            println!("└─");
            println!();
        }
    } else {
        println!("┌─────────────┬──────────────────────────────────────────────┬────────────────┬───────────────┐");
        println!("│ Req ID      │ Title                                        │ Category       │ Status        │");
        println!("├─────────────┼──────────────────────────────────────────────┼────────────────┼───────────────┤");
        
        for req in requirements {
            let truncated_title = if req.title.len() > 44 {
                format!("{}...", &req.title[..41])
            } else {
                req.title.clone()
            };
            
            println!("│ {:<11} │ {:<44} │ {:<14} │ {:<13} │", 
                req.req_id, truncated_title, req.category.as_str(), req.status.as_str());
        }
        
        println!("└─────────────┴──────────────────────────────────────────────┴────────────────┴───────────────┘");
    }
    
    Ok(())
}

fn handle_req_show(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Requirement ID is required".to_string());
    }
    
    if args.len() > 1 && args[1] == "--help" {
        print_req_show_help();
        return Ok(());
    }
    
    let req_id = &args[0];
    
    let project_path = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?;
    
    let manager = RequirementManager::new(&project_path)
        .map_err(|e| format!("Failed to initialize requirements system: {e}"))?;
    
    let requirement = manager.get_requirement_by_req_id(req_id)
        .ok_or_else(|| format!("Requirement '{req_id}' not found"))?;
    
    println!("📋 Requirement Details");
    println!("═══════════════════════");
    println!("🆔 ID: {}", requirement.id);
    println!("📝 Requirement ID: {}", requirement.req_id);
    println!("📄 Title: {}", requirement.title);
    println!("📊 Category: {}", requirement.category.as_str());
    println!("⭐ Priority: {}", requirement.priority.as_str());
    println!("📈 Status: {}", requirement.status.as_str());
    println!("🔬 Verification Method: {}", requirement.verification_method.as_str());
    println!();
    println!("📝 Description:");
    println!("{}", requirement.description);
    println!();
    
    if !requirement.source.is_empty() {
        println!("📚 Source: {}", requirement.source);
    }
    if !requirement.rationale.is_empty() {
        println!("💡 Rationale: {}", requirement.rationale);
    }
    if !requirement.acceptance_criteria.is_empty() {
        println!("✅ Acceptance Criteria: {}", requirement.acceptance_criteria);
    }
    
    println!();
    println!("👤 Created by: {} on {}", requirement.created_by, requirement.created_at);
    println!("🔄 Updated on: {}", requirement.updated_at);
    
    if let Some(assigned_to) = &requirement.assigned_to {
        println!("👥 Assigned to: {assigned_to}");
    }
    
    if !requirement.tags.is_empty() {
        println!("🏷️  Tags: {}", requirement.tags.join(", "));
    }
    
    if !requirement.linked_requirements.is_empty() {
        println!("🔗 Linked Requirements: {}", requirement.linked_requirements.join(", "));
    }
    
    if !requirement.linked_tests.is_empty() {
        println!("🧪 Linked Tests: {}", requirement.linked_tests.join(", "));
    }
    
    if !requirement.linked_risks.is_empty() {
        println!("⚠️  Linked Risks: {}", requirement.linked_risks.join(", "));
    }
    
    if !requirement.regulatory_mapping.is_empty() {
        println!("📋 Regulatory Mapping:");
        for mapping in &requirement.regulatory_mapping {
            println!("  - {}: {}", mapping.standard, mapping.requirement);
        }
    }
    
    Ok(())
}

fn handle_req_update(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Missing requirement ID. Usage: qms req update <REQ-ID> [OPTIONS]".to_string());
    }
    
    if args.len() > 1 && args[1] == "--help" {
        print_req_update_help();
        return Ok(());
    }
    
    let req_id = &args[0];
    let mut update = RequirementUpdate::new();
    
    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--title" => {
                if i + 1 >= args.len() {
                    return Err("Missing value for --title".to_string());
                }
                update = update.title(args[i + 1].clone());
                i += 2;
            }
            "--desc" => {
                if i + 1 >= args.len() {
                    return Err("Missing value for --desc".to_string());
                }
                update = update.description(args[i + 1].clone());
                i += 2;
            }
            "--category" => {
                if i + 1 >= args.len() {
                    return Err("Missing value for --category".to_string());
                }
                let category = match args[i + 1].to_lowercase().as_str() {
                    "functional" => RequirementCategory::Functional,
                    "performance" => RequirementCategory::Performance,
                    "usability" => RequirementCategory::Usability,
                    "reliability" => RequirementCategory::Reliability,
                    "safety" => RequirementCategory::Safety,
                    "security" => RequirementCategory::Security,
                    "regulatory" => RequirementCategory::Regulatory,
                    "interface" => RequirementCategory::Interface,
                    "data" => RequirementCategory::Data,
                    "system" => RequirementCategory::System,
                    _ => return Err(format!("Invalid category: {}", args[i + 1])),
                };
                update = update.category(category);
                i += 2;
            }
            "--priority" => {
                if i + 1 >= args.len() {
                    return Err("Missing value for --priority".to_string());
                }
                let priority = match args[i + 1].to_lowercase().as_str() {
                    "critical" => RequirementPriority::Critical,
                    "high" => RequirementPriority::High,
                    "medium" => RequirementPriority::Medium,
                    "low" => RequirementPriority::Low,
                    _ => return Err(format!("Invalid priority: {}", args[i + 1])),
                };
                update = update.priority(priority);
                i += 2;
            }
            "--status" => {
                if i + 1 >= args.len() {
                    return Err("Missing value for --status".to_string());
                }
                let status = match args[i + 1].to_lowercase().as_str() {
                    "draft" => RequirementStatus::Draft,
                    "underreview" => RequirementStatus::UnderReview,
                    "approved" => RequirementStatus::Approved,
                    "implemented" => RequirementStatus::Implemented,
                    "verified" => RequirementStatus::Verified,
                    "validated" => RequirementStatus::Validated,
                    "obsolete" => RequirementStatus::Obsolete,
                    _ => return Err(format!("Invalid status: {}", args[i + 1])),
                };
                update = update.status(status);
                i += 2;
            }
            "--source" => {
                if i + 1 >= args.len() {
                    return Err("Missing value for --source".to_string());
                }
                update = update.source(args[i + 1].clone());
                i += 2;
            }
            "--rationale" => {
                if i + 1 >= args.len() {
                    return Err("Missing value for --rationale".to_string());
                }
                update = update.rationale(args[i + 1].clone());
                i += 2;
            }
            "--acceptance-criteria" => {
                if i + 1 >= args.len() {
                    return Err("Missing value for --acceptance-criteria".to_string());
                }
                update = update.acceptance_criteria(args[i + 1].clone());
                i += 2;
            }
            "--verification-method" => {
                if i + 1 >= args.len() {
                    return Err("Missing value for --verification-method".to_string());
                }
                let method = match args[i + 1].to_lowercase().as_str() {
                    "test" => VerificationMethod::Test,
                    "analysis" => VerificationMethod::Analysis,
                    "inspection" => VerificationMethod::Inspection,
                    "demonstration" => VerificationMethod::Demonstration,
                    "review" => VerificationMethod::Review,
                    _ => return Err(format!("Invalid verification method: {}", args[i + 1])),
                };
                update = update.verification_method(method);
                i += 2;
            }
            _ => {
                return Err(format!("Unknown option: {}", args[i]));
            }
        }
    }
    
    // Get current project path
    let project_path = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?;
    
    // Load requirements manager
    let mut manager = RequirementManager::new(&project_path)
        .map_err(|e| format!("Failed to initialize requirements manager: {e}"))?;
    
    // Update requirement
    manager.update_requirement(req_id, update)
        .map_err(|e| format!("Failed to update requirement: {e}"))?;
    
    println!("✓ Requirement {req_id} updated successfully");
    
    Ok(())
}

fn handle_req_delete(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Missing requirement ID. Usage: qms req delete <REQ-ID>".to_string());
    }
    
    if args.len() > 1 && args[1] == "--help" {
        print_req_delete_help();
        return Ok(());
    }
    
    let req_id = &args[0];
    
    // Check for confirmation flag
    let mut confirm = false;
    for arg in args.iter().skip(1) {
        if arg == "--confirm" {
            confirm = true;
            break;
        }
    }
    
    if !confirm {
        println!("WARNING: This will permanently delete requirement {req_id}");
        println!("To confirm deletion, use: qms req delete {req_id} --confirm");
        return Ok(());
    }
    
    // Get current project path
    let project_path = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?;
    
    // Load requirements manager
    let mut manager = RequirementManager::new(&project_path)
        .map_err(|e| format!("Failed to initialize requirements manager: {e}"))?;
    
    // Delete requirement
    manager.delete_requirement(req_id)
        .map_err(|e| format!("Failed to delete requirement: {e}"))?;
    
    println!("✓ Requirement {req_id} deleted successfully");
    
    Ok(())
}

fn print_req_help() {
    println!("Manage QMS requirements\n");
    println!("USAGE:");
    println!("    qms req <COMMAND>\n");
    println!("COMMANDS:");
    println!("    init       Initialize requirements system");
    println!("    create     Create a new requirement");
    println!("    list       List all requirements");
    println!("    show       Show requirement details");
    println!("    update     Update a requirement");
    println!("    delete     Delete a requirement");
    println!("    help       Show this help message\n");
    println!("For more information on a specific command, use:");
    println!("    qms req <COMMAND> --help");
}

fn print_req_create_help() {
    println!("Create a new requirement\n");
    println!("USAGE:");
    println!("    qms req create --title <TITLE> --desc <DESCRIPTION> [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --title <TITLE>          Requirement title (required)");
    println!("    --desc <DESCRIPTION>     Requirement description (required)");
    println!("    --id <ID>                Custom requirement ID (e.g., REQ-001)");
    println!("    --auto-id                Auto-generate requirement ID");
    println!("    --category <CATEGORY>    Category (functional, performance, usability, reliability, safety, security, regulatory, interface, data, system)");
    println!("    --priority <PRIORITY>    Priority (critical, high, medium, low)");
    println!("    --source <SOURCE>        Source of requirement");
    println!("    --help                   Show this help message\n");
    println!("EXAMPLES:");
    println!("    qms req create --title \"User Authentication\" --desc \"System shall authenticate users\" --category security");
    println!("    qms req create --title \"Response Time\" --desc \"System shall respond within 2 seconds\" --category performance --priority high");
}

fn print_req_list_help() {
    println!("List requirements\n");
    println!("USAGE:");
    println!("    qms req list [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --category <CATEGORY>    Filter by category (functional, performance, etc.)");
    println!("    --status <STATUS>        Filter by status (draft, approved, implemented, etc.)");
    println!("    --priority <PRIORITY>    Filter by priority (critical, high, medium, low)");
    println!("    --details                Show detailed information");
    println!("    --help                   Show this help message\n");
    println!("EXAMPLES:");
    println!("    qms req list");
    println!("    qms req list --category functional --status approved");
    println!("    qms req list --priority critical --details");
}

fn print_req_show_help() {
    println!("Show detailed information about a requirement\n");
    println!("USAGE:");
    println!("    qms req show <REQ-ID>\n");
    println!("ARGUMENTS:");
    println!("    <REQ-ID>     Requirement ID (e.g., REQ-001)\n");
    println!("OPTIONS:");
    println!("    --help       Show this help message\n");
    println!("EXAMPLES:");
    println!("    qms req show REQ-001");
    println!("    qms req show REQ-015");
}

fn print_req_update_help() {
    println!("Update an existing requirement\n");
    println!("USAGE:");
    println!("    qms req update <REQ-ID> [OPTIONS]\n");
    println!("ARGUMENTS:");
    println!("    <REQ-ID>                 Requirement ID (e.g., REQ-001)\n");
    println!("OPTIONS:");
    println!("    --title <TITLE>          Update requirement title");
    println!("    --desc <DESCRIPTION>     Update requirement description");
    println!("    --category <CATEGORY>    Update category (functional, performance, usability, reliability, safety, security, regulatory, interface, data, system)");
    println!("    --priority <PRIORITY>    Update priority (critical, high, medium, low)");
    println!("    --status <STATUS>        Update status (draft, underreview, approved, implemented, verified, validated, obsolete)");
    println!("    --source <SOURCE>        Update source of requirement");
    println!("    --rationale <RATIONALE>  Update rationale");
    println!("    --acceptance-criteria <CRITERIA>  Update acceptance criteria");
    println!("    --verification-method <METHOD>     Update verification method (test, analysis, inspection, demonstration, review)");
    println!("    --help                   Show this help message\n");
    println!("EXAMPLES:");
    println!("    qms req update REQ-001 --status approved");
    println!("    qms req update REQ-001 --title \"Updated Title\" --priority high");
    println!("    qms req update REQ-001 --category security --status implemented");
}

fn handle_req_verify(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print_req_verify_help();
        return Ok(());
    }

    use crate::modules::traceability::verification::{RequirementVerificationManager, VerificationMethod};

    let mut verification_manager = RequirementVerificationManager::new("./verification.json");
    if let Err(e) = verification_manager.load() {
        eprintln!("Warning: Could not load verification data: {e}");
    }

    let mut requirement_id = String::new();
    let mut method = String::new();
    let mut evidence_id = String::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--method" => {
                if i + 1 < args.len() {
                    method = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --method requires a value");
                    return Err("Missing method".to_string());
                }
            }
            "--evidence" => {
                if i + 1 < args.len() {
                    evidence_id = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --evidence requires a value");
                    return Err("Missing evidence ID".to_string());
                }
            }
            _ => {
                if requirement_id.is_empty() {
                    requirement_id = args[i].clone();
                }
                i += 1;
            }
        }
    }

    if requirement_id.is_empty() {
        eprintln!("Error: Requirement ID is required");
        print_req_verify_help();
        return Err("Missing requirement ID".to_string());
    }

    if method.is_empty() {
        eprintln!("Error: Verification method is required");
        print_req_verify_help();
        return Err("Missing verification method".to_string());
    }

    if evidence_id.is_empty() {
        eprintln!("Error: Evidence ID is required");
        print_req_verify_help();
        return Err("Missing evidence ID".to_string());
    }

    let verification_method = VerificationMethod::from_str(&method)
        .ok_or_else(|| format!("Invalid verification method: {method}"))?;

    if let Err(e) = verification_manager.add_verification(&requirement_id, verification_method, &evidence_id) {
        eprintln!("Error adding verification: {e}");
        return Err(e.to_string());
    }

    if let Err(e) = verification_manager.save() {
        eprintln!("Error saving verification data: {e}");
        return Err(e.to_string());
    }

    println!("✅ Added verification for requirement {requirement_id} with method {method} and evidence {evidence_id}");

    Ok(())
}

fn handle_req_verification_report(args: &[String]) -> Result<(), String> {
    use crate::modules::traceability::verification::RequirementVerificationManager;
    use crate::modules::traceability::links::TraceabilityManager;

    let mut verification_manager = RequirementVerificationManager::new("./verification.json");
    if let Err(e) = verification_manager.load() {
        eprintln!("Warning: Could not load verification data: {e}");
    }

    let mut output_file = String::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--output" => {
                if i + 1 < args.len() {
                    output_file = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --output requires a value");
                    return Err("Missing output file".to_string());
                }
            }
            "--help" | "-h" => {
                print_req_verification_report_help();
                return Ok(());
            }
            _ => {
                i += 1;
            }
        }
    }

    let tm = match TraceabilityManager::new(std::path::Path::new(".")) {
        Ok(tm) => tm,
        Err(e) => {
            eprintln!("Error creating traceability manager: {e}");
            return Err(e.to_string());
        }
    };
    match verification_manager.generate_verification_report(&tm) {
        Ok(report) => {
            if !output_file.is_empty() {
                match std::fs::write(&output_file, &report) {
                    Ok(_) => println!("Verification report saved to: {output_file}"),
                    Err(e) => eprintln!("Error writing report file: {e}"),
                }
            } else {
                println!("{report}");
            }
        }
        Err(e) => eprintln!("Error generating verification report: {e}"),
    }

    Ok(())
}

fn print_req_verify_help() {
    println!("Verify a requirement with evidence\n");
    println!("USAGE:");
    println!("    qms req verify <REQ-ID> --method <METHOD> --evidence <EVIDENCE-ID>\n");
    println!("ARGUMENTS:");
    println!("    <REQ-ID>     Requirement ID (e.g., REQ-001)\n");
    println!("OPTIONS:");
    println!("    --method <METHOD>       Verification method (test, analysis, inspection, demonstration)");
    println!("    --evidence <EVIDENCE>   Evidence ID linking to verification");
    println!("    --help                  Show this help message\n");
    println!("VERIFICATION METHODS:");
    println!("    test                    Testing and verification through test execution");
    println!("    analysis                Analysis and mathematical verification");
    println!("    inspection              Visual inspection and code review");
    println!("    demonstration           Demonstration of functionality\n");
    println!("EXAMPLES:");
    println!("    qms req verify REQ-001 --method test --evidence TC-001");
    println!("    qms req verify REQ-002 --method analysis --evidence AR-001");
    println!("    qms req verify REQ-003 --method inspection --evidence IR-001");
}

fn print_req_verification_report_help() {
    println!("Generate requirement verification report\n");
    println!("USAGE:");
    println!("    qms req verification-report [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --output <FILE>         Output file path (default: stdout)");
    println!("    --help                  Show this help message\n");
    println!("EXAMPLES:");
    println!("    qms req verification-report");
    println!("    qms req verification-report --output verification_status.pdf");
}

fn print_req_delete_help() {
    println!("Delete a requirement\n");
    println!("USAGE:");
    println!("    qms req delete <REQ-ID> --confirm\n");
    println!("ARGUMENTS:");
    println!("    <REQ-ID>     Requirement ID (e.g., REQ-001)\n");
    println!("OPTIONS:");
    println!("    --confirm    Required flag to confirm deletion");
    println!("    --help       Show this help message\n");
    println!("EXAMPLES:");
    println!("    qms req delete REQ-001 --confirm");
    println!("    qms req delete REQ-999 --confirm");
}
