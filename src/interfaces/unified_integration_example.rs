//! Unified Interface Integration Example
//! 
//! Demonstrates how all unified interface components work together to provide
//! consistent functionality across CLI, TUI, and Web interfaces.

use crate::prelude::*;
use crate::interfaces::{
    InterfaceType, InterfaceContext,
    UnifiedInterfaceContext, UnifiedAuthFlow, UnifiedProjectManager, UnifiedRouter,
    UnifiedConfigManager, UnifiedValidationManager, FieldType
};
use crate::services::UnifiedServiceManager;
use std::sync::Arc;
use std::collections::HashMap;

/// Unified Interface Orchestrator
/// 
/// Demonstrates the complete integration of all unified interface components.
/// This serves as both an example and a template for implementing unified
/// interface functionality in the QMS system.
pub struct UnifiedInterfaceOrchestrator {
    /// Unified interface context
    context: Arc<UnifiedInterfaceContext>,
    
    /// Unified authentication flow
    auth_flow: UnifiedAuthFlow,
    
    /// Unified project manager
    project_manager: UnifiedProjectManager,
    
    /// Unified router
    router: UnifiedRouter,
    
    /// Unified configuration manager
    config_manager: UnifiedConfigManager,
    
    /// Unified validation manager
    validation_manager: UnifiedValidationManager,
    
    /// Service manager
    service_manager: Arc<UnifiedServiceManager>,
}

impl UnifiedInterfaceOrchestrator {
    /// Create new unified interface orchestrator
    pub fn new(project_path: Option<std::path::PathBuf>) -> QmsResult<Self> {
        // Initialize service manager
        let service_manager = Arc::new(UnifiedServiceManager::new(project_path.clone())?);
        
        // Initialize unified context
        let context = Arc::new(UnifiedInterfaceContext::new(service_manager.clone())?);
        
        // Initialize authentication flow
        let auth_flow = UnifiedAuthFlow::new(
            context.clone(),
            service_manager.clone(),
            service_manager.validation_service(),
        );
        
        // Initialize project manager
        let project_manager = UnifiedProjectManager::new(
            context.clone(),
            service_manager.clone(),
        );
        
        // Initialize router
        let router = UnifiedRouter::new(
            context.clone(),
            service_manager.clone(),
        );
        
        // Initialize configuration manager
        let config_manager = UnifiedConfigManager::new(project_path.clone())?;
        
        // Initialize validation manager
        let validation_manager = UnifiedValidationManager::new();
        
        Ok(Self {
            context,
            auth_flow,
            project_manager,
            router,
            config_manager,
            validation_manager,
            service_manager,
        })
    }
    
    /// Demonstrate complete user workflow across interfaces
    pub fn demonstrate_complete_workflow(&self) -> QmsResult<()> {
        println!("🚀 Starting Unified Interface Demonstration");
        println!("============================================\n");
        
        // Step 1: Demonstrate authentication across interfaces
        self.demonstrate_authentication()?;
        
        // Step 2: Demonstrate project management
        self.demonstrate_project_management()?;
        
        // Step 3: Demonstrate configuration management
        self.demonstrate_configuration_management()?;
        
        // Step 4: Demonstrate validation
        self.demonstrate_validation()?;
        
        // Step 5: Demonstrate routing
        self.demonstrate_routing()?;
        
        println!("✅ Unified Interface Demonstration Complete!\n");
        
        Ok(())
    }
    
    /// Demonstrate authentication across all interfaces
    fn demonstrate_authentication(&self) -> QmsResult<()> {
        println!("🔐 Authentication Demonstration");
        println!("-------------------------------");
        
        // Demonstrate CLI authentication
        println!("1. CLI Authentication:");
        let cli_context = InterfaceContext::new(InterfaceType::CLI);
        match self.auth_flow.authenticate(&cli_context) {
            Ok(result) => {
                println!("   ✅ CLI Authentication: {}", if result.success { "Success" } else { "Failed" });
                if let Some(session) = result.session {
                    println!("   👤 User: {}", session.username);
                }
            }
            Err(e) => println!("   ❌ CLI Authentication Error: {}", e),
        }
        
        // Demonstrate Web authentication
        println!("2. Web Authentication:");
        let web_context = InterfaceContext::new(InterfaceType::Web);
        match self.auth_flow.authenticate(&web_context) {
            Ok(result) => {
                println!("   ✅ Web Authentication: {}", if result.success { "Success" } else { "Failed" });
            }
            Err(e) => println!("   ❌ Web Authentication Error: {}", e),
        }
        
        // Demonstrate TUI authentication
        println!("3. TUI Authentication:");
        let tui_context = InterfaceContext::new(InterfaceType::TUI);
        match self.auth_flow.authenticate(&tui_context) {
            Ok(result) => {
                println!("   ✅ TUI Authentication: {}", if result.success { "Success" } else { "Failed" });
            }
            Err(e) => println!("   ❌ TUI Authentication Error: {}", e),
        }
        
        println!();
        Ok(())
    }
    
