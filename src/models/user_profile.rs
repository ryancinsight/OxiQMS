//! User Profile Model for Global User Management
//! 
//! This module defines the UserProfile structure that stores user preferences,
//! QMS folder locations, and persistent settings independently of projects.
//! Follows medical device compliance requirements for user data management.

use crate::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// User profile containing preferences and QMS folder settings
/// Stored globally and persists across application sessions
#[derive(Debug, Clone)]
pub struct UserProfile {
    /// Username (matches User.username)
    pub username: String,
    
    /// User's preferred QMS folder location
    /// Default: ~/Documents/QMS or %USERPROFILE%/Documents/QMS
    pub qms_folder_path: PathBuf,
    
    /// Default location for new projects within QMS folder
    /// If None, uses qms_folder_path/projects
    pub default_project_location: Option<PathBuf>,
    
    /// UI preferences and settings
    pub ui_preferences: HashMap<String, String>,
    
    /// Last successful login timestamp
    pub last_login: Option<u64>,
    
    /// Persistent session token for auto-login
    pub session_token: Option<String>,
    
    /// Profile creation timestamp
    pub created_at: u64,
    
    /// Last profile update timestamp
    pub updated_at: u64,
    
    /// User's preferred language/locale
    pub locale: String,
    
    /// Theme preferences
    pub theme: String,
    
    /// Recently accessed projects
    pub recent_projects: Vec<String>,
    
    /// Maximum number of recent projects to track
    pub max_recent_projects: usize,
}

impl UserProfile {
    /// Create a new user profile with default settings
    pub fn new(username: &str) -> QmsResult<Self> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let qms_folder_path = Self::get_default_qms_folder()?;
        
