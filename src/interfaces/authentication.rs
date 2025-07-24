//! Unified Authentication Abstractions for QMS Interfaces
//! 
//! This module provides common authentication patterns that can be shared
//! across CLI, web, and TUI interfaces, ensuring consistent authentication
//! flows and eliminating code duplication.

use crate::prelude::*;
use crate::interfaces::{InterfaceContext, CommandResult};
use crate::modules::user_manager::{UserSession, FileBasedAuthService};
use std::sync::Arc;

/// Authentication Flow trait - abstraction for authentication processes
/// Implements Interface Segregation Principle with focused auth concerns
pub trait AuthenticationFlow: Send + Sync {
    /// Handle login flow for the specific interface
    fn handle_login(&self, context: &mut InterfaceContext) -> QmsResult<CommandResult>;

    /// Handle logout flow for the specific interface
    fn handle_logout(&self, context: &mut InterfaceContext) -> QmsResult<CommandResult>;

    /// Handle user registration flow
    fn handle_registration(&self, context: &mut InterfaceContext) -> QmsResult<CommandResult>;

    /// Check if user is authenticated
    fn is_authenticated(&self, context: &InterfaceContext) -> bool;

    /// Get current user session
    fn get_current_session<'a>(&self, context: &'a InterfaceContext) -> Option<&'a UserSession>;

    /// Validate session and refresh if needed
    fn validate_and_refresh_session(&self, context: &mut InterfaceContext) -> QmsResult<bool>;
}

/// Authentication credentials for login
#[derive(Debug, Clone)]
pub struct AuthCredentials {
    pub username: String,
    pub password: String,
    pub remember_me: bool,
    pub interface_type: crate::interfaces::InterfaceType,
}

impl AuthCredentials {
    /// Create new authentication credentials
    pub fn new(username: String, password: String, interface_type: crate::interfaces::InterfaceType) -> Self {
        Self {
            username,
            password,
            remember_me: false,
            interface_type,
        }
    }

    /// Set remember me option
    pub fn with_remember_me(mut self, remember_me: bool) -> Self {
        self.remember_me = remember_me;
        self
    }
}

/// Registration data for new users
#[derive(Debug, Clone)]
pub struct RegistrationData {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub roles: Vec<String>,
}

impl RegistrationData {
    /// Create new registration data
    pub fn new(username: String, password: String) -> Self {
        Self {
            username,
            password,
            email: None,
            full_name: None,
            roles: vec!["user".to_string()], // Default role
        }
    }

    /// Add email to registration data
    pub fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    /// Add full name to registration data
    pub fn with_full_name(mut self, full_name: String) -> Self {
        self.full_name = Some(full_name);
        self
    }

    /// Set roles for registration data
    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles;
        self
    }
}

/// Base Authentication Flow implementation with common functionality
/// Template Method Pattern for shared authentication logic
pub struct BaseAuthenticationFlow {
    auth_service: Arc<FileBasedAuthService>,
    user_interaction: Arc<dyn crate::interfaces::user_interaction::UserInteractionProvider>,
}

impl BaseAuthenticationFlow {
    /// Create new base authentication flow
    pub fn new(
        auth_service: Arc<FileBasedAuthService>,
        user_interaction: Arc<dyn crate::interfaces::user_interaction::UserInteractionProvider>,
    ) -> Self {
        Self {
            auth_service,
            user_interaction,
        }
    }

    /// Collect login credentials from user
    fn collect_login_credentials(&self, context: &InterfaceContext) -> QmsResult<AuthCredentials> {
        let username = self.user_interaction.prompt_input(
            "Username",
            crate::interfaces::user_interaction::InputType::Text,
        )?;

        let password = self.user_interaction.prompt_input(
            "Password",
            crate::interfaces::user_interaction::InputType::Password,
        )?;

        let remember_me = self.user_interaction.confirm_action("Remember me?")?;

        Ok(AuthCredentials::new(username, password, context.interface_type.clone())
            .with_remember_me(remember_me))
    }

    /// Collect registration data from user
    fn collect_registration_data(&self) -> QmsResult<RegistrationData> {
        let username = self.user_interaction.prompt_input(
            "Username",
            crate::interfaces::user_interaction::InputType::Text,
        )?;

        let password = self.user_interaction.prompt_input(
            "Password",
            crate::interfaces::user_interaction::InputType::Password,
        )?;

        let confirm_password = self.user_interaction.prompt_input(
            "Confirm Password",
            crate::interfaces::user_interaction::InputType::Password,
        )?;

        if password != confirm_password {
            return Err(QmsError::validation_error("Passwords do not match"));
        }

        let email = self.user_interaction.prompt_input(
            "Email (optional)",
            crate::interfaces::user_interaction::InputType::Email,
        ).ok().filter(|s| !s.is_empty());

        let full_name = self.user_interaction.prompt_input(
            "Full Name (optional)",
            crate::interfaces::user_interaction::InputType::Text,
        ).ok().filter(|s| !s.is_empty());

        let mut registration_data = RegistrationData::new(username, password);
        
        if let Some(email) = email {
            registration_data = registration_data.with_email(email);
        }
        
        if let Some(full_name) = full_name {
            registration_data = registration_data.with_full_name(full_name);
        }

        Ok(registration_data)
    }

