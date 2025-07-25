//! In-memory user storage implementation for QMS
//! Uses Argon2 for secure password hashing

use crate::error::{QmsError, QmsResult};
use crate::modules::user_manager::interfaces::UserStorage;
use crate::models::{User, Role, Permission};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use argon2::{self, Config};
use rand::Rng;
use tracing::error;

/// Memory-based user storage implementation
/// Stores users in memory using HashMap for fast access
pub struct MemoryUserStorage {
    users: Arc<Mutex<HashMap<String, User>>>,
}

impl MemoryUserStorage {
    /// Create new memory user storage with default admin user
    pub fn new() -> QmsResult<Self> {
        let mut users = HashMap::new();
        let admin_user = Self::create_default_admin_user()?;
        users.insert(admin_user.username.clone(), admin_user);
        
        Ok(Self {
            users: Arc::new(Mutex::new(users)),
        })
    }
    
    /// Create default admin user with secure password hash
    fn create_default_admin_user() -> QmsResult<User> {
        let password_hash = Self::hash_password("admin123");
        
        Ok(User {
            username: "admin".to_string(),
            password_hash,
            roles: vec![Self::get_admin_role()],
            created_at: SystemTime::now().duration_since(UNIX_EPOCH)
                .map_err(|e| QmsError::domain_error(&format!("Failed to get current time: {e}")))?
                .as_secs(),
            last_login: None,
        })
    }
    
    /// Hash password using Argon2 for storage
    fn hash_password(password: &str) -> String {
        let salt = rand::thread_rng().gen::<[u8; 32]>();
        let config = Config::default();
        argon2::hash_encoded(password.as_bytes(), &salt, &config)
            .expect("Failed to hash password")
    }
    
    /// Get admin role with all permissions
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
    
    /// Clear all users (useful for testing)
    pub fn clear(&self) -> QmsResult<()> {
        let mut users = self.users.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire users lock"))?;
        users.clear();
        Ok(())
    }
    
    /// Get user count
    pub fn user_count(&self) -> QmsResult<usize> {
        let users = self.users.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire users lock"))?;
        Ok(users.len())
    }
}

impl Default for MemoryUserStorage {
    fn default() -> Self {
        Self::new().expect("Failed to create default MemoryUserStorage")
    }
}

impl UserStorage for MemoryUserStorage {
    /// Save user to memory storage
    fn save_user(&self, user: &User) -> QmsResult<()> {
        let mut users = self.users.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire users lock"))?;
        
        users.insert(user.username.clone(), user.clone());
        Ok(())
    }
    
    /// Load user from memory storage
    fn load_user(&self, username: &str) -> QmsResult<User> {
        let users = self.users.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire users lock"))?;
        
        users.get(username)
            .cloned()
            .ok_or_else(|| QmsError::NotFound(format!("User '{username}' not found")))
    }
    
    /// Check if user exists in memory storage
    fn user_exists(&self, username: &str) -> QmsResult<bool> {
        let users = self.users.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire users lock"))?;
        
        Ok(users.contains_key(username))
    }
    
    /// List all users from memory storage
    fn list_users(&self) -> QmsResult<Vec<User>> {
        let users = self.users.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire users lock"))?;
        
        Ok(users.values().cloned().collect())
    }
    
    /// Delete user from memory storage
    fn delete_user(&self, username: &str) -> QmsResult<()> {
        let mut users = self.users.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire users lock"))?;
        
        if users.remove(username).is_some() {
            Ok(())
        } else {
            Err(QmsError::NotFound(format!("User '{username}' not found")))
        }
    }
    
    /// Update user in memory storage
    fn update_user(&self, user: &User) -> QmsResult<()> {
        let mut users = self.users.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire users lock"))?;
        
        if users.contains_key(&user.username) {
            users.insert(user.username.clone(), user.clone());
            Ok(())
        } else {
            Err(QmsError::NotFound(format!("User '{}' not found", user.username)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_storage_creation() {
        let storage = MemoryUserStorage::new().unwrap();
        assert_eq!(storage.user_count().unwrap(), 1); // Default admin user
    }

    #[test]
    fn test_save_and_load_user() {
        let storage = MemoryUserStorage::new().unwrap();
        
        let user = User {
            username: "testuser".to_string(),
            password_hash: "hash123".to_string(),
            roles: vec![],
            created_at: 1234567890,
            last_login: None,
        };
        
        storage.save_user(&user).unwrap();
        let loaded_user = storage.load_user("testuser").unwrap();
        assert_eq!(loaded_user.username, "testuser");
    }

    #[test]
    fn test_user_exists() {
        let storage = MemoryUserStorage::new().unwrap();
        assert!(storage.user_exists("admin").unwrap());
        assert!(!storage.user_exists("nonexistent").unwrap());
    }

    #[test]
    fn test_list_users() {
        let storage = MemoryUserStorage::new().unwrap();
        let users = storage.list_users().unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].username, "admin");
    }

    #[test]
    fn test_delete_user() {
        let storage = MemoryUserStorage::new().unwrap();
        
        let user = User {
            username: "testuser".to_string(),
            password_hash: "hash123".to_string(),
            roles: vec![],
            created_at: 1234567890,
            last_login: None,
        };
        
        storage.save_user(&user).unwrap();
        assert!(storage.user_exists("testuser").unwrap());
        
        storage.delete_user("testuser").unwrap();
        assert!(!storage.user_exists("testuser").unwrap());
    }

    #[test]
    fn test_update_user() {
        let storage = MemoryUserStorage::new().unwrap();
        
        let mut user = User {
            username: "testuser".to_string(),
            password_hash: "hash123".to_string(),
            roles: vec![],
            created_at: 1234567890,
            last_login: None,
        };
        
        storage.save_user(&user).unwrap();
        
        user.password_hash = "newhash456".to_string();
        storage.update_user(&user).unwrap();
        
        let updated_user = storage.load_user("testuser").unwrap();
        assert_eq!(updated_user.password_hash, "newhash456");
    }

    #[test]
    fn test_clear_storage() {
        let storage = MemoryUserStorage::new().unwrap();
        assert_eq!(storage.user_count().unwrap(), 1);
        
        storage.clear().unwrap();
        assert_eq!(storage.user_count().unwrap(), 0);
    }
}
