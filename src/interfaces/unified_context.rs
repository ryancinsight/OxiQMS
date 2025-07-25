//! Unified Interface Context System
//! 
//! Consolidates login state, routing, project locations, and QMS settings across
//! CLI, TUI, and web interfaces following SOLID, CUPID, GRASP, ACID, KISS, DRY, and YAGNI principles.

use crate::prelude::*;
use crate::interfaces::{InterfaceType, InterfaceContext, CommandResult};
use crate::modules::user_manager::{UserSession, SessionType};
use crate::services::{UnifiedServiceManager, unified_service_manager::ServiceManagerInterface};
use crate::config::Config as QmsConfig;
use crate::json_utils::JsonValue;
use crate::interfaces::adapters::interface_adapters::{CliInterfaceAdapter, WebInterfaceAdapter, TuiInterfaceAdapter};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Unified Interface Context Manager
/// 
/// Central coordinator for all interface state management following Single Responsibility Principle.
/// Manages authentication, project context, configuration, and routing state consistently
/// across all interfaces (CLI, TUI, Web).
pub struct UnifiedInterfaceContext {
    /// Current authentication state
    authentication_state: Arc<Mutex<AuthenticationState>>,
    
    /// Project context management
    project_context: Arc<Mutex<ProjectContext>>,
    
    /// Configuration management
    configuration_manager: Arc<Mutex<ConfigurationManager>>,
    
    /// Routing state for each interface
    routing_states: Arc<Mutex<HashMap<InterfaceType, RoutingState>>>,
    
    /// Service manager for business operations
    service_manager: Arc<dyn ServiceManagerInterface>,
    
    /// Interface-specific adapters
    interface_adapters: HashMap<InterfaceType, Arc<dyn InterfaceAdapter>>,
}

/// Authentication state shared across all interfaces
#[derive(Debug, Clone)]
pub struct AuthenticationState {
    /// Current user session (None if not authenticated)
    pub current_session: Option<UserSession>,
    
    /// Authentication method used
    pub auth_method: Option<AuthenticationMethod>,
    
    /// Session persistence settings
    pub persistence_config: SessionPersistenceConfig,
    
    /// Last authentication attempt timestamp
    pub last_auth_attempt: Option<u64>,

    /// Failed authentication attempts counter
    pub failed_attempts: u32,

    /// Interfaces where user is currently authenticated (for cross-interface sync)
    pub authenticated_interfaces: std::collections::HashSet<InterfaceType>,

    /// Last activity timestamp across all interfaces
    pub last_activity: Option<u64>,
}

/// Project context shared across all interfaces
#[derive(Debug, Clone)]
pub struct ProjectContext {
    /// Currently active project path
    pub active_project_path: Option<PathBuf>,
    
    /// Available projects for current user
    pub available_projects: Vec<ProjectInfo>,
    
    /// Project detection service state
    pub detection_state: ProjectDetectionState,
    
    /// QMS folder configuration
    pub qms_folder_config: QmsFolderConfig,
}

/// Configuration manager for unified settings
#[derive(Debug, Clone)]
pub struct ConfigurationManager {
    /// Global QMS configuration
    pub global_config: QmsConfig,
    
    /// User-specific preferences
    pub user_preferences: HashMap<String, UserPreferences>,
    
    /// Interface-specific settings
    pub interface_settings: HashMap<InterfaceType, InterfaceSettings>,
    
    /// Configuration file paths
    pub config_paths: ConfigurationPaths,
}

/// Routing state for each interface
#[derive(Debug, Clone)]
pub struct RoutingState {
    /// Current route/location
    pub current_route: String,
    
    /// Navigation history
    pub navigation_history: Vec<NavigationEntry>,
    
    /// Available commands for current context
    pub available_commands: Vec<String>,
    
    /// Route parameters
    pub route_parameters: HashMap<String, String>,
    
    /// Breadcrumb trail
    pub breadcrumbs: Vec<Breadcrumb>,
}

