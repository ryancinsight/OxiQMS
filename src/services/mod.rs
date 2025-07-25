// Services Module
// Centralized business services following SOLID principles
// Consolidates business logic across CLI, TUI, and web interfaces

pub mod project_detection_service;

// Consolidated business logic services
pub mod document_service;
pub mod risk_service;
pub mod user_service;
pub mod project_service;
pub mod navigation_service;
pub mod data_access_service;
pub mod validation_service;
pub mod unified_service_manager;

// Re-export main services for easy access
pub use project_detection_service::{
    ProjectDetectionService, ProjectDetectionResult, ProjectMetadata,
    get_project_detection_service, find_user_project, list_user_projects, validate_project_directory
};

// Re-export consolidated services
pub use document_service::UnifiedDocumentService;
pub use risk_service::UnifiedRiskService;
pub use user_service::UnifiedUserService;
pub use project_service::UnifiedProjectService;
pub use navigation_service::{UnifiedNavigationService, NavigationServiceInterface};
pub use data_access_service::{UnifiedDataAccessService, DataAccessServiceInterface};
pub use validation_service::{UnifiedValidationService, ValidationServiceInterface};
pub use unified_service_manager::UnifiedServiceManager;
