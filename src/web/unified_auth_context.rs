// Unified Authentication Context - Shared authentication context for CLI and Web
// Follows SOLID, CUPID, GRASP, ACID, KISS, DRY, SOC, and YAGNI principles

use crate::prelude::*;
use crate::modules::user_manager::{UserSession, FileBasedAuthService, SessionType};
use crate::models::{Role, Permission};
use crate::commands::cli_auth_helper::{get_cli_session, get_cli_auth_helper};
use crate::web::{HttpRequest, command_bridge::WebCommandContext};
use std::path::PathBuf;
use std::fs;

/// Unified authentication context that works for both CLI and Web
#[derive(Debug, Clone)]
pub struct UnifiedAuthContext {
    pub session: UserSession,
    pub project_path: PathBuf,
    pub context_type: AuthContextType,
}

/// Type of authentication context
#[derive(Debug, Clone, PartialEq)]
pub enum AuthContextType {
    /// CLI context - session loaded from CLI session file
    Cli,
    /// Web context - session validated from HTTP request
    Web,
    /// Test context - mock session for testing
    Test,
}

impl UnifiedAuthContext {
    /// Create CLI authentication context
    pub fn from_cli() -> QmsResult<Self> {
        let session = get_cli_session()?;

        // Use unified project detection logic (same as CLI auth helper)
        let project_path = Self::find_user_project_path(&session.username)?;

        Ok(Self {
            session,
            project_path,
            context_type: AuthContextType::Cli,
        })
    }
    
    /// Create Web authentication context from HTTP request with bidirectional session recognition
    pub fn from_web_request(request: &HttpRequest) -> QmsResult<Self> {
        // Use UnifiedSessionAdapter for bidirectional session recognition
        let current_dir = std::env::current_dir()
            .map_err(|e| QmsError::Authentication(format!("Failed to get current directory: {}", e)))?;

        let session_adapter = crate::web::UnifiedSessionAdapter::new(&current_dir)?;

        // Get active session (web or CLI) using bidirectional recognition
        let session = session_adapter.get_active_session_for_web(request)
            .ok_or_else(|| QmsError::Authentication("No valid session found (web or CLI)".to_string()))?;

        // Use unified project detection logic (same as CLI auth helper)
        let project_path = Self::find_user_project_path(&session.username)?;

        println!("🔐 UnifiedAuthContext: Using session {} for user {} (type: {:?})",
            session.session_id, session.username, session.session_type);

        Ok(Self {
            session,
            project_path,
            context_type: AuthContextType::Web,
        })
    }
    
    /// Create Web authentication context from WebCommandContext
    pub fn from_web_command_context(web_context: &WebCommandContext) -> Self {
        Self {
            session: web_context.session.clone(),
            project_path: web_context.project_path.clone(),
            context_type: AuthContextType::Web,
        }
    }
    
    /// Create test authentication context
    pub fn for_test(username: &str, project_path: PathBuf) -> Self {
        // Create a mock session for testing
        let session = UserSession {
            session_id: format!("test_session_{}", username),
            user_id: username.to_string(),
            username: username.to_string(),
            roles: vec![Role {
                name: "TestUser".to_string(),
                permissions: vec![Permission::ReadDocuments, Permission::WriteDocuments],
            }],
            permissions: vec!["read_documents".to_string(), "write_documents".to_string()],
            login_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            last_activity: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            expires_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() + 3600, // 1 hour from now
            ip_address: None,
            user_agent: None,
            csrf_token: "test_csrf_token".to_string(),
            is_active: true,
            session_type: SessionType::CLI,
            data: std::collections::HashMap::new(),
        };
        
        Self {
            session,
            project_path,
            context_type: AuthContextType::Test,
        }
    }
    
