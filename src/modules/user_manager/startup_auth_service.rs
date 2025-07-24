//! Startup Authentication Service
//! 
//! This module handles the user-first authentication flow during application startup.
//! It detects if users exist, manages initial admin setup, and coordinates the
//! authentication workflow according to medical device compliance requirements.
//! 
//! SOLID Principles:
//! - Single Responsibility: Handles only startup authentication logic
//! - Open/Closed: Extensible for different authentication strategies
//! - Liskov Substitution: Uses abstractions for storage and authentication
//! - Interface Segregation: Focused interfaces for startup operations
//! - Dependency Inversion: Depends on abstractions, not concrete implementations

use crate::prelude::*;
use crate::models::user_profile::UserProfile;
use crate::modules::user_manager::implementations::global_user_storage::GlobalUserStorage;
use crate::modules::user_manager::interfaces::UserStorage;
use crate::modules::user_manager::profile_manager::{UserProfileManager, SetupValidation};
use crate::modules::audit_logger::audit_log_action;
use std::path::PathBuf;

/// Startup authentication service for managing initial system setup
pub struct StartupAuthService {
    user_storage: GlobalUserStorage,
    profile_manager: UserProfileManager,
}

/// Result of startup authentication check
#[derive(Debug, Clone)]
pub struct StartupAuthResult {
    pub requires_admin_setup: bool,
    pub has_existing_users: bool,
    pub system_initialized: bool,
    pub message: String,
}

/// Admin setup request data
#[derive(Debug, Clone)]
pub struct AdminSetupRequest {
    pub username: String,
    pub password: String,
    pub confirm_password: String,
    pub qms_folder_path: Option<PathBuf>,
}

/// QMS folder setup request data
#[derive(Debug, Clone)]
pub struct QmsFolderSetupRequest {
    pub username: String,
    pub qms_folder_path: PathBuf,
    pub use_default: bool,
}

impl StartupAuthService {
    /// Create new startup authentication service
    pub fn new() -> QmsResult<Self> {
        let user_storage = GlobalUserStorage::new()?;
        let profile_manager = UserProfileManager::new()?;

        Ok(StartupAuthService {
            user_storage,
            profile_manager,
        })
    }

    /// Create new startup authentication service with custom storage (for testing)
    /// SOLID: Dependency Inversion - allows injection of test storage
    #[cfg(test)]
    pub fn new_with_storage(user_storage: GlobalUserStorage, profile_manager: UserProfileManager) -> Self {
        StartupAuthService {
            user_storage,
            profile_manager,
        }
    }
    
    /// Check system startup authentication state
    /// GRASP: Information Expert - this service knows about system state
    pub fn check_startup_state(&self) -> QmsResult<StartupAuthResult> {
        let has_users = self.user_storage.has_any_users()?;
        
        if !has_users {
            // No users exist - require admin setup
            // Audit logging (optional - no project exists yet)
            if let Err(e) = audit_log_action("STARTUP_NO_USERS", "System", "admin_setup_required") {
                eprintln!("Warning: Could not log startup state: {}", e);
            }
            
            Ok(StartupAuthResult {
                requires_admin_setup: true,
                has_existing_users: false,
                system_initialized: false,
                message: "No users found. Admin setup required.".to_string(),
            })
        } else {
            // Users exist - system is initialized
            // Audit logging (optional - may not have project yet)
            if let Err(e) = audit_log_action("STARTUP_USERS_EXIST", "System", "system_initialized") {
                eprintln!("Warning: Could not log startup state: {}", e);
            }
            
            Ok(StartupAuthResult {
                requires_admin_setup: false,
                has_existing_users: true,
                system_initialized: true,
                message: "System initialized. Login required.".to_string(),
            })
        }
    }
    
    /// Validate admin setup request
    /// KISS: Simple validation with clear error messages
    pub fn validate_admin_setup(&self, request: &AdminSetupRequest) -> QmsResult<()> {
        // Check if users already exist
        if self.user_storage.has_any_users()? {
            return Err(QmsError::validation_error("Admin user already exists"));
        }
        
        // Validate username
        if request.username.len() < 3 || request.username.len() > 50 {
            return Err(QmsError::validation_error("Username must be 3-50 characters"));
        }
        
        if !request.username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(QmsError::validation_error("Username can only contain letters, numbers, and underscores"));
        }
        
        // Validate password
        if request.password.len() < 8 {
            return Err(QmsError::validation_error("Password must be at least 8 characters"));
        }
        
