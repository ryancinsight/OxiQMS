//! Command Pattern Implementation for Audit Operations
//! 
//! This module implements the Command pattern for audit operations,
//! supporting undo/redo functionality and operation queuing.
//! 
//! SOLID Principles Applied:
//! - Single Responsibility: Each command handles one specific audit operation
//! - Open/Closed: New commands can be added without modifying existing code
//! - Liskov Substitution: All commands can be used interchangeably
//! - Interface Segregation: Focused interfaces for different command types
//! - Dependency Inversion: Command invoker depends on command abstractions

use crate::prelude::*;
use crate::models::{AuditEntry, AuditAction};
use crate::json_utils::JsonSerializable;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Command pattern interface for audit operations
/// Interface Segregation Principle: Focused interface for audit commands
pub trait AuditCommand: Send + Sync {
    /// Execute the audit command
    fn execute(&self) -> QmsResult<AuditCommandResult>;
    
    /// Undo the audit command (if possible)
    fn undo(&self) -> QmsResult<AuditCommandResult>;
    
    /// Check if this command can be undone
    fn can_undo(&self) -> bool;
    
    /// Get command description for logging and debugging
    fn description(&self) -> String;
    
    /// Get command type for categorization
    fn command_type(&self) -> AuditCommandType;
    
    /// Get command priority (higher numbers = higher priority)
    fn priority(&self) -> u8 {
        50 // Default priority
    }
}

/// Types of audit commands
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AuditCommandType {
    CreateEntry,
    UpdateEntry,
    DeleteEntry,
    BatchOperation,
    SystemMaintenance,
    ComplianceCheck,
}

/// Result of command execution
#[derive(Debug, Clone)]
pub struct AuditCommandResult {
    pub success: bool,
    pub message: String,
    pub affected_entries: Vec<String>, // Entry IDs that were affected
    pub execution_time_ms: u64,
    pub metadata: std::collections::HashMap<String, String>,
}

impl AuditCommandResult {
    pub fn success(message: String) -> Self {
        Self {
            success: true,
            message,
            affected_entries: Vec::new(),
            execution_time_ms: 0,
            metadata: std::collections::HashMap::new(),
        }
    }
    
    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            message,
            affected_entries: Vec::new(),
            execution_time_ms: 0,
            metadata: std::collections::HashMap::new(),
        }
    }
    
    pub fn with_affected_entries(mut self, entries: Vec<String>) -> Self {
        self.affected_entries = entries;
        self
    }
    
    pub const fn with_execution_time(mut self, time_ms: u64) -> Self {
        self.execution_time_ms = time_ms;
        self
    }
}

/// Command to create a new audit entry
/// Single Responsibility Principle: Handles only audit entry creation
pub struct CreateAuditEntryCommand {
    entry: AuditEntry,
    storage_path: std::path::PathBuf,
}

impl CreateAuditEntryCommand {
    pub const fn new(entry: AuditEntry, storage_path: std::path::PathBuf) -> Self {
        Self { entry, storage_path }
    }
}

impl AuditCommand for CreateAuditEntryCommand {
    fn execute(&self) -> QmsResult<AuditCommandResult> {
        let start_time = crate::utils::current_timestamp();
        
        // Ensure audit directory exists
        if let Some(parent) = self.storage_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Append entry to audit log
        use std::fs::OpenOptions;
        use std::io::Write;
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.storage_path)?;
        
        let json_entry = self.entry.to_json();
        writeln!(file, "{json_entry}")?;
        file.flush()?;
        
        let end_time = crate::utils::current_timestamp();
        
        Ok(AuditCommandResult::success(
            format!("Created audit entry: {}", self.entry.id)
        )
        .with_affected_entries(vec![self.entry.id.clone()])
        .with_execution_time(end_time - start_time))
    }
    
    fn undo(&self) -> QmsResult<AuditCommandResult> {
        // For audit logs, we typically don't delete entries for compliance reasons
        // Instead, we create a compensating entry
        let compensating_entry = AuditEntry {
            id: crate::utils::generate_uuid(),
            timestamp: crate::utils::current_iso8601_timestamp(),
            user_id: "system".to_string(),
            session_id: None,
            action: AuditAction::Delete,
            entity_type: "AuditEntry".to_string(),
            entity_id: self.entry.id.clone(),
            old_value: Some(self.entry.to_json()),
            new_value: None,
            details: Some("Compensating entry for undone audit entry creation".to_string()),
            ip_address: None,
            signature: None,
            checksum: String::new(),
            previous_hash: None,
        };
        
        let compensating_command = CreateAuditEntryCommand::new(
            compensating_entry,
            self.storage_path.clone()
        );
        
        compensating_command.execute()
    }
    
    fn can_undo(&self) -> bool {
        true // Can create compensating entry
    }
    
    fn description(&self) -> String {
        format!("Create audit entry for {:?} action on {} {}",
            self.entry.action, self.entry.entity_type, self.entry.entity_id)
    }
    
    fn command_type(&self) -> AuditCommandType {
        AuditCommandType::CreateEntry
    }
    
    fn priority(&self) -> u8 {
        70 // High priority for audit entry creation
    }
}

