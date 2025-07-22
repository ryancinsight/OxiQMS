use std::process;
use crate::modules::audit_logger::{
    AuditConfig,
    initialize_audit_system, get_audit_statistics,
    rotate_audit_logs, set_current_session, clear_current_session,
    verify_audit_file, export_chain_verification_report,
    check_and_rotate_daily_logs, cleanup_old_logs_comprehensive, get_rotation_statistics,
    ElectronicSignatureManager, format_signature_verification,
    ExportFormat, ExportOptions, AuditExportEngine, format_export_stats,
    AuditBackupManager, format_backup_stats, format_backup_info,
    PerformanceAuditLogger, PerformanceConfig, format_performance_metrics
};
use crate::utils::{get_current_project_path, generate_uuid};

pub fn handle_audit_command(args: &[String]) -> Result<(), String> {
    if args.len() < 3 {
        print_audit_help();
        return Ok(());
    }

    match args[2].as_str() {
        "init" => handle_audit_init(&args[3..]),
        "stats" => handle_audit_stats(&args[3..]),
        "search" => handle_audit_search(&args[3..]),
        "export" => handle_audit_export(&args[3..]),
        "compliance" => handle_audit_compliance(&args[3..]),
        "validate" => handle_audit_validate(&args[3..]),
        "rotate" => handle_audit_rotate(&args[3..]),
        "cleanup" => handle_audit_cleanup(&args[3..]),
        "login" => handle_audit_login(&args[3..]),
        "logout" => handle_audit_logout(&args[3..]),
        "verify" => handle_audit_verify(&args[3..]),
        "dashboard" => handle_audit_dashboard(&args[3..]),
        "signature" => handle_audit_signature(&args[3..]),
        "backup" => handle_audit_backup(&args[3..]),
        "performance" | "perf" => handle_audit_performance(&args[3..]),
        "--help" | "-h" => {
            print_audit_help();
            Ok(())
        }
        _ => {
            eprintln!("Error: Unknown audit command '{}'", args[2]);
            print_audit_help();
            process::exit(1);
        }
    }
}

fn handle_audit_init(_args: &[String]) -> Result<(), String> {
    println!("Initializing audit system...");
    
    let project_path = get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;
    
    let config = AuditConfig {
        project_path: project_path.to_string_lossy().to_string(),
        retention_days: 2555, // 7 years for medical device compliance
        daily_rotation: true,
        max_file_size_mb: 100,
        require_checksums: true,
    };
    
    initialize_audit_system(config)
        .map_err(|e| format!("Failed to initialize audit system: {e}"))?;
    
    println!("Audit system initialized successfully!");
    println!("- Daily log rotation enabled");
    println!("- Retention period: 7 years (medical device compliance)");
    println!("- Checksums enabled for integrity verification");
    println!("- Thread-safe operation enabled");
    
    Ok(())
}

fn handle_audit_stats(_args: &[String]) -> Result<(), String> {
    println!("Audit Statistics:");
    println!("================");
    
    match get_audit_statistics() {
        Ok(stats) => {
            println!("Total entries: {}", stats.total_entries);
            println!("Entries today: {}", stats.entries_today);
            println!("Log files: {}", stats.file_count);
            println!("Total size: {} bytes ({:.2} MB)", 
                     stats.total_size_bytes, 
                     stats.total_size_bytes as f64 / 1_048_576.0);
            
            if let Some(oldest) = &stats.oldest_entry_date {
                println!("Oldest entry: {oldest}");
            }
            if let Some(newest) = &stats.newest_entry_date {
                println!("Newest entry: {newest}");
            }
        }
        Err(e) => {
            println!("Failed to get audit statistics: {e}");
            println!("Note: Run 'qms audit init' to initialize the audit system first");
        }
    }
    
    Ok(())
}

fn handle_audit_search(args: &[String]) -> Result<(), String> {
    use crate::modules::audit_logger::search::{AuditSearchCriteria, AuditSearchEngine, AuditOutputFormat, format_search_results, parse_date_to_timestamp};

    let mut criteria = AuditSearchCriteria::new();
    let mut format = AuditOutputFormat::Table;
    let mut output_file: Option<String> = None;
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--user" => {
                if i + 1 < args.len() {
                    criteria = criteria.with_user(&args[i + 1]);
                    i += 2;
                } else {
                    return Err("--user requires a value".to_string());
                }
            }
            "--action" => {
                if i + 1 < args.len() {
                    criteria = criteria.with_action(&args[i + 1]);
                    i += 2;
                } else {
                    return Err("--action requires a value".to_string());
                }
            }
            "--entity-type" => {
                if i + 1 < args.len() {
                    criteria = criteria.with_entity_type(&args[i + 1]);
                    i += 2;
                } else {
                    return Err("--entity-type requires a value".to_string());
                }
            }
            "--entity-id" => {
                if i + 1 < args.len() {
                    criteria = criteria.with_entity_id(&args[i + 1]);
                    i += 2;
                } else {
                    return Err("--entity-id requires a value".to_string());
                }
            }
            "--keyword" => {
                if i + 1 < args.len() {
                    criteria = criteria.with_details_keyword(&args[i + 1]);
                    i += 2;
                } else {
                    return Err("--keyword requires a value".to_string());
                }
            }
            "--date-start" => {
                if i + 1 < args.len() {
                    let timestamp = parse_date_to_timestamp(&args[i + 1])
                        .map_err(|e| format!("Invalid start date: {e}"))?;
                    if let Some(end) = criteria.date_end {
                        criteria = criteria.with_date_range(timestamp, end);
                    } else {
                        criteria.date_start = Some(timestamp);
                    }
                    i += 2;
                } else {
                    return Err("--date-start requires a value (YYYY-MM-DD)".to_string());
                }
            }
            "--date-end" => {
                if i + 1 < args.len() {
                    let timestamp = parse_date_to_timestamp(&args[i + 1])
                        .map_err(|e| format!("Invalid end date: {e}"))?;
                    if let Some(start) = criteria.date_start {
                        criteria = criteria.with_date_range(start, timestamp);
                    } else {
                        criteria.date_end = Some(timestamp);
                    }
                    i += 2;
                } else {
                    return Err("--date-end requires a value (YYYY-MM-DD)".to_string());
                }
            }
            "--limit" => {
                if i + 1 < args.len() {
                    let limit: usize = args[i + 1].parse()
                        .map_err(|_| "Invalid limit value")?;
                    criteria = criteria.with_limit(limit);
                    i += 2;
                } else {
                    return Err("--limit requires a number".to_string());
                }
            }
            "--offset" => {
                if i + 1 < args.len() {
                    let offset: usize = args[i + 1].parse()
                        .map_err(|_| "Invalid offset value")?;
                    criteria = criteria.with_offset(offset);
                    i += 2;
                } else {
                    return Err("--offset requires a number".to_string());
                }
            }
            "--format" => {
                if i + 1 < args.len() {
                    format = match args[i + 1].as_str() {
                        "table" => AuditOutputFormat::Table,
                        "json" => AuditOutputFormat::Json,
                        "csv" => AuditOutputFormat::Csv,
                        _ => return Err("Invalid format. Use: table, json, csv".to_string()),
                    };
                    i += 2;
                } else {
                    return Err("--format requires a value".to_string());
                }
            }
            "--output" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--output requires a file path".to_string());
                }
            }
            "--help" | "-h" => {
                print_audit_search_help();
                return Ok(());
            }
            _ => {
                return Err(format!("Unknown search option: {}", args[i]));
            }
        }
    }

    let project_path = get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;

    let search_engine = AuditSearchEngine::new(project_path);
    let results = search_engine.search(&criteria)
        .map_err(|e| format!("Search failed: {e}"))?;

    let output = format_search_results(&results, &format);

    if let Some(file_path) = output_file {
        std::fs::write(&file_path, &output)
            .map_err(|e| format!("Failed to write to file {file_path}: {e}"))?;
        println!("Search results saved to: {file_path}");
    } else {
        println!("{output}");
    }

    Ok(())
}

