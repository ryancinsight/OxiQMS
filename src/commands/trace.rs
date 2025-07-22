use std::process;
use std::path::Path;
use crate::modules::traceability::links::{TraceabilityManager, TraceLinkType, TraceabilityPathNode};
use crate::modules::traceability::rtm::{RTMGenerator, RTMConfig, RTMFormat, RTMSortBy};
use crate::modules::traceability::visualization::{GraphVisualizer, GraphFormat};
use crate::modules::traceability::coverage::{CoverageAnalyzer, CoverageType};
use crate::modules::traceability::impact::{ImpactAnalyzer, format_impact_analysis};

pub fn handle_trace_command(args: &[String]) -> Result<(), String> {
    if args.len() < 3 {
        print_trace_help();
        return Ok(());
    }

    match args[2].as_str() {
        "create" => handle_trace_create(&args[3..]),
        "link" => handle_trace_link(&args[3..]),
        "view" => handle_trace_view(&args[3..]),
        "list" => handle_trace_list(&args[3..]),
        "delete" => handle_trace_delete(&args[3..]),
        "forward" => handle_trace_forward(&args[3..]),
        "backward" => handle_trace_backward(&args[3..]),
        "orphans" => handle_trace_orphans(&args[3..]),
        "matrix" => handle_trace_matrix(&args[3..]),
        "import" => handle_trace_import(&args[3..]),
        "export" => handle_trace_export(&args[3..]),
        "stats" => handle_trace_stats(&args[3..]),
        "graph" => handle_trace_graph(&args[3..]),
        "coverage" => handle_trace_coverage(&args[3..]),
        "impact" => handle_trace_impact(&args[3..]),
        "verify" => handle_trace_verify(&args[3..]),
        "validate" => handle_trace_validate(&args[3..]),
        "--help" | "-h" => {
            print_trace_help();
            Ok(())
        }
        _ => {
            eprintln!("Error: Unknown traceability command '{}'", args[2]);
            print_trace_help();
            process::exit(1);
        }
    }
}

fn handle_trace_link(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print_trace_link_help();
        return Ok(());
    }

    let mut from_id = String::new();
    let mut to_id = String::new();
    let mut link_type = String::new();
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--from" => {
                if i + 1 < args.len() {
                    from_id = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--from requires a value".to_string());
                }
            }
            "--to" => {
                if i + 1 < args.len() {
                    to_id = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--to requires a value".to_string());
                }
            }
            "--type" => {
                if i + 1 < args.len() {
                    link_type = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--type requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown option: {}", args[i]));
            }
        }
    }

    if from_id.is_empty() || to_id.is_empty() || link_type.is_empty() {
        return Err("Missing required arguments. Use --from, --to, and --type".to_string());
    }

    // Get project root
    let project_root = std::env::current_dir().map_err(|e| e.to_string())?;
    
    // Create traceability manager
    let manager = TraceabilityManager::new(&project_root)
        .map_err(|e| format!("Failed to initialize traceability manager: {e}"))?;

    // Parse link type
    let parsed_link_type = TraceLinkType::from_str(&link_type)
        .map_err(|e| format!("Invalid link type: {e}"))?;

    // Create the link
    match manager.create_trace_link(&from_id, &to_id, parsed_link_type) {
        Ok(link) => {
            println!("✓ Created traceability link:");
            println!("  ID: {}", link.id);
            println!("  From: {} ({})", link.source_id, link.source_type);
            println!("  To: {} ({})", link.target_id, link.target_type);
            println!("  Type: {}", link.link_type.to_string());
            println!("  Created: {}", link.created_at);
            Ok(())
        }
        Err(e) => Err(format!("Failed to create link: {e}")),
    }
}

fn handle_trace_view(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print_trace_view_help();
        return Ok(());
    }

    let entity_id = &args[0];
    
    // Get project root
    let project_root = std::env::current_dir().map_err(|e| e.to_string())?;
    
    // Create traceability manager
    let manager = TraceabilityManager::new(&project_root)
        .map_err(|e| format!("Failed to initialize traceability manager: {e}"))?;

    // Get links for entity
    match manager.get_links_for_entity(entity_id) {
        Ok(links) => {
            if links.is_empty() {
                println!("No traceability links found for entity: {entity_id}");
                return Ok(());
            }

            println!("Traceability links for {entity_id}:");
            println!("─────────────────────────────────────────────────────────────");
            
            for link in links {
                let direction = if link.source_id == *entity_id {
                    format!("→ {}", link.target_id)
                } else {
                    format!("← {}", link.source_id)
                };
                
                println!("  {} [{}] ({})", direction, link.link_type.to_string(), link.created_at);
                if link.verified {
                    println!("    ✓ Verified");
                }
            }
            
            Ok(())
        }
        Err(e) => Err(format!("Failed to get links: {e}")),
    }
}

fn handle_trace_list(args: &[String]) -> Result<(), String> {
    if !args.is_empty() && (args[0] == "--help" || args[0] == "-h") {
        print_trace_list_help();
        return Ok(());
    }

    // Get project root
    let project_root = std::env::current_dir().map_err(|e| e.to_string())?;
    
    // Create traceability manager
    let manager = TraceabilityManager::new(&project_root)
        .map_err(|e| format!("Failed to initialize traceability manager: {e}"))?;

    // Get all links
    match manager.get_trace_links() {
        Ok(links) => {
            if links.is_empty() {
                println!("No traceability links found");
                return Ok(());
            }

            println!("All Traceability Links:");
            println!("─────────────────────────────────────────────────────────────");
            
            for link in links {
                println!("  {} → {} [{}]", link.source_id, link.target_id, link.link_type.to_string());
                println!("    ID: {}", link.id);
                println!("    Created: {} by {}", link.created_at, link.created_by);
                if link.verified {
                    println!("    ✓ Verified");
                }
                println!();
            }
            
            Ok(())
        }
        Err(e) => Err(format!("Failed to get links: {e}")),
    }
}

fn handle_trace_delete(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print_trace_delete_help();
        return Ok(());
    }

    let link_id = &args[0];
    
    // Get project root
    let project_root = std::env::current_dir().map_err(|e| e.to_string())?;
    
    // Create traceability manager
    let manager = TraceabilityManager::new(&project_root)
        .map_err(|e| format!("Failed to initialize traceability manager: {e}"))?;

    // Delete the link
    match manager.delete_trace_link(link_id) {
        Ok(()) => {
            println!("✓ Deleted traceability link: {link_id}");
            Ok(())
        }
        Err(e) => Err(format!("Failed to delete link: {e}")),
    }
}

fn handle_trace_create(args: &[String]) -> Result<(), String> {
    println!("Traceability create command - implementation pending");
    println!("Use 'qms req create' for requirements and 'qms test create' for test cases");
    println!("Args: {args:?}");
    Ok(())
}

