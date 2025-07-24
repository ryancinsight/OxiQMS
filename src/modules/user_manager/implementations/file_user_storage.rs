// File-based user storage implementation
// Implements UserStorage interface with JSON file persistence

use crate::prelude::*;
use crate::models::{User, Role, Permission};
use crate::modules::user_manager::interfaces::UserStorage;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// File-based user storage implementation
pub struct FileUserStorage {
    users_path: PathBuf,
}

impl FileUserStorage {
    /// Create new file user storage
    pub fn new(project_path: &Path) -> QmsResult<Self> {
        let users_path = project_path.join("users").join("users.json");
        
        // Ensure users directory exists
        if let Some(parent) = users_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Initialize users file if it doesn't exist
        if !users_path.exists() {
            Self::initialize_users_file(&users_path)?;
        }
        
        Ok(Self { users_path })
    }
    
    /// Initialize users file with default admin user
    fn initialize_users_file(path: &Path) -> QmsResult<()> {
        let admin_user = User {
            username: "admin".to_string(),
            password_hash: Self::hash_password("admin123"),
            roles: vec![Self::get_admin_role()],
            created_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            last_login: None,
        };

        // Manual JSON construction to avoid external dependencies
        let admin_role = &admin_user.roles[0];
        let permissions_json = admin_role.permissions.iter()
            .map(|p| format!("\"{}\"", format!("{:?}", p)))
            .collect::<Vec<_>>()
            .join(",");

        let users_json = format!(
            r#"{{
  "version": "1.0",
  "users": [
    {{
      "username": "{}",
      "password_hash": "{}",
      "roles": [
        {{
          "name": "{}",
          "permissions": [{}]
        }}
      ],
      "created_at": {},
      "last_login": null
    }}
  ]
}}"#,
            admin_user.username,
            admin_user.password_hash,
            admin_role.name,
            permissions_json,
            admin_user.created_at
        );

        fs::write(path, users_json)?;
        Ok(())
    }
    
    /// Hash password for storage
    fn hash_password(password: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        format!("{:x}", hasher.finish())
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
    
    /// Parse users from JSON file (simple manual parsing)
    fn parse_users_json(&self, content: &str) -> QmsResult<Vec<User>> {
        // Find the users array in the JSON
        let users_start = content.find("\"users\":")
            .ok_or_else(|| QmsError::io_error("Invalid users JSON format - no users array"))?;

        let array_start = content[users_start..].find('[')
            .ok_or_else(|| QmsError::io_error("Invalid users JSON format - no array start"))?;

        let array_end = content[users_start + array_start..].rfind(']')
            .ok_or_else(|| QmsError::io_error("Invalid users JSON format - no array end"))?;

        let users_content = &content[users_start + array_start + 1..users_start + array_start + array_end];

        // Parse individual user objects
        let mut users = Vec::new();
        let mut brace_count = 0;
        let mut current_user = String::new();
        let mut in_string = false;
        let mut escape_next = false;

        for ch in users_content.chars() {
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

            if !in_string {
                if ch == '{' {
                    brace_count += 1;
                } else if ch == '}' {
                    brace_count -= 1;
                }
            }

            current_user.push(ch);

            if brace_count == 0 && !current_user.trim().is_empty() {
                if let Some(user) = self.parse_single_user(&current_user)? {
                    users.push(user);
                }
                current_user.clear();
            }
        }

        Ok(users)
    }
    
    /// Parse single user from JSON string
    fn parse_single_user(&self, json_str: &str) -> QmsResult<Option<User>> {
        let trimmed = json_str.trim();
        if trimmed.is_empty() || trimmed == "," {
            return Ok(None);
        }

        // Extract field values using simple string parsing
        let username = self.extract_string_field(trimmed, "username")?;
        let password_hash = self.extract_string_field(trimmed, "password_hash")?;
        let created_at = self.extract_number_field(trimmed, "created_at")?;
        let last_login = self.extract_optional_number_field(trimmed, "last_login");

        // Parse roles array
        let roles = self.parse_roles_from_json(trimmed)?;

        Ok(Some(User {
            username,
            password_hash,
            roles,
            created_at,
            last_login,
        }))
    }

    /// Extract string field from JSON
    fn extract_string_field(&self, json: &str, field_name: &str) -> QmsResult<String> {
        let pattern = format!("\"{}\":", field_name);
        let start = json.find(&pattern)
            .ok_or_else(|| QmsError::io_error(&format!("Missing field: {field_name}")))?;

        let value_start = json[start + pattern.len()..].find('"')
            .ok_or_else(|| QmsError::io_error(&format!("Invalid string value for: {field_name}")))?;

        let value_end = json[start + pattern.len() + value_start + 1..].find('"')
            .ok_or_else(|| QmsError::io_error(&format!("Unterminated string for: {field_name}")))?;

        let value = &json[start + pattern.len() + value_start + 1..start + pattern.len() + value_start + 1 + value_end];
        Ok(value.to_string())
    }

    /// Extract number field from JSON
    fn extract_number_field(&self, json: &str, field_name: &str) -> QmsResult<u64> {
        let pattern = format!("\"{}\":", field_name);
        let start = json.find(&pattern)
            .ok_or_else(|| QmsError::io_error(&format!("Missing field: {field_name}")))?;

        let after_colon = &json[start + pattern.len()..].trim_start();
        let mut end = 0;
        for (i, ch) in after_colon.chars().enumerate() {
            if ch.is_ascii_digit() {
                end = i + 1;
            } else {
                break;
            }
        }

        if end == 0 {
            return Err(QmsError::io_error(&format!("Invalid number value for: {field_name}")));
        }

        after_colon[..end].parse()
            .map_err(|_| QmsError::io_error(&format!("Failed to parse number for: {field_name}")))
    }

    /// Extract optional number field from JSON
    fn extract_optional_number_field(&self, json: &str, field_name: &str) -> Option<u64> {
        let pattern = format!("\"{}\":", field_name);
        if let Some(start) = json.find(&pattern) {
            let after_colon = json[start + pattern.len()..].trim_start();
            if after_colon.starts_with("null") {
                return None;
            }

            let mut end = 0;
            for (i, ch) in after_colon.chars().enumerate() {
                if ch.is_ascii_digit() {
                    end = i + 1;
                } else {
                    break;
                }
            }

            if end > 0 {
                return after_colon[..end].parse().ok();
            }
        }
        None
    }
    
    /// Parse roles from JSON string
    fn parse_roles_from_json(&self, json: &str) -> QmsResult<Vec<Role>> {
        // Find roles array
        let roles_start = json.find("\"roles\":")
            .ok_or_else(|| QmsError::io_error("Missing roles field"))?;

        let array_start = json[roles_start..].find('[')
            .ok_or_else(|| QmsError::io_error("Invalid roles array"))?;

        let array_end = json[roles_start + array_start..].rfind(']')
            .ok_or_else(|| QmsError::io_error("Unterminated roles array"))?;

        let roles_content = &json[roles_start + array_start + 1..roles_start + array_start + array_end];

        let mut roles = Vec::new();
        let mut brace_count = 0;
        let mut current_role = String::new();
        let mut in_string = false;
        let mut escape_next = false;

        for ch in roles_content.chars() {
            if escape_next {
                current_role.push(ch);
                escape_next = false;
                continue;
            }

            if ch == '\\' {
                escape_next = true;
                current_role.push(ch);
                continue;
            }

            if ch == '"' {
                in_string = !in_string;
            }

            if !in_string {
                if ch == '{' {
                    brace_count += 1;
                } else if ch == '}' {
                    brace_count -= 1;
                }
            }

            current_role.push(ch);

            if brace_count == 0 && !current_role.trim().is_empty() {
                if let Some(role) = self.parse_single_role(&current_role)? {
                    roles.push(role);
                }
                current_role.clear();
            }
        }

        Ok(roles)
    }

    /// Parse single role from JSON string
    fn parse_single_role(&self, json_str: &str) -> QmsResult<Option<Role>> {
        let trimmed = json_str.trim();
        if trimmed.is_empty() || trimmed == "," {
            return Ok(None);
        }

        let name = self.extract_string_field(trimmed, "name")?;
        let permissions = self.parse_permissions_from_json(trimmed)?;

        Ok(Some(Role { name, permissions }))
    }

    /// Parse permissions array from JSON string
    fn parse_permissions_from_json(&self, json: &str) -> QmsResult<Vec<Permission>> {
        let perms_start = json.find("\"permissions\":")
            .ok_or_else(|| QmsError::io_error("Missing permissions field"))?;

        let array_start = json[perms_start..].find('[')
            .ok_or_else(|| QmsError::io_error("Invalid permissions array"))?;

        let array_end = json[perms_start + array_start..].find(']')
            .ok_or_else(|| QmsError::io_error("Unterminated permissions array"))?;

        let perms_content = &json[perms_start + array_start + 1..perms_start + array_start + array_end];

        let mut permissions = Vec::new();
        let mut current_perm = String::new();
        let mut in_string = false;
        let mut escape_next = false;

        for ch in perms_content.chars() {
            if escape_next {
                current_perm.push(ch);
                escape_next = false;
                continue;
            }

            if ch == '\\' {
                escape_next = true;
                current_perm.push(ch);
                continue;
            }

            if ch == '"' {
                if in_string {
                    // End of string - parse permission
                    if !current_perm.is_empty() {
                        permissions.push(self.parse_permission(&current_perm)?);
                        current_perm.clear();
                    }
                }
                in_string = !in_string;
            } else if in_string {
                current_perm.push(ch);
            }
        }

        Ok(permissions)
    }
    
    /// Convert permission to string for serialization
    fn permission_to_string(&self, permission: &Permission) -> &str {
        match permission {
            Permission::ReadDocuments => "ReadDocuments",
            Permission::WriteDocuments => "WriteDocuments",
            Permission::DeleteDocuments => "DeleteDocuments",
            Permission::ReadRisks => "ReadRisks",
            Permission::WriteRisks => "WriteRisks",
            Permission::DeleteRisks => "DeleteRisks",
            Permission::ReadTrace => "ReadTrace",
            Permission::WriteTrace => "WriteTrace",
            Permission::DeleteTrace => "DeleteTrace",
            Permission::ReadAudit => "ReadAudit",
            Permission::ExportAudit => "ExportAudit",
            Permission::ManageUsers => "ManageUsers",
            Permission::GenerateReports => "GenerateReports",
            // Additional permissions for global user management
            Permission::UserManagement => "UserManagement",
            Permission::ProjectManagement => "ProjectManagement",
            Permission::DocumentManagement => "DocumentManagement",
            Permission::RiskManagement => "RiskManagement",
            Permission::AuditAccess => "AuditAccess",
            Permission::SystemConfiguration => "SystemConfiguration",
        }
    }

    /// Parse permission from string
    fn parse_permission(&self, perm_str: &str) -> QmsResult<Permission> {
        match perm_str {
            "ReadDocuments" => Ok(Permission::ReadDocuments),
            "WriteDocuments" => Ok(Permission::WriteDocuments),
            "DeleteDocuments" => Ok(Permission::DeleteDocuments),
            "ReadRisks" => Ok(Permission::ReadRisks),
            "WriteRisks" => Ok(Permission::WriteRisks),
            "DeleteRisks" => Ok(Permission::DeleteRisks),
            "ReadTrace" => Ok(Permission::ReadTrace),
            "WriteTrace" => Ok(Permission::WriteTrace),
            "DeleteTrace" => Ok(Permission::DeleteTrace),
            "ReadAudit" => Ok(Permission::ReadAudit),
            "ExportAudit" => Ok(Permission::ExportAudit),
            "ManageUsers" => Ok(Permission::ManageUsers),
            "GenerateReports" => Ok(Permission::GenerateReports),
            // Additional permissions for global user management
            "UserManagement" => Ok(Permission::UserManagement),
            "ProjectManagement" => Ok(Permission::ProjectManagement),
            "DocumentManagement" => Ok(Permission::DocumentManagement),
            "RiskManagement" => Ok(Permission::RiskManagement),
            "AuditAccess" => Ok(Permission::AuditAccess),
            "SystemConfiguration" => Ok(Permission::SystemConfiguration),
            _ => Err(QmsError::validation_error(&format!("Unknown permission: {perm_str}"))),
        }
    }
    
    /// Save users to JSON file (manual JSON construction)
    fn save_users_json(&self, users: &[User]) -> QmsResult<()> {
        let mut json = String::from("{\n  \"version\": \"1.0\",\n  \"users\": [\n");

        for (i, user) in users.iter().enumerate() {
            if i > 0 {
                json.push_str(",\n");
            }

            json.push_str("    {\n");
            json.push_str(&format!("      \"username\": \"{}\",\n", user.username));
            json.push_str(&format!("      \"password_hash\": \"{}\",\n", user.password_hash));
            json.push_str("      \"roles\": [\n");

            for (j, role) in user.roles.iter().enumerate() {
                if j > 0 {
                    json.push_str(",\n");
                }

                json.push_str("        {\n");
                json.push_str(&format!("          \"name\": \"{}\",\n", role.name));
                json.push_str("          \"permissions\": [");

                for (k, permission) in role.permissions.iter().enumerate() {
                    if k > 0 {
                        json.push_str(",");
                    }
                    json.push_str(&format!("\"{}\"", self.permission_to_string(permission)));
                }

                json.push_str("]\n        }");
            }

            json.push_str("\n      ],\n");
            json.push_str(&format!("      \"created_at\": {},\n", user.created_at));

            match user.last_login {
                Some(login_time) => json.push_str(&format!("      \"last_login\": {}\n", login_time)),
                None => json.push_str("      \"last_login\": null\n"),
            }

            json.push_str("    }");
        }

        json.push_str("\n  ]\n}");

        fs::write(&self.users_path, json)?;
        Ok(())
    }
}

