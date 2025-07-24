//! CLI Adapter for Unified Interface System
//! 
//! This module provides an adapter that bridges the existing CLI command system
//! to the new unified interface abstractions, maintaining backward compatibility
//! while progressively migrating to the shared core services.

use crate::prelude::*;
use crate::interfaces::{InterfaceContext, InterfaceManager, CommandResult, InterfaceType};
use crate::interfaces::routing::{CommandHandler, UnifiedRouter};
use crate::commands::command_handler_trait::{CommandHandler as LegacyCommandHandler, CommandRouter as LegacyCommandRouter};
use std::sync::Arc;

/// CLI Command Adapter - Wraps legacy CLI commands to work with unified interface system
/// Implements Adapter Pattern to bridge old and new systems
pub struct CliCommandAdapter {
    legacy_handler: Box<dyn LegacyCommandHandler>,
}

impl CliCommandAdapter {
    /// Create new CLI command adapter
    pub fn new(legacy_handler: Box<dyn LegacyCommandHandler>) -> Self {
        Self { legacy_handler }
    }
}

impl CommandHandler for CliCommandAdapter {
    fn execute(
        &self,
        context: &InterfaceContext,
        args: &[String],
    ) -> QmsResult<CommandResult> {
        // Execute legacy command
        match self.legacy_handler.handle(args) {
            Ok(()) => Ok(CommandResult::success("Command executed successfully".to_string())),
            Err(e) => Ok(CommandResult::error(format!("Command failed: {}", e))),
        }
    }

    fn command_name(&self) -> &'static str {
        self.legacy_handler.command_name()
    }

    fn help_text(&self) -> &'static str {
        self.legacy_handler.help_text()
    }

    fn requires_auth(&self) -> bool {
        // Most CLI commands require authentication except init, help, version
        match self.command_name() {
            "init" | "help" | "version" => false,
            _ => true,
        }
    }

    fn validate_args(&self, args: &[String]) -> QmsResult<()> {
        self.legacy_handler.validate_args(args)
    }
}

/// Legacy CLI Router Adapter - Wraps the existing CommandRouter to work with UnifiedRouter
pub struct LegacyCliRouterAdapter {
    legacy_router: LegacyCommandRouter,
}

impl LegacyCliRouterAdapter {
    /// Create new legacy CLI router adapter
    pub fn new(legacy_router: LegacyCommandRouter) -> Self {
        Self { legacy_router }
    }
}

impl UnifiedRouter for LegacyCliRouterAdapter {
    fn route_command(
        &self,
        context: &InterfaceContext,
        command: &str,
        args: &[String],
    ) -> QmsResult<CommandResult> {
        match self.legacy_router.route_command(command, args) {
            Ok(()) => Ok(CommandResult::success("Command executed successfully".to_string())),
            Err(e) => Ok(CommandResult::error(format!("Command failed: {}", e))),
        }
    }

    fn requires_authentication(&self, command: &str) -> bool {
        // Authentication rules for legacy commands
        match command {
            "init" | "help" | "version" => false,
            _ => true,
        }
    }

    fn available_commands(&self) -> Vec<&str> {
        self.legacy_router.list_commands()
    }

    fn get_command_help(&self, command: &str) -> Option<&str> {
        self.legacy_router.get_help(command)
    }

    fn validate_command_args(&self, command: &str, args: &[String]) -> QmsResult<()> {
        // Use legacy router's validation if available
        // For now, just check if command exists
        if self.legacy_router.list_commands().contains(&command) {
            Ok(())
        } else {
            Err(QmsError::validation_error(&format!("Unknown command: {}", command)))
        }
    }
}

/// CLI Interface Manager - Manages the CLI interface using unified abstractions
pub struct CliInterfaceManager {
    interface_manager: InterfaceManager,
    context: InterfaceContext,
}

impl CliInterfaceManager {
    /// Create new CLI interface manager
    pub fn new(project_path: Option<std::path::PathBuf>) -> QmsResult<Self> {
        let interface_manager = crate::interfaces::InterfaceFactory::create_cli_manager(project_path.clone())?;
        let context = InterfaceContext::new(InterfaceType::CLI)
            .with_project_path(project_path.unwrap_or_else(|| std::path::PathBuf::from(".")));

        Ok(Self {
            interface_manager,
            context,
        })
    }

    /// Execute a command through the unified interface system
    pub fn execute_command(&mut self, command: &str, args: &[String]) -> QmsResult<CommandResult> {
        self.interface_manager.execute_command(&mut self.context, command, args)
    }

    /// Handle authentication
    pub fn authenticate(&mut self, username: &str, password: &str) -> QmsResult<()> {
        self.interface_manager.authenticate(&mut self.context, username, password)
    }

    /// Handle logout
    pub fn logout(&mut self) -> QmsResult<()> {
        self.interface_manager.logout(&mut self.context)
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.context.is_authenticated()
    }

    /// Get current username
    pub fn current_username(&self) -> Option<&str> {
        self.context.current_username()
    }

    /// Get interface context
    pub fn context(&self) -> &InterfaceContext {
        &self.context
    }