fn handle_trace_forward(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print_trace_forward_help();
        return Ok(());
    }

    let entity_id = &args[0];
    
    // Get project root
    let project_root = std::env::current_dir().map_err(|e| e.to_string())?;
    
    // Create traceability manager
    let manager = TraceabilityManager::new(&project_root)
        .map_err(|e| format!("Failed to initialize traceability manager: {e}"))?;

    // Trace forward
    match manager.trace_forward(entity_id) {
        Ok(path) => {
            println!("📋 Forward Traceability from {entity_id}");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!();
            
            println!("🎯 Source Entity:");
            println!("   ID: {}", path.entity_id);
            println!("   Type: {}", path.entity_type);
            println!("   Max Depth: {}", path.depth);
            println!();
            
            if path.path.is_empty() {
                println!("No forward traceability links found for {entity_id}");
            } else {
                println!("🔗 Forward Traceability Chain:");
                print_traceability_tree(&path.path, 0, true);
            }
            
            Ok(())
        }
        Err(e) => Err(format!("Failed to trace forward: {e}")),
    }
}

fn handle_trace_backward(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print_trace_backward_help();
        return Ok(());
    }

    let entity_id = &args[0];
    
    // Get project root
    let project_root = std::env::current_dir().map_err(|e| e.to_string())?;
    
    // Create traceability manager
    let manager = TraceabilityManager::new(&project_root)
        .map_err(|e| format!("Failed to initialize traceability manager: {e}"))?;

    // Trace backward
    match manager.trace_backward(entity_id) {
        Ok(path) => {
            println!("📋 Backward Traceability to {entity_id}");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!();
            
            println!("🎯 Target Entity:");
            println!("   ID: {}", path.entity_id);
            println!("   Type: {}", path.entity_type);
            println!("   Max Depth: {}", path.depth);
            println!();
            
            if path.path.is_empty() {
                println!("No backward traceability links found for {entity_id}");
            } else {
                println!("🔗 Backward Traceability Chain:");
                print_traceability_tree(&path.path, 0, false);
            }
            
            Ok(())
        }
        Err(e) => Err(format!("Failed to trace backward: {e}")),
    }
}

fn handle_trace_orphans(args: &[String]) -> Result<(), String> {
    if !args.is_empty() && (args[0] == "--help" || args[0] == "-h") {
        print_trace_orphans_help();
        return Ok(());
    }

    // Get project root
    let project_root = std::env::current_dir().map_err(|e| e.to_string())?;
    
    // Create traceability manager
    let manager = TraceabilityManager::new(&project_root)
        .map_err(|e| format!("Failed to initialize traceability manager: {e}"))?;

    // Find orphaned items
    match manager.find_orphaned_items() {
        Ok(orphans) => {
            println!("🔍 Orphaned Items Analysis");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!();
            
            if orphans.is_empty() {
                println!("✅ No orphaned items found");
                println!("   All entities have traceability links");
            } else {
                println!("⚠️  Found {} orphaned items:", orphans.len());
                println!();
                
                // Group by entity type
                let mut by_type = std::collections::HashMap::new();
                for orphan in &orphans {
                    by_type.entry(orphan.entity_type.clone())
                        .or_insert_with(Vec::new)
                        .push(orphan);
                }
                
                for (entity_type, orphan_list) in by_type {
                    println!("📂 {} ({})", entity_type, orphan_list.len());
                    for orphan in orphan_list {
                        println!("   • {} - {}", orphan.entity_id, orphan.reason);
                    }
                    println!();
                }
                
                println!("💡 Recommendations:");
                println!("   • Review orphaned items to determine if they should be linked");
                println!("   • Use 'qms trace link' to create missing traceability links");
                println!("   • Consider archiving or removing obsolete items");
            }
            
            Ok(())
        }
        Err(e) => Err(format!("Failed to find orphaned items: {e}")),
    }
}

// Helper function to print traceability tree
fn print_traceability_tree(nodes: &[TraceabilityPathNode], indent: usize, is_forward: bool) {
    for (i, node) in nodes.iter().enumerate() {
        let prefix = if i == nodes.len() - 1 { "└── " } else { "├── " };
        let indent_str = "    ".repeat(indent);
        
        let arrow = if is_forward { "→" } else { "←" };
        println!("{}{}[{}] {} {} ({})", 
            indent_str, prefix, node.link_type.to_string(), arrow, node.entity_id, node.entity_type);
        
        if !node.children.is_empty() {
            print_traceability_tree(&node.children, indent + 1, is_forward);
        }
    }
}

