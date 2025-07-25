//! Project Discovery and Selection Strategies
//! 
//! Interface-specific implementations for project discovery and selection
//! following the Strategy Pattern and SOLID principles.

use crate::prelude::*;
use crate::interfaces::{InterfaceType, InterfaceContext};
use crate::interfaces::unified_project_manager::{
    ProjectDiscoveryStrategy, ProjectSelectionStrategy, ProjectDiscoveryResult,
    ProjectValidationResult, DiscoveryErrorAction, ProjectSelectionPresentation,
    ProjectSelectionResult, NoProjectsAction, DiscoveryMethod, PresentationFormat,
    SelectionMethod
};
use crate::interfaces::unified_context::ProjectInfo;
use crate::modules::user_manager::UserSession;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::io::Write;

/// CLI Project Discovery Strategy
/// 
/// Implements CLI-specific project discovery using command-line arguments,
/// environment variables, and file system scanning.
pub struct CliProjectDiscoveryStrategy;

impl CliProjectDiscoveryStrategy {
    pub fn new() -> Self {
        Self
    }
    
    /// Scan directory for QMS projects
    fn scan_directory_for_projects(&self, dir: &Path) -> Vec<ProjectDiscoveryResult> {
        let mut results = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    let project_file = entry.path().join("project.json");
                    if project_file.exists() {
                        if let Ok(project_info) = self.parse_project_file(&project_file) {
                            results.push(ProjectDiscoveryResult {
                                project_info,
                                confidence_score: 0.9,
                                discovery_method: DiscoveryMethod::CurrentDirectory,
                                metadata: HashMap::new(),
                            });
                        }
                    }
                }
            }
        }
        
        results
    }
    
    /// Parse project.json file
    fn parse_project_file(&self, project_file: &Path) -> QmsResult<ProjectInfo> {
        let content = std::fs::read_to_string(project_file)
            .map_err(|e| QmsError::io_error(&format!("Failed to read project file: {}", e)))?;
        
        // Simplified JSON parsing - in real implementation would use proper JSON parser
        let project_dir = project_file.parent().unwrap_or(Path::new("."));
        
        Ok(ProjectInfo {
            id: format!("project_{}", project_dir.file_name().unwrap_or_default().to_string_lossy()),
            name: project_dir.file_name().unwrap_or_default().to_string_lossy().to_string(),
            path: project_dir.to_path_buf(),
            metadata: crate::interfaces::unified_context::ProjectMetadata {
                description: "QMS Project".to_string(),
                version: "1.0.0".to_string(),
                created_at: crate::utils::current_timestamp(),
                modified_at: crate::utils::current_timestamp(),
                tags: Vec::new(),
            },
            last_accessed: None,
        })
    }
}

impl ProjectDiscoveryStrategy for CliProjectDiscoveryStrategy {
    fn discover_projects(&self, user_session: &UserSession, _context: &InterfaceContext) -> QmsResult<Vec<ProjectDiscoveryResult>> {
        let mut results = Vec::new();
        
        // Strategy 1: Check current working directory
        if let Ok(current_dir) = std::env::current_dir() {
            let current_results = self.scan_directory_for_projects(&current_dir);
            results.extend(current_results);
        }
        
        // Strategy 2: Check user's home directory
        let search_paths = self.get_default_search_paths(user_session);
        for path in search_paths {
            if path.exists() {
                let path_results = self.scan_directory_for_projects(&path);
                results.extend(path_results);
            }
        }
        
        // Strategy 3: Check environment variable
        if let Ok(qms_project_path) = std::env::var("QMS_PROJECT_PATH") {
            let env_path = PathBuf::from(qms_project_path);
            if env_path.exists() {
                let env_results = self.scan_directory_for_projects(&env_path);
                results.extend(env_results.into_iter().map(|mut r| {
                    r.discovery_method = DiscoveryMethod::EnvironmentVariable;
                    r.confidence_score = 0.95;
                    r
                }));
            }
        }
        
        // Remove duplicates based on path
        results.sort_by(|a, b| a.project_info.path.cmp(&b.project_info.path));
        results.dedup_by(|a, b| a.project_info.path == b.project_info.path);
        
        Ok(results)
    }
    