fn handle_audit_export(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        print_audit_export_help();
        return Ok(());
    }

    let mut format = "json".to_string();
    let mut output_file = String::new();
    let mut filter: Option<String> = None;
    let mut max_entries: Option<usize> = None;
    let mut report_type = "audit_log".to_string(); // audit_log, summary, compliance

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--format" => {
                if i + 1 < args.len() {
                    format = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--format requires a value".to_string());
                }
            }
            "--output" => {
                if i + 1 < args.len() {
                    output_file = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--output requires a value".to_string());
                }
            }
            "--filter" => {
                if i + 1 < args.len() {
                    filter = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--filter requires a value".to_string());
                }
            }
            "--max-entries" => {
                if i + 1 < args.len() {
                    max_entries = Some(args[i + 1].parse()
                        .map_err(|_| "Invalid number for --max-entries")?);
                    i += 2;
                } else {
                    return Err("--max-entries requires a value".to_string());
                }
            }
            "--type" => {
                if i + 1 < args.len() {
                    report_type = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--type requires a value".to_string());
                }
            }
            "--help" | "-h" => {
                print_audit_export_help();
                return Ok(());
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }

    // Set default output file if not specified
    if output_file.is_empty() {
        let export_format = ExportFormat::from_string(&format)
            .map_err(|e| format!("Invalid format: {e}"))?;
        output_file = format!("audit_export.{}", export_format.extension());
    }

    let project_path = get_current_project_path().map_err(|e| e.to_string())?;
    let export_engine = AuditExportEngine::new(project_path);
    
    let export_format = ExportFormat::from_string(&format)
        .map_err(|e| format!("Invalid format: {e}"))?;
    
    let mut options = ExportOptions::new(export_format, output_file.into());
    
    if let Some(filter_str) = filter {
        options = options.with_filter(filter_str);
    }
    
    if let Some(max) = max_entries {
        options = options.with_max_entries(max);
    }

    println!("Exporting audit data...");
    
    let stats = match report_type.as_str() {
        "audit_log" | "log" => {
            export_engine.export_audit_logs(&options)
                .map_err(|e| format!("Export failed: {e}"))?
        }
        "summary" => {
            export_engine.generate_activity_summary(&options)
                .map_err(|e| format!("Summary generation failed: {e}"))?
        }
        "compliance" => {
            export_engine.generate_compliance_report(&options)
                .map_err(|e| format!("Compliance report generation failed: {e}"))?
        }
        _ => {
            return Err(format!("Unknown report type: {report_type}. Use 'audit_log', 'summary', or 'compliance'"));
        }
    };

    println!("{}", format_export_stats(&stats));
    println!("Export saved to: {}", options.output_path.display());
    
    Ok(())
}
fn handle_audit_rotate(args: &[String]) -> Result<(), String> {
    let project_path = get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;

    let mut force_rotation = false;
    let mut show_stats = false;

    // Parse arguments
    for arg in args {
        match arg.as_str() {
            "--force" => force_rotation = true,
            "--stats" => show_stats = true,
            _ => {}
        }
    }

    if show_stats {
        // Show rotation statistics
        match get_rotation_statistics(&project_path) {
            Ok(stats) => {
                println!("Audit Log Rotation Statistics");
                println!("==============================");
                println!("Daily files: {}", stats.daily_files_count);
                println!("Total daily log size: {:.2} MB", stats.total_daily_size as f64 / 1_048_576.0);
                println!("Compressed files: {}", stats.compressed_files_count);
                
                if let Some(oldest) = stats.oldest_daily_file {
                    println!("Oldest daily file: {oldest} (Unix timestamp)");
                }
                
                if let Some(newest) = stats.newest_daily_file {
                    println!("Newest daily file: {newest} (Unix timestamp)");
                }
                
                return Ok(());
            }
            Err(e) => {
                return Err(format!("Failed to get rotation statistics: {e}"));
            }
        }
    }

    if force_rotation {
        // Force immediate rotation
        println!("Performing forced log rotation...");
        match rotate_audit_logs() {
            Ok(message) => {
                println!("✓ {message}");
            }
            Err(e) => {
                return Err(format!("Failed to rotate audit logs: {e}"));
            }
        }
    } else {
        // Check if daily rotation is needed
        println!("Checking for daily rotation requirements...");
        match check_and_rotate_daily_logs(&project_path) {
            Ok(rotated) => {
                if rotated {
                    println!("✓ Daily log rotation completed");
                } else {
                    println!("ℹ️  No rotation needed - logs are current");
                }
            }
            Err(e) => {
                return Err(format!("Failed to check daily rotation: {e}"));
            }
        }
    }

    Ok(())
}

fn handle_audit_cleanup(args: &[String]) -> Result<(), String> {
    let project_path = get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;

    let mut confirm = false;
    let mut retention_days: Option<u32> = None;
    let mut dry_run = false;
    
    // Parse arguments
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--confirm" => confirm = true,
            "--dry-run" => dry_run = true,
            "--retention-days" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u32>() {
                        Ok(days) => {
                            retention_days = Some(days);
                            i += 1; // Skip next argument
                        }
                        Err(_) => {
                            return Err("Invalid retention days value".to_string());
                        }
                    }
                } else {
                    return Err("--retention-days requires a value".to_string());
                }
            }
            _ => {}
        }
        i += 1;
    }
    
    let retention = retention_days.unwrap_or(2555); // Default 7 years
    
    if dry_run {
        println!("DRY RUN: Audit Log Cleanup Simulation");
        println!("=====================================");
        println!("Retention period: {} days ({:.1} years)", retention, retention as f64 / 365.25);
        println!("This would clean up files older than the retention period.");
        println!("Use --confirm to perform actual cleanup.");
        return Ok(());
    }
    
    if !confirm {
        println!("Audit Log Cleanup");
        println!("================");
        println!("This will permanently delete old audit log files based on retention policy.");
        println!("Retention period: {} days ({:.1} years)", retention, retention as f64 / 365.25);
        println!();
        println!("Options:");
        println!("  --confirm           Proceed with cleanup");
        println!("  --retention-days N  Set retention period (default: 2555 days)");
        println!("  --dry-run          Show what would be deleted without deleting");
        return Ok(());
    }

    println!("Performing audit log cleanup...");
    println!("Retention period: {retention} days");
    
    match cleanup_old_logs_comprehensive(&project_path, retention) {
        Ok(report) => {
            println!("\n📊 Cleanup Report:");
            println!("==================");
            println!("Files deleted: {}", report.files_deleted);
            println!("Files compressed: {}", report.files_compressed);
            println!("Bytes freed: {:.2} MB", report.bytes_freed as f64 / 1_048_576.0);
            
            if !report.errors.is_empty() {
                println!("\n⚠️  Warnings:");
                for error in &report.errors {
                    println!("  {error}");
                }
            }
            
            if report.files_deleted > 0 || report.files_compressed > 0 {
                println!("\n✅ Cleanup completed successfully!");
            } else {
                println!("\nℹ️  No cleanup needed - all files are within retention period");
            }
        }
        Err(e) => {
            return Err(format!("Failed to cleanup audit logs: {e}"));
        }
    }

    Ok(())
}

fn handle_audit_login(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Usage: qms audit login <username>".to_string());
    }
    
    let username = &args[0];
    let session_id = generate_uuid();
    
    match set_current_session(username.clone(), session_id.clone(), None) {
        Ok(()) => {
            println!("Audit session started for user: {username}");
            println!("Session ID: {session_id}");
        }
        Err(e) => {
            return Err(format!("Failed to start audit session: {e}"));
        }
    }
    
    Ok(())
}

fn handle_audit_logout(_args: &[String]) -> Result<(), String> {
    match clear_current_session() {
        Ok(()) => {
            println!("Audit session ended");
        }
        Err(e) => {
            return Err(format!("Failed to end audit session: {e}"));
        }
    }
    
    Ok(())
}