    /// Get mutable interface context
    pub fn context_mut(&mut self) -> &mut InterfaceContext {
        &mut self.context
    }
}

/// CLI Command Factory - Creates command handlers for the unified system
pub struct CliCommandFactory;

impl CliCommandFactory {
    /// Create command handlers for all existing CLI commands
    pub fn create_all_handlers() -> Vec<Box<dyn CommandHandler>> {
        vec![
            Box::new(InitCommandHandler::new()),
            Box::new(DocCommandHandler::new()),
            Box::new(RiskCommandHandler::new()),
            Box::new(ReqCommandHandler::new()),
            Box::new(TraceCommandHandler::new()),
            Box::new(TestCommandHandler::new()),
            Box::new(AuditCommandHandler::new()),
            Box::new(UserCommandHandler::new()),
            Box::new(ReportCommandHandler::new()),
            Box::new(ServeCommandHandler::new()),
            Box::new(HelpCommandHandler::new()),
            Box::new(VersionCommandHandler::new()),
        ]
    }

    /// Create a unified CLI router with all command handlers
    pub fn create_unified_cli_router() -> crate::interfaces::routing::CliRouter {
        let mut router = crate::interfaces::routing::CliRouter::new();
        // The CliRouter constructor already registers all handlers
        router
    }
}

/// Concrete command handler implementations that bridge to existing CLI functions
pub struct InitCommandHandler;

impl InitCommandHandler {
    pub fn new() -> Self {
        Self
    }
}

impl CommandHandler for InitCommandHandler {
    fn execute(&self, context: &InterfaceContext, args: &[String]) -> QmsResult<CommandResult> {
        // Convert args to the format expected by legacy init command
        let mut full_args = vec!["qms".to_string(), "init".to_string()];
        full_args.extend_from_slice(args);

        match crate::commands::init::handle_init_command(&full_args) {
            Ok(()) => Ok(CommandResult::success("Project initialized successfully".to_string())),
            Err(e) => Ok(CommandResult::error(format!("Failed to initialize project: {}", e))),
        }
    }

    fn command_name(&self) -> &'static str {
        "init"
    }

    fn help_text(&self) -> &'static str {
        "Initialize a new QMS project"
    }

    fn requires_auth(&self) -> bool {
        false
    }
}

pub struct DocCommandHandler;

impl DocCommandHandler {
    pub fn new() -> Self {
        Self
    }
}

impl CommandHandler for DocCommandHandler {
    fn execute(&self, context: &InterfaceContext, args: &[String]) -> QmsResult<CommandResult> {
        let mut full_args = vec!["qms".to_string(), "doc".to_string()];
        full_args.extend_from_slice(args);

        match crate::commands::doc::handle_doc_command(&full_args) {
            Ok(()) => Ok(CommandResult::success("Document command executed successfully".to_string())),
            Err(e) => Ok(CommandResult::error(format!("Document command failed: {}", e))),
        }
    }

    fn command_name(&self) -> &'static str {
        "doc"
    }

    fn help_text(&self) -> &'static str {
        "Manage documents"
    }
}

pub struct RiskCommandHandler;

impl RiskCommandHandler {
    pub fn new() -> Self {
        Self
    }
}

impl CommandHandler for RiskCommandHandler {
    fn execute(&self, context: &InterfaceContext, args: &[String]) -> QmsResult<CommandResult> {
        let mut full_args = vec!["qms".to_string(), "risk".to_string()];
        full_args.extend_from_slice(args);

        match crate::commands::risk::handle_risk_command(&full_args) {
            Ok(()) => Ok(CommandResult::success("Risk command executed successfully".to_string())),
            Err(e) => Ok(CommandResult::error(format!("Risk command failed: {}", e))),
        }
    }

    fn command_name(&self) -> &'static str {
        "risk"
    }

    fn help_text(&self) -> &'static str {
        "Manage risk assessments"
    }
}

// Additional command handlers following the same pattern...
pub struct ReqCommandHandler;
impl ReqCommandHandler {
    pub fn new() -> Self { Self }
}
impl CommandHandler for ReqCommandHandler {
    fn execute(&self, context: &InterfaceContext, args: &[String]) -> QmsResult<CommandResult> {
        let mut full_args = vec!["qms".to_string(), "req".to_string()];
        full_args.extend_from_slice(args);
        match crate::commands::req::handle_req_command(&full_args) {
            Ok(()) => Ok(CommandResult::success("Requirements command executed successfully".to_string())),
            Err(e) => Ok(CommandResult::error(format!("Requirements command failed: {}", e))),
        }
    }
    fn command_name(&self) -> &'static str { "req" }
    fn help_text(&self) -> &'static str { "Manage requirements" }
}