    /// Demonstrate project management
    fn demonstrate_project_management(&self) -> QmsResult<()> {
        println!("📁 Project Management Demonstration");
        println!("-----------------------------------");
        
        // Demonstrate project discovery for each interface
        let interfaces = vec![InterfaceType::CLI, InterfaceType::Web, InterfaceType::TUI];
        
        for interface_type in interfaces {
            println!("{}. {:?} Project Discovery:", 
                match interface_type {
                    InterfaceType::CLI => "1",
                    InterfaceType::Web => "2",
                    InterfaceType::TUI => "3",
                },
                interface_type
            );
            
            let interface_context = InterfaceContext::new(interface_type);
            match self.project_manager.discover_and_select_project(&interface_context) {
                Ok(project) => {
                    println!("   ✅ Project Found: {} at {}", project.name, project.path.display());
                    
                    // Set as active project
                    if let Err(e) = self.project_manager.set_active_project(project) {
                        println!("   ⚠️  Failed to set active project: {}", e);
                    }
                }
                Err(e) => println!("   ❌ Project Discovery Error: {}", e),
            }
        }
        
        // Show available projects
        let available_projects = self.project_manager.get_available_projects();
        println!("📋 Available Projects: {}", available_projects.len());
        for (i, project) in available_projects.iter().enumerate() {
            println!("   {}. {} ({})", i + 1, project.name, project.path.display());
        }
        
        println!();
        Ok(())
    }
    
    /// Demonstrate configuration management
    fn demonstrate_configuration_management(&self) -> QmsResult<()> {
        println!("⚙️  Configuration Management Demonstration");
        println!("------------------------------------------");
        
        // Load all configurations
        println!("1. Loading Configurations:");
        match self.config_manager.load_all_configurations() {
            Ok(()) => println!("   ✅ All configurations loaded successfully"),
            Err(e) => println!("   ❌ Configuration loading error: {}", e),
        }
        
        // Demonstrate configuration value access
        println!("2. Configuration Values:");
        let config_keys = vec!["theme", "language", "project_path", "auto_save"];
        for key in config_keys {
            if let Some(value) = self.config_manager.get_config_value(key) {
                println!("   {} = {}", key, value);
            } else {
                println!("   {} = <not set>", key);
            }
        }
        
        // Demonstrate configuration setting
        println!("3. Setting Configuration:");
        match self.config_manager.set_config_value("theme", "dark", crate::interfaces::unified_config_manager::ConfigSource::User) {
            Ok(()) => println!("   ✅ Theme set to 'dark'"),
            Err(e) => println!("   ❌ Failed to set theme: {}", e),
        }
        
        // Demonstrate user preferences
        println!("4. User Preferences:");
        if let Some(session) = self.context.get_current_session() {
            if let Some(prefs) = self.config_manager.get_user_preferences(&session.username) {
                println!("   👤 User: {}", session.username);
                println!("   🎨 Theme: {}", prefs.theme);
                println!("   🌐 Locale: {}", prefs.locale);
            }
        } else {
            println!("   ⚠️  No authenticated user for preferences");
        }
        
        println!();
        Ok(())
    }
    
    /// Demonstrate validation
    fn demonstrate_validation(&self) -> QmsResult<()> {
        println!("✅ Validation Demonstration");
        println!("---------------------------");
        
        // Test data for validation
        let test_cases = vec![
            ("username", "john_doe", FieldType::Username),
            ("username", "a", FieldType::Username), // Too short
            ("password", "password123", FieldType::Password),
            ("password", "weak", FieldType::Password), // Too weak
            ("email", "user@example.com", FieldType::Email),
            ("email", "invalid-email", FieldType::Email), // Invalid format
            ("project_name", "My QMS Project", FieldType::ProjectName),
            ("project_name", "Invalid/Project*Name", FieldType::ProjectName), // Invalid chars
        ];
        
        for (field_name, value, field_type) in test_cases {
            println!("{}. Validating {} = '{}':", 
                match field_name {
                    "username" => if value == "john_doe" { "1" } else { "2" },
                    "password" => if value == "password123" { "3" } else { "4" },
                    "email" => if value == "user@example.com" { "5" } else { "6" },
                    "project_name" => if value == "My QMS Project" { "7" } else { "8" },
                    _ => "?",
                },
                field_name, value
            );
            
            let result = self.validation_manager.validate_field(field_name, value, field_type, InterfaceType::CLI);
            
            if result.is_valid {
                println!("   ✅ Valid");
            } else {
                println!("   ❌ Invalid:");
                for error in &result.errors {
                    println!("      • {}", error.message);
                }
            }
            
            if !result.warnings.is_empty() {
                println!("   ⚠️  Warnings:");
                for warning in &result.warnings {
                    println!("      • {}", warning.message);
                }
            }
        }
        
        println!();
        Ok(())
    }
    
