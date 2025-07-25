// File-based session storage implementation
// Implements SessionStorage interface with JSON file persistence
// Provides unified session storage for both CLI and web interfaces

use crate::prelude::*;
use crate::models::{Role, Permission};
use crate::modules::user_manager::interfaces::{SessionStorage, UserSession, SessionType};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

/// File-based session storage implementation
pub struct FileSessionStorage {
    sessions_path: PathBuf,
}

impl FileSessionStorage {
    /// Create new file session storage
    pub fn new(project_path: &Path) -> QmsResult<Self> {
        let sessions_path = project_path.join("sessions.json");
        
        // Create sessions file if it doesn't exist
        if !sessions_path.exists() {
            let empty_sessions = Vec::<UserSession>::new();
            let json_content = Self::serialize_sessions(&empty_sessions)?;
            fs::write(&sessions_path, json_content)?;
        }
        
        Ok(FileSessionStorage {
            sessions_path,
        })
    }
    
    /// Create from project path
    pub fn from_project_path(project_path: &Path) -> QmsResult<Self> {
        Self::new(project_path)
    }
    
    /// Serialize sessions to JSON
    fn serialize_sessions(sessions: &[UserSession]) -> QmsResult<String> {
        let mut json_sessions = Vec::new();
        
        for session in sessions {
            let permissions_json: Vec<String> = session.permissions.iter()
                .map(|p| format!("\"{}\"", p))
                .collect();
            
            let roles_json: Vec<String> = session.roles.iter()
                .map(|role| {
                    let role_permissions: Vec<String> = role.permissions.iter()
                        .map(|p| Self::permission_to_string(p))
                        .collect();
                    
                    format!(
                        "{{\"name\":\"{}\",\"permissions\":[{}]}}",
                        role.name,
                        role_permissions.join(",")
                    )
                })
                .collect();
            
            let data_json: Vec<String> = session.data.iter()
                .map(|(k, v)| format!("\"{}\":\"{}\"", k, v))
                .collect();
            
            let session_json = format!(
                "{{\"session_id\":\"{}\",\"user_id\":\"{}\",\"username\":\"{}\",\"roles\":[{}],\"permissions\":[{}],\"login_time\":{},\"last_activity\":{},\"expires_at\":{},\"ip_address\":{},\"user_agent\":{},\"csrf_token\":\"{}\",\"is_active\":{},\"session_type\":\"{}\",\"data\":{{{}}}}}",
                session.session_id,
                session.user_id,
                session.username,
                roles_json.join(","),
                permissions_json.join(","),
                session.login_time,
                session.last_activity,
                session.expires_at,
                session.ip_address.as_ref().map(|ip| format!("\"{}\"", ip)).unwrap_or("null".to_string()),
                session.user_agent.as_ref().map(|ua| format!("\"{}\"", ua)).unwrap_or("null".to_string()),
                session.csrf_token,
                session.is_active,
                match session.session_type {
                    SessionType::CLI => "CLI",
                    SessionType::TUI => "TUI",
                    SessionType::Web => "Web",
                },
                data_json.join(",")
            );
            
            json_sessions.push(session_json);
        }
        
        Ok(format!("[{}]", json_sessions.join(",")))
    }
    