/// Authentication method enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum AuthenticationMethod {
    /// Username/password authentication
    UsernamePassword,
    /// Session token authentication
    SessionToken,
    /// API key authentication
    ApiKey,
    /// Test/mock authentication
    Test,
}

/// Session persistence configuration
#[derive(Debug, Clone)]
pub struct SessionPersistenceConfig {
    /// Whether to persist sessions across interface restarts
    pub persist_sessions: bool,
    
    /// Session timeout in seconds
    pub session_timeout: u64,
    
    /// Auto-logout on inactivity
    pub auto_logout_enabled: bool,
    
    /// Remember login preference
    pub remember_login: bool,
}

/// Project information structure
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    /// Project unique identifier
    pub id: String,
    
    /// Project display name
    pub name: String,
    
    /// Project file system path
    pub path: PathBuf,
    
    /// Project metadata
    pub metadata: ProjectMetadata,
    
    /// Last accessed timestamp
    pub last_accessed: Option<u64>,
}

/// Project detection state
#[derive(Debug, Clone)]
pub struct ProjectDetectionState {
    /// Last detection scan timestamp
    pub last_scan: Option<u64>,
    
    /// Detection scan results
    pub scan_results: Vec<ProjectScanResult>,
    
    /// Auto-detection enabled
    pub auto_detection_enabled: bool,
}

/// QMS folder configuration
#[derive(Debug, Clone)]
pub struct QmsFolderConfig {
    /// Default QMS root folder
    pub default_qms_path: PathBuf,
    
    /// User-specific QMS folders
    pub user_qms_paths: HashMap<String, PathBuf>,
    
    /// QMS folder structure template
    pub folder_template: QmsFolderTemplate,
}

/// User preferences structure
#[derive(Debug, Clone)]
pub struct UserPreferences {
    /// Preferred interface theme
    pub theme: String,
    
    /// Language/locale preference
    pub locale: String,
    
    /// Default project selection
    pub default_project: Option<String>,
    
    /// Interface-specific preferences
    pub interface_prefs: HashMap<InterfaceType, JsonValue>,
    
    /// Accessibility settings
    pub accessibility: AccessibilitySettings,
}

/// Interface-specific settings
#[derive(Debug, Clone)]
pub struct InterfaceSettings {
    /// Interface display configuration
    pub display_config: DisplayConfiguration,
    
    /// Command aliases
    pub command_aliases: HashMap<String, String>,
    
    /// Keyboard shortcuts
    pub shortcuts: HashMap<String, String>,
    
    /// Interface behavior settings
    pub behavior_settings: BehaviorSettings,
}

/// Configuration file paths
#[derive(Debug, Clone)]
pub struct ConfigurationPaths {
    /// Global configuration file
    pub global_config_path: PathBuf,
    
    /// User preferences directory
    pub user_prefs_dir: PathBuf,
    
    /// Interface settings directory
    pub interface_settings_dir: PathBuf,
    
    /// Session storage directory
    pub session_storage_dir: PathBuf,
}

/// Navigation entry for history tracking
#[derive(Debug, Clone)]
pub struct NavigationEntry {
    /// Route/command that was navigated to
    pub route: String,
    
    /// Timestamp of navigation
    pub timestamp: u64,
    
    /// Interface type used
    pub interface_type: InterfaceType,
    
    /// Navigation context
    pub context: HashMap<String, String>,
}

/// Breadcrumb for navigation trail
#[derive(Debug, Clone)]
pub struct Breadcrumb {
    /// Display label
    pub label: String,
    
    /// Route to navigate back to
    pub route: String,
    
    /// Breadcrumb level
    pub level: u32,
}

/// Project metadata structure
#[derive(Debug, Clone)]
pub struct ProjectMetadata {
    /// Project description
    pub description: String,
    
    /// Project version
    pub version: String,
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Last modified timestamp
    pub modified_at: u64,
    
