//! Interface Adapters for Unified Context System
//! 
//! Implements the Adapter Pattern to translate unified context operations
//! to interface-specific implementations following SOLID principles.

use crate::prelude::*;
use crate::interfaces::unified_context::{
    InterfaceAdapter, AuthenticationState, RoutingState, ConfigurationManager
};
use crate::interfaces::InterfaceType;
use std::collections::HashMap;

/// CLI Interface Adapter
/// 
/// Translates unified context operations to CLI-specific implementations.
/// Follows Single Responsibility Principle by handling only CLI adaptations.
pub struct CliInterfaceAdapter {
    /// CLI-specific state
    cli_state: std::sync::Mutex<CliAdapterState>,
}

/// Web Interface Adapter
/// 
/// Translates unified context operations to Web-specific implementations.
/// Manages session state and HTTP-specific routing.
pub struct WebInterfaceAdapter {
    /// Web-specific state
    web_state: std::sync::Mutex<WebAdapterState>,
}

/// TUI Interface Adapter
/// 
/// Translates unified context operations to TUI-specific implementations.
/// Handles terminal-specific state and navigation.
pub struct TuiInterfaceAdapter {
    /// TUI-specific state
    tui_state: std::sync::Mutex<TuiAdapterState>,
}

/// CLI adapter state
#[derive(Debug, Clone)]
struct CliAdapterState {
    /// Current CLI session file path
    session_file_path: Option<std::path::PathBuf>,
    
    /// CLI command history
    command_history: Vec<String>,
    
    /// CLI environment variables
    environment: HashMap<String, String>,
}

/// Web adapter state
#[derive(Debug, Clone)]
struct WebAdapterState {
    /// Active HTTP sessions
    active_sessions: HashMap<String, WebSessionInfo>,
    
    /// Web route mappings
    route_mappings: HashMap<String, String>,
    
    /// CSRF tokens
    csrf_tokens: HashMap<String, String>,
}

/// TUI adapter state
#[derive(Debug, Clone)]
struct TuiAdapterState {
    /// Current TUI screen
    current_screen: String,
    
    /// TUI navigation stack
    navigation_stack: Vec<String>,
    
    /// Terminal capabilities
    terminal_capabilities: TerminalCapabilities,
}

/// Web session information
#[derive(Debug, Clone)]
struct WebSessionInfo {
    /// Session ID
    session_id: String,
    
    /// User agent
    user_agent: Option<String>,
    
    /// IP address
    ip_address: Option<String>,
    
    /// Last activity timestamp
    last_activity: u64,
}

/// Terminal capabilities
#[derive(Debug, Clone)]
struct TerminalCapabilities {
    /// Color support
    supports_color: bool,
    
    /// Unicode support
    supports_unicode: bool,
    
    /// Terminal width
    width: u16,
    
    /// Terminal height
    height: u16,
}

impl CliInterfaceAdapter {
    /// Create new CLI interface adapter
    pub fn new() -> Self {
        Self {
            cli_state: std::sync::Mutex::new(CliAdapterState {
                session_file_path: None,
                command_history: Vec::new(),
                environment: std::env::vars().collect(),
            }),
        }
    }
    
    /// Update CLI session file path
    fn update_session_file_path(&self, session_id: &str) -> QmsResult<()> {
        let session_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        
        let session_file = std::path::PathBuf::from(session_dir)
            .join(".qms")
            .join("cli_session.json");
        
        let mut state = self.cli_state.lock().unwrap();
        state.session_file_path = Some(session_file);
        
        Ok(())
    }
    
    /// Add command to CLI history
    fn add_to_history(&self, command: &str) {
        let mut state = self.cli_state.lock().unwrap();
        state.command_history.push(command.to_string());
        
        // Keep only last 100 commands
        if state.command_history.len() > 100 {
            state.command_history.remove(0);
        }
    }
}