fn handle_audit_verify(args: &[String]) -> Result<(), String> {
    println!("Audit Trail Integrity Verification");
    println!("==================================");
    
    let project_path = get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;
    
    let audit_dir = project_path.join("audit");
    
    // Check for --report flag to export verification report
    let export_report = args.contains(&"--report".to_string());
    let report_path = if export_report {
        Some(audit_dir.join("integrity_verification_report.md"))
    } else {
        None
    };
    
    // Verify main audit log
    let main_log = audit_dir.join("audit.log");
    if main_log.exists() {
        println!("\n🔍 Verifying main audit log...");
        match verify_audit_file(&main_log) {
            Ok(()) => {
                println!("✅ Main audit log verification passed");
                
                // Export report if requested
                if let Some(ref report_path) = report_path {
                    match export_chain_verification_report(&main_log, report_path) {
                        Ok(()) => println!("📄 Verification report exported to: {}", report_path.display()),
                        Err(e) => eprintln!("⚠️  Failed to export report: {e}"),
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Main audit log verification FAILED: {e}");
                return Err("Audit trail integrity compromised".to_string());
            }
        }
    } else {
        println!("ℹ️  No main audit log found - system may be new");
    }
    
    // Verify daily logs
    let daily_dir = audit_dir.join("daily");
    if daily_dir.exists() {
        println!("\n🔍 Verifying daily audit logs...");
        
        let mut verified_files = 0;
        let mut failed_files = 0;
        
        if let Ok(entries) = std::fs::read_dir(&daily_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "log") {
                    match verify_audit_file(&path) {
                        Ok(()) => {
                            verified_files += 1;
                            println!("  ✅ {}", path.file_name().unwrap().to_string_lossy());
                        }
                        Err(e) => {
                            failed_files += 1;
                            eprintln!("  ❌ {}: {}", path.file_name().unwrap().to_string_lossy(), e);
                        }
                    }
                }
            }
        }
        
        println!("\nDaily logs summary:");
        println!("  Verified: {verified_files} files");
        println!("  Failed: {failed_files} files");
        
        if failed_files > 0 {
            return Err("Some daily audit logs failed verification".to_string());
        }
    } else {
        println!("ℹ️  No daily audit logs found");
    }
    
    // Display basic audit statistics
    match get_audit_statistics() {
        Ok(stats) => {
            println!("\n📊 Audit System Statistics:");
            println!("  Total log files: {}", stats.file_count);
            println!("  Total entries: {}", stats.total_entries);
            println!("  Latest entry: {}", stats.newest_entry_date.unwrap_or_else(|| "None".to_string()));
        }
        Err(e) => {
            eprintln!("⚠️  Could not retrieve audit statistics: {e}");
        }
    }
    
    println!("\n🎉 Audit trail integrity verification completed successfully!");
    println!("   The immutable audit trail is secure and has not been tampered with.");
    
    if export_report {
        println!("\n📋 Use 'qms audit verify --report' to generate detailed verification reports");
    }
    
    Ok(())
}

fn print_audit_help() {
    println!("Manage QMS audit trail\n");
    println!("USAGE:");
    println!("    qms audit <COMMAND>\n");
    println!("COMMANDS:");
    println!("    init       Initialize audit system for current project");
    println!("    stats      Show audit trail statistics");
    println!("    search     Search audit trail with filters");
    println!("               --user <username>       Filter by user");
    println!("               --action <action>       Filter by action (create, update, etc.)");
    println!("               --entity-type <type>    Filter by entity type");
    println!("               --entity-id <id>        Filter by specific entity ID");
    println!("               --keyword <text>        Search in details field");
    println!("               --date-start <date>     Filter by start date (YYYY-MM-DD)");
    println!("               --date-end <date>       Filter by end date (YYYY-MM-DD)");
    println!("               --limit <num>           Limit number of results");
    println!("               --offset <num>          Skip first N results");
    println!("               --format <fmt>          Output format (table, json, csv)");
    println!("               --output <file>         Save to file");
    println!("    export     Export complete audit trail");
    println!("               --format <fmt>          Output format (json, csv)");
    println!("               --output <file>         Output file path");
    println!("    rotate     Manage audit log rotation");
    println!("               --force              Force immediate rotation");
    println!("               --stats              Show rotation statistics");
    println!("    cleanup    Clean up old audit logs");
    println!("               --confirm            Proceed with cleanup");
    println!("               --retention-days N   Set retention period (default: 2555)");
    println!("               --dry-run           Show what would be deleted");
    println!("    login      Start audit session for user");
    println!("    logout     End current audit session");
    println!("    verify     Verify audit trail integrity and hash chain");
    println!("               --report                Export verification report");
    println!("    dashboard  Generate comprehensive audit dashboard");
    println!("               --period <days>         Analysis period (default: 30)");
    println!("               --output <file>         Save to file");
    println!("    signature  Manage electronic signatures (21 CFR Part 11)");
    println!("               create                  Create new signature");
    println!("               verify <id>             Verify signature");
    println!("               list                    List entity signatures");
    println!("               requirements            Show requirements");
    println!("    compliance Generate 21 CFR Part 11 compliance report");
    println!("               --period <period>       Report period description");
    println!("               --output <file>         Save report to file");
    println!("               --format <format>       Output format (table, json)");
    println!("    validate   Validate 21 CFR Part 11 compliance");
    println!("               Quick validation of audit trail compliance");
    println!("    backup     Manage audit log backups");
    println!("               create                  Create new backup");
    println!("               list                    List available backups");
    println!("               restore <id>            Restore from backup");
    println!("               verify                  Verify backup integrity");
    println!("               cleanup                 Clean up old backups");
    println!("               info <id>               Show backup details");
    println!("    performance Audit performance optimization (alias: perf)");
    println!("               metrics                 Show performance metrics");
    println!("               configure               Configure performance settings");
    println!("               index                   Manage search indexes");
    println!("               search                  Fast indexed search");
    println!("               benchmark               Run performance benchmark");
    println!("               verify [id]             Verify backup integrity");
    println!("               cleanup                 Clean up old backups");
    println!("               info <id>               Show backup information");
    println!("    help       Show this help message\n");
    println!("EXAMPLES:");
    println!("    qms audit search --user john --action update");
    println!("    qms audit export --format csv --output audit_report.csv");
    println!("    qms audit search --entity-type Document --format json");
    println!("    qms audit search --keyword \"document\" --date-start 2025-07-01");
    println!("\nFor more information on a specific command, use:");
    println!("    qms audit <COMMAND> --help");
}

fn print_audit_search_help() {
    println!("Search audit trail entries with advanced filtering\n");
    println!("USAGE:");
    println!("    qms audit search [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --user <username>        Filter by user ID");
    println!("    --action <action>        Filter by action type");
    println!("    --entity-type <type>     Filter by entity type (Document, Risk, etc.)");
    println!("    --entity-id <id>         Filter by specific entity ID");
    println!("    --keyword <text>         Search for keyword in details field");
    println!("    --date-start <date>      Start date filter (YYYY-MM-DD)");
    println!("    --date-end <date>        End date filter (YYYY-MM-DD)");
    println!("    --limit <num>            Maximum number of results (default: 100)");
    println!("    --offset <num>           Skip first N results for pagination");
    println!("    --format <fmt>           Output format: table, json, csv (default: table)");
    println!("    --output <file>          Save results to file");
    println!("    --help, -h              Show this help message\n");
    println!("EXAMPLES:");
    println!("    qms audit search --user testuser");
    println!("    qms audit search --action Create --entity-type Document");
    println!("    qms audit search --keyword \"approved\" --format json");
    println!("    qms audit search --date-start 2025-07-01 --date-end 2025-07-15");
    println!("    qms audit search --limit 50 --offset 100  # Pagination");
    println!("    qms audit search --output audit_results.json --format json");
}