    /// Project tags
    pub tags: Vec<String>,
}

/// Project scan result
#[derive(Debug, Clone)]
pub struct ProjectScanResult {
    /// Scanned path
    pub path: PathBuf,
    
    /// Whether valid QMS project found
    pub is_valid_project: bool,
    
    /// Project information if valid
    pub project_info: Option<ProjectInfo>,
    
    /// Scan errors if any
    pub errors: Vec<String>,
}

/// QMS folder template
#[derive(Debug, Clone)]
pub struct QmsFolderTemplate {
    /// Required directories
    pub required_directories: Vec<String>,
    
    /// Optional directories
    pub optional_directories: Vec<String>,
    
    /// Template files to create
    pub template_files: Vec<TemplateFile>,
}

/// Template file definition
#[derive(Debug, Clone)]
pub struct TemplateFile {
    /// File path relative to project root
    pub path: String,
    
    /// File content template
    pub content: String,
    
    /// Whether file is required
    pub required: bool,
}

/// Accessibility settings
#[derive(Debug, Clone)]
pub struct AccessibilitySettings {
    /// High contrast mode
    pub high_contrast: bool,
    
    /// Screen reader support
    pub screen_reader_enabled: bool,
    
    /// Font size scaling
    pub font_scale: f32,
    
    /// Color blind friendly mode
    pub color_blind_friendly: bool,
}

/// Display configuration
#[derive(Debug, Clone)]
pub struct DisplayConfiguration {
    /// Color scheme
    pub color_scheme: String,
    
    /// Font family
    pub font_family: String,
    
    /// Font size
    pub font_size: u32,
    
    /// Layout preferences
    pub layout_prefs: HashMap<String, String>,
}

/// Behavior settings
#[derive(Debug, Clone)]
pub struct BehaviorSettings {
    /// Auto-save enabled
    pub auto_save: bool,
    
    /// Confirmation prompts enabled
    pub confirm_actions: bool,
    
    /// Auto-complete enabled
    pub auto_complete: bool,
    
    /// Command history size
    pub history_size: u32,
}

/// Interface adapter trait for interface-specific operations
pub trait InterfaceAdapter: Send + Sync {
    /// Initialize interface-specific context
    fn initialize_context(&self, context: &mut UnifiedInterfaceContext) -> QmsResult<()>;
    
    /// Handle interface-specific authentication
    fn handle_authentication(&self, auth_state: &AuthenticationState) -> QmsResult<()>;
    
    /// Update interface-specific routing
    fn update_routing(&self, routing_state: &RoutingState) -> QmsResult<()>;
    
    /// Apply interface-specific configuration
    fn apply_configuration(&self, config: &ConfigurationManager) -> QmsResult<()>;
    
    /// Handle interface-specific cleanup
    fn cleanup(&self) -> QmsResult<()>;
}

impl Default for AuthenticationState {
    fn default() -> Self {
        Self {
            current_session: None,
            auth_method: None,
            persistence_config: SessionPersistenceConfig::default(),
            last_auth_attempt: None,
            failed_attempts: 0,
            authenticated_interfaces: std::collections::HashSet::new(),
            last_activity: None,
        }
    }
}

impl Default for SessionPersistenceConfig {
    fn default() -> Self {
        Self {
            persist_sessions: true,
            session_timeout: 3600 * 8, // 8 hours
            auto_logout_enabled: true,
            remember_login: false,
        }
    }
}

impl Default for ProjectContext {
    fn default() -> Self {
        Self {
            active_project_path: None,
            available_projects: Vec::new(),
            detection_state: ProjectDetectionState::default(),
            qms_folder_config: QmsFolderConfig::default(),
        }
    }
}

impl Default for ProjectDetectionState {
    fn default() -> Self {
        Self {
            last_scan: None,
            scan_results: Vec::new(),
            auto_detection_enabled: true,
        }
    }
}

