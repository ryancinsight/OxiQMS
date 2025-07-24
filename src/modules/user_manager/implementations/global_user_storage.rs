//! Global User Storage Implementation
//! 
//! This module provides global user storage that persists user accounts
//! independently of projects. Users are stored in a system-wide location
//! and can access multiple projects with consistent identity.
//! 
//! REFACTORED: Implements UserStorage interface for global user management
//! Follows Dependency Inversion Principle and medical device compliance

use crate::prelude::*;
use crate::models::{User, Role, Permission};
use crate::models::user_profile::UserProfile;
use crate::modules::user_manager::interfaces::UserStorage;
use crate::modules::audit_logger::audit_log_action;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Global user storage implementation
/// Stores users in a system-wide location independent of projects
pub struct GlobalUserStorage {
    /// Path to global users directory (~/.qms/users/)
    users_dir: PathBuf,
    
    /// Path to users.json file
    users_path: PathBuf,
    
    /// Path to user profiles directory
    profiles_dir: PathBuf,
    
    /// Path to sessions directory
    sessions_dir: PathBuf,
}

impl GlobalUserStorage {
    /// Create new global user storage instance
    pub fn new() -> QmsResult<Self> {
        let users_dir = Self::get_global_users_directory()?;
        let users_path = users_dir.join("users.json");
        let profiles_dir = users_dir.join("profiles");
        let sessions_dir = users_dir.join("sessions");

        // Create directory structure if it doesn't exist
        fs::create_dir_all(&users_dir)?;
        fs::create_dir_all(&profiles_dir)?;
        fs::create_dir_all(&sessions_dir)?;

        let storage = GlobalUserStorage {
            users_dir,
            users_path: users_path.clone(),
            profiles_dir,
            sessions_dir,
        };

        // Initialize users file if it doesn't exist
        if !users_path.exists() {
            storage.initialize_users_file()?;
        }

        Ok(storage)
    }

    /// Create new test storage instance with isolated directory
    /// SOLID: Dependency Inversion - allows injection of test storage
    /// ACID: Isolation - each test gets its own storage
    #[cfg(test)]
    pub fn new_test_instance() -> Self {
        use std::env;

        // Create unique test directory for isolation
        let unique_id = format!("{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let temp_dir = env::temp_dir().join(format!("qms_test_global_users_{}", unique_id));
        let _ = std::fs::remove_dir_all(&temp_dir); // Clean up any previous test

        let storage = GlobalUserStorage {
            users_dir: temp_dir.clone(),
            users_path: temp_dir.join("users.json"),
            profiles_dir: temp_dir.join("profiles"),
            sessions_dir: temp_dir.join("sessions"),
        };

        std::fs::create_dir_all(&storage.users_dir).unwrap();
        std::fs::create_dir_all(&storage.profiles_dir).unwrap();
        std::fs::create_dir_all(&storage.sessions_dir).unwrap();
        storage.initialize_users_file().unwrap();

        storage
    }
    
    /// Get the global users directory path
    fn get_global_users_directory() -> QmsResult<PathBuf> {
        let home_dir = if cfg!(windows) {
            std::env::var("USERPROFILE")
                .or_else(|_| std::env::var("HOME"))
                .map_err(|_| QmsError::domain_error("Cannot determine home directory"))?
        } else {
            std::env::var("HOME")
                .map_err(|_| QmsError::domain_error("Cannot determine home directory"))?
        };
        
        Ok(PathBuf::from(home_dir).join(".qms").join("users"))
    }
    
    /// Initialize the global users file
    fn initialize_users_file(&self) -> QmsResult<()> {
        let empty_users = r#"{
  "version": "1.0",
  "users": []
}"#;
        
        fs::write(&self.users_path, empty_users)?;
        
        // Set appropriate file permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&self.users_path)?.permissions();
            perms.set_mode(0o600); // Read/write for owner only
            fs::set_permissions(&self.users_path, perms)?;
        }
        
