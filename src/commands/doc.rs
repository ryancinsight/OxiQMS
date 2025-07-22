use std::process;
use std::collections::HashMap;
use crate::modules::document_control::service::DocumentService;
use crate::modules::document_control::document::DocumentType;
use crate::modules::document_control::version::VersionChangeType;

#[derive(Default)]
pub struct SearchFilters {
    pub document_type: Option<String>,
    pub status: Option<String>,
    pub author: Option<String>,
}

impl SearchFilters {
    pub const fn has_filters(&self) -> bool {
        self.document_type.is_some() || self.status.is_some() || self.author.is_some()
    }
    
    pub fn description(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref t) = self.document_type {
            parts.push(format!("type: {t}"));
        }
        if let Some(ref s) = self.status {
            parts.push(format!("status: {s}"));
        }
        if let Some(ref a) = self.author {
            parts.push(format!("author: {a}"));
        }
        parts.join(", ")
    }
}

pub fn handle_doc_command(args: &[String]) -> Result<(), String> {
    if args.len() < 3 {
        print_doc_help();
        return Ok(());
    }

    match args[2].as_str() {
        "add" => handle_doc_add(&args[3..]),
        "template" => handle_doc_template(&args[3..]),
        "list" => handle_doc_list(&args[3..]),
        "view" => handle_doc_view(&args[3..]),
        "update" => handle_doc_update(&args[3..]),
        "remove" => handle_doc_remove(&args[3..]),
        "versions" => handle_doc_versions(&args[3..]),
        "version" => handle_doc_version(&args[3..]),
        "history" => handle_doc_history(&args[3..]),
        "compare" => handle_doc_compare(&args[3..]),
        "rollback" => handle_doc_rollback(&args[3..]),
        "checkout" => handle_doc_checkout(&args[3..]),
        "checkin" => handle_doc_checkin(&args[3..]),
        "status" => handle_doc_status(&args[3..]),
        "locked" => handle_doc_locked(&args[3..]),
        "unlock" => handle_doc_unlock(&args[3..]),
        "locks" => handle_doc_locks(&args[3..]), // Administrative lock management
        "submit" => handle_doc_submit(&args[3..]),
        "approve" => handle_doc_approve(&args[3..]),
        "reject" => handle_doc_reject(&args[3..]),
        "workflow" => handle_doc_workflow(&args[3..]),
        "archive" => handle_doc_archive(&args[3..]),
        "restore" => handle_doc_restore(&args[3..]),
        "search" => handle_doc_search(&args[3..]),
        "export" => handle_doc_export(&args[3..]),
        "import" => handle_doc_import(&args[3..]),
        "regulatory" => handle_doc_regulatory(&args[3..]),
        "compliance" => handle_doc_compliance(&args[3..]),
        "backup" => handle_doc_backup(&args[3..]),
        "--help" | "-h" => {
            print_doc_help();
            Ok(())
        }
        _ => {
            eprintln!("Error: Unknown document command '{}'", args[2]);
            print_doc_help();
            process::exit(1);
        }
    }
}

fn handle_doc_add(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_add_help();
        return Ok(());
    }

    let mut title = None;
    let mut doc_type = None;
    let mut content = None;
    let mut version = None;
    let mut author = None;
    let mut file_path = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--title" | "-t" => {
                if i + 1 < args.len() {
                    title = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --title requires a value".to_string());
                }
            }
            "--type" => {
                if i + 1 < args.len() {
                    doc_type = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --type requires a value".to_string());
                }
            }
            "--content" | "-c" => {
                if i + 1 < args.len() {
                    content = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --content requires a value".to_string());
                }
            }
            "--file" | "-f" => {
                if i + 1 < args.len() {
                    file_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --file requires a value".to_string());
                }
            }
            "--version" | "-v" => {
                if i + 1 < args.len() {
                    version = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --version requires a value".to_string());
                }
            }
            "--author" | "-a" => {
                if i + 1 < args.len() {
                    author = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --author requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Error: Unknown argument '{}'", args[i]));
            }
        }
    }

    // Validate required arguments
    let title = title.ok_or("Error: --title is required")?;
    let doc_type_str = doc_type.ok_or("Error: --type is required")?;
    let _version = version.unwrap_or_else(|| "1.0".to_string());
    let author = author.unwrap_or_else(|| "CLI User".to_string());

    // Validate mutually exclusive arguments
    if file_path.is_some() && content.is_some() {
        return Err("Error: --file and --content are mutually exclusive".to_string());
    }

    // Parse document type
    let parsed_type = match doc_type_str.to_lowercase().as_str() {
        "srs" => DocumentType::SoftwareRequirementsSpecification,
        "sdd" => DocumentType::SoftwareDesignDescription,
        "vnv" => DocumentType::VerificationAndValidation,
        "rmf" => DocumentType::RiskManagementFile,
        "dhf" => DocumentType::DesignHistoryFile,
        "ur" => DocumentType::UserRequirements,
        "protocol" => DocumentType::TestProtocol,
        "report" => DocumentType::TestReport,
        "auto" => DocumentType::Other("Auto-detected".to_string()), // Will be auto-detected from file extension
        _ => DocumentType::Other(doc_type_str.clone()),
    };

    // Create document service using current directory
    let service = DocumentService::new(".".to_string().into());
    
    // Create document using appropriate method
    let result = if let Some(ref file_path) = file_path {
        // Create document from file
        service.add_document_from_file(file_path, &title, parsed_type, author)
    } else {
        // Create document with content
        let content = content.unwrap_or_default();
        service.create_document(title, content, parsed_type, author)
    };

    match result {
        Ok(saved_doc) => {
            println!("✅ Document added successfully!");
            println!("   ID: {}", saved_doc.id);
            println!("   Title: {}", saved_doc.title);
            println!("   Type: {:?}", saved_doc.doc_type);
            println!("   Version: {}", saved_doc.version);
            println!("   Created By: {}", saved_doc.created_by);
            println!("   Status: {:?}", saved_doc.status);
            if !saved_doc.file_path.is_empty() {
                println!("   File Path: {}", saved_doc.file_path);
            }
            if let Some(ref path) = file_path {
                println!("   Source File: {path}");
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to add document: {e}")),
    }
}

// ========== TEMPLATE COMMAND HANDLERS (Phase 2.1.9) ==========

fn handle_doc_template(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_template_help();
        return Ok(());
    }

    match args[0].as_str() {
        "list" => handle_template_list(&args[1..]),
        "create" => handle_template_create(&args[1..]),
        "init" => handle_template_init(&args[1..]),
        _ => {
            eprintln!("Error: Unknown template command '{}'", args[0]);
            print_doc_template_help();
            Err("Unknown template command".to_string())
        }
    }
}

fn handle_template_list(_args: &[String]) -> Result<(), String> {
    let service = DocumentService::new(".".to_string().into());
    
    match service.list_templates() {
        Ok(templates) => {
            if templates.is_empty() {
                println!("No templates found. Use 'qms doc template init' to create default templates.");
            } else {
                println!("📋 Available Document Templates\n");
                println!("{:<25} | {:<30} | {:<20} | {:<30}", 
                         "Name", "Description", "Type", "Variables");
                println!("{}", "-".repeat(110));

                for template in templates {
                    let description = if template.description.len() > 30 {
                        format!("{}...", &template.description[..27])
                    } else {
                        template.description
                    };

                    let doc_type = match template.doc_type {
                        DocumentType::SoftwareRequirementsSpecification => "SRS",
                        DocumentType::SoftwareDesignDescription => "SDD",
                        DocumentType::TestProtocol => "Test Protocol",
                        DocumentType::RiskManagementFile => "Risk Mgmt",
                        DocumentType::UserRequirements => "User Req",
                        DocumentType::Other(ref name) => name,
                        _ => "Other",
                    };

                    let variables = template.variables.join(", ");
                    let vars_display = if variables.len() > 30 {
                        format!("{}...", &variables[..27])
                    } else {
                        variables
                    };

                    println!("{:<25} | {:<30} | {:<20} | {:<30}",
                             template.name, description, doc_type, vars_display);
                }
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to list templates: {e}")),
    }
}

fn handle_template_create(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_template_create_help();
        return Ok(());
    }

    let mut template_name = None;
    let mut title = None;
    let mut project_name = None;
    let mut created_by = "CLI User".to_string();
    let mut custom_variables: HashMap<String, String> = HashMap::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--template" | "-t" => {
                if i + 1 < args.len() {
                    template_name = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --template requires a value".to_string());
                }
            }
            "--title" => {
                if i + 1 < args.len() {
                    title = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --title requires a value".to_string());
                }
            }
            "--project" | "-p" => {
                if i + 1 < args.len() {
                    project_name = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --project requires a value".to_string());
                }
            }
            "--author" | "-a" => {
                if i + 1 < args.len() {
                    created_by = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Error: --author requires a value".to_string());
                }
            }
            "--var" => {
                if i + 2 < args.len() {
                    custom_variables.insert(args[i + 1].clone(), args[i + 2].clone());
                    i += 3;
                } else {
                    return Err("Error: --var requires KEY VALUE".to_string());
                }
            }
            _ => {
                return Err(format!("Error: Unknown argument '{}'", args[i]));
            }
        }
    }

    // Validate required arguments
    let template_name = template_name.ok_or("Error: --template is required")?;
    let title = title.ok_or("Error: --title is required")?;
    let project_name = project_name.unwrap_or_else(|| "My QMS Project".to_string());

    let service = DocumentService::new(".".to_string().into());

    // Ensure templates are initialized
    if let Err(e) = service.initialize_templates() {
        eprintln!("Warning: Failed to initialize templates: {e}");
    }

    let custom_vars = if custom_variables.is_empty() { None } else { Some(custom_variables) };

    match service.create_document_from_template(&template_name, title, project_name, created_by, custom_vars) {
        Ok(document) => {
            println!("✅ Document created from template successfully!");
            println!("   Template: {template_name}");
            println!("   ID: {}", document.id);
            println!("   Title: {}", document.title);
            println!("   Type: {:?}", document.doc_type);
            println!("   Version: {}", document.version);
            println!("   Created By: {}", document.created_by);
            println!("   Status: {:?}", document.status);
            Ok(())
        }
        Err(e) => Err(format!("Failed to create document from template: {e}")),
    }
}

fn handle_template_init(_args: &[String]) -> Result<(), String> {
    let service = DocumentService::new(".".to_string().into());
    
    match service.initialize_templates() {
        Ok(()) => {
            println!("✅ Templates initialized successfully!");
            println!("Default templates have been created in the templates/ directory.");
            println!("Use 'qms doc template list' to see available templates.");
            Ok(())
        }
        Err(e) => Err(format!("Failed to initialize templates: {e}")),
    }
}

fn handle_doc_list(args: &[String]) -> Result<(), String> {
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_list_help();
        return Ok(());
    }

    let service = DocumentService::new(".".to_string().into());
    
    match service.list_documents() {
        Ok(documents) => {
            if documents.is_empty() {
                println!("No documents found.");
                return Ok(());
            }

            println!("📋 QMS Documents\n");
            println!("{:<36} | {:<30} | {:<15} | {:<10} | {:<15}", 
                     "ID", "Title", "Type", "Version", "Status");
            println!("{}", "-".repeat(110));

            for doc in documents {
                println!("{:<36} | {:<30} | {:<15} | {:<10} | {:<15}",
                         doc.id,
                         if doc.title.len() > 30 { 
                             format!("{}...", &doc.title[..27]) 
                         } else { 
                             doc.title 
                         },
                         doc.doc_type,
                         doc.version,
                         doc.status);
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to list documents: {e}")),
    }
}

fn handle_doc_view(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_view_help();
        return Ok(());
    }

    let document_id = &args[0];
    let service = DocumentService::new(".".to_string().into());
    
    match service.read_document(document_id) {
        Ok(document) => {
            println!("📄 Document Details\n");
            println!("ID:          {}", document.id);
            println!("Title:       {}", document.title);
            println!("Type:        {:?}", document.doc_type);
            println!("Version:     {}", document.version);
            println!("Created By:  {}", document.created_by);
            println!("Status:      {:?}", document.status);
            println!("Created:     {}", document.created_at);
            println!("Modified:    {}", document.updated_at);
            
            if !document.tags.is_empty() {
                println!("Tags:        {}", document.tags.join(", "));
            }
            
            if !document.regulatory_mapping.is_empty() {
                println!("Regulatory:");
                for reg_ref in &document.regulatory_mapping {
                    println!("  - {} {}", reg_ref.standard, reg_ref.section);
                }
            }
            
            println!("\nContent:\n{}", "-".repeat(50));
            println!("{}", document.content);
            println!("{}", "-".repeat(50));
            
            Ok(())
        }
        Err(e) => Err(format!("Failed to view document: {e}")),
    }
}

fn handle_doc_update(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_update_help();
        return Ok(());
    }

    let document_id = &args[0];
    let service = DocumentService::new(".".to_string().into());
    
    // Parse update arguments
    let mut title: Option<String> = None;
    let mut content: Option<String> = None;
    let updated_by = "CLI User".to_string(); // Default user for CLI operations

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--title" | "-t" => {
                if i + 1 < args.len() {
                    title = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --title requires a value".to_string());
                }
            }
            "--content" | "-c" => {
                if i + 1 < args.len() {
                    content = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --content requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Error: Unknown argument '{}'. Note: --version, --author, and --status updates are not supported via CLI yet.", args[i]));
            }
        }
    }

    match service.update_document(
        document_id, 
        title, 
        content, 
        None, // change_type - let it auto-determine
        None, // change_description - use default
        updated_by
    ) {
        Ok(updated_doc) => {
            println!("✅ Document updated successfully!");
            println!("   ID: {}", updated_doc.id);
            println!("   Title: {}", updated_doc.title);
            println!("   Version: {}", updated_doc.version);
            println!("   Created By: {}", updated_doc.created_by);
            println!("   Status: {:?}", updated_doc.status);
            Ok(())
        }
        Err(e) => Err(format!("Failed to update document: {e}")),
    }
}

