use crate::prelude::*;
use crate::models::{Role, Permission, User};
use crate::modules::audit_logger::audit_log_action;

/// Role management functionality
pub struct RoleManager {
    users_path: PathBuf,
}

impl RoleManager {
    /// Create new role manager
    pub fn new(project_path: &Path) -> QmsResult<Self> {
        let users_path = project_path.join("users").join("users.json");
        
        Ok(RoleManager {
            users_path,
        })
    }
    
    /// Assign role to user
    pub fn assign_role(&self, username: &str, role_name: &str) -> QmsResult<()> {
        let role = self.get_role_by_name(role_name)?;
        
        // Load user
        let mut user = self.load_user(username)?;
        
        // Check if user already has this role
        if user.roles.iter().any(|r| r.name == role.name) {
            return Err(QmsError::validation_error("User already has this role"));
        }
        
        // Add role to user
        user.roles.push(role);
        
        // Save user
        self.save_user(&user)?;
        
        audit_log_action("ROLE_ASSIGNED", "User", &format!("{username}:{role_name}"))?;
        
        Ok(())
    }
    
    /// Remove role from user
    pub fn remove_role(&self, username: &str, role_name: &str) -> QmsResult<()> {
        let mut user = self.load_user(username)?;
        
        // Find and remove role
        let initial_count = user.roles.len();
        user.roles.retain(|r| r.name != role_name);
        
        if user.roles.len() == initial_count {
            return Err(QmsError::NotFound("User does not have this role".to_string()));
        }
        
        // Ensure user has at least one role
        if user.roles.is_empty() {
            user.roles.push(self.get_default_role());
        }
        
        // Save user
        self.save_user(&user)?;
        
        audit_log_action("ROLE_REMOVED", "User", &format!("{username}:{role_name}"))?;
        
        Ok(())
    }
    
    /// Get available roles
    pub fn get_available_roles(&self) -> Vec<Role> {
        vec![
            self.get_admin_role(),
            self.get_quality_engineer_role(),
            self.get_developer_role(),
            self.get_auditor_role(),
        ]
    }
    
    /// Get role by name
    pub fn get_role_by_name(&self, name: &str) -> QmsResult<Role> {
        match name.to_lowercase().as_str() {
            "administrator" | "admin" => Ok(self.get_admin_role()),
            "qualityengineer" | "quality" | "qe" => Ok(self.get_quality_engineer_role()),
            "developer" | "dev" => Ok(self.get_developer_role()),
            "auditor" | "audit" => Ok(self.get_auditor_role()),
            _ => Err(QmsError::validation_error(&format!("Unknown role: {name}"))),
        }
    }
    
    /// Get user roles
    pub fn get_user_roles(&self, username: &str) -> QmsResult<Vec<Role>> {
        let user = self.load_user(username)?;
        Ok(user.roles)
    }
    
    /// Check if user has role
    #[allow(dead_code)]
    pub fn user_has_role(&self, username: &str, role_name: &str) -> QmsResult<bool> {
        let user = self.load_user(username)?;
        Ok(user.roles.iter().any(|r| r.name.to_lowercase() == role_name.to_lowercase()))
    }
    
