//! Unified Routing System for QMS Interfaces
//! 
//! This module provides a common routing abstraction that can be shared across
//! CLI, web, and TUI interfaces, eliminating code duplication and providing
//! consistent command handling.

use crate::prelude::*;
use crate::interfaces::{InterfaceContext, CommandResult};
use std::collections::HashMap;

/// Unified Router trait - abstraction for routing commands across all interfaces
/// Implements Interface Segregation Principle with focused routing concerns
pub trait UnifiedRouter: Send + Sync {
    /// Route a command to the appropriate handler
    fn route_command(
        &self,
        context: &InterfaceContext,
        command: &str,
        args: &[String],
    ) -> QmsResult<CommandResult>;

    /// Check if a command requires authentication
    fn requires_authentication(&self, command: &str) -> bool;

    /// Get list of available commands for this interface
    fn available_commands(&self) -> Vec<&str>;

    /// Get help text for a specific command
    fn get_command_help(&self, command: &str) -> Option<&str>;

    /// Validate command arguments before execution
    fn validate_command_args(&self, command: &str, args: &[String]) -> QmsResult<()>;
}

/// Command Handler trait for individual command implementations
/// Follows Single Responsibility Principle - each handler manages one command
pub trait CommandHandler: Send + Sync {
    /// Execute the command with given context and arguments
    fn execute(
        &self,
        context: &InterfaceContext,
        args: &[String],
    ) -> QmsResult<CommandResult>;

    /// Get command name
    fn command_name(&self) -> &'static str;

    /// Get help text
    fn help_text(&self) -> &'static str;

    /// Check if command requires authentication
    fn requires_auth(&self) -> bool {
        true // Default to requiring authentication
    }

    /// Validate arguments
    fn validate_args(&self, args: &[String]) -> QmsResult<()> {
        // Default implementation - can be overridden
        Ok(())
    }
}

/// Base Router implementation with common functionality
/// Template Method Pattern for shared routing logic
pub struct BaseRouter {
    handlers: HashMap<String, Box<dyn CommandHandler>>,
    auth_required_commands: Vec<String>,
}

impl BaseRouter {
    /// Create new base router
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            auth_required_commands: Vec::new(),
        }
    }

    /// Register a command handler
    pub fn register_handler(&mut self, handler: Box<dyn CommandHandler>) {
        let command_name = handler.command_name().to_string();
        if handler.requires_auth() {
            self.auth_required_commands.push(command_name.clone());
        }
        self.handlers.insert(command_name, handler);
    }

    /// Register multiple handlers
    pub fn register_handlers(&mut self, handlers: Vec<Box<dyn CommandHandler>>) {
        for handler in handlers {
            self.register_handler(handler);
        }
    }

    /// Get handler for command
    fn get_handler(&self, command: &str) -> Option<&Box<dyn CommandHandler>> {
        self.handlers.get(command)
    }
}

impl UnifiedRouter for BaseRouter {
    fn route_command(
        &self,
        context: &InterfaceContext,
        command: &str,
        args: &[String],
    ) -> QmsResult<CommandResult> {
        // Fast path for performance-critical commands
        if command == "version" && args.is_empty() {
            return Ok(CommandResult::success("qms v1.0.0 - Medical Device Quality Management System".to_string()));
        }

        // Find handler
        let handler = self.get_handler(command)
            .ok_or_else(|| QmsError::validation_error(&format!("Unknown command: {}", command)))?;

        // Validate arguments
        handler.validate_args(args)?;

        // Execute command
        handler.execute(context, args)
    }

    fn requires_authentication(&self, command: &str) -> bool {
        self.auth_required_commands.contains(&command.to_string())
    }

    fn available_commands(&self) -> Vec<&str> {
        self.handlers.keys().map(|s| s.as_str()).collect()
    }

    fn get_command_help(&self, command: &str) -> Option<&str> {
        self.handlers.get(command).map(|h| h.help_text())
    }

    fn validate_command_args(&self, command: &str, args: &[String]) -> QmsResult<()> {
        if let Some(handler) = self.handlers.get(command) {
            handler.validate_args(args)
        } else {
            Err(QmsError::validation_error(&format!("Unknown command: {}", command)))
        }
    }
}

/// CLI-specific router implementation
pub struct CliRouter {
    base: BaseRouter,
}