impl Default for QmsFolderConfig {
    fn default() -> Self {
        Self {
            default_qms_path: PathBuf::from("./qms"),
            user_qms_paths: HashMap::new(),
            folder_template: QmsFolderTemplate::default(),
        }
    }
}

impl Default for QmsFolderTemplate {
    fn default() -> Self {
        Self {
            required_directories: vec![
                "documents".to_string(),
                "risks".to_string(),
                "requirements".to_string(),
                "audit".to_string(),
                "config".to_string(),
            ],
            optional_directories: vec![
                "templates".to_string(),
                "reports".to_string(),
                "backups".to_string(),
            ],
            template_files: Vec::new(),
        }
    }
}

impl UnifiedInterfaceContext {
    /// Create new unified interface context
    pub fn new(service_manager: Arc<dyn ServiceManagerInterface>) -> QmsResult<Self> {
        let mut interface_adapters = HashMap::new();

        // Initialize interface adapters
        interface_adapters.insert(InterfaceType::CLI, Arc::new(CliInterfaceAdapter::new()) as Arc<dyn InterfaceAdapter>);
        interface_adapters.insert(InterfaceType::Web, Arc::new(WebInterfaceAdapter::new()) as Arc<dyn InterfaceAdapter>);
        interface_adapters.insert(InterfaceType::TUI, Arc::new(TuiInterfaceAdapter::new()) as Arc<dyn InterfaceAdapter>);

        // Initialize routing states for all interfaces
        let mut routing_states = HashMap::new();
        routing_states.insert(InterfaceType::CLI, RoutingState::default_for_cli());
        routing_states.insert(InterfaceType::Web, RoutingState::default_for_web());
        routing_states.insert(InterfaceType::TUI, RoutingState::default_for_tui());

        Ok(Self {
            authentication_state: Arc::new(Mutex::new(AuthenticationState::default())),
            project_context: Arc::new(Mutex::new(ProjectContext::default())),
            configuration_manager: Arc::new(Mutex::new(ConfigurationManager::new()?)),
            routing_states: Arc::new(Mutex::new(routing_states)),
            service_manager,
            interface_adapters,
        })
    }

    /// Authenticate user across all interfaces
    pub fn authenticate(&self, username: &str, password: &str, interface_type: InterfaceType) -> QmsResult<UserSession> {
        // Use unified authentication service
        let user_service = self.service_manager.user_service();
        let session = user_service.authenticate_user(username, password, self.get_session_type(interface_type.clone()))?;

        // Update authentication state
        {
            let mut auth_state = self.authentication_state.lock().unwrap();
            auth_state.current_session = Some(session.clone());
            auth_state.auth_method = Some(AuthenticationMethod::UsernamePassword);
            auth_state.last_auth_attempt = Some(crate::utils::current_timestamp());
            auth_state.failed_attempts = 0;
        }

        // Update project context for authenticated user
        self.update_project_context_for_user(&session.username)?;

        // Notify interface adapters
        if let Some(adapter) = self.interface_adapters.get(&interface_type) {
            let auth_state = self.authentication_state.lock().unwrap();
            adapter.handle_authentication(&auth_state)?;
        }

        Ok(session)
    }

    /// Logout user from all interfaces
    pub fn logout(&self, interface_type: InterfaceType) -> QmsResult<()> {
        // Clear authentication state
        {
            let mut auth_state = self.authentication_state.lock().unwrap();
            auth_state.current_session = None;
            auth_state.auth_method = None;
        }

        // Clear project context
        {
            let mut project_context = self.project_context.lock().unwrap();
            project_context.active_project_path = None;
        }

        // Clear routing states
        {
            let mut routing_states = self.routing_states.lock().unwrap();
            for (_, state) in routing_states.iter_mut() {
                state.current_route = "/".to_string();
                state.route_parameters.clear();
                state.breadcrumbs.clear();
            }
        }

        // Notify interface adapters
        if let Some(adapter) = self.interface_adapters.get(&interface_type) {
            adapter.cleanup()?;
        }

        Ok(())
    }