impl UserStorage for FileUserStorage {
    fn save_user(&self, user: &User) -> QmsResult<()> {
        let content = fs::read_to_string(&self.users_path)?;
        let mut users = self.parse_users_json(&content)?;
        
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
        let content = fs::read_to_string(&self.users_path)?;
        let users = self.parse_users_json(&content)?;
        
        users.into_iter()
            .find(|user| user.username == username)
            .ok_or_else(|| QmsError::not_found(&format!("User '{username}' not found")))
    }
    
    fn user_exists(&self, username: &str) -> QmsResult<bool> {
        match self.load_user(username) {
            Ok(_) => Ok(true),
            Err(QmsError::NotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }
    
    fn list_users(&self) -> QmsResult<Vec<User>> {
        let content = fs::read_to_string(&self.users_path)?;
        self.parse_users_json(&content)
    }
    
    fn delete_user(&self, username: &str) -> QmsResult<()> {
        let content = fs::read_to_string(&self.users_path)?;
        let mut users = self.parse_users_json(&content)?;
        
        let initial_len = users.len();
        users.retain(|user| user.username != username);
        
        if users.len() == initial_len {
            return Err(QmsError::not_found(&format!("User '{username}' not found")));
        }
        
        self.save_users_json(&users)?;
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
    use tempfile::tempdir;
    
    #[test]
    fn test_file_storage_creation() {
        let temp_dir = tempdir().unwrap();
        let storage = FileUserStorage::new(temp_dir.path()).unwrap();
        
        // Should create users.json with default admin user
        let users_path = temp_dir.path().join("users").join("users.json");
        assert!(users_path.exists());
        
        // Should have one user (admin)
        let users = storage.list_users().unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].username, "admin");
    }
    
    #[test]
    fn test_user_crud_operations() {
        let temp_dir = tempdir().unwrap();
        let storage = FileUserStorage::new(temp_dir.path()).unwrap();
        
        // Create test user
        let test_user = User {
            username: "testuser".to_string(),
            password_hash: "hash123".to_string(),
            roles: vec![Role {
                name: "TestRole".to_string(),
                permissions: vec![Permission::ReadDocuments],
            }],
            created_at: 1234567890,
            last_login: None,
        };
        
        // Save user
        storage.save_user(&test_user).unwrap();
        
        // Load user
        let loaded_user = storage.load_user("testuser").unwrap();
        assert_eq!(loaded_user.username, "testuser");
        assert_eq!(loaded_user.password_hash, "hash123");
        
        // Check user exists
        assert!(storage.user_exists("testuser").unwrap());
        assert!(!storage.user_exists("nonexistent").unwrap());
        
        // Update user
        let mut updated_user = loaded_user.clone();
        updated_user.last_login = Some(9876543210);
        storage.update_user(&updated_user).unwrap();
        
        let reloaded_user = storage.load_user("testuser").unwrap();
        assert_eq!(reloaded_user.last_login, Some(9876543210));
        
        // Delete user
        storage.delete_user("testuser").unwrap();
        assert!(storage.load_user("testuser").is_err());
    }
    
    #[test]
    fn test_list_users() {
        let temp_dir = tempdir().unwrap();
        let storage = FileUserStorage::new(temp_dir.path()).unwrap();
        
        // Should start with admin user
        let users = storage.list_users().unwrap();
        assert_eq!(users.len(), 1);
        
        // Add another user
        let test_user = User {
            username: "testuser".to_string(),
            password_hash: "hash123".to_string(),
            roles: vec![],
            created_at: 1234567890,
            last_login: None,
        };
        storage.save_user(&test_user).unwrap();
        
        // Should now have two users
        let users = storage.list_users().unwrap();
        assert_eq!(users.len(), 2);
    }
}
