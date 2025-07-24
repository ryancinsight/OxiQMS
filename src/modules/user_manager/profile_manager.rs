//! User Profile Manager
//! 
//! This module provides high-level operations for managing user profiles,
//! including QMS folder preferences, session management, and user settings.
//! Follows the Facade pattern to simplify profile operations.

use crate::prelude::*;
use crate::models::user_profile::UserProfile;
use crate::modules::user_manager::implementations::global_user_storage::GlobalUserStorage;
use crate::modules::audit_logger::audit_log_action;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// User profile manager for handling profile operations
pub struct UserProfileManager {
    storage: GlobalUserStorage,
}

impl UserProfileManager {
    /// Create new user profile manager
    pub fn new() -> QmsResult<Self> {
        let storage = GlobalUserStorage::new()?;
        Ok(UserProfileManager { storage })
    }
    
    /// Get or create user profile
    pub fn get_or_create_profile(&self, username: &str) -> QmsResult<UserProfile> {
        match self.storage.load_user_profile(username) {
            Ok(profile) => Ok(profile),
            Err(_) => {
                // Create new profile with defaults
                let profile = UserProfile::new(username)?;
                self.storage.save_user_profile(&profile)?;
                audit_log_action("USER_PROFILE_CREATED", "UserProfile", username)?;
                Ok(profile)
            }
        }
    }
    
    /// Update user profile
    pub fn update_profile(&self, profile: &UserProfile) -> QmsResult<()> {
        profile.validate()?;
        self.storage.save_user_profile(profile)?;
        audit_log_action("USER_PROFILE_UPDATED", "UserProfile", &profile.username)?;
        Ok(())
    }
    
    /// Set user's QMS folder path
    pub fn set_qms_folder(&self, username: &str, qms_path: PathBuf) -> QmsResult<()> {
        let mut profile = self.get_or_create_profile(username)?;
        profile.set_qms_folder_path(qms_path)?;
        self.update_profile(&profile)?;
        
        audit_log_action(
            "QMS_FOLDER_UPDATED", 
            "UserProfile", 
            &format!("{}:{}", username, profile.qms_folder_path.display())
        )?;
        
        Ok(())
    }
    
    /// Get user's QMS folder path
    pub fn get_qms_folder(&self, username: &str) -> QmsResult<PathBuf> {
        let profile = self.get_or_create_profile(username)?;
        Ok(profile.qms_folder_path)
    }
    
    /// Get user's projects directory
    pub fn get_projects_directory(&self, username: &str) -> QmsResult<PathBuf> {
        let profile = self.get_or_create_profile(username)?;
        Ok(profile.get_projects_directory())
    }
    
    /// Add project to user's recent projects
    pub fn add_recent_project(&self, username: &str, project_id: &str) -> QmsResult<()> {
        let mut profile = self.get_or_create_profile(username)?;
        profile.add_recent_project(project_id);
        self.update_profile(&profile)?;
        Ok(())
    }
    
    /// Get user's recent projects
    pub fn get_recent_projects(&self, username: &str) -> QmsResult<Vec<String>> {
        let profile = self.get_or_create_profile(username)?;
        Ok(profile.recent_projects)
    }
    
    /// Set user UI preference
    pub fn set_ui_preference(&self, username: &str, key: &str, value: &str) -> QmsResult<()> {
        let mut profile = self.get_or_create_profile(username)?;
        profile.set_ui_preference(key, value);
        self.update_profile(&profile)?;
        Ok(())
    }
    
    /// Get user UI preference
    pub fn get_ui_preference(&self, username: &str, key: &str, default: Option<&str>) -> QmsResult<Option<String>> {
        let profile = self.get_or_create_profile(username)?;
        Ok(profile.get_ui_preference(key, default))
    }
    
    /// Update user's last login timestamp
    pub fn update_last_login(&self, username: &str) -> QmsResult<()> {
        let mut profile = self.get_or_create_profile(username)?;
        profile.update_last_login();
        self.update_profile(&profile)?;
        
        audit_log_action("USER_LOGIN", "UserProfile", username)?;
        Ok(())
    }
    
    /// Set user session token for persistent authentication
    pub fn set_session_token(&self, username: &str, token: Option<String>) -> QmsResult<()> {
        let mut profile = self.get_or_create_profile(username)?;
        let action = if token.is_some() { "SESSION_TOKEN_SET" } else { "SESSION_TOKEN_CLEARED" };
        profile.set_session_token(token);
        self.update_profile(&profile)?;

        audit_log_action(action, "UserProfile", username)?;

        Ok(())
    }
    
    /// Get user session token
    pub fn get_session_token(&self, username: &str) -> QmsResult<Option<String>> {
        let profile = self.get_or_create_profile(username)?;
        Ok(profile.session_token)
    }
    