    /// Get current authentication state
    pub fn get_authentication_state(&self) -> AuthenticationState {
        self.authentication_state.lock().unwrap().clone()
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.authentication_state.lock().unwrap().current_session.is_some()
    }

    /// Get current user session
    pub fn get_current_session(&self) -> Option<UserSession> {
        self.authentication_state.lock().unwrap().current_session.clone()
    }

    /// Synchronize authentication state across all interfaces
    ///
    /// Updates the authentication state to reflect that the user is authenticated
    /// across all interface types for cross-interface session synchronization.
    pub fn synchronize_authentication_state(&self, username: &str) -> QmsResult<()> {
        let mut auth_state = self.authentication_state.lock().unwrap();

        // Mark user as authenticated across all interfaces
        auth_state.authenticated_interfaces.insert(InterfaceType::CLI);
        auth_state.authenticated_interfaces.insert(InterfaceType::Web);
        auth_state.authenticated_interfaces.insert(InterfaceType::TUI);

        // Update last activity
        auth_state.last_activity = Some(crate::utils::current_timestamp());

        // Log synchronization
        let _ = crate::modules::audit_logger::audit_log_action(
            "AUTH_SYNC",
            "Authentication",
            &format!("Cross-interface authentication synchronized for user: {}", username),
        );

        Ok(())
    }

    /// Set active project
    pub fn set_active_project(&self, project_path: PathBuf) -> QmsResult<()> {
        // Validate project
        let project_service = self.service_manager.project_service();
        let project_info = project_service.get_project_by_path(&project_path)?;

        // Update project context
        {
            let mut project_context = self.project_context.lock().unwrap();
            project_context.active_project_path = Some(project_path.clone());

            // Update project info in available projects
            let project_info_struct = ProjectInfo {
                id: project_info.id.clone(),
                name: project_info.name.clone(),
                path: project_path,
                metadata: ProjectMetadata {
                    description: project_info.description.clone(),
                    version: project_info.version.clone(),
                    created_at: crate::utils::current_timestamp(),
                    modified_at: crate::utils::current_timestamp(),
                    tags: Vec::new(),
                },
                last_accessed: Some(crate::utils::current_timestamp()),
            };

            // Add or update in available projects
            if let Some(existing_index) = project_context.available_projects.iter().position(|p| p.id == project_info.id) {
                project_context.available_projects[existing_index] = project_info_struct;
            } else {
                project_context.available_projects.push(project_info_struct);
            }
        }

        // Update routing states to reflect project change
        self.update_routing_for_project_change()?;

        Ok(())
    }

    /// Get current project path
    pub fn get_active_project_path(&self) -> Option<PathBuf> {
        self.project_context.lock().unwrap().active_project_path.clone()
    }

    /// Get available projects for current user
    pub fn get_available_projects(&self) -> Vec<ProjectInfo> {
        self.project_context.lock().unwrap().available_projects.clone()
    }

    /// Navigate to route in specific interface
    pub fn navigate_to(&self, interface_type: InterfaceType, route: &str, parameters: HashMap<String, String>) -> QmsResult<()> {
        // Update routing state
        {
            let mut routing_states = self.routing_states.lock().unwrap();
            if let Some(state) = routing_states.get_mut(&interface_type) {
                // Add to navigation history
                state.navigation_history.push(NavigationEntry {
                    route: state.current_route.clone(),
                    timestamp: crate::utils::current_timestamp(),
                    interface_type: interface_type.clone(),
                    context: state.route_parameters.clone(),
                });

                // Update current route
                state.current_route = route.to_string();
                state.route_parameters = parameters;

                // Update breadcrumbs
                self.update_breadcrumbs(state, route);
            }
        }

        // Notify interface adapter
        if let Some(adapter) = self.interface_adapters.get(&interface_type) {
            let routing_states = self.routing_states.lock().unwrap();
            if let Some(state) = routing_states.get(&interface_type) {
                adapter.update_routing(state)?;
            }
        }

        Ok(())
    }

