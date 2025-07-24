//! Shared Interface Abstractions for QMS
//! 
//! This module provides common abstractions for routing, state management, and user interaction
//! that can be shared across CLI, web, and TUI interfaces using Dependency Inversion Principle.
//! 
//! ## Architecture Principles:
//! - **Single Responsibility**: Each interface component has one clear purpose
//! - **Open/Closed**: New interface types can be added without modifying existing code
//! - **Interface Segregation**: Small, focused interfaces for different concerns
//! - **Dependency Inversion**: Interfaces depend on abstractions, not concrete implementations
//! - **DRY**: Eliminate code duplication across interface implementations

use crate::prelude::*;
use crate::json_utils::JsonValue;
use std::collections::HashMap;
use std::sync::Arc;

pub mod routing;
pub mod state;
pub mod user_interaction;
pub mod authentication;
pub mod adapters;

#[cfg(test)]
mod integration_tests;

/// Common interface types supported by the QMS system
#[derive(Debug, Clone, PartialEq)]
pub enum InterfaceType {
    CLI,
    Web,
    TUI,
}

/// Interface Context - shared state and configuration across all interfaces
/// Implements Dependency Inversion Principle by providing abstractions
#[derive(Debug, Clone)]
pub struct InterfaceContext {
    pub interface_type: InterfaceType,
    pub project_path: Option<std::path::PathBuf>,
    pub user_session: Option<crate::modules::user_manager::UserSession>,
    pub configuration: HashMap<String, String>,
    pub audit_enabled: bool,
}

impl InterfaceContext {
    /// Create new interface context
    pub fn new(interface_type: InterfaceType) -> Self {
        Self {
            interface_type,
            project_path: None,
            user_session: None,
            configuration: HashMap::new(),
            audit_enabled: true,
        }
    }

    /// Set project path for context
    pub fn with_project_path(mut self, path: std::path::PathBuf) -> Self {
        self.project_path = Some(path);
        self
    }

    /// Set user session for context
    pub fn with_user_session(mut self, session: crate::modules::user_manager::UserSession) -> Self {
        self.user_session = Some(session);
        self
    }

    /// Add configuration value
    pub fn with_config(mut self, key: String, value: String) -> Self {
        self.configuration.insert(key, value);
        self
    }

    /// Disable audit logging for performance-critical operations
    pub fn with_audit_disabled(mut self) -> Self {
        self.audit_enabled = false;
        self
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.user_session.as_ref().map_or(false, |s| s.is_authenticated())
    }

    /// Get current username if authenticated
    pub fn current_username(&self) -> Option<&str> {
        self.user_session.as_ref().map(|s| s.username.as_str())
    }
}

/// Interface Manager - coordinates shared services across all interface types
/// Implements Single Responsibility Principle for interface coordination
pub struct InterfaceManager {
    auth_service: Arc<crate::modules::user_manager::FileBasedAuthService>,
    router: Arc<dyn routing::UnifiedRouter>,
    state_manager: Arc<dyn state::StateManager>,
    user_interaction: Arc<dyn user_interaction::UserInteractionProvider>,
}

impl InterfaceManager {
    /// Create new interface manager with dependency injection
    pub fn new(
        auth_service: Arc<crate::modules::user_manager::FileBasedAuthService>,
        router: Arc<dyn routing::UnifiedRouter>,
        state_manager: Arc<dyn state::StateManager>,
        user_interaction: Arc<dyn user_interaction::UserInteractionProvider>,
    ) -> Self {
        Self {
            auth_service,
            router,
            state_manager,
            user_interaction,
        }
    }

    /// Execute a command through the unified interface system
    pub fn execute_command(
        &self,
        context: &mut InterfaceContext,
        command: &str,
        args: &[String],
    ) -> QmsResult<CommandResult> {
        // Pre-execution: Validate authentication if required
        if self.router.requires_authentication(command) && !context.is_authenticated() {
            return Err(QmsError::Authentication("Authentication required".to_string()));
        }

        // Route and execute command
        let result = self.router.route_command(context, command, args)?;

        // Post-execution: Update state and audit log
        // Skip state updates for performance-critical read-only commands
        if !matches!(command, "version" | "help") {
            self.state_manager.update_state(context, &result)?;
        }
        
        if context.audit_enabled {
            self.audit_command_execution(context, command, args, &result)?;
        }

        Ok(result)
    }

    /// Handle authentication flow across all interfaces
    pub fn authenticate(
        &self,
        context: &mut InterfaceContext,
        username: &str,
        password: &str,
    ) -> QmsResult<()> {
        let session_type = match context.interface_type {
            InterfaceType::CLI => crate::modules::user_manager::interfaces::SessionType::CLI,
            InterfaceType::Web => crate::modules::user_manager::interfaces::SessionType::Web,
            InterfaceType::TUI => crate::modules::user_manager::interfaces::SessionType::CLI, // TUI uses CLI session type
        };

        let session = self.auth_service.login(
            username,
            password,
            session_type,
            None, // IP address - would be provided by the interface layer
            None, // User agent - would be provided by the interface layer
        )?;
        context.user_session = Some(session);

        // Update state with authenticated session
        self.state_manager.set_authenticated_session(context)?;

        Ok(())
    }