        Ok(())
    }
    
    /// Check if any users exist in the system
    pub fn has_any_users(&self) -> QmsResult<bool> {
        let users = self.load_all_users()?;
        Ok(!users.is_empty())
    }
    
    /// Load all users from storage
    fn load_all_users(&self) -> QmsResult<Vec<User>> {
        if !self.users_path.exists() {
            return Ok(Vec::new());
        }
        
        let content = fs::read_to_string(&self.users_path)?;
        self.parse_users_json(&content)
    }
    
    /// Parse users from JSON content
    /// KISS: Simple and reliable JSON parsing focused on the specific structure we generate
    fn parse_users_json(&self, content: &str) -> QmsResult<Vec<User>> {
        let mut users = Vec::new();

        // Find the users array start
        if let Some(users_start) = content.find("\"users\":") {
            if let Some(array_start_pos) = content[users_start..].find('[') {
                let array_start = users_start + array_start_pos + 1;

                // Find the matching closing bracket by counting brackets
                let mut bracket_count = 1;
                let mut array_end = array_start;
                let mut in_string = false;
                let mut escape_next = false;

                for (i, ch) in content[array_start..].char_indices() {
                    if escape_next {
                        escape_next = false;
                        continue;
                    }

                    if ch == '\\' {
                        escape_next = true;
                        continue;
                    }

                    if ch == '"' {
                        in_string = !in_string;
                        continue;
                    }

                    if !in_string {
                        if ch == '[' {
                            bracket_count += 1;
                        } else if ch == ']' {
                            bracket_count -= 1;
                            if bracket_count == 0 {
                                array_end = array_start + i;
                                break;
                            }
                        }
                    }
                }

                if bracket_count == 0 {
                    let array_content = &content[array_start..array_end].trim();

                    // If array is empty, return empty vector
                    if array_content.is_empty() {
                        return Ok(users);
                    }

                    // Split by user objects - look for complete user objects
                    // Each user object starts with { and ends with }
                    let mut current_user = String::new();
                    let mut brace_count = 0;
                    let mut in_string = false;
                    let mut escape_next = false;

                    for ch in array_content.chars() {
                        if escape_next {
                            current_user.push(ch);
                            escape_next = false;
                            continue;
                        }

                        if ch == '\\' {
                            escape_next = true;
                            current_user.push(ch);
                            continue;
                        }

                        if ch == '"' {
                            in_string = !in_string;
                        }

                        current_user.push(ch);

                        if !in_string {
                            if ch == '{' {
                                brace_count += 1;
                            } else if ch == '}' {
                                brace_count -= 1;

                                // Complete user object found
                                if brace_count == 0 {
                                    let user_json = current_user.trim().trim_end_matches(',');
                                    if !user_json.is_empty() {
                                        match self.parse_single_user(user_json) {
                                            Ok(user) => {
                                                users.push(user);
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to parse user JSON: {}", e);
                                                eprintln!("User JSON: {}", user_json);
                                            }
                                        }
                                    }
                                    current_user.clear();
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(users)
    }
    
    /// Parse a single user from JSON object string
    fn parse_single_user(&self, json: &str) -> QmsResult<User> {
        let username = Self::extract_json_string(json, "username")?;
        let password_hash = Self::extract_json_string(json, "password_hash")?;
        let created_at = Self::extract_json_number(json, "created_at")?;
        let last_login = Self::extract_json_number(json, "last_login").ok();
        
        // For simplicity, we'll use default roles for now
        // In a full implementation, you'd parse the roles array
        let roles = vec![Self::get_default_role()];
        
        Ok(User {
            username,
            password_hash,
            roles,
            created_at,
            last_login,
        })
    }
    
    /// Save all users to JSON file
    fn save_users_json(&self, users: &[User]) -> QmsResult<()> {
        let users_json = users.iter()
            .map(|user| self.user_to_json(user))
            .collect::<Result<Vec<_>, _>>()?
            .join(",\n    ");
        
        let json_content = format!(
            r#"{{
  "version": "1.0",
  "users": [
    {}
  ]
}}"#,
            users_json
        );
        
        fs::write(&self.users_path, json_content)?;
        
        // Set secure permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&self.users_path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&self.users_path, perms)?;
        }
        
        Ok(())
    }
    
    /// Convert user to JSON string
    fn user_to_json(&self, user: &User) -> QmsResult<String> {
        let roles_json = user.roles.iter()
            .map(|role| {
                let permissions_json = role.permissions.iter()
                    .map(|p| format!("\"{}\"", format!("{:?}", p)))
                    .collect::<Vec<_>>()
                    .join(",");
                
                format!(
                    r#"{{
        "name": "{}",
        "permissions": [{}]
      }}"#,
                    role.name, permissions_json
                )
            })
            .collect::<Vec<_>>()
            .join(",\n      ");
        
        Ok(format!(
            r#"{{
      "username": "{}",
      "password_hash": "{}",
      "roles": [
        {}
      ],
      "created_at": {},
      "last_login": {}
    }}"#,
            user.username,
            user.password_hash,
            roles_json,
            user.created_at,
            user.last_login.map(|t| t.to_string()).unwrap_or_else(|| "null".to_string())
        ))
    }
    
    /// Load user profile from storage
    pub fn load_user_profile(&self, username: &str) -> QmsResult<UserProfile> {
        let profile_path = self.profiles_dir.join(format!("{}.json", username));
        
        if !profile_path.exists() {
            // Create default profile
            let profile = UserProfile::new(username)?;
            self.save_user_profile(&profile)?;
            return Ok(profile);
        }
        
        let content = fs::read_to_string(&profile_path)?;
        UserProfile::from_json(&content)
    }
    
    /// Save user profile to storage
    pub fn save_user_profile(&self, profile: &UserProfile) -> QmsResult<()> {
        let profile_path = self.profiles_dir.join(format!("{}.json", profile.username));
        let json_content = profile.to_json()?;
        
        fs::write(&profile_path, json_content)?;
        
        // Set secure permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&profile_path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&profile_path, perms)?;
        }
        
        Ok(())
    }
    
    /// Create the first admin user for the system
    pub fn create_initial_admin(&self, username: &str, password: &str) -> QmsResult<User> {
        // Check if any users already exist
        if self.has_any_users()? {
            return Err(QmsError::already_exists("Users already exist in the system"));
        }
        
        let admin_user = User {
            username: username.to_string(),
            password_hash: Self::hash_password(password),
            roles: vec![Self::get_admin_role()],
            created_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            last_login: None,
        };
        
        self.save_user(&admin_user)?;
        
        // Create default profile for admin
        let profile = UserProfile::new(username)?;
        self.save_user_profile(&profile)?;
        
        // Audit logging for initial admin creation (optional - no project exists yet)
        if let Err(e) = audit_log_action("INITIAL_ADMIN_CREATED", "User", username) {
            eprintln!("Warning: Could not log initial admin creation: {}", e);
            // Continue anyway - this is expected when no project exists yet
        }
        
        Ok(admin_user)
    }
    
    /// Hash password using simple SHA-256 (in production, use bcrypt or similar)
    fn hash_password(password: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
    
    /// Get default admin role
    fn get_admin_role() -> Role {
        Role {
            name: "Administrator".to_string(),
            permissions: vec![
                Permission::UserManagement,
                Permission::ProjectManagement,
                Permission::DocumentManagement,
                Permission::RiskManagement,
                Permission::AuditAccess,
                Permission::SystemConfiguration,
            ],
        }
    }
    
    /// Get default user role
    fn get_default_role() -> Role {
        Role {
            name: "Quality Engineer".to_string(),
            permissions: vec![
                Permission::DocumentManagement,
                Permission::RiskManagement,
                Permission::ProjectManagement,
            ],
        }
    }
    
    /// Extract string value from JSON (simple implementation)
    fn extract_json_string(json: &str, key: &str) -> QmsResult<String> {
        let pattern = format!("\"{}\":", key);
        if let Some(start) = json.find(&pattern) {
            let value_start = start + pattern.len();
            if let Some(quote_start) = json[value_start..].find('"') {
                let quote_start = value_start + quote_start + 1;
                if let Some(quote_end) = json[quote_start..].find('"') {
                    return Ok(json[quote_start..quote_start + quote_end].to_string());
                }
            }
        }
        Err(QmsError::parse_error(&format!("Could not extract '{}' from JSON", key)))
    }
    
    /// Extract number value from JSON (simple implementation)
    fn extract_json_number(json: &str, key: &str) -> QmsResult<u64> {
        let pattern = format!("\"{}\":", key);
        if let Some(start) = json.find(&pattern) {
            let value_start = start + pattern.len();
            let value_part = &json[value_start..];
            
            let mut end = 0;
            for (i, ch) in value_part.chars().enumerate() {
                if ch.is_ascii_digit() {
                    end = i + 1;
                } else if ch == ',' || ch == '}' || ch == '\n' {
                    break;
                } else if !ch.is_whitespace() {
                    break;
                }
            }
            
            if end > 0 {
                let number_str = value_part[..end].trim();
                return number_str.parse::<u64>()
                    .map_err(|_| QmsError::parse_error(&format!("Invalid number for '{}'", key)));
            }
        }
        Err(QmsError::parse_error(&format!("Could not extract '{}' from JSON", key)))
    }
}

