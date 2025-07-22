/// SOLID Principles Enhancement: Audit Command Handlers
/// 
/// This module breaks down the monolithic audit command handler into focused,
/// single-responsibility handlers following SOLID principles.

use crate::prelude::*;
use crate::commands::command_handler_trait::{CommandHandler, BaseCommandHandler, CommandContext};
use crate::modules::audit_logger::{
    initialize_audit_system, get_audit_statistics, AuditConfig,
    set_current_session, clear_current_session, ExportFormat, ExportOptions,
    AuditExportEngine, AuditBackupManager
};
use crate::utils::generate_uuid;
use crate::impl_command_handler;

/// Audit Initialization Handler - Single Responsibility Principle
/// Focuses solely on initializing the audit system
pub struct AuditInitHandler {
    context: CommandContext,
}

impl AuditInitHandler {
    pub fn new(context: CommandContext) -> Self {
        Self { context }
    }
}

impl BaseCommandHandler for AuditInitHandler {
    fn do_execute(&self, args: &[String]) -> QmsResult<()> {
        let mut config = if args.contains(&"--medical-device".to_string()) {
            AuditConfig::medical_device_default()
        } else {
            AuditConfig::default()
        };

        // Set the project path from context
        config.project_path = self.context.project_path.to_string_lossy().to_string();

        initialize_audit_system(config)?;
        
        println!("✅ Audit system initialized successfully!");
        println!("📁 Audit logs will be stored in: {}/audit/", self.context.project_path.display());
        
        Ok(())
    }
}

impl_command_handler!(
    AuditInitHandler,
    "init",
    "Initialize the audit logging system for the current project"
);

/// Audit Statistics Handler - Single Responsibility Principle
/// Focuses solely on displaying audit statistics
pub struct AuditStatsHandler {
    context: CommandContext,
}

impl AuditStatsHandler {
    pub fn new(context: CommandContext) -> Self {
        Self { context }
    }
}

impl BaseCommandHandler for AuditStatsHandler {
    fn do_execute(&self, _args: &[String]) -> QmsResult<()> {
        let stats = get_audit_statistics()?;
        
        println!("📊 Audit System Statistics");
        println!("==========================");
        println!();
        println!("Total Entries: {}", stats.total_entries);
        println!("Total Files: {}", stats.file_count);
        println!("Total Size: {} bytes", stats.total_size_bytes);
        println!("Oldest Entry: {}", stats.oldest_entry_date.unwrap_or_else(|| "N/A".to_string()));
        println!("Newest Entry: {}", stats.newest_entry_date.unwrap_or_else(|| "N/A".to_string()));
        println!();
        println!("Entries Today: {}", stats.entries_today);
        
        Ok(())
    }
}

impl_command_handler!(
    AuditStatsHandler,
    "stats",
    "Display comprehensive audit system statistics"
);

/// Audit Session Handler - Single Responsibility Principle
/// Focuses solely on session management (login/logout)
pub struct AuditSessionHandler {
    context: CommandContext,
}

impl AuditSessionHandler {
    pub fn new(context: CommandContext) -> Self {
        Self { context }
    }
    
    pub fn handle_login(&self, args: &[String]) -> QmsResult<()> {
        if args.is_empty() {
            return Err(QmsError::validation_error("Usage: login <username>"));
        }
        
        let username = &args[0];
        let session_id = generate_uuid();
        
        set_current_session(username.clone(), session_id.clone(), None)?;
        
        println!("✅ Audit session started for user: {}", username);
        println!("Session ID: {}", session_id);
        
        Ok(())
    }
    
    pub fn handle_logout(&self, _args: &[String]) -> QmsResult<()> {
        clear_current_session()?;
        println!("✅ Audit session ended");
        Ok(())
    }
}

impl BaseCommandHandler for AuditSessionHandler {
    fn do_execute(&self, args: &[String]) -> QmsResult<()> {
        if args.is_empty() {
            return Err(QmsError::validation_error("Usage: session <login|logout> [args...]"));
        }
        
        match args[0].as_str() {
            "login" => self.handle_login(&args[1..]),
            "logout" => self.handle_logout(&args[1..]),
            _ => Err(QmsError::validation_error("Unknown session command. Use 'login' or 'logout'")),
        }
    }
}

impl_command_handler!(
    AuditSessionHandler,
    "session",
    "Manage audit sessions (login/logout)"
);

/// Audit Export Handler - Single Responsibility Principle
/// Focuses solely on exporting audit data
pub struct AuditExportHandler {
    context: CommandContext,
}

impl AuditExportHandler {
    pub fn new(context: CommandContext) -> Self {
        Self { context }
    }
}