    /// Check if user has a valid QMS folder configured
    pub fn has_valid_qms_folder(&self, username: &str) -> QmsResult<bool> {
        let profile = self.get_or_create_profile(username)?;
        
        // Check if QMS folder exists and is accessible
        if profile.qms_folder_path.exists() {
            // Try to create a test file to verify write access
            let test_file = profile.qms_folder_path.join(".qms_access_test");
            match std::fs::write(&test_file, "test") {
                Ok(_) => {
                    let _ = std::fs::remove_file(&test_file); // Clean up
                    Ok(true)
                }
                Err(_) => Ok(false),
            }
        } else {
            // Try to create the directory
            match std::fs::create_dir_all(&profile.qms_folder_path) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        }
    }
    
    /// Initialize QMS folder structure for user
    pub fn initialize_qms_folder(&self, username: &str) -> QmsResult<()> {
        let profile = self.get_or_create_profile(username)?;
        let qms_path = &profile.qms_folder_path;
        
        // Create main QMS directory
        std::fs::create_dir_all(qms_path)?;
        
        // Create subdirectories
        let subdirs = [
            "projects",
            "templates",
            "exports",
            "backups",
            "temp",
        ];
        
        for subdir in &subdirs {
            std::fs::create_dir_all(qms_path.join(subdir))?;
        }
        
        // Create a welcome file
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let welcome_content = format!(
            r#"# Welcome to QMS - Medical Device Quality Management System

This is your personal QMS folder for user: {}

## Directory Structure:
- projects/    - Your QMS projects
- templates/   - Document templates
- exports/     - Generated reports and exports
- backups/     - Automatic backups
- temp/        - Temporary files

## Getting Started:
1. Create your first project using the QMS interface
2. Add documents, requirements, and risk assessments
3. Generate compliance reports

For more information, visit the QMS documentation.

Created: {} (timestamp)
"#,
            username,
            now
        );
        
        std::fs::write(qms_path.join("README.md"), welcome_content)?;
        
        audit_log_action(
            "QMS_FOLDER_INITIALIZED", 
            "UserProfile", 
            &format!("{}:{}", username, qms_path.display())
        )?;
        
        Ok(())
    }
    
    /// Get user profile summary for display
    pub fn get_profile_summary(&self, username: &str) -> QmsResult<ProfileSummary> {
        let profile = self.get_or_create_profile(username)?;
        
        Ok(ProfileSummary {
            username: profile.username.clone(),
            qms_folder_path: profile.qms_folder_path.clone(),
            projects_directory: profile.get_projects_directory(),
            last_login: profile.last_login,
            recent_projects_count: profile.recent_projects.len(),
            has_session_token: profile.session_token.is_some(),
            locale: profile.locale.clone(),
            theme: profile.theme.clone(),
            created_at: profile.created_at,
        })
    }
    
    /// Validate user's QMS setup
    pub fn validate_user_setup(&self, username: &str) -> QmsResult<SetupValidation> {
        let profile = self.get_or_create_profile(username)?;
        
        let mut validation = SetupValidation {
            username: username.to_string(),
            has_qms_folder: false,
            qms_folder_accessible: false,
            projects_directory_exists: false,
            has_recent_projects: false,
            setup_complete: false,
        };
        
        // Check QMS folder
        validation.has_qms_folder = !profile.qms_folder_path.as_os_str().is_empty();
        
        if validation.has_qms_folder {
            validation.qms_folder_accessible = self.has_valid_qms_folder(username)?;
            
            if validation.qms_folder_accessible {
                let projects_dir = profile.get_projects_directory();
                validation.projects_directory_exists = projects_dir.exists();
                validation.has_recent_projects = !profile.recent_projects.is_empty();
            }
        }
        
        validation.setup_complete = validation.has_qms_folder 
            && validation.qms_folder_accessible 
            && validation.projects_directory_exists;
        
        Ok(validation)
    }
    
    /// Reset user profile to defaults (keeping username)
    pub fn reset_profile(&self, username: &str) -> QmsResult<()> {
        let mut profile = UserProfile::new(username)?;
        profile.username = username.to_string(); // Ensure username is preserved
        
        self.update_profile(&profile)?;
        
        audit_log_action("USER_PROFILE_RESET", "UserProfile", username)?;
        Ok(())
    }
}

/// Profile summary for display purposes
#[derive(Debug, Clone)]
pub struct ProfileSummary {
    pub username: String,
    pub qms_folder_path: PathBuf,
    pub projects_directory: PathBuf,
    pub last_login: Option<u64>,
    pub recent_projects_count: usize,
    pub has_session_token: bool,
    pub locale: String,
    pub theme: String,
    pub created_at: u64,
}

