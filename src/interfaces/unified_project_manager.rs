//! Unified Project Location Management
//! 
//! Creates shared project discovery, selection, and management logic that all
//! interfaces (CLI, TUI, Web) can use following SOLID, DRY, and KISS principles.

use crate::prelude::*;
use crate::interfaces::{InterfaceType, InterfaceContext};
use crate::interfaces::unified_context::{UnifiedInterfaceContext, ProjectInfo, ProjectContext};
use crate::interfaces::project_strategies::{
    CliProjectDiscoveryStrategy, WebProjectDiscoveryStrategy, TuiProjectDiscoveryStrategy,
    CliProjectSelectionStrategy, WebProjectSelectionStrategy, TuiProjectSelectionStrategy
};
use crate::services::{unified_service_manager::ServiceManagerInterface, project_service::ProjectServiceInterface};
use crate::modules::user_manager::UserSession;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::collections::HashMap;

/// Unified Project Manager
/// 
/// Central coordinator for project discovery, selection, and management across
/// all interfaces. Implements Single Responsibility Principle by focusing solely
/// on project location and lifecycle management.
pub struct UnifiedProjectManager {
    /// Unified interface context
    context: Arc<UnifiedInterfaceContext>,
    
    /// Service manager for business operations
    service_manager: Arc<dyn ServiceManagerInterface>,
    
    /// Project discovery strategies for each interface
    discovery_strategies: HashMap<InterfaceType, Box<dyn ProjectDiscoveryStrategy>>,
    
    /// Project selection strategies for each interface
    selection_strategies: HashMap<InterfaceType, Box<dyn ProjectSelectionStrategy>>,
}

/// Project discovery strategy trait
/// 
/// Defines interface-specific project discovery behavior while maintaining
/// consistent core discovery logic (Strategy Pattern).
pub trait ProjectDiscoveryStrategy: Send + Sync {
    /// Discover projects for user in interface-specific way
    fn discover_projects(&self, user_session: &UserSession, context: &InterfaceContext) -> QmsResult<Vec<ProjectDiscoveryResult>>;
    
    /// Validate project location
    fn validate_project_location(&self, path: &Path, context: &InterfaceContext) -> QmsResult<ProjectValidationResult>;
    
    /// Get default project search paths for interface
    fn get_default_search_paths(&self, user_session: &UserSession) -> Vec<PathBuf>;
    
    /// Handle project discovery errors
    fn handle_discovery_error(&self, error: &QmsError, context: &InterfaceContext) -> QmsResult<DiscoveryErrorAction>;
}

/// Project selection strategy trait
/// 
/// Defines interface-specific project selection behavior.
pub trait ProjectSelectionStrategy: Send + Sync {
    /// Present available projects to user for selection
    fn present_project_options(&self, projects: &[ProjectInfo], context: &InterfaceContext) -> QmsResult<ProjectSelectionPresentation>;
    
    /// Collect user's project selection
    fn collect_project_selection(&self, presentation: &ProjectSelectionPresentation) -> QmsResult<ProjectSelectionResult>;
    
    /// Handle project selection confirmation
    fn confirm_project_selection(&self, selection: &ProjectSelectionResult, context: &InterfaceContext) -> QmsResult<()>;
    
    /// Handle no projects found scenario
    fn handle_no_projects(&self, context: &InterfaceContext) -> QmsResult<NoProjectsAction>;
}

/// Project discovery result
#[derive(Debug, Clone)]
pub struct ProjectDiscoveryResult {
    /// Project information
    pub project_info: ProjectInfo,
    
    /// Discovery confidence score (0.0 to 1.0)
    pub confidence_score: f32,
    
    /// Discovery method used
    pub discovery_method: DiscoveryMethod,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Project validation result
#[derive(Debug, Clone)]
pub struct ProjectValidationResult {
    /// Whether project is valid
    pub is_valid: bool,
    
    /// Validation errors if any
    pub errors: Vec<String>,
    
    /// Validation warnings
    pub warnings: Vec<String>,
    
    /// Suggested fixes
    pub suggested_fixes: Vec<String>,
}

/// Discovery method enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum DiscoveryMethod {
    /// Found in current working directory
    CurrentDirectory,
    
    /// Found in user's home directory
    HomeDirectory,
    
    /// Found in QMS default location
    QmsDefaultLocation,
    
    /// Found via environment variable
    EnvironmentVariable,
    
    /// Found via configuration file
    ConfigurationFile,
    
    /// Found via user input
    UserInput,
    
    /// Found via recent projects list
    RecentProjects,
}

/// Discovery error action
#[derive(Debug, Clone)]
pub enum DiscoveryErrorAction {
    /// Retry discovery with different parameters
    Retry(HashMap<String, String>),
    
    /// Prompt user for manual project location
    PromptManualLocation,
    
    /// Create new project
    CreateNewProject,
    
    /// Use default project location
    UseDefault,
    
    /// Exit/cancel operation
    Exit,
}

/// Project selection presentation
#[derive(Debug, Clone)]
pub struct ProjectSelectionPresentation {
    /// Available projects
    pub projects: Vec<ProjectInfo>,
    