    /// Handle logout across all interfaces
    pub fn logout(&self, context: &mut InterfaceContext) -> QmsResult<()> {
        if let Some(session) = &context.user_session {
            self.auth_service.logout(&session.session_id)?;
            context.user_session = None;
            
            // Clear authenticated state
            self.state_manager.clear_authenticated_session(context)?;
        }
        
        Ok(())
    }

    /// Get authentication service
    pub fn auth_service(&self) -> Arc<crate::modules::user_manager::FileBasedAuthService> {
        self.auth_service.clone()
    }

    /// Get router
    pub fn router(&self) -> Arc<dyn routing::UnifiedRouter> {
        self.router.clone()
    }

    /// Get state manager
    pub fn state_manager(&self) -> Arc<dyn state::StateManager> {
        self.state_manager.clone()
    }

    /// Get user interaction provider
    pub fn user_interaction(&self) -> Arc<dyn user_interaction::UserInteractionProvider> {
        self.user_interaction.clone()
    }

    /// Audit command execution with optimized performance
    fn audit_command_execution(
        &self,
        context: &InterfaceContext,
        command: &str,
        args: &[String],
        result: &CommandResult,
    ) -> QmsResult<()> {
        // Skip audit logging for performance-critical commands like "version"
        if matches!(command, "version" | "help") {
            return Ok(());
        }

        let username = context.current_username().unwrap_or("anonymous");
        let interface_type = format!("{:?}", context.interface_type);
        let command_details = format!("{} {}", command, args.join(" "));

        // Use non-blocking audit logging to avoid I/O overhead
        if let Err(_) = crate::modules::audit_logger::audit_log_action(
            "COMMAND_EXECUTED",
            &interface_type,
            &format!("user:{} command:{} result:{}", username, command_details, result.success),
        ) {
            // Don't fail command execution if audit logging fails
            // This ensures system remains functional even if audit system has issues
        }

        Ok(())
    }
}

/// Command execution result
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub data: Option<JsonValue>,
    pub requires_user_input: bool,
    pub next_action: Option<String>,
}

impl CommandResult {
    /// Create successful result
    pub fn success(message: String) -> Self {
        Self {
            success: true,
            message,
            data: None,
            requires_user_input: false,
            next_action: None,
        }
    }

    /// Create successful result with data
    pub fn success_with_data(message: String, data: JsonValue) -> Self {
        Self {
            success: true,
            message,
            data: Some(data),
            requires_user_input: false,
            next_action: None,
        }
    }

    /// Create error result
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            message,
            data: None,
            requires_user_input: false,
            next_action: None,
        }
    }

    /// Create result requiring user input
    pub fn requires_input(message: String, next_action: String) -> Self {
        Self {
            success: true,
            message,
            data: None,
            requires_user_input: true,
            next_action: Some(next_action),
        }
    }
}

/// Interface Factory - creates appropriate interface managers using Factory Pattern
/// Implements Open/Closed Principle for adding new interface types
pub struct InterfaceFactory;

impl InterfaceFactory {
    /// Create interface manager for CLI
    pub fn create_cli_manager(project_path: Option<std::path::PathBuf>) -> QmsResult<InterfaceManager> {
        let auth_service = Self::create_auth_service(project_path.as_deref())?;
        let router = Arc::new(routing::CliRouter::new());
        let state_manager = Arc::new(state::FileStateManager::new(project_path.clone()));
        let user_interaction = Arc::new(user_interaction::CliUserInteraction::new());

        Ok(InterfaceManager::new(auth_service, router, state_manager, user_interaction))
    }

    /// Create interface manager for Web
    pub fn create_web_manager(project_path: Option<std::path::PathBuf>) -> QmsResult<InterfaceManager> {
        let auth_service = Self::create_auth_service(project_path.as_deref())?;
        let router = Arc::new(routing::WebRouter::new());
        let state_manager = Arc::new(state::SessionStateManager::new());
        let user_interaction = Arc::new(user_interaction::WebUserInteraction::new());

        Ok(InterfaceManager::new(auth_service, router, state_manager, user_interaction))
    }

    /// Create interface manager for TUI
    pub fn create_tui_manager(project_path: Option<std::path::PathBuf>) -> QmsResult<InterfaceManager> {
        let auth_service = Self::create_auth_service(project_path.as_deref())?;
        let router = Arc::new(routing::TuiRouter::new());
        let state_manager = Arc::new(state::FileStateManager::new(project_path.clone()));
        let user_interaction = Arc::new(user_interaction::TuiUserInteraction::new());

        Ok(InterfaceManager::new(auth_service, router, state_manager, user_interaction))
    }

    /// Create authentication service (shared across all interfaces)
    fn create_auth_service(project_path: Option<&std::path::Path>) -> QmsResult<Arc<crate::modules::user_manager::FileBasedAuthService>> {
        let service = match project_path {
            Some(path) => crate::modules::user_manager::FileBasedAuthService::from_project_path(path)?,
            None => crate::modules::user_manager::FileBasedAuthService::create_global()?,
        };
        Ok(Arc::new(service))
    }
}