fn handle_audit_compliance(args: &[String]) -> Result<(), String> {
    use crate::modules::audit_logger::regulatory::{RegulatoryCompliance, format_compliance_report};

    let mut report_period = "current".to_string();
    let mut output_file: Option<String> = None;
    let mut _format = "table".to_string();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_audit_compliance_help();
                return Ok(());
            }
            "--period" => {
                if i + 1 < args.len() {
                    report_period = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--period requires a value".to_string());
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
                    _format = args[i + 1].clone();
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

    let project_path = get_current_project_path().map_err(|e| e.to_string())?;
    let regulatory = RegulatoryCompliance::new(project_path);

    println!("Generating 21 CFR Part 11 compliance report...");
    
    match regulatory.generate_compliance_report(&report_period) {
        Ok(report) => {
            let formatted_report = format_compliance_report(&report);

            if let Some(output_path) = output_file {
                std::fs::write(&output_path, &formatted_report)
                    .map_err(|e| format!("Failed to write report to file: {e}"))?;
                println!("✅ Compliance report saved to: {output_path}");
            } else {
                println!("{formatted_report}");
            }
        }
        Err(e) => {
            return Err(format!("Failed to generate compliance report: {e}"));
        }
    }

    Ok(())
}

fn handle_audit_validate(args: &[String]) -> Result<(), String> {
    use crate::modules::audit_logger::regulatory::RegulatoryCompliance;

    if !args.is_empty() && (args[0] == "--help" || args[0] == "-h") {
        print_audit_validate_help();
        return Ok(());
    }

    let project_path = get_current_project_path().map_err(|e| e.to_string())?;
    let regulatory = RegulatoryCompliance::new(project_path);

    println!("Validating 21 CFR Part 11 compliance...");
    
    match regulatory.validate_cfr_part_11_compliance() {
        Ok(validation) => {
            let summary = &validation.validation_summary;
            
            println!("\n21 CFR PART 11 COMPLIANCE VALIDATION");
            println!("====================================");
            
            // Overall status
            if validation.is_compliant {
                println!("✅ Overall Status: COMPLIANT");
            } else {
                println!("❌ Overall Status: NON-COMPLIANT");
            }
            
            println!("📊 Compliance Score: {:.1}%", summary.cfr_part_11_score);
            println!();
            
            // Validation details
            println!("VALIDATION SUMMARY:");
            println!("- Total Entries Checked: {}", summary.total_entries_checked);
            println!("- Compliant Entries: {}", summary.compliant_entries);
            println!("- Critical Issues: {}", summary.critical_issues);
            println!("- Warning Issues: {}", summary.warning_issues);
            println!("- Info Issues: {}", summary.info_issues);
            println!("- Hash Chain Integrity: {}", 
                if summary.hash_chain_integrity { "✅ Valid" } else { "❌ Invalid" });
            
            // Issues details
            if !validation.issues.is_empty() {
                println!("\nCOMPLIANCE ISSUES:");
                for issue in &validation.issues {
                    let severity_icon = match issue.severity {
                        crate::modules::audit_logger::regulatory::IssueSeverity::Critical => "🔴",
                        crate::modules::audit_logger::regulatory::IssueSeverity::Warning => "🟡",
                        crate::modules::audit_logger::regulatory::IssueSeverity::Info => "🔵",
                    };
                    println!("{} {:?}: {}", severity_icon, issue.severity, issue.description);
                    if let Some(ref entry_id) = issue.affected_entry {
                        println!("   Affected Entry: {entry_id}");
                    }
                }
            }
            
            // Next steps
            if !validation.is_compliant {
                println!("\nNEXT STEPS:");
                println!("1. Address all critical issues immediately");
                println!("2. Verify and restore hash chain integrity if needed");
                println!("3. Re-run validation after fixes");
                println!("4. Generate compliance report for documentation");
            } else {
                println!("\n✅ System is compliant with 21 CFR Part 11 requirements");
                println!("💡 Consider generating a compliance report for documentation");
            }
        }
        Err(e) => {
            return Err(format!("Failed to validate compliance: {e}"));
        }
    }

    Ok(())
}

fn print_audit_compliance_help() {
    println!("Generate 21 CFR Part 11 compliance reports");
    println!("USAGE:");
    println!("    qms audit compliance [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --period <period>        Report period (default: current)");
    println!("    --output <file>          Save report to file");
    println!("    --format <format>        Output format: table, json (default: table)");
    println!("    --help, -h              Show this help message\n");
    println!("EXAMPLES:");
    println!("    qms audit compliance");
    println!("    qms audit compliance --period \"Q1 2025\" --output compliance_q1.txt");
    println!("    qms audit compliance --format json --output compliance.json");
}

fn print_audit_validate_help() {
    println!("Validate 21 CFR Part 11 compliance for audit trail");
    println!("USAGE:");
    println!("    qms audit validate [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --help, -h              Show this help message\n");
    println!("VALIDATION CHECKS:");
    println!("    - Required fields (user ID, timestamp, action)");
    println!("    - Timestamp format compliance");
    println!("    - Electronic signatures for critical actions");
    println!("    - Hash chain integrity for immutability");
    println!("    - Audit trail completeness");
    println!("\nEXAMPLES:");
    println!("    qms audit validate");
}

fn handle_audit_dashboard(args: &[String]) -> Result<(), String> {
    use crate::modules::audit_logger::dashboard::{AuditDashboardEngine, format_dashboard};

    let mut period_days = 30u32; // Default to 30 days
    let mut output_file: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_audit_dashboard_help();
                return Ok(());
            }
            "--period" => {
                if i + 1 < args.len() {
                    if let Some(period_str) = args[i + 1].strip_suffix("days") {
                        match period_str.parse::<u32>() {
                            Ok(days) => {
                                period_days = days;
                                i += 2;
                            }
                            Err(_) => {
                                return Err("Invalid period format. Use format like '30days'".to_string());
                            }
                        }
                    } else {
                        // Try parsing as just a number (assume days)
                        match args[i + 1].parse::<u32>() {
                            Ok(days) => {
                                period_days = days;
                                i += 2;
                            }
                            Err(_) => {
                                return Err("Invalid period value. Use number of days (e.g., 30)".to_string());
                            }
                        }
                    }
                } else {
                    return Err("--period requires a value".to_string());
                }
            }
            "--output" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--output requires a file path".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown dashboard option: {}", args[i]));
            }
        }
    }

    let project_path = get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;

    println!("Generating audit dashboard for {period_days} days...");

    let dashboard_engine = AuditDashboardEngine::new(project_path);
    match dashboard_engine.generate_dashboard(period_days) {
        Ok(dashboard) => {
            let formatted_dashboard = format_dashboard(&dashboard, period_days);

            if let Some(output_path) = output_file {
                std::fs::write(&output_path, &formatted_dashboard)
                    .map_err(|e| format!("Failed to write dashboard to file: {e}"))?;
                println!("✅ Dashboard saved to: {output_path}");
            } else {
                println!("{formatted_dashboard}");
            }
        }
        Err(e) => {
            return Err(format!("Failed to generate dashboard: {e}"));
        }
    }

    Ok(())
}

fn print_audit_dashboard_help() {
    println!("Generate comprehensive audit dashboard with metrics and analysis");
    println!("USAGE:");
    println!("    qms audit dashboard [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --period <days>          Analysis period in days (default: 30)");
    println!("    --output <file>          Save dashboard to file");
    println!("    --help, -h              Show this help message\n");
    println!("DASHBOARD SECTIONS:");
    println!("    - General metrics (total entries, daily activity)");
    println!("    - User activity analysis (most active users, patterns)");
    println!("    - Action metrics (critical actions, distributions)");
    println!("    - Time analysis (peak hours, daily trends)");
    println!("    - Security alerts (suspicious activity detection)");
    println!("    - Trend analysis (activity, compliance trends)");
    println!("\nEXAMPLES:");
    println!("    qms audit dashboard");
    println!("    qms audit dashboard --period 7");
    println!("    qms audit dashboard --period 30 --output dashboard.txt");
}

fn handle_audit_signature(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        print_audit_signature_help();
        return Ok(());
    }

    let project_path = get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;
    
    let signature_manager = ElectronicSignatureManager::new(project_path);

    match args[0].as_str() {
        "create" => handle_signature_create(&signature_manager, &args[1..]),
        "verify" => handle_signature_verify(&signature_manager, &args[1..]),
        "list" => handle_signature_list(&signature_manager, &args[1..]),
        "requirements" => handle_signature_requirements(&signature_manager, &args[1..]),
        "--help" | "-h" => {
            print_audit_signature_help();
            Ok(())
        }
        _ => {
            eprintln!("Unknown signature command: {}", args[0]);
            print_audit_signature_help();
            Ok(())
        }
    }
}

fn handle_signature_create(manager: &ElectronicSignatureManager, args: &[String]) -> Result<(), String> {
    let mut user_id = String::new();
    let mut action = String::new();
    let mut entity_type = String::new();
    let mut entity_id = String::new();
    let mut reason: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--user" => {
                if i + 1 < args.len() {
                    user_id = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--user requires a value".to_string());
                }
            }
            "--action" => {
                if i + 1 < args.len() {
                    action = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--action requires a value".to_string());
                }
            }
            "--entity-type" => {
                if i + 1 < args.len() {
                    entity_type = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--entity-type requires a value".to_string());
                }
            }
            "--entity-id" => {
                if i + 1 < args.len() {
                    entity_id = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--entity-id requires a value".to_string());
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
                return Err(format!("Unknown create option: {}", args[i]));
            }
        }
    }

    if user_id.is_empty() || action.is_empty() || entity_type.is_empty() || entity_id.is_empty() {
        return Err("Missing required parameters. Use: --user <user> --action <action> --entity-type <type> --entity-id <id>".to_string());
    }

    match manager.create_signature(user_id, &action, entity_type, entity_id, reason) {
        Ok(signature) => {
            println!("✅ Electronic signature created successfully!");
            println!("Signature ID: {}", signature.id);
            println!("User: {}", signature.user_id);
            println!("Meaning: {}", signature.meaning);
            println!("Timestamp: {}", signature.timestamp);
            println!("Hash: {}", signature.signature_hash);
        }
        Err(e) => {
            return Err(format!("Failed to create signature: {e}"));
        }
    }

    Ok(())
}

fn handle_signature_verify(manager: &ElectronicSignatureManager, args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Usage: qms audit signature verify <signature_id>".to_string());
    }

    let signature_id = &args[0];

    match manager.verify_signature(signature_id) {
        Ok(verification) => {
            println!("{}", format_signature_verification(&verification));
        }
        Err(e) => {
            return Err(format!("Failed to verify signature: {e}"));
        }
    }

    Ok(())
}

