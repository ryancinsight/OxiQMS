//! Unified Routing Logic
//! 
//! Extracts common routing patterns and creates interface-agnostic navigation
//! commands following SOLID, DRY, and Command Pattern principles.

use crate::prelude::*;
use crate::interfaces::{InterfaceType, InterfaceContext, CommandResult};
use crate::interfaces::unified_context::UnifiedInterfaceContext;
use crate::services::unified_service_manager::ServiceManagerInterface;
use crate::json_utils::JsonValue;
use std::sync::Arc;
use std::collections::HashMap;

/// Unified Router
/// 
/// Central routing coordinator that translates interface-agnostic commands
/// to interface-specific implementations using Command Pattern.
pub struct UnifiedRouter {
    /// Unified interface context
    context: Arc<UnifiedInterfaceContext>,
    
    /// Service manager for business operations
    service_manager: Arc<dyn ServiceManagerInterface>,
    
    /// Command registry mapping command names to handlers
    command_registry: HashMap<String, Arc<dyn UnifiedCommand>>,
    
    /// Route mappings for each interface type
    route_mappings: HashMap<InterfaceType, HashMap<String, String>>,
    
    /// Command aliases for backward compatibility
    command_aliases: HashMap<String, String>,
}

/// Unified Command trait
/// 
/// Defines interface-agnostic command execution with interface-specific adaptation.
/// Implements Command Pattern for consistent command handling.
pub trait UnifiedCommand: Send + Sync {
    /// Execute command with unified context
    fn execute(&self, context: &UnifiedCommandContext) -> QmsResult<UnifiedCommandResult>;
    
    /// Check if command requires authentication
    fn requires_authentication(&self) -> bool;
    
    /// Get command description
    fn description(&self) -> &str;
    
    /// Get command usage information
    fn usage(&self) -> &str;
    
    /// Validate command arguments
    fn validate_args(&self, args: &[String]) -> QmsResult<()>;
    
    /// Get command category for organization
    fn category(&self) -> CommandCategory;
}

/// Unified command context
#[derive(Debug, Clone)]
pub struct UnifiedCommandContext {
    /// Interface type executing the command
    pub interface_type: InterfaceType,
    
    /// Command name
    pub command: String,
    
    /// Command arguments
    pub args: Vec<String>,
    
    /// User session if authenticated
    pub user_session: Option<crate::modules::user_manager::UserSession>,
    
    /// Current project path
    pub project_path: Option<std::path::PathBuf>,
    
    /// Interface-specific context data
    pub interface_data: HashMap<String, String>,
    
    /// Request metadata
    pub metadata: CommandMetadata,
}

/// Unified command result
#[derive(Debug, Clone)]
pub struct UnifiedCommandResult {
    /// Whether command executed successfully
    pub success: bool,
    
    /// Result message
    pub message: String,
    
    /// Structured result data
    pub data: Option<JsonValue>,
    
    /// Next navigation action
    pub next_action: Option<NavigationAction>,
    
    /// Interface-specific result formatting
    pub interface_results: HashMap<InterfaceType, InterfaceCommandResult>,
}

/// Command metadata
#[derive(Debug, Clone)]
pub struct CommandMetadata {
    /// Command execution timestamp
    pub timestamp: u64,
    
    /// Request ID for tracking
    pub request_id: String,
    
    /// Client information
    pub client_info: Option<String>,
    
    /// Performance metrics
    pub metrics: PerformanceMetrics,
}

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Command execution start time
    pub start_time: std::time::Instant,
    
    /// Command execution duration (set after completion)
    pub duration: Option<std::time::Duration>,
    
    /// Memory usage before command
    pub memory_before: Option<usize>,
    
    /// Memory usage after command
    pub memory_after: Option<usize>,
}

/// Navigation action
#[derive(Debug, Clone)]
pub struct NavigationAction {
    /// Action type
    pub action_type: NavigationActionType,
    
    /// Target route or command
    pub target: String,
    
    /// Action parameters
    pub parameters: HashMap<String, String>,
    
    /// Interface-specific navigation data
    pub interface_data: HashMap<InterfaceType, String>,
}

/// Navigation action type
#[derive(Debug, Clone, PartialEq)]
pub enum NavigationActionType {
    /// Navigate to a specific route
    Navigate,
    
    /// Redirect to different location
    Redirect,
    
    /// Execute another command
    ExecuteCommand,
    
    /// Show modal or dialog
    ShowModal,
    
    /// Refresh current view
    Refresh,
    
    /// Go back in navigation history
    GoBack,
    
    /// Exit application
    Exit,
}

/// Command category for organization
#[derive(Debug, Clone, PartialEq)]
pub enum CommandCategory {
    /// Authentication commands
    Authentication,
    
    /// Project management commands
    Project,
    
    /// Document management commands
    Document,
    
    /// Risk management commands
    Risk,
    
    /// User management commands
    User,
    
    /// System commands
    System,
    
    /// Navigation commands
    Navigation,
    
    /// Configuration commands
    Configuration,
}

/// Interface-specific command result
#[derive(Debug, Clone)]
pub struct InterfaceCommandResult {
    /// Formatted output for interface
    pub formatted_output: String,
    
    /// Interface-specific status code
    pub status_code: i32,
    
