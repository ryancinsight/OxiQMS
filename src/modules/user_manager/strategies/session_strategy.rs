// Session management strategy
// Implements SessionManager interface with configurable session policies

use crate::prelude::*;
use crate::models::User;
use crate::modules::user_manager::interfaces::{SessionManager, UserSession, SessionType};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub session_timeout_hours: u64,
    pub max_concurrent_sessions: usize,
    pub cleanup_interval_minutes: u64,
    pub require_ip_validation: bool,
    pub extend_on_activity: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_timeout_hours: 24,
            max_concurrent_sessions: 5,
            cleanup_interval_minutes: 60,
            require_ip_validation: false,
            extend_on_activity: true,
        }
    }
}

/// Default session management strategy
pub struct DefaultSessionStrategy {
    sessions: Arc<Mutex<HashMap<String, UserSession>>>,
    config: SessionConfig,
    last_cleanup: Arc<Mutex<u64>>,
}

impl DefaultSessionStrategy {
    /// Create new session strategy with default configuration
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            config: SessionConfig::default(),
            last_cleanup: Arc::new(Mutex::new(0)),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: SessionConfig) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            config,
            last_cleanup: Arc::new(Mutex::new(0)),
        }
    }
    
    /// Get current timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
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
    
    /// Check if automatic cleanup should run
    fn should_run_cleanup(&self) -> bool {
        let now = Self::current_timestamp();
        let last_cleanup = *self.last_cleanup.lock().unwrap();
        now - last_cleanup > self.config.cleanup_interval_minutes * 60
    }
    
    /// Run automatic cleanup if needed
    fn auto_cleanup(&self) {
        if self.should_run_cleanup() {
            let _ = self.cleanup_expired_sessions();
            *self.last_cleanup.lock().unwrap() = Self::current_timestamp();
        }
    }
    
    /// Count active sessions for user
    fn count_user_sessions(&self, username: &str) -> usize {
        self.sessions
            .lock()
            .unwrap()
            .values()
            .filter(|session| session.username == username && session.is_active && !session.is_expired())
            .count()
    }
    
    /// Remove oldest session for user if at limit (Single Responsibility Principle)
    /// This method ensures we maintain the maximum concurrent session limit
    fn enforce_session_limit(&self, username: &str) -> QmsResult<()> {
        let mut sessions = self.sessions.lock().unwrap();

        // Collect active sessions for this user (Information Expert principle)
        let mut active_user_sessions: Vec<(String, u64)> = sessions.iter()
            .filter(|(_, session)| {
                session.username == username &&
                session.is_active &&
                !session.is_expired()
            })
            .map(|(id, session)| (id.clone(), session.login_time))
            .collect();



        // Apply session limit enforcement (ACID - Consistency)
        // Remove oldest sessions to make room for the new session we're about to add
        // We need to ensure that after adding one more session, we don't exceed the limit
        while active_user_sessions.len() >= self.config.max_concurrent_sessions {
            // Find the oldest session (KISS principle - simple comparison)
            if let Some((oldest_id, _)) = active_user_sessions.iter()
                .min_by_key(|(_, login_time)| *login_time) {

                let oldest_id = oldest_id.clone();

                // Remove from both our tracking and the actual sessions
                sessions.remove(&oldest_id);
                active_user_sessions.retain(|(id, _)| id != &oldest_id);
            } else {
                break; // No sessions to remove
            }
        }
        Ok(())
    }
}

impl Default for DefaultSessionStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager for DefaultSessionStrategy {
    fn create_session(&self, user: &User, session_type: SessionType, ip_address: Option<String>, user_agent: Option<String>) -> QmsResult<UserSession> {
        self.auto_cleanup();

        let now = Self::current_timestamp();
        let session = UserSession {
            session_id: UserSession::generate_session_id(),
            user_id: user.username.clone(),
            username: user.username.clone(),
            roles: user.roles.clone(),
            permissions: self.extract_permissions(&user.roles),
            login_time: now,
            last_activity: now,
            expires_at: now + self.config.session_timeout_hours * 3600,
            ip_address,
            user_agent,
            csrf_token: UserSession::generate_csrf_token(),
            is_active: true,
            session_type,
            data: std::collections::HashMap::new(),
        };

        // Enforce session limit before storing the new session
        self.enforce_session_limit(&user.username)?;

        // Store session
        self.sessions.lock().unwrap().insert(session.session_id.clone(), session.clone());

        Ok(session)
    }
    