    /// Perform authentication with credentials
    fn authenticate_user(&self, credentials: &AuthCredentials) -> QmsResult<UserSession> {
        let session_type = match credentials.interface_type {
            crate::interfaces::InterfaceType::CLI => crate::modules::user_manager::interfaces::SessionType::CLI,
            crate::interfaces::InterfaceType::Web => crate::modules::user_manager::interfaces::SessionType::Web,
            crate::interfaces::InterfaceType::TUI => crate::modules::user_manager::interfaces::SessionType::CLI, // TUI uses CLI session type
        };

        self.auth_service.login(
            &credentials.username,
            &credentials.password,
            session_type,
            None, // IP address - would be provided by the interface layer
            None, // User agent - would be provided by the interface layer
        )
    }

    /// Register new user
    fn register_user(&self, registration_data: &RegistrationData) -> QmsResult<()> {
        // Convert role names to Role objects (simplified - would need proper role lookup)
        let roles: Vec<crate::models::Role> = registration_data.roles.iter().map(|role_name| {
            crate::models::Role {
                name: role_name.clone(),
                permissions: Vec::new(), // Simplified - would need proper role definition
            }
        }).collect();

        self.auth_service.create_user(
            &registration_data.username,
            &registration_data.password,
            roles,
        )?;
        Ok(())
    }

    /// Update context with authenticated session
    fn update_context_with_session(&self, context: &mut InterfaceContext, session: UserSession) {
        context.user_session = Some(session);
    }

    /// Clear authentication from context
    fn clear_authentication_from_context(&self, context: &mut InterfaceContext) {
        context.user_session = None;
    }
}

impl AuthenticationFlow for BaseAuthenticationFlow {
    fn handle_login(&self, context: &mut InterfaceContext) -> QmsResult<CommandResult> {
        // Check if already authenticated
        if self.is_authenticated(context) {
            return Ok(CommandResult::success("Already authenticated".to_string()));
        }

        // Collect credentials
        let credentials = self.collect_login_credentials(context)?;

        // Authenticate user
        match self.authenticate_user(&credentials) {
            Ok(session) => {
                self.update_context_with_session(context, session.clone());
                
                self.user_interaction.display_success(
                    &format!("Successfully logged in as {}", session.username),
                    None,
                )?;

                Ok(CommandResult::success(format!("Logged in as {}", session.username)))
            }
            Err(e) => {
                self.user_interaction.display_error(&e, Some("Login"))?;
                Ok(CommandResult::error(format!("Login failed: {}", e)))
            }
        }
    }

    fn handle_logout(&self, context: &mut InterfaceContext) -> QmsResult<CommandResult> {
        // Check if authenticated
        if !self.is_authenticated(context) {
            return Ok(CommandResult::success("Not currently logged in".to_string()));
        }

        // Get current session
        let session_id = context.user_session.as_ref()
            .map(|s| s.session_id.clone())
            .unwrap_or_default();

        // Logout from auth service
        if let Err(e) = self.auth_service.logout(&session_id) {
            self.user_interaction.display_error(&e, Some("Logout"))?;
        }

        // Clear context
        self.clear_authentication_from_context(context);

        self.user_interaction.display_success("Successfully logged out", None)?;

        Ok(CommandResult::success("Logged out successfully".to_string()))
    }

    fn handle_registration(&self, context: &mut InterfaceContext) -> QmsResult<CommandResult> {
        // Collect registration data
        let registration_data = self.collect_registration_data()?;

        // Register user
        match self.register_user(&registration_data) {
            Ok(()) => {
                self.user_interaction.display_success(
                    &format!("Successfully registered user {}", registration_data.username),
                    Some("You can now log in with your credentials"),
                )?;

                Ok(CommandResult::success(format!("User {} registered successfully", registration_data.username)))
            }
            Err(e) => {
                self.user_interaction.display_error(&e, Some("Registration"))?;
                Ok(CommandResult::error(format!("Registration failed: {}", e)))
            }
        }
    }

    fn is_authenticated(&self, context: &InterfaceContext) -> bool {
        context.is_authenticated()
    }

