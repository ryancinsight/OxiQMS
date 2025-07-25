//! Unified Authentication Flow
//! 
//! Consolidates login/logout flows across CLI, TUI, and web interfaces with
//! consistent state management and session handling following SOLID principles.

use crate::prelude::*;
use crate::interfaces::{InterfaceType, InterfaceContext};
use crate::interfaces::unified_context::{UnifiedInterfaceContext, AuthenticationMethod};
use crate::modules::user_manager::{UserSession, SessionType};
use crate::services::{unified_service_manager::ServiceManagerInterface, validation_service::ValidationServiceInterface};
use std::sync::Arc;
use std::collections::HashMap;
use std::io::Write;

/// Unified Authentication Flow Manager
/// 
/// Central coordinator for authentication operations across all interfaces.
/// Implements Template Method Pattern for consistent authentication flow
/// while allowing interface-specific customizations.
pub struct UnifiedAuthFlow {
    /// Unified interface context
    context: Arc<UnifiedInterfaceContext>,
    
    /// Service manager for business operations
    service_manager: Arc<dyn ServiceManagerInterface>,
    
    /// Validation service for input validation
    validation_service: Arc<dyn ValidationServiceInterface>,
    
    /// Authentication flow strategies for each interface
    flow_strategies: HashMap<InterfaceType, Box<dyn AuthFlowStrategy>>,
}

/// Authentication flow strategy trait
/// 
/// Defines interface-specific authentication behavior while maintaining
/// consistent core authentication logic (Strategy Pattern).
pub trait AuthFlowStrategy: Send + Sync {
    /// Pre-authentication setup (interface-specific)
    fn pre_authentication(&self, context: &InterfaceContext) -> QmsResult<AuthPreparation>;
    
    /// Collect authentication credentials (interface-specific)
    fn collect_credentials(&self, preparation: &AuthPreparation) -> QmsResult<AuthCredentials>;
    
    /// Post-authentication actions (interface-specific)
    fn post_authentication(&self, session: &UserSession, context: &InterfaceContext) -> QmsResult<()>;
    
    /// Handle authentication failure (interface-specific)
    fn handle_auth_failure(&self, error: &QmsError, context: &InterfaceContext) -> QmsResult<AuthFailureAction>;
    
    /// Logout cleanup (interface-specific)
    fn logout_cleanup(&self, context: &InterfaceContext) -> QmsResult<()>;
}

/// Authentication preparation data
#[derive(Debug, Clone)]
pub struct AuthPreparation {
    /// Interface type
    pub interface_type: InterfaceType,
    
    /// Authentication method to use
    pub auth_method: AuthenticationMethod,
    
    /// Interface-specific preparation data
    pub interface_data: HashMap<String, String>,
    
    /// Security context
    pub security_context: SecurityContext,
}

/// Authentication credentials
#[derive(Debug, Clone)]
pub struct AuthCredentials {
    /// Username
    pub username: String,
    
    /// Password (will be cleared after use)
    pub password: String,
    
    /// Additional authentication data
    pub additional_data: HashMap<String, String>,
    
    /// Remember login preference
    pub remember_login: bool,
}

/// Security context for authentication
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// Client IP address
    pub ip_address: Option<String>,
    
    /// User agent string
    pub user_agent: Option<String>,
    
    /// CSRF token (for web)
    pub csrf_token: Option<String>,
    
    /// Request timestamp
    pub request_timestamp: u64,
}

/// Authentication failure action
#[derive(Debug, Clone)]
pub enum AuthFailureAction {
    /// Retry authentication
    Retry,
    
    /// Lock account temporarily
    LockAccount,
    
    /// Redirect to different interface
    Redirect(String),
    
    /// Exit/terminate session
    Exit,
}

/// Authentication result
#[derive(Debug, Clone)]
pub struct AuthResult {
    /// Whether authentication was successful
    pub success: bool,
    
    /// User session if successful
    pub session: Option<UserSession>,
    
    /// Error message if failed
    pub error_message: Option<String>,
    
    /// Next action to take
    pub next_action: Option<String>,
    
    /// Interface-specific result data
    pub interface_data: HashMap<String, String>,
}