fn handle_trace_matrix(args: &[String]) -> Result<(), String> {
    if !args.is_empty() && (args[0] == "--help" || args[0] == "-h") {
        print_trace_matrix_help();
        return Ok(());
    }

    let mut config = RTMConfig::default();
    let mut format = RTMFormat::CSV;
    let mut output_path: Option<String> = None;
    let mut show_stats = false;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--format" => {
                if i + 1 < args.len() {
                    format = RTMFormat::from_str(&args[i + 1]).map_err(|e| e.to_string())?;
                    i += 2;
                } else {
                    return Err("--format requires a value".to_string());
                }
            }
            "--output" => {
                if i + 1 < args.len() {
                    output_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--output requires a value".to_string());
                }
            }
            "--category" => {
                if i + 1 < args.len() {
                    let categories = args[i + 1].split(',').map(|s| s.trim().to_string()).collect();
                    config.include_categories = Some(categories);
                    i += 2;
                } else {
                    return Err("--category requires a value".to_string());
                }
            }
            "--priority" => {
                if i + 1 < args.len() {
                    let priorities = args[i + 1].split(',').map(|s| s.trim().to_string()).collect();
                    config.include_priorities = Some(priorities);
                    i += 2;
                } else {
                    return Err("--priority requires a value".to_string());
                }
            }
            "--status" => {
                if i + 1 < args.len() {
                    let statuses = args[i + 1].split(',').map(|s| s.trim().to_string()).collect();
                    config.include_statuses = Some(statuses);
                    i += 2;
                } else {
                    return Err("--status requires a value".to_string());
                }
            }
            "--verification-status" => {
                if i + 1 < args.len() {
                    let ver_statuses = args[i + 1].split(',').map(|s| s.trim().to_string()).collect();
                    config.include_verification_statuses = Some(ver_statuses);
                    i += 2;
                } else {
                    return Err("--verification-status requires a value".to_string());
                }
            }
            "--sort-by" => {
                if i + 1 < args.len() {
                    config.sort_by = RTMSortBy::from_str(&args[i + 1]).map_err(|e| e.to_string())?;
                    i += 2;
                } else {
                    return Err("--sort-by requires a value".to_string());
                }
            }
            "--show-descriptions" => {
                config.show_descriptions = true;
                i += 1;
            }
            "--show-verification-details" => {
                config.show_verification_details = true;
                i += 1;
            }
            "--hide-coverage" => {
                config.show_coverage_metrics = false;
                i += 1;
            }
            "--stats" => {
                show_stats = true;
                i += 1;
            }
            _ => {
                return Err(format!("Unknown option: {}", args[i]));
            }
        }
    }

    // Get project root
    let project_root = std::env::current_dir().map_err(|e| e.to_string())?;
    let mut rtm_generator = RTMGenerator::new(&project_root).map_err(|e| e.to_string())?;

    if show_stats {
        // Generate and display RTM statistics
        let stats = rtm_generator.generate_stats().map_err(|e| e.to_string())?;
        
        println!("📊 Requirements Traceability Matrix Statistics");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();
        
        // General statistics
        println!("📋 General Statistics:");
        println!("   Total Requirements: {}", stats.total_requirements);
        println!("   Total Test Cases: {}", stats.total_test_cases);
        println!("   Total Links: {}", stats.total_links);
        println!();
        
        // Coverage statistics
        println!("🎯 Coverage Statistics:");
        println!("   Requirements with Tests: {} ({:.1}%)", 
            stats.requirements_with_tests, 
            stats.verification_coverage
        );
        println!("   Requirements without Tests: {}", stats.requirements_without_tests);
        println!("   Test Cases with Requirements: {}", stats.test_cases_with_requirements);
        println!("   Orphaned Test Cases: {}", stats.orphaned_test_cases);
        println!();
        
        // Category breakdown
        if !stats.category_breakdown.is_empty() {
            println!("📂 Category Breakdown:");
            for (category, count) in &stats.category_breakdown {
                println!("   {category}: {count}");
            }
            println!();
        }
        
        // Priority breakdown
        if !stats.priority_breakdown.is_empty() {
            println!("⚡ Priority Breakdown:");
            for (priority, count) in &stats.priority_breakdown {
                println!("   {priority}: {count}");
            }
            println!();
        }
        
        // Verification status breakdown
        if !stats.verification_status_breakdown.is_empty() {
            println!("✅ Verification Status Breakdown:");
            for (status, count) in &stats.verification_status_breakdown {
                println!("   {status}: {count}");
            }
            println!();
        }
        
        // Coverage assessment
        if stats.verification_coverage >= 80.0 {
            println!("🎉 Coverage Assessment: EXCELLENT (≥80%)");
        } else if stats.verification_coverage >= 60.0 {
            println!("⚠️  Coverage Assessment: MODERATE (60-80%)");
        } else {
            println!("❌ Coverage Assessment: POOR (<60%)");
        }
        
        return Ok(());
    }

    if let Some(output_path) = output_path {
        // Generate and export RTM
        let output_file = std::path::Path::new(&output_path);
        rtm_generator.generate_and_export(&config, format.clone(), output_file).map_err(|e| e.to_string())?;
        
        println!("✅ Generated Requirements Traceability Matrix");
        println!("   Format: {format:?}");
        println!("   Output: {output_path}");
        println!("   Configuration:");
        
        if let Some(ref categories) = config.include_categories {
            println!("     Categories: {}", categories.join(", "));
        }
        if let Some(ref priorities) = config.include_priorities {
            println!("     Priorities: {}", priorities.join(", "));
        }
        if let Some(ref statuses) = config.include_statuses {
            println!("     Statuses: {}", statuses.join(", "));
        }
        if let Some(ref ver_statuses) = config.include_verification_statuses {
            println!("     Verification Statuses: {}", ver_statuses.join(", "));
        }
        
        println!("     Show Descriptions: {}", config.show_descriptions);
        println!("     Show Coverage Metrics: {}", config.show_coverage_metrics);
        println!("     Show Verification Details: {}", config.show_verification_details);
        println!("     Sort By: {:?}", config.sort_by);
        
    } else {
        // Generate RTM and display summary
        let entries = rtm_generator.generate_rtm(&config).map_err(|e| e.to_string())?;
        
        println!("📋 Requirements Traceability Matrix Summary");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();
        
        if entries.is_empty() {
            println!("No requirements found matching the specified criteria.");
            return Ok(());
        }
        
        println!("📊 Matrix Overview:");
        println!("   Total Requirements: {}", entries.len());
        
        let verified_count = entries.iter().filter(|e| e.verification_status == "Verified").count();
        let partial_count = entries.iter().filter(|e| e.verification_status == "Partially Verified").count();
        let not_verified_count = entries.iter().filter(|e| e.verification_status == "Not Verified").count();
        
        println!("   Verified: {} ({:.1}%)", verified_count, (verified_count as f64 / entries.len() as f64) * 100.0);
        println!("   Partially Verified: {} ({:.1}%)", partial_count, (partial_count as f64 / entries.len() as f64) * 100.0);
        println!("   Not Verified: {} ({:.1}%)", not_verified_count, (not_verified_count as f64 / entries.len() as f64) * 100.0);
        println!();
        
        // Show first few entries as preview
        println!("📋 Requirements Preview (first 5):");
        for (i, entry) in entries.iter().take(5).enumerate() {
            println!("   {}. {} - {}", i + 1, entry.requirement_id, entry.requirement_title);
            println!("      Category: {} | Priority: {} | Status: {}", 
                entry.requirement_category, entry.requirement_priority, entry.requirement_status);
            println!("      Test Cases: {} | Verification: {} ({:.1}%)", 
                if entry.linked_test_cases.is_empty() { "None".to_string() } else { entry.linked_test_cases.join(", ") },
                entry.verification_status, entry.coverage_percentage);
            println!();
        }
        
        if entries.len() > 5 {
            println!("   ... and {} more requirements", entries.len() - 5);
            println!();
        }
        
        println!("💡 Use --output <file> to export the complete matrix.");
        println!("   Example: qms trace matrix --format csv --output rtm.csv");
    }

    Ok(())
}

fn handle_trace_stats(args: &[String]) -> Result<(), String> {
    if !args.is_empty() && (args[0] == "--help" || args[0] == "-h") {
        print_trace_stats_help();
        return Ok(());
    }

    // Get project root
    let project_root = std::env::current_dir().map_err(|e| e.to_string())?;
    let mut rtm_generator = RTMGenerator::new(&project_root).map_err(|e| e.to_string())?;
    let stats = rtm_generator.generate_stats().map_err(|e| e.to_string())?;
    
    println!("📊 Traceability Statistics");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    
    println!("📋 Overview:");
    println!("   Total Requirements: {}", stats.total_requirements);
    println!("   Total Test Cases: {}", stats.total_test_cases);
    println!("   Total Traceability Links: {}", stats.total_links);
    println!();
    
    println!("🎯 Coverage Analysis:");
    println!("   Requirements with Tests: {} ({:.1}%)", 
        stats.requirements_with_tests, stats.verification_coverage);
    println!("   Requirements without Tests: {}", stats.requirements_without_tests);
    println!("   Test Cases with Requirements: {}", stats.test_cases_with_requirements);
    println!("   Orphaned Test Cases: {}", stats.orphaned_test_cases);
    println!();
    
    if !stats.category_breakdown.is_empty() {
        println!("📂 Requirements by Category:");
        for (category, count) in &stats.category_breakdown {
            println!("   {category}: {count}");
        }
        println!();
    }
    
    if !stats.priority_breakdown.is_empty() {
        println!("⚡ Requirements by Priority:");
        for (priority, count) in &stats.priority_breakdown {
            println!("   {priority}: {count}");
        }
        println!();
    }
    
    if !stats.verification_status_breakdown.is_empty() {
        println!("✅ Verification Status:");
        for (status, count) in &stats.verification_status_breakdown {
            println!("   {status}: {count}");
        }
        println!();
    }
    
    // Coverage assessment
    println!("🎯 Coverage Assessment:");
    if stats.verification_coverage >= 80.0 {
        println!("   Status: 🎉 EXCELLENT (≥80%)");
        println!("   Recommendation: Maintain current coverage levels");
    } else if stats.verification_coverage >= 60.0 {
        println!("   Status: ⚠️  MODERATE (60-80%)");
        println!("   Recommendation: Add test cases for {} untested requirements", stats.requirements_without_tests);
    } else {
        println!("   Status: ❌ POOR (<60%)");
        println!("   Recommendation: Significant improvement needed - {} requirements need tests", stats.requirements_without_tests);
    }
    
    if stats.orphaned_test_cases > 0 {
        println!("   ⚠️  Warning: {} orphaned test cases found", stats.orphaned_test_cases);
        println!("   Recommendation: Link test cases to requirements or remove if obsolete");
    }
    
    Ok(())
}