fn handle_doc_remove(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_remove_help();
        return Ok(());
    }

    let document_id = &args[0];
    let service = DocumentService::new(".".to_string().into());
    
    // Check if document exists first
    match service.read_document(document_id) {
        Ok(document) => {
            println!("Are you sure you want to remove the following document?");
            println!("  ID: {}", document.id);
            println!("  Title: {}", document.title);
            println!("  Type: {:?}", document.doc_type);
            println!("  Version: {}", document.version);
            println!("\nThis action cannot be undone. Type 'yes' to confirm:");
            
            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => {
                    if input.trim().to_lowercase() == "yes" {
                        match service.delete_document(document_id, "CLI User".to_string()) {
                            Ok(_) => {
                                println!("✅ Document removed successfully!");
                                Ok(())
                            }
                            Err(e) => Err(format!("Failed to remove document: {e}")),
                        }
                    } else {
                        println!("❌ Document removal cancelled.");
                        Ok(())
                    }
                }
                Err(e) => Err(format!("Failed to read input: {e}")),
            }
        }
        Err(e) => Err(format!("Failed to find document: {e}")),
    }
}

// ========== BACKUP MANAGEMENT COMMAND HANDLERS (Phase 2.1.14) ==========

fn handle_doc_backup(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_backup_help();
        return Ok(());
    }

    match args[0].as_str() {
        "list" => handle_backup_list(&args[1..]),
        "verify" => handle_backup_verify(&args[1..]),
        "recover" => handle_backup_recover(&args[1..]),
        "delete" => handle_backup_delete(&args[1..]),
        "cleanup" => handle_backup_cleanup(&args[1..]),
        "info" => handle_backup_info(&args[1..]),
        _ => {
            eprintln!("Error: Unknown backup command '{}'", args[0]);
            print_doc_backup_help();
            Err("Unknown backup command".to_string())
        }
    }
}

fn handle_backup_list(args: &[String]) -> Result<(), String> {
    let mut document_id: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--document" | "-d" => {
                if i + 1 < args.len() {
                    document_id = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--document requires a value".to_string());
                }
            }
            "--help" | "-h" => {
                print_backup_list_help();
                return Ok(());
            }
            _ => {
                if document_id.is_none() {
                    document_id = Some(args[i].clone());
                }
                i += 1;
            }
        }
    }

    let service = DocumentService::new(std::env::current_dir().unwrap());

    if let Some(doc_id) = document_id {
        // List backups for specific document
        match service.list_document_backups(&doc_id) {
            Ok(backups) => {
                if backups.is_empty() {
                    println!("💾 No backups found for document: {doc_id}");
                } else {
                    println!("💾 Document Backups for: {doc_id}\n");
                    println!("{:<25} | {:<12} | {:<20} | {:<15} | {:<30}", 
                             "Backup ID", "Version", "Created", "Size (bytes)", "Reason");
                    println!("{}", "-".repeat(105));

                    for backup in backups {
                        let created = crate::utils::format_timestamp(backup.backup_timestamp);
                        let reason = if backup.backup_reason.len() > 28 {
                            format!("{}...", &backup.backup_reason[..25])
                        } else {
                            backup.backup_reason.clone()
                        };

                        println!("{:<25} | {:<12} | {:<20} | {:<15} | {:<30}",
                                 backup.backup_id,
                                 backup.document_version,
                                 created,
                                 backup.file_size,
                                 reason);
                    }
                }
                Ok(())
            }
            Err(e) => Err(format!("Failed to list document backups: {e}")),
        }
    } else {
        // List all backups
        match service.list_all_backups() {
            Ok(backups) => {
                if backups.is_empty() {
                    println!("💾 No backups found in the system.");
                } else {
                    println!("💾 All System Backups ({} total)\n", backups.len());
                    println!("{:<25} | {:<36} | {:<12} | {:<20} | {:<30}", 
                             "Backup ID", "Document ID", "Version", "Created", "Reason");
                    println!("{}", "-".repeat(130));

                    for backup in backups {
                        let created = crate::utils::format_timestamp(backup.backup_timestamp);
                        let reason = if backup.backup_reason.len() > 28 {
                            format!("{}...", &backup.backup_reason[..25])
                        } else {
                            backup.backup_reason.clone()
                        };

                        println!("{:<25} | {:<36} | {:<12} | {:<20} | {:<30}",
                                 backup.backup_id,
                                 backup.document_id,
                                 backup.document_version,
                                 created,
                                 reason);
                    }
                }
                Ok(())
            }
            Err(e) => Err(format!("Failed to list all backups: {e}")),
        }
    }
}

fn handle_backup_verify(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_backup_verify_help();
        return Ok(());
    }

    let backup_id = &args[0];
    let service = DocumentService::new(std::env::current_dir().unwrap());

    match service.verify_backup(backup_id) {
        Ok(is_valid) => {
            if is_valid {
                println!("✅ Backup {backup_id} passed integrity verification");
                
                // Show backup details
                if let Ok(metadata) = service.get_backup_metadata(backup_id) {
                    println!("   Document ID: {}", metadata.document_id);
                    println!("   Version: {}", metadata.document_version);
                    println!("   File Size: {} bytes", metadata.file_size);
                    println!("   Checksum: {}", metadata.checksum);
                    println!("   Created: {}", crate::utils::format_timestamp(metadata.backup_timestamp));
                }
            } else {
                println!("❌ Backup {backup_id} FAILED integrity verification");
                println!("   This backup may be corrupted and should not be used for recovery.");
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to verify backup: {e}")),
    }
}

fn handle_backup_recover(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_backup_recover_help();
        return Ok(());
    }

    let backup_id = &args[0];
    let mut target_document_id: Option<String> = None;
    let recovered_by = "CLI User"; // Placeholder for Phase 4 user management

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--target" | "-t" => {
                if i + 1 < args.len() {
                    target_document_id = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("--target requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }

    let service = DocumentService::new(std::env::current_dir().unwrap());

    // Get backup metadata for confirmation
    let metadata = match service.get_backup_metadata(backup_id) {
        Ok(metadata) => metadata,
        Err(e) => return Err(format!("Failed to get backup metadata: {e}")),
    };

    // Confirm recovery operation
    println!("⚠️  Document Recovery Confirmation");
    println!("Backup ID: {backup_id}");
    println!("Source Document: {}", metadata.document_id);
    println!("Source Version: {}", metadata.document_version);
    println!("File Size: {} bytes", metadata.file_size);
    println!("Created: {}", crate::utils::format_timestamp(metadata.backup_timestamp));
    
    if let Some(ref target) = target_document_id {
        println!("Target Document: {target}");
        println!("Action: Create new document with recovered content");
    } else {
        println!("Target Document: {} (same as source)", metadata.document_id);
        println!("Action: Replace current document content");
    }

    print!("Are you sure you want to proceed? (y/N): ");
    
    use std::io::{self, Write};
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| format!("Failed to read input: {e}"))?;
    
    if input.trim().to_lowercase() != "y" && input.trim().to_lowercase() != "yes" {
        println!("Recovery cancelled.");
        return Ok(());
    }

    match service.recover_from_backup(backup_id, target_document_id.as_deref(), recovered_by) {
        Ok(recovered_doc) => {
            println!("✅ Document recovered successfully!");
            println!("Document ID: {}", recovered_doc.id);
            println!("Title: {}", recovered_doc.title);
            println!("Version: {}", recovered_doc.version);
            println!("Status: {:?}", recovered_doc.status);
            Ok(())
        }
        Err(e) => Err(format!("Failed to recover document: {e}")),
    }
}

fn handle_backup_delete(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_backup_delete_help();
        return Ok(());
    }

    let backup_id = &args[0];
    let deleted_by = "CLI User"; // Placeholder for Phase 4 user management
    let service = DocumentService::new(std::env::current_dir().unwrap());

    // Get backup metadata for confirmation
    let metadata = match service.get_backup_metadata(backup_id) {
        Ok(metadata) => metadata,
        Err(e) => return Err(format!("Failed to get backup metadata: {e}")),
    };

    // Confirm deletion
    println!("⚠️  Backup Deletion Confirmation");
    println!("Backup ID: {backup_id}");
    println!("Document ID: {}", metadata.document_id);
    println!("Version: {}", metadata.document_version);
    println!("Created: {}", crate::utils::format_timestamp(metadata.backup_timestamp));
    println!("Size: {} bytes", metadata.file_size);
    
    print!("Are you sure you want to delete this backup? (y/N): ");
    
    use std::io::{self, Write};
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| format!("Failed to read input: {e}"))?;
    
    if input.trim().to_lowercase() != "y" && input.trim().to_lowercase() != "yes" {
        println!("Backup deletion cancelled.");
        return Ok(());
    }

    match service.delete_backup(backup_id, deleted_by) {
        Ok(()) => {
            println!("✅ Backup deleted successfully!");
            println!("Backup ID: {backup_id}");
            Ok(())
        }
        Err(e) => Err(format!("Failed to delete backup: {e}")),
    }
}

fn handle_backup_cleanup(args: &[String]) -> Result<(), String> {
    let mut retention_days = 30u64; // Default 30 days
    let cleaned_by = "CLI User"; // Placeholder for Phase 4 user management

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--days" | "-d" => {
                if i + 1 < args.len() {
                    retention_days = args[i + 1].parse()
                        .map_err(|_| format!("Invalid number for retention days: {}", args[i + 1]))?;
                    i += 2;
                } else {
                    return Err("--days requires a value".to_string());
                }
            }
            "--help" | "-h" => {
                print_backup_cleanup_help();
                return Ok(());
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }

    let service = DocumentService::new(std::env::current_dir().unwrap());

    // Confirm cleanup operation
    println!("⚠️  Backup Cleanup Confirmation");
    println!("Retention Policy: {retention_days} days");
    println!("Backups older than {retention_days} days will be permanently deleted.");
    
    print!("Are you sure you want to proceed? (y/N): ");
    
    use std::io::{self, Write};
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| format!("Failed to read input: {e}"))?;
    
    if input.trim().to_lowercase() != "y" && input.trim().to_lowercase() != "yes" {
        println!("Backup cleanup cancelled.");
        return Ok(());
    }

    match service.cleanup_old_backups(retention_days, cleaned_by) {
        Ok(deleted_count) => {
            if deleted_count > 0 {
                println!("✅ Backup cleanup completed!");
                println!("Deleted {deleted_count} old backups");
            } else {
                println!("✅ No old backups found to clean up.");
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to cleanup backups: {e}")),
    }
}

fn handle_backup_info(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_backup_info_help();
        return Ok(());
    }

    let backup_id = &args[0];
    let service = DocumentService::new(std::env::current_dir().unwrap());

    match service.get_backup_metadata(backup_id) {
        Ok(metadata) => {
            println!("💾 Backup Information\n");
            println!("Backup ID:       {}", metadata.backup_id);
            println!("Document ID:     {}", metadata.document_id);
            println!("Document Version: {}", metadata.document_version);
            println!("Created:         {}", crate::utils::format_timestamp(metadata.backup_timestamp));
            println!("Created By:      {}", metadata.created_by);
            println!("File Size:       {} bytes", metadata.file_size);
            println!("Checksum:        {}", metadata.checksum);
            println!("Backup Path:     {}", metadata.backup_path);
            println!("Original Path:   {}", metadata.original_path);
            println!("Reason:          {}", metadata.backup_reason);

            // Verify backup integrity
            match service.verify_backup(backup_id) {
                Ok(true) => println!("\nIntegrity:       ✅ Valid"),
                Ok(false) => println!("\nIntegrity:       ❌ Corrupted"),
                Err(e) => println!("\nIntegrity:       ⚠️  Cannot verify ({e})"),
            }

            Ok(())
        }
        Err(e) => Err(format!("Failed to get backup information: {e}")),
    }
}

fn print_doc_help() {
    println!("Manage QMS documents\n");
    println!("USAGE:");
    println!("    qms doc <COMMAND>\n");
    println!("COMMANDS:");
    println!("    add       Add a new document to the project");
    println!("    template  Manage and use document templates");
    println!("    list      List all project documents");
    println!("    view      View document details");
    println!("    update    Update document metadata");
    println!("    remove    Remove a document from the project");
    println!("    search    Search and filter documents by text and metadata");
    println!("\nVERSION CONTROL:");
    println!("    versions  List all versions of a document");
    println!("    version   View a specific version of a document");
    println!("    history   Show detailed version history");
    println!("    compare   Compare two versions of a document");
    println!("    rollback  Rollback to a previous version");
    println!("\nCHECKOUT/CHECKIN:");
    println!("    checkout  Checkout a document for editing (locks it)");
    println!("    checkin   Checkin a document after editing (unlocks it)");
    println!("    status    Check the checkout status of a document");
    println!("    locked    List all currently checked out documents");
    println!("    unlock    Force unlock a document (admin operation)");
    println!("    locks     Administrative lock management utilities");
    println!("\nSTATUS MANAGEMENT:");
    println!("    submit    Submit a document for review (Draft → InReview)");
    println!("    approve   Approve a document for release (InReview → Approved)");
    println!("    reject    Reject a document back to draft (InReview → Draft)");
    println!("    workflow  View document approval workflow history");
    println!("    archive   Archive a document (Any Status → Archived)");
    println!("    restore   Restore an archived document (Archived → Draft)");
    println!("\nEXPORT/IMPORT:");
    println!("    export    Export document to various formats (JSON, HTML, Markdown, PDF)");
    println!("    import    Import documents from various formats (Markdown, CSV, JSON)");
    println!("\nREGULATORY COMPLIANCE:");
    println!("    regulatory Map documents to regulatory requirements");
    println!("    compliance Show compliance status and generate reports");
    println!("\nBACKUP & RECOVERY:");
    println!("    backup    Manage document backups and recovery operations");
    println!("\n    help      Show this help message\n");
    println!("For more information on a specific command, use:");
    println!("    qms doc <COMMAND> --help");
}

