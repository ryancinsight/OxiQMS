use crate::prelude::*;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::sync::Mutex;

use super::HttpRequest;

/// Session management for web authentication
#[derive(Debug)]
pub struct SessionManager {
    sessions: Mutex<HashMap<String, Session>>,
    session_timeout: Duration,
}

/// User session data
#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub user_id: Option<String>,
    pub created_at: u64,
    pub last_accessed: u64,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub authenticated: bool,
    pub csrf_token: String,
    pub permissions: Vec<String>,
    pub data: HashMap<String, String>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            session_timeout: Duration::from_secs(3600), // 1 hour default
        }
    }

    pub fn new_with_timeout(timeout_seconds: u64) -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            session_timeout: Duration::from_secs(timeout_seconds),
        }
    }

    /// Get existing session or create new one
    pub fn get_or_create_session(&self, request: &HttpRequest) -> Session {
        // Try to get existing session from cookie
        if let Some(session_id) = request.get_session_cookie() {
            if let Some(session) = self.get_session(&session_id) {
                // Update last accessed time
                let mut updated_session = session.clone();
                updated_session.last_accessed = Self::current_timestamp();
                self.update_session(updated_session.clone());
                return updated_session;
            }
        }

        // Create new session
        self.create_session(request)
    }

    /// Create a new session
    pub fn create_session(&self, request: &HttpRequest) -> Session {
        let session_id = self.generate_session_id();
        let csrf_token = self.generate_csrf_token();
        let now = Self::current_timestamp();

        let session = Session {
            id: session_id.clone(),
            user_id: None,
            created_at: now,
            last_accessed: now,
            ip_address: request.headers.get("x-forwarded-for")
                .or_else(|| request.headers.get("x-real-ip"))
                .cloned(),
            user_agent: request.get_user_agent().cloned(),
            authenticated: false,
            csrf_token,
            permissions: Vec::new(),
            data: HashMap::new(),
        };

        // Store session
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.insert(session_id, session.clone());
        }

        session
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: &str) -> Option<Session> {
        if let Ok(sessions) = self.sessions.lock() {
            if let Some(session) = sessions.get(session_id) {
                // Check if session is expired
                if self.is_session_expired(session) {
                    return None;
                }
                return Some(session.clone());
            }
        }
        None
    }

    /// Update existing session
    pub fn update_session(&self, session: Session) {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.insert(session.id.clone(), session);
        }
    }

    /// Remove session (logout)
    pub fn remove_session(&self, session_id: &str) -> bool {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.remove(session_id).is_some()
        } else {
            false
        }
    }

    /// Authenticate session with user credentials
    pub fn authenticate_session(&self, session_id: &str, user_id: &str, permissions: Vec<String>) -> QmsResult<Session> {
        if let Ok(mut sessions) = self.sessions.lock() {
            if let Some(session) = sessions.get_mut(session_id) {
                session.user_id = Some(user_id.to_string());
                session.authenticated = true;
                session.permissions = permissions;
                session.last_accessed = Self::current_timestamp();
                
                // Audit log authentication
                if let Err(e) = crate::modules::audit_logger::audit_log_action(
                    "SESSION_AUTHENTICATED",
                    "Session",
                    &format!("session:{session_id} user:{user_id}")
                ) {
                    eprintln!("Warning: Failed to log session authentication: {e}");
                }

                return Ok(session.clone());
            }
        }
        Err(QmsError::not_found("Session not found"))
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&self) -> usize {
        if let Ok(mut sessions) = self.sessions.lock() {
            let initial_count = sessions.len();
            sessions.retain(|_, session| !self.is_session_expired(session));
            initial_count - sessions.len()
        } else {
            0
        }
    }

    /// Get all active sessions (for admin purposes)
    pub fn get_active_sessions(&self) -> Vec<Session> {
        if let Ok(sessions) = self.sessions.lock() {
            sessions.values()
                .filter(|session| !self.is_session_expired(session))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get session count
    pub fn get_session_count(&self) -> usize {
        if let Ok(sessions) = self.sessions.lock() {
            sessions.len()
        } else {
            0
        }
    }

    /// Check if session is expired
    fn is_session_expired(&self, session: &Session) -> bool {
        let now = Self::current_timestamp();
        let session_age = now.saturating_sub(session.last_accessed);
        session_age > self.session_timeout.as_secs()
    }

    /// Generate unique session ID
    fn generate_session_id(&self) -> String {
        // Simple session ID generation using timestamp and counter
        // In production, use cryptographically secure random generation
        let timestamp = Self::current_timestamp();
        let session_count = self.get_session_count();
        let hash_input = format!("qms_session_{timestamp}_{session_count}");
        
        // Simple hash for session ID (stdlib only)
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        hash_input.hash(&mut hasher);
        format!("qms_{:x}", hasher.finish())
    }

    /// Generate CSRF token
    fn generate_csrf_token(&self) -> String {
        // Simple CSRF token generation
        let timestamp = Self::current_timestamp();
        let hash_input = format!("csrf_{}_{}", timestamp, "qms_secret");
        
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        hash_input.hash(&mut hasher);
        format!("csrf_{:x}", hasher.finish())
    }

    /// Get current Unix timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs()
    }

    /// Validate CSRF token
    pub fn validate_csrf_token(&self, session_id: &str, token: &str) -> bool {
        if let Some(session) = self.get_session(session_id) {
            session.csrf_token == token
        } else {
            false
        }
    }

    /// Set session timeout
    pub fn set_timeout(&mut self, timeout_seconds: u64) {
        self.session_timeout = Duration::from_secs(timeout_seconds);
    }

    /// Get statistics about sessions
    pub fn get_statistics(&self) -> SessionStatistics {
        if let Ok(sessions) = self.sessions.lock() {
            let total_sessions = sessions.len();
            let authenticated_sessions = sessions.values()
                .filter(|s| s.authenticated)
                .count();
            let expired_sessions = sessions.values()
                .filter(|s| self.is_session_expired(s))
                .count();

            SessionStatistics {
                total_sessions,
                authenticated_sessions,
                expired_sessions,
                active_sessions: total_sessions - expired_sessions,
            }
        } else {
            SessionStatistics::default()
        }
    }
}