fn handle_trace_validate(args: &[String]) -> Result<(), String> {
    println!("Traceability validation command - implementation pending");
    println!("Args: {args:?}");
    Ok(())
}

fn handle_trace_import(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print_trace_import_help();
        return Ok(());
    }
    
    let current_dir = std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;
    let manager = TraceabilityManager::new(&current_dir).map_err(|e| format!("Failed to create TraceabilityManager: {e}"))?;
    
    let mut file_path: Option<String> = None;
    let mut format = "csv";
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "--file" | "-f" => {
                if i + 1 < args.len() {
                    file_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --file requires a value".to_string());
                }
            }
            "--format" => {
                if i + 1 < args.len() {
                    format = &args[i + 1];
                    i += 2;
                } else {
                    return Err("Error: --format requires a value".to_string());
                }
            }
            _ => {
                if file_path.is_none() {
                    file_path = Some(args[i].clone());
                }
                i += 1;
            }
        }
    }
    
    let file_path = file_path.ok_or("Error: File path is required")?;
    let path = Path::new(&file_path);
    
    if !path.exists() {
        return Err(format!("Error: File '{file_path}' does not exist"));
    }
    
    println!("📥 Importing traceability links from '{file_path}'...");
    println!("Format: {format}");
    
    let stats = match format.to_lowercase().as_str() {
        "csv" => manager.import_from_csv(path).map_err(|e| format!("CSV import failed: {e}"))?,
        "json" => manager.import_from_json(path).map_err(|e| format!("JSON import failed: {e}"))?,
        _ => return Err(format!("Error: Unsupported format '{format}'. Supported formats: csv, json")),
    };
    
    println!("✅ Import completed successfully!");
    println!("   📊 Total processed: {}", stats.total_processed);
    println!("   ✅ Successful imports: {}", stats.successful_imports);
    println!("   ❌ Failed imports: {}", stats.failed_imports);
    println!("   🔄 Duplicates skipped: {}", stats.duplicates_found);
    
    if !stats.validation_errors.is_empty() {
        println!("   ⚠️  Validation errors:");
        for error in &stats.validation_errors {
            println!("      • {error}");
        }
    }
    
    Ok(())
}

fn handle_trace_export(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print_trace_export_help();
        return Ok(());
    }
    
    let current_dir = std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;
    let manager = TraceabilityManager::new(&current_dir).map_err(|e| format!("Failed to create TraceabilityManager: {e}"))?;
    
    let mut output_path: Option<String> = None;
    let mut format = "csv";
    let mut export_type = "rtm";
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --output requires a value".to_string());
                }
            }
            "--format" | "-f" => {
                if i + 1 < args.len() {
                    format = &args[i + 1];
                    i += 2;
                } else {
                    return Err("Error: --format requires a value".to_string());
                }
            }
            "--type" | "-t" => {
                if i + 1 < args.len() {
                    export_type = &args[i + 1];
                    i += 2;
                } else {
                    return Err("Error: --type requires a value".to_string());
                }
            }
            _ => {
                if output_path.is_none() {
                    output_path = Some(args[i].clone());
                }
                i += 1;
            }
        }
    }
    
    let output_path = output_path.unwrap_or_else(|| format!("export.{format}"));
    let path = Path::new(&output_path);
    
    println!("📤 Exporting traceability data...");
    println!("   Type: {export_type}");
    println!("   Format: {format}");
    println!("   Output: {output_path}");
    
    match export_type.to_lowercase().as_str() {
        "rtm" | "matrix" => {
            match format.to_lowercase().as_str() {
                "csv" => manager.export_rtm_csv(path).map_err(|e| format!("RTM CSV export failed: {e}"))?,
                "json" => manager.export_rtm_json(path).map_err(|e| format!("RTM JSON export failed: {e}"))?,
                _ => return Err(format!("Error: Unsupported format '{format}' for RTM export. Supported formats: csv, json")),
            }
        }
        "graph" | "dot" => {
            manager.export_dependency_graph(path).map_err(|e| format!("Dependency graph export failed: {e}"))?;
        }
        _ => return Err(format!("Error: Unsupported export type '{export_type}'. Supported types: rtm, matrix, graph, dot")),
    }
    
    println!("✅ Export completed successfully!");
    println!("   📄 Output file: {output_path}");
    
    Ok(())
}

fn print_trace_help() {
    println!("Manage QMS traceability\n");
    println!("USAGE:");
    println!("    qms trace <COMMAND>\n");
    println!("COMMANDS:");
    println!("    link       Create traceability links between entities");
    println!("    view       View traceability links for an entity");
    println!("    list       List all traceability links");
    println!("    delete     Delete a traceability link");
    println!("    forward    Trace forward from source to targets");
    println!("    backward   Trace backward from target to sources");
    println!("    orphans    Find orphaned items with no traceability links");
    println!("    matrix     Generate Requirements Traceability Matrix (RTM)");
    println!("    import     Import traceability links from CSV/JSON files");
    println!("    export     Export traceability data to various formats");
    println!("    stats      Show traceability statistics and coverage analysis");
    println!("    graph      Generate visual dependency graphs");
    println!("    coverage   Analyze traceability coverage and gaps");
    println!("    impact     Analyze impact of requirement changes");
    println!("    validate   Validate traceability completeness (pending)");
    println!("    help       Show this help message\n");
    println!("For more information on a specific command, use:");
    println!("    qms trace <COMMAND> --help");
}

fn print_trace_link_help() {
    println!("Create traceability links between entities\n");
    println!("USAGE:");
    println!("    qms trace link --from <SOURCE_ID> --to <TARGET_ID> --type <LINK_TYPE>\n");
    println!("ARGUMENTS:");
    println!("    --from <SOURCE_ID>   Source entity ID (e.g., REQ-001)");
    println!("    --to <TARGET_ID>     Target entity ID (e.g., TC-001)");
    println!("    --type <LINK_TYPE>   Type of relationship\n");
    println!("LINK TYPES:");
    println!("    verifies      Target verifies source (e.g., test case verifies requirement)");
    println!("    implements    Target implements source (e.g., design implements requirement)");
    println!("    depends_on    Target depends on source");
    println!("    derived_from  Target is derived from source");
    println!("    conflicts     Target conflicts with source");
    println!("    duplicates    Target duplicates source");
    println!("    related       General relationship\n");
    println!("EXAMPLES:");
    println!("    qms trace link --from REQ-001 --to TC-001 --type verifies");
    println!("    qms trace link --from REQ-001 --to REQ-002 --type depends_on");
    println!("    qms trace link --from RISK-001 --to REQ-001 --type related");
}

