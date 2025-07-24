//! Project Service Adapter for Unified Interface System
//! 
//! This module provides shared project management services that can be used
//! across all interface types (CLI, web, TUI), eliminating code duplication
//! and providing consistent project operations.

use crate::prelude::*;
use crate::models::Project;
use crate::modules::repository::project::{Repository, RepositoryError};
use crate::interfaces::{InterfaceContext, CommandResult};
use crate::json_utils::JsonValue;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Project Service Adapter - Bridges existing Repository to unified interface system
pub struct ProjectServiceAdapter {
    // Using Arc<Mutex<>> for thread-safe access across interfaces
    cache: Arc<Mutex<HashMap<String, Project>>>,
}

impl ProjectServiceAdapter {
    /// Create new project service adapter
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new project
    pub fn create_project(
        &self,
        context: &InterfaceContext,
        name: &str,
        description: Option<&str>,
        custom_path: Option<&str>,
    ) -> QmsResult<CommandResult> {
        // Validate project name
        if !Project::validate_name(name) {
            return Ok(CommandResult::error("Invalid project name".to_string()));
        }

        // Create project using existing Repository
        match Repository::init_project(name, custom_path) {
            Ok(mut project) => {
                // Update description if provided
                if let Some(desc) = description {
                    project.description = desc.to_string();
                }

                // Cache the project
                {
                    let mut cache = self.cache.lock().unwrap();
                    cache.insert(project.id.clone(), project.clone());
                }

                // Create success response with project data
                let project_json = self.project_to_json(&project);
                Ok(CommandResult::success_with_data(
                    format!("Project '{}' created successfully", name),
                    project_json,
                ))
            }
            Err(RepositoryError::ProjectExists(_)) => {
                Ok(CommandResult::error(format!("Project '{}' already exists", name)))
            }
            Err(e) => {
                Ok(CommandResult::error(format!("Failed to create project: {}", e)))
            }
        }
    }

    /// List all projects
    pub fn list_projects(&self, context: &InterfaceContext) -> QmsResult<CommandResult> {
        match Repository::list_projects() {
            Ok(projects) => {
                // Update cache
                {
                    let mut cache = self.cache.lock().unwrap();
                    for project in &projects {
                        cache.insert(project.id.clone(), project.clone());
                    }
                }

                // Convert projects to JSON array
                let projects_json: Vec<JsonValue> = projects.iter()
                    .map(|p| self.project_to_json(p))
                    .collect();

                Ok(CommandResult::success_with_data(
                    format!("Found {} projects", projects.len()),
                    JsonValue::Array(projects_json),
                ))
            }
            Err(e) => {
                Ok(CommandResult::error(format!("Failed to list projects: {}", e)))
            }
        }
    }

    /// Get project by ID
    pub fn get_project(&self, context: &InterfaceContext, project_id: &str) -> QmsResult<CommandResult> {
        // Check cache first
        {
            let cache = self.cache.lock().unwrap();
            if let Some(project) = cache.get(project_id) {
                let project_json = self.project_to_json(project);
                return Ok(CommandResult::success_with_data(
                    format!("Project '{}' found", project.name),
                    project_json,
                ));
            }
        }

        // For now, search through all projects to find by ID
        // TODO: Implement proper load_project method in Repository
        match Repository::list_projects() {
            Ok(projects) => {
                if let Some(project) = projects.iter().find(|p| p.id == project_id) {
                    // Cache the project
                    {
                        let mut cache = self.cache.lock().unwrap();
                        cache.insert(project.id.clone(), project.clone());
                    }

                    let project_json = self.project_to_json(project);
                    Ok(CommandResult::success_with_data(
                        format!("Project '{}' found", project.name),
                        project_json,
                    ))
                } else {
                    Ok(CommandResult::error(format!("Project '{}' not found", project_id)))
                }
            }
            Err(e) => {
                Ok(CommandResult::error(format!("Failed to search projects: {}", e)))
            }
        }
    }