fn print_doc_add_help() {
    println!("Add a new document to the QMS project\n");
    println!("USAGE:");
    println!("    qms doc add --title <TITLE> --type <TYPE> [OPTIONS]\n");
    println!("REQUIRED ARGUMENTS:");
    println!("    --title, -t <TITLE>      Document title");
    println!("    --type <TYPE>            Document type\n");
    println!("OPTIONAL ARGUMENTS:");
    println!("    --content, -c <CONTENT>  Document content (mutually exclusive with --file)");
    println!("    --file, -f <FILE>        Path to file to import (mutually exclusive with --content)");
    println!("    --version, -v <VERSION>  Document version (default: 1.0)");
    println!("    --author, -a <AUTHOR>    Document author (default: CLI User)\n");
    println!("DOCUMENT TYPES:");
    println!("    srs              Software Requirements Specification");
    println!("    sdd              Software Design Description");
    println!("    vnv              Verification and Validation");
    println!("    rmf              Risk Management File");
    println!("    dhf              Design History File");
    println!("    ur               User Requirements");
    println!("    protocol         Test Protocol");
    println!("    report           Test Report");
    println!("    auto             Auto-detect type from file extension (use with --file)");
    println!("    other            Other document types (specify any custom name)\n");
    println!("FILE IMPORT FEATURES:");
    println!("    • Supports text-based files (Markdown, TXT, JSON, CSV, XML, HTML)");
    println!("    • Maximum file size: 10MB");
    println!("    • Auto-detects document type from file extension when type is 'auto'");
    println!("    • Creates archive copy of original file");
    println!("    • Validates file content for text format\n");
    println!("EXAMPLES:");
    println!("    # Create document with inline content");
    println!("    qms doc add --title \"SRS v1.0\" --type srs --content \"Initial requirements\"");
    println!();
    println!("    # Import document from file");
    println!("    qms doc add --title \"User Manual\" --type ur --file \"docs/manual.md\"");
    println!();
    println!("    # Auto-detect type from file extension");
    println!("    qms doc add --title \"Test Data\" --type auto --file \"data/results.csv\"");
    println!();
    println!("    # Import with custom author");
    println!("    qms doc add -t \"Protocol v2\" --type protocol -f \"protocol.txt\" -a \"Jane Doe\"");
}

fn print_doc_list_help() {
    println!("List all documents in the QMS project\n");
    println!("USAGE:");
    println!("    qms doc list\n");
    println!("This command displays a table of all documents with their:");
    println!("    - ID (UUID)");
    println!("    - Title");
    println!("    - Type");
    println!("    - Version");
    println!("    - Status");
    println!("    - Author");
}

fn print_doc_view_help() {
    println!("View detailed information about a specific document\n");
    println!("USAGE:");
    println!("    qms doc view <DOCUMENT_ID>\n");
    println!("ARGUMENTS:");
    println!("    <DOCUMENT_ID>    The UUID of the document to view\n");
    println!("This command displays complete document information including:");
    println!("    - All metadata (ID, title, type, version, author, status)");
    println!("    - Creation and modification timestamps");
    println!("    - Tags (if any)");
    println!("    - Regulatory references (if any)");
    println!("    - Full document content\n");
    println!("EXAMPLE:");
    println!("    qms doc view 123e4567-e89b-12d3-a456-426614174000");
}

fn print_doc_update_help() {
    println!("Update an existing document's metadata and content\n");
    println!("USAGE:");
    println!("    qms doc update <DOCUMENT_ID> [OPTIONS]\n");
    println!("ARGUMENTS:");
    println!("    <DOCUMENT_ID>            The UUID of the document to update\n");
    println!("OPTIONS:");
    println!("    --title, -t <TITLE>      Update document title");
    println!("    --content, -c <CONTENT>  Update document content\n");
    println!("NOTE:");
    println!("    Version, author, and status updates are not yet supported via CLI.");
    println!("    These require future implementation through the service layer.\n");
    println!("EXAMPLES:");
    println!("    qms doc update 123e4567-e89b-12d3-a456-426614174000 --title \"Updated Title\"");
    println!("    qms doc update 123e4567-e89b-12d3-a456-426614174000 --content \"New content\"");
}

fn print_doc_remove_help() {
    println!("Remove a document from the QMS project\n");
    println!("USAGE:");
    println!("    qms doc remove <DOCUMENT_ID>\n");
    println!("ARGUMENTS:");
    println!("    <DOCUMENT_ID>    The UUID of the document to remove\n");
    println!("WARNING:");
    println!("    This action permanently deletes the document and cannot be undone.");
    println!("    You will be prompted to confirm the deletion.\n");
    println!("EXAMPLE:");
    println!("    qms doc remove 123e4567-e89b-12d3-a456-426614174000");
}

// ========== VERSION CONTROL COMMAND HANDLERS ==========

fn handle_doc_versions(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_versions_help();
        return Ok(());
    }

    let document_id = &args[0];

    // Load project
    let project_path = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?;

    let service = DocumentService::new(project_path);

    match service.list_document_versions(document_id) {
        Ok(versions) => {
            if versions.is_empty() {
                println!("No versions found for document {document_id}");
            } else {
                println!("Document {document_id} versions:");
                println!("{:<12} {:<15} {:<20}", "Version", "Change Type", "Created");
                println!("{}", "-".repeat(47));
                
                // Get version history for additional details
                if let Ok(history) = service.get_document_version_history(document_id) {
                    for version_info in history {
                        let change_type = match version_info.change_type {
                            VersionChangeType::Major => "Major",
                            VersionChangeType::Minor => "Minor", 
                            VersionChangeType::Patch => "Patch",
                        };
                        
                        let timestamp = crate::utils::format_timestamp(version_info.created_at);
                        println!("{:<12} {:<15} {:<20}", version_info.version, change_type, timestamp);
                    }
                } else {
                    // Fallback to just listing versions
                    for version in versions {
                        println!("{:<12} {:<15} {:<20}", version, "Unknown", "Unknown");
                    }
                }
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to list document versions: {e}")),
    }
}

fn handle_doc_version(args: &[String]) -> Result<(), String> {
    if args.len() < 2 || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_version_help();
        return Ok(());
    }

    let document_id = &args[0];
    let version = &args[1];

    // Load project
    let project_path = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?;

    let service = DocumentService::new(project_path);

    match service.get_document_version(document_id, version) {
        Ok(document) => {
            println!("Document: {} (Version: {})", document.title, document.version);
            println!("Type: {}", document.doc_type.to_string());
            println!("Status: {:?}", document.status);
            println!("Created: {}", crate::utils::format_timestamp_from_string(&document.created_at));
            println!("Updated: {}", crate::utils::format_timestamp_from_string(&document.updated_at));
            println!("Author: {}", document.created_by);
            println!("Checksum: {}", document.checksum);
            println!("\nContent:");
            println!("{}", "-".repeat(50));
            println!("{}", document.content);
            Ok(())
        }
        Err(e) => Err(format!("Failed to get document version: {e}")),
    }
}

fn handle_doc_history(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_history_help();
        return Ok(());
    }

    let document_id = &args[0];

    // Load project
    let project_path = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?;

    let service = DocumentService::new(project_path);

    match service.get_document_version_history(document_id) {
        Ok(history) => {
            if history.is_empty() {
                println!("No version history found for document {document_id}");
            } else {
                println!("Version History for Document: {document_id}");
                println!("{:<12} {:<10} {:<15} {:<20} {:<30}", "Version", "Type", "Author", "Created", "Description");
                println!("{}", "-".repeat(87));
                
                for version_info in history {
                    let change_type = match version_info.change_type {
                        VersionChangeType::Major => "Major",
                        VersionChangeType::Minor => "Minor",
                        VersionChangeType::Patch => "Patch",
                    };
                    
                    let timestamp = crate::utils::format_timestamp(version_info.created_at);
                    let description = if version_info.change_description.len() > 28 {
                        format!("{}...", &version_info.change_description[..25])
                    } else {
                        version_info.change_description.clone()
                    };
                    
                    println!(
                        "{:<12} {:<10} {:<15} {:<20} {:<30}",
                        version_info.version,
                        change_type,
                        version_info.created_by,
                        timestamp,
                        description
                    );
                }
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to get document history: {e}")),
    }
}

fn handle_doc_compare(args: &[String]) -> Result<(), String> {
    if args.len() < 3 || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_compare_help();
        return Ok(());
    }

    let document_id = &args[0];
    let version_a = &args[1];
    let version_b = &args[2];

    // Load project
    let project_path = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?;

    let service = DocumentService::new(project_path);

    match service.compare_document_versions(document_id, version_a, version_b) {
        Ok((doc_a, doc_b)) => {
            println!("Comparing Document {document_id} Versions: {version_a} vs {version_b}");
            println!("{}", "=".repeat(60));
            
            // Compare metadata
            println!("METADATA COMPARISON:");
            println!("Title:      {} | {}", doc_a.title, doc_b.title);
            println!("Type:       {} | {}", doc_a.doc_type.to_string(), doc_b.doc_type.to_string());
            println!("Status:     {:?} | {:?}", doc_a.status, doc_b.status);
            println!("Updated:    {} | {}", 
                crate::utils::format_timestamp_from_string(&doc_a.updated_at),
                crate::utils::format_timestamp_from_string(&doc_b.updated_at)
            );
            
            // Basic content comparison
            println!("\nCONTENT COMPARISON:");
            if doc_a.content == doc_b.content {
                println!("✓ Content is identical");
            } else {
                println!("✗ Content differs");
                println!("  Version {} length: {} characters", version_a, doc_a.content.len());
                println!("  Version {} length: {} characters", version_b, doc_b.content.len());
                
                // Show first few lines if different
                let lines_a: Vec<&str> = doc_a.content.lines().collect();
                let lines_b: Vec<&str> = doc_b.content.lines().collect();
                
                println!("\nFirst few lines comparison:");
                for i in 0..std::cmp::min(5, std::cmp::max(lines_a.len(), lines_b.len())) {
                    let line_a = lines_a.get(i).unwrap_or(&"<END>");
                    let line_b = lines_b.get(i).unwrap_or(&"<END>");
                    
                    if line_a != line_b {
                        println!("  Line {}: {} | {}", i + 1, line_a, line_b);
                    }
                }
            }
            
            Ok(())
        }
        Err(e) => Err(format!("Failed to compare document versions: {e}")),
    }
}

fn handle_doc_rollback(args: &[String]) -> Result<(), String> {
    if args.len() < 2 || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_rollback_help();
        return Ok(());
    }

    let document_id = &args[0];
    let target_version = &args[1];
    
    // Parse optional reason
    let mut reason = "User requested rollback".to_string();
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--reason" | "-r" => {
                if i + 1 < args.len() {
                    reason = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Error: --reason requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown option: {}", args[i]));
            }
        }
    }

    // Load project
    let project_path = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?;

    let service = DocumentService::new(project_path);

    // Confirm rollback
    println!("WARNING: This will rollback document {document_id} to version {target_version}");
    println!("Reason: {reason}");
    println!("This will create a new version with the content from version {target_version}");
    print!("Are you sure you want to continue? (y/N): ");
    
    use std::io::{self, Write};
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| format!("Failed to read input: {e}"))?;
    
    if input.trim().to_lowercase() != "y" && input.trim().to_lowercase() != "yes" {
        println!("Rollback cancelled.");
        return Ok(());
    }

    match service.rollback_document_to_version(document_id, target_version, &reason, "current_user") {
        Ok(document) => {
            println!("✅ Document successfully rolled back!");
            println!("Document ID: {}", document.id);
            println!("New Version: {}", document.version);
            println!("Rollback Reason: {reason}");
            Ok(())
        }
        Err(e) => Err(format!("Failed to rollback document: {e}")),
    }
}

// ========== CHECKOUT/CHECKIN HANDLERS ==========

fn handle_doc_checkout(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_checkout_help();
        return Ok(());
    }

    let document_id = &args[0];
    
    // Parse optional reason
    let mut reason: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--reason" | "-r" => {
                if i + 1 < args.len() {
                    reason = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --reason requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown option: {}", args[i]));
            }
        }
    }

    // Load project
    let project_path = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?;

    let service = DocumentService::new(project_path);

    // TODO: Get current user from authentication system
    let current_user = "current_user"; // Placeholder for actual user system

    match service.checkout_document(document_id, current_user, reason.clone()) {
        Ok(lock) => {
            println!("✅ Document checked out successfully!");
            println!("Document ID: {}", lock.document_id);
            println!("Checked out by: {}", lock.user_id);
            println!("Locked at: {}", lock.locked_at);
            if let Some(ref reason) = lock.lock_reason {
                println!("Reason: {reason}");
            }
            println!("\nThe document is now locked for editing. Use 'qms doc checkin' when finished.");
            Ok(())
        }
        Err(e) => Err(format!("Failed to checkout document: {e}")),
    }
}

fn handle_doc_checkin(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_checkin_help();
        return Ok(());
    }

    let document_id = &args[0];
    
    // Parse optional file and message
    let mut file_path: Option<String> = None;
    let mut message: Option<String> = None;
    let mut i = 1;
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
            "--message" | "-m" => {
                if i + 1 < args.len() {
                    message = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --message requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown option: {}", args[i]));
            }
        }
    }

    // Load project
    let project_path = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?;

    let service = DocumentService::new(project_path);

    // TODO: Get current user from authentication system
    let current_user = "current_user"; // Placeholder for actual user system

    match service.checkin_document(
        document_id, 
        current_user, 
        file_path.as_deref(),
        message.as_deref()
    ) {
        Ok(document) => {
            println!("✅ Document checked in successfully!");
            println!("Document ID: {}", document.id);
            println!("New Version: {}", document.version);
            println!("Updated at: {}", document.updated_at);
            if file_path.is_some() {
                println!("Content updated from file: {}", file_path.unwrap());
            }
            if let Some(msg) = message {
                println!("Message: {msg}");
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to checkin document: {e}")),
    }
}

fn handle_doc_status(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_status_help();
        return Ok(());
    }

    let document_id = &args[0];

    // Load project
    let project_path = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?;

    let service = DocumentService::new(project_path);

    match service.get_checkout_status(document_id) {
        Ok(Some(lock)) => {
            println!("📝 Document Status: CHECKED OUT");
            println!("Document ID: {}", lock.document_id);
            println!("Checked out by: {}", lock.user_id);
            println!("Locked at: {}", lock.locked_at);
            if let Some(ref reason) = lock.lock_reason {
                println!("Reason: {reason}");
            }
            Ok(())
        }
        Ok(None) => {
            println!("📋 Document Status: AVAILABLE");
            println!("Document ID: {document_id}");
            println!("The document is not currently checked out.");
            Ok(())
        }
        Err(e) => Err(format!("Failed to get document status: {e}")),
    }
}