impl InterfaceAdapter for CliInterfaceAdapter {
    fn initialize_context(&self, _context: &mut crate::interfaces::unified_context::UnifiedInterfaceContext) -> QmsResult<()> {
        // Initialize CLI-specific context
        let mut state = self.cli_state.lock().unwrap();
        
        // Set up CLI environment
        state.environment.insert("QMS_INTERFACE".to_string(), "CLI".to_string());
        state.environment.insert("QMS_VERSION".to_string(), env!("CARGO_PKG_VERSION").to_string());
        
        Ok(())
    }
    
    fn handle_authentication(&self, auth_state: &AuthenticationState) -> QmsResult<()> {
        if let Some(ref session) = auth_state.current_session {
            // Update CLI session file
            self.update_session_file_path(&session.session_id)?;
            
            // Set CLI environment variables
            let mut state = self.cli_state.lock().unwrap();
            state.environment.insert("QMS_USER".to_string(), session.username.clone());
            state.environment.insert("QMS_SESSION_ID".to_string(), session.session_id.clone());
        }
        
        Ok(())
    }
    
    fn update_routing(&self, routing_state: &RoutingState) -> QmsResult<()> {
        // Add current route to CLI command history
        self.add_to_history(&routing_state.current_route);
        
        // Update CLI prompt or context based on current route
        let mut state = self.cli_state.lock().unwrap();
        state.environment.insert("QMS_CURRENT_ROUTE".to_string(), routing_state.current_route.clone());
        
        Ok(())
    }
    
    fn apply_configuration(&self, _config: &ConfigurationManager) -> QmsResult<()> {
        // Apply CLI-specific configuration
        // This could include setting up CLI aliases, colors, etc.
        Ok(())
    }
    
    fn cleanup(&self) -> QmsResult<()> {
        // Clean up CLI-specific resources
        let mut state = self.cli_state.lock().unwrap();
        state.session_file_path = None;
        state.environment.remove("QMS_USER");
        state.environment.remove("QMS_SESSION_ID");
        
        Ok(())
    }
}

impl WebInterfaceAdapter {
    /// Create new Web interface adapter
    pub fn new() -> Self {
        Self {
            web_state: std::sync::Mutex::new(WebAdapterState {
                active_sessions: HashMap::new(),
                route_mappings: Self::create_default_route_mappings(),
                csrf_tokens: HashMap::new(),
            }),
        }
    }
    
    /// Create default route mappings for web interface
    fn create_default_route_mappings() -> HashMap<String, String> {
        let mut mappings = HashMap::new();
        
        // Map unified routes to web API endpoints
        mappings.insert("/".to_string(), "/api/dashboard".to_string());
        mappings.insert("/login".to_string(), "/api/auth/login".to_string());
        mappings.insert("/logout".to_string(), "/api/auth/logout".to_string());
        mappings.insert("/documents".to_string(), "/api/documents".to_string());
        mappings.insert("/risks".to_string(), "/api/risks".to_string());
        mappings.insert("/users".to_string(), "/api/users".to_string());
        mappings.insert("/projects".to_string(), "/api/projects".to_string());
        
        mappings
    }
    
    /// Generate CSRF token for session
    fn generate_csrf_token(&self, session_id: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        session_id.hash(&mut hasher);
        crate::utils::current_timestamp().hash(&mut hasher);
        
        format!("csrf_{:x}", hasher.finish())
    }
}

impl InterfaceAdapter for WebInterfaceAdapter {
    fn initialize_context(&self, _context: &mut crate::interfaces::unified_context::UnifiedInterfaceContext) -> QmsResult<()> {
        // Initialize web-specific context
        // Set up CORS headers, security policies, etc.
        Ok(())
    }
    
    fn handle_authentication(&self, auth_state: &AuthenticationState) -> QmsResult<()> {
        if let Some(ref session) = auth_state.current_session {
            let mut state = self.web_state.lock().unwrap();
            
            // Add session to active sessions
            state.active_sessions.insert(session.session_id.clone(), WebSessionInfo {
                session_id: session.session_id.clone(),
                user_agent: session.user_agent.clone(),
                ip_address: session.ip_address.clone(),
                last_activity: crate::utils::current_timestamp(),
            });
            
            // Generate CSRF token
            let csrf_token = self.generate_csrf_token(&session.session_id);
            state.csrf_tokens.insert(session.session_id.clone(), csrf_token);
        }
        
        Ok(())
    }
    