impl BaseCommandHandler for AuditExportHandler {
    fn do_execute(&self, args: &[String]) -> QmsResult<()> {
        let mut format = ExportFormat::JSON;
        let mut output_path = std::path::PathBuf::from("audit_export");
        let mut filter = None;
        
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--format" if i + 1 < args.len() => {
                    format = match args[i + 1].as_str() {
                        "json" => ExportFormat::JSON,
                        "csv" => ExportFormat::CSV,
                        "pdf" => ExportFormat::PDF,
                        "xml" => ExportFormat::XML,
                        _ => return Err(QmsError::validation_error("Invalid format. Use: json, csv, pdf, xml")),
                    };
                    i += 2;
                }
                "--output" if i + 1 < args.len() => {
                    output_path = std::path::PathBuf::from(&args[i + 1]);
                    i += 2;
                }
                "--filter" if i + 1 < args.len() => {
                    filter = Some(args[i + 1].clone());
                    i += 2;
                }
                _ => i += 1,
            }
        }
        
        let options = ExportOptions {
            format,
            output_path,
            filter,
            include_headers: true,
            include_metadata: true,
            max_entries: None,
        };
        
        let export_engine = AuditExportEngine::new(self.context.project_path.clone());
        let result = export_engine.export_audit_logs(&options)?;
        
        println!("✅ Audit data exported successfully!");
        println!("📄 Export format: {:?}", result.export_format);
        println!("📊 Entries exported: {}", result.exported_entries);
        println!("💾 File size: {} bytes", result.file_size);
        
        Ok(())
    }
}

impl_command_handler!(
    AuditExportHandler,
    "export",
    "Export audit data in various formats (JSON, CSV, PDF, XML)"
);

/// Audit Backup Handler - Single Responsibility Principle
/// Focuses solely on backup operations
pub struct AuditBackupHandler {
    context: CommandContext,
}

impl AuditBackupHandler {
    pub fn new(context: CommandContext) -> Self {
        Self { context }
    }
}

impl BaseCommandHandler for AuditBackupHandler {
    fn do_execute(&self, args: &[String]) -> QmsResult<()> {
        if args.is_empty() {
            return Err(QmsError::validation_error(
                "Usage: backup <create|list|restore|verify> [options...]"
            ));
        }
        
        let backup_manager = AuditBackupManager::new(self.context.project_path.clone());
        
        match args[0].as_str() {
            "create" => {
                let backup_stats = backup_manager.create_backup()?;
                println!("✅ Backup created successfully!");
                println!("Files backed up: {}", backup_stats.files_backed_up);
                println!("Bytes backed up: {}", backup_stats.bytes_backed_up);
                println!("Duration: {}ms", backup_stats.backup_duration_ms);
            }
            "list" => {
                let backups = backup_manager.list_backups()?;
                println!("📋 Available Backups");
                println!("====================");
                for backup in backups {
                    println!("🔸 {} - {} ({} bytes)",
                        backup.backup_id, backup.timestamp, backup.total_size);
                }
            }
            "restore" => {
                if args.len() < 2 {
                    return Err(QmsError::validation_error("Usage: backup restore <backup_id>"));
                }
                backup_manager.restore_backup(&args[1])?;
                println!("✅ Backup restored successfully!");
            }
            "verify" => {
                if args.len() < 2 {
                    return Err(QmsError::validation_error("Usage: backup verify <backup_id>"));
                }
                let is_valid = backup_manager.verify_backup(&args[1])?;
                if is_valid {
                    println!("✅ Backup verification passed");
                } else {
                    println!("❌ Backup verification failed");
                }
            }
            _ => return Err(QmsError::validation_error(
                "Unknown backup command. Use: create, list, restore, verify"
            )),
        }
        
        Ok(())
    }
}

impl_command_handler!(
    AuditBackupHandler,
    "backup",
    "Manage audit data backups (create, list, restore, verify)"
);

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    fn create_test_context() -> CommandContext {
        CommandContext {
            project_path: PathBuf::from("test"),
            user_id: Some("test_user".to_string()),
            session_id: Some("test_session".to_string()),
        }
    }
    
    #[test]
    fn test_audit_session_handler_validation() {
        let handler = AuditSessionHandler::new(create_test_context());
        let result = handler.do_execute(&[]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_audit_backup_handler_validation() {
        let handler = AuditBackupHandler::new(create_test_context());
        let result = handler.do_execute(&[]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_command_names() {
        let init_handler = AuditInitHandler::new(create_test_context());
        let stats_handler = AuditStatsHandler::new(create_test_context());
        let session_handler = AuditSessionHandler::new(create_test_context());
        
        assert_eq!(init_handler.command_name(), "init");
        assert_eq!(stats_handler.command_name(), "stats");
        assert_eq!(session_handler.command_name(), "session");
    }
}
