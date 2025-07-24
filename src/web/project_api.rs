// Project API Handler - Medical Device Quality Management System
// SOLID Principles Implementation:
// - Single Responsibility: Handles only project management API operations
// - Open/Closed: Extensible through strategy pattern for different project types
// - Liskov Substitution: Project handlers implement common interface
// - Interface Segregation: Focused interfaces for project operations
// - Dependency Inversion: Depends on project repository abstractions

use crate::error::QmsResult;
use crate::web::{HttpRequest, HttpResponse};
use crate::web::response::HttpStatus;
use crate::json_utils::JsonValue;
use crate::modules::repository::project::Repository;
use crate::models::Project;
use std::collections::HashMap;

/// Project API Handler
/// 
/// Implements SOLID principles:
/// - Single Responsibility: Manages project API endpoints only
/// - Open/Closed: Extensible through project type strategies
/// - Interface Segregation: Focused on project-specific operations
pub struct ProjectApiHandler;

/// Project Provider Trait (Interface Segregation Principle)
/// Abstracts project management for better testability and flexibility
pub trait ProjectProvider {
    fn create_project(&self, name: &str, description: Option<&str>, custom_path: Option<&str>) -> QmsResult<Project>;
    fn list_projects(&self) -> QmsResult<Vec<Project>>;
    fn get_project(&self, project_id: &str) -> QmsResult<Project>;
    fn delete_project(&self, project_id: &str) -> QmsResult<()>;
}

/// Medical Device Project Provider (Dependency Inversion Principle)
/// Concrete implementation that depends on repository abstractions
pub struct MedicalDeviceProjectProvider;

impl ProjectProvider for MedicalDeviceProjectProvider {
    fn create_project(&self, name: &str, description: Option<&str>, custom_path: Option<&str>) -> QmsResult<Project> {
        // Use the existing Repository::init_project method
        let mut project = Repository::init_project(name, custom_path)
            .map_err(|e| crate::error::QmsError::domain_error(&format!("Failed to create project: {}", e)))?;
        
        // Update description if provided
        if let Some(desc) = description {
            project.description = desc.to_string();
            // TODO: Save updated metadata
        }
        
        Ok(project)
    }
    
    fn list_projects(&self) -> QmsResult<Vec<Project>> {
        Repository::list_projects()
            .map_err(|e| crate::error::QmsError::domain_error(&format!("Failed to list projects: {}", e)))
    }
    
    fn get_project(&self, project_id: &str) -> QmsResult<Project> {
        let projects = self.list_projects()?;
        projects.into_iter()
            .find(|p| p.id == project_id)
            .ok_or_else(|| crate::error::QmsError::not_found(&format!("Project not found: {}", project_id)))
    }
    
    fn delete_project(&self, project_id: &str) -> QmsResult<()> {
        let project = self.get_project(project_id)?;
        
        // Remove project directory
        std::fs::remove_dir_all(&project.path)
            .map_err(|e| crate::error::QmsError::io_error(&format!("Failed to delete project directory: {}", e)))?;
        
        // Log audit entry
        crate::audit::log_audit(&format!("PROJECT_DELETED: {} (ID: {})", project.name, project.id));
        
        Ok(())
    }
}

impl ProjectApiHandler {


    /// Handle GET /api/projects - List all projects
    pub fn handle_list_projects(_request: &HttpRequest) -> QmsResult<HttpResponse> {
        let provider = MedicalDeviceProjectProvider;
        let projects = provider.list_projects()?;
        
        // Convert to JSON response
        let response_data = Self::serialize_project_list(&projects)?;
        let json_response = Self::create_json_response(response_data)?;
        Ok(json_response)
    }

    /// Handle POST /api/projects - Create a new project
    pub fn handle_create_project(request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Parse request body
        let body_str = String::from_utf8(request.body.clone())
            .map_err(|e| crate::error::QmsError::Parse(format!("Invalid UTF-8 in request body: {e}")))?;
        
        let request_data = Self::parse_json_request(&body_str)?;
        
        // Extract project parameters
        let name = Self::get_string_field(&request_data, "name")?;
        let description = Self::get_optional_string_field(&request_data, "description");
        let custom_path = Self::get_optional_string_field(&request_data, "custom_path");
        
        // Validate project name
        if !crate::models::Project::validate_name(&name) {
            return Err(crate::error::QmsError::validation_error("Project name must be 1-100 characters"));
        }
        
        let provider = MedicalDeviceProjectProvider;
        let project = provider.create_project(&name, description.as_deref(), custom_path.as_deref())?;
        
        // Convert to JSON response
        let response_data = Self::serialize_project(&project)?;
        let mut json_response = Self::create_json_response(response_data)?;
        json_response.status = HttpStatus::Created;
        Ok(json_response)
    }

