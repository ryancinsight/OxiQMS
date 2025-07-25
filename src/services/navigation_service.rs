//! Unified Navigation Service
//! 
//! Consolidates routing and navigation logic across CLI, TUI, and web interfaces
//! following SOLID principles and DRY methodology.

use crate::prelude::*;
use crate::interfaces::{
    InterfaceContext, InterfaceType, CommandResult,
    routing::{UnifiedRouter, CliRouter, WebRouter, TuiRouter, CommandHandler}
};
use crate::services::unified_service_manager::{ServiceManagerInterface, UnifiedServiceManager};
use crate::modules::audit_logger::audit_log_action;
use std::collections::HashMap;
use std::sync::Arc;

/// Navigation Service Interface
/// 
/// Provides unified navigation and routing capabilities that can be used
/// by CLI, TUI, and web interfaces, eliminating routing code duplication.
pub trait NavigationServiceInterface: Send + Sync {
    /// Route command to appropriate handler
    fn route_command(
        &self,
        interface_type: InterfaceType,
        command: &str,
        args: &[String],
        context: &InterfaceContext,
    ) -> QmsResult<CommandResult>;

    /// Check if command requires authentication
    fn requires_authentication(&self, interface_type: InterfaceType, command: &str) -> bool;

    /// Get available commands for interface
    fn get_available_commands(&self, interface_type: InterfaceType) -> Vec<String>;

    /// Get command help text
    fn get_command_help(&self, interface_type: InterfaceType, command: &str) -> Option<String>;

    /// Validate command arguments
    fn validate_command_args(
        &self,
        interface_type: InterfaceType,
        command: &str,
        args: &[String],
    ) -> QmsResult<()>;

    /// Navigate to specific route (for TUI/Web)
    fn navigate_to(&self, interface_type: InterfaceType, route: &str) -> QmsResult<NavigationResult>;

    /// Get current navigation state
    fn get_navigation_state(&self, interface_type: InterfaceType) -> NavigationState;

    /// Update navigation history
    fn update_navigation_history(&self, interface_type: &InterfaceType, route: &str) -> QmsResult<()>;
}

/// Navigation result
#[derive(Debug, Clone)]
pub struct NavigationResult {
    pub success: bool,
    pub route: String,
    pub data: Option<HashMap<String, String>>,
    pub redirect: Option<String>,
}

/// Navigation state
#[derive(Debug, Clone)]
pub struct NavigationState {
    pub current_route: String,
    pub history: Vec<String>,
    pub breadcrumbs: Vec<Breadcrumb>,
    pub can_go_back: bool,
    pub can_go_forward: bool,
}

/// Breadcrumb for navigation
#[derive(Debug, Clone)]
pub struct Breadcrumb {
    pub label: String,
    pub route: String,
    pub is_current: bool,
}

/// Unified Navigation Service Implementation
/// 
/// Central navigation coordinator that manages routing across all interfaces
/// using the unified router system and service manager integration.
pub struct UnifiedNavigationService {
    cli_router: Arc<dyn UnifiedRouter>,
    web_router: Arc<dyn UnifiedRouter>,
    tui_router: Arc<dyn UnifiedRouter>,
    service_manager: Arc<dyn ServiceManagerInterface>,
    navigation_states: Arc<std::sync::Mutex<HashMap<InterfaceType, NavigationState>>>,
}