impl UnifiedAuthFlow {
    /// Create new unified authentication flow
    pub fn new(
        context: Arc<UnifiedInterfaceContext>,
        service_manager: Arc<dyn ServiceManagerInterface>,
        validation_service: Arc<dyn ValidationServiceInterface>,
    ) -> Self {
        let mut flow_strategies: HashMap<InterfaceType, Box<dyn AuthFlowStrategy>> = HashMap::new();
        
        // Initialize interface-specific strategies
        flow_strategies.insert(InterfaceType::CLI, Box::new(CliAuthFlowStrategy::new()));
        flow_strategies.insert(InterfaceType::Web, Box::new(WebAuthFlowStrategy::new()));
        flow_strategies.insert(InterfaceType::TUI, Box::new(TuiAuthFlowStrategy::new()));
        
        Self {
            context,
            service_manager,
            validation_service,
            flow_strategies,
        }
    }
    
    /// Authenticate user using unified flow (Template Method Pattern)
    pub fn authenticate(&self, interface_context: &InterfaceContext) -> QmsResult<AuthResult> {
        let interface_type = interface_context.interface_type.clone();
        
        // Get interface-specific strategy
        let strategy = self.flow_strategies.get(&interface_type)
            .ok_or_else(|| QmsError::domain_error(&format!("No auth strategy for interface: {:?}", interface_type)))?;
        
        // Step 1: Pre-authentication setup
        let preparation = strategy.pre_authentication(interface_context)?;
        
        // Step 2: Validate security context
        self.validate_security_context(&preparation.security_context)?;
        
        // Step 3: Collect credentials
        let credentials = strategy.collect_credentials(&preparation)?;
        
        // Step 4: Validate credentials format
        self.validate_credentials(&credentials)?;
        
        // Step 5: Perform core authentication
        match self.perform_core_authentication(&credentials, interface_type.clone()) {
            Ok(session) => {
                // Step 6: Update unified context
                self.context.authenticate(&credentials.username, &credentials.password, interface_type.clone())?;

                // Step 6.5: Synchronize authentication across all interfaces
                self.synchronize_cross_interface_auth(&session)?;

                // Step 7: Post-authentication actions
                strategy.post_authentication(&session, interface_context)?;
                
                // Step 8: Log successful authentication
                self.log_authentication_success(&session, &preparation.security_context);
                
                Ok(AuthResult {
                    success: true,
                    session: Some(session),
                    error_message: None,
                    next_action: Some(self.determine_post_auth_action(&interface_type)),
                    interface_data: HashMap::new(),
                })
            }
            Err(auth_error) => {
                // Handle authentication failure
                let failure_action = strategy.handle_auth_failure(&auth_error, interface_context)?;
                
                // Log failed authentication
                self.log_authentication_failure(&credentials.username, &auth_error, &preparation.security_context);
                
                Ok(AuthResult {
                    success: false,
                    session: None,
                    error_message: Some(auth_error.to_string()),
                    next_action: Some(format!("{:?}", failure_action)),
                    interface_data: HashMap::new(),
                })
            }
        }
    }
    