/// Command to perform batch audit operations
/// Single Responsibility Principle: Handles only batch operations
pub struct BatchAuditCommand {
    commands: Vec<Box<dyn AuditCommand>>,
    description: String,
}

impl BatchAuditCommand {
    pub fn new(commands: Vec<Box<dyn AuditCommand>>, description: String) -> Self {
        Self { commands, description }
    }
}

impl AuditCommand for BatchAuditCommand {
    fn execute(&self) -> QmsResult<AuditCommandResult> {
        let start_time = crate::utils::current_timestamp();
        let mut affected_entries = Vec::new();
        let mut errors = Vec::new();
        
        for (i, command) in self.commands.iter().enumerate() {
            match command.execute() {
                Ok(result) => {
                    affected_entries.extend(result.affected_entries);
                }
                Err(e) => {
                    errors.push(format!("Command {i}: {e}"));
                }
            }
        }
        
        let end_time = crate::utils::current_timestamp();
        
        if errors.is_empty() {
            Ok(AuditCommandResult::success(
                format!("Batch operation completed: {}", self.description)
            )
            .with_affected_entries(affected_entries)
            .with_execution_time(end_time - start_time))
        } else {
            Err(QmsError::domain_error(&format!(
                "Batch operation failed: {}", errors.join("; ")
            )))
        }
    }
    
    fn undo(&self) -> QmsResult<AuditCommandResult> {
        let start_time = crate::utils::current_timestamp();
        let mut affected_entries = Vec::new();
        let mut errors = Vec::new();
        
        // Undo commands in reverse order
        for (i, command) in self.commands.iter().rev().enumerate() {
            if command.can_undo() {
                match command.undo() {
                    Ok(result) => {
                        affected_entries.extend(result.affected_entries);
                    }
                    Err(e) => {
                        errors.push(format!("Undo command {i}: {e}"));
                    }
                }
            }
        }
        
        let end_time = crate::utils::current_timestamp();
        
        if errors.is_empty() {
            Ok(AuditCommandResult::success(
                format!("Batch operation undone: {}", self.description)
            )
            .with_affected_entries(affected_entries)
            .with_execution_time(end_time - start_time))
        } else {
            Err(QmsError::domain_error(&format!(
                "Batch undo failed: {}", errors.join("; ")
            )))
        }
    }
    
    fn can_undo(&self) -> bool {
        self.commands.iter().all(|cmd| cmd.can_undo())
    }
    
    fn description(&self) -> String {
        self.description.clone()
    }
    
    fn command_type(&self) -> AuditCommandType {
        AuditCommandType::BatchOperation
    }
    
    fn priority(&self) -> u8 {
        60 // Medium-high priority for batch operations
    }
}

/// Command invoker that manages command execution and history
/// Single Responsibility Principle: Manages command execution and history
pub struct AuditCommandInvoker {
    command_history: Arc<Mutex<VecDeque<Box<dyn AuditCommand>>>>,
    undo_stack: Arc<Mutex<VecDeque<Box<dyn AuditCommand>>>>,
    max_history_size: usize,
}

impl AuditCommandInvoker {
    pub fn new(max_history_size: usize) -> Self {
        Self {
            command_history: Arc::new(Mutex::new(VecDeque::new())),
            undo_stack: Arc::new(Mutex::new(VecDeque::new())),
            max_history_size,
        }
    }
    
    /// Execute a command and add it to history
    /// Open/Closed Principle: Can execute any command that implements AuditCommand
    pub fn execute_command(&self, command: Box<dyn AuditCommand>) -> QmsResult<AuditCommandResult> {
        let result = command.execute()?;
        
        // Add to history if execution was successful
        if result.success {
            let mut history = self.command_history.lock()
                .map_err(|_| QmsError::domain_error("Failed to acquire command history lock"))?;
            
            history.push_back(command);
            
            // Maintain history size limit
            while history.len() > self.max_history_size {
                history.pop_front();
            }
            
            // Clear undo stack when new command is executed
            let mut undo_stack = self.undo_stack.lock()
                .map_err(|_| QmsError::domain_error("Failed to acquire undo stack lock"))?;
            undo_stack.clear();
        }
        
        Ok(result)
    }
    
    /// Undo the last command
    pub fn undo_last_command(&self) -> QmsResult<AuditCommandResult> {
        let mut history = self.command_history.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire command history lock"))?;
        
        if let Some(command) = history.pop_back() {
            if command.can_undo() {
                let result = command.undo()?;
                
                // Move to undo stack
                let mut undo_stack = self.undo_stack.lock()
                    .map_err(|_| QmsError::domain_error("Failed to acquire undo stack lock"))?;
                undo_stack.push_back(command);
                
                Ok(result)
            } else {
                // Put command back in history
                history.push_back(command);
                Err(QmsError::domain_error("Last command cannot be undone"))
            }
        } else {
            Err(QmsError::domain_error("No commands to undo"))
        }
    }
    
