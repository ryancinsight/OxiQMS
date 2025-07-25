//! Unified Authentication Service for QMS
//! Provides centralized authentication with secure password hashing using Argon2

use crate::error::{QmsError, QmsResult};
use crate::modules::user_manager::interfaces::{UserSession, User, UserRole, SessionType};
use crate::audit::{log_user_action, log_system_event};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use argon2::{self, Config};
use rand::Rng;
use tracing::error;

/// Unified authentication service for both CLI and web
pub struct UnifiedAuthenticationService<U: UserStorage, S: SessionStorage> {
    user_storage: U,
    session_storage: Arc<Mutex<S>>,
    session_timeout_hours: u64,
}

impl<U: UserStorage, S: SessionStorage> UnifiedAuthenticationService<U, S> {
    /// Create new unified authentication service
    pub fn new(user_storage: U, session_storage: S) -> Self {
        Self {
            user_storage,
            session_storage: Arc::new(Mutex::new(session_storage)),
            session_timeout_hours: 24, // Default 24 hours
        }
    }
    
    /// Create with custom session timeout
    pub fn with_timeout(user_storage: U, session_storage: S, timeout_hours: u64) -> Self {
        Self {
            user_storage,
            session_storage: Arc::new(Mutex::new(session_storage)),
            session_timeout_hours: timeout_hours,
        }
    }
    
    /// Authenticate user and create session
    pub fn login(
        &self, 
        username: &str, 
        password: &str, 
        session_type: SessionType,
        ip_address: Option<String>,
        user_agent: Option<String>
    ) -> QmsResult<UserSession> {
        // Load user from storage
        let user = self.user_storage.load_user(username)?;
        
        // Verify password
        if !Self::verify_password(password, &user.password_hash) {
            let _ = audit_log_action("LOGIN_FAILED", "User", username);
            return Err(QmsError::Authentication("Invalid credentials".to_string()));
        }
        
        // Create session
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let session = UserSession {
            session_id: UserSession::generate_session_id(),
            user_id: user.username.clone(),
            username: user.username.clone(),
            roles: user.roles.clone(),
            permissions: self.extract_permissions(&user.roles),
            login_time: now,
            last_activity: now,
            expires_at: now + (self.session_timeout_hours * 3600),
            ip_address,
            user_agent,
            csrf_token: UserSession::generate_csrf_token(),
            is_active: true,
            session_type,
            data: HashMap::new(),
        };
        
        // Store session
        {
            let session_storage = self.session_storage.lock()
                .map_err(|_| QmsError::domain_error("Failed to acquire session storage lock"))?;
            session_storage.save_session(&session)?;
        }
        
        // Audit log successful login (skip if no project context)
        let _ = audit_log_action("LOGIN_SUCCESS", "User", username);
        let _ = audit_log_action(
            "SESSION_CREATED",
            "Session",
            &format!("session:{} user:{} type:{:?}", session.session_id, username, session.session_type)
        );
        
        Ok(session)
    }
    
    /// Validate existing session
    pub fn validate_session(&self, session_id: &str) -> QmsResult<UserSession> {
        let session_storage = self.session_storage.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire session storage lock"))?;
        
        let mut session = session_storage.load_session(session_id)?;
        
        // Check if session is expired or inactive
        if !session.is_authenticated() {
            return Err(QmsError::Authentication("Session expired or inactive".to_string()));
        }
        
        // Update last activity
        session.update_activity();
        session_storage.save_session(&session)?;
        
        Ok(session)
    }
    
    /// Create session for already authenticated user (for cross-interface sync)
    pub fn create_session_for_user(&self, username: &str, session_type: SessionType) -> QmsResult<UserSession> {
        // Load user to get roles and permissions
        let user = self.user_storage.load_user(username)?;

        // Create session without password verification (user already authenticated)
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let session = UserSession {
            session_id: UserSession::generate_session_id(),
            user_id: user.username.clone(),
            username: user.username.clone(),
            roles: user.roles.clone(),
            permissions: self.extract_permissions(&user.roles),
            login_time: now,
            last_activity: now,
            expires_at: now + (self.session_timeout_hours * 3600),
            ip_address: None, // Cross-interface sessions don't track IP
            user_agent: None, // Cross-interface sessions don't track user agent
            csrf_token: UserSession::generate_csrf_token(),
            is_active: true,
            session_type,
            data: HashMap::new(),
        };

        // Store session
        {
            let session_storage = self.session_storage.lock()
                .map_err(|_| QmsError::domain_error("Failed to acquire session storage lock"))?;
            session_storage.save_session(&session)?;
        }

        // Audit log
        let _ = audit_log_action("SESSION_CREATED", "Authentication",
            &format!("Cross-interface session created for user: {}, type: {:?}", username, session_type));

        Ok(session)
    }

    /// Update session activity
    pub fn update_session_activity(&self, session_id: &str) -> QmsResult<()> {
        let session_storage = self.session_storage.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire session storage lock"))?;

        let mut session = session_storage.load_session(session_id)?;
        session.update_activity();
        session_storage.save_session(&session)?;

        Ok(())
    }
    
    /// Logout user (terminate session)
    pub fn logout(&self, session_id: &str) -> QmsResult<()> {
        let session_storage = self.session_storage.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire session storage lock"))?;
        
        // Get session info for audit log before deletion
        let session = session_storage.load_session(session_id)?;
        let username = session.username.clone();
        
        // Delete session
        session_storage.delete_session(session_id)?;
        
        // Audit log logout (skip if no project context)
        let _ = audit_log_action("LOGOUT", "User", &username);
        let _ = audit_log_action("SESSION_TERMINATED", "Session", session_id);
        
        Ok(())
    }
    
