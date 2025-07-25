//! Authentication Service for QMS User Management
//! FDA 21 CFR Part 11 compliant user authentication with comprehensive audit trails

use crate::error::{QmsError, QmsResult};
use crate::audit::{log_user_action, log_system_event};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

/// User authentication credentials
#[derive(Debug, Clone)]
pub struct UserCredentials {
    pub username: String,
    pub password: String,
}

/// User session information
#[derive(Debug, Clone)]
pub struct UserSession {
    pub session_id: String,
    pub username: String,
    pub role: UserRole,
    pub created_at: u64,
    pub expires_at: u64,
    pub last_activity: u64,
}

/// User roles for access control
#[derive(Debug, Clone, PartialEq)]
pub enum UserRole {
    Admin,
    QualityEngineer,
    Developer,
    Auditor,
    ReadOnly,
}

impl UserRole {
    pub fn has_permission(&self, permission: &Permission) -> bool {
        match (self, permission) {
            (UserRole::Admin, _) => true,
            (UserRole::QualityEngineer, Permission::DocumentWrite) => true,
            (UserRole::QualityEngineer, Permission::DocumentRead) => true,
            (UserRole::QualityEngineer, Permission::RiskManagement) => true,
            (UserRole::QualityEngineer, Permission::AuditRead) => true,
            (UserRole::Developer, Permission::DocumentRead) => true,
            (UserRole::Developer, Permission::RiskRead) => true,
            (UserRole::Auditor, Permission::AuditRead) => true,
            (UserRole::Auditor, Permission::DocumentRead) => true,
            (UserRole::ReadOnly, Permission::DocumentRead) => true,
            (UserRole::ReadOnly, Permission::RiskRead) => true,
            _ => false,
        }
    }
}

/// System permissions
#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
    DocumentRead,
    DocumentWrite,
    RiskManagement,
    RiskRead,
    AuditRead,
    UserManagement,
    SystemAdmin,
}

/// User entity with FDA compliance fields
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub password_hash: String,
    pub salt: String,
    pub created_at: u64,
    pub last_login: Option<u64>,
    pub failed_login_attempts: u32,
    pub account_locked: bool,
    pub password_expires_at: u64,
    pub mfa_enabled: bool,
    pub mfa_secret: Option<String>,
}

impl User {
    pub fn new(username: String, email: String, role: UserRole) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            username,
            email,
            role,
            password_hash: String::new(),
            salt: String::new(),
            created_at: now,
            last_login: None,
            failed_login_attempts: 0,
            account_locked: false,
            password_expires_at: now + (90 * 24 * 60 * 60), // 90 days
            mfa_enabled: false,
            mfa_secret: None,
        }
    }

    pub fn is_account_valid(&self) -> bool {
        !self.account_locked && !self.is_password_expired()
    }

    pub fn is_password_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.password_expires_at
    }
}

/// Authentication service with FDA compliance
pub struct AuthenticationService {
    users: HashMap<String, User>,
    sessions: HashMap<String, UserSession>,
    session_timeout_minutes: u64,
    max_failed_attempts: u32,
}