    /// Parse sessions from JSON
    fn parse_sessions_json(&self, content: &str) -> QmsResult<Vec<UserSession>> {
        if content.trim().is_empty() || content.trim() == "[]" {
            return Ok(Vec::new());
        }
        
        // Simple JSON parsing for sessions array
        let content = content.trim();
        if !content.starts_with('[') || !content.ends_with(']') {
            return Err(QmsError::domain_error("Invalid sessions JSON format"));
        }
        
        let content = &content[1..content.len()-1]; // Remove [ ]
        if content.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        let mut sessions = Vec::new();
        let mut brace_count = 0;
        let mut current_session = String::new();
        let mut in_string = false;
        let mut escape_next = false;
        
        for ch in content.chars() {
            if escape_next {
                current_session.push(ch);
                escape_next = false;
                continue;
            }
            
            if ch == '\\' {
                escape_next = true;
                current_session.push(ch);
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
            
            current_session.push(ch);
            
            if !in_string && brace_count == 0 && ch == '}' {
                // Parse this session
                if let Ok(session) = self.parse_single_session(&current_session) {
                    sessions.push(session);
                }
                current_session.clear();
                
                // Skip comma and whitespace
                continue;
            }
        }
        
        Ok(sessions)
    }
    
    /// Parse a single session from JSON
    fn parse_single_session(&self, json: &str) -> QmsResult<UserSession> {
        // Extract fields using simple string parsing
        let session_id = self.extract_json_string(json, "session_id")?;
        let user_id = self.extract_json_string(json, "user_id")?;
        let username = self.extract_json_string(json, "username")?;
        let csrf_token = self.extract_json_string(json, "csrf_token")?;
        let login_time = self.extract_json_number(json, "login_time")?;
        let last_activity = self.extract_json_number(json, "last_activity")?;
        let expires_at = self.extract_json_number(json, "expires_at")?;
        let is_active = self.extract_json_bool(json, "is_active")?;
        
        let ip_address = self.extract_json_optional_string(json, "ip_address")?;
        let user_agent = self.extract_json_optional_string(json, "user_agent")?;
        
        let session_type_str = self.extract_json_string(json, "session_type")?;
        let session_type = match session_type_str.as_str() {
            "CLI" => SessionType::CLI,
            "Web" => SessionType::Web,
            _ => SessionType::Web, // Default to Web
        };
        
        // Parse roles (simplified)
        let roles = self.parse_roles_from_json(json)?;
        
        // Parse permissions
        let permissions = self.parse_permissions_from_json(json)?;
        
        // Parse data (simplified)
        let data = HashMap::new(); // TODO: Implement data parsing if needed
        
        Ok(UserSession {
            session_id,
            user_id,
            username,
            roles,
            permissions,
            login_time,
            last_activity,
            expires_at,
            ip_address,
            user_agent,
            csrf_token,
            is_active,
            session_type,
            data,
        })
    }
    
    /// Extract string value from JSON
    fn extract_json_string(&self, json: &str, key: &str) -> QmsResult<String> {
        let pattern = format!("\"{}\":\"", key);
        if let Some(start) = json.find(&pattern) {
            let start = start + pattern.len();
            if let Some(end) = json[start..].find('"') {
                return Ok(json[start..start + end].to_string());
            }
        }
        Err(QmsError::domain_error(&format!("Missing or invalid field: {}", key)))
    }
    
    /// Extract optional string value from JSON
    fn extract_json_optional_string(&self, json: &str, key: &str) -> QmsResult<Option<String>> {
        let pattern = format!("\"{}\":", key);
        if let Some(start) = json.find(&pattern) {
            let start = start + pattern.len();
            let remaining = &json[start..];
            
            if remaining.trim_start().starts_with("null") {
                return Ok(None);
            }
            
            if let Some(quote_start) = remaining.find('"') {
                let quote_start = start + quote_start + 1;
                if let Some(quote_end) = json[quote_start..].find('"') {
                    return Ok(Some(json[quote_start..quote_start + quote_end].to_string()));
                }
            }
        }
        Ok(None)
    }
    
    /// Extract number value from JSON
    fn extract_json_number(&self, json: &str, key: &str) -> QmsResult<u64> {
        let pattern = format!("\"{}\":", key);
        if let Some(start) = json.find(&pattern) {
            let start = start + pattern.len();
            let remaining = &json[start..];
            
            // Find the number
            let mut end = 0;
            for (i, ch) in remaining.chars().enumerate() {
                if ch.is_ascii_digit() {
                    end = i + 1;
                } else if end > 0 {
                    break;
                }
            }
            
            if end > 0 {
                if let Ok(num) = remaining[..end].parse::<u64>() {
                    return Ok(num);
                }
            }
        }
        Err(QmsError::domain_error(&format!("Missing or invalid number field: {}", key)))
    }
    
    /// Extract boolean value from JSON
    fn extract_json_bool(&self, json: &str, key: &str) -> QmsResult<bool> {
        let pattern = format!("\"{}\":", key);
        if let Some(start) = json.find(&pattern) {
            let start = start + pattern.len();
            let remaining = &json[start..];
            
            if remaining.trim_start().starts_with("true") {
                return Ok(true);
            } else if remaining.trim_start().starts_with("false") {
                return Ok(false);
            }
        }
        Err(QmsError::domain_error(&format!("Missing or invalid boolean field: {}", key)))
    }
    
    /// Parse roles from JSON (simplified implementation)
    fn parse_roles_from_json(&self, _json: &str) -> QmsResult<Vec<Role>> {
        // For now, return empty roles - can be enhanced later
        Ok(Vec::new())
    }
    
    /// Parse permissions from JSON
    fn parse_permissions_from_json(&self, json: &str) -> QmsResult<Vec<String>> {
        // Simple implementation - extract permissions array
        if let Some(start) = json.find("\"permissions\":[") {
            let start = start + 15; // Length of "\"permissions\":["
            if let Some(end) = json[start..].find(']') {
                let permissions_str = &json[start..start + end];
                let mut permissions = Vec::new();
                
                // Parse permission strings
                for part in permissions_str.split(',') {
                    let part = part.trim();
                    if part.starts_with('"') && part.ends_with('"') {
                        permissions.push(part[1..part.len()-1].to_string());
                    }
                }
                
                return Ok(permissions);
            }
        }
        Ok(Vec::new())
    }
    
    /// Convert permission to string
    fn permission_to_string(permission: &Permission) -> String {
        format!("\"{}\"", match permission {
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
        })
    }
    
    /// Load all sessions from file
    fn load_sessions(&self) -> QmsResult<Vec<UserSession>> {
        // Handle case where sessions file doesn't exist yet
        if !self.sessions_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.sessions_path)?;
        self.parse_sessions_json(&content)
    }
    