    /// Additional interface data
    pub additional_data: HashMap<String, String>,
}

impl UnifiedRouter {
    /// Create new unified router
    pub fn new(
        context: Arc<UnifiedInterfaceContext>,
        service_manager: Arc<dyn ServiceManagerInterface>,
    ) -> Self {
        let mut router = Self {
            context,
            service_manager,
            command_registry: HashMap::new(),
            route_mappings: HashMap::new(),
            command_aliases: HashMap::new(),
        };
        
        // Initialize default commands and mappings
        router.initialize_default_commands();
        router.initialize_route_mappings();
        router.initialize_command_aliases();
        
        router
    }
    
    /// Route command through unified system
    pub fn route_command(&self, interface_context: &InterfaceContext, command: &str, args: &[String]) -> QmsResult<CommandResult> {
        // Resolve command aliases
        let resolved_command = self.resolve_command_alias(command);
        
        // Create unified command context
        let unified_context = self.create_unified_context(interface_context, &resolved_command, args)?;
        
        // Find command handler
        let command_handler = self.command_registry.get(&resolved_command)
            .ok_or_else(|| QmsError::not_found(&format!("Command not found: {}", resolved_command)))?;
        
        // Check authentication requirements
        if command_handler.requires_authentication() && !self.context.is_authenticated() {
            return Ok(CommandResult::error("Authentication required".to_string()));
        }
        
        // Validate command arguments
        command_handler.validate_args(args)?;
        
        // Execute command
        let start_time = std::time::Instant::now();
        let unified_result = command_handler.execute(&unified_context)?;
        let duration = start_time.elapsed();
        
        // Convert to interface-specific result
        self.convert_to_interface_result(unified_result, interface_context.interface_type.clone(), duration)
    }
    
    /// Get available commands for interface
    pub fn get_available_commands(&self, interface_type: InterfaceType, authenticated: bool) -> Vec<String> {
        self.command_registry.iter()
            .filter(|(_, handler)| !handler.requires_authentication() || authenticated)
            .map(|(name, _)| name.clone())
            .collect()
    }
    
    /// Get command help
    pub fn get_command_help(&self, command: &str) -> Option<String> {
        let resolved_command = self.resolve_command_alias(command);
        self.command_registry.get(&resolved_command)
            .map(|handler| format!("{}\n\nUsage: {}", handler.description(), handler.usage()))
    }
    
    /// Register new command
    pub fn register_command(&mut self, name: String, handler: Arc<dyn UnifiedCommand>) {
        self.command_registry.insert(name, handler);
    }
    
    /// Add command alias
    pub fn add_command_alias(&mut self, alias: String, target: String) {
        self.command_aliases.insert(alias, target);
    }
    
    // Private helper methods
    
    /// Initialize default commands
    fn initialize_default_commands(&mut self) {
        // Authentication commands
        self.register_command("login".to_string(), Arc::new(LoginCommand::new(self.context.clone())));
        self.register_command("logout".to_string(), Arc::new(LogoutCommand::new(self.context.clone())));

        // Navigation commands
        self.register_command("navigate".to_string(), Arc::new(NavigateCommand::new(self.context.clone())));
        self.register_command("back".to_string(), Arc::new(BackCommand::new(self.context.clone())));
        self.register_command("home".to_string(), Arc::new(HomeCommand::new(self.context.clone())));

        // Project commands
        self.register_command("project".to_string(), Arc::new(ProjectCommand::new(self.service_manager.clone())));
        self.register_command("init".to_string(), Arc::new(InitCommand::new(self.service_manager.clone())));

        // Document commands
        self.register_command("doc".to_string(), Arc::new(DocumentCommand::new(self.service_manager.clone())));

        // Risk commands
        self.register_command("risk".to_string(), Arc::new(RiskCommand::new(self.service_manager.clone())));

        // User commands
        self.register_command("user".to_string(), Arc::new(UserCommand::new(self.service_manager.clone())));

        // System commands
        self.register_command("help".to_string(), Arc::new(HelpCommand::new()));
        self.register_command("version".to_string(), Arc::new(VersionCommand::new()));
        self.register_command("status".to_string(), Arc::new(StatusCommand::new(self.context.clone())));
    }
    
    /// Initialize route mappings for each interface
    fn initialize_route_mappings(&mut self) {
        // CLI route mappings
        let mut cli_mappings = HashMap::new();
        cli_mappings.insert("login".to_string(), "auth/login".to_string());
        cli_mappings.insert("logout".to_string(), "auth/logout".to_string());
        cli_mappings.insert("doc".to_string(), "documents".to_string());
        cli_mappings.insert("risk".to_string(), "risks".to_string());
        cli_mappings.insert("user".to_string(), "users".to_string());
        cli_mappings.insert("project".to_string(), "projects".to_string());
        self.route_mappings.insert(InterfaceType::CLI, cli_mappings);
        
        // Web route mappings
        let mut web_mappings = HashMap::new();
        web_mappings.insert("login".to_string(), "/api/auth/login".to_string());
        web_mappings.insert("logout".to_string(), "/api/auth/logout".to_string());
        web_mappings.insert("doc".to_string(), "/api/documents".to_string());
        web_mappings.insert("risk".to_string(), "/api/risks".to_string());
        web_mappings.insert("user".to_string(), "/api/users".to_string());
        web_mappings.insert("project".to_string(), "/api/projects".to_string());
        web_mappings.insert("navigate".to_string(), "/navigate".to_string());
        self.route_mappings.insert(InterfaceType::Web, web_mappings);
        
        // TUI route mappings
        let mut tui_mappings = HashMap::new();
        tui_mappings.insert("login".to_string(), "screens/login".to_string());
        tui_mappings.insert("logout".to_string(), "screens/logout".to_string());
        tui_mappings.insert("doc".to_string(), "screens/documents".to_string());
        tui_mappings.insert("risk".to_string(), "screens/risks".to_string());
        tui_mappings.insert("user".to_string(), "screens/users".to_string());
        tui_mappings.insert("project".to_string(), "screens/projects".to_string());
        tui_mappings.insert("navigate".to_string(), "screens/navigate".to_string());
        self.route_mappings.insert(InterfaceType::TUI, tui_mappings);
    }
    