        Ok(UserProfile {
            username: username.to_string(),
            qms_folder_path,
            default_project_location: None,
            ui_preferences: HashMap::new(),
            last_login: None,
            session_token: None,
            created_at: now,
            updated_at: now,
            locale: "en-US".to_string(),
            theme: "default".to_string(),
            recent_projects: Vec::new(),
            max_recent_projects: 10,
        })
    }
    
    /// Get the default QMS folder path for the current platform
    pub fn get_default_qms_folder() -> QmsResult<PathBuf> {
        let home_dir = if cfg!(windows) {
            std::env::var("USERPROFILE")
                .or_else(|_| std::env::var("HOME"))
                .map_err(|_| QmsError::domain_error("Cannot determine home directory"))?
        } else {
            std::env::var("HOME")
                .map_err(|_| QmsError::domain_error("Cannot determine home directory"))?
        };
        
        Ok(PathBuf::from(home_dir).join("Documents").join("QMS"))
    }
    
    /// Update the QMS folder path and validate it
    pub fn set_qms_folder_path(&mut self, path: PathBuf) -> QmsResult<()> {
        // Validate the path is accessible
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                return Err(QmsError::validation_error(
                    "Parent directory does not exist"
                ));
            }
        }
        
        // Create the QMS folder if it doesn't exist
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }
        
        self.qms_folder_path = path;
        self.updated_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        
        Ok(())
    }
    
    /// Get the effective project location (default_project_location or qms_folder_path/projects)
    pub fn get_projects_directory(&self) -> PathBuf {
        self.default_project_location
            .clone()
            .unwrap_or_else(|| self.qms_folder_path.join("projects"))
    }
    
    /// Add a project to the recent projects list
    pub fn add_recent_project(&mut self, project_id: &str) {
        // Remove if already exists
        self.recent_projects.retain(|p| p != project_id);
        
        // Add to front
        self.recent_projects.insert(0, project_id.to_string());
        
        // Trim to max size
        if self.recent_projects.len() > self.max_recent_projects {
            self.recent_projects.truncate(self.max_recent_projects);
        }
        
        self.updated_at = SystemTime::now().duration_since(UNIX_EPOCH)
            .unwrap_or_default().as_secs();
    }
    
    /// Set a UI preference
    pub fn set_ui_preference(&mut self, key: &str, value: &str) {
        self.ui_preferences.insert(key.to_string(), value.to_string());
        self.updated_at = SystemTime::now().duration_since(UNIX_EPOCH)
            .unwrap_or_default().as_secs();
    }
    
    /// Get a UI preference with optional default
    pub fn get_ui_preference(&self, key: &str, default: Option<&str>) -> Option<String> {
        self.ui_preferences.get(key)
            .cloned()
            .or_else(|| default.map(|d| d.to_string()))
    }
    
    /// Update last login timestamp
    pub fn update_last_login(&mut self) {
        self.last_login = Some(SystemTime::now().duration_since(UNIX_EPOCH)
            .unwrap_or_default().as_secs());
        self.updated_at = self.last_login.unwrap();
    }
    
    /// Set session token for persistent authentication
    pub fn set_session_token(&mut self, token: Option<String>) {
        self.session_token = token;
        self.updated_at = SystemTime::now().duration_since(UNIX_EPOCH)
            .unwrap_or_default().as_secs();
    }
    
    /// Validate the profile data
    pub fn validate(&self) -> QmsResult<()> {
        if self.username.is_empty() {
            return Err(QmsError::validation_error("Username cannot be empty"));
        }
        
        if !self.qms_folder_path.is_absolute() {
            return Err(QmsError::validation_error("QMS folder path must be absolute"));
        }
        
        Ok(())
    }
    
    /// Convert to JSON string for storage
    pub fn to_json(&self) -> QmsResult<String> {
        // Manual JSON construction to avoid external dependencies
        let ui_prefs_json = self.ui_preferences.iter()
            .map(|(k, v)| format!("\"{}\":\"{}\"", k, v))
            .collect::<Vec<_>>()
            .join(",");
        
        let recent_projects_json = self.recent_projects.iter()
            .map(|p| format!("\"{}\"", p))
            .collect::<Vec<_>>()
            .join(",");
        
        let json = format!(
            r#"{{
  "username": "{}",
  "qms_folder_path": "{}",
  "default_project_location": {},
  "ui_preferences": {{{}}},
  "last_login": {},
  "session_token": {},
  "created_at": {},
  "updated_at": {},
  "locale": "{}",
  "theme": "{}",
  "recent_projects": [{}],
  "max_recent_projects": {}
}}"#,
            self.username,
            self.qms_folder_path.to_string_lossy().replace("\\", "\\\\"),
            self.default_project_location.as_ref()
                .map(|p| format!("\"{}\"", p.to_string_lossy().replace("\\", "\\\\")))
                .unwrap_or_else(|| "null".to_string()),
            ui_prefs_json,
            self.last_login.map(|t| t.to_string()).unwrap_or_else(|| "null".to_string()),
            self.session_token.as_ref()
                .map(|t| format!("\"{}\"", t))
                .unwrap_or_else(|| "null".to_string()),
            self.created_at,
            self.updated_at,
            self.locale,
            self.theme,
            recent_projects_json,
            self.max_recent_projects
        );
        
        Ok(json)
    }
    
    /// Parse from JSON string
    pub fn from_json(json: &str) -> QmsResult<Self> {
        // Simple JSON parsing for the profile structure
        // In a production system, you might want more robust JSON parsing
        
        let username = Self::extract_json_string(json, "username")?;
        let qms_folder_path = PathBuf::from(Self::extract_json_string(json, "qms_folder_path")?);
        let created_at = Self::extract_json_number(json, "created_at")?;
        let updated_at = Self::extract_json_number(json, "updated_at")?;
        let locale = Self::extract_json_string(json, "locale").unwrap_or_else(|_| "en-US".to_string());
        let theme = Self::extract_json_string(json, "theme").unwrap_or_else(|_| "default".to_string());
        let max_recent_projects = Self::extract_json_number(json, "max_recent_projects")
            .unwrap_or(10) as usize;
        
        let last_login = Self::extract_json_number(json, "last_login").ok();
        let session_token = Self::extract_json_string(json, "session_token").ok();

        // Parse recent projects array
        let recent_projects = Self::extract_json_array(json, "recent_projects").unwrap_or_else(|_| Vec::new());

        // For simplicity, we'll initialize empty collections for UI preferences
        let ui_preferences = HashMap::new();
        
        Ok(UserProfile {
            username,
            qms_folder_path,
            default_project_location: None, // TODO: Parse from JSON
            ui_preferences,
            last_login,
            session_token,
            created_at,
            updated_at,
            locale,
            theme,
            recent_projects,
            max_recent_projects,
        })
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

            // Find the end of the number (comma, brace, or end of string)
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

    /// Extract array value from JSON (simple implementation)
    fn extract_json_array(json: &str, key: &str) -> QmsResult<Vec<String>> {
        let pattern = format!("\"{}\":", key);
        if let Some(start) = json.find(&pattern) {
            let value_start = start + pattern.len();
            if let Some(array_start) = json[value_start..].find('[') {
                let array_start = value_start + array_start + 1;
                if let Some(array_end) = json[array_start..].find(']') {
                    let array_content = &json[array_start..array_start + array_end];

                    let mut items = Vec::new();
                    let mut current_item = String::new();
                    let mut in_string = false;
                    let mut escape_next = false;

                    for ch in array_content.chars() {
                        if escape_next {
                            current_item.push(ch);
                            escape_next = false;
                            continue;
                        }

                        if ch == '\\' {
                            escape_next = true;
                            continue;
                        }

                        if ch == '"' {
                            if in_string {
                                // End of string item
                                items.push(current_item.clone());
                                current_item.clear();
                                in_string = false;
                            } else {
                                // Start of string item
                                in_string = true;
                            }
                        } else if in_string {
                            current_item.push(ch);
                        }
                    }

                    return Ok(items);
                }
            }
        }
        Err(QmsError::parse_error(&format!("Could not extract array '{}' from JSON", key)))
    }
}