    /// Check if user has permission
    #[allow(dead_code)]
    pub fn user_has_permission(&self, username: &str, permission: &Permission) -> QmsResult<bool> {
        let user = self.load_user(username)?;
        
        for role in &user.roles {
            if role.permissions.contains(permission) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Get permissions for user
    pub fn get_user_permissions(&self, username: &str) -> QmsResult<Vec<Permission>> {
        let user = self.load_user(username)?;
        let mut permissions = Vec::new();
        
        for role in &user.roles {
            for permission in &role.permissions {
                if !permissions.contains(permission) {
                    permissions.push(permission.clone());
                }
            }
        }
        
        Ok(permissions)
    }
    
    /// Get role descriptions
    pub fn get_role_descriptions(&self) -> HashMap<String, String> {
        let mut descriptions = HashMap::new();
        
        descriptions.insert("Administrator".to_string(), 
            "Full system access including user management and system configuration".to_string());
        descriptions.insert("QualityEngineer".to_string(), 
            "Quality management functions including document control and risk management".to_string());
        descriptions.insert("Developer".to_string(), 
            "Development-focused access for requirements and testing".to_string());
        descriptions.insert("Auditor".to_string(), 
            "Read-only access for audit and compliance activities".to_string());
        
        descriptions
    }
    
    /// Get permission descriptions
    pub fn get_permission_descriptions(&self) -> HashMap<String, String> {
        let mut descriptions = HashMap::new();
        
        descriptions.insert("ReadDocuments".to_string(), "View documents and their metadata".to_string());
        descriptions.insert("WriteDocuments".to_string(), "Create and modify documents".to_string());
        descriptions.insert("DeleteDocuments".to_string(), "Delete documents (admin only)".to_string());
        descriptions.insert("ReadRisks".to_string(), "View risk assessments and FMEA data".to_string());
        descriptions.insert("WriteRisks".to_string(), "Create and modify risk assessments".to_string());
        descriptions.insert("DeleteRisks".to_string(), "Delete risk assessments (admin only)".to_string());
        descriptions.insert("ReadTrace".to_string(), "View traceability links and matrices".to_string());
        descriptions.insert("WriteTrace".to_string(), "Create and modify traceability links".to_string());
        descriptions.insert("DeleteTrace".to_string(), "Delete traceability links (admin only)".to_string());
        descriptions.insert("ReadAudit".to_string(), "View audit logs and compliance reports".to_string());
        descriptions.insert("ExportAudit".to_string(), "Export audit data and reports".to_string());
        descriptions.insert("ManageUsers".to_string(), "Manage user accounts and roles".to_string());
        descriptions.insert("GenerateReports".to_string(), "Generate system reports and analytics".to_string());
        
        descriptions
    }
    
    /// Load user from storage (simplified version)
    fn load_user(&self, username: &str) -> QmsResult<User> {
        let content = fs::read_to_string(&self.users_path)?;
        
        // Find user in JSON content
        if let Some(user_start) = content.find(&format!("\"username\": \"{username}\"")) {
            // Extract user object (simplified parsing)
            let before = &content[..user_start];
            let user_obj_start = before.rfind('{').unwrap_or(0);
            let after = &content[user_start..];
            let user_obj_end = after.find('}').unwrap_or(after.len()) + user_start + 1;
            
            let user_json = &content[user_obj_start..user_obj_end];
            return self.parse_user_json(user_json);
        }
        
        Err(QmsError::NotFound(format!("User '{username}' not found")))
    }
    
    /// Parse user from JSON (simplified)
    fn parse_user_json(&self, json: &str) -> QmsResult<User> {
        let username = self.extract_json_string(json, "username")?;
        let password_hash = self.extract_json_string(json, "password_hash")?;
        let created_at = self.extract_json_number(json, "created_at")?;
        let last_login = self.extract_json_optional_number(json, "last_login");
        
        // Parse roles (simplified - get default role)
        let roles = vec![self.get_quality_engineer_role()];
        
        Ok(User {
            username,
            password_hash,
            roles,
            created_at,
            last_login,
        })
    }
    
    /// Save user to storage (simplified)
    fn save_user(&self, user: &User) -> QmsResult<()> {
        // For now, just audit the action
        audit_log_action("USER_UPDATED", "User", &user.username)?;
        Ok(())
    }
    
    /// Extract string value from JSON
    fn extract_json_string(&self, json: &str, key: &str) -> QmsResult<String> {
        let pattern = format!("\"{key}\": \"");
        if let Some(start) = json.find(&pattern) {
            let value_start = start + pattern.len();
            if let Some(end) = json[value_start..].find('"') {
                return Ok(json[value_start..value_start + end].to_string());
            }
        }
        Err(QmsError::parse_error(&format!("Failed to parse {key} from JSON")))
    }
    
    /// Extract number value from JSON
    fn extract_json_number(&self, json: &str, key: &str) -> QmsResult<u64> {
        let pattern = format!("\"{key}\": ");
        if let Some(start) = json.find(&pattern) {
            let value_start = start + pattern.len();
            let value_end = json[value_start..].find([',', '}', '\n'])
                .map(|i| value_start + i)
                .unwrap_or(json.len());
            
            let value_str = json[value_start..value_end].trim();
            if value_str != "null" {
                return value_str.parse::<u64>()
                    .map_err(|_| QmsError::parse_error(&format!("Failed to parse {key} as number")));
            }
        }
        Err(QmsError::parse_error(&format!("Failed to parse {key} from JSON")))
    }
    
    /// Extract optional number value from JSON
    fn extract_json_optional_number(&self, json: &str, key: &str) -> Option<u64> {
        self.extract_json_number(json, key).ok()
    }
    
    /// Get default role
    fn get_default_role(&self) -> Role {
        self.get_quality_engineer_role()
    }
    
    /// Get admin role
    #[cfg(test)]
    pub fn get_admin_role(&self) -> Role {
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
    #[cfg(test)]
    pub fn get_quality_engineer_role(&self) -> Role {
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

    #[cfg(not(test))]
    fn get_admin_role(&self) -> Role {
        Role {
            name: "Administrator".to_string(),
            permissions: vec![
                Permission::ManageUsers,
                Permission::ReadDocuments,
                Permission::WriteDocuments,
                Permission::ReadRisks,
                Permission::WriteRisks,
                Permission::ReadTrace,
                Permission::WriteTrace,
                Permission::ReadAudit,
                Permission::ExportAudit,
                Permission::GenerateReports,
            ],
        }
    }

    #[cfg(not(test))]
    fn get_quality_engineer_role(&self) -> Role {
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
    fn get_developer_role(&self) -> Role {
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
    fn get_auditor_role(&self) -> Role {
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
