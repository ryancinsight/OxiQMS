// Services Module
// Centralized business services following SOLID principles

pub mod project_detection_service;

// Re-export main services for easy access
pub use project_detection_service::{
    ProjectDetectionService, ProjectDetectionResult, ProjectMetadata,
    get_project_detection_service, find_user_project, list_user_projects, validate_project_directory
};
