// Unified Authentication Factory
// Creates configured authentication services for both CLI and web use
// Follows Factory Pattern and Dependency Injection principles

use crate::prelude::*;
use crate::modules::user_manager::{
    UnifiedAuthenticationService, FileUserStorage, FileSessionStorage, SessionType
};
use std::path::Path;

/// Factory for creating unified authentication services
pub struct UnifiedAuthFactory;

impl UnifiedAuthFactory {
    /// Create a file-based unified authentication service
    pub fn create_file_based(project_path: &Path) -> QmsResult<UnifiedAuthenticationService<FileUserStorage, FileSessionStorage>> {
        let user_storage = FileUserStorage::new(project_path)?;
        let session_storage = FileSessionStorage::new(project_path)?;

        Ok(UnifiedAuthenticationService::new(user_storage, session_storage))
    }

    /// Create a global authentication service (for CLI use)
    pub fn create_global() -> QmsResult<UnifiedAuthenticationService<FileUserStorage, FileSessionStorage>> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| QmsError::io_error("Cannot determine home directory"))?;

        let qms_dir = Path::new(&home).join(".qms");
        std::fs::create_dir_all(&qms_dir)?;

        let user_storage = FileUserStorage::new(&qms_dir)?;
        let session_storage = FileSessionStorage::new(&qms_dir)?;

        Ok(UnifiedAuthenticationService::new(user_storage, session_storage))
    }

    /// Create a file-based unified authentication service with custom timeout
    pub fn create_file_based_with_timeout(
        project_path: &Path,
        timeout_hours: u64
    ) -> QmsResult<UnifiedAuthenticationService<FileUserStorage, FileSessionStorage>> {
        let user_storage = FileUserStorage::new(project_path)?;
        let session_storage = FileSessionStorage::new(project_path)?;

        Ok(UnifiedAuthenticationService::with_timeout(user_storage, session_storage, timeout_hours))
    }
}

/// Type alias for the standard file-based authentication service
pub type FileBasedAuthService = UnifiedAuthenticationService<FileUserStorage, FileSessionStorage>;

/// Helper functions for common authentication operations
impl FileBasedAuthService {
    /// Create from project path (convenience method)
    pub fn from_project_path(project_path: &Path) -> QmsResult<Self> {
        UnifiedAuthFactory::create_file_based(project_path)
    }

    /// Create global authentication service (for CLI use)
    pub fn create_global() -> QmsResult<Self> {
        UnifiedAuthFactory::create_global()
    }
    
    /// Login for CLI usage
    pub fn cli_login(&self, username: &str, password: &str) -> QmsResult<crate::modules::user_manager::UserSession> {
        self.login(username, password, SessionType::CLI, None, None)
    }
    