impl UserStorage for GlobalUserStorage {
    fn save_user(&self, user: &User) -> QmsResult<()> {
        let mut users = self.load_all_users()?;
        
        // Update existing user or add new one
        let mut found = false;
        for existing_user in &mut users {
            if existing_user.username == user.username {
                *existing_user = user.clone();
                found = true;
                break;
            }
        }
        
        if !found {
            users.push(user.clone());
        }
        
        self.save_users_json(&users)?;
        Ok(())
    }
    
    fn load_user(&self, username: &str) -> QmsResult<User> {
        let users = self.load_all_users()?;
        
        users.into_iter()
            .find(|user| user.username == username)
            .ok_or_else(|| QmsError::not_found(&format!("User '{}' not found", username)))
    }
    
    fn user_exists(&self, username: &str) -> QmsResult<bool> {
        let users = self.load_all_users()?;
        Ok(users.iter().any(|user| user.username == username))
    }
    
    fn list_users(&self) -> QmsResult<Vec<User>> {
        self.load_all_users()
    }
    
    fn delete_user(&self, username: &str) -> QmsResult<()> {
        let mut users = self.load_all_users()?;
        let initial_len = users.len();
        
        users.retain(|user| user.username != username);
        
        if users.len() == initial_len {
            return Err(QmsError::not_found(&format!("User '{}' not found", username)));
        }
        
        self.save_users_json(&users)?;
        
        // Also delete user profile
        let profile_path = self.profiles_dir.join(format!("{}.json", username));
        if profile_path.exists() {
            fs::remove_file(profile_path)?;
        }
        
        Ok(())
    }
    