impl AuthenticationService {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            sessions: HashMap::new(),
            session_timeout_minutes: 30, // FDA recommended session timeout
            max_failed_attempts: 3,
        }
    }

    /// Authenticate user and create session
    pub fn authenticate(&mut self, credentials: UserCredentials) -> QmsResult<UserSession> {
        log_user_action(&credentials.username, "LOGIN_ATTEMPT", "authentication_service", "INITIATED");

        // Check if user exists
        let user = self.users.get_mut(&credentials.username)
            .ok_or_else(|| {
                log_user_action(&credentials.username, "LOGIN_ATTEMPT", "authentication_service", "FAILED_USER_NOT_FOUND");
                QmsError::authentication_error("Invalid credentials")
            })?;

        // Check if account is locked
        if user.account_locked {
            log_user_action(&credentials.username, "LOGIN_ATTEMPT", "authentication_service", "FAILED_ACCOUNT_LOCKED");
            return Err(QmsError::authentication_error("Account locked"));
        }

        // Check if password is expired
        if user.is_password_expired() {
            log_user_action(&credentials.username, "LOGIN_ATTEMPT", "authentication_service", "FAILED_PASSWORD_EXPIRED");
            return Err(QmsError::authentication_error("Password expired"));
        }

        // Verify password
        if !self.verify_password(&credentials.password, &user.password_hash, &user.salt) {
            user.failed_login_attempts += 1;
            
            if user.failed_login_attempts >= self.max_failed_attempts {
                user.account_locked = true;
                log_user_action(&credentials.username, "ACCOUNT_LOCKED", "authentication_service", "MAX_ATTEMPTS_EXCEEDED");
            }
            
            log_user_action(&credentials.username, "LOGIN_ATTEMPT", "authentication_service", "FAILED_INVALID_PASSWORD");
            return Err(QmsError::authentication_error("Invalid credentials"));
        }

        // Reset failed attempts on successful authentication
        user.failed_login_attempts = 0;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        user.last_login = Some(now);

        // Create session
        let session = self.create_session(user)?;
        
        log_user_action(&credentials.username, "LOGIN_SUCCESS", "authentication_service", "SESSION_CREATED");
        log_system_event("USER_SESSION_CREATED", "authentication_service", &format!("User: {}", credentials.username));

        Ok(session)
    }

    /// Create a new user session
    fn create_session(&mut self, user: &User) -> QmsResult<UserSession> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let session = UserSession {
            session_id: uuid::Uuid::new_v4().to_string(),
            username: user.username.clone(),
            role: user.role.clone(),
            created_at: now,
            expires_at: now + (self.session_timeout_minutes * 60),
            last_activity: now,
        };

        self.sessions.insert(session.session_id.clone(), session.clone());
        Ok(session)
    }

    /// Validate session and check if it's still active
    pub fn validate_session(&mut self, session_id: &str) -> QmsResult<UserSession> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| QmsError::authentication_error("Invalid session"))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check if session expired
        if now > session.expires_at {
            self.sessions.remove(session_id);
            log_user_action(&session.username, "SESSION_EXPIRED", "authentication_service", "TIMEOUT");
            return Err(QmsError::authentication_error("Session expired"));
        }

        // Update last activity and extend session
        session.last_activity = now;
        session.expires_at = now + (self.session_timeout_minutes * 60);

        Ok(session.clone())
    }

    /// Logout user and invalidate session
    pub fn logout(&mut self, session_id: &str) -> QmsResult<()> {
        if let Some(session) = self.sessions.remove(session_id) {
            log_user_action(&session.username, "LOGOUT", "authentication_service", "SUCCESS");
            log_system_event("USER_SESSION_TERMINATED", "authentication_service", &format!("User: {}", session.username));
        }
        Ok(())
    }

    /// Register a new user
    pub fn register_user(&mut self, username: String, email: String, password: String, role: UserRole) -> QmsResult<User> {
        // Check if username already exists
        if self.users.contains_key(&username) {
            return Err(QmsError::validation_error("Username already exists"));
        }

        // Validate password strength
        self.validate_password_strength(&password)?;

        // Create user
        let mut user = User::new(username.clone(), email, role);
        let (password_hash, salt) = self.hash_password(&password)?;
        user.password_hash = password_hash;
        user.salt = salt;

        self.users.insert(username.clone(), user.clone());
        
        log_user_action(&username, "USER_REGISTERED", "authentication_service", "SUCCESS");
        log_system_event("USER_CREATED", "authentication_service", &format!("User: {}", username));

        Ok(user)
    }

    /// Change user password
    pub fn change_password(&mut self, username: &str, old_password: &str, new_password: &str) -> QmsResult<()> {
        let user = self.users.get_mut(username)
            .ok_or_else(|| QmsError::not_found("User not found"))?;

        // Verify old password
        if !self.verify_password(old_password, &user.password_hash, &user.salt) {
            log_user_action(username, "PASSWORD_CHANGE", "authentication_service", "FAILED_INVALID_OLD_PASSWORD");
            return Err(QmsError::authentication_error("Invalid old password"));
        }

        // Validate new password strength
        self.validate_password_strength(new_password)?;

        // Update password
        let (password_hash, salt) = self.hash_password(new_password)?;
        user.password_hash = password_hash;
        user.salt = salt;
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        user.password_expires_at = now + (90 * 24 * 60 * 60); // 90 days

        log_user_action(username, "PASSWORD_CHANGED", "authentication_service", "SUCCESS");
        
        Ok(())
    }

    /// Check if user has permission
    pub fn check_permission(&self, session_id: &str, permission: Permission) -> QmsResult<bool> {
        let session = self.sessions.get(session_id)
            .ok_or_else(|| QmsError::authentication_error("Invalid session"))?;

        Ok(session.role.has_permission(&permission))
    }

    /// Hash password with salt
    fn hash_password(&self, password: &str) -> QmsResult<(String, String)> {
        let salt = uuid::Uuid::new_v4().to_string();
        let salted_password = format!("{}{}", password, salt);
        let hash = format!("{:x}", md5::compute(salted_password.as_bytes()));
        Ok((hash, salt))
    }

    /// Verify password against hash
    fn verify_password(&self, password: &str, hash: &str, salt: &str) -> bool {
        let salted_password = format!("{}{}", password, salt);
        let computed_hash = format!("{:x}", md5::compute(salted_password.as_bytes()));
        computed_hash == hash
    }

    /// Validate password strength per FDA requirements
    fn validate_password_strength(&self, password: &str) -> QmsResult<()> {
        if password.len() < 8 {
            return Err(QmsError::validation_error("Password must be at least 8 characters"));
        }

        let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));

        if !has_uppercase || !has_lowercase || !has_digit || !has_special {
            return Err(QmsError::validation_error(
                "Password must contain uppercase, lowercase, digit, and special character"
            ));
        }

        Ok(())
    }

    /// Get user by username
    pub fn get_user(&self, username: &str) -> Option<&User> {
        self.users.get(username)
    }

    /// Get all active sessions
    pub fn get_active_sessions(&self) -> Vec<&UserSession> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.sessions.values()
            .filter(|session| session.expires_at > now)
            .collect()
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let expired_sessions: Vec<String> = self.sessions
            .iter()
            .filter(|(_, session)| session.expires_at <= now)
            .map(|(id, _)| id.clone())
            .collect();

        for session_id in expired_sessions {
            if let Some(session) = self.sessions.remove(&session_id) {
                log_user_action(&session.username, "SESSION_EXPIRED", "authentication_service", "CLEANUP");
            }
        }
    }
}

