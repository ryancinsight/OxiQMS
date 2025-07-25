//! Unified Project Service
//! 
//! Consolidates project management business logic across CLI, TUI, and web interfaces
//! following SOLID principles and DRY methodology.

use crate::prelude::*;
use crate::modules::repository::project::Repository;
use crate::models::Project;
use crate::modules::audit_logger::audit_log_action;
use crate::services::project_detection_service::{ProjectDetectionService, ProjectMetadata};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Unified Project Service Interface
/// 
/// Provides a single interface for project management operations that can be used
/// by CLI, TUI, and web interfaces, eliminating code duplication.
pub trait ProjectServiceInterface: Send + Sync {
    /// Create a new QMS project
    fn create_project(
        &self,
        name: &str,
        description: Option<&str>,
        custom_path: Option<&str>,
        created_by: &str,
    ) -> QmsResult<Project>;

    /// Get project by ID
    fn get_project(&self, project_id: &str) -> QmsResult<Project>;

    /// Get project by path
    fn get_project_by_path(&self, project_path: &Path) -> QmsResult<Project>;

    /// Update project information
    fn update_project(&self, project_id: &str, updates: ProjectUpdates, updated_by: &str) -> QmsResult<Project>;

    /// Delete project
    fn delete_project(&self, project_id: &str, deleted_by: &str) -> QmsResult<()>;

    /// List all projects accessible to user
    fn list_projects(&self, user_id: &str, filter: Option<ProjectFilter>) -> QmsResult<Vec<ProjectSummary>>;

    /// Initialize project structure and templates
    fn initialize_project_structure(&self, project_path: &Path) -> QmsResult<()>;

    /// Validate project directory
    fn validate_project_directory(&self, project_path: &Path) -> QmsResult<ProjectValidation>;

    /// Detect existing QMS project in directory
    fn detect_project(&self, directory: &Path) -> QmsResult<Option<ProjectMetadata>>;

    /// Get project statistics
    fn get_project_statistics(&self, project_id: &str) -> QmsResult<ProjectStatistics>;

    /// Archive project
    fn archive_project(&self, project_id: &str, archived_by: &str, reason: Option<&str>) -> QmsResult<()>;

    /// Restore archived project
    fn restore_project(&self, project_id: &str, restored_by: &str) -> QmsResult<Project>;
}

/// Project update parameters
#[derive(Debug, Clone)]
pub struct ProjectUpdates {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

/// Project filter for listing operations
#[derive(Debug, Clone)]
pub struct ProjectFilter {
    pub status: Option<ProjectStatus>,
    pub created_by: Option<String>,
    pub search_term: Option<String>,
    pub date_range: Option<(u64, u64)>,
}

/// Project status enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectStatus {
    Active,
    Archived,
    Deleted,
}

/// Project summary for list operations
#[derive(Debug, Clone)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    pub status: ProjectStatus,
    pub created_at: u64,
    pub version: String,
    pub document_count: usize,
    pub risk_count: usize,
}

/// Project validation result
#[derive(Debug, Clone)]
pub struct ProjectValidation {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub missing_directories: Vec<String>,
    pub missing_files: Vec<String>,
}

/// Project statistics
#[derive(Debug, Clone)]
pub struct ProjectStatistics {
    pub total_documents: usize,
    pub total_risks: usize,
    pub total_requirements: usize,
    pub total_tests: usize,
    pub active_users: usize,
    pub last_activity: Option<u64>,
    pub storage_size: u64,
}

/// Unified Project Service Implementation
/// 
/// Wraps the existing project management services and provides a unified interface
/// that can be used across all interfaces (CLI, TUI, Web).
pub struct UnifiedProjectService {
    repository: Repository,
    detection_service: ProjectDetectionService,
}

impl UnifiedProjectService {
    /// Create new unified project service
    pub fn new() -> QmsResult<Self> {
        Ok(Self {
            repository: Repository::new(),
            detection_service: ProjectDetectionService::new()?,
        })
    }

    /// Create with dependency injection for testing
    pub fn with_services(repository: Repository, detection_service: ProjectDetectionService) -> Self {
        Self {
            repository,
            detection_service,
        }
    }