    fn validate_project_location(&self, path: &Path, _context: &InterfaceContext) -> QmsResult<ProjectValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut suggested_fixes = Vec::new();
        
        // Check if path exists
        if !path.exists() {
            errors.push("Project path does not exist".to_string());
            suggested_fixes.push("Create the directory or check the path".to_string());
        }
        
        // Check if it's a directory
        if path.exists() && !path.is_dir() {
            errors.push("Project path is not a directory".to_string());
        }
        
        // Check for project.json
        let project_file = path.join("project.json");
        if !project_file.exists() {
            warnings.push("No project.json file found".to_string());
            suggested_fixes.push("Run 'qms init' to initialize the project".to_string());
        }
        
        // Check for required directories
        let required_dirs = ["documents", "risks", "requirements", "audit"];
        for dir in &required_dirs {
            if !path.join(dir).exists() {
                warnings.push(format!("Missing required directory: {}", dir));
                suggested_fixes.push(format!("Create directory: {}", dir));
            }
        }
        
        Ok(ProjectValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            suggested_fixes,
        })
    }
    
    fn get_default_search_paths(&self, user_session: &UserSession) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        // User's home directory
        if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            let home_path = PathBuf::from(home);
            paths.push(home_path.join("qms_projects"));
            paths.push(home_path.join("Documents").join("QMS"));
            paths.push(home_path.join(format!("qms_projects_{}", user_session.username)));
        }
        
        // System-wide QMS directory
        paths.push(PathBuf::from("/opt/qms/projects"));
        paths.push(PathBuf::from("C:\\QMS\\Projects"));
        
        paths
    }
    
    fn handle_discovery_error(&self, error: &QmsError, _context: &InterfaceContext) -> QmsResult<DiscoveryErrorAction> {
        println!("Project discovery error: {}", error);
        
        print!("Would you like to specify a project location manually? (y/n): ");
        std::io::stdout().flush().map_err(|e| QmsError::io_error(&format!("Failed to flush stdout: {}", e)))?;
        
        let mut response = String::new();
        std::io::stdin().read_line(&mut response).map_err(|e| QmsError::io_error(&format!("Failed to read response: {}", e)))?;
        
        if response.trim().to_lowercase().starts_with('y') {
            Ok(DiscoveryErrorAction::PromptManualLocation)
        } else {
            Ok(DiscoveryErrorAction::UseDefault)
        }
    }
}

/// CLI Project Selection Strategy
/// 
/// Implements CLI-specific project selection using command-line interface.
pub struct CliProjectSelectionStrategy;

impl CliProjectSelectionStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl ProjectSelectionStrategy for CliProjectSelectionStrategy {
    fn present_project_options(&self, projects: &[ProjectInfo], _context: &InterfaceContext) -> QmsResult<ProjectSelectionPresentation> {
        println!("\n📁 Available QMS Projects:");
        println!("─────────────────────────────");
        
        for (index, project) in projects.iter().enumerate() {
            println!("{}. {} ({})", 
                index + 1, 
                project.name, 
                project.path.display()
            );
            
            if !project.metadata.description.is_empty() {
                println!("   Description: {}", project.metadata.description);
            }
            
            if let Some(last_accessed) = project.last_accessed {
                let timestamp = std::time::UNIX_EPOCH + std::time::Duration::from_secs(last_accessed);
                if let Ok(datetime) = timestamp.duration_since(std::time::UNIX_EPOCH) {
                    println!("   Last accessed: {} seconds ago", datetime.as_secs());
                }
            }
            println!();
        }
        
        Ok(ProjectSelectionPresentation {
            projects: projects.to_vec(),
            format: PresentationFormat::List,
            display_data: HashMap::new(),
            default_selection: Some(0), // Default to first project
        })
    }
    
