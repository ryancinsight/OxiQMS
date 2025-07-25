// Unified Project Detection Service
// Single source of truth for project detection and path resolution
// Follows SOLID, DRY, and Single Responsibility principles

use crate::prelude::*;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;

/// Unified Project Detection Service
/// Single responsibility: Detect and resolve project paths consistently across CLI and Web
pub struct ProjectDetectionService {
    qms_directory: PathBuf,
}

/// Project detection result
#[derive(Debug, Clone)]
pub struct ProjectDetectionResult {
    pub project_id: String,
    pub project_name: String,
    pub project_path: PathBuf,
    pub project_file: PathBuf,
    pub is_valid: bool,
    pub metadata: ProjectMetadata,
}

/// Project metadata extracted from project.json
#[derive(Debug, Clone)]
pub struct ProjectMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub created_at: u64,
    pub path: String,
}

impl ProjectDetectionService {
    /// Create new project detection service
    pub fn new() -> QmsResult<Self> {
        let qms_directory = Self::get_qms_directory()?;
        Ok(Self { qms_directory })
    }
    
    /// Get the QMS base directory (~/.qms)
    fn get_qms_directory() -> QmsResult<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| QmsError::io_error("Cannot determine home directory"))?;
        
        let qms_dir = Path::new(&home).join(".qms");
        fs::create_dir_all(&qms_dir)?;
        Ok(qms_dir)
    }
    
    /// Find user's active project (primary method used by both CLI and Web)
    pub fn find_user_project(&self, username: &str, project_hint: Option<&str>) -> QmsResult<ProjectDetectionResult> {
        let user_projects_dir = self.get_user_projects_directory(username)?;
        
        // Strategy 1: If project hint provided, search by name or ID
        if let Some(hint) = project_hint {
            if let Ok(result) = self.find_project_by_hint(&user_projects_dir, hint) {
                return Ok(result);
            }
        }
        
        // Strategy 2: Check current working directory (for CLI compatibility)
        if let Ok(result) = self.find_project_in_current_directory(&user_projects_dir) {
            return Ok(result);
        }
        
        // Strategy 3: Find the most recently accessed project
        if let Ok(result) = self.find_most_recent_project(&user_projects_dir) {
            return Ok(result);
        }
        
        // Strategy 4: Find any valid project (fallback)
        self.find_any_valid_project(&user_projects_dir)
    }
    
    /// Get user's projects directory
    pub fn get_user_projects_directory(&self, username: &str) -> QmsResult<PathBuf> {
        let user_projects_dir = self.qms_directory.join(username).join("projects");
        fs::create_dir_all(&user_projects_dir)?;
        Ok(user_projects_dir)
    }
    
    /// List all projects for a user
    pub fn list_user_projects(&self, username: &str) -> QmsResult<Vec<ProjectDetectionResult>> {
        let user_projects_dir = self.get_user_projects_directory(username)?;
        let mut projects = Vec::new();
        
        if let Ok(entries) = fs::read_dir(&user_projects_dir) {
            for entry in entries.flatten() {
                let project_path = entry.path();
                if project_path.is_dir() {
                    if let Ok(result) = self.validate_project_directory(&project_path) {
                        projects.push(result);
                    }
                }
            }
        }
        
        // Sort by most recently created
        projects.sort_by(|a, b| b.metadata.created_at.cmp(&a.metadata.created_at));
        Ok(projects)
    }
    
    /// Check if a project exists and is valid
    pub fn is_valid_project(&self, project_path: &Path) -> bool {
        let project_file = project_path.join("project.json");
        project_file.exists() && project_path.join("documents").exists()
    }
    
    /// Validate and extract metadata from a project directory
    pub fn validate_project_directory(&self, project_path: &Path) -> QmsResult<ProjectDetectionResult> {
        let project_file = project_path.join("project.json");
        
        if !project_file.exists() {
            return Err(QmsError::not_found("project.json not found"));
        }
        
        let metadata = self.parse_project_metadata(&project_file)?;
        
        // Verify required directories exist
        let required_dirs = ["documents", "risks", "requirements", "audit", "config"];
        for dir in &required_dirs {
            if !project_path.join(dir).exists() {
                return Err(QmsError::validation_error(&format!("Required directory '{}' missing", dir)));
            }
        }
        
        Ok(ProjectDetectionResult {
            project_id: metadata.id.clone(),
            project_name: metadata.name.clone(),
            project_path: project_path.to_path_buf(),
            project_file,
            is_valid: true,
            metadata,
        })
    }
    
    /// Find project by name or ID hint
    fn find_project_by_hint(&self, user_projects_dir: &Path, hint: &str) -> QmsResult<ProjectDetectionResult> {
        if let Ok(entries) = fs::read_dir(user_projects_dir) {
            for entry in entries.flatten() {
                let project_path = entry.path();
                if project_path.is_dir() {
                    if let Ok(result) = self.validate_project_directory(&project_path) {
                        // Check if hint matches project ID or name
                        if result.metadata.id == hint || result.metadata.name == hint {
                            return Ok(result);
                        }
                    }
                }
            }
        }
        
        Err(QmsError::not_found(&format!("Project '{}' not found", hint)))
    }
    
    /// Find project in current working directory (CLI compatibility)
    fn find_project_in_current_directory(&self, user_projects_dir: &Path) -> QmsResult<ProjectDetectionResult> {
        if let Ok(current_dir) = std::env::current_dir() {
            let mut path = current_dir.as_path();
            
            // Search up the directory tree
            loop {
                let project_file = path.join("project.json");
                if project_file.exists() {
                    // Verify this project belongs to the current user
                    if path.starts_with(user_projects_dir) {
                        return self.validate_project_directory(path);
                    }
                }
                
                match path.parent() {
                    Some(parent) => path = parent,
                    None => break,
                }
            }
        }
        
        Err(QmsError::not_found("No project found in current directory tree"))
    }
    
    /// Find the most recently accessed project
    fn find_most_recent_project(&self, user_projects_dir: &Path) -> QmsResult<ProjectDetectionResult> {
        let mut most_recent: Option<ProjectDetectionResult> = None;
        let mut most_recent_time = 0u64;
        
        if let Ok(entries) = fs::read_dir(user_projects_dir) {
            for entry in entries.flatten() {
                let project_path = entry.path();
                if project_path.is_dir() {
                    if let Ok(result) = self.validate_project_directory(&project_path) {
                        if result.metadata.created_at > most_recent_time {
                            most_recent_time = result.metadata.created_at;
                            most_recent = Some(result);
                        }
                    }
                }
            }
        }
        
        most_recent.ok_or_else(|| QmsError::not_found("No recent project found"))
    }
    
    /// Find any valid project (fallback)
    fn find_any_valid_project(&self, user_projects_dir: &Path) -> QmsResult<ProjectDetectionResult> {
        if let Ok(entries) = fs::read_dir(user_projects_dir) {
            for entry in entries.flatten() {
                let project_path = entry.path();
                if project_path.is_dir() {
                    if let Ok(result) = self.validate_project_directory(&project_path) {
                        return Ok(result);
                    }
                }
            }
        }
        
        Err(QmsError::not_found("No valid QMS project found. Use 'qms init' to create a project."))
    }
    
    /// Detect project in a directory (compatibility method)
    pub fn detect_project(&self, directory: &Path) -> QmsResult<Option<ProjectMetadata>> {
        match self.validate_project_directory(directory) {
            Ok(result) => Ok(Some(result.metadata)),
            Err(_) => Ok(None),
        }
    }

    /// Parse project metadata from project.json
    fn parse_project_metadata(&self, project_file: &Path) -> QmsResult<ProjectMetadata> {
        let content = fs::read_to_string(project_file)?;
        
        // Simple JSON parsing for project metadata
        let id = self.extract_json_field(&content, "id")
            .ok_or_else(|| QmsError::parse_error("Project ID not found in project.json"))?;
        let name = self.extract_json_field(&content, "name")
            .ok_or_else(|| QmsError::parse_error("Project name not found in project.json"))?;
        let description = self.extract_json_field(&content, "description").unwrap_or_default();
        let version = self.extract_json_field(&content, "version").unwrap_or_else(|| "1.0".to_string());
        let path = self.extract_json_field(&content, "path").unwrap_or_default();
        
        // Parse created_at timestamp
        let created_at = self.extract_json_field(&content, "created_at")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        
        Ok(ProjectMetadata {
            id,
            name,
            description,
            version,
            created_at,
            path,
        })
    }
    
    /// Simple JSON field extraction
    fn extract_json_field(&self, json: &str, field: &str) -> Option<String> {
        // Look for "field": "value" pattern
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

impl Default for ProjectDetectionService {
    fn default() -> Self {
        Self::new().expect("Failed to create ProjectDetectionService")
    }
}

/// Get project detection service instance (creates new instance each time for safety)
pub fn get_project_detection_service() -> QmsResult<ProjectDetectionService> {
    ProjectDetectionService::new()
}

/// Convenience function: Find user's active project
pub fn find_user_project(username: &str, project_hint: Option<&str>) -> QmsResult<ProjectDetectionResult> {
    let service = get_project_detection_service()?;
    service.find_user_project(username, project_hint)
}

/// Convenience function: List user's projects
pub fn list_user_projects(username: &str) -> QmsResult<Vec<ProjectDetectionResult>> {
    let service = get_project_detection_service()?;
    service.list_user_projects(username)
}

/// Convenience function: Validate project directory
pub fn validate_project_directory(project_path: &Path) -> QmsResult<ProjectDetectionResult> {
    let service = get_project_detection_service()?;
    service.validate_project_directory(project_path)
}