fn handle_signature_list(manager: &ElectronicSignatureManager, args: &[String]) -> Result<(), String> {
    let mut entity_type = String::new();
    let mut entity_id = String::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--entity-type" => {
                if i + 1 < args.len() {
                    entity_type = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--entity-type requires a value".to_string());
                }
            }
            "--entity-id" => {
                if i + 1 < args.len() {
                    entity_id = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--entity-id requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown list option: {}", args[i]));
            }
        }
    }

    if entity_type.is_empty() || entity_id.is_empty() {
        return Err("Usage: qms audit signature list --entity-type <type> --entity-id <id>".to_string());
    }

    match manager.list_signatures_for_entity(&entity_type, &entity_id) {
        Ok(signatures) => {
            if signatures.is_empty() {
                println!("No signatures found for {entity_type} {entity_id}");
            } else {
                println!("Electronic Signatures for {entity_type} {entity_id}:");
                println!("{:-<60}", "");
                for signature in &signatures {
                    println!("ID: {}", signature.id);
                    println!("  User: {}", signature.user_id);
                    println!("  Meaning: {}", signature.meaning);
                    println!("  Timestamp: {}", signature.timestamp);
                    println!("  Method: {:?}", signature.signature_method);
                    if let Some(ref reason) = signature.reason {
                        println!("  Reason: {reason}");
                    }
                    println!("  Valid: {}", if signature.verify_hash() { "✅ Yes" } else { "❌ No" });
                    println!();
                }
            }
        }
        Err(e) => {
            return Err(format!("Failed to list signatures: {e}"));
        }
    }

    Ok(())
}

fn handle_signature_requirements(manager: &ElectronicSignatureManager, _args: &[String]) -> Result<(), String> {
    let report = manager.generate_requirements_report();
    println!("{report}");
    Ok(())
}

fn handle_audit_backup(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        print_audit_backup_help();
        return Ok(());
    }

    let project_path = get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;
    
    let backup_manager = AuditBackupManager::new(project_path);

    match args[0].as_str() {
        "create" => handle_backup_create(&backup_manager, &args[1..]),
        "list" => handle_backup_list(&backup_manager, &args[1..]),
        "restore" => handle_backup_restore(&backup_manager, &args[1..]),
        "delete" => handle_backup_delete(&backup_manager, &args[1..]),
        "verify" => handle_backup_verify(&backup_manager, &args[1..]),
        "cleanup" => handle_backup_cleanup(&backup_manager, &args[1..]),
        "info" => handle_backup_info(&backup_manager, &args[1..]),
        "--help" | "-h" => {
            print_audit_backup_help();
            Ok(())
        }
        _ => {
            eprintln!("Unknown backup command: {}", args[0]);
            print_audit_backup_help();
            Ok(())
        }
    }
}

fn handle_backup_create(manager: &AuditBackupManager, args: &[String]) -> Result<(), String> {
    let mut confirm = false;

    // Parse arguments
    for arg in args {
        match arg.as_str() {
            "--confirm" => confirm = true,
            "--help" | "-h" => {
                print_backup_create_help();
                return Ok(());
            }
            _ => {
                return Err(format!("Unknown create option: {arg}"));
            }
        }
    }

    if !confirm {
        println!("Audit Backup Creation");
        println!("====================");
        println!("This will create a backup of all audit logs to ensure data preservation.");
        println!("Backup will include compression and integrity verification.");
        println!();
        println!("Use --confirm to proceed with backup creation.");
        return Ok(());
    }

    println!("Creating audit backup...");
    
    match manager.create_backup() {
        Ok(stats) => {
            println!("✅ Backup created successfully!");
            println!("{}", format_backup_stats(&stats));
        }
        Err(e) => {
            return Err(format!("Failed to create backup: {e}"));
        }
    }

    Ok(())
}

fn handle_backup_list(manager: &AuditBackupManager, args: &[String]) -> Result<(), String> {
    let mut show_details = false;

    // Parse arguments
    for arg in args {
        match arg.as_str() {
            "--details" => show_details = true,
            "--help" | "-h" => {
                print_backup_list_help();
                return Ok(());
            }
            _ => {
                return Err(format!("Unknown list option: {arg}"));
            }
        }
    }

    match manager.list_backups() {
        Ok(backups) => {
            if backups.is_empty() {
                println!("No audit backups found.");
                println!("Use 'qms audit backup create --confirm' to create your first backup.");
            } else {
                println!("Available Audit Backups");
                println!("======================");
                
                for backup in &backups {
                    if show_details {
                        println!("{}", format_backup_info(backup));
                        println!("{:-<60}", "");
                    } else {
                        let timestamp_str = if backup.timestamp > 0 {
                            let datetime = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(backup.timestamp);
                            format!("{datetime:?}")
                        } else {
                            "Unknown".to_string()
                        };
                        
                        println!("ID: {} | {} | {} files | {:.1} MB",
                            backup.backup_id,
                            timestamp_str,
                            backup.file_count,
                            backup.total_size as f64 / 1_048_576.0
                        );
                    }
                }
                
                if !show_details {
                    println!("\nUse --details to show full backup information");
                }
            }
        }
        Err(e) => {
            return Err(format!("Failed to list backups: {e}"));
        }
    }

    Ok(())
}

fn handle_backup_restore(manager: &AuditBackupManager, args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Usage: qms audit backup restore <backup_id> [--confirm]".to_string());
    }

    let backup_id = &args[0];
    let mut confirm = false;

    // Parse remaining arguments
    for arg in &args[1..] {
        match arg.as_str() {
            "--confirm" => confirm = true,
            "--help" | "-h" => {
                print_backup_restore_help();
                return Ok(());
            }
            _ => {
                return Err(format!("Unknown restore option: {arg}"));
            }
        }
    }

    if !confirm {
        println!("Audit Backup Restoration");
        println!("========================");
        println!("This will replace all current audit logs with the backup '{backup_id}'.");
        println!("Current audit logs will be backed up as 'pre_restore_<timestamp>' before restoration.");
        println!();
        println!("⚠️  WARNING: This operation cannot be undone (except by restoring the pre-restore backup).");
        println!();
        println!("Use --confirm to proceed with restoration.");
        return Ok(());
    }

    println!("Restoring audit backup: {backup_id}");
    
    match manager.restore_backup(backup_id) {
        Ok(()) => {
            println!("✅ Backup restored successfully!");
            println!("Current audit logs have been replaced with backup '{backup_id}'.");
            println!("Previous audit logs were backed up for safety.");
        }
        Err(e) => {
            return Err(format!("Failed to restore backup: {e}"));
        }
    }

    Ok(())
}

fn handle_backup_delete(manager: &AuditBackupManager, args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Usage: qms audit backup delete <backup_id> [--confirm]".to_string());
    }

    let backup_id = &args[0];
    let mut confirm = false;

    // Parse remaining arguments
    for arg in &args[1..] {
        match arg.as_str() {
            "--confirm" => confirm = true,
            "--help" | "-h" => {
                print_backup_delete_help();
                return Ok(());
            }
            _ => {
                return Err(format!("Unknown delete option: {arg}"));
            }
        }
    }

    if !confirm {
        println!("Audit Backup Deletion");
        println!("=====================");
        println!("This will permanently delete backup '{backup_id}'.");
        println!("This operation cannot be undone.");
        println!();
        println!("Use --confirm to proceed with deletion.");
        return Ok(());
    }

    // First verify the backup exists
    match manager.verify_backup(backup_id) {
        Ok(_) => {
            // For now, we'll simulate deletion by removing from metadata
            // In a real implementation, this would delete the actual backup files
            println!("⚠️  Backup deletion not yet fully implemented.");
            println!("This feature will be completed in the next development cycle.");
            Ok(())
        }
        Err(e) => {
            Err(format!("Backup verification failed: {e}"))
        }
    }
}