    /// Redo the last undone command
    pub fn redo_last_command(&self) -> QmsResult<AuditCommandResult> {
        let mut undo_stack = self.undo_stack.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire undo stack lock"))?;
        
        if let Some(command) = undo_stack.pop_back() {
            let result = command.execute()?;
            
            // Move back to history
            let mut history = self.command_history.lock()
                .map_err(|_| QmsError::domain_error("Failed to acquire command history lock"))?;
            history.push_back(command);
            
            Ok(result)
        } else {
            Err(QmsError::domain_error("No commands to redo"))
        }
    }
    
    /// Get command history summary
    pub fn get_history_summary(&self) -> QmsResult<Vec<String>> {
        let history = self.command_history.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire command history lock"))?;
        
        Ok(history.iter().map(|cmd| cmd.description()).collect())
    }
    
    /// Clear command history
    pub fn clear_history(&self) -> QmsResult<()> {
        let mut history = self.command_history.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire command history lock"))?;
        let mut undo_stack = self.undo_stack.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire undo stack lock"))?;
        
        history.clear();
        undo_stack.clear();
        
        Ok(())
    }
    
    /// Get statistics about command execution
    pub fn get_statistics(&self) -> QmsResult<CommandStatistics> {
        let history = self.command_history.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire command history lock"))?;
        let undo_stack = self.undo_stack.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire undo stack lock"))?;
        
        let mut type_counts = std::collections::HashMap::new();
        for command in history.iter() {
            *type_counts.entry(command.command_type()).or_insert(0) += 1;
        }
        
        Ok(CommandStatistics {
            total_commands: history.len(),
            undoable_commands: undo_stack.len(),
            command_type_counts: type_counts,
        })
    }
}

impl Default for AuditCommandInvoker {
    fn default() -> Self {
        Self::new(1000) // Default history size
    }
}

/// Statistics about command execution
#[derive(Debug, Clone)]
pub struct CommandStatistics {
    pub total_commands: usize,
    pub undoable_commands: usize,
    pub command_type_counts: std::collections::HashMap<AuditCommandType, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_audit_entry_command() {
        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        
        let entry = AuditEntry {
            id: "test-id".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            user_id: "test-user".to_string(),
            session_id: None,
            action: AuditAction::Create,
            entity_type: "Document".to_string(),
            entity_id: "DOC-001".to_string(),
            old_value: None,
            new_value: None,
            details: None,
            ip_address: None,
            signature: None,
            checksum: "test-checksum".to_string(),
            previous_hash: None,
        };
        
        let command = CreateAuditEntryCommand::new(entry, log_path.clone());
        let result = command.execute().unwrap();
        
        assert!(result.success);
        assert_eq!(result.affected_entries.len(), 1);
        assert!(log_path.exists());
    }

    #[test]
    fn test_command_invoker() {
        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        
        let invoker = AuditCommandInvoker::new(10);
        
        let entry = AuditEntry {
            id: "test-id".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            user_id: "test-user".to_string(),
            session_id: None,
            action: AuditAction::Create,
            entity_type: "Document".to_string(),
            entity_id: "DOC-001".to_string(),
            old_value: None,
            new_value: None,
            details: None,
            ip_address: None,
            signature: None,
            checksum: "test-checksum".to_string(),
            previous_hash: None,
        };
        
        let command = Box::new(CreateAuditEntryCommand::new(entry, log_path));
        let result = invoker.execute_command(command).unwrap();
        
        assert!(result.success);
        
        let history = invoker.get_history_summary().unwrap();
        assert_eq!(history.len(), 1);
        
        // Test undo
        let undo_result = invoker.undo_last_command().unwrap();
        assert!(undo_result.success);
    }

    #[test]
    fn test_batch_command() {
        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        
        let mut commands: Vec<Box<dyn AuditCommand>> = Vec::new();
        
        for i in 0..3 {
            let entry = AuditEntry {
                id: format!("test-id-{}", i),
                timestamp: "2024-01-01T00:00:00Z".to_string(),
                user_id: "test-user".to_string(),
                session_id: None,
                action: AuditAction::Create,
                entity_type: "Document".to_string(),
                entity_id: format!("DOC-{:03}", i),
                old_value: None,
                new_value: None,
                details: None,
                ip_address: None,
                signature: None,
                checksum: "test-checksum".to_string(),
                previous_hash: None,
            };
            
            commands.push(Box::new(CreateAuditEntryCommand::new(entry, log_path.clone())));
        }
        
        let batch_command = BatchAuditCommand::new(commands, "Test batch operation".to_string());
        let result = batch_command.execute().unwrap();
        
        assert!(result.success);
        assert_eq!(result.affected_entries.len(), 3);
    }
}
