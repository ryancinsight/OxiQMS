use crate::prelude::*;
use crate::models::{User, Role, Permission};
use crate::modules::audit_logger::audit_log_action;
use crate::modules::user_manager::interfaces::UserStorage; // REFACTORED: Use abstraction
use crate::modules::user_manager::implementations::{FileUserStorage, MemoryUserStorage}; // REFACTORED: Concrete implementations
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// User authentication manager
/// REFACTORED: Now uses dependency injection with UserStorage abstraction
/// Follows Dependency Inversion Principle - depends on abstractions, not concretions
pub struct AuthManager<S: UserStorage> {
    storage: S,
    current_user: Option<String>,
    active_sessions: Arc<Mutex<HashMap<String, UserSession>>>,
}

/// Active user session
#[derive(Debug, Clone)]
pub struct UserSession {
    #[allow(dead_code)]
    pub user_id: String,
    pub username: String,
    pub roles: Vec<Role>,
    pub session_id: String,
    pub login_time: u64,
    pub last_activity: u64,
    #[allow(dead_code)]
    pub ip_address: Option<String>,
}

impl<S: UserStorage> AuthManager<S> {
    /// Create new authentication manager with dependency injection
    /// REFACTORED: Uses UserStorage abstraction instead of direct file operations
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            current_user: None,
            active_sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// Convenience type aliases for common configurations
pub type FileAuthManager = AuthManager<FileUserStorage>;
pub type MemoryAuthManager = AuthManager<MemoryUserStorage>;

impl FileAuthManager {
    /// Create new file-based authentication manager (backward compatibility)
    /// REFACTORED: Maintains API compatibility while using dependency injection internally
    pub fn from_project_path(project_path: &Path) -> QmsResult<Self> {
        let storage = FileUserStorage::new(project_path)?;
        Ok(Self::new(storage))
    }
}

impl MemoryAuthManager {
    /// Create new memory-based authentication manager (for testing)
    pub fn for_testing() -> QmsResult<Self> {
        let storage = MemoryUserStorage::new()?;
        Ok(Self::new(storage))
    }
}

impl<S: UserStorage> AuthManager<S> {
    /// Hash password using stdlib DefaultHasher
    pub fn hash_password(password: &str) -> String {
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
    
    /// Add new user
    pub fn add_user(&self, username: &str, password: &str, roles: Option<Vec<Role>>) -> QmsResult<User> {
        if !User::validate_username(username) {
            return Err(QmsError::validation_error("Invalid username format"));
        }
        
        if password.len() < 8 {
            return Err(QmsError::validation_error("Password must be at least 8 characters"));
        }
        
        // Check if user already exists
        if self.user_exists(username)? {
            return Err(QmsError::already_exists("User already exists"));
        }
        
        let user = User {
            username: username.to_string(),
            password_hash: Self::hash_password(password),
            roles: roles.unwrap_or_else(|| vec![Self::get_quality_engineer_role()]),
            created_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            last_login: None,
        };
        
        self.save_user(&user)?;

        // Attempt audit logging, but don't fail user creation if audit logging fails
        // This is especially important during setup when the audit system may not be fully initialized
        if let Err(e) = audit_log_action("USER_CREATED", "User", username) {
            eprintln!("Warning: Failed to log user creation audit entry: {}", e);
            // Continue with user creation - audit logging failure should not prevent user creation
        }
        
        Ok(user)
    }
    
    /// Authenticate user and create session
    pub fn login(&mut self, username: &str, password: &str) -> QmsResult<UserSession> {
        let user = self.load_user(username)?;
        
        if user.password_hash != Self::hash_password(password) {
            // Attempt audit logging, but don't fail login if audit logging fails
            if let Err(e) = audit_log_action("LOGIN_FAILED", "User", username) {
                eprintln!("Warning: Failed to log login failure audit entry: {}", e);
            }
            return Err(QmsError::Authentication("Invalid credentials".to_string()));
        }
        
        let session = UserSession {
            user_id: user.username.clone(),
            username: user.username.clone(),
            roles: user.roles.clone(),
            session_id: Self::generate_session_id(),
            login_time: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            last_activity: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            ip_address: None,
        };
        
        // Update user last login
        let mut updated_user = user;
        updated_user.last_login = Some(session.login_time);
        self.save_user(&updated_user)?;
        
        // Store session
        {
            let mut sessions = self.active_sessions.lock().unwrap();
            sessions.insert(session.session_id.clone(), session.clone());
        }
        
        self.current_user = Some(username.to_string());

        // Attempt audit logging, but don't fail login if audit logging fails
        if let Err(e) = audit_log_action("LOGIN_SUCCESS", "User", username) {
            eprintln!("Warning: Failed to log login success audit entry: {}", e);
        }
        
        Ok(session)
    }
    
    /// Logout user
    pub fn logout(&mut self, session_id: &str) -> QmsResult<()> {
        let username = {
            let mut sessions = self.active_sessions.lock().unwrap();
            if let Some(session) = sessions.remove(session_id) {
                session.username
            } else {
                return Err(QmsError::NotFound("Session not found".to_string()));
            }
        };
        
        if self.current_user.as_ref() == Some(&username) {
            self.current_user = None;
        }
        
        audit_log_action("LOGOUT", "User", &username)?;
        Ok(())
    }
    
    /// Get current user
    #[allow(dead_code)]
    pub const fn get_current_user(&self) -> Option<&String> {
        self.current_user.as_ref()
    }
    
    /// Validate session
    pub fn validate_session(&self, session_id: &str) -> QmsResult<UserSession> {
        let sessions = self.active_sessions.lock().unwrap();
        if let Some(session) = sessions.get(session_id) {
            // Check if session has expired (24 hours)
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            if now - session.last_activity > 24 * 3600 {
                return Err(QmsError::Authentication("Session expired".to_string()));
            }
            
            Ok(session.clone())
        } else {
            Err(QmsError::NotFound("Session not found".to_string()))
        }
    }
    
    /// Check if user has permission
    #[allow(dead_code)]
    pub fn has_permission(&self, username: &str, permission: &Permission) -> QmsResult<bool> {
        let user = self.load_user(username)?;
        
        for role in &user.roles {
            if role.permissions.contains(permission) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    

    
    /// Load user from storage
    /// REFACTORED: Delegates to UserStorage abstraction
    pub fn load_user(&self, username: &str) -> QmsResult<User> {
        self.storage.load_user(username)
    }

    /// Save user to storage
    /// REFACTORED: Delegates to UserStorage abstraction
    pub fn save_user(&self, user: &User) -> QmsResult<()> {
        self.storage.save_user(user)
    }
    
    /// List all users
    /// REFACTORED: Delegates to UserStorage abstraction
    pub fn list_users(&self) -> QmsResult<Vec<User>> {
        self.storage.list_users()
    }

    /// Check if user exists
    /// REFACTORED: Delegates to UserStorage abstraction
    pub fn user_exists(&self, username: &str) -> QmsResult<bool> {
        self.storage.user_exists(username)
    }
    
    /// Generate session ID
    fn generate_session_id() -> String {
        let mut hasher = DefaultHasher::new();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        timestamp.hash(&mut hasher);
        format!("ses-{:x}", hasher.finish())
    }

    /// Get admin role
    fn get_admin_role() -> Role {
        Role {
            name: "Administrator".to_string(),
            permissions: vec![
                Permission::ManageUsers,
                Permission::ReadDocuments,
                Permission::WriteDocuments,
                Permission::DeleteDocuments,
                Permission::ReadRisks,
                Permission::WriteRisks,
                Permission::DeleteRisks,
                Permission::ReadTrace,
                Permission::WriteTrace,
                Permission::DeleteTrace,
                Permission::ReadAudit,
                Permission::ExportAudit,
                Permission::GenerateReports,
            ],
        }
    }
    
    /// Get quality engineer role
    fn get_quality_engineer_role() -> Role {
        Role {
            name: "QualityEngineer".to_string(),
            permissions: vec![
                Permission::ReadDocuments,
                Permission::WriteDocuments,
                Permission::ReadRisks,
                Permission::WriteRisks,
                Permission::ReadTrace,
                Permission::WriteTrace,
                Permission::ReadAudit,
                Permission::GenerateReports,
            ],
        }
    }
    
    /// Get developer role
    #[allow(dead_code)]
    fn get_developer_role() -> Role {
        Role {
            name: "Developer".to_string(),
            permissions: vec![
                Permission::ReadDocuments,
                Permission::WriteDocuments,
                Permission::ReadRisks,
                Permission::ReadTrace,
                Permission::WriteTrace,
            ],
        }
    }
    
    /// Get auditor role
    #[allow(dead_code)]
    fn get_auditor_role() -> Role {
        Role {
            name: "Auditor".to_string(),
            permissions: vec![
                Permission::ReadDocuments,
                Permission::ReadRisks,
                Permission::ReadTrace,
                Permission::ReadAudit,
                Permission::ExportAudit,
            ],
        }
    }
}