fn print_trace_view_help() {
    println!("View traceability links for an entity\n");
    println!("USAGE:");
    println!("    qms trace view <ENTITY_ID>\n");
    println!("ARGUMENTS:");
    println!("    <ENTITY_ID>   Entity ID to view links for (e.g., REQ-001)\n");
    println!("EXAMPLES:");
    println!("    qms trace view REQ-001   # View all links for requirement REQ-001");
    println!("    qms trace view TC-001    # View all links for test case TC-001");
}

fn print_trace_list_help() {
    println!("List all traceability links\n");
    println!("USAGE:");
    println!("    qms trace list\n");
    println!("DESCRIPTION:");
    println!("    Shows all traceability links in the current project with their");
    println!("    source, target, relationship type, and creation information.");
}

fn print_trace_delete_help() {
    println!("Delete a traceability link\n");
    println!("USAGE:");
    println!("    qms trace delete <LINK_ID>\n");
    println!("ARGUMENTS:");
    println!("    <LINK_ID>   ID of the traceability link to delete\n");
    println!("EXAMPLES:");
    println!("    qms trace delete abc-123-def   # Delete link with ID abc-123-def");
    println!("\nNOTE: Use 'qms trace list' to see all link IDs");
}

fn print_trace_matrix_help() {
    println!("Generate Requirements Traceability Matrix (RTM)\n");
    println!("USAGE:");
    println!("    qms trace matrix [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --format <FORMAT>              Output format (csv, json, html, pdf, markdown)");
    println!("    --output <FILE>                Output file path");
    println!("    --category <CATEGORIES>        Filter by requirement categories (comma-separated)");
    println!("    --priority <PRIORITIES>        Filter by requirement priorities (comma-separated)");
    println!("    --status <STATUSES>            Filter by requirement statuses (comma-separated)");
    println!("    --verification-status <STATUS> Filter by verification status (comma-separated)");
    println!("    --sort-by <CRITERIA>           Sort by (id, title, priority, status, verification_status, coverage)");
    println!("    --show-descriptions            Include requirement descriptions in output");
    println!("    --show-verification-details    Include verification details in output");
    println!("    --hide-coverage                Hide coverage metrics from output");
    println!("    --stats                        Show RTM statistics instead of matrix");
    println!("    --help                         Show this help message\n");
    println!("FORMATS:");
    println!("    csv        Comma-separated values (default)");
    println!("    json       JSON format");
    println!("    html       HTML table format");
    println!("    pdf        PDF format (requires external tools)");
    println!("    markdown   Markdown table format\n");
    println!("EXAMPLES:");
    println!("    qms trace matrix --format csv --output rtm.csv");
    println!("    qms trace matrix --category Functional,Safety --priority High,Critical");
    println!("    qms trace matrix --stats");
    println!("    qms trace matrix --format html --output rtm.html --show-descriptions");
    println!("    qms trace matrix --verification-status \"Not Verified\" --sort-by priority");
}

fn print_trace_stats_help() {
    println!("Show traceability statistics and coverage analysis\n");
    println!("USAGE:");
    println!("    qms trace stats\n");
    println!("DESCRIPTION:");
    println!("    Displays comprehensive statistics about requirements traceability including:");
    println!("    - Total requirements, test cases, and traceability links");
    println!("    - Coverage analysis (requirements with/without tests)");
    println!("    - Verification status breakdown");
    println!("    - Requirements breakdown by category and priority");
    println!("    - Coverage assessment with recommendations\n");
    println!("EXAMPLES:");
    println!("    qms trace stats    # Show all traceability statistics");
}

fn handle_trace_graph(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print_trace_graph_help();
        return Ok(());
    }

    let mut output_path = String::new();
    let mut format = GraphFormat::SVG;
    let mut show_interactive = false;
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--output" => {
                if i + 1 < args.len() {
                    output_path = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--output requires a value".to_string());
                }
            }
            "--format" => {
                if i + 1 < args.len() {
                    format = match args[i + 1].as_str() {
                        "ascii" => GraphFormat::ASCII,
                        "svg" => GraphFormat::SVG,
                        "dot" => GraphFormat::DOT,
                        other => return Err(format!("Unknown format: {other}")),
                    };
                    i += 2;
                } else {
                    return Err("--format requires a value".to_string());
                }
            }
            "--interactive" => {
                show_interactive = true;
                i += 1;
            }
            "--help" | "-h" => {
                print_trace_graph_help();
                return Ok(());
            }
            _ => {
                return Err(format!("Unknown option: {}", args[i]));
            }
        }
    }

    if output_path.is_empty() {
        output_path = format!("dependency_graph.{}", format.as_str());
    }

    // Get current working directory
    let current_dir = std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;
    
    // Create visualization
    let visualizer = GraphVisualizer::new(&current_dir).map_err(|e| format!("Failed to create visualizer: {e}"))?;
    
    // Load data (placeholder - in real implementation would load from files)
    let requirements = vec![];
    let test_cases = vec![];
    let links = vec![];
    
    if show_interactive {
        // Generate interactive HTML
        let output_path_buf = std::path::PathBuf::from(&output_path);
        let interactive_path = output_path_buf.with_extension("html");
        
        visualizer.generate_interactive_html(&[], &[], &interactive_path)
            .map_err(|e| format!("Failed to generate interactive graph: {e}"))?;
            
        println!("✅ Generated Interactive Traceability Graph");
        println!("   Output: {}", interactive_path.display());
        println!("   Format: Interactive HTML");
        println!("   Features: Clickable nodes, detailed information");
    } else {
        // Generate regular graph
        let output_path_buf = std::path::PathBuf::from(&output_path);
        
        visualizer.generate_graph(&requirements, &test_cases, &links, format.clone(), &output_path_buf)
            .map_err(|e| format!("Failed to generate graph: {e}"))?;
            
        println!("✅ Generated Traceability Dependency Graph");
        println!("   Format: {}", format.as_str().to_uppercase());
        println!("   Output: {output_path}");
        println!("   Configuration:");
        println!("     Format: {format}");
        println!("     Interactive: {show_interactive}");
    }
    
    Ok(())
}

fn print_trace_graph_help() {
    println!("Generate visual dependency graphs for traceability analysis\n");
    println!("USAGE:");
    println!("    qms trace graph [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --output <FILE>                Output file path (default: dependency_graph.svg)");
    println!("    --format <FORMAT>              Output format (svg, ascii, dot)");
    println!("    --interactive                  Generate interactive HTML graph");
    println!("    --help                         Show this help message\n");
    println!("FORMATS:");
    println!("    svg        Scalable Vector Graphics (default)");
    println!("    ascii      ASCII text-based tree");
    println!("    dot        DOT format for Graphviz\n");
    println!("DESCRIPTION:");
    println!("    Generates visual dependency graphs showing relationships between:");
    println!("    - Requirements and test cases");
    println!("    - Test cases and design elements");
    println!("    - Requirements and risks");
    println!("    - Design elements and documents\n");
    println!("    ASCII format shows a tree structure:");
    println!("    REQ-001 (User Auth)");
    println!("    ├── TC-001 (Login Test)");
    println!("    ├── TC-002 (Logout Test)");
    println!("    └── DESIGN-001 (Auth Module)");
    println!("        └── TC-003 (Module Test)\n");
    println!("EXAMPLES:");
    println!("    qms trace graph --output dependency_graph.svg");
    println!("    qms trace graph --format ascii --output dependencies.txt");
    println!("    qms trace graph --format dot --output graph.dot");
    println!("    qms trace graph --interactive --output interactive_graph.html");
}