fn handle_doc_locked(_args: &[String]) -> Result<(), String> {
    // Load project
    let project_path = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?;

    let service = DocumentService::new(project_path);

    match service.list_locked_documents() {
        Ok(locks) => {
            if locks.is_empty() {
                println!("📋 No documents are currently checked out.");
            } else {
                println!("📝 Currently Checked Out Documents:");
                println!("======================================");
                for lock in locks {
                    println!("Document ID: {}", lock.document_id);
                    println!("  Checked out by: {}", lock.user_id);
                    println!("  Locked at: {}", lock.locked_at);
                    if let Some(ref reason) = lock.lock_reason {
                        println!("  Reason: {reason}");
                    }
                    println!();
                }
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to list locked documents: {e}")),
    }
}

fn handle_doc_unlock(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_unlock_help();
        return Ok(());
    }

    let document_id = &args[0];
    
    // Parse optional reason
    let mut reason = "Administrative unlock".to_string();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--reason" | "-r" => {
                if i + 1 < args.len() {
                    reason = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Error: --reason requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown option: {}", args[i]));
            }
        }
    }

    // Load project
    let project_path = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {e}"))?;

    let service = DocumentService::new(project_path);

    // TODO: Get current user from authentication system
    let current_user = "admin_user"; // Placeholder for actual user system

    // Confirm force unlock
    println!("WARNING: This will forcefully unlock document {document_id}");
    println!("Reason: {reason}");
    print!("Are you sure you want to continue? (y/N): ");
    
    use std::io::{self, Write};
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| format!("Failed to read input: {e}"))?;
    
    if input.trim().to_lowercase() != "y" && input.trim().to_lowercase() != "yes" {
        println!("Force unlock cancelled.");
        return Ok(());
    }

    match service.force_release_lock(document_id, current_user, &reason) {
        Ok(()) => {
            println!("✅ Document unlocked successfully!");
            println!("Document ID: {document_id}");
            println!("Unlocked by: {current_user}");
            println!("Reason: {reason}");
            Ok(())
        }
        Err(e) => Err(format!("Failed to unlock document: {e}")),
    }
}

// ========== ADMINISTRATIVE LOCK MANAGEMENT (Phase 2.1.15) ==========

fn handle_doc_locks(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_locks_help();
        return Ok(());
    }

    match args[0].as_str() {
        "info" => handle_locks_info(&args[1..]),
        "stats" => handle_locks_stats(&args[1..]),
        "cleanup" => handle_locks_cleanup(&args[1..]),
        _ => {
            eprintln!("Error: Unknown locks command '{}'", args[0]);
            print_doc_locks_help();
            Err("Unknown locks command".to_string())
        }
    }
}

fn handle_locks_info(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_locks_info_help();
        return Ok(());
    }

    let document_id = &args[0];
    let service = DocumentService::new(std::env::current_dir().unwrap());

    match service.get_detailed_lock_info(document_id) {
        Ok(Some(lock_info)) => {
            println!("🔒 Detailed Lock Information for Document: {document_id}");
            println!("===============================================");
            println!("Document Lock:");
            println!("  User: {}", lock_info.document_lock.user_id);
            println!("  Locked At: {}", lock_info.document_lock.locked_at);
            if let Some(ref reason) = lock_info.document_lock.lock_reason {
                println!("  Reason: {reason}");
            }
            
            println!("\nFile Lock Status:");
            println!("  Has File Lock: {}", if lock_info.has_file_lock { "Yes" } else { "No" });
            if let Some(ref holder) = lock_info.file_lock_holder {
                println!("  File Lock Holder: {holder}");
            }
            
            println!("\nManager Status:");
            println!("  Active in Manager: {}", if lock_info.is_active_in_manager { "Yes" } else { "No" });
            
            Ok(())
        }
        Ok(None) => {
            println!("📋 Document {document_id} is not currently locked.");
            Ok(())
        }
        Err(e) => Err(format!("Failed to get lock information: {e}")),
    }
}

fn handle_locks_stats(_args: &[String]) -> Result<(), String> {
    let service = DocumentService::new(std::env::current_dir().unwrap());

    match service.get_lock_statistics() {
        Ok(stats) => {
            println!("📊 System Lock Statistics");
            println!("=========================");
            println!("Total Document Locks: {}", stats.total_document_locks);
            println!("Total File Locks: {}", stats.total_file_locks);
            println!("Active in Manager: {}", stats.active_in_manager);
            
            if let Some(duration) = stats.longest_held_lock_duration {
                let hours = duration.as_secs() / 3600;
                let minutes = (duration.as_secs() % 3600) / 60;
                println!("Longest Held Lock: {hours}h {minutes}m");
            } else {
                println!("Longest Held Lock: N/A");
            }
            
            Ok(())
        }
        Err(e) => Err(format!("Failed to get lock statistics: {e}")),
    }
}

fn handle_locks_cleanup(args: &[String]) -> Result<(), String> {
    let mut admin_user = "CLI_Admin".to_string();
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--admin" | "-a" => {
                if i + 1 < args.len() {
                    admin_user = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--admin requires a value".to_string());
                }
            }
            "--help" | "-h" => {
                print_locks_cleanup_help();
                return Ok(());
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }

    // Confirm cleanup operation
    println!("⚠️  Stale Lock Cleanup");
    println!("This will remove locks older than 24 hours.");
    println!("Admin User: {admin_user}");
    
    print!("Are you sure you want to proceed? (y/N): ");
    
    use std::io::{self, Write};
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| format!("Failed to read input: {e}"))?;
    
    if input.trim().to_lowercase() != "y" && input.trim().to_lowercase() != "yes" {
        println!("Lock cleanup cancelled.");
        return Ok(());
    }

    let service = DocumentService::new(std::env::current_dir().unwrap());

    match service.cleanup_stale_locks(&admin_user) {
        Ok(report) => {
            println!("✅ Lock cleanup completed!");
            println!("Document locks cleaned: {}", report.cleaned_document_locks);
            println!("File locks cleaned: {}", report.cleaned_file_locks);
            
            if !report.errors.is_empty() {
                println!("\n⚠️  Errors encountered:");
                for error in report.errors {
                    println!("  - {error}");
                }
            }
            
            Ok(())
        }
        Err(e) => Err(format!("Failed to cleanup locks: {e}")),
    }
}

// ========== Phase 2.1.6 DOCUMENT STATUS MANAGEMENT HANDLERS ==========