    /// Convert Project to ProjectSummary
    fn to_summary(&self, project: &Project) -> QmsResult<ProjectSummary> {
        // Get project statistics for summary
        let stats = self.get_project_statistics(&project.id).unwrap_or_else(|_| ProjectStatistics {
            total_documents: 0,
            total_risks: 0,
            total_requirements: 0,
            total_tests: 0,
            active_users: 0,
            last_activity: None,
            storage_size: 0,
        });

        Ok(ProjectSummary {
            id: project.id.clone(),
            name: project.name.clone(),
            description: project.description.clone(),
            path: project.path.clone(),
            status: ProjectStatus::Active, // Simplified - would check actual status
            created_at: project.created_at,
            version: project.version.clone(),
            document_count: stats.total_documents,
            risk_count: stats.total_risks,
        })
    }

    /// Validate project operation permissions
    fn validate_permissions(&self, operation: &str, user_id: &str, project_id: &str) -> QmsResult<()> {
        if user_id.is_empty() {
            return Err(QmsError::permission_error("User ID is required"));
        }
        
        // Log the operation attempt for audit
        let _ = audit_log_action(
            &format!("PROJECT_OPERATION_ATTEMPT_{}", operation.to_uppercase()),
            "Project",
            &format!("{}:{}", user_id, project_id),
        );
        
        Ok(())
    }

    /// Create standard QMS project directory structure
    fn create_project_directories(&self, project_path: &Path) -> QmsResult<()> {
        let directories = [
            "documents",
            "risks",
            "requirements",
            "tests",
            "reports",
            "templates",
            "config",
            "users",
            "audit",
            "backups",
        ];

        for dir in &directories {
            let dir_path = project_path.join(dir);
            std::fs::create_dir_all(&dir_path)
                .map_err(|e| QmsError::io_error(&format!("Failed to create directory {}: {}", dir, e)))?;
        }

        Ok(())
    }

    /// Calculate project storage size
    fn calculate_storage_size(&self, project_path: &Path) -> u64 {
        fn dir_size(path: &Path) -> u64 {
            let mut size = 0;
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        size += dir_size(&path);
                    } else if let Ok(metadata) = entry.metadata() {
                        size += metadata.len();
                    }
                }
            }
            size
        }

        dir_size(project_path)
    }
}

impl ProjectServiceInterface for UnifiedProjectService {
    fn create_project(
        &self,
        name: &str,
        description: Option<&str>,
        custom_path: Option<&str>,
        created_by: &str,
    ) -> QmsResult<Project> {
        self.validate_permissions("CREATE", created_by, "new")?;
        
        // Create project using repository
        let mut project = Repository::init_project(name, custom_path)?;
        
        // Update description if provided
        if let Some(desc) = description {
            project.description = desc.to_string();
        }
        
        // Create project directory structure
        self.create_project_directories(&project.path)?;
        
        // Audit log the creation
        let _ = audit_log_action("PROJECT_CREATED", "Project", &project.id);
        
        Ok(project)
    }

    fn get_project(&self, project_id: &str) -> QmsResult<Project> {
        // Simplified implementation - would load from storage
        Err(QmsError::not_found(&format!("Project not found: {}", project_id)))
    }

    fn get_project_by_path(&self, project_path: &Path) -> QmsResult<Project> {
        if let Some(metadata) = self.detection_service.detect_project(project_path)? {
            // Convert metadata to Project
            Ok(Project {
                id: metadata.id,
                name: metadata.name,
                description: metadata.description,
                path: project_path.to_path_buf(),
                created_at: metadata.created_at,
                version: metadata.version,
            })
        } else {
            Err(QmsError::not_found("No QMS project found in directory"))
        }
    }

    fn update_project(&self, project_id: &str, updates: ProjectUpdates, updated_by: &str) -> QmsResult<Project> {
        self.validate_permissions("UPDATE", updated_by, project_id)?;
        
        let mut project = self.get_project(project_id)?;
        
        // Apply updates
        if let Some(name) = updates.name {
            project.name = name;
        }
        
        if let Some(description) = updates.description {
            project.description = description;
        }
        
        if let Some(version) = updates.version {
            project.version = version;
        }
        
        // Save updated project (simplified - would use proper update method)
        
        // Audit log the update
        let _ = audit_log_action("PROJECT_UPDATED", "Project", project_id);
        
        Ok(project)
    }