/// Handle trace coverage command
fn handle_trace_coverage(args: &[String]) -> Result<(), String> {
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_trace_coverage_help();
        return Ok(());
    }

    let mut coverage_type = CoverageType::Requirements;
    let mut output_file = None;
    let mut format = "text".to_string();
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--type" => {
                if i + 1 < args.len() {
                    coverage_type = match args[i + 1].as_str() {
                        "requirements" => CoverageType::Requirements,
                        "tests" => CoverageType::Tests,
                        "risks" => CoverageType::Risks,
                        _ => return Err(format!("Invalid coverage type: {}", args[i + 1])),
                    };
                    i += 2;
                } else {
                    return Err("--type requires a value".to_string());
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
            "--format" => {
                if i + 1 < args.len() {
                    format = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--format requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown option: {}", args[i]));
            }
        }
    }

    // Create traceability manager
    let project_root = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?;
    let traceability_manager = TraceabilityManager::new(&project_root)
        .map_err(|e| format!("Failed to create traceability manager: {e}"))?;
    
    // Create coverage analyzer
    let analyzer = CoverageAnalyzer::new(traceability_manager);
    
    // Analyze coverage
    let analysis = analyzer.analyze_coverage(coverage_type)
        .map_err(|e| format!("Coverage analysis failed: {e}"))?;
    
    // Generate report
    let report = analyzer.generate_report(&analysis, &format)
        .map_err(|e| format!("Report generation failed: {e}"))?;
    
    // Output results
    if let Some(output_path) = output_file {
        std::fs::write(&output_path, &report)
            .map_err(|e| format!("Failed to write output file: {e}"))?;
        
        println!("✅ Generated Coverage Analysis Report");
        println!("   Type: {coverage_type:?}");
        println!("   Format: {format}");
        println!("   Output: {output_path}");
        println!("   Overall Score: {:.1}%", analysis.overall_score);
        
        // Show status
        let status = if analysis.overall_score >= 90.0 {
            "✅ EXCELLENT"
        } else if analysis.overall_score >= 80.0 {
            "✅ GOOD"
        } else if analysis.overall_score >= 70.0 {
            "⚠️ NEEDS IMPROVEMENT"
        } else {
            "❌ CRITICAL"
        };
        println!("   Status: {status}");
        
        if !analysis.gap_analysis.critical_gaps.is_empty() {
            println!("   Critical Gaps: {}", analysis.gap_analysis.critical_gaps.len());
        }
        
    } else {
        println!("{report}");
    }
    
    Ok(())
}

/// Print coverage help
fn print_trace_coverage_help() {
    println!("Analyze traceability coverage and identify gaps\n");
    println!("USAGE:");
    println!("    qms trace coverage [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --type <TYPE>        Coverage type to analyze (requirements, tests, risks)");
    println!("    --output <FILE>      Output file path (default: stdout)");
    println!("    --format <FORMAT>    Output format (text, csv, json)");
    println!("    --help               Show this help message\n");
    println!("COVERAGE TYPES:");
    println!("    requirements         Requirements coverage analysis");
    println!("    tests                Test coverage analysis");
    println!("    risks                Risk coverage analysis\n");
    println!("FORMATS:");
    println!("    text                 Human-readable dashboard format (default)");
    println!("    csv                  Comma-separated values");
    println!("    json                 JSON format for programmatic use\n");
    println!("METRICS:");
    println!("    - Requirements coverage: % of requirements with test cases");
    println!("    - Test coverage: % of test cases linked to requirements");
    println!("    - Verification coverage: % of requirements with verification evidence");
    println!("    - Risk coverage: % of risks with mitigation measures");
    println!("    - Overall score: Weighted average of all coverage metrics\n");
    println!("EXAMPLES:");
    println!("    qms trace coverage --type requirements");
    println!("    qms trace coverage --type tests --format csv --output test_coverage.csv");
    println!("    qms trace coverage --format json --output coverage_report.json");
    println!("    qms trace coverage --type risks --output risk_analysis.txt");
}

/// Handle trace impact command
fn handle_trace_impact(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print_trace_impact_help();
        return Ok(());
    }

    let requirement_id = &args[0];
    let mut change_description = String::new();
    let mut output_file = None;
    let mut format = "text".to_string();
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--analyze-change" => {
                if i + 1 < args.len() {
                    change_description = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--analyze-change requires a value".to_string());
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
            "--format" => {
                if i + 1 < args.len() {
                    format = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--format requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown option: {}", args[i]));
            }
        }
    }

    if change_description.is_empty() {
        return Err("Change description is required. Use --analyze-change \"description\"".to_string());
    }

    // Create impact analyzer
    let project_root = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?;
    let analyzer = ImpactAnalyzer::new(&project_root)
        .map_err(|e| format!("Failed to create impact analyzer: {e}"))?;
    
    // Analyze impact
    let analysis = analyzer.analyze_requirement_impact(requirement_id, &change_description)
        .map_err(|e| format!("Impact analysis failed: {e}"))?;
    
    // Format report
    let report = match format.as_str() {
        "text" => format_impact_analysis(&analysis),
        "json" => {
            // Simple JSON serialization for the analysis
            format!(
                r#"{{
  "source_entity_id": "{}",
  "change_description": "{}",
  "analysis_timestamp": "{}",
  "total_effort_estimate": {},
  "direct_impacts": {},
  "indirect_impacts": {},
  "critical_path_items": {},
  "risk_assessment": "{}",
  "stakeholder_count": {}
}}"#,
                analysis.source_entity_id,
                analysis.change_description,
                analysis.analysis_timestamp,
                analysis.total_effort_estimate,
                analysis.direct_impacts.len(),
                analysis.indirect_impacts.len(),
                analysis.critical_path_items.len(),
                analysis.risk_assessment,
                analysis.stakeholder_summary.len()
            )
        }
        "csv" => {
            let mut csv_report = "Entity ID,Entity Type,Impact Level,Effort Hours,Stakeholders\n".to_string();
            for impact in &analysis.direct_impacts {
                csv_report.push_str(&format!(
                    "{},{:?},{:?},{},{}\n",
                    impact.entity_id,
                    impact.entity_type,
                    impact.impact_level,
                    impact.estimated_effort_hours.unwrap_or(0),
                    impact.stakeholders.join(";")
                ));
            }
            for impact in &analysis.indirect_impacts {
                csv_report.push_str(&format!(
                    "{},{:?},{:?},{},{}\n",
                    impact.entity_id,
                    impact.entity_type,
                    impact.impact_level,
                    impact.estimated_effort_hours.unwrap_or(0),
                    impact.stakeholders.join(";")
                ));
            }
            csv_report
        }
        _ => return Err(format!("Unknown format: {format}")),
    };
    
    // Output results
    if let Some(output_path) = output_file {
        std::fs::write(&output_path, &report)
            .map_err(|e| format!("Failed to write output file: {e}"))?;
        
        println!("✅ Generated Impact Analysis Report");
        println!("   Requirement: {requirement_id}");
        println!("   Change: {change_description}");
        println!("   Format: {format}");
        println!("   Output: {output_path}");
        println!("   Direct Impacts: {}", analysis.direct_impacts.len());
        println!("   Indirect Impacts: {}", analysis.indirect_impacts.len());
        println!("   Total Effort: {} hours", analysis.total_effort_estimate);
        println!("   Critical Items: {}", analysis.critical_path_items.len());
        
        // Show risk assessment
        println!("   Risk Assessment: {}", analysis.risk_assessment);
        
        if !analysis.critical_path_items.is_empty() {
            println!("   ⚠️  Critical Path Items:");
            for item in &analysis.critical_path_items {
                println!("     • {item}");
            }
        }
        
    } else {
        println!("{report}");
    }
    
    Ok(())
}