impl UnifiedNavigationService {
    /// Create new unified navigation service
    pub fn new(service_manager: Arc<dyn ServiceManagerInterface>) -> QmsResult<Self> {
        // Create interface-specific routers
        let cli_router = Arc::new(CliRouter::new());
        let web_router = Arc::new(WebRouter::new());
        let tui_router = Arc::new(TuiRouter::new());

        // Initialize navigation states
        let mut navigation_states = HashMap::new();
        navigation_states.insert(InterfaceType::CLI, NavigationState {
            current_route: "/".to_string(),
            history: vec!["/".to_string()],
            breadcrumbs: vec![Breadcrumb {
                label: "Home".to_string(),
                route: "/".to_string(),
                is_current: true,
            }],
            can_go_back: false,
            can_go_forward: false,
        });

        navigation_states.insert(InterfaceType::Web, NavigationState {
            current_route: "/".to_string(),
            history: vec!["/".to_string()],
            breadcrumbs: vec![Breadcrumb {
                label: "Dashboard".to_string(),
                route: "/".to_string(),
                is_current: true,
            }],
            can_go_back: false,
            can_go_forward: false,
        });

        navigation_states.insert(InterfaceType::TUI, NavigationState {
            current_route: "/main".to_string(),
            history: vec!["/main".to_string()],
            breadcrumbs: vec![Breadcrumb {
                label: "Main Menu".to_string(),
                route: "/main".to_string(),
                is_current: true,
            }],
            can_go_back: false,
            can_go_forward: false,
        });

        Ok(Self {
            cli_router,
            web_router,
            tui_router,
            service_manager,
            navigation_states: Arc::new(std::sync::Mutex::new(navigation_states)),
        })
    }

    /// Get router for interface type
    fn get_router(&self, interface_type: &InterfaceType) -> Arc<dyn UnifiedRouter> {
        match interface_type {
            InterfaceType::CLI => self.cli_router.clone(),
            InterfaceType::Web => self.web_router.clone(),
            InterfaceType::TUI => self.tui_router.clone(),
        }
    }

    /// Convert command to route format
    fn command_to_route(&self, interface_type: &InterfaceType, command: &str) -> String {
        match interface_type {
            InterfaceType::CLI => format!("/{}", command),
            InterfaceType::Web => {
                if command.starts_with("api/") {
                    format!("/{}", command)
                } else {
                    format!("/api/{}", command)
                }
            }
            InterfaceType::TUI => format!("/{}", command),
        }
    }

    /// Update breadcrumbs based on route
    fn update_breadcrumbs(&self, route: &str) -> Vec<Breadcrumb> {
        let mut breadcrumbs = Vec::new();
        
        // Always start with home/root
        breadcrumbs.push(Breadcrumb {
            label: "Home".to_string(),
            route: "/".to_string(),
            is_current: route == "/",
        });

        // Parse route segments
        let segments: Vec<&str> = route.split('/').filter(|s| !s.is_empty()).collect();
        let mut current_path = String::new();

        for (i, segment) in segments.iter().enumerate() {
            current_path.push('/');
            current_path.push_str(segment);
            
            let is_current = i == segments.len() - 1;
            let label = match *segment {
                "doc" | "documents" => "Documents",
                "risk" | "risks" => "Risk Management",
                "user" | "users" => "User Management",
                "report" | "reports" => "Reports",
                "audit" => "Audit",
                "req" | "requirements" => "Requirements",
                "test" | "tests" => "Testing",
                "trace" | "traceability" => "Traceability",
                _ => segment,
            };

            breadcrumbs.push(Breadcrumb {
                label: label.to_string(),
                route: current_path.clone(),
                is_current,
            });
        }

        breadcrumbs
    }

    /// Log navigation event for audit
    fn log_navigation(&self, interface_type: &InterfaceType, command: &str, success: bool) {
        let event_type = if success {
            "NAVIGATION_SUCCESS"
        } else {
            "NAVIGATION_FAILED"
        };

        let _ = audit_log_action(
            event_type,
            "Navigation",
            &format!("{:?}:{}", interface_type, command),
        );
    }
}

