//! Project repository management for QMS
//! Phase 2 infrastructure - project data persistence and management

#![allow(dead_code)] // Phase 2 infrastructure - project repository for document control

use crate::audit::log_audit;
use crate::json_utils::{calculate_checksum, JsonValue};
use crate::models::Project;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub enum RepositoryError {
    IoError(std::io::Error),
    InvalidInput(String),
    ProjectExists(String),
}

impl std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryError::IoError(e) => write!(f, "IO error: {e}"),
            RepositoryError::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
            RepositoryError::ProjectExists(name) => write!(f, "Project '{name}' already exists"),
        }
    }
}

impl From<std::io::Error> for RepositoryError {
    fn from(error: std::io::Error) -> Self {
        RepositoryError::IoError(error)
    }
}

impl std::error::Error for RepositoryError {}

pub struct Repository;

impl Repository {
    /// Create a new Repository instance
    pub fn new() -> Self {
        Self
    }

    /// Initialize project in a specific directory (for user-specific projects)
    pub fn init_project_in_directory(name: &str, base_directory: &std::path::Path) -> Result<Project, RepositoryError> {
        // Validate project name
        if !Project::validate_name(name) {
            return Err(RepositoryError::InvalidInput(
                "Project name must be 1-100 characters".to_string(),
            ));
        }

        // Generate project ID (UUID v4 format - simplified for std-only)
        let project_id = Self::generate_project_id();
        let project_path = base_directory.join(&project_id);

        // Check if project already exists
        if project_path.exists() {
            return Err(RepositoryError::ProjectExists(name.to_string()));
        }

        // Create directory structure
        Self::create_project_structure(&project_path)?;

        // Create project metadata
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let project = Project {
            id: project_id,
            name: name.to_string(),
            description: "QMS Project".to_string(),
            version: "1.0".to_string(),
            path: project_path.clone(),
            created_at: now,
        };

        // Save project metadata
        Self::save_project_metadata(&project)?;

        Ok(project)
    }

    pub fn init_project(name: &str, custom_path: Option<&str>) -> Result<Project, RepositoryError> {
        // Validate project name
        if !Project::validate_name(name) {
            return Err(RepositoryError::InvalidInput(
                "Project name must be 1-100 characters".to_string(),
            ));
        }

        // Generate project ID (UUID v4 format - simplified for std-only)
        let project_id = Self::generate_project_id();

        // Determine base directory
        let base_path = if let Some(custom) = custom_path {
            PathBuf::from(custom)
        } else {
            Self::get_default_qms_path()?
        };

        let project_path = base_path.join(&project_id);

        // Check if project already exists
        if project_path.exists() {
            return Err(RepositoryError::ProjectExists(name.to_string()));
        }

        // Create directory structure
        Self::create_project_structure(&project_path)?;

        // Create project metadata
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let project = Project {
            id: project_id,
            name: name.to_string(),
            description: "QMS Project".to_string(),
            path: project_path.clone(),
            created_at: now,
            version: "1.0".to_string(),
        };

        // Persist project metadata
        Self::save_project_metadata(&project)?;

        // Create initial configuration
        Self::create_initial_config(&project_path)?;

        // Create empty index files
        Self::create_empty_indices(&project_path)?;

        // Log audit entry
        log_audit(&format!("PROJECT_CREATED: {} (ID: {})", name, project.id));

        Ok(project)
    }

    fn generate_project_id() -> String {
        // Simplified UUID v4 generation using timestamp and random-like hash
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create a pseudo-random string using hash of timestamp
        let hash_input = format!("{}_{}", now, std::process::id());
        let checksum = calculate_checksum(&hash_input);

        // Format as UUID-like string
        format!(
            "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
            (now & 0xFFFFFFFF) as u32,
            ((now >> 32) & 0xFFFF) as u16,
            (checksum.len() as u64 & 0xFFF) as u16,
            (std::process::id() & 0xFFFF) as u16,
            (now.wrapping_add(checksum.len() as u64) & 0xFFFFFFFFFFFF) as u64
        )
    }

    fn get_default_qms_path() -> Result<PathBuf, RepositoryError> {
        let home_dir = if cfg!(windows) {
            std::env::var("USERPROFILE")
                .or_else(|_| std::env::var("HOME"))
                .map_err(|_| {
                    RepositoryError::InvalidInput("Cannot determine home directory".to_string())
                })?
        } else {
            std::env::var("HOME").map_err(|_| {
                RepositoryError::InvalidInput("Cannot determine home directory".to_string())
            })?
        };

        Ok(PathBuf::from(home_dir).join(".qms").join("projects"))
    }

    fn create_project_structure(project_path: &Path) -> Result<(), RepositoryError> {
        // Create main project directory
        fs::create_dir_all(project_path)?;

        // Create subdirectories
        let subdirs = [
            "documents",
            "documents/versions",
            "documents/templates",
            "risks",
            "requirements",
            "tests",
            "audit",
            "users",
            "reports",
            "config",
        ];

        for subdir in &subdirs {
            fs::create_dir_all(project_path.join(subdir))?;
        }

        // Set permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            // Set directory permissions to 0755
            let mut perms = fs::metadata(project_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(project_path, perms)?;

            // Set subdirectory permissions
            for subdir in &subdirs {
                let subdir_path = project_path.join(subdir);
                let mut perms = fs::metadata(&subdir_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&subdir_path, perms)?;
            }
        }

        Ok(())
    }