/// Handle trace verification command
fn handle_trace_verify(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print_trace_verify_help();
        return Ok(());
    }

    use crate::modules::traceability::verification::{RequirementVerificationManager, VerificationMethod, VerificationStatus};

    let mut verification_manager = RequirementVerificationManager::new("./verification.json");
    if let Err(e) = verification_manager.load() {
        eprintln!("Warning: Could not load verification data: {e}");
    }

    let mut requirement_id = String::new();
    let mut method = String::new();
    let mut evidence_id = String::new();
    let mut status = String::new();
    let mut output_file = String::new();
    let mut show_report = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--requirement" | "-r" => {
                if i + 1 < args.len() {
                    requirement_id = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --requirement requires a value");
                    return Err("Missing requirement ID".to_string());
                }
            }
            "--method" | "-m" => {
                if i + 1 < args.len() {
                    method = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --method requires a value");
                    return Err("Missing method".to_string());
                }
            }
            "--evidence" | "-e" => {
                if i + 1 < args.len() {
                    evidence_id = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --evidence requires a value");
                    return Err("Missing evidence ID".to_string());
                }
            }
            "--status" | "-s" => {
                if i + 1 < args.len() {
                    status = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --status requires a value");
                    return Err("Missing status".to_string());
                }
            }
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_file = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --output requires a value");
                    return Err("Missing output file".to_string());
                }
            }
            "--report" => {
                show_report = true;
                i += 1;
            }
            _ => {
                if requirement_id.is_empty() {
                    requirement_id = args[i].clone();
                }
                i += 1;
            }
        }
    }

    // Handle report generation
    if show_report {
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
        return Ok(());
    }

    // Validate required parameters
    if requirement_id.is_empty() {
        eprintln!("Error: Requirement ID is required");
        print_trace_verify_help();
        return Err("Missing requirement ID".to_string());
    }

    // Add verification if method and evidence are provided
    if !method.is_empty() && !evidence_id.is_empty() {
        let verification_method = VerificationMethod::from_str(&method)
            .ok_or_else(|| format!("Invalid verification method: {method}"))?;

        if let Err(e) = verification_manager.add_verification(&requirement_id, verification_method, &evidence_id) {
            eprintln!("Error adding verification: {e}");
            return Err(e.to_string());
        }

        println!("✅ Added verification for requirement {requirement_id} with method {method} and evidence {evidence_id}");
    }

    // Update status if provided
    if !status.is_empty() {
        let verification_status = VerificationStatus::from_str(&status)
            .ok_or_else(|| format!("Invalid verification status: {status}"))?;

        if let Err(e) = verification_manager.update_verification_status(&requirement_id, verification_status) {
            eprintln!("Error updating verification status: {e}");
            return Err(e.to_string());
        }

        println!("✅ Updated verification status for requirement {requirement_id} to {status}");
    }

    // Save verification data
    if let Err(e) = verification_manager.save() {
        eprintln!("Error saving verification data: {e}");
        return Err(e.to_string());
    }

    // Show verification status
    if let Some(verification) = verification_manager.get_verification(&requirement_id) {
        println!("
📋 Verification Status for {requirement_id}");
        println!("   Status: {} {}", verification.verification_status.color(), verification.verification_status.to_string());
        println!("   Method: {}", verification.verification_method.to_string());
        println!("   Evidence Count: {}", verification.evidence.len());
        
        if let Some(verified_at) = &verification.verified_at {
            println!("   Verified: {verified_at}");
        }
        
        if let Some(verified_by) = &verification.verified_by {
            println!("   Verified By: {verified_by}");
        }
    }

    Ok(())
}