    /// Delete project
    pub fn delete_project(&self, context: &InterfaceContext, project_id: &str) -> QmsResult<CommandResult> {
        // For now, just remove from cache and return success
        // TODO: Implement proper delete_project method in Repository
        let project_name = {
            let mut cache = self.cache.lock().unwrap();
            if let Some(project) = cache.remove(project_id) {
                project.name
            } else {
                // Try to find in all projects
                match Repository::list_projects() {
                    Ok(projects) => {
                        if let Some(project) = projects.iter().find(|p| p.id == project_id) {
                            project.name.clone()
                        } else {
                            return Ok(CommandResult::error(format!("Project '{}' not found", project_id)));
                        }
                    }
                    Err(e) => {
                        return Ok(CommandResult::error(format!("Failed to search projects: {}", e)));
                    }
                }
            }
        };

        // TODO: Actually delete the project directory and files
        // For now, just report success
        Ok(CommandResult::success(format!("Project '{}' would be deleted (not implemented yet)", project_name)))
    }

    /// Get current project path
    pub fn get_current_project_path(&self, context: &InterfaceContext) -> QmsResult<CommandResult> {
        match context.project_path.as_ref() {
            Some(path) => {
                let path_json = JsonValue::String(path.to_string_lossy().to_string());
                Ok(CommandResult::success_with_data(
                    format!("Current project path: {}", path.display()),
                    path_json,
                ))
            }
            None => {
                Ok(CommandResult::error("No current project path set".to_string()))
            }
        }
    }

    /// Set current project path
    pub fn set_current_project_path(
        &self,
        context: &mut InterfaceContext,
        path: PathBuf,
    ) -> QmsResult<CommandResult> {
        // Validate that the path exists and contains a QMS project
        if !path.exists() {
            return Ok(CommandResult::error("Project path does not exist".to_string()));
        }

        let qms_config_path = path.join(".qms").join("config.json");
        if !qms_config_path.exists() {
            return Ok(CommandResult::error("Path does not contain a QMS project".to_string()));
        }

        // Update context
        context.project_path = Some(path.clone());

        Ok(CommandResult::success(format!("Current project path set to: {}", path.display())))
    }

    /// Convert Project to JSON
    fn project_to_json(&self, project: &Project) -> JsonValue {
        let mut json_obj = HashMap::new();
        json_obj.insert("id".to_string(), JsonValue::String(project.id.clone()));
        json_obj.insert("name".to_string(), JsonValue::String(project.name.clone()));
        json_obj.insert("description".to_string(), JsonValue::String(project.description.clone()));
        json_obj.insert("version".to_string(), JsonValue::String(project.version.clone()));
        json_obj.insert("path".to_string(), JsonValue::String(project.path.to_string_lossy().to_string()));
        json_obj.insert("created_at".to_string(), JsonValue::Number(project.created_at as f64));

        JsonValue::Object(json_obj)
    }
}

/// Shared Project Manager - High-level project management interface
pub struct SharedProjectManager {
    adapter: ProjectServiceAdapter,
    current_context: Arc<Mutex<Option<InterfaceContext>>>,
}

impl SharedProjectManager {
    /// Create new shared project manager
    pub fn new() -> Self {
        Self {
            adapter: ProjectServiceAdapter::new(),
            current_context: Arc::new(Mutex::new(None)),
        }
    }

    /// Initialize with context
    pub fn with_context(mut self, context: InterfaceContext) -> Self {
        {
            let mut current_context = self.current_context.lock().unwrap();
            *current_context = Some(context);
        }
        self
    }

    /// Create project with current context
    pub fn create_project(
        &self,
        name: &str,
        description: Option<&str>,
        custom_path: Option<&str>,
    ) -> QmsResult<CommandResult> {
        let context = self.get_current_context()?;
        self.adapter.create_project(&context, name, description, custom_path)
    }

    /// List projects with current context
    pub fn list_projects(&self) -> QmsResult<CommandResult> {
        let context = self.get_current_context()?;
        self.adapter.list_projects(&context)
    }

    /// Get project with current context
    pub fn get_project(&self, project_id: &str) -> QmsResult<CommandResult> {
        let context = self.get_current_context()?;
        self.adapter.get_project(&context, project_id)
    }