fn handle_backup_verify(manager: &AuditBackupManager, args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        // Verify all backups
        match manager.list_backups() {
            Ok(backups) => {
                if backups.is_empty() {
                    println!("No backups to verify.");
                    return Ok(());
                }

                println!("Verifying All Audit Backups");
                println!("===========================");
                
                let mut verified = 0;
                let mut failed = 0;
                
                for backup in &backups {
                    match manager.verify_backup(&backup.backup_id) {
                        Ok(is_valid) => {
                            if is_valid {
                                println!("✅ {} - Integrity verified", backup.backup_id);
                                verified += 1;
                            } else {
                                println!("❌ {} - Integrity check FAILED", backup.backup_id);
                                failed += 1;
                            }
                        }
                        Err(e) => {
                            println!("⚠️  {} - Verification error: {}", backup.backup_id, e);
                            failed += 1;
                        }
                    }
                }
                
                println!("\nVerification Summary:");
                println!("  Verified: {verified} backups");
                println!("  Failed: {failed} backups");
                
                if failed > 0 {
                    return Err("Some backups failed verification".to_string());
                }
            }
            Err(e) => {
                return Err(format!("Failed to list backups: {e}"));
            }
        }
    } else {
        // Verify specific backup
        let backup_id = &args[0];
        
        println!("Verifying backup: {backup_id}");
        
        match manager.verify_backup(backup_id) {
            Ok(is_valid) => {
                if is_valid {
                    println!("✅ Backup integrity verified successfully");
                } else {
                    println!("❌ Backup integrity check FAILED");
                    println!("The backup may be corrupted or tampered with.");
                    return Err("Backup verification failed".to_string());
                }
            }
            Err(e) => {
                return Err(format!("Failed to verify backup: {e}"));
            }
        }
    }

    Ok(())
}

fn handle_backup_cleanup(manager: &AuditBackupManager, args: &[String]) -> Result<(), String> {
    let mut confirm = false;
    let mut dry_run = false;

    // Parse arguments
    for arg in args {
        match arg.as_str() {
            "--confirm" => confirm = true,
            "--dry-run" => dry_run = true,
            "--help" | "-h" => {
                print_backup_cleanup_help();
                return Ok(());
            }
            _ => {
                return Err(format!("Unknown cleanup option: {arg}"));
            }
        }
    }

    if dry_run {
        println!("DRY RUN: Backup Cleanup Simulation");
        println!("===================================");
        println!("This would clean up old backups based on retention policy.");
        println!("Use --confirm to perform actual cleanup.");
        return Ok(());
    }

    if !confirm {
        println!("Backup Cleanup");
        println!("==============");
        println!("This will permanently delete old backup files based on retention policy.");
        println!("Current retention: 7 years (2555 days) for medical device compliance");
        println!();
        println!("Options:");
        println!("  --confirm    Proceed with cleanup");
        println!("  --dry-run   Show what would be deleted without deleting");
        return Ok(());
    }

    println!("Performing backup cleanup...");
    
    match manager.cleanup_old_backups() {
        Ok(deleted_count) => {
            if deleted_count > 0 {
                println!("✅ Cleanup completed successfully!");
                println!("Deleted {deleted_count} old backup(s)");
            } else {
                println!("ℹ️  No cleanup needed - all backups are within retention period");
            }
        }
        Err(e) => {
            return Err(format!("Failed to cleanup backups: {e}"));
        }
    }

    Ok(())
}

fn handle_backup_info(manager: &AuditBackupManager, args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("Usage: qms audit backup info <backup_id>".to_string());
    }

    let backup_id = &args[0];
    
    match manager.list_backups() {
        Ok(backups) => {
            if let Some(backup) = backups.iter().find(|b| b.backup_id == *backup_id) {
                println!("Audit Backup Information");
                println!("=======================");
                println!("{}", format_backup_info(backup));
                
                // Verify integrity
                println!("\nIntegrity Check:");
                match manager.verify_backup(backup_id) {
                    Ok(is_valid) => {
                        if is_valid {
                            println!("✅ Backup integrity verified");
                        } else {
                            println!("❌ Backup integrity check FAILED");
                        }
                    }
                    Err(e) => {
                        println!("⚠️  Verification error: {e}");
                    }
                }
            } else {
                return Err(format!("Backup '{backup_id}' not found"));
            }
        }
        Err(e) => {
            return Err(format!("Failed to get backup information: {e}"));
        }
    }

    Ok(())
}

fn print_audit_export_help() {
    println!("Export audit logs in various formats for reporting and compliance");
    println!("USAGE:");
    println!("    qms audit export [OPTIONS]");
    println!("OPTIONS:");
    println!("    --format <FORMAT>       Export format: pdf, csv, json, xml (default: json)");
    println!("    --output <FILE>         Output file path (default: audit_export.<ext>)");
    println!("    --filter <FILTER>       Filter criteria: user:john,action:create,entity_type:Document");
    println!("    --max-entries <NUM>     Maximum number of entries to export");
    println!("    --type <TYPE>           Report type: audit_log, summary, compliance (default: audit_log)");
    println!("    --help                  Show this help message");
    println!("EXAMPLES:");
    println!("    qms audit export --format csv --output report.csv");
    println!("    qms audit export --format pdf --filter user:john --output user_activity.pdf");
    println!("    qms audit export --type summary --format json --output summary.json");
    println!("    qms audit export --type compliance --format pdf --output compliance_report.pdf");
    println!("    qms audit export --filter action:delete --max-entries 100");
}
fn print_audit_backup_help() {
    println!("Manage audit log backups for data preservation and recovery");
    println!("USAGE:");
    println!("    qms audit backup <COMMAND>\n");
    println!("COMMANDS:");
    println!("    create      Create new backup of all audit logs");
    println!("                --confirm               Proceed with backup creation");
    println!("    list        List all available backups");
    println!("                --details               Show detailed backup information");
    println!("    restore     Restore from backup");
    println!("                <backup_id>             Backup to restore");
    println!("                --confirm               Proceed with restoration");
    println!("    delete      Delete a backup");
    println!("                <backup_id>             Backup to delete");
    println!("                --confirm               Proceed with deletion");
    println!("    verify      Verify backup integrity");
    println!("                [backup_id]             Specific backup to verify (or all if omitted)");
    println!("    cleanup     Clean up old backups");
    println!("                --confirm               Proceed with cleanup");
    println!("                --dry-run              Show what would be deleted");
    println!("    info        Show detailed information about a backup");
    println!("                <backup_id>             Backup to show info for");
    println!("    help        Show this help message\n");
    println!("FEATURES:");
    println!("    - Automatic compression for space efficiency");
    println!("    - Integrity verification with checksums");
    println!("    - 7-year retention for medical device compliance");
    println!("    - Safe restoration with pre-restore backups");
    println!("    - Comprehensive audit trail preservation\n");
    println!("EXAMPLES:");
    println!("    qms audit backup create --confirm");
    println!("    qms audit backup list --details");
    println!("    qms audit backup restore audit_backup_1721900000 --confirm");
    println!("    qms audit backup verify");
    println!("    qms audit backup cleanup --dry-run");
    println!("    qms audit backup info audit_backup_1721900000");
}

fn print_backup_create_help() {
    println!("Create a new backup of all audit logs");
    println!("USAGE:");
    println!("    qms audit backup create [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --confirm    Proceed with backup creation");
    println!("    --help, -h   Show this help message\n");
    println!("DESCRIPTION:");
    println!("    Creates a compressed backup of all audit logs including:");
    println!("    - Main audit log (audit.log)");
    println!("    - Daily rotated logs");
    println!("    - Electronic signature records");
    println!("    - Audit chain verification data");
    println!("\nEXAMPLES:");
    println!("    qms audit backup create --confirm");
}

fn print_backup_list_help() {
    println!("List all available audit backups");
    println!("USAGE:");
    println!("    qms audit backup list [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --details    Show detailed backup information");
    println!("    --help, -h   Show this help message\n");
    println!("DESCRIPTION:");
    println!("    Lists all available backups sorted by creation date (newest first)");
    println!("    Shows backup ID, timestamp, file count, and total size");
    println!("\nEXAMPLES:");
    println!("    qms audit backup list");
    println!("    qms audit backup list --details");
}

fn print_backup_restore_help() {
    println!("Restore audit logs from a backup");
    println!("USAGE:");
    println!("    qms audit backup restore <backup_id> [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --confirm    Proceed with restoration");
    println!("    --help, -h   Show this help message\n");
    println!("DESCRIPTION:");
    println!("    Restores all audit logs from the specified backup.");
    println!("    Current audit logs are backed up before restoration for safety.");
    println!("    This operation replaces all current audit data.");
    println!("\nEXAMPLES:");
    println!("    qms audit backup restore audit_backup_1721900000 --confirm");
}

fn print_backup_delete_help() {
    println!("Delete an audit backup");
    println!("USAGE:");
    println!("    qms audit backup delete <backup_id> [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --confirm    Proceed with deletion");
    println!("    --help, -h   Show this help message\n");
    println!("DESCRIPTION:");
    println!("    Permanently deletes the specified backup and all associated files.");
    println!("    This operation cannot be undone.");
    println!("\nEXAMPLES:");
    println!("    qms audit backup delete audit_backup_1721900000 --confirm");
}