    /// List active sessions for user
    pub fn list_user_sessions(&self, username: &str) -> QmsResult<Vec<UserSession>> {
        let session_storage = self.session_storage.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire session storage lock"))?;

        let sessions = session_storage.list_user_sessions(username)?;

        // Filter out expired sessions
        let active_sessions: Vec<UserSession> = sessions
            .into_iter()
            .filter(|session| session.is_authenticated())
            .collect();

        Ok(active_sessions)
    }

    /// Get all active sessions (for bidirectional session recognition)
    pub fn get_all_active_sessions(&self) -> QmsResult<Vec<UserSession>> {
        let session_storage = self.session_storage.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire session storage lock"))?;

        let all_sessions = session_storage.list_sessions()?;

        // Filter out expired and inactive sessions
        let active_sessions: Vec<UserSession> = all_sessions
            .into_iter()
            .filter(|session| session.is_active && session.is_authenticated() && !session.is_expired())
            .collect();

        Ok(active_sessions)
    }
    
    /// Cleanup expired sessions
    pub fn cleanup_expired_sessions(&self) -> QmsResult<usize> {
        let session_storage = self.session_storage.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire session storage lock"))?;
        
        let cleaned_count = session_storage.cleanup_expired_sessions()?;
        
        if cleaned_count > 0 {
            let _ = audit_log_action(
                "SESSIONS_CLEANED",
                "System",
                &format!("Cleaned {} expired sessions", cleaned_count)
            );
        }
        
        Ok(cleaned_count)
    }
    
    /// Create user (admin function)
    pub fn create_user(&self, username: &str, password: &str, roles: Vec<Role>) -> QmsResult<User> {
        // Check if user already exists
        if self.user_storage.user_exists(username)? {
            return Err(QmsError::domain_error(&format!("User '{}' already exists", username)));
        }
        
        // Create user
        let user = User {
            username: username.to_string(),
            password_hash: Self::hash_password(password),
            roles,
            created_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            last_login: None,
        };
        
        // Save user
        self.user_storage.save_user(&user)?;
        
        // Audit log (skip if no project context)
        let _ = audit_log_action("USER_CREATED", "User", username);
        
        Ok(user)
    }
    
    /// Get user by username
    pub fn get_user(&self, username: &str) -> QmsResult<User> {
        self.user_storage.load_user(username)
    }

    /// Save user (admin function)
    pub fn save_user(&self, user: &User) -> QmsResult<()> {
        self.user_storage.save_user(user)
    }

    /// Hash password using Argon2 for security and FDA compliance
    pub fn hash_password(password: &str) -> String {
        let salt = rand::thread_rng().gen::<[u8; 32]>();
        let config = Config::default();
        argon2::hash_encoded(password.as_bytes(), &salt, &config)
            .expect("Failed to hash password")
    }
    
    /// Verify password against Argon2 hash
    pub fn verify_password(password: &str, hash: &str) -> bool {
        argon2::verify_encoded(hash, password.as_bytes()).is_ok()
    }
    
    /// Check if user has permission
    pub fn has_permission(&self, session_id: &str, permission: &str) -> QmsResult<bool> {
        let session = self.validate_session(session_id)?;
        Ok(session.permissions.contains(&permission.to_string()))
    }
    

    
    /// Extract permissions from roles
    fn extract_permissions(&self, roles: &[Role]) -> Vec<String> {
        let mut permissions = Vec::new();
        
        for role in roles {
            for permission in &role.permissions {
                let perm_str = match permission {
                    Permission::ReadDocuments => "read_documents",
                    Permission::WriteDocuments => "write_documents",
                    Permission::DeleteDocuments => "delete_documents",
                    Permission::ReadRisks => "read_risks",
                    Permission::WriteRisks => "write_risks",
                    Permission::DeleteRisks => "delete_risks",
                    Permission::ReadTrace => "read_trace",
                    Permission::WriteTrace => "write_trace",
                    Permission::DeleteTrace => "delete_trace",
                    Permission::ReadAudit => "read_audit",
                    Permission::ExportAudit => "export_audit",
                    Permission::ManageUsers => "manage_users",
                    Permission::GenerateReports => "generate_reports",
                    Permission::UserManagement => "user_management",
                    Permission::ProjectManagement => "project_management",
                    Permission::DocumentManagement => "document_management",
                    Permission::RiskManagement => "risk_management",
                    Permission::AuditAccess => "audit_access",
                    Permission::SystemConfiguration => "system_configuration",
                };

                if !permissions.contains(&perm_str.to_string()) {
                    permissions.push(perm_str.to_string());
                }
            }
        }
        
        permissions
    }
}

// Implement Clone for the service if both storage types implement Clone
impl<U: UserStorage + Clone, S: SessionStorage + Clone> Clone for UnifiedAuthenticationService<U, S> {
    fn clone(&self) -> Self {
        let session_storage = self.session_storage.lock().unwrap().clone();
        Self {
            user_storage: self.user_storage.clone(),
            session_storage: Arc::new(Mutex::new(session_storage)),
            session_timeout_hours: self.session_timeout_hours,
        }
    }
}