impl Session {
    /// Check if session has specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.authenticated && self.permissions.contains(&permission.to_string())
    }

    /// Check if session is authenticated
    pub const fn is_authenticated(&self) -> bool {
        self.authenticated && self.user_id.is_some()
    }

    /// Get session age in seconds
    pub fn get_age(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        now.saturating_sub(self.created_at)
    }

    /// Get time since last access in seconds
    pub fn get_idle_time(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        now.saturating_sub(self.last_accessed)
    }

    /// Set session data
    pub fn set_data(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    /// Get session data
    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    /// Remove session data
    pub fn remove_data(&mut self, key: &str) -> Option<String> {
        self.data.remove(key)
    }
}

/// Session statistics
#[derive(Debug, Default)]
pub struct SessionStatistics {
    pub total_sessions: usize,
    pub authenticated_sessions: usize,
    pub expired_sessions: usize,
    pub active_sessions: usize,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::HttpRequest;

    fn create_test_request() -> HttpRequest {
        let mut request = HttpRequest::new("GET".to_string(), "/".to_string());
        request.headers.insert("user-agent".to_string(), "QMS Test".to_string());
        request
    }

    #[test]
    fn test_session_manager_creation() {
        let manager = SessionManager::new();
        assert_eq!(manager.get_session_count(), 0);
        
        let manager_with_timeout = SessionManager::new_with_timeout(7200);
        assert_eq!(manager_with_timeout.session_timeout.as_secs(), 7200);
    }

    #[test]
    fn test_session_creation() {
        let manager = SessionManager::new();
        let request = create_test_request();
        
        let session = manager.create_session(&request);
        assert!(!session.id.is_empty());
        assert!(!session.csrf_token.is_empty());
        assert!(!session.authenticated);
        assert!(session.user_id.is_none());
        assert_eq!(session.permissions.len(), 0);
    }

    #[test]
    fn test_get_or_create_session() {
        let manager = SessionManager::new();
        let request = create_test_request();
        
        // First call should create new session
        let session1 = manager.get_or_create_session(&request);
        assert_eq!(manager.get_session_count(), 1);
        
        // Second call without session cookie should create another session
        let session2 = manager.get_or_create_session(&request);
        assert_ne!(session1.id, session2.id);
        assert_eq!(manager.get_session_count(), 2);
    }

    #[test]
    fn test_session_retrieval() {
        let manager = SessionManager::new();
        let request = create_test_request();
        
        let session = manager.create_session(&request);
        let session_id = session.id.clone();
        
        // Should retrieve the session
        let retrieved = manager.get_session(&session_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, session_id);
        
        // Non-existent session should return None
        let missing = manager.get_session("invalid_id");
        assert!(missing.is_none());
    }