    /// Initialize command aliases for backward compatibility
    fn initialize_command_aliases(&mut self) {
        // Common aliases
        self.add_command_alias("l".to_string(), "login".to_string());
        self.add_command_alias("q".to_string(), "logout".to_string());
        self.add_command_alias("h".to_string(), "help".to_string());
        self.add_command_alias("v".to_string(), "version".to_string());
        self.add_command_alias("s".to_string(), "status".to_string());
        
        // Document aliases
        self.add_command_alias("document".to_string(), "doc".to_string());
        self.add_command_alias("documents".to_string(), "doc".to_string());
        
        // Risk aliases
        self.add_command_alias("risks".to_string(), "risk".to_string());
        
        // User aliases
        self.add_command_alias("users".to_string(), "user".to_string());
        
        // Project aliases
        self.add_command_alias("projects".to_string(), "project".to_string());
        self.add_command_alias("proj".to_string(), "project".to_string());
        
        // Navigation aliases
        self.add_command_alias("nav".to_string(), "navigate".to_string());
        self.add_command_alias("go".to_string(), "navigate".to_string());
        self.add_command_alias("cd".to_string(), "navigate".to_string());
    }
    
    /// Resolve command alias to actual command
    fn resolve_command_alias(&self, command: &str) -> String {
        self.command_aliases.get(command)
            .cloned()
            .unwrap_or_else(|| command.to_string())
    }
    
    /// Create unified command context
    fn create_unified_context(&self, interface_context: &InterfaceContext, command: &str, args: &[String]) -> QmsResult<UnifiedCommandContext> {
        Ok(UnifiedCommandContext {
            interface_type: interface_context.interface_type.clone(),
            command: command.to_string(),
            args: args.to_vec(),
            user_session: self.context.get_current_session(),
            project_path: self.context.get_active_project_path(),
            interface_data: HashMap::new(),
            metadata: CommandMetadata {
                timestamp: crate::utils::current_timestamp(),
                request_id: format!("req_{}", crate::utils::current_timestamp()),
                client_info: Some(format!("{:?}", interface_context.interface_type)),
                metrics: PerformanceMetrics {
                    start_time: std::time::Instant::now(),
                    duration: None,
                    memory_before: None,
                    memory_after: None,
                },
            },
        })
    }
    
    /// Convert unified result to interface-specific result
    fn convert_to_interface_result(&self, mut unified_result: UnifiedCommandResult, interface_type: InterfaceType, duration: std::time::Duration) -> QmsResult<CommandResult> {
        // Get interface-specific result if available
        let interface_result = unified_result.interface_results.get(&interface_type);
        
        let formatted_message = if let Some(result) = interface_result {
            result.formatted_output.clone()
        } else {
            unified_result.message.clone()
        };
        
        // Create command result
        let mut command_result = if unified_result.success {
            CommandResult::success(formatted_message)
        } else {
            CommandResult::error(formatted_message)
        };
        
        // Add structured data if available
        if let Some(data) = unified_result.data {
            command_result.data = Some(data);
        }
        
        // Add next action if available
        if let Some(nav_action) = unified_result.next_action {
            command_result.next_action = Some(nav_action.target);
        }
        
        // Add performance metrics
        command_result.metadata.insert("execution_time_ms".to_string(), duration.as_millis().to_string());
        
        Ok(command_result)
    }
}

/// Login Command Implementation
///
/// Unified login command that works across all interfaces.
pub struct LoginCommand {
    context: Arc<UnifiedInterfaceContext>,
}

impl LoginCommand {
    pub fn new(context: Arc<UnifiedInterfaceContext>) -> Self {
        Self { context }
    }
}

