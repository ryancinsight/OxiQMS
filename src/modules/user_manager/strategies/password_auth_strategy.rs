// Password-based authentication strategy
// Implements UserAuthenticator interface using password hashing

use crate::prelude::*;
use crate::models::User;
use crate::modules::user_manager::interfaces::{UserAuthenticator, UserStorage, AuthenticationResult, UserSession};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Password-based authentication strategy
pub struct PasswordAuthenticationStrategy<S: UserStorage> {
    storage: S,
    min_password_length: usize,
    require_special_chars: bool,
}

impl<S: UserStorage> PasswordAuthenticationStrategy<S> {
    /// Create new password authentication strategy
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            min_password_length: 8,
            require_special_chars: false,
        }
    }
    
    /// Create with custom password requirements
    pub fn with_requirements(storage: S, min_length: usize, require_special: bool) -> Self {
        Self {
            storage,
            min_password_length: min_length,
            require_special_chars: require_special,
        }
    }
    
    /// Check if password contains special characters
    fn has_special_chars(&self, password: &str) -> bool {
        password.chars().any(|c| !c.is_alphanumeric())
    }
}

impl<S: UserStorage> UserAuthenticator for PasswordAuthenticationStrategy<S> {
    fn authenticate(&self, username: &str, password: &str) -> QmsResult<AuthenticationResult> {
        // Load user from storage
        let user = match self.storage.load_user(username) {
            Ok(user) => user,
            Err(_) => {
                return Ok(AuthenticationResult {
                    success: false,
                    user: None,
                    session: None,
                    message: "Invalid credentials".to_string(),
                });
            }
        };
        
        // Verify password
        if !self.verify_password(password, &user.password_hash) {
            return Ok(AuthenticationResult {
                success: false,
                user: None,
                session: None,
                message: "Invalid credentials".to_string(),
            });
        }
        
        // Create session
        let session = UserSession {
            session_id: UserSession::generate_session_id(),
            user_id: user.username.clone(),
            username: user.username.clone(),
            roles: user.roles.clone(),
            login_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            last_activity: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            ip_address: None,
            expires_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs() + 24 * 3600, // 24 hours
            is_active: true,
        };
        
        Ok(AuthenticationResult {
            success: true,
            user: Some(user),
            session: Some(session),
            message: "Authentication successful".to_string(),
        })
    }
    
    fn validate_password(&self, password: &str) -> QmsResult<()> {
        if password.len() < self.min_password_length {
            return Err(QmsError::validation_error(&format!(
                "Password must be at least {} characters long",
                self.min_password_length
            )));
        }
        
        if self.require_special_chars && !self.has_special_chars(password) {
            return Err(QmsError::validation_error(
                "Password must contain at least one special character"
            ));
        }
        
        Ok(())
    }
    
    fn hash_password(&self, password: &str) -> String {
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
    
    fn verify_password(&self, password: &str, hash: &str) -> bool {
        self.hash_password(password) == hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{User, Role, Permission};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    
    // Mock storage for testing
    struct MockUserStorage {
        users: Arc<Mutex<HashMap<String, User>>>,
    }
    
    impl MockUserStorage {
        fn new() -> Self {
            Self {
                users: Arc::new(Mutex::new(HashMap::new())),
            }
        }
        
        fn add_test_user(&self, username: &str, password_hash: &str) {
            let user = User {
                username: username.to_string(),
                password_hash: password_hash.to_string(),
                roles: vec![Role {
                    name: "TestRole".to_string(),
                    permissions: vec![Permission::ReadDocuments],
                }],
                created_at: 0,
                last_login: None,
            };
            
            self.users.lock().unwrap().insert(username.to_string(), user);
        }
    }
    
    impl UserStorage for MockUserStorage {
        fn save_user(&self, user: &User) -> QmsResult<()> {
            self.users.lock().unwrap().insert(user.username.clone(), user.clone());
            Ok(())
        }
        
        fn load_user(&self, username: &str) -> QmsResult<User> {
            self.users.lock().unwrap()
                .get(username)
                .cloned()
                .ok_or_else(|| QmsError::not_found("User not found"))
        }
        
        fn user_exists(&self, username: &str) -> QmsResult<bool> {
            Ok(self.users.lock().unwrap().contains_key(username))
        }
        
        fn list_users(&self) -> QmsResult<Vec<User>> {
            Ok(self.users.lock().unwrap().values().cloned().collect())
        }
        
        fn delete_user(&self, username: &str) -> QmsResult<()> {
            self.users.lock().unwrap().remove(username);
            Ok(())
        }
        
        fn update_user(&self, user: &User) -> QmsResult<()> {
            self.save_user(user)
        }
    }
    
    #[test]
    fn test_password_authentication() {
        let storage = MockUserStorage::new();
        let auth_strategy = PasswordAuthenticationStrategy::new(storage);
        
        // Add test user with known password hash
        let password = "testpassword123";
        let hash = auth_strategy.hash_password(password);
        auth_strategy.storage.add_test_user("testuser", &hash);
        
        // Test successful authentication
        let result = auth_strategy.authenticate("testuser", password).unwrap();
        assert!(result.success);
        assert!(result.user.is_some());
        assert!(result.session.is_some());
        
        // Test failed authentication
        let result = auth_strategy.authenticate("testuser", "wrongpassword").unwrap();
        assert!(!result.success);
        assert!(result.user.is_none());
        assert!(result.session.is_none());
    }
    
    #[test]
    fn test_password_validation() {
        let storage = MockUserStorage::new();
        let auth_strategy = PasswordAuthenticationStrategy::new(storage);
        
        // Test valid password
        assert!(auth_strategy.validate_password("validpassword123").is_ok());
        
        // Test short password
        assert!(auth_strategy.validate_password("short").is_err());
        
        // Test with special character requirements
        let auth_strategy = PasswordAuthenticationStrategy::with_requirements(
            MockUserStorage::new(), 8, true
        );
        
        assert!(auth_strategy.validate_password("password123!").is_ok());
        assert!(auth_strategy.validate_password("password123").is_err());
    }
    
    #[test]
    fn test_password_hashing() {
        let storage = MockUserStorage::new();
        let auth_strategy = PasswordAuthenticationStrategy::new(storage);
        
        let password = "testpassword";
        let hash1 = auth_strategy.hash_password(password);
        let hash2 = auth_strategy.hash_password(password);
        
        // Same password should produce same hash
        assert_eq!(hash1, hash2);
        
        // Verification should work
        assert!(auth_strategy.verify_password(password, &hash1));
        assert!(!auth_strategy.verify_password("wrongpassword", &hash1));
    }
}