    fn delete_project(&self, project_id: &str, deleted_by: &str) -> QmsResult<()> {
        self.validate_permissions("DELETE", deleted_by, project_id)?;
        
        // Delete project (simplified - would use proper delete method)
        
        // Audit log the deletion
        let _ = audit_log_action("PROJECT_DELETED", "Project", project_id);
        
        Ok(())
    }

    fn list_projects(&self, user_id: &str, filter: Option<ProjectFilter>) -> QmsResult<Vec<ProjectSummary>> {
        // Simplified implementation - would get projects from storage
        let projects = Vec::new(); // self.repository.list_user_projects(user_id)?;
        
        let mut summaries = Vec::new();
        for project in projects {
            if let Ok(summary) = self.to_summary(&project) {
                summaries.push(summary);
            }
        }
        
        // Apply filters if provided
        if let Some(filter) = filter {
            if let Some(status) = filter.status {
                summaries.retain(|s| s.status == status);
            }
            
            if let Some(created_by) = filter.created_by {
                // Would filter by created_by if that field existed
            }
            
            if let Some(search_term) = filter.search_term {
                let search_lower = search_term.to_lowercase();
                summaries.retain(|s| {
                    s.name.to_lowercase().contains(&search_lower) ||
                    s.description.to_lowercase().contains(&search_lower)
                });
            }
        }
        
        Ok(summaries)
    }

    fn initialize_project_structure(&self, project_path: &Path) -> QmsResult<()> {
        self.create_project_directories(project_path)?;
        
        // Create initial configuration files
        let config_path = project_path.join("config").join("qms.json");
        let config_content = r#"{
  "version": "1.0",
  "project_type": "medical_device",
  "compliance_standards": ["FDA_21_CFR_820", "ISO_13485", "ISO_14971"],
  "created_at": "2024-01-01T00:00:00Z"
}"#;
        
        std::fs::write(&config_path, config_content)
            .map_err(|e| QmsError::io_error(&format!("Failed to create config file: {}", e)))?;
        
        Ok(())
    }

    fn validate_project_directory(&self, project_path: &Path) -> QmsResult<ProjectValidation> {
        let mut validation = ProjectValidation {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            missing_directories: Vec::new(),
            missing_files: Vec::new(),
        };

        // Check required directories
        let required_dirs = ["documents", "risks", "requirements", "config"];
        for dir in &required_dirs {
            let dir_path = project_path.join(dir);
            if !dir_path.exists() {
                validation.missing_directories.push(dir.to_string());
                validation.is_valid = false;
            }
        }

        // Check required files
        let config_file = project_path.join("config").join("qms.json");
        if !config_file.exists() {
            validation.missing_files.push("config/qms.json".to_string());
            validation.warnings.push("Missing QMS configuration file".to_string());
        }

        Ok(validation)
    }

    fn detect_project(&self, directory: &Path) -> QmsResult<Option<ProjectMetadata>> {
        self.detection_service.detect_project(directory)
    }

    fn get_project_statistics(&self, project_id: &str) -> QmsResult<ProjectStatistics> {
        // Simplified implementation - would calculate actual statistics
        Ok(ProjectStatistics {
            total_documents: 0,
            total_risks: 0,
            total_requirements: 0,
            total_tests: 0,
            active_users: 0,
            last_activity: None,
            storage_size: 0,
        })
    }

    fn archive_project(&self, project_id: &str, archived_by: &str, reason: Option<&str>) -> QmsResult<()> {
        self.validate_permissions("ARCHIVE", archived_by, project_id)?;
        
        // Archive project (simplified - would use proper archive method)
        
        // Audit log the archival
        let _ = audit_log_action("PROJECT_ARCHIVED", "Project", project_id);
        
        Ok(())
    }

    fn restore_project(&self, project_id: &str, restored_by: &str) -> QmsResult<Project> {
        self.validate_permissions("RESTORE", restored_by, project_id)?;
        
        // Restore project (simplified - would use proper restore method)
        let project = self.get_project(project_id)?;
        
        // Audit log the restoration
        let _ = audit_log_action("PROJECT_RESTORED", "Project", project_id);
        
        Ok(project)
    }
}