    fn collect_project_selection(&self, presentation: &ProjectSelectionPresentation) -> QmsResult<ProjectSelectionResult> {
        print!("Select a project (1-{}): ", presentation.projects.len());
        std::io::stdout().flush().map_err(|e| QmsError::io_error(&format!("Failed to flush stdout: {}", e)))?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).map_err(|e| QmsError::io_error(&format!("Failed to read input: {}", e)))?;
        
        let selection = input.trim().parse::<usize>()
            .map_err(|_| QmsError::validation_error("Invalid selection"))?;
        
        if selection == 0 || selection > presentation.projects.len() {
            return Err(QmsError::validation_error("Selection out of range"));
        }
        
        let selected_project = presentation.projects[selection - 1].clone();
        
        Ok(ProjectSelectionResult {
            selected_project,
            selection_method: SelectionMethod::UserSelection,
            preferences: HashMap::new(),
        })
    }
    
    fn confirm_project_selection(&self, selection: &ProjectSelectionResult, _context: &InterfaceContext) -> QmsResult<()> {
        println!("✅ Selected project: {} at {}", 
            selection.selected_project.name,
            selection.selected_project.path.display()
        );
        Ok(())
    }
    
    fn handle_no_projects(&self, _context: &InterfaceContext) -> QmsResult<NoProjectsAction> {
        println!("\n❌ No QMS projects found.");
        println!("What would you like to do?");
        println!("1. Create a new project");
        println!("2. Import an existing project");
        println!("3. Use demo project");
        println!("4. Exit");
        
        print!("Choice (1-4): ");
        std::io::stdout().flush().map_err(|e| QmsError::io_error(&format!("Failed to flush stdout: {}", e)))?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).map_err(|e| QmsError::io_error(&format!("Failed to read input: {}", e)))?;
        
        match input.trim() {
            "1" => Ok(NoProjectsAction::CreateNew),
            "2" => Ok(NoProjectsAction::ImportExisting),
            "3" => Ok(NoProjectsAction::UseDemoProject),
            "4" => Ok(NoProjectsAction::Exit),
            _ => {
                println!("Invalid choice, creating new project...");
                Ok(NoProjectsAction::CreateNew)
            }
        }
    }
}

/// Web Project Discovery Strategy
/// 
/// Implements web-specific project discovery using HTTP requests and user sessions.
pub struct WebProjectDiscoveryStrategy;

impl WebProjectDiscoveryStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl ProjectDiscoveryStrategy for WebProjectDiscoveryStrategy {
    fn discover_projects(&self, user_session: &UserSession, _context: &InterfaceContext) -> QmsResult<Vec<ProjectDiscoveryResult>> {
        // In a real web implementation, this would:
        // 1. Query database for user's projects
        // 2. Check user's accessible project directories
        // 3. Return projects with web-specific metadata
        
        let mut results = Vec::new();
        
        // Placeholder implementation
        let search_paths = self.get_default_search_paths(user_session);
        for path in search_paths {
            if path.exists() && path.join("project.json").exists() {
                let project_info = ProjectInfo {
                    id: format!("web_project_{}", path.file_name().unwrap_or_default().to_string_lossy()),
                    name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                    path: path.clone(),
                    metadata: crate::interfaces::unified_context::ProjectMetadata {
                        description: "Web-discovered QMS Project".to_string(),
                        version: "1.0.0".to_string(),
                        created_at: crate::utils::current_timestamp(),
                        modified_at: crate::utils::current_timestamp(),
                        tags: vec!["web".to_string()],
                    },
                    last_accessed: Some(crate::utils::current_timestamp()),
                };
                
                results.push(ProjectDiscoveryResult {
                    project_info,
                    confidence_score: 0.8,
                    discovery_method: DiscoveryMethod::ConfigurationFile,
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("interface".to_string(), "web".to_string());
                        meta
                    },
                });
            }
        }
        