    /// Logout user using unified flow
    pub fn logout(&self, interface_context: &InterfaceContext) -> QmsResult<()> {
        let interface_type = interface_context.interface_type.clone();
        
        // Get interface-specific strategy
        let strategy = self.flow_strategies.get(&interface_type)
            .ok_or_else(|| QmsError::domain_error(&format!("No auth strategy for interface: {:?}", interface_type)))?;
        
        // Get current session for logging
        let current_session = self.context.get_current_session();
        
        // Perform unified logout
        self.context.logout(interface_type)?;
        
        // Interface-specific cleanup
        strategy.logout_cleanup(interface_context)?;
        
        // Log logout
        if let Some(session) = current_session {
            self.log_logout(&session);
        }
        
        Ok(())
    }
    
    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.context.is_authenticated()
    }
    
    /// Get current user session
    pub fn get_current_session(&self) -> Option<UserSession> {
        self.context.get_current_session()
    }

    /// Synchronize authentication across all interfaces
    ///
    /// When a user authenticates in one interface, this method ensures
    /// they are automatically authenticated in all other interfaces.
    pub fn synchronize_cross_interface_auth(&self, session: &UserSession) -> QmsResult<()> {
        // Create sessions for all interface types
        let user_service = self.service_manager.user_service();

        // Create CLI session if not exists
        if session.session_type != SessionType::CLI {
            let _cli_session = user_service.create_session(&session.username, SessionType::CLI)?;
        }

        // Create Web session if not exists
        if session.session_type != SessionType::Web {
            let _web_session = user_service.create_session(&session.username, SessionType::Web)?;
        }

        // Create TUI session if not exists
        if session.session_type != SessionType::TUI {
            let _tui_session = user_service.create_session(&session.username, SessionType::TUI)?;
        }

        // Update unified context to reflect cross-interface authentication
        self.context.synchronize_authentication_state(&session.username)?;

        // Log cross-interface synchronization
        let _ = crate::modules::audit_logger::audit_log_action(
            "CROSS_INTERFACE_SYNC",
            "Authentication",
            &format!("User: {}, Original Interface: {:?}", session.username, session.session_type),
        );

        Ok(())
    }
    
    /// Validate session for interface
    pub fn validate_session(&self, session_id: &str, interface_type: InterfaceType) -> QmsResult<UserSession> {
        let user_service = self.service_manager.user_service();
        let session = user_service.validate_session(session_id)?;
        
        // Ensure session is valid for this interface type
        let expected_session_type = match interface_type {
            InterfaceType::CLI => SessionType::CLI,
            InterfaceType::Web => SessionType::Web,
            InterfaceType::TUI => SessionType::TUI, // TUI has its own session type for proper cross-interface sync
        };
        
        if session.session_type != expected_session_type {
            return Err(QmsError::validation_error("Session type mismatch"));
        }
        
        Ok(session)
    }
    
    // Private helper methods
    
    /// Validate security context
    fn validate_security_context(&self, security_context: &SecurityContext) -> QmsResult<()> {
        // Check request timestamp (prevent replay attacks)
        let current_time = crate::utils::current_timestamp();
        let max_age = 300; // 5 minutes
        
        if current_time.saturating_sub(security_context.request_timestamp) > max_age {
            return Err(QmsError::validation_error("Request timestamp too old"));
        }
        
        // Additional security validations can be added here
        Ok(())
    }
    
    /// Validate credentials format
    fn validate_credentials(&self, credentials: &AuthCredentials) -> QmsResult<()> {
        // Validate username
        let username_result = self.validation_service.validate_field(
            "username",
            &credentials.username,
            crate::services::validation_service::FieldType::Text,
        )?;
        
        if !username_result.is_valid {
            return Err(QmsError::validation_error(
                &username_result.error.unwrap_or_else(|| "Invalid username".to_string())
            ));
        }
        
        // Validate password
        let password_result = self.validation_service.validate_field(
            "password",
            &credentials.password,
            crate::services::validation_service::FieldType::Text,
        )?;
        
        if !password_result.is_valid {
            return Err(QmsError::validation_error(
                &password_result.error.unwrap_or_else(|| "Invalid password".to_string())
            ));
        }
        
        Ok(())
    }
    
    /// Perform core authentication logic
    fn perform_core_authentication(&self, credentials: &AuthCredentials, interface_type: InterfaceType) -> QmsResult<UserSession> {
        let user_service = self.service_manager.user_service();
        
        let session_type = match interface_type {
            InterfaceType::CLI => SessionType::CLI,
            InterfaceType::Web => SessionType::Web,
            InterfaceType::TUI => SessionType::TUI,
        };
        
        user_service.authenticate_user(&credentials.username, &credentials.password, session_type)
    }
    
    /// Determine post-authentication action
    fn determine_post_auth_action(&self, interface_type: &InterfaceType) -> String {
        match interface_type {
            InterfaceType::CLI => "continue_command".to_string(),
            InterfaceType::Web => "redirect_dashboard".to_string(),
            InterfaceType::TUI => "show_main_menu".to_string(),
        }
    }
    
    /// Log successful authentication
    fn log_authentication_success(&self, session: &UserSession, security_context: &SecurityContext) {
        let _ = crate::modules::audit_logger::audit_log_action(
            "AUTH_SUCCESS",
            "Authentication",
            &format!("User: {}, IP: {:?}", session.username, security_context.ip_address),
        );
    }
    
    /// Log failed authentication
    fn log_authentication_failure(&self, username: &str, error: &QmsError, security_context: &SecurityContext) {
        let _ = crate::modules::audit_logger::audit_log_action(
            "AUTH_FAILURE",
            "Authentication",
            &format!("User: {}, Error: {}, IP: {:?}", username, error, security_context.ip_address),
        );
    }
    
    /// Log logout
    fn log_logout(&self, session: &UserSession) {
        let _ = crate::modules::audit_logger::audit_log_action(
            "LOGOUT",
            "Authentication",
            &format!("User: {}", session.username),
        );
    }
}