impl Default for AuthenticationService {
    fn default() -> Self {
        Self::new()
    }
}

// Add uuid dependency for unique IDs
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "test_user".to_string(),
            "test@example.com".to_string(),
            UserRole::QualityEngineer
        );

        assert_eq!(user.username, "test_user");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.role, UserRole::QualityEngineer);
        assert!(!user.account_locked);
        assert_eq!(user.failed_login_attempts, 0);
        assert!(!user.is_password_expired());
    }

    #[test]
    fn test_user_role_permissions() {
        assert!(UserRole::Admin.has_permission(&Permission::DocumentWrite));
        assert!(UserRole::Admin.has_permission(&Permission::UserManagement));
        assert!(UserRole::QualityEngineer.has_permission(&Permission::DocumentWrite));
        assert!(UserRole::QualityEngineer.has_permission(&Permission::RiskManagement));
        assert!(!UserRole::Developer.has_permission(&Permission::DocumentWrite));
        assert!(UserRole::Developer.has_permission(&Permission::DocumentRead));
        assert!(!UserRole::ReadOnly.has_permission(&Permission::DocumentWrite));
        assert!(UserRole::ReadOnly.has_permission(&Permission::DocumentRead));
    }

    #[test]
    fn test_authentication_service_creation() {
        let auth_service = AuthenticationService::new();
        assert_eq!(auth_service.session_timeout_minutes, 30);
        assert_eq!(auth_service.max_failed_attempts, 3);
        assert!(auth_service.users.is_empty());
        assert!(auth_service.sessions.is_empty());
    }

    #[test]
    fn test_user_registration() {
        let mut auth_service = AuthenticationService::new();
        
        let result = auth_service.register_user(
            "test_user".to_string(),
            "test@example.com".to_string(),
            "Password123!".to_string(),
            UserRole::QualityEngineer
        );

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.username, "test_user");
        assert_eq!(user.role, UserRole::QualityEngineer);
        assert!(!user.password_hash.is_empty());
        assert!(!user.salt.is_empty());
    }

    #[test]
    fn test_duplicate_user_registration() {
        let mut auth_service = AuthenticationService::new();
        
        // Register first user
        auth_service.register_user(
            "test_user".to_string(),
            "test@example.com".to_string(),
            "Password123!".to_string(),
            UserRole::QualityEngineer
        ).unwrap();

        // Try to register duplicate user
        let result = auth_service.register_user(
            "test_user".to_string(),
            "test2@example.com".to_string(),
            "Password123!".to_string(),
            UserRole::Developer
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Validation error: Username already exists");
    }

    #[test]
    fn test_password_strength_validation() {
        let auth_service = AuthenticationService::new();

        // Test weak passwords
        assert!(auth_service.validate_password_strength("weak").is_err());
        assert!(auth_service.validate_password_strength("password").is_err());
        assert!(auth_service.validate_password_strength("Password").is_err());
        assert!(auth_service.validate_password_strength("Password123").is_err());

        // Test strong password
        assert!(auth_service.validate_password_strength("Password123!").is_ok());
    }

    #[test]
    fn test_successful_authentication() {
        let mut auth_service = AuthenticationService::new();
        
        // Register user
        auth_service.register_user(
            "test_user".to_string(),
            "test@example.com".to_string(),
            "Password123!".to_string(),
            UserRole::QualityEngineer
        ).unwrap();

        // Authenticate
        let credentials = UserCredentials {
            username: "test_user".to_string(),
            password: "Password123!".to_string(),
        };

        let result = auth_service.authenticate(credentials);
        assert!(result.is_ok());
        
        let session = result.unwrap();
        assert_eq!(session.username, "test_user");
        assert_eq!(session.role, UserRole::QualityEngineer);
        assert!(!session.session_id.is_empty());
    }

    #[test]
    fn test_failed_authentication_invalid_user() {
        let mut auth_service = AuthenticationService::new();

        let credentials = UserCredentials {
            username: "nonexistent".to_string(),
            password: "Password123!".to_string(),
        };

        let result = auth_service.authenticate(credentials);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Authentication error: Invalid credentials");
    }

    #[test]
    fn test_failed_authentication_invalid_password() {
        let mut auth_service = AuthenticationService::new();
        
        // Register user
        auth_service.register_user(
            "test_user".to_string(),
            "test@example.com".to_string(),
            "Password123!".to_string(),
            UserRole::QualityEngineer
        ).unwrap();

        // Try with wrong password
        let credentials = UserCredentials {
            username: "test_user".to_string(),
            password: "WrongPassword!".to_string(),
        };

        let result = auth_service.authenticate(credentials);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Authentication error: Invalid credentials");
    }

    #[test]
    fn test_account_lockout() {
        let mut auth_service = AuthenticationService::new();
        
        // Register user
        auth_service.register_user(
            "test_user".to_string(),
            "test@example.com".to_string(),
            "Password123!".to_string(),
            UserRole::QualityEngineer
        ).unwrap();

        // Fail authentication 3 times
        for _ in 0..3 {
            let credentials = UserCredentials {
                username: "test_user".to_string(),
                password: "WrongPassword!".to_string(),
            };
            let _ = auth_service.authenticate(credentials);
        }

        // Check that account is locked
        let user = auth_service.get_user("test_user").unwrap();
        assert!(user.account_locked);

        // Try to authenticate with correct password - should fail
        let credentials = UserCredentials {
            username: "test_user".to_string(),
            password: "Password123!".to_string(),
        };

        let result = auth_service.authenticate(credentials);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Authentication error: Account locked");
    }

    #[test]
    fn test_session_validation() {
        let mut auth_service = AuthenticationService::new();
        
        // Register and authenticate user
        auth_service.register_user(
            "test_user".to_string(),
            "test@example.com".to_string(),
            "Password123!".to_string(),
            UserRole::QualityEngineer
        ).unwrap();

        let credentials = UserCredentials {
            username: "test_user".to_string(),
            password: "Password123!".to_string(),
        };

        let session = auth_service.authenticate(credentials).unwrap();

        // Validate session
        let result = auth_service.validate_session(&session.session_id);
        assert!(result.is_ok());
        
        let validated_session = result.unwrap();
        assert_eq!(validated_session.username, "test_user");
    }

    #[test]
    fn test_session_logout() {
        let mut auth_service = AuthenticationService::new();
        
        // Register and authenticate user
        auth_service.register_user(
            "test_user".to_string(),
            "test@example.com".to_string(),
            "Password123!".to_string(),
            UserRole::QualityEngineer
        ).unwrap();

        let credentials = UserCredentials {
            username: "test_user".to_string(),
            password: "Password123!".to_string(),
        };

        let session = auth_service.authenticate(credentials).unwrap();

        // Logout
        let result = auth_service.logout(&session.session_id);
        assert!(result.is_ok());

        // Try to validate session after logout
        let result = auth_service.validate_session(&session.session_id);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Authentication error: Invalid session");
    }

    #[test]
    fn test_permission_check() {
        let mut auth_service = AuthenticationService::new();
        
        // Register and authenticate user
        auth_service.register_user(
            "test_user".to_string(),
            "test@example.com".to_string(),
            "Password123!".to_string(),
            UserRole::QualityEngineer
        ).unwrap();

        let credentials = UserCredentials {
            username: "test_user".to_string(),
            password: "Password123!".to_string(),
        };

        let session = auth_service.authenticate(credentials).unwrap();

        // Check permissions
        assert!(auth_service.check_permission(&session.session_id, Permission::DocumentRead).unwrap());
        assert!(auth_service.check_permission(&session.session_id, Permission::DocumentWrite).unwrap());
        assert!(!auth_service.check_permission(&session.session_id, Permission::UserManagement).unwrap());
    }

    #[test]
    fn test_password_change() {
        let mut auth_service = AuthenticationService::new();
        
        // Register user
        auth_service.register_user(
            "test_user".to_string(),
            "test@example.com".to_string(),
            "Password123!".to_string(),
            UserRole::QualityEngineer
        ).unwrap();

        // Change password
        let result = auth_service.change_password(
            "test_user",
            "Password123!",
            "NewPassword456!"
        );
        assert!(result.is_ok());

        // Try to authenticate with old password - should fail
        let old_credentials = UserCredentials {
            username: "test_user".to_string(),
            password: "Password123!".to_string(),
        };
        assert!(auth_service.authenticate(old_credentials).is_err());

        // Authenticate with new password - should succeed
        let new_credentials = UserCredentials {
            username: "test_user".to_string(),
            password: "NewPassword456!".to_string(),
        };
        assert!(auth_service.authenticate(new_credentials).is_ok());
    }

    #[test]
    fn test_expired_session_cleanup() {
        let mut auth_service = AuthenticationService::new();
        auth_service.session_timeout_minutes = 0; // Immediate expiry for testing
        
        // Register and authenticate user
        auth_service.register_user(
            "test_user".to_string(),
            "test@example.com".to_string(),
            "Password123!".to_string(),
            UserRole::QualityEngineer
        ).unwrap();

        let credentials = UserCredentials {
            username: "test_user".to_string(),
            password: "Password123!".to_string(),
        };

        let session = auth_service.authenticate(credentials).unwrap();
        
        // Wait a moment and cleanup
        std::thread::sleep(std::time::Duration::from_millis(10));
        auth_service.cleanup_expired_sessions();

        // Session should be removed
        let result = auth_service.validate_session(&session.session_id);
        assert!(result.is_err());
    }
}