    /// Handle GET /api/projects/{id} - Get project details
    pub fn handle_get_project(request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Extract project ID from path
        let project_id = Self::extract_project_id(&request.uri)?;
        
        let provider = MedicalDeviceProjectProvider;
        let project = provider.get_project(&project_id)?;
        
        // Convert to JSON response
        let response_data = Self::serialize_project(&project)?;
        let json_response = Self::create_json_response(response_data)?;
        Ok(json_response)
    }

    /// Handle DELETE /api/projects/{id} - Delete a project
    pub fn handle_delete_project(request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Extract project ID from path
        let project_id = Self::extract_project_id(&request.uri)?;
        
        let provider = MedicalDeviceProjectProvider;
        provider.delete_project(&project_id)?;
        
        // Return success response
        let mut response_data = HashMap::new();
        response_data.insert("message".to_string(), JsonValue::String("Project deleted successfully".to_string()));
        response_data.insert("project_id".to_string(), JsonValue::String(project_id));
        
        let json_response = Self::create_json_response(response_data)?;
        Ok(json_response)
    }

    /// Extract project ID from URI path
    fn extract_project_id(uri: &str) -> QmsResult<String> {
        let parts: Vec<&str> = uri.split('/').collect();
        if parts.len() >= 4 && parts[1] == "api" && parts[2] == "projects" {
            Ok(parts[3].to_string())
        } else {
            Err(crate::error::QmsError::validation_error("Invalid project ID in path"))
        }
    }

    /// Serialize project list to JSON
    fn serialize_project_list(projects: &[Project]) -> QmsResult<HashMap<String, JsonValue>> {
        let mut data = HashMap::new();
        
        let projects_json: Vec<JsonValue> = projects.iter().map(|project| {
            Self::project_to_json(project)
        }).collect();
        
        data.insert("projects".to_string(), JsonValue::Array(projects_json));
        data.insert("total_count".to_string(), JsonValue::Number(projects.len() as f64));
        
        Ok(data)
    }

    /// Serialize single project to JSON
    fn serialize_project(project: &Project) -> QmsResult<HashMap<String, JsonValue>> {
        let mut data = HashMap::new();
        data.insert("project".to_string(), Self::project_to_json(project));
        Ok(data)
    }

    /// Convert Project to JsonValue
    fn project_to_json(project: &Project) -> JsonValue {
        let mut project_obj = HashMap::new();
        project_obj.insert("id".to_string(), JsonValue::String(project.id.clone()));
        project_obj.insert("name".to_string(), JsonValue::String(project.name.clone()));
        project_obj.insert("description".to_string(), JsonValue::String(project.description.clone()));
        project_obj.insert("version".to_string(), JsonValue::String(project.version.clone()));
        project_obj.insert("path".to_string(), JsonValue::String(project.path.to_string_lossy().to_string()));
        project_obj.insert("created_at".to_string(), JsonValue::Number(project.created_at as f64));
        JsonValue::Object(project_obj)
    }

    /// Parse JSON request body
    fn parse_json_request(body: &str) -> QmsResult<HashMap<String, JsonValue>> {
        let json_value = JsonValue::parse_from_str(body)
            .map_err(|e| crate::error::QmsError::Parse(format!("Invalid JSON: {e}")))?;
        
        match json_value {
            JsonValue::Object(obj) => Ok(obj),
            _ => Err(crate::error::QmsError::Parse("Expected JSON object".to_string())),
        }
    }

    /// Get required string field from JSON object
    fn get_string_field(data: &HashMap<String, JsonValue>, field: &str) -> QmsResult<String> {
        match data.get(field) {
            Some(JsonValue::String(s)) => Ok(s.clone()),
            Some(_) => Err(crate::error::QmsError::validation_error(&format!("Field '{}' must be a string", field))),
            None => Err(crate::error::QmsError::validation_error(&format!("Field '{}' is required", field))),
        }
    }

    /// Get optional string field from JSON object
    fn get_optional_string_field(data: &HashMap<String, JsonValue>, field: &str) -> Option<String> {
        match data.get(field) {
            Some(JsonValue::String(s)) => Some(s.clone()),
            _ => None,
        }
    }

    /// Create standardized JSON response
    fn create_json_response(data: HashMap<String, JsonValue>) -> QmsResult<HttpResponse> {
        let json_value = JsonValue::Object(data);
        let json_string = json_value.to_string();
        
        let mut response = HttpResponse::new(HttpStatus::Ok);
        response.set_content_type("application/json");
        response.set_body(json_string.as_bytes().to_vec());
        Ok(response)
    }
}