    /// Login for web usage
    pub fn web_login(
        &self, 
        username: &str, 
        password: &str, 
        ip_address: Option<String>, 
        user_agent: Option<String>
    ) -> QmsResult<crate::modules::user_manager::UserSession> {
        self.login(username, password, SessionType::Web, ip_address, user_agent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::modules::audit_logger::{AuditConfig, initialize_audit_system};
    
    /// Initialize audit system for tests
    fn init_audit_for_test(temp_dir: &std::path::Path) {
        let audit_dir = temp_dir.join("audit");
        let _ = std::fs::create_dir_all(&audit_dir);

        // Create a unique config to avoid conflicts
        let config = AuditConfig {
            project_path: temp_dir.to_string_lossy().to_string(),
            retention_days: 30,
            daily_rotation: false,
            max_file_size_mb: 10,
            require_checksums: false,
        };

        // Use a timeout-safe initialization approach
        match initialize_audit_system(config) {
            Ok(_) => {},
            Err(_) => {
                // If initialization fails, continue with test - audit is not critical for auth logic
                eprintln!("Warning: Audit system initialization failed in test, continuing without audit");
            }
        }
    }
    
    #[test]
    fn test_unified_auth_factory() {
        let temp_dir = tempdir().unwrap();
        init_audit_for_test(temp_dir.path());
        
        let auth_service = UnifiedAuthFactory::create_file_based(temp_dir.path()).unwrap();
        
        // Test that we can create a user
        let roles = vec![];
        let user = auth_service.create_user("testuser", "password123", roles).unwrap();
        assert_eq!(user.username, "testuser");
    }
    
    #[test]
    fn test_cli_login() {
        let temp_dir = tempdir().unwrap();
        init_audit_for_test(temp_dir.path());
        
        let auth_service = FileBasedAuthService::from_project_path(temp_dir.path()).unwrap();
        
        // Create a user first
        let roles = vec![];
        auth_service.create_user("testuser", "password123", roles).unwrap();
        
        // Test CLI login
        let session = auth_service.cli_login("testuser", "password123").unwrap();
        assert_eq!(session.username, "testuser");
        assert_eq!(session.session_type, SessionType::CLI);
        assert!(session.ip_address.is_none());
        assert!(session.user_agent.is_none());
    }
    
    #[test]
    fn test_web_login() {
        let temp_dir = tempdir().unwrap();
        init_audit_for_test(temp_dir.path());
        
        let auth_service = FileBasedAuthService::from_project_path(temp_dir.path()).unwrap();
        
        // Create a user first
        let roles = vec![];
        auth_service.create_user("testuser", "password123", roles).unwrap();
        
        // Test web login
        let session = auth_service.web_login(
            "testuser", 
            "password123", 
            Some("127.0.0.1".to_string()), 
            Some("Mozilla/5.0".to_string())
        ).unwrap();
        
        assert_eq!(session.username, "testuser");
        assert_eq!(session.session_type, SessionType::Web);
        assert_eq!(session.ip_address, Some("127.0.0.1".to_string()));
        assert_eq!(session.user_agent, Some("Mozilla/5.0".to_string()));
    }
    
    #[test]
    fn test_session_validation() {
        let temp_dir = tempdir().unwrap();
        init_audit_for_test(temp_dir.path());
        
        let auth_service = FileBasedAuthService::from_project_path(temp_dir.path()).unwrap();
        
        // Create a user and login
        let roles = vec![];
        auth_service.create_user("testuser", "password123", roles).unwrap();
        let session = auth_service.cli_login("testuser", "password123").unwrap();
        
        // Validate the session
        let validated_session = auth_service.validate_session(&session.session_id).unwrap();
        assert_eq!(validated_session.username, "testuser");
        assert_eq!(validated_session.session_id, session.session_id);
    }
    
    #[test]
    fn test_session_logout() {
        let temp_dir = tempdir().unwrap();
        init_audit_for_test(temp_dir.path());
        
        let auth_service = FileBasedAuthService::from_project_path(temp_dir.path()).unwrap();
        
        // Create a user and login
        let roles = vec![];
        auth_service.create_user("testuser", "password123", roles).unwrap();
        let session = auth_service.cli_login("testuser", "password123").unwrap();
        
        // Logout
        auth_service.logout(&session.session_id).unwrap();
        
        // Session should be invalid after logout
        let result = auth_service.validate_session(&session.session_id);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_session_persistence() {
        let temp_dir = tempdir().unwrap();
        init_audit_for_test(temp_dir.path());
        
        let session_id = {
            let auth_service = FileBasedAuthService::from_project_path(temp_dir.path()).unwrap();
            
            // Create a user and login
            let roles = vec![];
            auth_service.create_user("testuser", "password123", roles).unwrap();
            let session = auth_service.cli_login("testuser", "password123").unwrap();
            session.session_id
        };
        
        // Create a new auth service instance (simulating server restart)
        let auth_service2 = FileBasedAuthService::from_project_path(temp_dir.path()).unwrap();
        
        // Session should still be valid
        let validated_session = auth_service2.validate_session(&session_id).unwrap();
        assert_eq!(validated_session.username, "testuser");
    }
}
