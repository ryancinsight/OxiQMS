/// SOLID Principles Enhancement: Command Handlers Module
/// 
/// This module organizes command handlers following SOLID principles:
/// - Single Responsibility: Each handler focuses on one specific command type
/// - Open/Closed: New handlers can be added without modifying existing code
/// - Interface Segregation: Small, focused interfaces for each command type
/// - Dependency Inversion: Handlers depend on abstractions, not concrete implementations

pub mod risk_handlers;
pub mod audit_handlers;
pub mod document_handlers;

// Re-export handlers for convenience
pub use risk_handlers::{
    RiskCreateHandler, RiskAssessHandler, RiskListHandler, RiskViewHandler
};

pub use audit_handlers::{
    AuditInitHandler, AuditStatsHandler, AuditSessionHandler, 
    AuditExportHandler, AuditBackupHandler
};

use crate::prelude::*;
use crate::commands::command_handler_trait::{CommandRouter, CommandContext};

/// Factory for creating command routers with all handlers registered
/// Implements Factory Pattern and Dependency Injection
pub struct CommandHandlerFactory;

impl CommandHandlerFactory {
    /// Create a fully configured command router for risk management
    pub fn create_risk_router(context: CommandContext) -> CommandRouter {
        let mut router = CommandRouter::new();
        
        // Register risk handlers
        router.register_handler(Box::new(RiskCreateHandler::new(context.clone())));
        router.register_handler(Box::new(RiskAssessHandler::new(context.clone())));
        router.register_handler(Box::new(RiskListHandler::new(context.clone())));
        router.register_handler(Box::new(RiskViewHandler::new(context)));
        
        router
    }
    
    /// Create a fully configured command router for audit management
    pub fn create_audit_router(context: CommandContext) -> CommandRouter {
        let mut router = CommandRouter::new();
        
        // Register audit handlers
        router.register_handler(Box::new(AuditInitHandler::new(context.clone())));
        router.register_handler(Box::new(AuditStatsHandler::new(context.clone())));
        router.register_handler(Box::new(AuditSessionHandler::new(context.clone())));
        router.register_handler(Box::new(AuditExportHandler::new(context.clone())));
        router.register_handler(Box::new(AuditBackupHandler::new(context)));
        
        router
    }
    
    /// Create a master router that handles all command types
    /// Demonstrates composition and delegation
    pub fn create_master_router(context: CommandContext) -> MasterCommandRouter {
        MasterCommandRouter::new(context)
    }
}

/// Master Command Router that delegates to specialized routers
/// Implements Composite Pattern and Command Pattern
pub struct MasterCommandRouter {
    risk_router: CommandRouter,
    audit_router: CommandRouter,
    context: CommandContext,
}

impl MasterCommandRouter {
    pub fn new(context: CommandContext) -> Self {
        Self {
            risk_router: CommandHandlerFactory::create_risk_router(context.clone()),
            audit_router: CommandHandlerFactory::create_audit_router(context.clone()),
            context,
        }
    }
    
    /// Route command to appropriate specialized router
    pub fn route_command(&self, module: &str, command: &str, args: &[String]) -> QmsResult<()> {
        match module {
            "risk" => self.risk_router.route_command(command, args),
            "audit" => self.audit_router.route_command(command, args),
            _ => Err(QmsError::validation_error(&format!("Unknown module: {}", module))),
        }
    }
    
    /// List all available commands for a module
    pub fn list_commands(&self, module: &str) -> QmsResult<Vec<&str>> {
        match module {
            "risk" => Ok(self.risk_router.list_commands()),
            "audit" => Ok(self.audit_router.list_commands()),
            _ => Err(QmsError::validation_error(&format!("Unknown module: {}", module))),
        }
    }
    
    /// Get help for a specific command in a module
    pub fn get_help(&self, module: &str, command: &str) -> QmsResult<&str> {
        match module {
            "risk" => self.risk_router.get_help(command)
                .ok_or_else(|| QmsError::validation_error(&format!("Unknown risk command: {}", command))),
            "audit" => self.audit_router.get_help(command)
                .ok_or_else(|| QmsError::validation_error(&format!("Unknown audit command: {}", command))),
            _ => Err(QmsError::validation_error(&format!("Unknown module: {}", module))),
        }
    }
    
    /// Get all available modules
    pub fn list_modules(&self) -> Vec<&str> {
        vec!["risk", "audit"]
    }
}

/// Command execution context builder for dependency injection
pub struct CommandContextBuilder {
    project_path: Option<std::path::PathBuf>,
    user_id: Option<String>,
    session_id: Option<String>,
}

impl CommandContextBuilder {
    pub fn new() -> Self {
        Self {
            project_path: None,
            user_id: None,
            session_id: None,
        }
    }
    
    pub fn with_project_path(mut self, path: std::path::PathBuf) -> Self {
        self.project_path = Some(path);
        self
    }
    
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
    
    pub fn build(self) -> QmsResult<CommandContext> {
        let project_path = self.project_path
            .or_else(|| crate::utils::get_current_project_path().ok())
            .unwrap_or_else(|| std::path::PathBuf::from("."));
            
        Ok(CommandContext {
            project_path,
            user_id: self.user_id,
            session_id: self.session_id,
        })
    }
}

impl Default for CommandContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_command_context_builder() {
        let context = CommandContextBuilder::new()
            .with_project_path(PathBuf::from("test"))
            .with_user_id("test_user".to_string())
            .with_session_id("test_session".to_string())
            .build()
            .unwrap();
            
        assert_eq!(context.project_path, PathBuf::from("test"));
        assert_eq!(context.user_id, Some("test_user".to_string()));
        assert_eq!(context.session_id, Some("test_session".to_string()));
    }
    
    #[test]
    fn test_master_router_creation() {
        let context = CommandContextBuilder::new()
            .with_project_path(PathBuf::from("test"))
            .build()
            .unwrap();
            
        let router = CommandHandlerFactory::create_master_router(context);
        let modules = router.list_modules();
        
        assert!(modules.contains(&"risk"));
        assert!(modules.contains(&"audit"));
    }
    
    #[test]
    fn test_specialized_routers() {
        let context = CommandContextBuilder::new()
            .with_project_path(PathBuf::from("test"))
            .build()
            .unwrap();
            
        let risk_router = CommandHandlerFactory::create_risk_router(context.clone());
        let audit_router = CommandHandlerFactory::create_audit_router(context);
        
        let risk_commands = risk_router.list_commands();
        let audit_commands = audit_router.list_commands();
        
        assert!(risk_commands.contains(&"create"));
        assert!(risk_commands.contains(&"assess"));
        assert!(risk_commands.contains(&"list"));
        assert!(risk_commands.contains(&"view"));
        
        assert!(audit_commands.contains(&"init"));
        assert!(audit_commands.contains(&"stats"));
        assert!(audit_commands.contains(&"session"));
        assert!(audit_commands.contains(&"export"));
        assert!(audit_commands.contains(&"backup"));
    }
}