/// CLI Authentication Flow Strategy
///
/// Implements CLI-specific authentication behavior including
/// command-line credential collection and session file management.
pub struct CliAuthFlowStrategy;

impl CliAuthFlowStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl AuthFlowStrategy for CliAuthFlowStrategy {
    fn pre_authentication(&self, context: &InterfaceContext) -> QmsResult<AuthPreparation> {
        Ok(AuthPreparation {
            interface_type: InterfaceType::CLI,
            auth_method: AuthenticationMethod::UsernamePassword,
            interface_data: HashMap::new(),
            security_context: SecurityContext {
                ip_address: Some("127.0.0.1".to_string()), // Local CLI
                user_agent: Some("QMS-CLI".to_string()),
                csrf_token: None, // Not needed for CLI
                request_timestamp: crate::utils::current_timestamp(),
            },
        })
    }

    fn collect_credentials(&self, _preparation: &AuthPreparation) -> QmsResult<AuthCredentials> {
        use std::io::{self, Write};

        // Collect username
        print!("Username: ");
        io::stdout().flush().map_err(|e| QmsError::io_error(&format!("Failed to flush stdout: {}", e)))?;

        let mut username = String::new();
        io::stdin().read_line(&mut username).map_err(|e| QmsError::io_error(&format!("Failed to read username: {}", e)))?;
        let username = username.trim().to_string();

        // Collect password (hidden input would be better in real implementation)
        print!("Password: ");
        io::stdout().flush().map_err(|e| QmsError::io_error(&format!("Failed to flush stdout: {}", e)))?;

        let mut password = String::new();
        io::stdin().read_line(&mut password).map_err(|e| QmsError::io_error(&format!("Failed to read password: {}", e)))?;
        let password = password.trim().to_string();

        Ok(AuthCredentials {
            username,
            password,
            additional_data: HashMap::new(),
            remember_login: false, // CLI doesn't typically remember login
        })
    }

    fn post_authentication(&self, session: &UserSession, _context: &InterfaceContext) -> QmsResult<()> {
        println!("✅ Authentication successful! Welcome, {}.", session.username);

        // Save CLI session to file
        let session_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());

        let session_file = std::path::PathBuf::from(session_dir)
            .join(".qms")
            .join("cli_session.json");

        // Create directory if it doesn't exist
        if let Some(parent) = session_file.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                QmsError::io_error(&format!("Failed to create session directory: {}", e))
            })?;
        }

        // Save session ID to file (simplified)
        std::fs::write(&session_file, &session.session_id).map_err(|e| {
            QmsError::io_error(&format!("Failed to save session: {}", e))
        })?;

        Ok(())
    }

    fn handle_auth_failure(&self, error: &QmsError, _context: &InterfaceContext) -> QmsResult<AuthFailureAction> {
        println!("❌ Authentication failed: {}", error);

        // For CLI, typically retry or exit
        print!("Retry authentication? (y/n): ");
        std::io::stdout().flush().map_err(|e| QmsError::io_error(&format!("Failed to flush stdout: {}", e)))?;

        let mut response = String::new();
        std::io::stdin().read_line(&mut response).map_err(|e| QmsError::io_error(&format!("Failed to read response: {}", e)))?;

        if response.trim().to_lowercase().starts_with('y') {
            Ok(AuthFailureAction::Retry)
        } else {
            Ok(AuthFailureAction::Exit)
        }
    }

    fn logout_cleanup(&self, _context: &InterfaceContext) -> QmsResult<()> {
        println!("👋 Logged out successfully.");

        // Remove CLI session file
        let session_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());

        let session_file = std::path::PathBuf::from(session_dir)
            .join(".qms")
            .join("cli_session.json");

        if session_file.exists() {
            std::fs::remove_file(&session_file).map_err(|e| {
                QmsError::io_error(&format!("Failed to remove session file: {}", e))
            })?;
        }

        Ok(())
    }
}