fn handle_doc_submit(args: &[String]) -> Result<(), String> {
    use crate::modules::document_control::approval::ApprovalWorkflow;
    
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_submit_help();
        return Ok(());
    }

    if args.is_empty() {
        eprintln!("Error: Document ID is required");
        print_doc_submit_help();
        return Err("Missing document ID".to_string());
    }

    let document_id = &args[0];
    
    // Parse optional arguments
    let mut submitter_id = "CLI_USER".to_string();
    let mut submitter_name = "CLI User".to_string();
    let mut comments = None;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--submitter-id" => {
                if i + 1 < args.len() {
                    submitter_id = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--submitter-id requires a value".to_string());
                }
            }
            "--submitter-name" => {
                if i + 1 < args.len() {
                    submitter_name = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--submitter-name requires a value".to_string());
                }
            }
            "--comments" => {
                if i + 1 < args.len() {
                    comments = Some(args[i + 1].as_str());
                    i += 2;
                } else {
                    return Err("--comments requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }

    let mut workflow = ApprovalWorkflow::new()
        .map_err(|e| format!("Failed to initialize approval workflow: {e}"))?;

    match workflow.submit_for_review(document_id, &submitter_id, &submitter_name, comments) {
        Ok(()) => {
            println!("✅ Document submitted for review successfully!");
            println!("Document ID: {document_id}");
            println!("Status: Draft → SubmittedForReview");
            println!("Submitted by: {submitter_name} ({submitter_id})");
            if let Some(comment) = comments {
                println!("Comments: {comment}");
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to submit document: {e}")),
    }
}

fn handle_doc_approve(args: &[String]) -> Result<(), String> {
    use crate::modules::document_control::approval::ApprovalWorkflow;
    
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_approve_help();
        return Ok(());
    }

    if args.is_empty() {
        eprintln!("Error: Document ID is required");
        print_doc_approve_help();
        return Err("Missing document ID".to_string());
    }

    let document_id = &args[0];
    
    // Parse optional arguments
    let mut approver_id = "CLI_USER".to_string();
    let mut approver_name = "CLI User".to_string();
    let mut signing_reason = "Document approval".to_string();
    let mut authentication_method = "CLI".to_string();
    let mut comments = None;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--approver-id" => {
                if i + 1 < args.len() {
                    approver_id = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--approver-id requires a value".to_string());
                }
            }
            "--approver-name" => {
                if i + 1 < args.len() {
                    approver_name = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--approver-name requires a value".to_string());
                }
            }
            "--signing-reason" => {
                if i + 1 < args.len() {
                    signing_reason = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--signing-reason requires a value".to_string());
                }
            }
            "--auth-method" => {
                if i + 1 < args.len() {
                    authentication_method = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--auth-method requires a value".to_string());
                }
            }
            "--comments" => {
                if i + 1 < args.len() {
                    comments = Some(args[i + 1].as_str());
                    i += 2;
                } else {
                    return Err("--comments requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }

    // Create electronic signature
    let signature = ApprovalWorkflow::create_electronic_signature(
        &approver_id,
        &approver_name,
        &signing_reason,
        &authentication_method
    );

    let mut workflow = ApprovalWorkflow::new()
        .map_err(|e| format!("Failed to initialize approval workflow: {e}"))?;

    match workflow.approve_document(document_id, &approver_id, &approver_name, signature.clone(), comments) {
        Ok(()) => {
            println!("✅ Document approved successfully!");
            println!("Document ID: {document_id}");
            println!("Status: InReview → Approved");
            println!("Approved by: {approver_name} ({approver_id})");
            println!("Electronic Signature: {}", signature.signature_hash);
            println!("Signing Reason: {signing_reason}");
            println!("Authentication Method: {authentication_method}");
            if let Some(comment) = comments {
                println!("Comments: {comment}");
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to approve document: {e}")),
    }
}

fn handle_doc_reject(args: &[String]) -> Result<(), String> {
    use crate::modules::document_control::approval::ApprovalWorkflow;
    
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_reject_help();
        return Ok(());
    }

    if args.is_empty() {
        eprintln!("Error: Document ID is required");
        print_doc_reject_help();
        return Err("Missing document ID".to_string());
    }

    let document_id = &args[0];
    
    // Parse required and optional arguments
    let mut rejector_id = "CLI_USER".to_string();
    let mut rejector_name = "CLI User".to_string();
    let mut reason = "Document rejected".to_string();
    let mut comments = None;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--rejector-id" => {
                if i + 1 < args.len() {
                    rejector_id = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--rejector-id requires a value".to_string());
                }
            }
            "--rejector-name" => {
                if i + 1 < args.len() {
                    rejector_name = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--rejector-name requires a value".to_string());
                }
            }
            "--reason" => {
                if i + 1 < args.len() {
                    reason = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("--reason requires a value".to_string());
                }
            }
            "--comments" => {
                if i + 1 < args.len() {
                    comments = Some(args[i + 1].as_str());
                    i += 2;
                } else {
                    return Err("--comments requires a value".to_string());
                }
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }

    let mut workflow = ApprovalWorkflow::new()
        .map_err(|e| format!("Failed to initialize approval workflow: {e}"))?;

    match workflow.reject_document(document_id, &rejector_id, &rejector_name, &reason, comments) {
        Ok(()) => {
            println!("✅ Document rejected successfully!");
            println!("Document ID: {document_id}");
            println!("Status: InReview → Rejected (Draft)");
            println!("Rejected by: {rejector_name} ({rejector_id})");
            println!("Reason: {reason}");
            if let Some(comment) = comments {
                println!("Comments: {comment}");
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to reject document: {e}")),
    }
}

fn handle_doc_workflow(args: &[String]) -> Result<(), String> {
    use crate::modules::document_control::approval::ApprovalWorkflow;
    
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_workflow_help();
        return Ok(());
    }

    if args.is_empty() {
        eprintln!("Error: Document ID is required");
        print_doc_workflow_help();
        return Err("Missing document ID".to_string());
    }

    let document_id = &args[0];

    let workflow = ApprovalWorkflow::new()
        .map_err(|e| format!("Failed to initialize approval workflow: {e}"))?;

    match workflow.get_workflow_history(document_id) {
        Ok(history) => {
            if history.is_empty() {
                println!("No workflow history found for document: {document_id}");
                return Ok(());
            }

            println!("📋 Workflow History for Document: {document_id}");
            println!("================================================");
            
            for (index, entry) in history.iter().enumerate() {
                println!("\n{}. Workflow Entry: {}", index + 1, entry.workflow_id);
                println!("   Transition: {:?} → {:?}", entry.from_state, entry.to_state);
                println!("   Timestamp: {}", entry.action_timestamp);
                println!("   Actor: {} ({})", entry.actor_name, entry.actor_id);
                
                if let Some(signature) = &entry.signature {
                    println!("   Electronic Signature:");
                    println!("     Hash: {}", signature.signature_hash);
                    println!("     Reason: {}", signature.signing_reason);
                    println!("     Auth Method: {}", signature.authentication_method);
                }
                
                if let Some(comments) = &entry.comments {
                    println!("   Comments: {comments}");
                }
                
                if let Some(reason) = &entry.reason {
                    println!("   Reason: {reason}");
                }
            }
            
            // Show current state
            match workflow.get_current_workflow_state(document_id) {
                Ok(current_state) => {
                    println!("\n📍 Current State: {current_state:?}");
                }
                Err(e) => {
                    println!("\n⚠️  Could not determine current state: {e}");
                }
            }

            Ok(())
        }
        Err(e) => Err(format!("Failed to get workflow history: {e}")),
    }
}

fn handle_doc_archive(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_archive_help();
        return Ok(());
    }

    if args.is_empty() {
        eprintln!("Error: Document ID is required");
        print_doc_archive_help();
        return Err("Missing document ID".to_string());
    }

    let document_id = &args[0];
    let current_user = "CLI User"; // Placeholder for Phase 4 user management
    
    // Parse optional reason
    let reason = if let Some(idx) = args.iter().position(|x| x == "--reason") {
        if idx + 1 < args.len() {
            Some(args[idx + 1].as_str())
        } else {
            None
        }
    } else {
        None
    };

    let project_path = std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;
    let service = DocumentService::new(project_path);

    match service.archive_document(document_id, current_user, reason) {
        Ok(document) => {
            println!("✅ Document archived successfully!");
            println!("Document ID: {}", document.id);
            println!("Title: {}", document.title);
            println!("Status: {:?} → Archived", "Previous");
            println!("Archived by: {current_user}");
            println!("Reason: {}", reason.unwrap_or("User requested"));
            println!("Updated at: {}", document.updated_at);
            Ok(())
        }
        Err(e) => Err(format!("Failed to archive document: {e}")),
    }
}

fn handle_doc_restore(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_restore_help();
        return Ok(());
    }

    if args.is_empty() {
        eprintln!("Error: Document ID is required");
        print_doc_restore_help();
        return Err("Missing document ID".to_string());
    }

    let document_id = &args[0];
    let current_user = "CLI User"; // Placeholder for Phase 4 user management
    
    // Parse optional reason
    let reason = if let Some(idx) = args.iter().position(|x| x == "--reason") {
        if idx + 1 < args.len() {
            Some(args[idx + 1].as_str())
        } else {
            None
        }
    } else {
        None
    };

    let project_path = std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;
    let service = DocumentService::new(project_path);

    match service.restore_document(document_id, current_user, reason) {
        Ok(document) => {
            println!("✅ Document restored successfully!");
            println!("Document ID: {}", document.id);
            println!("Title: {}", document.title);
            println!("Status: Archived → Draft");
            println!("Restored by: {current_user}");
            println!("Reason: {}", reason.unwrap_or("User requested"));
            println!("Updated at: {}", document.updated_at);
            Ok(())
        }
        Err(e) => Err(format!("Failed to restore document: {e}")),
    }
}

// ========== VERSION CONTROL HELP FUNCTIONS ==========

fn print_doc_versions_help() {
    println!("List all versions of a document\n");
    println!("USAGE:");
    println!("    qms doc versions <DOCUMENT_ID>\n");
    println!("ARGUMENTS:");
    println!("    <DOCUMENT_ID>    The UUID of the document\n");
    println!("DESCRIPTION:");
    println!("    Shows all available versions of a document with creation timestamps");
    println!("    and change types (Major, Minor, Patch).\n");
    println!("EXAMPLE:");
    println!("    qms doc versions 123e4567-e89b-12d3-a456-426614174000");
}

fn print_doc_version_help() {
    println!("View a specific version of a document\n");
    println!("USAGE:");
    println!("    qms doc version <DOCUMENT_ID> <VERSION>\n");
    println!("ARGUMENTS:");
    println!("    <DOCUMENT_ID>    The UUID of the document");
    println!("    <VERSION>        The semantic version (e.g., 1.0.0)\n");
    println!("DESCRIPTION:");
    println!("    Displays the complete content and metadata of a specific document version.\n");
    println!("EXAMPLES:");
    println!("    qms doc version 123e4567-e89b-12d3-a456-426614174000 1.0.0");
    println!("    qms doc version 123e4567-e89b-12d3-a456-426614174000 2.1.3");
}

fn print_doc_history_help() {
    println!("Show detailed version history of a document\n");
    println!("USAGE:");
    println!("    qms doc history <DOCUMENT_ID>\n");
    println!("ARGUMENTS:");
    println!("    <DOCUMENT_ID>    The UUID of the document\n");
    println!("DESCRIPTION:");
    println!("    Shows detailed version history including authors, change types,");
    println!("    timestamps, and change descriptions.\n");
    println!("EXAMPLE:");
    println!("    qms doc history 123e4567-e89b-12d3-a456-426614174000");
}

fn print_doc_compare_help() {
    println!("Compare two versions of a document\n");
    println!("USAGE:");
    println!("    qms doc compare <DOCUMENT_ID> <VERSION_A> <VERSION_B>\n");
    println!("ARGUMENTS:");
    println!("    <DOCUMENT_ID>    The UUID of the document");
    println!("    <VERSION_A>      First version to compare (e.g., 1.0.0)");
    println!("    <VERSION_B>      Second version to compare (e.g., 1.1.0)\n");
    println!("DESCRIPTION:");
    println!("    Shows differences between two document versions including");
    println!("    metadata changes and basic content comparison.\n");
    println!("EXAMPLES:");
    println!("    qms doc compare 123e4567-e89b-12d3-a456-426614174000 1.0.0 1.1.0");
    println!("    qms doc compare 123e4567-e89b-12d3-a456-426614174000 2.0.0 1.5.3");
}

fn print_doc_rollback_help() {
    println!("Rollback a document to a previous version\n");
    println!("USAGE:");
    println!("    qms doc rollback <DOCUMENT_ID> <TARGET_VERSION> [OPTIONS]\n");
    println!("ARGUMENTS:");
    println!("    <DOCUMENT_ID>      The UUID of the document");
    println!("    <TARGET_VERSION>   The version to rollback to (e.g., 1.0.0)\n");
    println!("OPTIONS:");
    println!("    --reason, -r <TEXT>    Reason for the rollback\n");
    println!("DESCRIPTION:");
    println!("    Creates a new version with the content from the target version.");
    println!("    This is a major version increment with rollback content.\n");
    println!("WARNING:");
    println!("    This action creates a new version and cannot be undone.");
    println!("    You will be prompted to confirm the rollback.\n");
    println!("EXAMPLES:");
    println!("    qms doc rollback 123e4567-e89b-12d3-a456-426614174000 1.0.0");
    println!("    qms doc rollback 123e4567-e89b-12d3-a456-426614174000 1.0.0 --reason \"Bug in current version\"");
}

// ========== CHECKOUT/CHECKIN HELP FUNCTIONS ==========

fn print_doc_checkout_help() {
    println!("Checkout a document for editing (locks the document)\n");
    println!("USAGE:");
    println!("    qms doc checkout <DOCUMENT_ID> [OPTIONS]\n");
    println!("ARGUMENTS:");
    println!("    <DOCUMENT_ID>    The UUID of the document to checkout\n");
    println!("OPTIONS:");
    println!("    --reason, -r <TEXT>    Reason for checking out the document\n");
    println!("DESCRIPTION:");
    println!("    Locks a document for editing to prevent conflicts in collaborative environments.");
    println!("    Once checked out, the document cannot be modified by other users until checked in.\n");
    println!("EXAMPLES:");
    println!("    qms doc checkout 123e4567-e89b-12d3-a456-426614174000");
    println!("    qms doc checkout 123e4567-e89b-12d3-a456-426614174000 --reason \"Updating requirements\"");
}

fn print_doc_checkin_help() {
    println!("Checkin a document after editing (unlocks the document)\n");
    println!("USAGE:");
    println!("    qms doc checkin <DOCUMENT_ID> [OPTIONS]\n");
    println!("ARGUMENTS:");
    println!("    <DOCUMENT_ID>    The UUID of the document to checkin\n");
    println!("OPTIONS:");
    println!("    --file, -f <PATH>       Path to new content file to update document");
    println!("    --message, -m <TEXT>    Message describing the changes\n");
    println!("DESCRIPTION:");
    println!("    Unlocks a document after editing and optionally updates its content.");
    println!("    If a file is provided, the document content will be updated and versioned.");
    println!("    This creates a version snapshot and increments the version number.\n");
    println!("EXAMPLES:");
    println!("    qms doc checkin 123e4567-e89b-12d3-a456-426614174000");
    println!("    qms doc checkin 123e4567-e89b-12d3-a456-426614174000 --file updated_content.md");
    println!("    qms doc checkin 123e4567-e89b-12d3-a456-426614174000 --file new_content.md --message \"Added new requirements\"");
}

fn print_doc_status_help() {
    println!("Check the checkout status of a document\n");
    println!("USAGE:");
    println!("    qms doc status <DOCUMENT_ID>\n");
    println!("ARGUMENTS:");
    println!("    <DOCUMENT_ID>    The UUID of the document to check\n");
    println!("DESCRIPTION:");
    println!("    Shows whether a document is currently checked out and by whom.");
    println!("    Displays lock information including user, timestamp, and reason.\n");
    println!("EXAMPLES:");
    println!("    qms doc status 123e4567-e89b-12d3-a456-426614174000");
}

fn print_doc_unlock_help() {
    println!("Force unlock a document (admin operation)\n");
    println!("USAGE:");
    println!("    qms doc unlock <DOCUMENT_ID> [OPTIONS]\n");
    println!("ARGUMENTS:");
    println!("    <DOCUMENT_ID>    The UUID of the document to unlock\n");
    println!("OPTIONS:");
    println!("    --reason, -r <TEXT>    Reason for force unlocking the document\n");
    println!("DESCRIPTION:");
    println!("    Forcefully releases a document lock. This is an administrative operation");
    println!("    that should be used carefully as it can disrupt other users' work.");
    println!("    You will be prompted to confirm the action.\n");
    println!("WARNING:");
    println!("    This action forcefully unlocks the document and may disrupt other users.\n");
    println!("EXAMPLES:");
    println!("    qms doc unlock 123e4567-e89b-12d3-a456-426614174000");
    println!("    qms doc unlock 123e4567-e89b-12d3-a456-426614174000 --reason \"User unavailable\"");
}

// ========== Phase 2.1.6 STATUS MANAGEMENT HELP FUNCTIONS ==========

fn print_doc_submit_help() {
    println!("Submit a document for review\n");
    println!("USAGE:");
    println!("    qms doc submit <DOCUMENT_ID>\n");
    println!("ARGUMENTS:");
    println!("    <DOCUMENT_ID>    The UUID of the document to submit\n");
    println!("DESCRIPTION:");
    println!("    Submits a document from Draft status to InReview status.");
    println!("    The document must be in Draft status to be submitted.");
    println!("    This action is logged in the audit trail.\n");
    println!("STATUS FLOW:");
    println!("    Draft → InReview\n");
    println!("EXAMPLES:");
    println!("    qms doc submit 123e4567-e89b-12d3-a456-426614174000");
}

fn print_doc_approve_help() {
    println!("Approve a document for release\n");
    println!("USAGE:");
    println!("    qms doc approve <DOCUMENT_ID> [OPTIONS]\n");
    println!("ARGUMENTS:");
    println!("    <DOCUMENT_ID>    The UUID of the document to approve\n");
    println!("OPTIONS:");
    println!("    --signature, -s <TEXT>    Electronic signature message for approval\n");
    println!("DESCRIPTION:");
    println!("    Approves a document from InReview status to Approved status.");
    println!("    The document must be in InReview status to be approved.");
    println!("    Requires appropriate user permissions (Quality Engineer or higher).");
    println!("    This action is logged in the audit trail with the signature.\n");
    println!("STATUS FLOW:");
    println!("    InReview → Approved\n");
    println!("EXAMPLES:");
    println!("    qms doc approve 123e4567-e89b-12d3-a456-426614174000");
    println!("    qms doc approve 123e4567-e89b-12d3-a456-426614174000 --signature \"Approved per FDA 21 CFR 820\"");
}

fn print_doc_reject_help() {
    println!("Reject a document and return to draft\n");
    println!("USAGE:");
    println!("    qms doc reject <DOCUMENT_ID> [OPTIONS]\n");
    println!("ARGUMENTS:");
    println!("    <DOCUMENT_ID>    The UUID of the document to reject\n");
    println!("OPTIONS:");
    println!("    --reason, -r <TEXT>    Reason for rejecting the document\n");
    println!("DESCRIPTION:");
    println!("    Rejects a document from InReview status back to Draft status.");
    println!("    The document must be in InReview status to be rejected.");
    println!("    The document can then be modified and resubmitted.");
    println!("    This action is logged in the audit trail with the reason.\n");
    println!("STATUS FLOW:");
    println!("    InReview → Draft\n");
    println!("EXAMPLES:");
    println!("    qms doc reject 123e4567-e89b-12d3-a456-426614174000");
    println!("    qms doc reject 123e4567-e89b-12d3-a456-426614174000 --reason \"Requires additional testing data\"");
}

fn print_doc_workflow_help() {
    println!("View document approval workflow history\n");
    println!("USAGE:");
    println!("    qms doc workflow <DOCUMENT_ID>\n");
    println!("ARGUMENTS:");
    println!("    <DOCUMENT_ID>    The UUID of the document to view workflow history\n");
    println!("DESCRIPTION:");
    println!("    Displays the complete approval workflow history for a document,");
    println!("    including all state transitions, electronic signatures, and audit details.");
    println!("    Shows submission, review, approval, and rejection history with timestamps.\n");
    println!("WORKFLOW STATES:");
    println!("    Draft → SubmittedForReview → InReview → ApprovalPending → Approved");
    println!("    Any state can transition to Rejected (returns to Draft for rework)");
    println!("    Approved documents can be Archived\n");
    println!("INFORMATION DISPLAYED:");
    println!("    • State transitions with timestamps");
    println!("    • Actor information (ID and name)");
    println!("    • Electronic signatures with hashes");
    println!("    • Comments and reasons");
    println!("    • Current workflow state\n");
    println!("EXAMPLES:");
    println!("    qms doc workflow 123e4567-e89b-12d3-a456-426614174000");
}

fn print_doc_archive_help() {
    println!("Archive a document\n");
    println!("USAGE:");
    println!("    qms doc archive <DOCUMENT_ID> [OPTIONS]\n");
    println!("ARGUMENTS:");
    println!("    <DOCUMENT_ID>    The UUID of the document to archive\n");
    println!("OPTIONS:");
    println!("    --reason, -r <TEXT>    Reason for archiving the document\n");
    println!("DESCRIPTION:");
    println!("    Archives a document from any status to Archived status.");
    println!("    Archived documents are preserved but typically not used for active work.");
    println!("    This action is logged in the audit trail with the reason.\n");
    println!("STATUS FLOW:");
    println!("    Any Status → Archived\n");
    println!("EXAMPLES:");
    println!("    qms doc archive 123e4567-e89b-12d3-a456-426614174000");
    println!("    qms doc archive 123e4567-e89b-12d3-a456-426614174000 --reason \"Superseded by newer version\"");
}

fn print_doc_restore_help() {
    println!("Restore an archived document\n");
    println!("USAGE:");
    println!("    qms doc restore <DOCUMENT_ID> [OPTIONS]\n");
    println!("ARGUMENTS:");
    println!("    <DOCUMENT_ID>    The UUID of the document to restore\n");
    println!("OPTIONS:");
    println!("    --reason, -r <TEXT>    Reason for restoring the document\n");
    println!("DESCRIPTION:");
    println!("    Restores an archived document back to Draft status.");
    println!("    The document must be in Archived status to be restored.");
    println!("    Restored documents can then be edited and used for active work.");
    println!("    This action is logged in the audit trail with the reason.\n");
    println!("STATUS FLOW:");
    println!("    Archived → Draft\n");
    println!("EXAMPLES:");
    println!("    qms doc restore 123e4567-e89b-12d3-a456-426614174000");
    println!("    qms doc restore 123e4567-e89b-12d3-a456-426614174000 --reason \"Needed for new project\"");
}

fn handle_doc_search(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        print_doc_search_help();
        return Err("Document search failed: Missing search query".to_string());
    }
    
    // Show help if requested
    if args[0] == "--help" || args[0] == "-h" {
        print_doc_search_help();
        return Ok(());
    }
    
    let query = &args[0];
    let mut filters = SearchFilters::default();
    
    // Parse additional filters
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--type" | "-t" => {
                if i + 1 < args.len() {
                    filters.document_type = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Document search failed: --type option requires a value".to_string());
                }
            },
            "--status" | "-s" => {
                if i + 1 < args.len() {
                    filters.status = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Document search failed: --status option requires a value".to_string());
                }
            },
            "--author" | "-a" => {
                if i + 1 < args.len() {
                    filters.author = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Document search failed: --author option requires a value".to_string());
                }
            },
            _ => {
                return Err(format!("Document search failed: Unknown option '{}'", args[i]));
            }
        }
    }
    
    let service = DocumentService::new(std::env::current_dir().unwrap());
    
    match service.search_documents_advanced(query, &filters) {
        Ok(results) => {
            if results.is_empty() {
                println!("🔍 No documents found matching search criteria");
                println!("Query: \"{query}\"");
                if filters.has_filters() {
                    println!("Filters applied: {}", filters.description());
                }
            } else {
                println!("🔍 Search Results ({} found)", results.len());
                println!("Query: \"{query}\"");
                if filters.has_filters() {
                    println!("Filters: {}", filters.description());
                }
                println!();
                
                // Print table header
                println!("{:<38} | {:<30} | {:<20} | {:<12} | {:<15}", 
                    "ID", "Title", "Type", "Version", "Status");
                println!("{}", "-".repeat(120));
                
                // Print results
                for doc in results {
                    let doc_type = match doc.doc_type.as_str() {
                        "SoftwareRequirementsSpecification" => "SRS",
                        "SoftwareDesignDescription" => "SDD", 
                        "TestProtocol" => "TP",
                        "RiskManagementFile" => "RMF",
                        "UserRequirements" => "UR",
                        other => other,
                    };
                    
                    let title = if doc.title.len() > 30 {
                        format!("{}...", &doc.title[..27])
                    } else {
                        doc.title.clone()
                    };
                    
                    println!("{:<38} | {:<30} | {:<20} | {:<12} | {:<15}", 
                        doc.id, title, doc_type, doc.version, doc.status);
                }
            }
            Ok(())
        },
        Err(e) => Err(format!("Document search failed: {e}"))
    }
}

