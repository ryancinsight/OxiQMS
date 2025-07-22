// User Manager Factory - Creates configured user management components
// Implements Factory pattern with dependency injection

use crate::prelude::*;
use crate::modules::user_manager::interfaces::*;
use crate::modules::user_manager::strategies::{
    PasswordAuthenticationStrategy, RoleBasedAuthorizationStrategy,
    DefaultSessionStrategy
};
use crate::modules::user_manager::strategies::role_auth_strategy::HierarchicalAuthorizationStrategy;
use crate::modules::user_manager::strategies::session_strategy::SessionConfig;
use crate::modules::user_manager::implementations::FileUserStorage;
use std::path::Path;
use std::sync::Arc;

/// Configuration for user manager factory
#[derive(Debug, Clone)]
pub struct UserManagerConfig {
    pub project_path: PathBuf,
    pub session_timeout_hours: u64,
    pub max_concurrent_sessions: usize,
    pub min_password_length: usize,
    pub require_special_chars: bool,
    pub strict_authorization: bool,
    pub enable_permission_hierarchy: bool,
}

impl UserManagerConfig {
    /// Create default configuration
    pub fn new(project_path: &Path) -> Self {
        Self {
            project_path: project_path.to_path_buf(),
            session_timeout_hours: 24,
            max_concurrent_sessions: 5,
            min_password_length: 8,
            require_special_chars: false,
            strict_authorization: true,
            enable_permission_hierarchy: true,
        }
    }
    
    /// Create configuration for medical device compliance
    pub fn medical_device_compliant(project_path: &Path) -> Self {
        Self {
            project_path: project_path.to_path_buf(),
            session_timeout_hours: 8, // Shorter sessions for compliance
            max_concurrent_sessions: 3, // Limit concurrent access
            min_password_length: 12, // Stronger passwords
            require_special_chars: true, // Require special characters
            strict_authorization: true, // Strict permission checking
            enable_permission_hierarchy: true, // Use permission hierarchy
        }
    }
    
    /// Create configuration for development/testing
    pub fn development(project_path: &Path) -> Self {
        Self {
            project_path: project_path.to_path_buf(),
            session_timeout_hours: 72, // Longer sessions for development
            max_concurrent_sessions: 10, // More concurrent sessions
            min_password_length: 6, // Shorter passwords for testing
            require_special_chars: false, // No special char requirement
            strict_authorization: false, // Permissive authorization
            enable_permission_hierarchy: true, // Still use hierarchy
        }
    }
}

/// User manager factory - creates configured components
pub struct UserManagerFactory {
    config: UserManagerConfig,
}

impl UserManagerFactory {
    /// Create new factory with configuration
    pub fn new(config: UserManagerConfig) -> Self {
        Self { config }
    }
    
    /// Create factory with default configuration
    pub fn with_defaults(project_path: &Path) -> Self {
        Self::new(UserManagerConfig::new(project_path))
    }
    
    /// Create factory for medical device compliance
    pub fn for_medical_device(project_path: &Path) -> Self {
        Self::new(UserManagerConfig::medical_device_compliant(project_path))
    }
    
    /// Create factory for development
    pub fn for_development(project_path: &Path) -> Self {
        Self::new(UserManagerConfig::development(project_path))
    }
    
    /// Create user storage implementation
    pub fn create_user_storage(&self) -> QmsResult<Box<dyn UserStorage>> {
        Ok(Box::new(FileUserStorage::new(&self.config.project_path)?))
    }
    
    /// Create authentication strategy
    pub fn create_authenticator(&self) -> QmsResult<Box<dyn UserAuthenticator>> {
        let storage = self.create_user_storage()?;
        let auth_strategy = PasswordAuthenticationStrategy::with_requirements(
            FileUserStorage::new(&self.config.project_path)?,
            self.config.min_password_length,
            self.config.require_special_chars,
        );
        Ok(Box::new(auth_strategy))
    }
    
    /// Create authorization strategy
    pub fn create_authorizer(&self) -> Box<dyn UserAuthorizer> {
        if self.config.enable_permission_hierarchy {
            Box::new(HierarchicalAuthorizationStrategy::new())
        } else if self.config.strict_authorization {
            Box::new(RoleBasedAuthorizationStrategy::new())
        } else {
            Box::new(RoleBasedAuthorizationStrategy::with_permissive_mode())
        }
    }
    
    /// Create session manager
    pub fn create_session_manager(&self) -> Box<dyn SessionManager> {
        let session_config = SessionConfig {
            session_timeout_hours: self.config.session_timeout_hours,
            max_concurrent_sessions: self.config.max_concurrent_sessions,
            cleanup_interval_minutes: 60,
            require_ip_validation: false,
            extend_on_activity: true,
        };
        
        Box::new(DefaultSessionStrategy::with_config(session_config))
    }
    