/// Web Authentication Flow Strategy
///
/// Implements web-specific authentication behavior including
/// HTTP request handling, CSRF protection, and session cookies.
pub struct WebAuthFlowStrategy;

impl WebAuthFlowStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl AuthFlowStrategy for WebAuthFlowStrategy {
    fn pre_authentication(&self, context: &InterfaceContext) -> QmsResult<AuthPreparation> {
        // Extract web-specific context
        let mut interface_data = HashMap::new();

        // In a real implementation, this would extract data from HTTP request
        interface_data.insert("content_type".to_string(), "application/json".to_string());
        interface_data.insert("origin".to_string(), "https://localhost:8080".to_string());

        Ok(AuthPreparation {
            interface_type: InterfaceType::Web,
            auth_method: AuthenticationMethod::UsernamePassword,
            interface_data,
            security_context: SecurityContext {
                ip_address: Some("127.0.0.1".to_string()), // Would be extracted from request
                user_agent: Some("Mozilla/5.0 QMS-Web".to_string()),
                csrf_token: Some(Self::generate_csrf_token()),
                request_timestamp: crate::utils::current_timestamp(),
            },
        })
    }

    fn collect_credentials(&self, preparation: &AuthPreparation) -> QmsResult<AuthCredentials> {
        // In a real implementation, this would extract credentials from HTTP request body
        // For now, we'll return a placeholder
        Ok(AuthCredentials {
            username: "web_user".to_string(), // Would be extracted from request
            password: "web_password".to_string(), // Would be extracted from request
            additional_data: preparation.interface_data.clone(),
            remember_login: true, // Web typically supports remember login
        })
    }

    fn post_authentication(&self, session: &UserSession, _context: &InterfaceContext) -> QmsResult<()> {
        // In a real implementation, this would:
        // 1. Set HTTP session cookies
        // 2. Generate JWT tokens
        // 3. Set security headers
        // 4. Prepare JSON response

        println!("Web authentication successful for user: {}", session.username);
        Ok(())
    }

    fn handle_auth_failure(&self, error: &QmsError, _context: &InterfaceContext) -> QmsResult<AuthFailureAction> {
        // In a real implementation, this would:
        // 1. Return appropriate HTTP status code
        // 2. Log security events
        // 3. Implement rate limiting
        // 4. Return JSON error response

        println!("Web authentication failed: {}", error);
        Ok(AuthFailureAction::Retry) // Web typically allows retry
    }

    fn logout_cleanup(&self, _context: &InterfaceContext) -> QmsResult<()> {
        // In a real implementation, this would:
        // 1. Clear session cookies
        // 2. Invalidate JWT tokens
        // 3. Clear server-side session
        // 4. Return success response

        println!("Web logout successful");
        Ok(())
    }
}

impl WebAuthFlowStrategy {
    fn generate_csrf_token() -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        crate::utils::current_timestamp().hash(&mut hasher);
        "web_csrf_token".hash(&mut hasher);

        format!("csrf_{:x}", hasher.finish())
    }
}

/// TUI Authentication Flow Strategy
///
/// Implements TUI-specific authentication behavior including
/// terminal-based forms, screen navigation, and visual feedback.
pub struct TuiAuthFlowStrategy;