impl UnifiedCommand for LoginCommand {
    fn execute(&self, context: &UnifiedCommandContext) -> QmsResult<UnifiedCommandResult> {
        // Login logic would be handled by the unified auth flow
        // For now, return a placeholder result

        let mut interface_results = HashMap::new();

        // CLI-specific formatting
        interface_results.insert(InterfaceType::CLI, InterfaceCommandResult {
            formatted_output: "Login successful! Use 'help' to see available commands.".to_string(),
            status_code: 0,
            additional_data: HashMap::new(),
        });

        // Web-specific formatting
        interface_results.insert(InterfaceType::Web, InterfaceCommandResult {
            formatted_output: r#"{"success": true, "message": "Login successful", "redirect": "/dashboard"}"#.to_string(),
            status_code: 200,
            additional_data: HashMap::new(),
        });

        // TUI-specific formatting
        interface_results.insert(InterfaceType::TUI, InterfaceCommandResult {
            formatted_output: "✅ Login successful! Welcome to QMS TUI.".to_string(),
            status_code: 0,
            additional_data: HashMap::new(),
        });

        Ok(UnifiedCommandResult {
            success: true,
            message: "Login successful".to_string(),
            data: None,
            next_action: Some(NavigationAction {
                action_type: NavigationActionType::Navigate,
                target: "/dashboard".to_string(),
                parameters: HashMap::new(),
                interface_data: HashMap::new(),
            }),
            interface_results,
        })
    }

    fn requires_authentication(&self) -> bool {
        false
    }

    fn description(&self) -> &str {
        "Authenticate user and start session"
    }

    fn usage(&self) -> &str {
        "login [username] [password]"
    }

    fn validate_args(&self, _args: &[String]) -> QmsResult<()> {
        // Login validation would be more comprehensive in real implementation
        Ok(())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Authentication
    }
}

/// Navigate Command Implementation
///
/// Unified navigation command for moving between different sections.
pub struct NavigateCommand {
    context: Arc<UnifiedInterfaceContext>,
}

impl NavigateCommand {
    pub fn new(context: Arc<UnifiedInterfaceContext>) -> Self {
        Self { context }
    }
}

impl UnifiedCommand for NavigateCommand {
    fn execute(&self, context: &UnifiedCommandContext) -> QmsResult<UnifiedCommandResult> {
        let target = context.args.get(0)
            .ok_or_else(|| QmsError::validation_error("Navigation target required"))?;

        // Update navigation in unified context
        let mut parameters = HashMap::new();
        for (i, arg) in context.args.iter().skip(1).enumerate() {
            parameters.insert(format!("param_{}", i), arg.clone());
        }

        self.context.navigate_to(context.interface_type.clone(), target, parameters)?;

        let mut interface_results = HashMap::new();

        // CLI-specific formatting
        interface_results.insert(InterfaceType::CLI, InterfaceCommandResult {
            formatted_output: format!("Navigated to: {}", target),
            status_code: 0,
            additional_data: HashMap::new(),
        });

        // Web-specific formatting
        interface_results.insert(InterfaceType::Web, InterfaceCommandResult {
            formatted_output: format!(r#"{{"success": true, "route": "{}"}}"#, target),
            status_code: 200,
            additional_data: HashMap::new(),
        });

        // TUI-specific formatting
        interface_results.insert(InterfaceType::TUI, InterfaceCommandResult {
            formatted_output: format!("📍 Now at: {}", target),
            status_code: 0,
            additional_data: HashMap::new(),
        });

        Ok(UnifiedCommandResult {
            success: true,
            message: format!("Navigated to {}", target),
            data: None,
            next_action: Some(NavigationAction {
                action_type: NavigationActionType::Navigate,
                target: target.clone(),
                parameters: HashMap::new(),
                interface_data: HashMap::new(),
            }),
            interface_results,
        })
    }

    fn requires_authentication(&self) -> bool {
        true
    }

    fn description(&self) -> &str {
        "Navigate to a different section or page"
    }

    fn usage(&self) -> &str {
        "navigate <target> [parameters...]"
    }

    fn validate_args(&self, args: &[String]) -> QmsResult<()> {
        if args.is_empty() {
            return Err(QmsError::validation_error("Navigation target required"));
        }
        Ok(())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Navigation
    }
}

/// Help Command Implementation
///
/// Unified help command that provides interface-appropriate help information.
pub struct HelpCommand;

impl HelpCommand {
    pub fn new() -> Self {
        Self
    }
}

impl UnifiedCommand for HelpCommand {
    fn execute(&self, context: &UnifiedCommandContext) -> QmsResult<UnifiedCommandResult> {
        let help_content = match context.interface_type {
            InterfaceType::CLI => self.generate_cli_help(),
            InterfaceType::Web => self.generate_web_help(),
            InterfaceType::TUI => self.generate_tui_help(),
        };

        let mut interface_results = HashMap::new();
        interface_results.insert(context.interface_type.clone(), InterfaceCommandResult {
            formatted_output: help_content.clone(),
            status_code: 0,
            additional_data: HashMap::new(),
        });

        Ok(UnifiedCommandResult {
            success: true,
            message: "Help information displayed".to_string(),
            data: None,
            next_action: None,
            interface_results,
        })
    }

    fn requires_authentication(&self) -> bool {
        false
    }

    fn description(&self) -> &str {
        "Display help information"
    }

    fn usage(&self) -> &str {
        "help [command]"
    }

    fn validate_args(&self, _args: &[String]) -> QmsResult<()> {
        Ok(())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::System
    }
}

impl HelpCommand {
    fn generate_cli_help(&self) -> String {
        r#"QMS CLI Help
=============

Available Commands:
  login          - Authenticate user
  logout         - End user session
  project        - Project management commands
  doc            - Document management commands
  risk           - Risk management commands
  user           - User management commands
  navigate       - Navigate to different sections
  help           - Show this help
  version        - Show version information
  status         - Show system status

Use 'help <command>' for detailed information about a specific command.
"#.to_string()
    }