    /// Get current route for interface
    pub fn get_current_route(&self, interface_type: InterfaceType) -> String {
        self.routing_states.lock().unwrap()
            .get(&interface_type)
            .map(|state| state.current_route.clone())
            .unwrap_or_else(|| "/".to_string())
    }

    /// Get navigation history for interface
    pub fn get_navigation_history(&self, interface_type: InterfaceType) -> Vec<NavigationEntry> {
        self.routing_states.lock().unwrap()
            .get(&interface_type)
            .map(|state| state.navigation_history.clone())
            .unwrap_or_default()
    }

    /// Update configuration
    pub fn update_configuration(&self, config: QmsConfig) -> QmsResult<()> {
        {
            let mut config_manager = self.configuration_manager.lock().unwrap();
            config_manager.global_config = config;
        }

        // Notify all interface adapters
        let config_manager = self.configuration_manager.lock().unwrap();
        for adapter in self.interface_adapters.values() {
            adapter.apply_configuration(&config_manager)?;
        }

        Ok(())
    }

    /// Get current configuration
    pub fn get_configuration(&self) -> QmsConfig {
        self.configuration_manager.lock().unwrap().global_config.clone()
    }

    /// Update user preferences
    pub fn update_user_preferences(&self, username: &str, preferences: UserPreferences) -> QmsResult<()> {
        {
            let mut config_manager = self.configuration_manager.lock().unwrap();
            config_manager.user_preferences.insert(username.to_string(), preferences);
        }

        // Save preferences to file
        self.save_user_preferences(username)?;

        Ok(())
    }

    /// Get user preferences
    pub fn get_user_preferences(&self, username: &str) -> Option<UserPreferences> {
        self.configuration_manager.lock().unwrap()
            .user_preferences.get(username).cloned()
    }

    // Private helper methods

    /// Get session type for interface
    fn get_session_type(&self, interface_type: InterfaceType) -> SessionType {
        match interface_type {
            InterfaceType::CLI => SessionType::CLI,
            InterfaceType::Web => SessionType::Web,
            InterfaceType::TUI => SessionType::CLI, // TUI uses CLI-style sessions
        }
    }

    /// Update project context for authenticated user
    fn update_project_context_for_user(&self, username: &str) -> QmsResult<()> {
        let project_service = self.service_manager.project_service();
        let available_projects = project_service.list_projects(username, None)?;

        {
            let mut project_context = self.project_context.lock().unwrap();
            project_context.available_projects = available_projects.into_iter().map(|p| ProjectInfo {
                id: p.id,
                name: p.name,
                path: p.path,
                metadata: ProjectMetadata {
                    description: p.description,
                    version: p.version,
                    created_at: crate::utils::current_timestamp(),
                    modified_at: crate::utils::current_timestamp(),
                    tags: Vec::new(),
                },
                last_accessed: None,
            }).collect();
        }

        Ok(())
    }

    /// Update routing states when project changes
    fn update_routing_for_project_change(&self) -> QmsResult<()> {
        let mut routing_states = self.routing_states.lock().unwrap();

        for (interface_type, state) in routing_states.iter_mut() {
            match interface_type {
                InterfaceType::CLI => {
                    state.current_route = "/project".to_string();
                    state.available_commands = vec![
                        "doc".to_string(),
                        "risk".to_string(),
                        "user".to_string(),
                        "project".to_string(),
                    ];
                }
                InterfaceType::Web => {
                    state.current_route = "/dashboard".to_string();
                    state.available_commands = vec![
                        "api/documents".to_string(),
                        "api/risks".to_string(),
                        "api/users".to_string(),
                        "api/projects".to_string(),
                    ];
                }
                InterfaceType::TUI => {
                    state.current_route = "/main_menu".to_string();
                    state.available_commands = vec![
                        "documents".to_string(),
                        "risks".to_string(),
                        "users".to_string(),
                        "settings".to_string(),
                    ];
                }
            }
        }

        Ok(())
    }