impl Default for UserProfile {
    fn default() -> Self {
        UserProfile {
            username: String::new(),
            qms_folder_path: PathBuf::new(),
            default_project_location: None,
            ui_preferences: HashMap::new(),
            last_login: None,
            session_token: None,
            created_at: 0,
            updated_at: 0,
            locale: "en-US".to_string(),
            theme: "default".to_string(),
            recent_projects: Vec::new(),
            max_recent_projects: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_user_profile_creation() {
        let profile = UserProfile::new("testuser").unwrap();
        assert_eq!(profile.username, "testuser");
        assert!(!profile.qms_folder_path.as_os_str().is_empty());
        assert_eq!(profile.locale, "en-US");
        assert_eq!(profile.theme, "default");
    }
    
    #[test]
    fn test_qms_folder_path_setting() {
        let mut profile = UserProfile::new("testuser").unwrap();
        let temp_dir = env::temp_dir().join("qms_test");
        
        profile.set_qms_folder_path(temp_dir.clone()).unwrap();
        assert_eq!(profile.qms_folder_path, temp_dir);
        assert!(temp_dir.exists());
        
        // Cleanup
        let _ = std::fs::remove_dir_all(temp_dir);
    }
    
    #[test]
    fn test_recent_projects_management() {
        let mut profile = UserProfile::new("testuser").unwrap();
        
        profile.add_recent_project("project1");
        profile.add_recent_project("project2");
        profile.add_recent_project("project1"); // Should move to front
        
        assert_eq!(profile.recent_projects[0], "project1");
        assert_eq!(profile.recent_projects[1], "project2");
        assert_eq!(profile.recent_projects.len(), 2);
    }
    
    #[test]
    fn test_ui_preferences() {
        let mut profile = UserProfile::new("testuser").unwrap();
        
        profile.set_ui_preference("theme", "dark");
        profile.set_ui_preference("language", "en");
        
        assert_eq!(profile.get_ui_preference("theme", None), Some("dark".to_string()));
        assert_eq!(profile.get_ui_preference("language", None), Some("en".to_string()));
        assert_eq!(profile.get_ui_preference("missing", Some("default")), Some("default".to_string()));
    }
}