        Ok(results)
    }
    
    fn validate_project_location(&self, path: &Path, _context: &InterfaceContext) -> QmsResult<ProjectValidationResult> {
        // Web-specific validation (similar to CLI but with web considerations)
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        if !path.exists() {
            errors.push("Project path does not exist".to_string());
        }
        
        if path.exists() && !path.is_dir() {
            errors.push("Project path is not a directory".to_string());
        }
        
        // Check web-specific requirements
        if !path.join("project.json").exists() {
            warnings.push("No project.json file found".to_string());
        }
        
        Ok(ProjectValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            suggested_fixes: vec!["Initialize project via web interface".to_string()],
        })
    }
    
    fn get_default_search_paths(&self, user_session: &UserSession) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        // Web-specific search paths
        if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            let home_path = PathBuf::from(home);
            paths.push(home_path.join("web_qms_projects"));
            paths.push(home_path.join(format!("qms_web_{}", user_session.username)));
        }
        
        paths
    }
    
    fn handle_discovery_error(&self, _error: &QmsError, _context: &InterfaceContext) -> QmsResult<DiscoveryErrorAction> {
        // Web interface would typically show an error dialog or redirect
        Ok(DiscoveryErrorAction::PromptManualLocation)
    }
}

/// Web Project Selection Strategy
/// 
/// Implements web-specific project selection using HTML forms and AJAX.
pub struct WebProjectSelectionStrategy;

impl WebProjectSelectionStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl ProjectSelectionStrategy for WebProjectSelectionStrategy {
    fn present_project_options(&self, projects: &[ProjectInfo], _context: &InterfaceContext) -> QmsResult<ProjectSelectionPresentation> {
        // In a real web implementation, this would generate HTML/JSON for the frontend
        Ok(ProjectSelectionPresentation {
            projects: projects.to_vec(),
            format: PresentationFormat::Cards,
            display_data: {
                let mut data = HashMap::new();
                data.insert("view_type".to_string(), "cards".to_string());
                data.insert("sortable".to_string(), "true".to_string());
                data
            },
            default_selection: projects.iter().position(|p| p.last_accessed.is_some()),
        })
    }
    
    fn collect_project_selection(&self, presentation: &ProjectSelectionPresentation) -> QmsResult<ProjectSelectionResult> {
        // In a real web implementation, this would process HTTP request data
        // For now, select the first project or the most recently accessed
        let selected_project = if let Some(default_idx) = presentation.default_selection {
            presentation.projects[default_idx].clone()
        } else {
            presentation.projects[0].clone()
        };
        
        Ok(ProjectSelectionResult {
            selected_project,
            selection_method: SelectionMethod::MostRecentlyUsed,
            preferences: HashMap::new(),
        })
    }
    
    fn confirm_project_selection(&self, _selection: &ProjectSelectionResult, _context: &InterfaceContext) -> QmsResult<()> {
        // Web confirmation would be handled via HTTP response
        Ok(())
    }
    
    fn handle_no_projects(&self, _context: &InterfaceContext) -> QmsResult<NoProjectsAction> {
        // Web interface would typically show a project creation wizard
        Ok(NoProjectsAction::CreateNew)
    }
}

/// TUI Project Discovery Strategy
/// 
/// Implements TUI-specific project discovery with terminal-based interface.
pub struct TuiProjectDiscoveryStrategy;

impl TuiProjectDiscoveryStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl ProjectDiscoveryStrategy for TuiProjectDiscoveryStrategy {
    fn discover_projects(&self, user_session: &UserSession, _context: &InterfaceContext) -> QmsResult<Vec<ProjectDiscoveryResult>> {
        // TUI discovery similar to CLI but with visual feedback
        let search_paths = self.get_default_search_paths(user_session);
        let mut results = Vec::new();
        
        println!("🔍 Scanning for QMS projects...");
        
        for (i, path) in search_paths.iter().enumerate() {
            print!("  [{}/{}] Checking: {} ", i + 1, search_paths.len(), path.display());
            
            if path.exists() && path.join("project.json").exists() {
                println!("✅");
                
                let project_info = ProjectInfo {
                    id: format!("tui_project_{}", path.file_name().unwrap_or_default().to_string_lossy()),
                    name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                    path: path.clone(),
                    metadata: crate::interfaces::unified_context::ProjectMetadata {
                        description: "TUI-discovered QMS Project".to_string(),
                        version: "1.0.0".to_string(),
                        created_at: crate::utils::current_timestamp(),
                        modified_at: crate::utils::current_timestamp(),
                        tags: vec!["tui".to_string()],
                    },
                    last_accessed: Some(crate::utils::current_timestamp()),
                };
                
                results.push(ProjectDiscoveryResult {
                    project_info,
                    confidence_score: 0.85,
                    discovery_method: DiscoveryMethod::HomeDirectory,
                    metadata: HashMap::new(),
                });
            } else {
                println!("❌");
            }
        }
        
        Ok(results)
    }
    
    fn validate_project_location(&self, path: &Path, _context: &InterfaceContext) -> QmsResult<ProjectValidationResult> {
        // TUI validation with visual feedback
        println!("🔍 Validating project location: {}", path.display());
        
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        if !path.exists() {
            errors.push("Project path does not exist".to_string());
            println!("  ❌ Path does not exist");
        } else {
            println!("  ✅ Path exists");
        }
        
        if path.exists() && !path.is_dir() {
            errors.push("Project path is not a directory".to_string());
            println!("  ❌ Not a directory");
        } else if path.exists() {
            println!("  ✅ Is a directory");
        }
        
        if !path.join("project.json").exists() {
            warnings.push("No project.json file found".to_string());
            println!("  ⚠️  No project.json file");
        } else {
            println!("  ✅ project.json found");
        }
        
        Ok(ProjectValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            suggested_fixes: vec!["Use TUI project initialization wizard".to_string()],
        })
    }
    
    fn get_default_search_paths(&self, user_session: &UserSession) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        // TUI-specific search paths
        if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
            let home_path = PathBuf::from(home);
            paths.push(home_path.join("tui_qms_projects"));
            paths.push(home_path.join(format!("qms_tui_{}", user_session.username)));
        }
        
        paths
    }
    
    fn handle_discovery_error(&self, error: &QmsError, _context: &InterfaceContext) -> QmsResult<DiscoveryErrorAction> {
        println!("❌ Discovery error: {}", error);
        println!("What would you like to do?");
        println!("  [R] Retry discovery");
        println!("  [M] Manual location entry");
        println!("  [D] Use default location");
        println!("  [E] Exit");
        
        print!("Choice: ");
        std::io::stdout().flush().map_err(|e| QmsError::io_error(&format!("Failed to flush stdout: {}", e)))?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).map_err(|e| QmsError::io_error(&format!("Failed to read input: {}", e)))?;
        
        match input.trim().to_uppercase().as_str() {
            "R" => Ok(DiscoveryErrorAction::Retry(HashMap::new())),
            "M" => Ok(DiscoveryErrorAction::PromptManualLocation),
            "D" => Ok(DiscoveryErrorAction::UseDefault),
            "E" => Ok(DiscoveryErrorAction::Exit),
            _ => Ok(DiscoveryErrorAction::UseDefault),
        }
    }
}

/// TUI Project Selection Strategy
/// 
/// Implements TUI-specific project selection with terminal-based menus.
pub struct TuiProjectSelectionStrategy;

