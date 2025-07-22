// Interface Segregation Principle - Focused interfaces for user management
// Each interface has a single, well-defined responsibility

use crate::prelude::*;
use crate::models::{User, Role, Permission};
use std::collections::HashMap;

/// Authentication interface - handles user credential verification
pub trait UserAuthenticator {
    /// Authenticate user with credentials
    fn authenticate(&self, username: &str, password: &str) -> QmsResult<AuthenticationResult>;
    
    /// Validate password strength
    fn validate_password(&self, password: &str) -> QmsResult<()>;
    
    /// Hash password for storage
    fn hash_password(&self, password: &str) -> String;
    
    /// Verify password against hash
    fn verify_password(&self, password: &str, hash: &str) -> bool;
}

/// Authorization interface - handles permission checking
pub trait UserAuthorizer {
    /// Check if user has specific permission
    fn has_permission(&self, user: &User, permission: &Permission) -> bool;
    
    /// Check if user has any of the specified permissions
    fn has_any_permission(&self, user: &User, permissions: &[Permission]) -> bool;
    
    /// Check if user has all of the specified permissions
    fn has_all_permissions(&self, user: &User, permissions: &[Permission]) -> bool;
    
    /// Get all permissions for user
    fn get_user_permissions(&self, user: &User) -> Vec<Permission>;
}

/// User storage interface - handles user persistence
pub trait UserStorage {
    /// Save user to storage
    fn save_user(&self, user: &User) -> QmsResult<()>;
    
    /// Load user from storage
    fn load_user(&self, username: &str) -> QmsResult<User>;
    
    /// Check if user exists
    fn user_exists(&self, username: &str) -> QmsResult<bool>;
    
    /// List all users
    fn list_users(&self) -> QmsResult<Vec<User>>;
    
    /// Delete user from storage
    fn delete_user(&self, username: &str) -> QmsResult<()>;
    
    /// Update user in storage
    fn update_user(&self, user: &User) -> QmsResult<()>;
}

/// Session management interface - handles user sessions
pub trait SessionManager {
    /// Create new session for user
    fn create_session(&self, user: &User) -> QmsResult<UserSession>;
    
    /// Validate existing session
    fn validate_session(&self, session_id: &str) -> QmsResult<UserSession>;
    
    /// Update session activity
    fn update_session_activity(&self, session_id: &str) -> QmsResult<()>;
    
    /// Terminate session
    fn terminate_session(&self, session_id: &str) -> QmsResult<()>;
    
    /// List active sessions
    fn list_active_sessions(&self) -> QmsResult<Vec<UserSession>>;
    
    /// Cleanup expired sessions
    fn cleanup_expired_sessions(&self) -> QmsResult<usize>;
}

/// Role management interface - handles role operations
pub trait RoleProvider {
    /// Get role by name
    fn get_role_by_name(&self, name: &str) -> QmsResult<Role>;
    
    /// Get all available roles
    fn get_available_roles(&self) -> Vec<Role>;
    
    /// Assign role to user
    fn assign_role_to_user(&self, username: &str, role_name: &str) -> QmsResult<()>;
    
    /// Remove role from user
    fn remove_role_from_user(&self, username: &str, role_name: &str) -> QmsResult<()>;
    
    /// Get user roles
    fn get_user_roles(&self, username: &str) -> QmsResult<Vec<Role>>;
}

/// Authentication result
#[derive(Debug, Clone)]
pub struct AuthenticationResult {
    pub success: bool,
    pub user: Option<User>,
    pub session: Option<UserSession>,
    pub message: String,
}

/// User session (consolidated from multiple definitions)
#[derive(Debug, Clone)]
pub struct UserSession {
    pub session_id: String,
    pub user_id: String,
    pub username: String,
    pub roles: Vec<Role>,
    pub login_time: u64,
    pub last_activity: u64,
    pub ip_address: Option<String>,
    pub expires_at: u64,
    pub is_active: bool,
}

impl UserSession {
    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        now > self.expires_at
    }
    
    /// Generate new session ID
    pub fn generate_session_id() -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let mut hasher = DefaultHasher::new();
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
        format!("session_{:x}", hasher.finish())
    }
}

/// User validation interface - handles user data validation
pub trait UserValidator {
    /// Validate username format
    fn validate_username(&self, username: &str) -> QmsResult<()>;
    
    /// Validate user data
    fn validate_user(&self, user: &User) -> QmsResult<()>;
    
    /// Validate role assignment
    fn validate_role_assignment(&self, user: &User, role: &Role) -> QmsResult<()>;
}

/// Audit logging interface for user operations
pub trait UserAuditor {
    /// Log user operation
    fn log_user_operation(&self, operation: &str, username: &str, details: &str) -> QmsResult<()>;
    
    /// Log authentication attempt
    fn log_authentication(&self, username: &str, success: bool, ip: Option<&str>) -> QmsResult<()>;
    
    /// Log session operation
    fn log_session_operation(&self, operation: &str, session_id: &str, username: &str) -> QmsResult<()>;
    
    /// Log role operation
    fn log_role_operation(&self, operation: &str, username: &str, role: &str) -> QmsResult<()>;
}