    fn get_current_session<'a>(&self, context: &'a InterfaceContext) -> Option<&'a UserSession> {
        context.user_session.as_ref()
    }

    fn validate_and_refresh_session(&self, context: &mut InterfaceContext) -> QmsResult<bool> {
        if let Some(session) = &context.user_session {
            // Validate session with auth service
            match self.auth_service.validate_session(&session.session_id) {
                Ok(validated_session) => {
                    // Update context with refreshed session
                    context.user_session = Some(validated_session);
                    Ok(true)
                }
                Err(_) => {
                    // Session is invalid, clear it
                    self.clear_authentication_from_context(context);
                    Ok(false)
                }
            }
        } else {
            Ok(false)
        }
    }
}

/// CLI-specific authentication flow
pub struct CliAuthenticationFlow {
    base: BaseAuthenticationFlow,
}

impl CliAuthenticationFlow {
    pub fn new(
        auth_service: Arc<FileBasedAuthService>,
        user_interaction: Arc<dyn crate::interfaces::user_interaction::UserInteractionProvider>,
    ) -> Self {
        Self {
            base: BaseAuthenticationFlow::new(auth_service, user_interaction),
        }
    }
}

impl AuthenticationFlow for CliAuthenticationFlow {
    fn handle_login(&self, context: &mut InterfaceContext) -> QmsResult<CommandResult> {
        self.base.handle_login(context)
    }

    fn handle_logout(&self, context: &mut InterfaceContext) -> QmsResult<CommandResult> {
        self.base.handle_logout(context)
    }

    fn handle_registration(&self, context: &mut InterfaceContext) -> QmsResult<CommandResult> {
        self.base.handle_registration(context)
    }

    fn is_authenticated(&self, context: &InterfaceContext) -> bool {
        self.base.is_authenticated(context)
    }

    fn get_current_session<'a>(&self, context: &'a InterfaceContext) -> Option<&'a UserSession> {
        self.base.get_current_session(context)
    }

    fn validate_and_refresh_session(&self, context: &mut InterfaceContext) -> QmsResult<bool> {
        self.base.validate_and_refresh_session(context)
    }
}

/// Web-specific authentication flow
pub struct WebAuthenticationFlow {
    base: BaseAuthenticationFlow,
}

impl WebAuthenticationFlow {
    pub fn new(
        auth_service: Arc<FileBasedAuthService>,
        user_interaction: Arc<dyn crate::interfaces::user_interaction::UserInteractionProvider>,
    ) -> Self {
        Self {
            base: BaseAuthenticationFlow::new(auth_service, user_interaction),
        }
    }
}

impl AuthenticationFlow for WebAuthenticationFlow {
    fn handle_login(&self, context: &mut InterfaceContext) -> QmsResult<CommandResult> {
        // Web login is typically handled through form data, not interactive prompts
        // This would be customized for web-specific authentication
        self.base.handle_login(context)
    }

    fn handle_logout(&self, context: &mut InterfaceContext) -> QmsResult<CommandResult> {
        self.base.handle_logout(context)
    }

    fn handle_registration(&self, context: &mut InterfaceContext) -> QmsResult<CommandResult> {
        // Web registration is typically handled through form data
        self.base.handle_registration(context)
    }

    fn is_authenticated(&self, context: &InterfaceContext) -> bool {
        self.base.is_authenticated(context)
    }

    fn get_current_session<'a>(&self, context: &'a InterfaceContext) -> Option<&'a UserSession> {
        self.base.get_current_session(context)
    }

    fn validate_and_refresh_session(&self, context: &mut InterfaceContext) -> QmsResult<bool> {
        self.base.validate_and_refresh_session(context)
    }
}

/// TUI-specific authentication flow
pub struct TuiAuthenticationFlow {
    base: BaseAuthenticationFlow,
}

impl TuiAuthenticationFlow {
    pub fn new(
        auth_service: Arc<FileBasedAuthService>,
        user_interaction: Arc<dyn crate::interfaces::user_interaction::UserInteractionProvider>,
    ) -> Self {
        Self {
            base: BaseAuthenticationFlow::new(auth_service, user_interaction),
        }
    }
}

impl AuthenticationFlow for TuiAuthenticationFlow {
    fn handle_login(&self, context: &mut InterfaceContext) -> QmsResult<CommandResult> {
        // TUI login would show a login form in the terminal interface
        self.base.handle_login(context)
    }

    fn handle_logout(&self, context: &mut InterfaceContext) -> QmsResult<CommandResult> {
        self.base.handle_logout(context)
    }

    fn handle_registration(&self, context: &mut InterfaceContext) -> QmsResult<CommandResult> {
        // TUI registration would show a registration form in the terminal interface
        self.base.handle_registration(context)
    }

    fn is_authenticated(&self, context: &InterfaceContext) -> bool {
        self.base.is_authenticated(context)
    }

    fn get_current_session<'a>(&self, context: &'a InterfaceContext) -> Option<&'a UserSession> {
        self.base.get_current_session(context)
    }

    fn validate_and_refresh_session(&self, context: &mut InterfaceContext) -> QmsResult<bool> {
        self.base.validate_and_refresh_session(context)
    }
}