    // TODO: Implement role provider and consolidated user manager
    // These will be added in the next phase of consolidation
    
    /// Get configuration
    pub fn get_config(&self) -> &UserManagerConfig {
        &self.config
    }
}

/// Builder pattern for user manager factory
pub struct UserManagerFactoryBuilder {
    config: UserManagerConfig,
}

impl UserManagerFactoryBuilder {
    /// Create new builder
    pub fn new(project_path: &Path) -> Self {
        Self {
            config: UserManagerConfig::new(project_path),
        }
    }
    
    /// Set session timeout
    pub fn with_session_timeout(mut self, hours: u64) -> Self {
        self.config.session_timeout_hours = hours;
        self
    }
    
    /// Set maximum concurrent sessions
    pub fn with_max_sessions(mut self, max: usize) -> Self {
        self.config.max_concurrent_sessions = max;
        self
    }
    
    /// Set password requirements
    pub fn with_password_requirements(mut self, min_length: usize, require_special: bool) -> Self {
        self.config.min_password_length = min_length;
        self.config.require_special_chars = require_special;
        self
    }
    
    /// Set authorization mode
    pub fn with_authorization_mode(mut self, strict: bool, hierarchy: bool) -> Self {
        self.config.strict_authorization = strict;
        self.config.enable_permission_hierarchy = hierarchy;
        self
    }
    
    /// Build the factory
    pub fn build(self) -> UserManagerFactory {
        UserManagerFactory::new(self.config)
    }
}

/// Factory for creating legacy components (backward compatibility)
pub struct LegacyUserManagerFactory;

impl LegacyUserManagerFactory {
    /// Create legacy AuthManager
    pub fn create_auth_manager(project_path: &Path) -> QmsResult<crate::modules::user_manager::FileAuthManager> {
        crate::modules::user_manager::FileAuthManager::from_project_path(project_path)
    }
    
    /// Create legacy RoleManager
    pub fn create_role_manager(project_path: &Path) -> QmsResult<crate::modules::user_manager::RoleManager> {
        crate::modules::user_manager::RoleManager::new(project_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_factory_creation() {
        let temp_dir = tempdir().unwrap();
        let factory = UserManagerFactory::with_defaults(temp_dir.path());

        // Test component creation
        assert!(factory.create_user_storage().is_ok());
        assert!(factory.create_authenticator().is_ok());

        let _authorizer = factory.create_authorizer();
        let _session_manager = factory.create_session_manager();
        // TODO: Add role provider test when implemented
    }
    
    #[test]
    fn test_medical_device_config() {
        let temp_dir = tempdir().unwrap();
        let factory = UserManagerFactory::for_medical_device(temp_dir.path());
        let config = factory.get_config();
        
        // Medical device compliance should have stricter settings
        assert_eq!(config.session_timeout_hours, 8);
        assert_eq!(config.max_concurrent_sessions, 3);
        assert_eq!(config.min_password_length, 12);
        assert!(config.require_special_chars);
        assert!(config.strict_authorization);
        assert!(config.enable_permission_hierarchy);
    }
    
    #[test]
    fn test_development_config() {
        let temp_dir = tempdir().unwrap();
        let factory = UserManagerFactory::for_development(temp_dir.path());
        let config = factory.get_config();
        
        // Development should have more permissive settings
        assert_eq!(config.session_timeout_hours, 72);
        assert_eq!(config.max_concurrent_sessions, 10);
        assert_eq!(config.min_password_length, 6);
        assert!(!config.require_special_chars);
        assert!(!config.strict_authorization);
    }
    
    #[test]
    fn test_builder_pattern() {
        let temp_dir = tempdir().unwrap();
        let factory = UserManagerFactoryBuilder::new(temp_dir.path())
            .with_session_timeout(12)
            .with_max_sessions(8)
            .with_password_requirements(10, true)
            .with_authorization_mode(false, true)
            .build();
        
        let config = factory.get_config();
        assert_eq!(config.session_timeout_hours, 12);
        assert_eq!(config.max_concurrent_sessions, 8);
        assert_eq!(config.min_password_length, 10);
        assert!(config.require_special_chars);
        assert!(!config.strict_authorization);
        assert!(config.enable_permission_hierarchy);
    }
    
    #[test]
    fn test_legacy_factory() {
        let temp_dir = tempdir().unwrap();
        
        // Legacy components should still be creatable
        assert!(LegacyUserManagerFactory::create_auth_manager(temp_dir.path()).is_ok());
        assert!(LegacyUserManagerFactory::create_role_manager(temp_dir.path()).is_ok());
    }
}