    fn save_project_metadata(project: &Project) -> Result<(), RepositoryError> {
        let project_file = project.path.join("project.json");

        // Create JSON manually
        let json_content = format!(
            r#"{{
  "version": "1.0",
  "data": {{
    "id": "{}",
    "name": "{}",
    "path": "{}",
    "created_at": {},
    "version": "{}"
  }}
}}"#,
            project.id,
            project.name.replace("\"", "\\\""), // Escape quotes
            project
                .path
                .to_string_lossy()
                .replace("\\", "\\\\")
                .replace("\"", "\\\""),
            project.created_at,
            project.version
        );

        fs::write(&project_file, json_content)?;

        // Set file permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&project_file)?.permissions();
            perms.set_mode(0o644);
            fs::set_permissions(&project_file, perms)?;
        }

        Ok(())
    }

    fn create_initial_config(project_path: &Path) -> Result<(), RepositoryError> {
        let config_file = project_path.join("config").join("config.json");

        let config_content = r#"{
  "version": "1.0",
  "data": {
    "audit_retention_days": 2555,
    "document_auto_backup": true,
    "risk_approval_required": true,
    "user_session_timeout": 3600,
    "max_document_size_mb": 100,
    "compliance_mode": "FDA_21CFR820"
  }
}"#;

        fs::write(&config_file, config_content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&config_file)?.permissions();
            perms.set_mode(0o644);
            fs::set_permissions(&config_file, perms)?;
        }

        Ok(())
    }

    fn create_empty_indices(project_path: &Path) -> Result<(), RepositoryError> {
        let empty_index = r#"{
  "version": "1.0",
  "data": []
}"#;

        let index_files = [
            "documents/index.json",
            "risks/index.json",
            "requirements/index.json",
            "tests/index.json",
            "users/index.json",
        ];

        for index_file in &index_files {
            let file_path = project_path.join(index_file);
            fs::write(&file_path, empty_index)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&file_path)?.permissions();
                perms.set_mode(0o644);
                fs::set_permissions(&file_path, perms)?;
            }
        }

        Ok(())
    }

    pub fn list_projects() -> Result<Vec<Project>, RepositoryError> {
        let projects_path = Self::get_default_qms_path()?;

        if !projects_path.exists() {
            return Ok(Vec::new());
        }

        let mut projects = Vec::new();

        for entry in fs::read_dir(&projects_path)? {
            let entry = entry?;
            let project_path = entry.path();

            if project_path.is_dir() {
                let project_file = project_path.join("project.json");
                if project_file.exists() {
                    match Self::load_project_metadata(&project_file) {
                        Ok(project) => projects.push(project),
                        Err(_) => continue, // Skip invalid projects
                    }
                }
            }
        }

        Ok(projects)
    }

    fn load_project_metadata(project_file: &Path) -> Result<Project, RepositoryError> {
        let content = fs::read_to_string(project_file)?;

        // Parse JSON manually (simplified)
        if let Ok(json) = JsonValue::parse(&content) {
            if let JsonValue::Object(root) = json {
                if let Some(JsonValue::Object(data)) = root.get("data") {
                    let id = match data.get("id") {
                        Some(JsonValue::String(s)) => s.clone(),
                        _ => {
                            return Err(RepositoryError::InvalidInput(
                                "Missing project id".to_string(),
                            ))
                        }
                    };

                    let name = match data.get("name") {
                        Some(JsonValue::String(s)) => s.clone(),
                        _ => {
                            return Err(RepositoryError::InvalidInput(
                                "Missing project name".to_string(),
                            ))
                        }
                    };

                    let description = match data.get("description") {
                        Some(JsonValue::String(s)) => s.clone(),
                        _ => "QMS Project".to_string(),
                    };

                    let path_str = match data.get("path") {
                        Some(JsonValue::String(s)) => s.clone(),
                        _ => {
                            return Err(RepositoryError::InvalidInput(
                                "Missing project path".to_string(),
                            ))
                        }
                    };

                    let created_at = match data.get("created_at") {
                        Some(JsonValue::Number(n)) => *n as u64,
                        _ => {
                            return Err(RepositoryError::InvalidInput(
                                "Missing created_at".to_string(),
                            ))
                        }
                    };

                    let version = match data.get("version") {
                        Some(JsonValue::String(s)) => s.clone(),
                        _ => "1.0".to_string(),
                    };

                    return Ok(Project {
                        id,
                        name,
                        description,
                        path: PathBuf::from(path_str),
                        created_at,
                        version,
                    });
                }
            }
        }

        Err(RepositoryError::InvalidInput(
            "Invalid project file format".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_project_id() {
        let id = Repository::generate_project_id();
        println!("Generated ID: {}", id);

        // Check basic format (should have 4 hyphens)
        assert_eq!(id.matches('-').count(), 4);
        assert!(id.len() >= 30); // Minimum reasonable length
    }

    #[test]
    fn test_validate_project_name() {
        assert!(Project::validate_name("Valid Project"));
        assert!(Project::validate_name("A"));
        assert!(!Project::validate_name(""));
        assert!(!Project::validate_name(&"x".repeat(101))); // Too long
    }
}