    fn generate_web_help(&self) -> String {
        r#"{"help": {
  "title": "QMS Web API Help",
  "endpoints": [
    {"path": "/api/auth/login", "method": "POST", "description": "Authenticate user"},
    {"path": "/api/auth/logout", "method": "POST", "description": "End user session"},
    {"path": "/api/projects", "method": "GET", "description": "List projects"},
    {"path": "/api/documents", "method": "GET", "description": "List documents"},
    {"path": "/api/risks", "method": "GET", "description": "List risks"},
    {"path": "/api/users", "method": "GET", "description": "List users"}
  ]
}}"#.to_string()
    }

    fn generate_tui_help(&self) -> String {
        r#"╔══════════════════════════════════════╗
║              QMS TUI Help            ║
╠══════════════════════════════════════╣
║                                      ║
║  Navigation:                         ║
║    ↑/↓    - Move up/down             ║
║    Enter  - Select item              ║
║    Esc    - Go back                  ║
║    Tab    - Switch panels            ║
║                                      ║
║  Commands:                           ║
║    F1     - Help                     ║
║    F2     - Projects                 ║
║    F3     - Documents                ║
║    F4     - Risks                    ║
║    F5     - Users                    ║
║    F10    - Logout                   ║
║                                      ║
║  Press any key to continue...        ║
╚══════════════════════════════════════╝"#.to_string()
    }
}

/// Status Command Implementation
///
/// Unified status command that shows system and session information.
pub struct StatusCommand {
    context: Arc<UnifiedInterfaceContext>,
}

impl StatusCommand {
    pub fn new(context: Arc<UnifiedInterfaceContext>) -> Self {
        Self { context }
    }
}

impl UnifiedCommand for StatusCommand {
    fn execute(&self, context: &UnifiedCommandContext) -> QmsResult<UnifiedCommandResult> {
        let auth_state = self.context.get_authentication_state();
        let active_project = self.context.get_active_project_path();

        let status_info = match context.interface_type {
            InterfaceType::CLI => self.generate_cli_status(&auth_state, &active_project),
            InterfaceType::Web => self.generate_web_status(&auth_state, &active_project),
            InterfaceType::TUI => self.generate_tui_status(&auth_state, &active_project),
        };

        let mut interface_results = HashMap::new();
        interface_results.insert(context.interface_type.clone(), InterfaceCommandResult {
            formatted_output: status_info.clone(),
            status_code: 0,
            additional_data: HashMap::new(),
        });

        Ok(UnifiedCommandResult {
            success: true,
            message: "Status information displayed".to_string(),
            data: None,
            next_action: None,
            interface_results,
        })
    }

    fn requires_authentication(&self) -> bool {
        false
    }

    fn description(&self) -> &str {
        "Display system and session status"
    }

    fn usage(&self) -> &str {
        "status"
    }

    fn validate_args(&self, _args: &[String]) -> QmsResult<()> {
        Ok(())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::System
    }
}

impl StatusCommand {
    fn generate_cli_status(&self, auth_state: &crate::interfaces::unified_context::AuthenticationState, active_project: &Option<std::path::PathBuf>) -> String {
        let mut status = String::new();
        status.push_str("QMS Status\n");
        status.push_str("==========\n\n");

        // Authentication status
        if let Some(ref session) = auth_state.current_session {
            status.push_str(&format!("User: {} ({:?})\n", session.username, session.session_type));
            status.push_str(&format!("Session ID: {}\n", session.session_id));
        } else {
            status.push_str("User: Not authenticated\n");
        }

        // Project status
        if let Some(ref project_path) = active_project {
            status.push_str(&format!("Active Project: {}\n", project_path.display()));
        } else {
            status.push_str("Active Project: None\n");
        }

        // System info
        status.push_str(&format!("Version: {}\n", env!("CARGO_PKG_VERSION")));
        status.push_str(&format!("Interface: CLI\n"));

        status
    }

    fn generate_web_status(&self, auth_state: &crate::interfaces::unified_context::AuthenticationState, active_project: &Option<std::path::PathBuf>) -> String {
        let authenticated = auth_state.current_session.is_some();
        let username = auth_state.current_session.as_ref().map(|s| s.username.clone()).unwrap_or_else(|| "anonymous".to_string());
        let project_path = active_project.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "none".to_string());