/// Setup validation result
#[derive(Debug, Clone)]
pub struct SetupValidation {
    pub username: String,
    pub has_qms_folder: bool,
    pub qms_folder_accessible: bool,
    pub projects_directory_exists: bool,
    pub has_recent_projects: bool,
    pub setup_complete: bool,
}

impl SetupValidation {
    /// Get list of setup issues
    pub fn get_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();
        
        if !self.has_qms_folder {
            issues.push("QMS folder path not configured".to_string());
        } else if !self.qms_folder_accessible {
            issues.push("QMS folder is not accessible or writable".to_string());
        } else if !self.projects_directory_exists {
            issues.push("Projects directory does not exist".to_string());
        }
        
        issues
    }
    
    /// Get next setup step
    pub fn get_next_step(&self) -> Option<String> {
        if !self.has_qms_folder {
            Some("Configure QMS folder location".to_string())
        } else if !self.qms_folder_accessible {
            Some("Fix QMS folder permissions or choose different location".to_string())
        } else if !self.projects_directory_exists {
            Some("Initialize QMS folder structure".to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::audit_logger::{AuditConfig, initialize_audit_system};
    use std::env;

    /// Initialize audit system for tests - lightweight version to avoid hanging
    fn init_audit_for_test() {
        let temp_dir = env::temp_dir().join("qms_profile_test");
        let _ = std::fs::create_dir_all(&temp_dir);

        let audit_dir = temp_dir.join("audit");
        let _ = std::fs::create_dir_all(&audit_dir);

        let config = AuditConfig {
            project_path: temp_dir.to_string_lossy().to_string(),
            retention_days: 30,
            daily_rotation: false,
            max_file_size_mb: 10,
            require_checksums: false,
        };

        // Use a timeout-safe initialization approach
        let _ = initialize_audit_system(config);
    }

    #[test]
    fn test_profile_manager_creation() {
        init_audit_for_test();
        let manager = UserProfileManager::new().unwrap();
        let test_username = format!("testuser_creation_{}", std::process::id());
        let profile = manager.get_or_create_profile(&test_username).unwrap();
        assert_eq!(profile.username, test_username);
    }

    #[test]
    fn test_qms_folder_management() {
        init_audit_for_test();
        let manager = UserProfileManager::new().unwrap();
        let test_username = format!("testuser_folder_{}", std::process::id());
        let temp_dir = env::temp_dir().join("qms_test_profile");

        manager.set_qms_folder(&test_username, temp_dir.clone()).unwrap();
        let retrieved_path = manager.get_qms_folder(&test_username).unwrap();
        assert_eq!(retrieved_path, temp_dir);

        // Cleanup
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_recent_projects() {
        init_audit_for_test();
        let manager = UserProfileManager::new().unwrap();

        // Use a unique username to avoid conflicts with other tests
        let test_username = format!("testuser_recent_{}", std::process::id());

        // First, verify the profile starts empty
        let initial_recent = manager.get_recent_projects(&test_username).unwrap();
        assert_eq!(initial_recent.len(), 0);

        // Add first project
        manager.add_recent_project(&test_username, "project1").unwrap();
        let recent_after_first = manager.get_recent_projects(&test_username).unwrap();
        assert_eq!(recent_after_first.len(), 1);
        assert_eq!(recent_after_first[0], "project1");

        // Add second project
        manager.add_recent_project(&test_username, "project2").unwrap();
        let recent_after_second = manager.get_recent_projects(&test_username).unwrap();
        assert_eq!(recent_after_second.len(), 2);
        assert_eq!(recent_after_second[0], "project2"); // Most recent first
        assert_eq!(recent_after_second[1], "project1");
    }

    #[test]
    fn test_setup_validation() {
        init_audit_for_test();
        let manager = UserProfileManager::new().unwrap();

        // Use a truly unique username with timestamp to avoid conflicts with unified auth system
        let test_username = format!("testuser_validation_{}_{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        );

        // Create a profile with a non-existent QMS folder to ensure setup is incomplete
        let mut profile = crate::models::user_profile::UserProfile::new(&test_username).unwrap();

        // Use an absolute but non-existent path for testing
        let non_existent_path = if cfg!(windows) {
            std::path::PathBuf::from(format!("C:\\non_existent_test_path_{}\\qms",
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()))
        } else {
            std::path::PathBuf::from(format!("/tmp/non_existent_test_path_{}/qms",
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()))
        };

        profile.qms_folder_path = non_existent_path;
        manager.update_profile(&profile).unwrap();

        let validation = manager.validate_user_setup(&test_username).unwrap();

        assert!(!validation.setup_complete, "Setup should not be complete for user with non-existent QMS folder");
        assert!(!validation.get_issues().is_empty(), "Should have setup issues for user with non-existent QMS folder");
        assert!(validation.get_next_step().is_some(), "Should have next step for user with non-existent QMS folder");
    }
}