impl NavigationServiceInterface for UnifiedNavigationService {
    fn route_command(
        &self,
        interface_type: InterfaceType,
        command: &str,
        args: &[String],
        context: &InterfaceContext,
    ) -> QmsResult<CommandResult> {
        let router = self.get_router(&interface_type);

        // Route command through unified router
        let result = router.route_command(context, command, args);

        // Log navigation event
        self.log_navigation(&interface_type, command, result.is_ok());

        // Update navigation history on success
        if result.is_ok() {
            let route = self.command_to_route(&interface_type, command);
            let _ = self.update_navigation_history(&interface_type, &route);
        }
        
        result
    }

    fn requires_authentication(&self, interface_type: InterfaceType, command: &str) -> bool {
        let router = self.get_router(&interface_type);
        router.requires_authentication(command)
    }

    fn get_available_commands(&self, interface_type: InterfaceType) -> Vec<String> {
        let router = self.get_router(&interface_type);
        router.available_commands().iter().map(|s| s.to_string()).collect()
    }

    fn get_command_help(&self, interface_type: InterfaceType, command: &str) -> Option<String> {
        let router = self.get_router(&interface_type);
        router.get_command_help(command).map(|s| s.to_string())
    }

    fn validate_command_args(
        &self,
        interface_type: InterfaceType,
        command: &str,
        args: &[String],
    ) -> QmsResult<()> {
        let router = self.get_router(&interface_type);
        router.validate_command_args(command, args)
    }

    fn navigate_to(&self, interface_type: InterfaceType, route: &str) -> QmsResult<NavigationResult> {
        // Update navigation state
        self.update_navigation_history(&interface_type, route)?;

        // Create navigation result
        let result = NavigationResult {
            success: true,
            route: route.to_string(),
            data: None,
            redirect: None,
        };

        // Log navigation
        self.log_navigation(&interface_type, route, true);

        Ok(result)
    }

    fn get_navigation_state(&self, interface_type: InterfaceType) -> NavigationState {
        let states = self.navigation_states.lock().unwrap();
        states.get(&interface_type).cloned().unwrap_or_else(|| NavigationState {
            current_route: "/".to_string(),
            history: vec!["/".to_string()],
            breadcrumbs: vec![],
            can_go_back: false,
            can_go_forward: false,
        })
    }

    fn update_navigation_history(&self, interface_type: &InterfaceType, route: &str) -> QmsResult<()> {
        let mut states = self.navigation_states.lock().unwrap();
        
        if let Some(state) = states.get_mut(&interface_type) {
            // Don't add duplicate consecutive routes
            if state.current_route != route {
                state.history.push(route.to_string());
                state.current_route = route.to_string();
                state.breadcrumbs = self.update_breadcrumbs(route);
                
                // Update navigation capabilities
                state.can_go_back = state.history.len() > 1;
                state.can_go_forward = false; // Reset forward capability
                
                // Limit history size to prevent memory issues
                if state.history.len() > 100 {
                    state.history.remove(0);
                }
            }
        }
        
        Ok(())
    }
}

/// Navigation Service Factory
/// 
/// Factory for creating navigation services with different configurations.
pub struct NavigationServiceFactory;

impl NavigationServiceFactory {
    /// Create navigation service for CLI interface
    pub fn create_for_cli(service_manager: Arc<dyn ServiceManagerInterface>) -> QmsResult<UnifiedNavigationService> {
        UnifiedNavigationService::new(service_manager)
    }

    /// Create navigation service for web interface
    pub fn create_for_web(service_manager: Arc<dyn ServiceManagerInterface>) -> QmsResult<UnifiedNavigationService> {
        UnifiedNavigationService::new(service_manager)
    }

    /// Create navigation service for TUI interface
    pub fn create_for_tui(service_manager: Arc<dyn ServiceManagerInterface>) -> QmsResult<UnifiedNavigationService> {
        UnifiedNavigationService::new(service_manager)
    }

    /// Create navigation service for testing
    pub fn create_for_testing(service_manager: Arc<dyn ServiceManagerInterface>) -> QmsResult<UnifiedNavigationService> {
        UnifiedNavigationService::new(service_manager)
    }
}