    /// Check if user has permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.session.permissions.contains(&permission.to_string())
    }
    
    /// Check if user has role
    pub fn has_role(&self, role_name: &str) -> bool {
        self.session.roles.iter().any(|role| role.name == role_name)
    }
    
    /// Get user ID
    pub fn user_id(&self) -> &str {
        &self.session.user_id
    }
    
    /// Get username
    pub fn username(&self) -> &str {
        &self.session.username
    }
    
    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session.session_id
    }
    
    /// Get project path
    pub fn project_path(&self) -> &PathBuf {
        &self.project_path
    }
    
    /// Check if session is still valid
    pub fn is_session_valid(&self) -> bool {
        if let Ok(auth_service) = FileBasedAuthService::create_global() {
            auth_service.validate_session(&self.session.session_id).is_ok()
        } else {
            false
        }
    }

    /// Refresh session activity
    pub fn refresh_session(&mut self) -> QmsResult<()> {
        // Update last activity time
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.session.last_activity = current_time;

        // Update session in storage
        let auth_service = FileBasedAuthService::create_global()?;
        auth_service.update_session_activity(&self.session.session_id)?;

        Ok(())
    }
    
    /// Convert to WebCommandContext for compatibility
    pub fn to_web_command_context(&self, request_id: String, client_info: crate::web::command_bridge::ClientInfo) -> WebCommandContext {
        WebCommandContext {
            session: self.session.clone(),
            project_path: self.project_path.clone(),
            request_id,
            client_info,
        }
    }
    
    /// Extract session ID from HTTP request
    fn extract_session_id_from_request(request: &HttpRequest) -> Option<String> {
        // Try cookie first
        if let Some(cookie_header) = request.get_header("cookie") {
            for cookie in cookie_header.split(';') {
                let cookie = cookie.trim();
                if let Some(session_id) = cookie.strip_prefix("session_id=") {
                    return Some(session_id.to_string());
                }
            }
        }
        
        // Try Authorization header
        if let Some(auth_header) = request.get_header("authorization") {
            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                return Some(token.to_string());
            }
        }
        
        None
    }
    
    /// Unified project detection logic (consolidates CLI and Web approaches)
    /// This is the single source of truth for project path resolution
    pub fn find_user_project_path(username: &str) -> QmsResult<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| QmsError::io_error("Cannot determine home directory"))?;

        let user_projects_dir = std::path::Path::new(&home)
            .join(".qms")
            .join(username)
            .join("projects");

        // Create directory if it doesn't exist
        fs::create_dir_all(&user_projects_dir)?;

        // Strategy 1: Check current working directory (for CLI compatibility)
        if let Ok(current_dir) = std::env::current_dir() {
            let mut path = current_dir.as_path();

            // Search up the directory tree
            loop {
                let project_file = path.join("project.json");
                if project_file.exists() {
                    // Verify this project belongs to the current user
                    if path.starts_with(&user_projects_dir) {
                        return Ok(path.to_path_buf());
                    }
                }

                match path.parent() {
                    Some(parent) => path = parent,
                    None => break,
                }
            }
        }

        // Strategy 2: Find the most recently created project
        let mut most_recent_path: Option<PathBuf> = None;
        let mut most_recent_time = 0u64;

        if let Ok(entries) = fs::read_dir(&user_projects_dir) {
            for entry in entries.flatten() {
                let project_path = entry.path();
                if project_path.is_dir() {
                    let project_file = project_path.join("project.json");
                    if project_file.exists() {
                        // Parse project.json to get created_at timestamp
                        if let Ok(content) = fs::read_to_string(&project_file) {
                            if let Some(created_at_str) = Self::extract_json_field(&content, "created_at") {
                                if let Ok(created_at) = created_at_str.parse::<u64>() {
                                    if created_at > most_recent_time {
                                        most_recent_time = created_at;
                                        most_recent_path = Some(project_path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Return the most recent project if found
        if let Some(path) = most_recent_path {
            return Ok(path);
        }

        // Strategy 3: Find any valid project (fallback)
        if let Ok(entries) = fs::read_dir(&user_projects_dir) {
            for entry in entries.flatten() {
                let project_path = entry.path();
                if project_path.is_dir() {
                    let project_file = project_path.join("project.json");
                    if project_file.exists() {
                        return Ok(project_path);
                    }
                }
            }
        }

        Err(QmsError::not_found("No QMS project found. Use 'qms init' to create a project."))
    }

    /// Simple JSON field extraction helper
    fn extract_json_field(json: &str, field: &str) -> Option<String> {
        let pattern = format!("\"{}\":", field);
        if let Some(start) = json.find(&pattern) {
            let after_colon = &json[start + pattern.len()..];
            if let Some(quote_start) = after_colon.find('"') {
                let value_start = quote_start + 1;
                if let Some(quote_end) = after_colon[value_start..].find('"') {
                    return Some(after_colon[value_start..value_start + quote_end].to_string());
                }
            }
        }
        None
    }
}

/// Authentication context manager - provides unified access to authentication context
pub struct AuthContextManager;

impl AuthContextManager {
    /// Get authentication context for CLI commands
    pub fn get_cli_context() -> QmsResult<UnifiedAuthContext> {
        UnifiedAuthContext::from_cli()
    }
    
    /// Get authentication context for web requests
    pub fn get_web_context(request: &HttpRequest) -> QmsResult<UnifiedAuthContext> {
        UnifiedAuthContext::from_web_request(request)
    }
    
    /// Validate that user has required permission for operation
    pub fn require_permission(context: &UnifiedAuthContext, permission: &str) -> QmsResult<()> {
        if !context.has_permission(permission) {
            return Err(QmsError::Authentication(format!(
                "User {} does not have required permission: {}",
                context.username(),
                permission
            )));
        }
        Ok(())
    }
    
    /// Validate that user has required role for operation
    pub fn require_role(context: &UnifiedAuthContext, role: &str) -> QmsResult<()> {
        if !context.has_role(role) {
            return Err(QmsError::Authentication(format!(
                "User {} does not have required role: {}",
                context.username(),
                role
            )));
        }
        Ok(())
    }
    
    /// Create execution context for commands
    pub fn create_command_execution_context(
        auth_context: &UnifiedAuthContext,
    ) -> crate::commands::command_execution_context::CommandExecutionContext {
        match auth_context.context_type {
            AuthContextType::Cli => {
                crate::commands::command_execution_context::CommandExecutionContext::cli()
            }
            AuthContextType::Web => {
                crate::commands::command_execution_context::CommandExecutionContext::web(
                    auth_context.session.clone(),
                    auth_context.project_path.clone(),
                )
            }
            AuthContextType::Test => {
                crate::commands::command_execution_context::CommandExecutionContext::test()
            }
        }
    }
}

/// Trait for commands that require authentication
pub trait AuthenticatedCommand {
    /// Execute command with authentication context
    fn execute_authenticated(
        &self,
        auth_context: &UnifiedAuthContext,
        args: &[String],
    ) -> QmsResult<()>;
    
    /// Get required permissions for this command
    fn required_permissions(&self) -> Vec<&'static str> {
        Vec::new()
    }
    
    /// Get required roles for this command
    fn required_roles(&self) -> Vec<&'static str> {
        Vec::new()
    }
}

/// Macro to create authenticated command handlers
#[macro_export]
macro_rules! authenticated_command {
    ($handler:ty, $permissions:expr, $roles:expr) => {
        impl AuthenticatedCommand for $handler {
            fn required_permissions(&self) -> Vec<&'static str> {
                $permissions
            }
            
            fn required_roles(&self) -> Vec<&'static str> {
                $roles
            }
            
            fn execute_authenticated(
                &self,
                auth_context: &UnifiedAuthContext,
                args: &[String],
            ) -> QmsResult<()> {
                // Validate permissions
                for permission in self.required_permissions() {
                    AuthContextManager::require_permission(auth_context, permission)?;
                }
                
                // Validate roles
                for role in self.required_roles() {
                    AuthContextManager::require_role(auth_context, role)?;
                }
                
                // Create execution context
                let mut exec_context = AuthContextManager::create_command_execution_context(auth_context);
                
                // Execute command
                self.execute_unified(&mut exec_context, args)
            }
        }
    };
}