    /// Save all sessions to file
    fn save_sessions(&self, sessions: &[UserSession]) -> QmsResult<()> {
        let json_content = Self::serialize_sessions(sessions)?;
        fs::write(&self.sessions_path, json_content)?;
        Ok(())
    }
}

impl SessionStorage for FileSessionStorage {
    /// Save session to storage
    fn save_session(&self, session: &UserSession) -> QmsResult<()> {
        let mut sessions = self.load_sessions()?;

        // Update existing session or add new one
        let mut found = false;
        for existing_session in &mut sessions {
            if existing_session.session_id == session.session_id {
                *existing_session = session.clone();
                found = true;
                break;
            }
        }

        if !found {
            sessions.push(session.clone());
        }

        self.save_sessions(&sessions)?;
        Ok(())
    }

    /// Load session from storage
    fn load_session(&self, session_id: &str) -> QmsResult<UserSession> {
        let sessions = self.load_sessions()?;

        for session in sessions {
            if session.session_id == session_id {
                return Ok(session);
            }
        }

        Err(QmsError::NotFound(format!("Session '{}' not found", session_id)))
    }

    /// Check if session exists
    fn session_exists(&self, session_id: &str) -> QmsResult<bool> {
        match self.load_session(session_id) {
            Ok(_) => Ok(true),
            Err(QmsError::NotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Delete session from storage
    fn delete_session(&self, session_id: &str) -> QmsResult<()> {
        let mut sessions = self.load_sessions()?;
        let original_len = sessions.len();

        sessions.retain(|session| session.session_id != session_id);

        if sessions.len() == original_len {
            return Err(QmsError::NotFound(format!("Session '{}' not found", session_id)));
        }

        self.save_sessions(&sessions)?;
        Ok(())
    }

    /// List all sessions
    fn list_sessions(&self) -> QmsResult<Vec<UserSession>> {
        self.load_sessions()
    }

    /// List sessions for specific user
    fn list_user_sessions(&self, username: &str) -> QmsResult<Vec<UserSession>> {
        let sessions = self.load_sessions()?;
        let user_sessions: Vec<UserSession> = sessions
            .into_iter()
            .filter(|session| session.username == username)
            .collect();

        Ok(user_sessions)
    }

    /// Delete all sessions for user
    fn delete_user_sessions(&self, username: &str) -> QmsResult<usize> {
        let mut sessions = self.load_sessions()?;
        let original_len = sessions.len();

        sessions.retain(|session| session.username != username);
        let deleted_count = original_len - sessions.len();

        self.save_sessions(&sessions)?;
        Ok(deleted_count)
    }

    /// Cleanup expired sessions
    fn cleanup_expired_sessions(&self) -> QmsResult<usize> {
        let mut sessions = self.load_sessions()?;
        let original_len = sessions.len();

        sessions.retain(|session| !session.is_expired());
        let cleaned_count = original_len - sessions.len();

        self.save_sessions(&sessions)?;
        Ok(cleaned_count)
    }
}