    fn validate_session(&self, session_id: &str) -> QmsResult<UserSession> {
        self.auto_cleanup();
        
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(session_id) {
            if !session.is_active {
                return Err(QmsError::Authentication("Session is inactive".to_string()));
            }
            
            if session.is_expired() {
                return Err(QmsError::Authentication("Session has expired".to_string()));
            }
            
            Ok(session.clone())
        } else {
            Err(QmsError::not_found("Session not found"))
        }
    }
    
    fn update_session_activity(&self, session_id: &str) -> QmsResult<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            if session.is_active && !session.is_expired() {
                let now = Self::current_timestamp();
                session.last_activity = now;

                // Extend expiration if configured
                if self.config.extend_on_activity {
                    session.expires_at = now + self.config.session_timeout_hours * 3600;
                }

                Ok(())
            } else {
                Err(QmsError::Authentication("Session is inactive or expired".to_string()))
            }
        } else {
            Err(QmsError::not_found("Session not found"))
        }
    }
    
    fn terminate_session(&self, session_id: &str) -> QmsResult<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            session.is_active = false;
            Ok(())
        } else {
            Err(QmsError::not_found("Session not found"))
        }
    }
    
    fn list_active_sessions(&self) -> QmsResult<Vec<UserSession>> {
        self.auto_cleanup();
        
        let sessions = self.sessions.lock().unwrap();
        let active_sessions: Vec<UserSession> = sessions
            .values()
            .filter(|session| session.is_active && !session.is_expired())
            .cloned()
            .collect();
        
        Ok(active_sessions)
    }
    
    fn cleanup_expired_sessions(&self) -> QmsResult<usize> {
        let mut sessions = self.sessions.lock().unwrap();
        let initial_count = sessions.len();
        
        // Remove expired or inactive sessions
        sessions.retain(|_, session| session.is_active && !session.is_expired());
        
        let removed_count = initial_count - sessions.len();
        Ok(removed_count)
    }
}

/// In-memory session strategy for testing
pub struct InMemorySessionStrategy {
    base_strategy: DefaultSessionStrategy,
}

impl InMemorySessionStrategy {
    /// Create new in-memory session strategy
    pub fn new() -> Self {
        Self {
            base_strategy: DefaultSessionStrategy::new(),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: SessionConfig) -> Self {
        Self {
            base_strategy: DefaultSessionStrategy::with_config(config),
        }
    }
}

impl Default for InMemorySessionStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager for InMemorySessionStrategy {
    fn create_session(&self, user: &User, session_type: SessionType, ip_address: Option<String>, user_agent: Option<String>) -> QmsResult<UserSession> {
        self.base_strategy.create_session(user, session_type, ip_address, user_agent)
    }
    
    fn validate_session(&self, session_id: &str) -> QmsResult<UserSession> {
        self.base_strategy.validate_session(session_id)
    }
    
    fn update_session_activity(&self, session_id: &str) -> QmsResult<()> {
        self.base_strategy.update_session_activity(session_id)
    }
    
    fn terminate_session(&self, session_id: &str) -> QmsResult<()> {
        self.base_strategy.terminate_session(session_id)
    }
    
    fn list_active_sessions(&self) -> QmsResult<Vec<UserSession>> {
        self.base_strategy.list_active_sessions()
    }
    
    fn cleanup_expired_sessions(&self) -> QmsResult<usize> {
        self.base_strategy.cleanup_expired_sessions()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Role, Permission};
    
    fn create_test_user() -> User {
        User {
            username: "testuser".to_string(),
            password_hash: "hash".to_string(),
            roles: vec![Role {
                name: "TestRole".to_string(),
                permissions: vec![Permission::ReadDocuments],
            }],
            created_at: 0,
            last_login: None,
        }
    }
    
    #[test]
    fn test_session_creation() {
        let session_manager = DefaultSessionStrategy::new();
        let user = create_test_user();
        
        let session = session_manager.create_session(&user, SessionType::CLI, None, None).unwrap();
        
        assert_eq!(session.username, "testuser");
        assert!(session.is_active);
        assert!(!session.is_expired());
        assert!(!session.session_id.is_empty());
    }
    
    #[test]
    fn test_session_validation() {
        let session_manager = DefaultSessionStrategy::new();
        let user = create_test_user();
        
        let session = session_manager.create_session(&user, SessionType::CLI, None, None).unwrap();
        let session_id = session.session_id.clone();
        
        // Valid session should be retrievable
        let validated = session_manager.validate_session(&session_id).unwrap();
        assert_eq!(validated.username, "testuser");
        
        // Invalid session ID should fail
        assert!(session_manager.validate_session("invalid_id").is_err());
    }
    