        format!(r#"{{
  "status": "ok",
  "authenticated": {},
  "user": "{}",
  "activeProject": "{}",
  "version": "{}",
  "interface": "web"
}}"#, authenticated, username, project_path, env!("CARGO_PKG_VERSION"))
    }

    fn generate_tui_status(&self, auth_state: &crate::interfaces::unified_context::AuthenticationState, active_project: &Option<std::path::PathBuf>) -> String {
        let mut status = String::new();
        status.push_str("╔══════════════════════════════════════╗\n");
        status.push_str("║              QMS Status              ║\n");
        status.push_str("╠══════════════════════════════════════╣\n");

        if let Some(ref session) = auth_state.current_session {
            status.push_str(&format!("║ User: {:<28} ║\n",
                if session.username.len() > 28 {
                    format!("{}...", &session.username[..25])
                } else {
                    session.username.clone()
                }
            ));
        } else {
            status.push_str("║ User: Not authenticated              ║\n");
        }

        if let Some(ref project_path) = active_project {
            let path_str = project_path.to_string_lossy();
            status.push_str(&format!("║ Project: {:<25} ║\n",
                if path_str.len() > 25 {
                    format!("...{}", &path_str[path_str.len()-22..])
                } else {
                    path_str.to_string()
                }
            ));
        } else {
            status.push_str("║ Project: None                        ║\n");
        }

        status.push_str(&format!("║ Version: {:<27} ║\n", env!("CARGO_PKG_VERSION")));
        status.push_str("║ Interface: TUI                       ║\n");
        status.push_str("╚══════════════════════════════════════╝");

        status
    }
}

/// Logout Command Implementation
pub struct LogoutCommand {
    context: Arc<UnifiedInterfaceContext>,
}

impl LogoutCommand {
    pub fn new(context: Arc<UnifiedInterfaceContext>) -> Self {
        Self { context }
    }
}

impl UnifiedCommand for LogoutCommand {
    fn execute(&self, context: &UnifiedCommandContext) -> QmsResult<UnifiedCommandResult> {
        // Logout through unified context
        self.context.logout(context.interface_type.clone())?;

        let mut interface_results = HashMap::new();

        // CLI-specific formatting
        interface_results.insert(InterfaceType::CLI, InterfaceCommandResult {
            formatted_output: "Logged out successfully. Goodbye!".to_string(),
            status_code: 0,
            additional_data: HashMap::new(),
        });

        // Web-specific formatting
        interface_results.insert(InterfaceType::Web, InterfaceCommandResult {
            formatted_output: r#"{"success": true, "message": "Logged out successfully", "redirect": "/login"}"#.to_string(),
            status_code: 200,
            additional_data: HashMap::new(),
        });

        // TUI-specific formatting
        interface_results.insert(InterfaceType::TUI, InterfaceCommandResult {
            formatted_output: "👋 Logged out successfully!".to_string(),
            status_code: 0,
            additional_data: HashMap::new(),
        });

        Ok(UnifiedCommandResult {
            success: true,
            message: "Logged out successfully".to_string(),
            data: None,
            next_action: Some(NavigationAction {
                action_type: NavigationActionType::Navigate,
                target: "/login".to_string(),
                parameters: HashMap::new(),
                interface_data: HashMap::new(),
            }),
            interface_results,
        })
    }

    fn requires_authentication(&self) -> bool {
        true
    }

    fn description(&self) -> &str {
        "End user session and logout"
    }

    fn usage(&self) -> &str {
        "logout"
    }

    fn validate_args(&self, _args: &[String]) -> QmsResult<()> {
        Ok(())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Authentication
    }
}

/// Back Command Implementation
pub struct BackCommand {
    context: Arc<UnifiedInterfaceContext>,
}

impl BackCommand {
    pub fn new(context: Arc<UnifiedInterfaceContext>) -> Self {
        Self { context }
    }
}

impl UnifiedCommand for BackCommand {
    fn execute(&self, context: &UnifiedCommandContext) -> QmsResult<UnifiedCommandResult> {
        // Get navigation history and go back
        let history = self.context.get_navigation_history(context.interface_type.clone());

        let target = if let Some(last_entry) = history.last() {
            last_entry.route.clone()
        } else {
            "/".to_string()
        };

        // Navigate back
        self.context.navigate_to(context.interface_type.clone(), &target, HashMap::new())?;

        let mut interface_results = HashMap::new();
        interface_results.insert(context.interface_type.clone(), InterfaceCommandResult {
            formatted_output: format!("Navigated back to: {}", target),
            status_code: 0,
            additional_data: HashMap::new(),
        });

        Ok(UnifiedCommandResult {
            success: true,
            message: format!("Navigated back to {}", target),
            data: None,
            next_action: Some(NavigationAction {
                action_type: NavigationActionType::GoBack,
                target,
                parameters: HashMap::new(),
                interface_data: HashMap::new(),
            }),
            interface_results,
        })
    }

    fn requires_authentication(&self) -> bool {
        true
    }

    fn description(&self) -> &str {
        "Go back to previous location"
    }

    fn usage(&self) -> &str {
        "back"
    }

    fn validate_args(&self, _args: &[String]) -> QmsResult<()> {
        Ok(())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Navigation
    }
}

/// Home Command Implementation
pub struct HomeCommand {
    context: Arc<UnifiedInterfaceContext>,
}

impl HomeCommand {
    pub fn new(context: Arc<UnifiedInterfaceContext>) -> Self {
        Self { context }
    }
}