    /// Demonstrate routing
    fn demonstrate_routing(&self) -> QmsResult<()> {
        println!("🧭 Routing Demonstration");
        println!("------------------------");
        
        // Test commands for each interface
        let test_commands = vec![
            (InterfaceType::CLI, "help", vec![]),
            (InterfaceType::CLI, "status", vec![]),
            (InterfaceType::Web, "navigate", vec!["dashboard".to_string()]),
            (InterfaceType::TUI, "navigate", vec!["main_menu".to_string()]),
        ];
        
        for (i, (interface_type, command, args)) in test_commands.iter().enumerate() {
            println!("{}. {:?} Command: {} {}", 
                i + 1, interface_type, command, args.join(" ")
            );
            
            let interface_context = InterfaceContext::new(interface_type.clone());
            match self.router.route_command(&interface_context, command, args) {
                Ok(result) => {
                    println!("   ✅ Success: {}", result.message);
                    if let Some(next_action) = result.next_action {
                        println!("   ➡️  Next: {}", next_action);
                    }
                }
                Err(e) => println!("   ❌ Error: {}", e),
            }
        }
        
        // Show available commands
        println!("📋 Available Commands:");
        let available_commands = self.router.get_available_commands(InterfaceType::CLI, true);
        for (i, command) in available_commands.iter().enumerate() {
            println!("   {}. {}", i + 1, command);
        }
        
        println!();
        Ok(())
    }
    
    /// Get current system status
    pub fn get_system_status(&self) -> SystemStatus {
        SystemStatus {
            authenticated: self.context.is_authenticated(),
            current_user: self.context.get_current_session().map(|s| s.username),
            active_project: self.context.get_active_project_path(),
            available_projects: self.project_manager.get_available_projects().len(),
            configuration_loaded: true, // Would check actual state
            validation_enabled: true,
            routing_active: true,
        }
    }
}

/// System status information
#[derive(Debug, Clone)]
pub struct SystemStatus {
    /// Whether user is authenticated
    pub authenticated: bool,
    
    /// Current user if authenticated
    pub current_user: Option<String>,
    
    /// Active project path
    pub active_project: Option<std::path::PathBuf>,
    
    /// Number of available projects
    pub available_projects: usize,
    
    /// Whether configuration is loaded
    pub configuration_loaded: bool,
    
    /// Whether validation is enabled
    pub validation_enabled: bool,
    
    /// Whether routing is active
    pub routing_active: bool,
}

impl std::fmt::Display for SystemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "QMS Unified Interface System Status")?;
        writeln!(f, "===================================")?;
        writeln!(f, "Authentication: {}", if self.authenticated { "✅ Active" } else { "❌ Not authenticated" })?;
        
        if let Some(ref user) = self.current_user {
            writeln!(f, "Current User: {}", user)?;
        }
        
        if let Some(ref project) = self.active_project {
            writeln!(f, "Active Project: {}", project.display())?;
        } else {
            writeln!(f, "Active Project: None")?;
        }
        
        writeln!(f, "Available Projects: {}", self.available_projects)?;
        writeln!(f, "Configuration: {}", if self.configuration_loaded { "✅ Loaded" } else { "❌ Not loaded" })?;
        writeln!(f, "Validation: {}", if self.validation_enabled { "✅ Enabled" } else { "❌ Disabled" })?;
        writeln!(f, "Routing: {}", if self.routing_active { "✅ Active" } else { "❌ Inactive" })?;
        
        Ok(())
    }
}

/// Example usage function
pub fn run_unified_interface_example() -> QmsResult<()> {
    println!("🎯 QMS Unified Interface System Example");
    println!("=======================================\n");
    
    // Initialize the orchestrator
    let orchestrator = UnifiedInterfaceOrchestrator::new(None)?;
    
    // Run the complete demonstration
    orchestrator.demonstrate_complete_workflow()?;
    
    // Show final system status
    let status = orchestrator.get_system_status();
    println!("{}", status);
    
    Ok(())
}