        if request.password != request.confirm_password {
            return Err(QmsError::validation_error("Passwords do not match"));
        }
        
        // Validate QMS folder path if provided
        if let Some(ref path) = request.qms_folder_path {
            if !path.is_absolute() {
                return Err(QmsError::validation_error("QMS folder path must be absolute"));
            }
            
            // Check if parent directory exists
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    return Err(QmsError::validation_error("Parent directory does not exist"));
                }
            }
        }
        
        Ok(())
    }
    
    /// Create initial admin user and setup system
    /// ACID: Ensures atomicity of admin creation and profile setup
    pub fn create_initial_admin(&self, request: &AdminSetupRequest) -> QmsResult<UserProfile> {
        // Validate request
        self.validate_admin_setup(request)?;
        
        // Create admin user
        let admin_user = self.user_storage.create_initial_admin(&request.username, &request.password)?;
        
        // Create user profile
        let mut profile = UserProfile::new(&request.username)?;
        
        // Set QMS folder path
        if let Some(ref qms_path) = request.qms_folder_path {
            profile.set_qms_folder_path(qms_path.clone())?;
        }
        
        // Save profile
        self.profile_manager.update_profile(&profile)?;
        
        // Initialize QMS folder structure
        self.profile_manager.initialize_qms_folder(&request.username)?;
        
        // Audit logging (optional - may not have project yet)
        if let Err(e) = audit_log_action(
            "INITIAL_ADMIN_SETUP_COMPLETE",
            "System",
            &format!("admin:{} qms_folder:{}", request.username, profile.qms_folder_path.display())
        ) {
            eprintln!("Warning: Could not log admin setup completion: {}", e);
        }
        
        Ok(profile)
    }
    
    /// Setup QMS folder for existing user
    /// DRY: Reuses profile manager functionality
    pub fn setup_qms_folder(&self, request: &QmsFolderSetupRequest) -> QmsResult<UserProfile> {
        // Validate that user exists
        if !self.user_storage.user_exists(&request.username)? {
            return Err(QmsError::not_found("User not found"));
        }
        
        // Get or create profile
        let mut profile = self.profile_manager.get_or_create_profile(&request.username)?;
        
        // Set QMS folder path
        profile.set_qms_folder_path(request.qms_folder_path.clone())?;
        
        // Update profile
        self.profile_manager.update_profile(&profile)?;
        
        // Initialize QMS folder structure
        self.profile_manager.initialize_qms_folder(&request.username)?;
        
        // Audit logging (optional - may not have project yet)
        if let Err(e) = audit_log_action(
            "QMS_FOLDER_SETUP",
            "UserProfile",
            &format!("user:{} path:{}", request.username, request.qms_folder_path.display())
        ) {
            eprintln!("Warning: Could not log QMS folder setup: {}", e);
        }
        
        Ok(profile)
    }
    
    /// Get default QMS folder path for the current platform
    /// YAGNI: Simple implementation, can be extended later if needed
    pub fn get_default_qms_path() -> QmsResult<PathBuf> {
        UserProfile::get_default_qms_folder()
    }
    
    /// Validate user setup completeness
    /// GRASP: Controller - coordinates validation across multiple components
    pub fn validate_user_setup(&self, username: &str) -> QmsResult<SetupValidation> {
        self.profile_manager.validate_user_setup(username)
    }
    
    /// Check if system requires QMS folder setup for user
    pub fn requires_qms_setup(&self, username: &str) -> QmsResult<bool> {
        let validation = self.validate_user_setup(username)?;
        Ok(!validation.setup_complete)
    }
    
    /// Get user profile summary for display
    pub fn get_user_summary(&self, username: &str) -> QmsResult<crate::modules::user_manager::profile_manager::ProfileSummary> {
        self.profile_manager.get_profile_summary(username)
    }
    
    /// List all users in the system (admin function)
    pub fn list_all_users(&self) -> QmsResult<Vec<String>> {
        let users = self.user_storage.list_users()?;
        Ok(users.into_iter().map(|user| user.username).collect())
    }
    
    /// Check if a specific user exists
    pub fn user_exists(&self, username: &str) -> QmsResult<bool> {
        self.user_storage.user_exists(username)
    }
    
    /// Get system statistics for admin dashboard
    pub fn get_system_stats(&self) -> QmsResult<SystemStats> {
        let users = self.user_storage.list_users()?;
        let user_count = users.len();
        
        let mut profiles_with_qms_setup = 0;
        let mut total_projects = 0;
        
        for user in &users {
            if let Ok(validation) = self.validate_user_setup(&user.username) {
                if validation.setup_complete {
                    profiles_with_qms_setup += 1;
                }
            }
            
            if let Ok(recent_projects) = self.profile_manager.get_recent_projects(&user.username) {
                total_projects += recent_projects.len();
            }
        }
        
        Ok(SystemStats {
            total_users: user_count,
            users_with_qms_setup: profiles_with_qms_setup,
            total_recent_projects: total_projects,
            system_initialized: user_count > 0,
        })
    }
}