    #[test]
    fn test_session_authentication() {
        let manager = SessionManager::new();
        let request = create_test_request();
        
        let session = manager.create_session(&request);
        let session_id = session.id.clone();
        
        // Authenticate session
        let permissions = vec!["read_documents".to_string(), "write_documents".to_string()];
        let result = manager.authenticate_session(&session_id, "test_user", permissions.clone());
        
        assert!(result.is_ok());
        let auth_session = result.unwrap();
        assert!(auth_session.authenticated);
        assert_eq!(auth_session.user_id, Some("test_user".to_string()));
        assert_eq!(auth_session.permissions, permissions);
    }

    #[test]
    fn test_session_removal() {
        let manager = SessionManager::new();
        let request = create_test_request();
        
        let session = manager.create_session(&request);
        let session_id = session.id.clone();
        
        // Session should exist
        assert!(manager.get_session(&session_id).is_some());
        
        // Remove session
        let removed = manager.remove_session(&session_id);
        assert!(removed);
        
        // Session should no longer exist
        assert!(manager.get_session(&session_id).is_none());
        
        // Removing non-existent session should return false
        let not_removed = manager.remove_session("invalid_id");
        assert!(!not_removed);
    }

    #[test]
    fn test_session_permissions() {
        let mut session = Session {
            id: "test_session".to_string(),
            user_id: Some("test_user".to_string()),
            created_at: 0,
            last_accessed: 0,
            ip_address: None,
            user_agent: None,
            authenticated: true,
            csrf_token: "test_csrf".to_string(),
            permissions: vec!["read_documents".to_string(), "write_risks".to_string()],
            data: HashMap::new(),
        };

        assert!(session.has_permission("read_documents"));
        assert!(session.has_permission("write_risks"));
        assert!(!session.has_permission("delete_users"));
        assert!(session.is_authenticated());

        // Test unauthenticated session
        session.authenticated = false;
        assert!(!session.has_permission("read_documents"));
        assert!(!session.is_authenticated());
    }

    #[test]
    fn test_session_data() {
        let mut session = Session {
            id: "test_session".to_string(),
            user_id: None,
            created_at: 0,
            last_accessed: 0,
            ip_address: None,
            user_agent: None,
            authenticated: false,
            csrf_token: "test_csrf".to_string(),
            permissions: Vec::new(),
            data: HashMap::new(),
        };

        // Set and get data
        session.set_data("key1", "value1");
        session.set_data("key2", "value2");
        
        assert_eq!(session.get_data("key1"), Some(&"value1".to_string()));
        assert_eq!(session.get_data("key2"), Some(&"value2".to_string()));
        assert_eq!(session.get_data("key3"), None);

        // Remove data
        let removed = session.remove_data("key1");
        assert_eq!(removed, Some("value1".to_string()));
        assert_eq!(session.get_data("key1"), None);
    }

    #[test]
    fn test_csrf_token_validation() {
        let manager = SessionManager::new();
        let request = create_test_request();
        
        let session = manager.create_session(&request);
        let session_id = session.id.clone();
        let csrf_token = session.csrf_token.clone();
        
        // Valid CSRF token should pass
        assert!(manager.validate_csrf_token(&session_id, &csrf_token));
        
        // Invalid CSRF token should fail
        assert!(!manager.validate_csrf_token(&session_id, "invalid_token"));
        
        // Invalid session ID should fail
        assert!(!manager.validate_csrf_token("invalid_session", &csrf_token));
    }

    #[test]
    fn test_session_statistics() {
        let manager = SessionManager::new();
        let request = create_test_request();
        
        // Create some sessions
        let session1 = manager.create_session(&request);
        let session2 = manager.create_session(&request);
        
        // Authenticate one session
        let _ = manager.authenticate_session(&session1.id, "user1", vec!["perm1".to_string()]);
        
        let stats = manager.get_statistics();
        assert_eq!(stats.total_sessions, 2);
        assert_eq!(stats.authenticated_sessions, 1);
        assert_eq!(stats.active_sessions, 2);
        assert_eq!(stats.expired_sessions, 0);
    }

    #[test]
    fn test_active_sessions_listing() {
        let manager = SessionManager::new();
        let request = create_test_request();
        
        // Create sessions
        let _session1 = manager.create_session(&request);
        let _session2 = manager.create_session(&request);
        
        let active_sessions = manager.get_active_sessions();
        assert_eq!(active_sessions.len(), 2);
    }
}