    /// Presentation format (list, table, cards, etc.)
    pub format: PresentationFormat,
    
    /// Interface-specific display data
    pub display_data: HashMap<String, String>,
    
    /// Default selection index
    pub default_selection: Option<usize>,
}

/// Project selection result
#[derive(Debug, Clone)]
pub struct ProjectSelectionResult {
    /// Selected project
    pub selected_project: ProjectInfo,
    
    /// Selection method used
    pub selection_method: SelectionMethod,
    
    /// User preferences for future selections
    pub preferences: HashMap<String, String>,
}

/// Presentation format enumeration
#[derive(Debug, Clone)]
pub enum PresentationFormat {
    /// Simple list format
    List,
    
    /// Table format with columns
    Table,
    
    /// Card-based format
    Cards,
    
    /// Tree format for hierarchical projects
    Tree,
    
    /// Interactive menu
    InteractiveMenu,
}

/// Selection method enumeration
#[derive(Debug, Clone)]
pub enum SelectionMethod {
    /// User clicked/selected from list
    UserSelection,
    
    /// Default project was used
    DefaultProject,
    
    /// Most recently used project
    MostRecentlyUsed,
    
    /// Only one project available
    OnlyOption,
    
    /// Automatic selection based on context
    AutomaticSelection,
}

/// No projects action
#[derive(Debug, Clone)]
pub enum NoProjectsAction {
    /// Create new project
    CreateNew,
    
    /// Import existing project
    ImportExisting,
    
    /// Use demo/sample project
    UseDemoProject,
    
    /// Exit application
    Exit,
}

impl UnifiedProjectManager {
    /// Create new unified project manager
    pub fn new(
        context: Arc<UnifiedInterfaceContext>,
        service_manager: Arc<dyn ServiceManagerInterface>,
    ) -> Self {
        let mut discovery_strategies: HashMap<InterfaceType, Box<dyn ProjectDiscoveryStrategy>> = HashMap::new();
        let mut selection_strategies: HashMap<InterfaceType, Box<dyn ProjectSelectionStrategy>> = HashMap::new();
        
        // Initialize interface-specific strategies
        discovery_strategies.insert(InterfaceType::CLI, Box::new(CliProjectDiscoveryStrategy::new()));
        discovery_strategies.insert(InterfaceType::Web, Box::new(WebProjectDiscoveryStrategy::new()));
        discovery_strategies.insert(InterfaceType::TUI, Box::new(TuiProjectDiscoveryStrategy::new()));
        
        selection_strategies.insert(InterfaceType::CLI, Box::new(CliProjectSelectionStrategy::new()));
        selection_strategies.insert(InterfaceType::Web, Box::new(WebProjectSelectionStrategy::new()));
        selection_strategies.insert(InterfaceType::TUI, Box::new(TuiProjectSelectionStrategy::new()));
        
        Self {
            context,
            service_manager,
            discovery_strategies,
            selection_strategies,
        }
    }
    
    /// Discover and select project for interface
    pub fn discover_and_select_project(&self, interface_context: &InterfaceContext) -> QmsResult<ProjectInfo> {
        let interface_type = interface_context.interface_type.clone();
        
        // Get current user session
        let user_session = self.context.get_current_session()
            .ok_or_else(|| QmsError::validation_error("No user session available"))?;
        
        // Step 1: Discover available projects
        let discovery_strategy = self.discovery_strategies.get(&interface_type)
            .ok_or_else(|| QmsError::domain_error(&format!("No discovery strategy for interface: {:?}", interface_type)))?;
        
        let discovered_projects = discovery_strategy.discover_projects(&user_session, interface_context)?;
        
        // Step 2: Convert discovery results to project info
        let available_projects: Vec<ProjectInfo> = discovered_projects.into_iter()
            .map(|result| result.project_info)
            .collect();
        
        // Step 3: Handle project selection
        if available_projects.is_empty() {
            self.handle_no_projects_found(interface_context)
        } else if available_projects.len() == 1 {
            // Only one project available, select it automatically
            let project = available_projects.into_iter().next().unwrap();
            self.set_active_project(project.clone())?;
            Ok(project)
        } else {
            // Multiple projects available, let user select
            self.select_project_from_options(available_projects, interface_context)
        }
    }
    
    /// Set active project in unified context
    pub fn set_active_project(&self, project: ProjectInfo) -> QmsResult<()> {
        self.context.set_active_project(project.path.clone())?;
        
        // Log project selection
        let _ = crate::modules::audit_logger::audit_log_action(
            "PROJECT_SELECTED",
            "Project",
            &format!("Project: {} at {}", project.name, project.path.display()),
        );
        
        Ok(())
    }
    
    /// Get current active project
    pub fn get_active_project(&self) -> Option<PathBuf> {
        self.context.get_active_project_path()
    }
    
    /// Get available projects for current user
    pub fn get_available_projects(&self) -> Vec<ProjectInfo> {
        self.context.get_available_projects()
    }
    