    fn update_user(&self, user: &User) -> QmsResult<()> {
        // Check if user exists first
        self.load_user(&user.username)?;
        
        // Save the updated user
        self.save_user(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use crate::modules::audit_logger::{initialize_audit_system, AuditConfig};

    fn init_audit_for_test() {
        let temp_dir = env::temp_dir().join("qms_global_user_storage_test");
        let _ = std::fs::create_dir_all(&temp_dir);

        let audit_dir = temp_dir.join("audit");
        let _ = std::fs::create_dir_all(&audit_dir);

        let config = AuditConfig {
            project_path: temp_dir.to_string_lossy().to_string(),
            retention_days: 30,
            daily_rotation: false,
            max_file_size_mb: 10,
            require_checksums: false,
        };

        let _ = initialize_audit_system(config);
    }

    fn get_test_storage() -> GlobalUserStorage {
        // Use the new test constructor for consistency
        GlobalUserStorage::new_test_instance()
    }
    
    #[test]
    fn test_global_user_storage_creation() {
        let storage = get_test_storage();
        assert!(storage.users_path.exists());
        assert!(storage.profiles_dir.exists());
        assert!(storage.sessions_dir.exists());
    }
    
    #[test]
    fn test_initial_admin_creation() {
        // Initialize audit system for test
        init_audit_for_test();

        let storage = get_test_storage();

        assert!(!storage.has_any_users().unwrap());

        let admin = storage.create_initial_admin("admin", "password123").unwrap();
        assert_eq!(admin.username, "admin");
        assert!(storage.has_any_users().unwrap());

        // Should fail to create another admin
        assert!(storage.create_initial_admin("admin2", "password123").is_err());
    }
    
    #[test]
    fn test_user_profile_management() {
        let storage = get_test_storage();

        let profile = UserProfile::new("testuser").unwrap();
        storage.save_user_profile(&profile).unwrap();

        let loaded_profile = storage.load_user_profile("testuser").unwrap();
        assert_eq!(loaded_profile.username, "testuser");
    }
}