    /// Delete project with current context
    pub fn delete_project(&self, project_id: &str) -> QmsResult<CommandResult> {
        let context = self.get_current_context()?;
        self.adapter.delete_project(&context, project_id)
    }

    /// Get current project path
    pub fn get_current_project_path(&self) -> QmsResult<CommandResult> {
        let context = self.get_current_context()?;
        self.adapter.get_current_project_path(&context)
    }

    /// Set current project path
    pub fn set_current_project_path(&self, path: PathBuf) -> QmsResult<CommandResult> {
        let mut context_guard = self.current_context.lock().unwrap();
        if let Some(ref mut context) = *context_guard {
            self.adapter.set_current_project_path(context, path)
        } else {
            Err(QmsError::InvalidOperation("No context available".to_string()))
        }
    }

    /// Update context
    pub fn update_context(&self, context: InterfaceContext) {
        let mut current_context = self.current_context.lock().unwrap();
        *current_context = Some(context);
    }

    /// Get current context
    fn get_current_context(&self) -> QmsResult<InterfaceContext> {
        let context_guard = self.current_context.lock().unwrap();
        context_guard.clone().ok_or_else(|| {
            QmsError::InvalidOperation("No context available".to_string())
        })
    }
}

impl Default for SharedProjectManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Project Command Handler - Unified project command handling for all interfaces
pub struct ProjectCommandHandler {
    manager: SharedProjectManager,
}

impl ProjectCommandHandler {
    /// Create new project command handler
    pub fn new(manager: SharedProjectManager) -> Self {
        Self { manager }
    }
}

impl crate::interfaces::routing::CommandHandler for ProjectCommandHandler {
    fn execute(
        &self,
        context: &InterfaceContext,
        args: &[String],
    ) -> QmsResult<CommandResult> {
        // Update manager context
        self.manager.update_context(context.clone());

        // Parse command arguments
        if args.is_empty() {
            return Ok(CommandResult::error("No project command specified".to_string()));
        }

        match args[0].as_str() {
            "create" => {
                if args.len() < 2 {
                    return Ok(CommandResult::error("Project name required".to_string()));
                }
                let name = &args[1];
                let description = args.get(2).map(|s| s.as_str());
                let custom_path = args.get(3).map(|s| s.as_str());
                
                self.manager.create_project(name, description, custom_path)
            }
            "list" => {
                self.manager.list_projects()
            }
            "get" => {
                if args.len() < 2 {
                    return Ok(CommandResult::error("Project ID required".to_string()));
                }
                self.manager.get_project(&args[1])
            }
            "delete" => {
                if args.len() < 2 {
                    return Ok(CommandResult::error("Project ID required".to_string()));
                }
                self.manager.delete_project(&args[1])
            }
            "current" => {
                self.manager.get_current_project_path()
            }
            "set-path" => {
                if args.len() < 2 {
                    return Ok(CommandResult::error("Project path required".to_string()));
                }
                let path = PathBuf::from(&args[1]);
                self.manager.set_current_project_path(path)
            }
            _ => {
                Ok(CommandResult::error(format!("Unknown project command: {}", args[0])))
            }
        }
    }

    fn command_name(&self) -> &'static str {
        "project"
    }

    fn help_text(&self) -> &'static str {
        "Manage QMS projects: create, list, get, delete, current, set-path"
    }

    fn requires_auth(&self) -> bool {
        true
    }

    fn validate_args(&self, args: &[String]) -> QmsResult<()> {
        if args.is_empty() {
            return Err(QmsError::validation_error("No project command specified"));
        }

        match args[0].as_str() {
            "create" | "get" | "delete" | "set-path" => {
                if args.len() < 2 {
                    return Err(QmsError::validation_error("Additional argument required"));
                }
            }
            "list" | "current" => {
                // No additional arguments required
            }
            _ => {
                return Err(QmsError::validation_error(&format!("Unknown project command: {}", args[0])));
            }
        }

        Ok(())
    }
}