    /// Update breadcrumbs for navigation
    fn update_breadcrumbs(&self, state: &mut RoutingState, route: &str) {
        state.breadcrumbs.clear();

        let parts: Vec<&str> = route.split('/').filter(|s| !s.is_empty()).collect();
        let mut current_path = String::new();

        // Add root breadcrumb
        state.breadcrumbs.push(Breadcrumb {
            label: "Home".to_string(),
            route: "/".to_string(),
            level: 0,
        });

        // Add breadcrumbs for each path segment
        for (index, part) in parts.iter().enumerate() {
            current_path.push('/');
            current_path.push_str(part);

            state.breadcrumbs.push(Breadcrumb {
                label: part.to_string(),
                route: current_path.clone(),
                level: (index + 1) as u32,
            });
        }
    }

    /// Save user preferences to file
    fn save_user_preferences(&self, username: &str) -> QmsResult<()> {
        let config_manager = self.configuration_manager.lock().unwrap();
        let prefs_path = config_manager.config_paths.user_prefs_dir.join(format!("{}.json", username));

        if let Some(preferences) = config_manager.user_preferences.get(username) {
            // In a real implementation, this would serialize preferences to JSON
            // For now, we'll just create the directory structure
            if let Some(parent) = prefs_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    QmsError::io_error(&format!("Failed to create preferences directory: {}", e))
                })?;
            }
        }

        Ok(())
    }
}

impl ConfigurationManager {
    /// Create new configuration manager
    pub fn new() -> QmsResult<Self> {
        let global_config = QmsConfig::new_default();
        let config_paths = ConfigurationPaths::default();

        Ok(Self {
            global_config,
            user_preferences: HashMap::new(),
            interface_settings: HashMap::new(),
            config_paths,
        })
    }
}

impl Default for ConfigurationPaths {
    fn default() -> Self {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());

        let qms_config_dir = PathBuf::from(home_dir).join(".qms");

        Self {
            global_config_path: qms_config_dir.join("config.json"),
            user_prefs_dir: qms_config_dir.join("preferences"),
            interface_settings_dir: qms_config_dir.join("interfaces"),
            session_storage_dir: qms_config_dir.join("sessions"),
        }
    }
}

impl RoutingState {
    /// Create default CLI routing state
    pub fn default_for_cli() -> Self {
        Self {
            current_route: "/".to_string(),
            navigation_history: Vec::new(),
            available_commands: vec![
                "init".to_string(),
                "login".to_string(),
                "help".to_string(),
                "version".to_string(),
            ],
            route_parameters: HashMap::new(),
            breadcrumbs: vec![Breadcrumb {
                label: "CLI".to_string(),
                route: "/".to_string(),
                level: 0,
            }],
        }
    }

    /// Create default Web routing state
    pub fn default_for_web() -> Self {
        Self {
            current_route: "/".to_string(),
            navigation_history: Vec::new(),
            available_commands: vec![
                "api/auth/login".to_string(),
                "api/auth/startup-state".to_string(),
                "api/health".to_string(),
            ],
            route_parameters: HashMap::new(),
            breadcrumbs: vec![Breadcrumb {
                label: "Home".to_string(),
                route: "/".to_string(),
                level: 0,
            }],
        }
    }

    /// Create default TUI routing state
    pub fn default_for_tui() -> Self {
        Self {
            current_route: "/login".to_string(),
            navigation_history: Vec::new(),
            available_commands: vec![
                "login".to_string(),
                "exit".to_string(),
                "help".to_string(),
            ],
            route_parameters: HashMap::new(),
            breadcrumbs: vec![Breadcrumb {
                label: "TUI".to_string(),
                route: "/".to_string(),
                level: 0,
            }],
        }
    }
}