impl UnifiedCommand for HomeCommand {
    fn execute(&self, context: &UnifiedCommandContext) -> QmsResult<UnifiedCommandResult> {
        let home_route = match context.interface_type {
            InterfaceType::CLI => "/",
            InterfaceType::Web => "/dashboard",
            InterfaceType::TUI => "/main_menu",
        };

        // Navigate to home
        self.context.navigate_to(context.interface_type.clone(), home_route, HashMap::new())?;

        let mut interface_results = HashMap::new();
        interface_results.insert(context.interface_type.clone(), InterfaceCommandResult {
            formatted_output: format!("Navigated to home: {}", home_route),
            status_code: 0,
            additional_data: HashMap::new(),
        });

        Ok(UnifiedCommandResult {
            success: true,
            message: format!("Navigated to home"),
            data: None,
            next_action: Some(NavigationAction {
                action_type: NavigationActionType::Navigate,
                target: home_route.to_string(),
                parameters: HashMap::new(),
                interface_data: HashMap::new(),
            }),
            interface_results,
        })
    }

    fn requires_authentication(&self) -> bool {
        true
    }

    fn description(&self) -> &str {
        "Navigate to home/dashboard"
    }

    fn usage(&self) -> &str {
        "home"
    }

    fn validate_args(&self, _args: &[String]) -> QmsResult<()> {
        Ok(())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Navigation
    }
}

/// Project Command Implementation
pub struct ProjectCommand {
    service_manager: Arc<dyn ServiceManagerInterface>,
}

impl ProjectCommand {
    pub fn new(service_manager: Arc<dyn ServiceManagerInterface>) -> Self {
        Self { service_manager }
    }
}

impl UnifiedCommand for ProjectCommand {
    fn execute(&self, context: &UnifiedCommandContext) -> QmsResult<UnifiedCommandResult> {
        let subcommand = context.args.get(0).unwrap_or(&"list".to_string()).clone();

        match subcommand.as_str() {
            "list" => {
                // List available projects
                let project_service = self.service_manager.project_service();
                let projects = if let Some(ref session) = context.user_session {
                    project_service.list_projects(&session.username, None)?
                } else {
                    Vec::new()
                };

                let mut interface_results = HashMap::new();
                let formatted_output = if projects.is_empty() {
                    "No projects found".to_string()
                } else {
                    format!("Found {} projects:\n{}",
                        projects.len(),
                        projects.iter().enumerate()
                            .map(|(i, p)| format!("  {}. {} ({})", i + 1, p.name, p.path.display()))
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                };

                interface_results.insert(context.interface_type.clone(), InterfaceCommandResult {
                    formatted_output,
                    status_code: 0,
                    additional_data: HashMap::new(),
                });

                Ok(UnifiedCommandResult {
                    success: true,
                    message: "Project list retrieved".to_string(),
                    data: None,
                    next_action: None,
                    interface_results,
                })
            }
            _ => {
                Err(QmsError::validation_error(&format!("Unknown project subcommand: {}", subcommand)))
            }
        }
    }

    fn requires_authentication(&self) -> bool {
        true
    }

    fn description(&self) -> &str {
        "Project management commands"
    }

    fn usage(&self) -> &str {
        "project [list|create|select]"
    }

    fn validate_args(&self, _args: &[String]) -> QmsResult<()> {
        Ok(())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Project
    }
}

/// Init Command Implementation
pub struct InitCommand {
    service_manager: Arc<dyn ServiceManagerInterface>,
}

impl InitCommand {
    pub fn new(service_manager: Arc<dyn ServiceManagerInterface>) -> Self {
        Self { service_manager }
    }
}

impl UnifiedCommand for InitCommand {
    fn execute(&self, context: &UnifiedCommandContext) -> QmsResult<UnifiedCommandResult> {
        let project_name = context.args.get(0).unwrap_or(&"New QMS Project".to_string()).clone();
        let project_path = context.project_path.clone().unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        // Create new project
        let project_service = self.service_manager.project_service();
        let created_by = context.user_session.as_ref().map(|s| s.username.as_str()).unwrap_or("system");
        let project_summary = project_service.create_project(&project_name, None, Some(project_path.to_string_lossy().as_ref()), created_by)?;

        let mut interface_results = HashMap::new();
        let formatted_output = format!("✅ Project '{}' initialized at {}", project_summary.name, project_path.display());

        interface_results.insert(context.interface_type.clone(), InterfaceCommandResult {
            formatted_output,
            status_code: 0,
            additional_data: HashMap::new(),
        });

        Ok(UnifiedCommandResult {
            success: true,
            message: "Project initialized successfully".to_string(),
            data: None,
            next_action: None,
            interface_results,
        })
    }

    fn requires_authentication(&self) -> bool {
        true
    }

    fn description(&self) -> &str {
        "Initialize a new QMS project"
    }

    fn usage(&self) -> &str {
        "init [project_name]"
    }

    fn validate_args(&self, _args: &[String]) -> QmsResult<()> {
        Ok(())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Project
    }
}

/// Document Command Implementation
pub struct DocumentCommand {
    service_manager: Arc<dyn ServiceManagerInterface>,
}

impl DocumentCommand {
    pub fn new(service_manager: Arc<dyn ServiceManagerInterface>) -> Self {
        Self { service_manager }
    }
}

impl UnifiedCommand for DocumentCommand {
    fn execute(&self, context: &UnifiedCommandContext) -> QmsResult<UnifiedCommandResult> {
        let subcommand = context.args.get(0).unwrap_or(&"list".to_string()).clone();

        match subcommand.as_str() {
            "list" => {
                let mut interface_results = HashMap::new();
                let formatted_output = "Document management - list functionality would be implemented here".to_string();

                interface_results.insert(context.interface_type.clone(), InterfaceCommandResult {
                    formatted_output,
                    status_code: 0,
                    additional_data: HashMap::new(),
                });

                Ok(UnifiedCommandResult {
                    success: true,
                    message: "Document command executed".to_string(),
                    data: None,
                    next_action: None,
                    interface_results,
                })
            }
            _ => {
                Err(QmsError::validation_error(&format!("Unknown document subcommand: {}", subcommand)))
            }
        }
    }