    /// Create new project
    pub fn create_new_project(&self, name: &str, location: &Path, interface_context: &InterfaceContext) -> QmsResult<ProjectInfo> {
        let project_service = self.service_manager.project_service();

        // Create project using service
        let created_by = self.context.get_current_session().map(|s| s.username).unwrap_or_else(|| "system".to_string());
        let project_summary = project_service.create_project(name, None, Some(location.to_string_lossy().as_ref()), &created_by)?;
        
        // Convert to ProjectInfo
        let project_info = ProjectInfo {
            id: project_summary.id,
            name: project_summary.name,
            path: location.to_path_buf(),
            metadata: crate::interfaces::unified_context::ProjectMetadata {
                description: project_summary.description,
                version: "1.0.0".to_string(),
                created_at: crate::utils::current_timestamp(),
                modified_at: crate::utils::current_timestamp(),
                tags: Vec::new(),
            },
            last_accessed: Some(crate::utils::current_timestamp()),
        };
        
        // Set as active project
        self.set_active_project(project_info.clone())?;
        
        Ok(project_info)
    }
    
    /// Import existing project
    pub fn import_existing_project(&self, path: &Path, interface_context: &InterfaceContext) -> QmsResult<ProjectInfo> {
        let project_service = self.service_manager.project_service();
        
        // Validate project location
        let discovery_strategy = self.discovery_strategies.get(&interface_context.interface_type)
            .ok_or_else(|| QmsError::domain_error("No discovery strategy available"))?;
        
        let validation_result = discovery_strategy.validate_project_location(path, interface_context)?;
        
        if !validation_result.is_valid {
            return Err(QmsError::validation_error(&format!(
                "Invalid project location: {}",
                validation_result.errors.join(", ")
            )));
        }
        
        // Detect project information
        let project_detection = project_service.get_project_by_path(path)?;
        
        // Convert to ProjectInfo
        let project_info = ProjectInfo {
            id: project_detection.id,
            name: project_detection.name,
            path: path.to_path_buf(),
            metadata: crate::interfaces::unified_context::ProjectMetadata {
                description: project_detection.description,
                version: project_detection.version,
                created_at: crate::utils::current_timestamp(),
                modified_at: crate::utils::current_timestamp(),
                tags: Vec::new(),
            },
            last_accessed: Some(crate::utils::current_timestamp()),
        };
        
        // Set as active project
        self.set_active_project(project_info.clone())?;
        
        Ok(project_info)
    }
    
    // Private helper methods
    
    /// Handle no projects found scenario
    fn handle_no_projects_found(&self, interface_context: &InterfaceContext) -> QmsResult<ProjectInfo> {
        let selection_strategy = self.selection_strategies.get(&interface_context.interface_type)
            .ok_or_else(|| QmsError::domain_error("No selection strategy available"))?;
        
        let action = selection_strategy.handle_no_projects(interface_context)?;
        
        match action {
            NoProjectsAction::CreateNew => {
                // Prompt for new project details
                self.prompt_for_new_project(interface_context)
            }
            NoProjectsAction::ImportExisting => {
                // Prompt for existing project location
                self.prompt_for_existing_project(interface_context)
            }
            NoProjectsAction::UseDemoProject => {
                // Create demo project
                self.create_demo_project(interface_context)
            }
            NoProjectsAction::Exit => {
                Err(QmsError::domain_error("No project selected, exiting"))
            }
        }
    }
    
    /// Select project from multiple options
    fn select_project_from_options(&self, projects: Vec<ProjectInfo>, interface_context: &InterfaceContext) -> QmsResult<ProjectInfo> {
        let selection_strategy = self.selection_strategies.get(&interface_context.interface_type)
            .ok_or_else(|| QmsError::domain_error("No selection strategy available"))?;
        
        // Present options to user
        let presentation = selection_strategy.present_project_options(&projects, interface_context)?;
        
        // Collect user selection
        let selection_result = selection_strategy.collect_project_selection(&presentation)?;
        
        // Confirm selection
        selection_strategy.confirm_project_selection(&selection_result, interface_context)?;
        
        // Set as active project
        self.set_active_project(selection_result.selected_project.clone())?;
        
        Ok(selection_result.selected_project)
    }
    
    /// Prompt for new project creation
    fn prompt_for_new_project(&self, interface_context: &InterfaceContext) -> QmsResult<ProjectInfo> {
        // This would be implemented with interface-specific prompting
        // For now, create a default project
        let default_location = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("new_qms_project");
        
        self.create_new_project("New QMS Project", &default_location, interface_context)
    }
    
    /// Prompt for existing project import
    fn prompt_for_existing_project(&self, interface_context: &InterfaceContext) -> QmsResult<ProjectInfo> {
        // This would be implemented with interface-specific prompting
        // For now, use current directory
        let current_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."));
        
        self.import_existing_project(&current_dir, interface_context)
    }
    
    /// Create demo project
    fn create_demo_project(&self, interface_context: &InterfaceContext) -> QmsResult<ProjectInfo> {
        let demo_location = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("qms_demo_project");
        
        self.create_new_project("QMS Demo Project", &demo_location, interface_context)
    }
}