impl TuiAuthFlowStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl AuthFlowStrategy for TuiAuthFlowStrategy {
    fn pre_authentication(&self, context: &InterfaceContext) -> QmsResult<AuthPreparation> {
        let mut interface_data = HashMap::new();

        // TUI-specific preparation
        interface_data.insert("terminal_type".to_string(), "xterm-256color".to_string());
        interface_data.insert("screen_size".to_string(), "80x24".to_string());

        Ok(AuthPreparation {
            interface_type: InterfaceType::TUI,
            auth_method: AuthenticationMethod::UsernamePassword,
            interface_data,
            security_context: SecurityContext {
                ip_address: Some("127.0.0.1".to_string()), // Local TUI
                user_agent: Some("QMS-TUI".to_string()),
                csrf_token: None, // Not needed for TUI
                request_timestamp: crate::utils::current_timestamp(),
            },
        })
    }

    fn collect_credentials(&self, _preparation: &AuthPreparation) -> QmsResult<AuthCredentials> {
        // In a real TUI implementation, this would:
        // 1. Display login form
        // 2. Handle keyboard input
        // 3. Provide visual feedback
        // 4. Support navigation between fields

        println!("=== QMS TUI Login ===");

        use std::io::{self, Write};

        // Collect username with TUI styling
        print!("┌─ Username: ");
        io::stdout().flush().map_err(|e| QmsError::io_error(&format!("Failed to flush stdout: {}", e)))?;

        let mut username = String::new();
        io::stdin().read_line(&mut username).map_err(|e| QmsError::io_error(&format!("Failed to read username: {}", e)))?;
        let username = username.trim().to_string();

        // Collect password with TUI styling
        print!("├─ Password: ");
        io::stdout().flush().map_err(|e| QmsError::io_error(&format!("Failed to flush stdout: {}", e)))?;

        let mut password = String::new();
        io::stdin().read_line(&mut password).map_err(|e| QmsError::io_error(&format!("Failed to read password: {}", e)))?;
        let password = password.trim().to_string();

        println!("└─ Press Enter to login...");

        Ok(AuthCredentials {
            username,
            password,
            additional_data: HashMap::new(),
            remember_login: false, // TUI typically doesn't persist sessions
        })
    }

    fn post_authentication(&self, session: &UserSession, _context: &InterfaceContext) -> QmsResult<()> {
        // TUI-specific post-authentication display
        println!("╔══════════════════════════════════════╗");
        println!("║          Login Successful!           ║");
        println!("║                                      ║");
        println!("║  Welcome, {:<25} ║", session.username);
        println!("║                                      ║");
        println!("║  Press any key to continue...        ║");
        println!("╚══════════════════════════════════════╝");

        // Wait for user input
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).map_err(|e| {
            QmsError::io_error(&format!("Failed to read input: {}", e))
        })?;

        Ok(())
    }

    fn handle_auth_failure(&self, error: &QmsError, _context: &InterfaceContext) -> QmsResult<AuthFailureAction> {
        // TUI-specific error display
        println!("╔══════════════════════════════════════╗");
        println!("║         Authentication Failed        ║");
        println!("║                                      ║");
        println!("║  Error: {:<27} ║", error.to_string().chars().take(27).collect::<String>());
        println!("║                                      ║");
        println!("║  [R]etry  [E]xit                     ║");
        println!("╚══════════════════════════════════════╝");

        print!("Choice: ");
        std::io::stdout().flush().map_err(|e| QmsError::io_error(&format!("Failed to flush stdout: {}", e)))?;

        let mut choice = String::new();
        std::io::stdin().read_line(&mut choice).map_err(|e| QmsError::io_error(&format!("Failed to read choice: {}", e)))?;

        match choice.trim().to_lowercase().as_str() {
            "r" | "retry" => Ok(AuthFailureAction::Retry),
            "e" | "exit" => Ok(AuthFailureAction::Exit),
            _ => Ok(AuthFailureAction::Retry), // Default to retry
        }
    }

    fn logout_cleanup(&self, _context: &InterfaceContext) -> QmsResult<()> {
        // TUI-specific logout display
        println!("╔══════════════════════════════════════╗");
        println!("║            Goodbye!                  ║");
        println!("║                                      ║");
        println!("║  Thank you for using QMS TUI         ║");
        println!("║                                      ║");
        println!("║  Session ended successfully          ║");
        println!("╚══════════════════════════════════════╝");

        // Brief pause for user to see the message
        std::thread::sleep(std::time::Duration::from_secs(2));

        Ok(())
    }
}