/// Print verification help
fn print_trace_verify_help() {
    println!("Manage requirement verification
");
    println!("USAGE:");
    println!("    qms trace verify <REQUIREMENT_ID> [OPTIONS]
");
    println!("ARGUMENTS:");
    println!("    <REQUIREMENT_ID>      ID of the requirement to verify
");
    println!("OPTIONS:");
    println!("    --method <METHOD>     Verification method (test, analysis, inspection, demonstration)");
    println!("    --evidence <ID>       Evidence ID linking to verification");
    println!("    --status <STATUS>     Verification status (not_verified, partially_verified, fully_verified)");
    println!("    --report              Generate verification report");
    println!("    --output <FILE>       Output file path (for report)");
    println!("    --help                Show this help message
");
    println!("VERIFICATION METHODS:");
    println!("    test                  Testing and verification through test execution");
    println!("    analysis              Analysis and mathematical verification");
    println!("    inspection            Visual inspection and code review");
    println!("    demonstration         Demonstration of functionality
");
    println!("VERIFICATION STATUS:");
    println!("    not_verified          No verification performed");
    println!("    partially_verified    Some verification evidence exists");
    println!("    fully_verified        Complete verification with all evidence
");
    println!("EXAMPLES:");
    println!("    qms trace verify REQ-001 --method test --evidence TC-001");
    println!("    qms trace verify REQ-002 --status fully_verified");
    println!("    qms trace verify REQ-003 --method analysis --evidence AR-001 --status partially_verified");
    println!("    qms trace verify --report --output verification_report.txt");
}

/// Print impact help
fn print_trace_impact_help() {
    println!("Analyze impact of requirement changes
");
    println!("USAGE:");
    println!("    qms trace impact <REQUIREMENT_ID> --analyze-change <DESCRIPTION> [OPTIONS]
");
    println!("ARGUMENTS:");
    println!("    <REQUIREMENT_ID>      ID of the requirement being changed (e.g., REQ-001)");
    println!("    --analyze-change      Description of the change being made
");
    println!("OPTIONS:");
    println!("    --output <FILE>       Output file path (default: stdout)");
    println!("    --format <FORMAT>     Output format (text, json, csv)");
    println!("    --help                Show this help message
");
    println!("FORMATS:");
    println!("    text                  Human-readable impact analysis report (default)");
    println!("    json                  JSON format for programmatic use");
    println!("    csv                   CSV format for spreadsheet analysis
");
    println!("ANALYSIS INCLUDES:");
    println!("    - Direct impacts: Entities directly linked to the requirement");
    println!("    - Indirect impacts: Second-degree impacts through linked entities");
    println!("    - Effort estimates: Time required to address each impact");
    println!("    - Risk assessment: Overall risk level of the change");
    println!("    - Stakeholder analysis: Affected team members and roles");
    println!("    - Critical path: High-priority items requiring immediate attention");
    println!("    - Recommendations: Suggested actions for managing the change
");
    println!("EXAMPLES:");
    println!("    qms trace impact REQ-001 --analyze-change \"Updated authentication method\"");
    println!("    qms trace impact REQ-002 --analyze-change \"Changed data validation rules\" --output impact.txt");
    println!("    qms trace impact REQ-003 --analyze-change \"Modified API interface\" --format json --output impact.json");
    println!("    qms trace impact REQ-004 --analyze-change \"Enhanced security requirements\" --format csv --output impact.csv");
}

fn print_trace_forward_help() {
    println!("Trace forward from source entity to all connected targets\n");
    println!("USAGE:");
    println!("    qms trace forward <ENTITY_ID>\n");
    println!("ARGUMENTS:");
    println!("    <ENTITY_ID>   Entity ID to trace forward from (e.g., REQ-001)\n");
    println!("DESCRIPTION:");
    println!("    Follows traceability links forward from the source entity to show all");
    println!("    connected targets. This reveals the complete downstream impact of a");
    println!("    requirement, showing what designs, tests, and other entities depend on it.\n");
    println!("    Forward tracing follows the direction: Requirements → Design → Tests → Results\n");
    println!("OUTPUT:");
    println!("    Displays a hierarchical tree showing:");
    println!("    - Source entity information");
    println!("    - All directly linked targets");
    println!("    - Recursively linked entities (indirect dependencies)");
    println!("    - Link types and relationship depth");
    println!("    - Maximum traceability depth reached\n");
    println!("EXAMPLES:");
    println!("    qms trace forward REQ-001   # Trace forward from requirement REQ-001");
    println!("    qms trace forward TC-001    # Trace forward from test case TC-001");
    println!("    qms trace forward RISK-001  # Trace forward from risk RISK-001");
}

fn print_trace_backward_help() {
    println!("Trace backward from target entity to all connected sources\n");
    println!("USAGE:");
    println!("    qms trace backward <ENTITY_ID>\n");
    println!("ARGUMENTS:");
    println!("    <ENTITY_ID>   Entity ID to trace backward from (e.g., TC-001)\n");
    println!("DESCRIPTION:");
    println!("    Follows traceability links backward from the target entity to show all");
    println!("    connected sources. This reveals the complete upstream justification for");
    println!("    a test case, design, or other entity.\n");
    println!("    Backward tracing follows the direction: Tests ← Design ← Requirements ← Stakeholder Needs\n");
    println!("OUTPUT:");
    println!("    Displays a hierarchical tree showing:");
    println!("    - Target entity information");
    println!("    - All directly linked sources");
    println!("    - Recursively linked entities (indirect sources)");
    println!("    - Link types and relationship depth");
    println!("    - Maximum traceability depth reached\n");
    println!("EXAMPLES:");
    println!("    qms trace backward TC-001    # Trace backward from test case TC-001");
    println!("    qms trace backward REQ-001   # Trace backward from requirement REQ-001");
    println!("    qms trace backward DOC-001   # Trace backward from document DOC-001");
}

fn print_trace_orphans_help() {
    println!("Find orphaned items with no traceability links\n");
    println!("USAGE:");
    println!("    qms trace orphans\n");
    println!("DESCRIPTION:");
    println!("    Identifies all entities in the QMS that have no traceability links");
    println!("    (neither incoming nor outgoing). Orphaned items may indicate:");
    println!("    - Missing traceability links");
    println!("    - Obsolete entities that should be archived");
    println!("    - Incomplete requirements analysis\n");
    println!("ENTITY TYPES ANALYZED:");
    println!("    - Requirements (REQ-*)");
    println!("    - Test Cases (TC-*)");
    println!("    - Risks (RISK-*)");
    println!("    - Documents (DOC-*)\n");
    println!("OUTPUT:");
    println!("    Displays orphaned items grouped by type:");
    println!("    - Entity ID and type");
    println!("    - Reason for orphaned status");
    println!("    - Count by entity type");
    println!("    - Recommendations for resolution\n");
    println!("EXAMPLES:");
    println!("    qms trace orphans    # Find all orphaned items");
    println!("\nNOTE: Use 'qms trace link' to create missing traceability links");
}

fn print_trace_import_help() {
    println!("Import traceability links from CSV/JSON files\n");
    println!("USAGE:");
    println!("    qms trace import [OPTIONS] <FILE>\n");
    println!("ARGUMENTS:");
    println!("    <FILE>    Path to import file (CSV or JSON format)\n");
    println!("OPTIONS:");
    println!("    -f, --file <FILE>       Import file path");
    println!("    --format <FORMAT>       File format (csv, json) [default: csv]\n");
    println!("DESCRIPTION:");
    println!("    Imports traceability links from external files. Supports bulk import");
    println!("    of requirements, test cases, and other entities with their traceability");
    println!("    relationships. Includes validation and duplicate detection.\n");
    println!("CSV FORMAT:");
    println!("    Header: SourceType,SourceID,TargetType,TargetID,LinkType,CreatedBy");
    println!("    Example: Requirement,REQ-001,TestCase,TC-001,Verifies,user123\n");
    println!("JSON FORMAT:");
    println!("    Array of link objects with source_type, source_id, target_type,");
    println!("    target_id, link_type, and created_by fields.\n");
    println!("SUPPORTED LINK TYPES:");
    println!("    - Verifies: Target verifies source");
    println!("    - Implements: Target implements source");
    println!("    - DependsOn: Target depends on source");
    println!("    - DerivedFrom: Target derives from source");
    println!("    - Related: General relationship");
    println!("    - Conflicts: Target conflicts with source");
    println!("    - Duplicates: Target duplicates source\n");
    println!("EXAMPLES:");
    println!("    qms trace import requirements.csv");
    println!("    qms trace import --file links.json --format json");
    println!("    qms trace import --format csv data/traceability.csv");
}

fn print_trace_export_help() {
    println!("Export traceability data to various formats\n");
    println!("USAGE:");
    println!("    qms trace export [OPTIONS] [OUTPUT_FILE]\n");
    println!("ARGUMENTS:");
    println!("    [OUTPUT_FILE]    Output file path [default: export.csv]\n");
    println!("OPTIONS:");
    println!("    -o, --output <FILE>     Output file path");
    println!("    -f, --format <FORMAT>   Export format (csv, json, dot) [default: csv]");
    println!("    -t, --type <TYPE>       Export type (rtm, matrix, graph) [default: rtm]\n");
    println!("DESCRIPTION:");
    println!("    Exports traceability data in various formats for analysis, reporting,");
    println!("    and integration with external tools. Supports Requirements Traceability");
    println!("    Matrix (RTM) and dependency graph exports.\n");
    println!("EXPORT TYPES:");
    println!("    rtm, matrix    Requirements Traceability Matrix");
    println!("    graph, dot     Dependency graph in DOT format\n");
    println!("SUPPORTED FORMATS:");
    println!("    csv      Comma-separated values (for RTM)");
    println!("    json     JSON format (for RTM)");
    println!("    dot      GraphViz DOT format (for graphs)\n");
    println!("EXAMPLES:");
    println!("    qms trace export rtm.csv");
    println!("    qms trace export --format json --output matrix.json");
    println!("    qms trace export --type graph --format dot --output deps.dot");
    println!("    qms trace export --type rtm --format csv --output traceability.csv");
}