    #[test]
    fn test_session_termination() {
        let session_manager = DefaultSessionStrategy::new();
        let user = create_test_user();
        
        let session = session_manager.create_session(&user, SessionType::CLI, None, None).unwrap();
        let session_id = session.session_id.clone();
        
        // Terminate session
        session_manager.terminate_session(&session_id).unwrap();
        
        // Session should no longer be valid
        assert!(session_manager.validate_session(&session_id).is_err());
    }
    
    #[test]
    fn test_session_activity_update() {
        let session_manager = DefaultSessionStrategy::new();
        let user = create_test_user();

        let session = session_manager.create_session(&user, SessionType::CLI, None, None).unwrap();
        let session_id = session.session_id.clone();
        let initial_activity = session.last_activity;

        // Wait a moment to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_secs(1));
        session_manager.update_session_activity(&session_id).unwrap();

        // Activity should be updated
        let updated_session = session_manager.validate_session(&session_id).unwrap();
        assert!(updated_session.last_activity > initial_activity);
    }
    
    #[test]
    fn test_session_limit_enforcement() {
        let config = SessionConfig {
            max_concurrent_sessions: 2,
            cleanup_interval_minutes: 0, // Disable auto-cleanup for predictable testing
            ..Default::default()
        };
        let session_manager = DefaultSessionStrategy::with_config(config);

        // Create unique user for this test to avoid interference from other tests
        let user = User {
            username: format!("test_user_{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()),
            password_hash: "hash".to_string(),
            roles: vec![Role {
                name: "TestRole".to_string(),
                permissions: vec![Permission::ReadDocuments],
            }],
            created_at: 0,
            last_login: None,
        };

        // Create maximum number of sessions with sufficient delays to ensure different timestamps
        let session1 = session_manager.create_session(&user, SessionType::CLI, None, None).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1)); // Longer delay to ensure different timestamps

        let session2 = session_manager.create_session(&user, SessionType::CLI, None, None).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1)); // Longer delay to ensure different timestamps

        // Both should be valid
        assert!(session_manager.validate_session(&session1.session_id).is_ok(),
                "Session1 should be valid before limit enforcement");
        assert!(session_manager.validate_session(&session2.session_id).is_ok(),
                "Session2 should be valid before limit enforcement");

        // Verify we have exactly 2 active sessions
        let active_sessions = session_manager.list_active_sessions().unwrap();
        let user_sessions: Vec<_> = active_sessions.iter()
            .filter(|s| s.username == user.username)
            .collect();
        assert_eq!(user_sessions.len(), 2, "Should have exactly 2 active sessions before creating third");

        // Creating a third session should remove the oldest (session1)
        let session3 = session_manager.create_session(&user, SessionType::CLI, None, None).unwrap();

        // Verify session3 was created
        assert!(session_manager.validate_session(&session3.session_id).is_ok(),
                "Session3 should be valid after creation");

        // First session should be removed due to limit enforcement
        assert!(session_manager.validate_session(&session1.session_id).is_err(),
                "Session1 should be removed due to session limit enforcement");

        // Second session should still be valid
        assert!(session_manager.validate_session(&session2.session_id).is_ok(),
                "Session2 should still be valid after limit enforcement");

        // Verify we still have exactly 2 active sessions
        let final_active_sessions = session_manager.list_active_sessions().unwrap();
        let final_user_sessions: Vec<_> = final_active_sessions.iter()
            .filter(|s| s.username == user.username)
            .collect();
        assert_eq!(final_user_sessions.len(), 2, "Should have exactly 2 active sessions after limit enforcement");
    }
    
    #[test]
    fn test_expired_session_cleanup() {
        let config = SessionConfig {
            session_timeout_hours: 0, // Immediate expiration for testing
            ..Default::default()
        };
        let session_manager = DefaultSessionStrategy::with_config(config);
        let user = create_test_user();

        let _session = session_manager.create_session(&user, SessionType::CLI, None, None).unwrap();

        // Wait a moment to ensure expiration (since expires_at = now, we need time to pass)
        std::thread::sleep(std::time::Duration::from_secs(1));

        // Cleanup should remove the expired session
        let cleanup_count = session_manager.cleanup_expired_sessions().unwrap();
        assert_eq!(cleanup_count, 1);

        // No active sessions should remain
        let active_sessions = session_manager.list_active_sessions().unwrap();
        assert_eq!(active_sessions.len(), 0);
    }
}