/// System statistics for admin dashboard
#[derive(Debug, Clone)]
pub struct SystemStats {
    pub total_users: usize,
    pub users_with_qms_setup: usize,
    pub total_recent_projects: usize,
    pub system_initialized: bool,
}

impl Default for StartupAuthService {
    fn default() -> Self {
        Self::new().expect("Failed to create StartupAuthService")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::audit_logger::{AuditConfig, initialize_audit_system};
    use std::env;
    
    fn init_audit_for_test() {
        let temp_dir = env::temp_dir().join("qms_startup_auth_test");
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

        let _ = initialize_audit_system(config);
    }

    /// Create isolated test storage for StartupAuthService tests
    /// ACID: Isolation - each test gets its own storage
    fn get_test_startup_service() -> StartupAuthService {
        use crate::modules::user_manager::implementations::global_user_storage::GlobalUserStorage;

        // Create isolated GlobalUserStorage using the test constructor
        let user_storage = GlobalUserStorage::new_test_instance();

        // Create UserProfileManager (it will create its own GlobalUserStorage, but that's OK for now)
        let profile_manager = UserProfileManager::new().unwrap();

        StartupAuthService::new_with_storage(user_storage, profile_manager)
    }
    
    #[test]
    fn test_startup_auth_service_creation() {
        init_audit_for_test();
        let service = get_test_startup_service();
        let state = service.check_startup_state().unwrap();

        // Should require admin setup when no users exist
        assert!(state.requires_admin_setup);
        assert!(!state.has_existing_users);
        assert!(!state.system_initialized);
    }
    
    #[test]
    fn test_admin_setup_validation() {
        init_audit_for_test();
        let service = get_test_startup_service();
        
        // Valid request
        let valid_request = AdminSetupRequest {
            username: "admin".to_string(),
            password: "password123".to_string(),
            confirm_password: "password123".to_string(),
            qms_folder_path: Some(env::temp_dir().join("qms_test")),
        };
        
        assert!(service.validate_admin_setup(&valid_request).is_ok());
        
        // Invalid username
        let invalid_username = AdminSetupRequest {
            username: "ab".to_string(),
            password: "password123".to_string(),
            confirm_password: "password123".to_string(),
            qms_folder_path: None,
        };
        
        assert!(service.validate_admin_setup(&invalid_username).is_err());
        
        // Password mismatch
        let password_mismatch = AdminSetupRequest {
            username: "admin".to_string(),
            password: "password123".to_string(),
            confirm_password: "different".to_string(),
            qms_folder_path: None,
        };
        
        assert!(service.validate_admin_setup(&password_mismatch).is_err());
    }
    
    #[test]
    fn test_initial_admin_creation() {
        init_audit_for_test();
        let service = get_test_startup_service();

        let unique_id = format!("{}_{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
        let request = AdminSetupRequest {
            username: format!("admin_{}", unique_id),
            password: "password123".to_string(),
            confirm_password: "password123".to_string(),
            qms_folder_path: Some(env::temp_dir().join(format!("qms_test_admin_{}", unique_id))),
        };

        let profile = service.create_initial_admin(&request).unwrap();
        assert_eq!(profile.username, request.username);
        assert!(profile.qms_folder_path.exists());

        // Verify that the service now reports users exist
        let state_after = service.check_startup_state().unwrap();
        assert!(!state_after.requires_admin_setup);
        assert!(state_after.has_existing_users);
        assert!(state_after.system_initialized);

        // Should not allow creating another admin with the same service instance
        let duplicate_request = AdminSetupRequest {
            username: format!("admin2_{}", unique_id),
            password: "password123".to_string(),
            confirm_password: "password123".to_string(),
            qms_folder_path: None,
        };

        assert!(service.create_initial_admin(&duplicate_request).is_err());
    }
    
    #[test]
    fn test_system_stats() {
        init_audit_for_test();
        let service = get_test_startup_service();
        
        let stats = service.get_system_stats().unwrap();
        assert_eq!(stats.total_users, 0);
        assert!(!stats.system_initialized);
    }
}