impl CliRouter {
    pub fn new() -> Self {
        let mut base = BaseRouter::new();
        
        // Register CLI-specific handlers using the adapter system
        base.register_handlers(vec![
            Box::new(crate::interfaces::adapters::cli_adapter::InitCommandHandler::new()),
            Box::new(crate::interfaces::adapters::cli_adapter::DocCommandHandler::new()),
            Box::new(crate::interfaces::adapters::cli_adapter::RiskCommandHandler::new()),
            Box::new(crate::interfaces::adapters::cli_adapter::ReqCommandHandler::new()),
            Box::new(crate::interfaces::adapters::cli_adapter::TraceCommandHandler::new()),
            Box::new(crate::interfaces::adapters::cli_adapter::TestCommandHandler::new()),
            Box::new(crate::interfaces::adapters::cli_adapter::AuditCommandHandler::new()),
            Box::new(crate::interfaces::adapters::cli_adapter::UserCommandHandler::new()),
            Box::new(crate::interfaces::adapters::cli_adapter::ReportCommandHandler::new()),
            Box::new(crate::interfaces::adapters::cli_adapter::ServeCommandHandler::new()),
            Box::new(crate::interfaces::adapters::cli_adapter::HelpCommandHandler::new()),
            Box::new(crate::interfaces::adapters::cli_adapter::VersionCommandHandler::new()),
        ]);

        Self { base }
    }
}

impl UnifiedRouter for CliRouter {
    fn route_command(
        &self,
        context: &InterfaceContext,
        command: &str,
        args: &[String],
    ) -> QmsResult<CommandResult> {
        self.base.route_command(context, command, args)
    }

    fn requires_authentication(&self, command: &str) -> bool {
        // CLI-specific authentication rules
        match command {
            "help" | "init" | "login" => false,
            _ => self.base.requires_authentication(command),
        }
    }

    fn available_commands(&self) -> Vec<&str> {
        self.base.available_commands()
    }

    fn get_command_help(&self, command: &str) -> Option<&str> {
        self.base.get_command_help(command)
    }

    fn validate_command_args(&self, command: &str, args: &[String]) -> QmsResult<()> {
        self.base.validate_command_args(command, args)
    }
}

/// Web-specific router implementation
pub struct WebRouter {
    base: BaseRouter,
}

impl WebRouter {
    pub fn new() -> Self {
        let mut base = BaseRouter::new();
        
        // Register web-specific handlers (API endpoints)
        // For now, use simplified placeholder handlers
        // TODO: Implement proper web API handlers

        Self { base }
    }
}

impl UnifiedRouter for WebRouter {
    fn route_command(
        &self,
        context: &InterfaceContext,
        command: &str,
        args: &[String],
    ) -> QmsResult<CommandResult> {
        self.base.route_command(context, command, args)
    }

    fn requires_authentication(&self, command: &str) -> bool {
        // Web-specific authentication rules
        match command {
            "api/auth/login" | "api/auth/startup-state" | "api/health" => false,
            _ => self.base.requires_authentication(command),
        }
    }

    fn available_commands(&self) -> Vec<&str> {
        self.base.available_commands()
    }

    fn get_command_help(&self, command: &str) -> Option<&str> {
        self.base.get_command_help(command)
    }

    fn validate_command_args(&self, command: &str, args: &[String]) -> QmsResult<()> {
        self.base.validate_command_args(command, args)
    }
}

/// TUI-specific router implementation
pub struct TuiRouter {
    base: BaseRouter,
}

impl TuiRouter {
    pub fn new() -> Self {
        let mut base = BaseRouter::new();
        
        // Register TUI-specific handlers (menu actions)
        // For now, use simplified placeholder handlers
        // TODO: Implement proper TUI handlers

        Self { base }
    }
}

impl UnifiedRouter for TuiRouter {
    fn route_command(
        &self,
        context: &InterfaceContext,
        command: &str,
        args: &[String],
    ) -> QmsResult<CommandResult> {
        self.base.route_command(context, command, args)
    }

    fn requires_authentication(&self, command: &str) -> bool {
        // TUI-specific authentication rules
        match command {
            "login" | "navigate_to_login" | "exit" => false,
            _ => self.base.requires_authentication(command),
        }
    }

    fn available_commands(&self) -> Vec<&str> {
        self.base.available_commands()
    }

    fn get_command_help(&self, command: &str) -> Option<&str> {
        self.base.get_command_help(command)
    }

    fn validate_command_args(&self, command: &str, args: &[String]) -> QmsResult<()> {
        self.base.validate_command_args(command, args)
    }
}

/// Module for command handlers
/// Note: Actual command handlers are now implemented in the adapters module
/// to bridge existing CLI commands to the unified interface system
pub mod handlers {
    // This module is kept for backward compatibility
    // Actual handlers are in crate::interfaces::adapters::cli_adapter
}