impl TuiProjectSelectionStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl ProjectSelectionStrategy for TuiProjectSelectionStrategy {
    fn present_project_options(&self, projects: &[ProjectInfo], _context: &InterfaceContext) -> QmsResult<ProjectSelectionPresentation> {
        println!("╔══════════════════════════════════════╗");
        println!("║          Select QMS Project          ║");
        println!("╠══════════════════════════════════════╣");
        
        for (index, project) in projects.iter().enumerate() {
            println!("║ {}. {:<31} ║", index + 1, 
                if project.name.len() > 31 {
                    format!("{}...", &project.name[..28])
                } else {
                    project.name.clone()
                }
            );
            println!("║    {:<34} ║", 
                if project.path.to_string_lossy().len() > 34 {
                    format!("...{}", &project.path.to_string_lossy()[project.path.to_string_lossy().len()-31..])
                } else {
                    project.path.to_string_lossy().to_string()
                }
            );
            if index < projects.len() - 1 {
                println!("║                                      ║");
            }
        }
        
        println!("╚══════════════════════════════════════╝");
        
        Ok(ProjectSelectionPresentation {
            projects: projects.to_vec(),
            format: PresentationFormat::InteractiveMenu,
            display_data: HashMap::new(),
            default_selection: Some(0),
        })
    }
    
    fn collect_project_selection(&self, presentation: &ProjectSelectionPresentation) -> QmsResult<ProjectSelectionResult> {
        print!("Enter selection (1-{}): ", presentation.projects.len());
        std::io::stdout().flush().map_err(|e| QmsError::io_error(&format!("Failed to flush stdout: {}", e)))?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).map_err(|e| QmsError::io_error(&format!("Failed to read input: {}", e)))?;
        
        let selection = input.trim().parse::<usize>()
            .map_err(|_| QmsError::validation_error("Invalid selection"))?;
        
        if selection == 0 || selection > presentation.projects.len() {
            return Err(QmsError::validation_error("Selection out of range"));
        }
        
        let selected_project = presentation.projects[selection - 1].clone();
        
        Ok(ProjectSelectionResult {
            selected_project,
            selection_method: SelectionMethod::UserSelection,
            preferences: HashMap::new(),
        })
    }
    
    fn confirm_project_selection(&self, selection: &ProjectSelectionResult, _context: &InterfaceContext) -> QmsResult<()> {
        println!("╔══════════════════════════════════════╗");
        println!("║        Project Selected!             ║");
        println!("║                                      ║");
        println!("║  Name: {:<28} ║", 
            if selection.selected_project.name.len() > 28 {
                format!("{}...", &selection.selected_project.name[..25])
            } else {
                selection.selected_project.name.clone()
            }
        );
        println!("║  Path: {:<28} ║", 
            if selection.selected_project.path.to_string_lossy().len() > 28 {
                format!("...{}", &selection.selected_project.path.to_string_lossy()[selection.selected_project.path.to_string_lossy().len()-25..])
            } else {
                selection.selected_project.path.to_string_lossy().to_string()
            }
        );
        println!("║                                      ║");
        println!("║  Press Enter to continue...          ║");
        println!("╚══════════════════════════════════════╝");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).map_err(|e| QmsError::io_error(&format!("Failed to read input: {}", e)))?;
        
        Ok(())
    }
    
    fn handle_no_projects(&self, _context: &InterfaceContext) -> QmsResult<NoProjectsAction> {
        println!("╔══════════════════════════════════════╗");
        println!("║         No Projects Found            ║");
        println!("║                                      ║");
        println!("║  What would you like to do?          ║");
        println!("║                                      ║");
        println!("║  1. Create new project               ║");
        println!("║  2. Import existing project          ║");
        println!("║  3. Use demo project                 ║");
        println!("║  4. Exit                             ║");
        println!("║                                      ║");
        println!("╚══════════════════════════════════════╝");
        
        print!("Choice (1-4): ");
        std::io::stdout().flush().map_err(|e| QmsError::io_error(&format!("Failed to flush stdout: {}", e)))?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).map_err(|e| QmsError::io_error(&format!("Failed to read input: {}", e)))?;
        
        match input.trim() {
            "1" => Ok(NoProjectsAction::CreateNew),
            "2" => Ok(NoProjectsAction::ImportExisting),
            "3" => Ok(NoProjectsAction::UseDemoProject),
            "4" => Ok(NoProjectsAction::Exit),
            _ => Ok(NoProjectsAction::CreateNew),
        }
    }
}