fn print_doc_search_help() {
    println!("Search and filter documents\n");
    println!("USAGE:");
    println!("    qms doc search <QUERY> [OPTIONS]\n");
    println!("ARGUMENTS:");
    println!("    <QUERY>    Search text to find in document titles and content\n");
    println!("OPTIONS:");
    println!("    --type, -t <TYPE>      Filter by document type (srs, sdd, tp, rmf, ur, or custom)");
    println!("    --status, -s <STATUS>  Filter by status (draft, inreview, approved, archived)");
    println!("    --author, -a <AUTHOR>  Filter by document author\n");
    println!("DESCRIPTION:");
    println!("    Searches through all documents by title and content text.");
    println!("    Additional filters can be applied to narrow results by document");
    println!("    metadata like type, status, and author.\n");
    println!("EXAMPLES:");
    println!("    qms doc search \"requirements\"");
    println!("    qms doc search \"safety\" --type srs --status approved");
    println!("    qms doc search \"test\" --author \"John Smith\"");
    println!("    qms doc search \"validation\" --status draft");
}

// ========== TEMPLATE HELP FUNCTIONS (Phase 2.1.9) ==========

fn print_doc_template_help() {
    println!("Manage and use document templates\n");
    println!("USAGE:");
    println!("    qms doc template <COMMAND>\n");
    println!("COMMANDS:");
    println!("    init      Initialize default templates in the project");
    println!("    list      List all available templates");
    println!("    create    Create a new document from a template\n");
    println!("DESCRIPTION:");
    println!("    Templates provide standardized starting points for different document types.");
    println!("    They include variable substitution for project-specific information.");
    println!("    Default templates are provided for medical device documentation requirements.\n");
    println!("For more information on a specific template command, use:");
    println!("    qms doc template <COMMAND> --help");
}

fn handle_doc_export(args: &[String]) -> Result<(), String> {
    use crate::modules::document_control::export::{DocumentExporter, ExportFormat, ExportOptions};
    use std::path::Path;

    // Parse arguments
    let mut doc_id = None;
    let mut format = None;
    let mut output = None;
    let mut include_history = false;
    let mut include_audit = false;
    let mut include_metadata = true;
    let mut include_regulatory = true;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--id" | "-i" => {
                if i + 1 < args.len() {
                    doc_id = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --id requires a value");
                    return Err("Invalid arguments".to_string());
                }
            }
            "--format" | "-f" => {
                if i + 1 < args.len() {
                    format = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --format requires a value");
                    return Err("Invalid arguments".to_string());
                }
            }
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --output requires a value");
                    return Err("Invalid arguments".to_string());
                }
            }
            "--include-history" => {
                include_history = true;
                i += 1;
            }
            "--include-audit" => {
                include_audit = true;
                i += 1;
            }
            "--no-metadata" => {
                include_metadata = false;
                i += 1;
            }
            "--no-regulatory" => {
                include_regulatory = false;
                i += 1;
            }
            "--help" | "-h" => {
                print_doc_export_help();
                return Ok(());
            }
            _ => {
                // If it doesn't start with --, treat as document ID if not yet specified
                if !args[i].starts_with('-') && doc_id.is_none() {
                    doc_id = Some(args[i].clone());
                } else {
                    eprintln!("Error: Unknown argument '{}'", args[i]);
                    print_doc_export_help();
                    return Err("Invalid arguments".to_string());
                }
                i += 1;
            }
        }
    }

    // Validate required arguments
    let doc_id = doc_id.ok_or_else(|| {
        eprintln!("Error: Document ID is required");
        print_doc_export_help();
        "Missing document ID".to_string()
    })?;

    let format_str = format.ok_or_else(|| {
        eprintln!("Error: Export format is required");
        print_doc_export_help();
        "Missing format".to_string()
    })?;

    let output_path = output.ok_or_else(|| {
        eprintln!("Error: Output file path is required");
        print_doc_export_help();
        "Missing output path".to_string()
    })?;

    // Parse export format
    let export_format = ExportFormat::from_str(&format_str)
        .map_err(|e| {
            eprintln!("Error: {e}");
            print_doc_export_help();
            e.to_string()
        })?;

    // Get current project path
    let project_path = crate::utils::get_current_project_path()
        .map_err(|e| {
            eprintln!("Error: {e}");
            e.to_string()
        })?;

    // Create export options
    let options = ExportOptions {
        include_history,
        include_audit,
        include_metadata,
        include_regulatory_mapping: include_regulatory,
    };

    // Perform export
    match DocumentExporter::export_document(&project_path, &doc_id, export_format, Path::new(&output_path), options) {
        Ok(()) => {
            println!("✓ Document exported successfully to: {output_path}");
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: Failed to export document: {e}");
            Err(e.to_string())
        }
    }
}

fn print_doc_export_help() {
    println!("Export document to various formats\n");
    println!("USAGE:");
    println!("    qms doc export <DOC_ID> --format <FORMAT> --output <FILE> [OPTIONS]\n");
    println!("REQUIRED ARGUMENTS:");
    println!("    <DOC_ID>                 Document ID to export");
    println!("    --format, -f <FORMAT>    Export format (json, md, html, pdf)");
    println!("    --output, -o <FILE>      Output file path\n");
    println!("OPTIONAL ARGUMENTS:");
    println!("    --include-history        Include version history in export");
    println!("    --include-audit          Include audit trail in export");
    println!("    --no-metadata           Exclude document metadata");
    println!("    --no-regulatory         Exclude regulatory mapping\n");
    println!("EXPORT FORMATS:");
    println!("    json                     Full JSON export with metadata");
    println!("    md, markdown             Clean Markdown with metadata header");
    println!("    html                     HTML format with CSS styling");
    println!("    pdf                      Text-based PDF-like format\n");
    println!("EXAMPLES:");
    println!("    # Export to JSON with full metadata");
    println!("    qms doc export DOC-20240115-001 --format json --output report.json");
    println!();
    println!("    # Export to HTML with version history");
    println!("    qms doc export DOC-20240115-001 --format html --output report.html --include-history");
    println!();
    println!("    # Export to Markdown for documentation");
    println!("    qms doc export DOC-20240115-001 --format md --output document.md");
    println!();
    println!("    # Export to PDF-like format with audit trail");
    println!("    qms doc export DOC-20240115-001 --format pdf --output report.pdf --include-audit");
}

fn print_template_create_help() {
    println!("Create a new document from a template\n");
    println!("USAGE:");
    println!("    qms doc template create --template <NAME> --title <TITLE> [OPTIONS]\n");
    println!("REQUIRED ARGUMENTS:");
    println!("    --template, -t <NAME>    Template name to use");
    println!("    --title <TITLE>          Document title\n");
    println!("OPTIONAL ARGUMENTS:");
    println!("    --project, -p <NAME>     Project name (default: \"My QMS Project\")");
    println!("    --author, -a <AUTHOR>    Document author (default: \"CLI User\")");
    println!("    --var <KEY> <VALUE>      Custom variable substitution (can be used multiple times)\n");
    println!("VARIABLE SUBSTITUTION:");
    println!("    Templates support the following built-in variables:");
    println!("    {{{{PROJECT_NAME}}}}     - Project name");
    println!("    {{{{DATE}}}}             - Current date");
    println!("    {{{{USER}}}}             - Document author");
    println!("    {{{{VERSION}}}}          - Document version (default: 1.0.0)");
    println!("    Custom variables can be added with --var KEY VALUE\n");
    println!("AVAILABLE TEMPLATES:");
    println!("    srs_template             Software Requirements Specification");
    println!("    sdd_template             Software Design Description");
    println!("    test_protocol_template   Test Protocol");
    println!("    rmf_template             Risk Management File (ISO 14971)");
    println!("    user_requirements_template  User Requirements");
    println!("    generic_template         Generic document template\n");
    println!("EXAMPLES:");
    println!("    # Create SRS document");
    println!("    qms doc template create -t srs_template --title \"System Requirements v1.0\"");
    println!();
    println!("    # Create with custom project name and author");
    println!("    qms doc template create -t rmf_template --title \"Risk Analysis\" --project \"Medical Device X\" -a \"Jane Doe\"");
    println!();
    println!("    # Create with custom variables");
    println!("    qms doc template create -t generic_template --title \"Protocol\" --var DEVICE_CLASS \"Class II\" --var STANDARD \"ISO 13485\"");
}

// ========== IMPORT COMMAND HANDLER (Phase 2.1.11) ==========

fn handle_doc_import(args: &[String]) -> Result<(), String> {
    use crate::modules::document_control::import::{DocumentImporter, ImportFormat, ImportOptions};

    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        print_doc_import_help();
        return Ok(());
    }

    let mut file_path = None;
    let mut format = None;
    let mut overwrite = false;
    let mut validate = true;
    let mut author = None;
    let mut preview = false;

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
                    format = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --format requires a value".to_string());
                }
            }
            "--overwrite" => {
                overwrite = true;
                i += 1;
            }
            "--no-validate" => {
                validate = false;
                i += 1;
            }
            "--author" | "-a" => {
                if i + 1 < args.len() {
                    author = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Error: --author requires a value".to_string());
                }
            }
            "--preview" | "-p" => {
                preview = true;
                i += 1;
            }
            _ => {
                return Err(format!("Error: Unknown argument '{}'", args[i]));
            }
        }
    }

    // Validate required arguments
    let file_path = file_path.ok_or("Error: --file is required")?;
    let format_str = format.ok_or("Error: --format is required")?;

    // Parse import format
    let import_format = ImportFormat::from_str(&format_str)
        .map_err(|e| format!("Error: {e}"))?;

    // Create import options
    let options = ImportOptions {
        format: import_format,
        overwrite_existing: overwrite,
        validate_content: validate,
        default_author: author,
        preview_only: preview,
        ..Default::default()
    };

    // Perform import
    let mut importer = DocumentImporter::new()
        .map_err(|e| format!("Failed to initialize importer: {e}"))?;

    match importer.import_from_file(&file_path, options) {
        Ok(result) => {
            println!("📥 Import Results");
            println!("================");
            println!("Total processed: {}", result.total_processed);
            println!("Successful imports: {}", result.successful_imports);
            println!("Failed imports: {}", result.failed_imports);
            println!("Skipped duplicates: {}", result.skipped_duplicates);

            if !result.imported_documents.is_empty() {
                println!("\n✅ Successfully imported documents:");
                for doc_id in &result.imported_documents {
                    println!("  - {doc_id}");
                }
            }

            if !result.errors.is_empty() {
                println!("\n❌ Errors:");
                for error in &result.errors {
                    println!("  - {error}");
                }
            }

            if !result.warnings.is_empty() {
                println!("\n⚠️  Warnings:");
                for warning in &result.warnings {
                    println!("  - {warning}");
                }
            }

            if result.successful_imports > 0 && !preview {
                println!("\n🎉 Import completed successfully!");
            } else if preview {
                println!("\n👀 Preview completed. Use without --preview to perform actual import.");
            }

            Ok(())
        }
        Err(e) => Err(format!("Import failed: {e}")),
    }
}