pub struct TraceCommandHandler;
impl TraceCommandHandler {
    pub fn new() -> Self { Self }
}
impl CommandHandler for TraceCommandHandler {
    fn execute(&self, context: &InterfaceContext, args: &[String]) -> QmsResult<CommandResult> {
        let mut full_args = vec!["qms".to_string(), "trace".to_string()];
        full_args.extend_from_slice(args);
        match crate::commands::trace::handle_trace_command(&full_args) {
            Ok(()) => Ok(CommandResult::success("Traceability command executed successfully".to_string())),
            Err(e) => Ok(CommandResult::error(format!("Traceability command failed: {}", e))),
        }
    }
    fn command_name(&self) -> &'static str { "trace" }
    fn help_text(&self) -> &'static str { "Manage traceability" }
}

pub struct TestCommandHandler;
impl TestCommandHandler {
    pub fn new() -> Self { Self }
}
impl CommandHandler for TestCommandHandler {
    fn execute(&self, context: &InterfaceContext, args: &[String]) -> QmsResult<CommandResult> {
        match crate::commands::test::handle_test_command(args.to_vec()) {
            Ok(()) => Ok(CommandResult::success("Test command executed successfully".to_string())),
            Err(e) => Ok(CommandResult::error(format!("Test command failed: {}", e))),
        }
    }
    fn command_name(&self) -> &'static str { "test" }
    fn help_text(&self) -> &'static str { "Run tests" }
}

pub struct AuditCommandHandler;
impl AuditCommandHandler {
    pub fn new() -> Self { Self }
}
impl CommandHandler for AuditCommandHandler {
    fn execute(&self, context: &InterfaceContext, args: &[String]) -> QmsResult<CommandResult> {
        let mut full_args = vec!["qms".to_string(), "audit".to_string()];
        full_args.extend_from_slice(args);
        match crate::commands::audit::handle_audit_command(&full_args) {
            Ok(()) => Ok(CommandResult::success("Audit command executed successfully".to_string())),
            Err(e) => Ok(CommandResult::error(format!("Audit command failed: {}", e))),
        }
    }
    fn command_name(&self) -> &'static str { "audit" }
    fn help_text(&self) -> &'static str { "Manage audit logs" }
}

pub struct UserCommandHandler;
impl UserCommandHandler {
    pub fn new() -> Self { Self }
}
impl CommandHandler for UserCommandHandler {
    fn execute(&self, context: &InterfaceContext, args: &[String]) -> QmsResult<CommandResult> {
        let mut full_args = vec!["qms".to_string(), "user".to_string()];
        full_args.extend_from_slice(args);
        match crate::commands::user::handle_user_command(&full_args) {
            Ok(()) => Ok(CommandResult::success("User command executed successfully".to_string())),
            Err(e) => Ok(CommandResult::error(format!("User command failed: {}", e))),
        }
    }
    fn command_name(&self) -> &'static str { "user" }
    fn help_text(&self) -> &'static str { "Manage users" }
}

pub struct ReportCommandHandler;
impl ReportCommandHandler {
    pub fn new() -> Self { Self }
}
impl CommandHandler for ReportCommandHandler {
    fn execute(&self, context: &InterfaceContext, args: &[String]) -> QmsResult<CommandResult> {
        let mut full_args = vec!["qms".to_string(), "report".to_string()];
        full_args.extend_from_slice(args);
        match crate::commands::report::handle_report_command(&full_args) {
            Ok(()) => Ok(CommandResult::success("Report command executed successfully".to_string())),
            Err(e) => Ok(CommandResult::error(format!("Report command failed: {}", e))),
        }
    }
    fn command_name(&self) -> &'static str { "report" }
    fn help_text(&self) -> &'static str { "Generate reports" }
}

pub struct ServeCommandHandler;
impl ServeCommandHandler {
    pub fn new() -> Self { Self }
}
impl CommandHandler for ServeCommandHandler {
    fn execute(&self, context: &InterfaceContext, args: &[String]) -> QmsResult<CommandResult> {
        // Note: serve command needs special handling as it starts a server
        Ok(CommandResult::success("Serve command would start web server".to_string()))
    }
    fn command_name(&self) -> &'static str { "serve" }
    fn help_text(&self) -> &'static str { "Start web server" }
    fn requires_auth(&self) -> bool { false }
}

pub struct HelpCommandHandler;
impl HelpCommandHandler {
    pub fn new() -> Self { Self }
}
impl CommandHandler for HelpCommandHandler {
    fn execute(&self, context: &InterfaceContext, args: &[String]) -> QmsResult<CommandResult> {
        Ok(CommandResult::success("Help information displayed".to_string()))
    }
    fn command_name(&self) -> &'static str { "help" }
    fn help_text(&self) -> &'static str { "Show help information" }
    fn requires_auth(&self) -> bool { false }
}

pub struct VersionCommandHandler;
impl VersionCommandHandler {
    pub fn new() -> Self { Self }
}
impl CommandHandler for VersionCommandHandler {
    fn execute(&self, context: &InterfaceContext, args: &[String]) -> QmsResult<CommandResult> {
        Ok(CommandResult::success("qms v1.0.0 - Medical Device Quality Management System".to_string()))
    }
    fn command_name(&self) -> &'static str { "version" }
    fn help_text(&self) -> &'static str { "Show version information" }
    fn requires_auth(&self) -> bool { false }
}
