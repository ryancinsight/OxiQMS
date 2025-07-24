// CLI Authentication Helper
// Provides unified authentication for CLI commands using the same service as web
// Handles session persistence and user-specific project paths

use crate::prelude::*;
use crate::modules::user_manager::{FileBasedAuthService, UserSession, SessionType};
use crate::web::unified_auth_context::UnifiedAuthContext;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Write};

/// CLI Authentication Helper - manages authentication state for CLI commands
pub struct CliAuthHelper {
    auth_service: FileBasedAuthService,
    session_file_path: PathBuf,
}

impl CliAuthHelper {
    /// Create new CLI auth helper
    pub fn new() -> QmsResult<Self> {
        // Use global authentication service
        let auth_service = FileBasedAuthService::create_global()?;
        let qms_dir = Self::get_qms_directory()?;
        let session_file_path = qms_dir.join("cli_session.json");

        Ok(Self {
            auth_service,
            session_file_path,
        })
    }
    
    /// Get the global QMS directory
    fn get_qms_directory() -> QmsResult<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| QmsError::io_error("Cannot determine home directory"))?;
        
        let qms_dir = Path::new(&home).join(".qms");
        fs::create_dir_all(&qms_dir)?;
        
        Ok(qms_dir)
    }
    
    /// Check if user is currently logged in (has valid session)
    pub fn is_logged_in(&self) -> bool {
        if let Ok(session_id) = self.load_session_id() {
            self.auth_service.validate_session(&session_id).is_ok()
        } else {
            false
        }
    }
    
    /// Get current user session if logged in
    pub fn get_current_session(&self) -> QmsResult<UserSession> {
        let session_id = self.load_session_id()?;
        self.auth_service.validate_session(&session_id)
    }
    
    /// Login user and save session
    pub fn login(&self, username: &str, password: &str) -> QmsResult<UserSession> {
        let session = self.auth_service.cli_login(username, password)?;
        self.save_session_id(&session.session_id)?;
        Ok(session)
    }
    
    /// Interactive login with prompts
    pub fn interactive_login(&self) -> QmsResult<UserSession> {
        print!("Username: ");
        io::stdout().flush()?;
        let mut username = String::new();
        io::stdin().read_line(&mut username)?;
        let username = username.trim();
        
        print!("Password: ");
        io::stdout().flush()?;
        let mut password = String::new();
        io::stdin().read_line(&mut password)?;
        let password = password.trim();
        
        self.login(username, password)
    }
    
    /// Logout current user
    pub fn logout(&self) -> QmsResult<()> {
        if let Ok(session_id) = self.load_session_id() {
            let _ = self.auth_service.logout(&session_id);
        }
        self.clear_session_id()?;
        Ok(())
    }
    
    /// Require authentication - login if not already authenticated
    pub fn require_authentication(&self) -> QmsResult<UserSession> {
        if let Ok(session) = self.get_current_session() {
            return Ok(session);
        }
        
        println!("Authentication required. Please log in:");
        self.interactive_login()
    }
    
    /// Get user-specific project directory
    pub fn get_user_project_directory(&self, username: &str) -> QmsResult<PathBuf> {
        let qms_dir = Self::get_qms_directory()?;
        let user_dir = qms_dir.join(username).join("projects");
        fs::create_dir_all(&user_dir)?;
        Ok(user_dir)
    }
    
    /// Get current user's project directory
    pub fn get_current_user_project_directory(&self) -> QmsResult<PathBuf> {
        let session = self.get_current_session()?;
        self.get_user_project_directory(&session.username)
    }
    
    /// Find project path for current user (now uses unified project detection logic)
    pub fn find_user_project_path(&self, _project_name_or_id: Option<&str>) -> QmsResult<PathBuf> {
        let session = self.get_current_session()?;

        // Use the same unified project detection logic as UnifiedAuthContext
        // This ensures CLI and Web use identical project resolution
        UnifiedAuthContext::find_user_project_path(&session.username)
    }
    
    /// Create user if not exists (for initial setup)
    pub fn create_user_if_not_exists(&self, username: &str, password: &str) -> QmsResult<()> {
        match self.auth_service.get_user(username) {
            Ok(_) => {
                // User already exists
                Ok(())
            }
            Err(QmsError::NotFound(_)) => {
                // Create user with admin permissions for initial setup
                let admin_role = crate::models::Role {
                    name: "Admin".to_string(),
                    permissions: vec![
                        crate::models::Permission::UserManagement,
                        crate::models::Permission::ProjectManagement,
                        crate::models::Permission::SystemConfiguration,
                    ],
                };
                self.auth_service.create_user(username, password, vec![admin_role])?;
                println!("✓ User '{}' created successfully with admin privileges", username);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Check if any users exist in the system
    pub fn has_any_users(&self) -> bool {
        // Try to load the users file to see if any users exist
        let qms_dir = match Self::get_qms_directory() {
            Ok(dir) => dir,
            Err(_) => return false,
        };

        let users_file = qms_dir.join("users.json");
        if !users_file.exists() {
            return false;
        }

        // Try to read and parse the users file
        if let Ok(content) = std::fs::read_to_string(&users_file) {
            // Simple check - if the file contains any user entries
            content.contains("\"username\":")
        } else {
            false
        }
    }
    
    /// Save session ID to file
    fn save_session_id(&self, session_id: &str) -> QmsResult<()> {
        fs::write(&self.session_file_path, session_id)?;
        Ok(())
    }
    
    /// Load session ID from file
    fn load_session_id(&self) -> QmsResult<String> {
        let content = fs::read_to_string(&self.session_file_path)?;
        Ok(content.trim().to_string())
    }
    
    /// Clear saved session ID
    fn clear_session_id(&self) -> QmsResult<()> {
        if self.session_file_path.exists() {
            fs::remove_file(&self.session_file_path)?;
        }
        Ok(())
    }
}

/// Get CLI authentication helper (creates new instance each time to avoid unsafe code)
pub fn get_cli_auth_helper() -> QmsResult<CliAuthHelper> {
    CliAuthHelper::new()
}

/// Require authentication for CLI command
pub fn require_cli_authentication() -> QmsResult<UserSession> {
    let auth_helper = get_cli_auth_helper()?;
    auth_helper.require_authentication()
}

/// Check if CLI user is logged in
pub fn is_cli_logged_in() -> bool {
    if let Ok(auth_helper) = get_cli_auth_helper() {
        auth_helper.is_logged_in()
    } else {
        false
    }
}

/// Get current CLI user session
pub fn get_cli_session() -> QmsResult<UserSession> {
    let auth_helper = get_cli_auth_helper()?;
    auth_helper.get_current_session()
}

/// Get user-specific project path (replaces get_current_project_path for authenticated commands)
pub fn get_authenticated_project_path() -> QmsResult<PathBuf> {
    let session = get_cli_session()?;

    // Use unified project detection logic directly
    UnifiedAuthContext::find_user_project_path(&session.username)
}