fn print_backup_cleanup_help() {
    println!("Clean up old audit backups based on retention policy");
    println!("USAGE:");
    println!("    qms audit backup cleanup [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --confirm    Proceed with cleanup");
    println!("    --dry-run   Show what would be deleted without deleting");
    println!("    --help, -h   Show this help message\n");
    println!("DESCRIPTION:");
    println!("    Deletes backups older than the retention period (7 years for medical devices).");
    println!("    Use --dry-run to preview what would be deleted.");
    println!("\nEXAMPLES:");
    println!("    qms audit backup cleanup --dry-run");
    println!("    qms audit backup cleanup --confirm");
}fn print_audit_signature_help() {
    println!("Manage electronic signatures for 21 CFR Part 11 compliance");
    println!("USAGE:");
    println!("    qms audit signature <COMMAND>\n");
    println!("COMMANDS:");
    println!("    create      Create new electronic signature");
    println!("                --user <user>           Signing user");
    println!("                --action <action>       Action requiring signature");
    println!("                --entity-type <type>    Type of entity");
    println!("                --entity-id <id>        Entity ID");
    println!("                --reason <reason>       Reason for signature (if required)");
    println!("    verify      Verify electronic signature");
    println!("                <signature_id>          Signature to verify");
    println!("    list        List signatures for entity");
    println!("                --entity-type <type>    Entity type");
    println!("                --entity-id <id>        Entity ID");
    println!("    requirements Show signature requirements");
    println!("    help        Show this help message\n");
    println!("EXAMPLES:");
    println!("    qms audit signature create --user john --action document_approve --entity-type Document --entity-id DOC-001");
    println!("    qms audit signature verify 12345678-abcd-ef12-3456-789012345678");
    println!("    qms audit signature list --entity-type Document --entity-id DOC-001");
    println!("    qms audit signature requirements");
}/// Handle audit performance optimization commands
pub fn handle_audit_performance(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        print_audit_performance_help();
        return Ok(());
    }

    match args[0].as_str() {
        "metrics" => handle_performance_metrics(&args[1..]),
        "configure" | "config" => handle_performance_configure(&args[1..]),
        "index" => handle_performance_index(&args[1..]),
        "search" => handle_performance_search(&args[1..]),
        "benchmark" => handle_performance_benchmark(&args[1..]),
        "--help" | "-h" | "help" => {
            print_audit_performance_help();
            Ok(())
        }
        _ => {
            println!("❌ Unknown performance command: {}", args[0]);
            println!("Use 'qms audit performance --help' for usage information.");
            Ok(())
        }
    }
}

/// Handle performance metrics display
fn handle_performance_metrics(args: &[String]) -> Result<(), String> {
    let project_path = get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;

    // Create performance logger to get metrics
    let perf_logger = PerformanceAuditLogger::new(project_path);
    let metrics = perf_logger.get_metrics();

    // Check for reset flag
    let reset = args.contains(&"--reset".to_string());
    
    if reset {
        println!("⚠️  Metrics reset functionality requires active performance session");
        println!("   Start a performance session first with 'qms audit performance configure'");
    } else {
        println!("{}", format_performance_metrics(metrics));
    }

    if args.contains(&"--detailed".to_string()) {
        println!("\n📈 Additional Details:");
        println!("Cache efficiency: {:.1}%", metrics.cache_hit_ratio() * 100.0);
        println!("Operations per batch: {:.1}", 
            if metrics.flushed_batches > 0 { 
                metrics.buffered_writes as f64 / metrics.flushed_batches as f64 
            } else { 0.0 });
    }

    Ok(())
}

/// Handle performance configuration
fn handle_performance_configure(args: &[String]) -> Result<(), String> {
    let project_path = get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;

    if args.contains(&"--help".to_string()) {
        print_performance_configure_help();
        return Ok(());
    }

    // Parse configuration options
    let mut config = PerformanceConfig::default();
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "--buffer-size" => {
                if i + 1 < args.len() {
                    config.buffer_size = args[i + 1].parse()
                        .map_err(|_| "Invalid buffer size value")?;
                    i += 2;
                } else {
                    return Err("--buffer-size requires a value".to_string());
                }
            }
            "--flush-interval" => {
                if i + 1 < args.len() {
                    config.flush_interval_ms = args[i + 1].parse()
                        .map_err(|_| "Invalid flush interval value")?;
                    i += 2;
                } else {
                    return Err("--flush-interval requires a value".to_string());
                }
            }
            "--cache-size" => {
                if i + 1 < args.len() {
                    config.cache_size = args[i + 1].parse()
                        .map_err(|_| "Invalid cache size value")?;
                    i += 2;
                } else {
                    return Err("--cache-size requires a value".to_string());
                }
            }
            "--disable-index" => {
                config.index_enabled = false;
                i += 1;
            }
            "--disable-batch" => {
                config.batch_write_enabled = false;
                i += 1;
            }
            _ => {
                return Err(format!("Unknown configuration option: {}", args[i]));
            }
        }
    }

    println!("⚙️  Performance Configuration Applied:");
    println!("Buffer size: {} entries", config.buffer_size);
    println!("Flush interval: {} ms", config.flush_interval_ms);
    println!("Cache size: {} entries", config.cache_size);
    println!("Indexing: {}", if config.index_enabled { "enabled" } else { "disabled" });
    println!("Batch writes: {}", if config.batch_write_enabled { "enabled" } else { "disabled" });

    // Create performance logger with configuration
    let mut perf_logger = PerformanceAuditLogger::with_config(project_path, config);
    
    // Build initial index
    perf_logger.build_index_from_logs()
        .map_err(|e| format!("Failed to build search index: {e}"))?;

    println!("✅ Performance optimization configured successfully!");
    
    Ok(())
}

/// Handle performance index operations
fn handle_performance_index(args: &[String]) -> Result<(), String> {
    let project_path = get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;

    if args.is_empty() || args[0] == "--help" || args[0] == "help" {
        print_performance_index_help();
        return Ok(());
    }

    match args[0].as_str() {
        "build" => {
            println!("🔍 Building search index from audit logs...");
            let mut perf_logger = PerformanceAuditLogger::new(project_path);
            
            perf_logger.build_index_from_logs()
                .map_err(|e| format!("Failed to build index: {e}"))?;
            
            println!("✅ Search index built successfully!");
            
            let metrics = perf_logger.get_metrics();
            println!("📊 Index contains {} indexed operations", metrics.total_operations);
        }
        "stats" => {
            println!("📈 Index Statistics:");
            println!("Performance indexing provides fast search across:");
            println!("  • User-based searches");
            println!("  • Action-based searches");
            println!("  • Entity-based searches");
            println!("  • Date-based searches");
            println!("\nUse 'qms audit performance search' for indexed searching");
        }
        _ => {
            return Err(format!("Unknown index command: {}", args[0]));
        }
    }

    Ok(())
}

/// Handle performance-optimized search
fn handle_performance_search(args: &[String]) -> Result<(), String> {
    let project_path = get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;

    if args.contains(&"--help".to_string()) {
        print_performance_search_help();
        return Ok(());
    }

    let mut perf_logger = PerformanceAuditLogger::new(project_path);
    
    // Parse search criteria
    let mut user = None;
    let mut action = None;
    let mut entity_id = None;
    let mut date = None;
    let mut i = 0;
    
    while i < args.len() {
        match args[i].as_str() {
            "--user" => {
                if i + 1 < args.len() {
                    user = Some(args[i + 1].as_str());
                    i += 2;
                } else {
                    return Err("--user requires a value".to_string());
                }
            }
            "--action" => {
                if i + 1 < args.len() {
                    action = Some(args[i + 1].as_str());
                    i += 2;
                } else {
                    return Err("--action requires a value".to_string());
                }
            }
            "--entity-id" => {
                if i + 1 < args.len() {
                    entity_id = Some(args[i + 1].as_str());
                    i += 2;
                } else {
                    return Err("--entity-id requires a value".to_string());
                }
            }
            "--date" => {
                if i + 1 < args.len() {
                    date = Some(args[i + 1].as_str());
                    i += 2;
                } else {
                    return Err("--date requires a value".to_string());
                }
            }
            _ => i += 1,
        }
    }

    if user.is_none() && action.is_none() && entity_id.is_none() && date.is_none() {
        return Err("At least one search criteria must be specified".to_string());
    }

    println!("🔍 Performing indexed search...");
    
    let timestamps = perf_logger.indexed_search(user, action, entity_id, date)
        .map_err(|e| format!("Search failed: {e}"))?;

    println!("📊 Search Results:");
    println!("Found {} matching entries", timestamps.len());
    
    if !timestamps.is_empty() {
        println!("\nTimestamps (newest first):");
        for (i, timestamp) in timestamps.iter().take(10).enumerate() {
            println!("  {}. {}", i + 1, timestamp);
        }
        
        if timestamps.len() > 10 {
            println!("  ... and {} more entries", timestamps.len() - 10);
        }
    }

    let metrics = perf_logger.get_metrics();
    println!("\n⚡ Performance: {:.2}ms average search time", metrics.avg_search_time_ms);
    println!("💾 Cache hit ratio: {:.1}%", metrics.cache_hit_ratio() * 100.0);

    Ok(())
}