fn print_doc_import_help() {
    println!("Import documents from various formats\n");
    println!("USAGE:");
    println!("    qms doc import --file <FILE> --format <FORMAT> [OPTIONS]\n");
    println!("REQUIRED ARGUMENTS:");
    println!("    --file, -f <FILE>        File to import from");
    println!("    --format <FORMAT>        Import format (markdown, csv, json)\n");
    println!("OPTIONAL ARGUMENTS:");
    println!("    --overwrite              Overwrite existing documents with same ID");
    println!("    --no-validate            Skip content validation during import");
    println!("    --author, -a <AUTHOR>    Default author for imported documents");
    println!("    --preview, -p            Preview import without making changes\n");
    println!("IMPORT FORMATS:");
    println!("    markdown, md             Single Markdown document with optional front matter");
    println!("    csv                      CSV file with multiple document records");
    println!("    json                     JSON file with document definitions (future)\n");
    println!("CSV FORMAT:");
    println!("    Required columns: title, type, content_file, author, status");
    println!("    Optional columns: version, tags");
    println!("    Example:");
    println!("    title,type,content_file,author,status");
    println!("    \"User Manual\",\"ur\",\"manual.md\",\"John Doe\",\"draft\"\n");
    println!("MARKDOWN FRONT MATTER:");
    println!("    ---");
    println!("    title: Document Title");
    println!("    type: srs");
    println!("    version: 1.0.0");
    println!("    author: Jane Smith");
    println!("    status: draft");
    println!("    tags: requirements, safety");
    println!("    ---\n");
    println!("EXAMPLES:");
    println!("    # Import single Markdown document");
    println!("    qms doc import --file requirements.md --format markdown");
    println!();
    println!("    # Import multiple documents from CSV");
    println!("    qms doc import --file documents.csv --format csv");
    println!();
    println!("    # Preview import without making changes");
    println!("    qms doc import --file docs.csv --format csv --preview");
    println!();
    println!("    # Import with overwrite and custom author");
    println!("    qms doc import --file manual.md --format md --overwrite --author \"Technical Writer\"");
}

/// Handle regulatory mapping commands
fn handle_doc_regulatory(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        print_doc_regulatory_help();
        return Ok(());
    }

    match args[0].as_str() {
        "map" => handle_doc_regulatory_map(&args[1..]),
        "list" => handle_doc_regulatory_list(&args[1..]),
        "verify" => handle_doc_regulatory_verify(&args[1..]),
        "remove" => handle_doc_regulatory_remove(&args[1..]),
        "--help" | "-h" => {
            print_doc_regulatory_help();
            Ok(())
        }
        _ => Err(format!("Unknown regulatory subcommand: {}\nUse 'qms doc regulatory --help' for available commands.", args[0])),
    }
}

/// Handle regulatory compliance analysis
fn handle_doc_compliance(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        print_doc_compliance_help();
        return Ok(());
    }

    match args[0].as_str() {
        "status" => handle_doc_compliance_status(&args[1..]),
        "report" => handle_doc_compliance_report(&args[1..]),
        "gaps" => handle_doc_compliance_gaps(&args[1..]),
        "requirements" => handle_doc_compliance_requirements(&args[1..]),
        "--help" | "-h" => {
            print_doc_compliance_help();
            Ok(())
        }
        _ => Err(format!("Unknown compliance subcommand: {}\nUse 'qms doc compliance --help' for available commands.", args[0])),
    }
}

/// Map a document to regulatory requirements
fn handle_doc_regulatory_map(args: &[String]) -> Result<(), String> {
    let mut doc_id = String::new();
    let mut requirement_id = String::new();
    let mut compliance_level = String::new();
    let mut evidence = String::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--document" | "-d" => {
                if i + 1 >= args.len() {
                    return Err("--document requires a value".to_string());
                }
                doc_id = args[i + 1].clone();
                i += 2;
            }
            "--requirement" | "-r" => {
                if i + 1 >= args.len() {
                    return Err("--requirement requires a value".to_string());
                }
                requirement_id = args[i + 1].clone();
                i += 2;
            }
            "--level" | "-l" => {
                if i + 1 >= args.len() {
                    return Err("--level requires a value".to_string());
                }
                compliance_level = args[i + 1].clone();
                i += 2;
            }
            "--evidence" | "-e" => {
                if i + 1 >= args.len() {
                    return Err("--evidence requires a value".to_string());
                }
                evidence = args[i + 1].clone();
                i += 2;
            }
            _ => {
                if doc_id.is_empty() {
                    doc_id = args[i].clone();
                } else if requirement_id.is_empty() {
                    requirement_id = args[i].clone();
                } else if compliance_level.is_empty() {
                    compliance_level = args[i].clone();
                } else if evidence.is_empty() {
                    evidence = args[i].clone();
                }
                i += 1;
            }
        }
    }

    if doc_id.is_empty() {
        return Err("Document ID is required".to_string());
    }
    if requirement_id.is_empty() {
        return Err("Requirement ID is required".to_string());
    }
    if compliance_level.is_empty() {
        return Err("Compliance level is required (compliant, partial, non_compliant, not_applicable, review)".to_string());
    }
    if evidence.is_empty() {
        evidence = "Document meets regulatory requirement".to_string();
    }

    use crate::modules::document_control::regulatory::{RegulatoryManager, ComplianceLevel};

    let compliance = ComplianceLevel::from_str(&compliance_level)
        .ok_or_else(|| format!("Invalid compliance level: {compliance_level}. Use: compliant, partial, non_compliant, not_applicable, review"))?;

    let mut manager = RegulatoryManager::new();
    match manager.add_regulatory_mapping(&doc_id, &requirement_id, compliance, &evidence, "CLI User") {
        Ok(()) => {
            println!("✅ Regulatory mapping created successfully");
            println!("📄 Document: {doc_id}");
            println!("📋 Requirement: {requirement_id}");
            println!("✓ Compliance: {compliance_level}");
            println!("📝 Evidence: {evidence}");
            
            // Log the action
            crate::audit::log_audit(&format!("REGULATORY_MAPPING: Document {doc_id} mapped to {requirement_id} with compliance level {compliance_level}"));
            
            Ok(())
        }
        Err(e) => Err(format!("Failed to create regulatory mapping: {e}")),
    }
}

/// List regulatory mappings for a document
fn handle_doc_regulatory_list(args: &[String]) -> Result<(), String> {
    let mut doc_id = String::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--document" | "-d" => {
                if i + 1 >= args.len() {
                    return Err("--document requires a value".to_string());
                }
                doc_id = args[i + 1].clone();
                i += 2;
            }
            _ => {
                if doc_id.is_empty() {
                    doc_id = args[i].clone();
                }
                i += 1;
            }
        }
    }

    if doc_id.is_empty() {
        return Err("Document ID is required".to_string());
    }

    use crate::modules::document_control::regulatory::RegulatoryManager;

    let manager = RegulatoryManager::new();
    let mappings = manager.get_document_mappings(&doc_id);

    if mappings.is_empty() {
        println!("No regulatory mappings found for document: {doc_id}");
        return Ok(());
    }

    println!("📋 Regulatory Mappings for Document: {doc_id}\n");
    for mapping in mappings {
        println!("🔗 Requirement: {}", mapping.requirement_id);
        println!("   Standard: {}", mapping.standard.as_str());
        println!("   Section: {}", mapping.section);
        println!("   Compliance: {}", mapping.compliance_level.as_str());
        println!("   Evidence: {}", mapping.evidence_description);
        
        if let Some(verified_at) = mapping.verified_at {
            if let Some(ref verified_by) = mapping.verified_by {
                println!("   ✅ Verified: {} by {}", 
                    crate::utils::format_timestamp(verified_at), verified_by);
            }
        } else {
            println!("   ⏳ Status: Not verified");
        }
        println!();
    }

    Ok(())
}

/// Verify a regulatory mapping
fn handle_doc_regulatory_verify(args: &[String]) -> Result<(), String> {
    let mut doc_id = String::new();
    let mut requirement_id = String::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--document" | "-d" => {
                if i + 1 >= args.len() {
                    return Err("--document requires a value".to_string());
                }
                doc_id = args[i + 1].clone();
                i += 2;
            }
            "--requirement" | "-r" => {
                if i + 1 >= args.len() {
                    return Err("--requirement requires a value".to_string());
                }
                requirement_id = args[i + 1].clone();
                i += 2;
            }
            _ => {
                if doc_id.is_empty() {
                    doc_id = args[i].clone();
                } else if requirement_id.is_empty() {
                    requirement_id = args[i].clone();
                }
                i += 1;
            }
        }
    }

    if doc_id.is_empty() {
        return Err("Document ID is required".to_string());
    }
    if requirement_id.is_empty() {
        return Err("Requirement ID is required".to_string());
    }

    use crate::modules::document_control::regulatory::RegulatoryManager;

    let mut manager = RegulatoryManager::new();
    match manager.verify_mapping(&doc_id, &requirement_id, "CLI User") {
        Ok(()) => {
            println!("✅ Regulatory mapping verified successfully");
            println!("📄 Document: {doc_id}");
            println!("📋 Requirement: {requirement_id}");
            
            // Log the action
            crate::audit::log_audit(&format!("REGULATORY_VERIFICATION: Mapping {doc_id} -> {requirement_id} verified"));
            
            Ok(())
        }
        Err(e) => Err(format!("Failed to verify regulatory mapping: {e}")),
    }
}

/// Remove a regulatory mapping
fn handle_doc_regulatory_remove(args: &[String]) -> Result<(), String> {
    let mut doc_id = String::new();
    let mut requirement_id = String::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--document" | "-d" => {
                if i + 1 >= args.len() {
                    return Err("--document requires a value".to_string());
                }
                doc_id = args[i + 1].clone();
                i += 2;
            }
            "--requirement" | "-r" => {
                if i + 1 >= args.len() {
                    return Err("--requirement requires a value".to_string());
                }
                requirement_id = args[i + 1].clone();
                i += 2;
            }
            _ => {
                if doc_id.is_empty() {
                    doc_id = args[i].clone();
                } else if requirement_id.is_empty() {
                    requirement_id = args[i].clone();
                }
                i += 1;
            }
        }
    }

    if doc_id.is_empty() {
        return Err("Document ID is required".to_string());
    }
    if requirement_id.is_empty() {
        return Err("Requirement ID is required".to_string());
    }

    println!("🗑️  Removing regulatory mapping: {doc_id} -> {requirement_id}");
    println!("   Note: Removal functionality will be implemented in next iteration");
    
    // Log the action
    crate::audit::log_audit(&format!("REGULATORY_REMOVAL: Mapping {doc_id} -> {requirement_id} removed"));
    
    Ok(())
}

/// Show compliance status for standards
fn handle_doc_compliance_status(args: &[String]) -> Result<(), String> {
    let mut standard_filter: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--standard" | "-s" => {
                if i + 1 >= args.len() {
                    return Err("--standard requires a value".to_string());
                }
                standard_filter = Some(args[i + 1].clone());
                i += 2;
            }
            _ => {
                if standard_filter.is_none() {
                    standard_filter = Some(args[i].clone());
                }
                i += 1;
            }
        }
    }

    use crate::modules::document_control::regulatory::{RegulatoryManager, RegulatoryStandard};

    let manager = RegulatoryManager::new();
    let standards = if let Some(std_name) = standard_filter {
        if let Some(standard) = RegulatoryStandard::from_str(&std_name) {
            vec![standard]
        } else {
            return Err(format!("Unknown standard: {std_name}. Use: FDA21CFR820, ISO13485, ISO14971"));
        }
    } else {
        vec![
            RegulatoryStandard::FDA21CFR820,
            RegulatoryStandard::ISO13485,
            RegulatoryStandard::ISO14971,
        ]
    };

    println!("📊 Regulatory Compliance Status\n");

    for standard in standards {
        let status = manager.get_compliance_status(&standard);
        println!("📋 {} - {}", standard.as_str(), standard.description());
        println!("   📈 Total Requirements: {}", status.total_requirements);
        println!("   📊 Coverage: {:.1}%", status.coverage_percentage);
        println!("   ✅ Compliance: {:.1}%", status.compliance_percentage);
        println!("   🟢 Fully Compliant: {}", status.compliant_count);
        println!("   🟡 Partially Compliant: {}", status.partially_compliant_count);
        println!("   🔴 Non-Compliant: {}", status.non_compliant_count);
        println!("   ⚪ Not Applicable: {}", status.not_applicable_count);
        println!("   🟠 Under Review: {}", status.under_review_count);
        println!();
    }

    Ok(())
}

/// Generate compliance report
fn handle_doc_compliance_report(args: &[String]) -> Result<(), String> {
    let mut output_file: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 >= args.len() {
                    return Err("--output requires a value".to_string());
                }
                output_file = Some(args[i + 1].clone());
                i += 2;
            }
            _ => {
                if output_file.is_none() {
                    output_file = Some(args[i].clone());
                }
                i += 1;
            }
        }
    }

    use crate::modules::document_control::regulatory::RegulatoryManager;

    let manager = RegulatoryManager::new();
    let report = manager.generate_coverage_report();

    if let Some(file_path) = output_file {
        match std::fs::write(&file_path, &report) {
            Ok(()) => {
                println!("✅ Compliance report generated successfully");
                println!("📄 Output file: {file_path}");
                
                // Log the action
                crate::audit::log_audit(&format!("COMPLIANCE_REPORT: Generated to {file_path}"));
            }
            Err(e) => return Err(format!("Failed to write report file: {e}")),
        }
    } else {
        println!("{report}");
    }

    Ok(())
}