    fn requires_authentication(&self) -> bool {
        true
    }

    fn description(&self) -> &str {
        "Document management commands"
    }

    fn usage(&self) -> &str {
        "doc [list|create|edit|delete]"
    }

    fn validate_args(&self, _args: &[String]) -> QmsResult<()> {
        Ok(())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Document
    }
}

/// Risk Command Implementation
pub struct RiskCommand {
    service_manager: Arc<dyn ServiceManagerInterface>,
}

impl RiskCommand {
    pub fn new(service_manager: Arc<dyn ServiceManagerInterface>) -> Self {
        Self { service_manager }
    }
}

impl UnifiedCommand for RiskCommand {
    fn execute(&self, context: &UnifiedCommandContext) -> QmsResult<UnifiedCommandResult> {
        let subcommand = context.args.get(0).unwrap_or(&"list".to_string()).clone();

        match subcommand.as_str() {
            "list" => {
                let mut interface_results = HashMap::new();
                let formatted_output = "Risk management - list functionality would be implemented here".to_string();

                interface_results.insert(context.interface_type.clone(), InterfaceCommandResult {
                    formatted_output,
                    status_code: 0,
                    additional_data: HashMap::new(),
                });

                Ok(UnifiedCommandResult {
                    success: true,
                    message: "Risk command executed".to_string(),
                    data: None,
                    next_action: None,
                    interface_results,
                })
            }
            _ => {
                Err(QmsError::validation_error(&format!("Unknown risk subcommand: {}", subcommand)))
            }
        }
    }

    fn requires_authentication(&self) -> bool {
        true
    }

    fn description(&self) -> &str {
        "Risk management commands"
    }

    fn usage(&self) -> &str {
        "risk [list|create|assess|mitigate]"
    }

    fn validate_args(&self, _args: &[String]) -> QmsResult<()> {
        Ok(())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::Risk
    }
}

/// User Command Implementation
pub struct UserCommand {
    service_manager: Arc<dyn ServiceManagerInterface>,
}

impl UserCommand {
    pub fn new(service_manager: Arc<dyn ServiceManagerInterface>) -> Self {
        Self { service_manager }
    }
}

impl UnifiedCommand for UserCommand {
    fn execute(&self, context: &UnifiedCommandContext) -> QmsResult<UnifiedCommandResult> {
        let subcommand = context.args.get(0).unwrap_or(&"list".to_string()).clone();

        match subcommand.as_str() {
            "list" => {
                let mut interface_results = HashMap::new();
                let formatted_output = "User management - list functionality would be implemented here".to_string();

                interface_results.insert(context.interface_type.clone(), InterfaceCommandResult {
                    formatted_output,
                    status_code: 0,
                    additional_data: HashMap::new(),
                });

                Ok(UnifiedCommandResult {
                    success: true,
                    message: "User command executed".to_string(),
                    data: None,
                    next_action: None,
                    interface_results,
                })
            }
            _ => {
                Err(QmsError::validation_error(&format!("Unknown user subcommand: {}", subcommand)))
            }
        }
    }

    fn requires_authentication(&self) -> bool {
        true
    }

    fn description(&self) -> &str {
        "User management commands"
    }

    fn usage(&self) -> &str {
        "user [list|create|edit|delete]"
    }

    fn validate_args(&self, _args: &[String]) -> QmsResult<()> {
        Ok(())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::User
    }
}

/// Version Command Implementation
pub struct VersionCommand;

impl VersionCommand {
    pub fn new() -> Self {
        Self
    }
}

impl UnifiedCommand for VersionCommand {
    fn execute(&self, context: &UnifiedCommandContext) -> QmsResult<UnifiedCommandResult> {
        let version = env!("CARGO_PKG_VERSION");
        let mut interface_results = HashMap::new();

        let formatted_output = match context.interface_type {
            InterfaceType::CLI => format!("QMS CLI v{}", version),
            InterfaceType::Web => format!(r#"{{"version": "{}"}}"#, version),
            InterfaceType::TUI => format!("QMS TUI v{}", version),
        };

        interface_results.insert(context.interface_type.clone(), InterfaceCommandResult {
            formatted_output,
            status_code: 0,
            additional_data: HashMap::new(),
        });

        Ok(UnifiedCommandResult {
            success: true,
            message: format!("QMS v{}", version),
            data: None,
            next_action: None,
            interface_results,
        })
    }

    fn requires_authentication(&self) -> bool {
        false
    }

    fn description(&self) -> &str {
        "Display version information"
    }

    fn usage(&self) -> &str {
        "version"
    }

    fn validate_args(&self, _args: &[String]) -> QmsResult<()> {
        Ok(())
    }

    fn category(&self) -> CommandCategory {
        CommandCategory::System
    }
}