    fn update_routing(&self, routing_state: &RoutingState) -> QmsResult<()> {
        let state = self.web_state.lock().unwrap();
        
        // Map unified route to web-specific route
        if let Some(web_route) = state.route_mappings.get(&routing_state.current_route) {
            // Update web routing context
            // This could trigger client-side navigation updates
        }
        
        Ok(())
    }
    
    fn apply_configuration(&self, _config: &ConfigurationManager) -> QmsResult<()> {
        // Apply web-specific configuration
        // This could include CORS settings, security headers, etc.
        Ok(())
    }
    
    fn cleanup(&self) -> QmsResult<()> {
        // Clean up web-specific resources
        let mut state = self.web_state.lock().unwrap();
        state.active_sessions.clear();
        state.csrf_tokens.clear();
        
        Ok(())
    }
}

impl TuiInterfaceAdapter {
    /// Create new TUI interface adapter
    pub fn new() -> Self {
        Self {
            tui_state: std::sync::Mutex::new(TuiAdapterState {
                current_screen: "login".to_string(),
                navigation_stack: vec!["login".to_string()],
                terminal_capabilities: TerminalCapabilities::detect(),
            }),
        }
    }
}

impl InterfaceAdapter for TuiInterfaceAdapter {
    fn initialize_context(&self, _context: &mut crate::interfaces::unified_context::UnifiedInterfaceContext) -> QmsResult<()> {
        // Initialize TUI-specific context
        let mut state = self.tui_state.lock().unwrap();
        state.terminal_capabilities = TerminalCapabilities::detect();
        
        Ok(())
    }
    
    fn handle_authentication(&self, auth_state: &AuthenticationState) -> QmsResult<()> {
        let mut state = self.tui_state.lock().unwrap();
        
        if auth_state.current_session.is_some() {
            // Navigate to main menu after successful authentication
            state.current_screen = "main_menu".to_string();
            state.navigation_stack.push("main_menu".to_string());
        } else {
            // Navigate back to login screen
            state.current_screen = "login".to_string();
            state.navigation_stack.clear();
            state.navigation_stack.push("login".to_string());
        }
        
        Ok(())
    }
    
    fn update_routing(&self, routing_state: &RoutingState) -> QmsResult<()> {
        let mut state = self.tui_state.lock().unwrap();
        
        // Map unified route to TUI screen
        let tui_screen = match routing_state.current_route.as_str() {
            "/" | "/login" => "login",
            "/main_menu" | "/dashboard" => "main_menu",
            "/documents" => "documents",
            "/risks" => "risks",
            "/users" => "users",
            "/settings" => "settings",
            _ => "main_menu",
        };
        
        state.current_screen = tui_screen.to_string();
        state.navigation_stack.push(tui_screen.to_string());
        
        Ok(())
    }
    
    fn apply_configuration(&self, _config: &ConfigurationManager) -> QmsResult<()> {
        // Apply TUI-specific configuration
        // This could include color schemes, key bindings, etc.
        Ok(())
    }
    
    fn cleanup(&self) -> QmsResult<()> {
        // Clean up TUI-specific resources
        let mut state = self.tui_state.lock().unwrap();
        state.navigation_stack.clear();
        state.current_screen = "login".to_string();
        
        Ok(())
    }
}

impl TerminalCapabilities {
    /// Detect terminal capabilities
    fn detect() -> Self {
        Self {
            supports_color: std::env::var("NO_COLOR").is_err(),
            supports_unicode: std::env::var("LANG").unwrap_or_default().contains("UTF"),
            width: 80,  // Default width
            height: 24, // Default height
        }
    }
}