/// Handle performance benchmarking
fn handle_performance_benchmark(args: &[String]) -> Result<(), String> {
    let project_path = get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;

    if args.contains(&"--help".to_string()) {
        print_performance_benchmark_help();
        return Ok(());
    }

    println!("🏃 Running audit performance benchmark...");
    
    let mut perf_logger = PerformanceAuditLogger::new(project_path);
    
    // Benchmark buffered writes
    println!("\n📝 Testing buffered write performance...");
    let test_entries = 100;
    let start_time = std::time::Instant::now();
    
    for i in 0..test_entries {
        let test_entry = format!(
            r#"{{
    "timestamp": {},
    "user_id": "test_user_{}",
    "action": "benchmark_test",
    "entity_type": "TestEntity",
    "entity_id": "TEST-{:03}",
    "details": "Performance benchmark entry"
}}"#,
            i, i % 5, i
        );
        
        perf_logger.buffer_entry(&test_entry)
            .map_err(|e| format!("Failed to buffer entry: {e}"))?;
    }
    
    let buffer_duration = start_time.elapsed();
    println!("✅ Buffered {} entries in {:.2}ms", test_entries, buffer_duration.as_millis());
    
    // Flush buffer and measure
    let flush_start = std::time::Instant::now();
    perf_logger.flush_buffer()
        .map_err(|e| format!("Failed to flush buffer: {e}"))?;
    let flush_duration = flush_start.elapsed();
    println!("✅ Flushed buffer in {:.2}ms", flush_duration.as_millis());
    
    // Test search performance
    println!("\n🔍 Testing search performance...");
    let search_start = std::time::Instant::now();
    let results = perf_logger.indexed_search(Some("test_user_1"), None, None, None)
        .map_err(|e| format!("Search failed: {e}"))?;
    let search_duration = search_start.elapsed();
    println!("✅ Search completed in {:.2}ms, found {} results", 
        search_duration.as_millis(), results.len());
    
    // Display final metrics
    let metrics = perf_logger.get_metrics();
    println!("\n📊 Benchmark Results:");
    println!("Total operations: {}", metrics.total_operations);
    println!("Average write time: {:.2}ms", metrics.avg_write_time_ms);
    println!("Average search time: {:.2}ms", metrics.avg_search_time_ms);
    println!("Cache hit ratio: {:.1}%", metrics.cache_hit_ratio() * 100.0);
    
    println!("\n🎯 Performance Grade:");
    let grade = if metrics.avg_write_time_ms < 1.0 && metrics.avg_search_time_ms < 5.0 {
        "A+ Excellent"
    } else if metrics.avg_write_time_ms < 2.0 && metrics.avg_search_time_ms < 10.0 {
        "A Good"
    } else if metrics.avg_write_time_ms < 5.0 && metrics.avg_search_time_ms < 25.0 {
        "B Fair"
    } else {
        "C Needs Optimization"
    };
    println!("{grade}");

    Ok(())
}

/// Print audit performance help
fn print_audit_performance_help() {
    println!("Audit performance optimization and monitoring");
    println!("USAGE:");
    println!("    qms audit performance <COMMAND>\n");
    println!("COMMANDS:");
    println!("    metrics     Display performance metrics");
    println!("                --detailed              Show detailed metrics");
    println!("                --reset                 Reset metrics counters");
    println!("    configure   Configure performance settings");
    println!("                --buffer-size <size>    Entries to buffer (default: 100)");
    println!("                --flush-interval <ms>   Auto-flush interval (default: 5000)");
    println!("                --cache-size <size>     Cache size (default: 1000)");
    println!("                --disable-index         Disable search indexing");
    println!("                --disable-batch         Disable batch operations");
    println!("    index       Manage search indexes");
    println!("                build                   Build index from existing logs");
    println!("                stats                   Show index statistics");
    println!("    search      Fast indexed search");
    println!("                --user <user>           Search by user");
    println!("                --action <action>       Search by action");
    println!("                --entity-id <id>        Search by entity ID");
    println!("                --date <YYYY-MM-DD>     Search by date");
    println!("    benchmark   Run performance benchmark");
    println!("    help        Show this help message\n");
    println!("DESCRIPTION:");
    println!("    Performance optimization features for audit logging including:");
    println!("    • Buffered writes for reduced I/O");
    println!("    • Search indexing for fast queries");
    println!("    • Memory caching for frequent operations");
    println!("    • Batch operations for efficiency");
    println!("    • Stream processing for large files\n");
    println!("EXAMPLES:");
    println!("    qms audit performance metrics --detailed");
    println!("    qms audit performance configure --buffer-size 200 --cache-size 2000");
    println!("    qms audit performance index build");
    println!("    qms audit performance search --user john --action create");
    println!("    qms audit performance benchmark");
}

fn print_performance_configure_help() {
    println!("Configure audit performance optimization settings");
    println!("USAGE:");
    println!("    qms audit performance configure [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --buffer-size <size>      Number of entries to buffer before writing (default: 100)");
    println!("    --flush-interval <ms>     Auto-flush interval in milliseconds (default: 5000)");
    println!("    --cache-size <size>       Maximum cache entries (default: 1000)");
    println!("    --disable-index           Disable search indexing for better write performance");
    println!("    --disable-batch           Disable batch write operations");
    println!("    --help, -h                Show this help message\n");
    println!("DESCRIPTION:");
    println!("    Configures performance optimization settings for audit logging.");
    println!("    Higher buffer sizes improve write performance but use more memory.");
    println!("    Lower flush intervals improve data safety but reduce performance.");
    println!("    Larger caches improve read performance but use more memory.\n");
    println!("EXAMPLES:");
    println!("    qms audit performance configure --buffer-size 200");
    println!("    qms audit performance configure --flush-interval 1000 --cache-size 2000");
    println!("    qms audit performance configure --disable-index");
}

fn print_performance_index_help() {
    println!("Manage search indexes for fast audit log queries");
    println!("USAGE:");
    println!("    qms audit performance index <COMMAND>\n");
    println!("COMMANDS:");
    println!("    build       Build search index from existing audit logs");
    println!("    stats       Show index statistics and capabilities");
    println!("    help        Show this help message\n");
    println!("DESCRIPTION:");
    println!("    Search indexes enable fast queries by user, action, entity, and date.");
    println!("    Building indexes from existing logs enables fast searching of historical data.");
    println!("    Indexes are automatically maintained for new log entries.\n");
    println!("EXAMPLES:");
    println!("    qms audit performance index build");
    println!("    qms audit performance index stats");
}

fn print_performance_search_help() {
    println!("Fast indexed search for audit logs");
    println!("USAGE:");
    println!("    qms audit performance search [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --user <user>             Search by user ID");
    println!("    --action <action>         Search by action type");
    println!("    --entity-id <id>          Search by entity ID");
    println!("    --date <YYYY-MM-DD>       Search by date");
    println!("    --help, -h                Show this help message\n");
    println!("DESCRIPTION:");
    println!("    Performs fast indexed search across audit logs using built indexes.");
    println!("    Multiple criteria can be combined for refined searches.");
    println!("    Results are returned as timestamps for matching entries.\n");
    println!("EXAMPLES:");
    println!("    qms audit performance search --user john");
    println!("    qms audit performance search --action create --entity-id DOC-001");
    println!("    qms audit performance search --date 2024-01-15");
    println!("    qms audit performance search --user admin --action delete");
}

fn print_performance_benchmark_help() {
    println!("Run performance benchmark tests for audit logging");
    println!("USAGE:");
    println!("    qms audit performance benchmark [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --help, -h                Show this help message\n");
    println!("DESCRIPTION:");
    println!("    Runs comprehensive performance tests including:");
    println!("    • Buffered write performance");
    println!("    • Buffer flush timing");
    println!("    • Search index performance");
    println!("    • Cache hit ratio analysis");
    println!("    • Overall performance grading\n");
    println!("EXAMPLE:");
    println!("    qms audit performance benchmark");
}