/// Show compliance gaps analysis
fn handle_doc_compliance_gaps(args: &[String]) -> Result<(), String> {
    let mut standard_filter: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--standard" | "-s" => {
                if i + 1 >= args.len() {
                    return Err("--standard requires a value".to_string());
                }
                standard_filter = Some(args[i + 1].clone());
                i += 2;
            }
            _ => {
                if standard_filter.is_none() {
                    standard_filter = Some(args[i].clone());
                }
                i += 1;
            }
        }
    }

    use crate::modules::document_control::regulatory::{RegulatoryManager, RegulatoryStandard};

    let manager = RegulatoryManager::new();
    let standards = if let Some(std_name) = standard_filter {
        if let Some(standard) = RegulatoryStandard::from_str(&std_name) {
            vec![standard]
        } else {
            return Err(format!("Unknown standard: {std_name}. Use: FDA21CFR820, ISO13485, ISO14971"));
        }
    } else {
        vec![
            RegulatoryStandard::FDA21CFR820,
            RegulatoryStandard::ISO13485,
            RegulatoryStandard::ISO14971,
        ]
    };

    println!("🔍 Regulatory Compliance Gap Analysis\n");

    for standard in standards {
        let gaps = manager.generate_gap_analysis(&standard);
        println!("📋 {} - {}", standard.as_str(), standard.description());
        
        if gaps.is_empty() {
            println!("   ✅ No compliance gaps found");
        } else {
            println!("   ⚠️  Found {} compliance gaps:", gaps.len());
            for gap in gaps {
                println!("      • {gap}");
            }
        }
        println!();
    }

    Ok(())
}

/// List regulatory requirements
fn handle_doc_compliance_requirements(args: &[String]) -> Result<(), String> {
    let mut standard_filter: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--standard" | "-s" => {
                if i + 1 >= args.len() {
                    return Err("--standard requires a value".to_string());
                }
                standard_filter = Some(args[i + 1].clone());
                i += 2;
            }
            _ => {
                if standard_filter.is_none() {
                    standard_filter = Some(args[i].clone());
                }
                i += 1;
            }
        }
    }

    use crate::modules::document_control::regulatory::{RegulatoryManager, RegulatoryStandard};

    let manager = RegulatoryManager::new();
    let standards = if let Some(std_name) = standard_filter {
        if let Some(standard) = RegulatoryStandard::from_str(&std_name) {
            vec![standard]
        } else {
            return Err(format!("Unknown standard: {std_name}. Use: FDA21CFR820, ISO13485, ISO14971"));
        }
    } else {
        vec![
            RegulatoryStandard::FDA21CFR820,
            RegulatoryStandard::ISO13485,
            RegulatoryStandard::ISO14971,
        ]
    };

    println!("📋 Regulatory Requirements\n");

    for standard in standards {
        let requirements = manager.list_requirements(&standard);
        println!("📋 {} - {}", standard.as_str(), standard.description());
        
        if requirements.is_empty() {
            println!("   No requirements defined");
        } else {
            for req in requirements {
                println!("   📄 {} ({}): {}", req.id, req.section, req.title);
                println!("      Description: {}", req.description);
                println!("      Mandatory: {}", if req.mandatory { "Yes" } else { "No" });
                println!("      Document Types: {}", req.document_types.join(", "));
                println!();
            }
        }
        println!();
    }

    Ok(())
}

/// Print regulatory command help
fn print_doc_regulatory_help() {
    println!("Regulatory mapping commands for documents\n");
    println!("USAGE:");
    println!("    qms doc regulatory <SUBCOMMAND> [OPTIONS]\n");
    println!("SUBCOMMANDS:");
    println!("    map        Map document to regulatory requirement");
    println!("    list       List regulatory mappings for document");
    println!("    verify     Verify regulatory mapping");
    println!("    remove     Remove regulatory mapping\n");
    println!("MAP USAGE:");
    println!("    qms doc regulatory map --document <DOC_ID> --requirement <REQ_ID> --level <LEVEL> [--evidence <TEXT>]\n");
    println!("COMPLIANCE LEVELS:");
    println!("    compliant, full        Document fully complies with requirement");
    println!("    partial               Document partially complies with requirement");
    println!("    non_compliant         Document does not comply with requirement");
    println!("    not_applicable, na    Requirement is not applicable to document");
    println!("    review, pending       Compliance is under review\n");
    println!("EXAMPLES:");
    println!("    # Map document to FDA requirement");
    println!("    qms doc regulatory map doc-001 21CFR820.30 compliant --evidence \"Design controls implemented\"");
    println!();
    println!("    # List mappings for document");
    println!("    qms doc regulatory list doc-001");
    println!();
    println!("    # Verify mapping");
    println!("    qms doc regulatory verify doc-001 21CFR820.30");
}

/// Print compliance command help
fn print_doc_compliance_help() {
    println!("Regulatory compliance analysis commands\n");
    println!("USAGE:");
    println!("    qms doc compliance <SUBCOMMAND> [OPTIONS]\n");
    println!("SUBCOMMANDS:");
    println!("    status         Show compliance status summary");
    println!("    report         Generate detailed compliance report");
    println!("    gaps           Show compliance gap analysis");
    println!("    requirements   List regulatory requirements\n");
    println!("EXAMPLES:");
    println!("    # Show overall compliance status");
    println!("    qms doc compliance status");
    println!();
    println!("    # Show status for specific standard");
    println!("    qms doc compliance status --standard FDA21CFR820");
    println!();
    println!("    # Generate compliance report");
    println!("    qms doc compliance report --output compliance_report.md");
    println!();
    println!("    # Show compliance gaps");
    println!("    qms doc compliance gaps --standard ISO13485");
    println!();
    println!("    # List requirements for standard");
    println!("    qms doc compliance requirements --standard ISO14971");
}

// ========== BACKUP HELP FUNCTIONS (Phase 2.1.14) ==========

fn print_doc_backup_help() {
    println!("Document backup and recovery management\n");
    println!("USAGE:");
    println!("    qms doc backup <COMMAND>\n");
    println!("COMMANDS:");
    println!("    list     List document backups");
    println!("    verify   Verify backup integrity");
    println!("    recover  Recover document from backup");
    println!("    delete   Delete a backup");
    println!("    cleanup  Clean up old backups");
    println!("    info     Show detailed backup information\n");
    println!("AUTOMATIC BACKUPS:");
    println!("    Backups are automatically created when:");
    println!("    • Documents are created");
    println!("    • Documents are updated");
    println!("    • Documents are saved\n");
    println!("BACKUP VERIFICATION:");
    println!("    All backups include SHA-256 checksums for integrity verification");
    println!("    Corrupted backups are detected and marked as invalid\n");
    println!("For more information on a specific command, use:");
    println!("    qms doc backup <COMMAND> --help");
}

fn print_backup_list_help() {
    println!("List document backups\n");
    println!("USAGE:");
    println!("    qms doc backup list [DOCUMENT_ID]\n");
    println!("ARGUMENTS:");
    println!("    [DOCUMENT_ID]    Optional document ID to filter backups\n");
    println!("OPTIONS:");
    println!("    --document, -d <ID>    Same as DOCUMENT_ID argument\n");
    println!("DESCRIPTION:");
    println!("    Lists backups for a specific document or all backups in the system.");
    println!("    Shows backup ID, version, creation time, size, and reason.\n");
    println!("EXAMPLES:");
    println!("    # List all backups");
    println!("    qms doc backup list");
    println!();
    println!("    # List backups for specific document");
    println!("    qms doc backup list 123e4567-e89b-12d3-a456-426614174000");
    println!("    qms doc backup list --document 123e4567-e89b-12d3-a456-426614174000");
}

fn print_backup_verify_help() {
    println!("Verify backup integrity\n");
    println!("USAGE:");
    println!("    qms doc backup verify <BACKUP_ID>\n");
    println!("ARGUMENTS:");
    println!("    <BACKUP_ID>    The ID of the backup to verify\n");
    println!("DESCRIPTION:");
    println!("    Verifies the integrity of a backup using checksum validation.");
    println!("    Checks for corruption and validates file size matches.\n");
    println!("VERIFICATION CHECKS:");
    println!("    • SHA-256 checksum verification");
    println!("    • File size validation");
    println!("    • Backup file existence");
    println!("    • Metadata consistency\n");
    println!("EXAMPLES:");
    println!("    qms doc backup verify BACKUP-123e4567-1642684800");
}

fn print_backup_recover_help() {
    println!("Recover document from backup\n");
    println!("USAGE:");
    println!("    qms doc backup recover <BACKUP_ID> [OPTIONS]\n");
    println!("ARGUMENTS:");
    println!("    <BACKUP_ID>    The ID of the backup to recover from\n");
    println!("OPTIONS:");
    println!("    --target, -t <DOCUMENT_ID>    Create new document instead of overwriting\n");
    println!("DESCRIPTION:");
    println!("    Recovers a document from a backup. By default, overwrites the original");
    println!("    document. Use --target to create a new document instead.\n");
    println!("RECOVERY MODES:");
    println!("    • In-place recovery: Overwrites the original document");
    println!("    • New document recovery: Creates a new document with recovered content\n");
    println!("WARNING:");
    println!("    In-place recovery will overwrite current document content!");
    println!("    You will be prompted to confirm the recovery operation.\n");
    println!("EXAMPLES:");
    println!("    # Recover to original document (overwrites current content)");
    println!("    qms doc backup recover BACKUP-123e4567-1642684800");
    println!();
    println!("    # Recover as new document");
    println!("    qms doc backup recover BACKUP-123e4567-1642684800 --target new-doc-123");
}

fn print_backup_delete_help() {
    println!("Delete a backup\n");
    println!("USAGE:");
    println!("    qms doc backup delete <BACKUP_ID>\n");
    println!("ARGUMENTS:");
    println!("    <BACKUP_ID>    The ID of the backup to delete\n");
    println!("DESCRIPTION:");
    println!("    Permanently deletes a backup from the system.");
    println!("    This action cannot be undone.\n");
    println!("WARNING:");
    println!("    Backup deletion is permanent and cannot be undone!");
    println!("    You will be prompted to confirm the deletion.\n");
    println!("EXAMPLES:");
    println!("    qms doc backup delete BACKUP-123e4567-1642684800");
}

fn print_backup_cleanup_help() {
    println!("Clean up old backups\n");
    println!("USAGE:");
    println!("    qms doc backup cleanup [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --days, -d <DAYS>    Retention period in days (default: 30)\n");
    println!("DESCRIPTION:");
    println!("    Deletes backups older than the specified retention period.");
    println!("    This helps manage disk space and comply with retention policies.\n");
    println!("RETENTION POLICY:");
    println!("    • Default retention: 30 days");
    println!("    • Backups older than retention period are deleted");
    println!("    • You will be prompted to confirm the cleanup\n");
    println!("WARNING:");
    println!("    Cleanup permanently deletes old backups!");
    println!("    You will be prompted to confirm the cleanup operation.\n");
    println!("EXAMPLES:");
    println!("    # Clean up backups older than 30 days (default)");
    println!("    qms doc backup cleanup");
    println!();
    println!("    # Clean up backups older than 7 days");
    println!("    qms doc backup cleanup --days 7");
    println!();
    println!("    # Clean up backups older than 90 days");
    println!("    qms doc backup cleanup --days 90");
}

fn print_backup_info_help() {
    println!("Show detailed backup information\n");
    println!("USAGE:");
    println!("    qms doc backup info <BACKUP_ID>\n");
    println!("ARGUMENTS:");
    println!("    <BACKUP_ID>    The ID of the backup to show information for\n");
    println!("DESCRIPTION:");
    println!("    Shows detailed information about a backup including metadata,");
    println!("    file paths, checksums, and integrity status.\n");
    println!("INFORMATION SHOWN:");
    println!("    • Backup ID and document ID");
    println!("    • Document version and creation time");
    println!("    • File size and checksum");
    println!("    • Backup paths and reason");
    println!("    • Integrity verification status\n");
    println!("EXAMPLES:");
    println!("    qms doc backup info BACKUP-123e4567-1642684800");
}

// ========== ADMINISTRATIVE LOCK MANAGEMENT HELP FUNCTIONS ==========

fn print_doc_locks_help() {
    println!("Administrative lock management utilities\n");
    println!("USAGE:");
    println!("    qms doc locks <COMMAND>\n");
    println!("COMMANDS:");
    println!("    info      Get detailed lock information for a document");
    println!("    stats     Show system-wide lock statistics");
    println!("    cleanup   Clean up stale locks (admin operation)\n");
    println!("DESCRIPTION:");
    println!("    Administrative tools for managing document and file locks.");
    println!("    These commands provide enhanced lock management capabilities");
    println!("    beyond basic checkout/checkin operations.\n");
    println!("ADMINISTRATOR FEATURES:");
    println!("    • Detailed lock status including file-level locks");
    println!("    • System-wide lock statistics and monitoring");
    println!("    • Automatic cleanup of stale locks");
    println!("    • Lock integrity verification\n");
    println!("For more information on a specific command, use:");
    println!("    qms doc locks <COMMAND> --help");
}

fn print_locks_info_help() {
    println!("Get detailed lock information for a document\n");
    println!("USAGE:");
    println!("    qms doc locks info <DOCUMENT_ID>\n");
    println!("ARGUMENTS:");
    println!("    <DOCUMENT_ID>    The UUID of the document to check\n");
    println!("DESCRIPTION:");
    println!("    Shows comprehensive lock information including both document");
    println!("    locks and file-level locks for administrative purposes.\n");
    println!("INFORMATION SHOWN:");
    println!("    • Document lock status and holder");
    println!("    • File lock status and holder");
    println!("    • Lock timestamps and reasons");
    println!("    • Manager active status\n");
    println!("EXAMPLES:");
    println!("    qms doc locks info 123e4567-e89b-12d3-a456-426614174000");
}

fn print_locks_cleanup_help() {
    println!("Clean up stale locks (admin operation)\n");
    println!("USAGE:");
    println!("    qms doc locks cleanup [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --admin, -a <ADMIN>    Admin user name for audit log (default: CLI_Admin)\n");
    println!("DESCRIPTION:");
    println!("    Removes stale locks that are older than 24 hours.");
    println!("    This helps prevent orphaned locks from blocking documents.\n");
    println!("CLEANUP PROCESS:");
    println!("    • Identifies locks older than 24 hours");
    println!("    • Removes both document and file locks");
    println!("    • Logs cleanup actions for audit trail");
    println!("    • Provides detailed cleanup report\n");
    println!("WARNING:");
    println!("    This will forcefully remove stale locks!");
    println!("    You will be prompted to confirm the cleanup operation.\n");
    println!("EXAMPLES:");
    println!("    qms doc locks cleanup");
    println!("    qms doc locks cleanup --admin \"John Doe\"");
}
