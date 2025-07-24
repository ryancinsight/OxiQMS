//! TUI Authentication System
//! 
//! This module provides authentication functionality for the TUI interface,
//! integrating with the unified QMS authentication system while maintaining
//! medical device compliance and audit trail requirements.

use crate::prelude::*;
use crate::modules::user_manager::auth::{AuthManager, UserSession};
use crate::modules::user_manager::implementations::FileUserStorage;
use crate::modules::audit_logger::audit_log_action;
use crate::models::{Permission, Role};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// TUI Authentication Manager
/// Integrates with the unified QMS authentication system
pub struct TuiAuthManager {
    auth_manager: Arc<Mutex<AuthManager<FileUserStorage>>>,
    current_session: Option<UserSession>,
    project_path: PathBuf,
}

/// Authentication result for TUI operations
#[derive(Debug, Clone)]
pub struct TuiAuthResult {
    pub success: bool,
    pub message: String,
    pub session: Option<UserSession>,
}

impl TuiAuthManager {
    /// Create new TUI authentication manager
    pub fn new(project_path: Option<PathBuf>) -> QmsResult<Self> {
        let project_path = project_path.unwrap_or_else(|| PathBuf::from("."));

        // Initialize file-based user storage
        let storage = FileUserStorage::new(&project_path)?;
        let auth_manager = AuthManager::new(storage);

        Ok(Self {
            auth_manager: Arc::new(Mutex::new(auth_manager)),
            current_session: None,
            project_path,
        })
    }

    /// Authenticate user with username and password
    pub fn login(&mut self, username: &str, password: &str) -> QmsResult<TuiAuthResult> {
        // Validate input
        if username.trim().is_empty() {
            return Ok(TuiAuthResult {
                success: false,
                message: "Username cannot be empty".to_string(),
                session: None,
            });
        }

        if password.trim().is_empty() {
            return Ok(TuiAuthResult {
                success: false,
                message: "Password cannot be empty".to_string(),
                session: None,
            });
        }

        // Attempt authentication through unified auth system
        let login_result = {
            let mut auth_manager = self.auth_manager.lock().unwrap();
            auth_manager.login(username, password)
        };

        match login_result {
            Ok(session) => {
                // Log successful TUI login
                if let Err(e) = audit_log_action("TUI_LOGIN_SUCCESS", "Session", &session.username) {
                    eprintln!("Warning: Failed to log TUI login: {}", e);
                }

                // Store session
                self.current_session = Some(session.clone());

                Ok(TuiAuthResult {
                    success: true,
                    message: format!("Welcome, {}! Login successful.", session.username),
                    session: Some(session),
                })
            }
            Err(e) => {
                // Log failed login attempt
                if let Err(audit_err) = audit_log_action("TUI_LOGIN_FAILED", "Session", username) {
                    eprintln!("Warning: Failed to log TUI login failure: {}", audit_err);
                }

                Ok(TuiAuthResult {
                    success: false,
                    message: format!("Login failed: {}", e),
                    session: None,
                })
            }
        }
    }

    /// Logout current user
    pub fn logout(&mut self) -> QmsResult<TuiAuthResult> {
        if let Some(ref session) = self.current_session {
            // Log logout
            if let Err(e) = audit_log_action("TUI_LOGOUT", "Session", &session.username) {
                eprintln!("Warning: Failed to log TUI logout: {}", e);
            }

            let username = session.username.clone();
            self.current_session = None;

            Ok(TuiAuthResult {
                success: true,
                message: format!("Goodbye, {}! You have been logged out.", username),
                session: None,
            })
        } else {
            Ok(TuiAuthResult {
                success: false,
                message: "No active session to logout".to_string(),
                session: None,
            })
        }
    }

    /// Check if user is currently authenticated
    pub fn is_authenticated(&self) -> bool {
        self.current_session.is_some()
    }

    /// Get current user session
    pub fn current_session(&self) -> Option<&UserSession> {
        self.current_session.as_ref()
    }

    /// Get current username
    pub fn current_username(&self) -> Option<&str> {
        self.current_session.as_ref().map(|s| s.username.as_str())
    }

    /// Get current user roles
    pub fn current_user_roles(&self) -> Vec<String> {
        self.current_session
            .as_ref()
            .map(|s| s.roles.iter().map(|r| r.name.clone()).collect())
            .unwrap_or_default()
    }

    /// Validate current session (check if still valid)
    pub fn validate_current_session(&self) -> QmsResult<bool> {
        if let Some(ref session) = self.current_session {
            let auth_manager = self.auth_manager.lock().unwrap();
            match auth_manager.validate_session(&session.session_id) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        } else {
            Ok(false)
        }
    }

    /// Update session activity (extend session)
    pub fn update_session_activity(&mut self) -> QmsResult<()> {
        if let Some(ref mut session) = self.current_session {
            use std::time::{SystemTime, UNIX_EPOCH};
            session.last_activity = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        }
        Ok(())
    }

    /// Check if user has specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        if let Some(ref session) = self.current_session {
            session.roles.iter().any(|role| {
                role.permissions.iter().any(|perm| {
                    match perm {
                        Permission::ReadDocuments => permission == "read_documents",
                        Permission::WriteDocuments => permission == "write_documents",
                        Permission::DeleteDocuments => permission == "delete_documents",
                        Permission::ReadRisks => permission == "read_risks",
                        Permission::WriteRisks => permission == "write_risks",
                        Permission::DeleteRisks => permission == "delete_risks",
                        Permission::ReadTrace => permission == "read_trace",
                        Permission::WriteTrace => permission == "write_trace",
                        Permission::DeleteTrace => permission == "delete_trace",
                        Permission::ReadAudit => permission == "read_audit",
                        Permission::ExportAudit => permission == "export_audit",
                        Permission::ManageUsers => permission == "manage_users",
                        Permission::GenerateReports => permission == "generate_reports",
                        Permission::UserManagement => permission == "user_management",
                        Permission::ProjectManagement => permission == "project_management",
                        Permission::DocumentManagement => permission == "document_management",
                        Permission::RiskManagement => permission == "risk_management",
                        Permission::AuditAccess => permission == "audit_access",
                        Permission::SystemConfiguration => permission == "system_configuration",
                    }
                })
            })
        } else {
            false
        }
    }

    /// Get session information for display
    pub fn get_session_info(&self) -> Option<TuiSessionInfo> {
        self.current_session.as_ref().map(|session| {
            TuiSessionInfo {
                username: session.username.clone(),
                session_id: session.session_id.clone(),
                login_time: session.login_time,
                last_activity: session.last_activity,
                roles: session.roles.iter().map(|r| r.name.clone()).collect(),
                permissions: session.roles.iter()
                    .flat_map(|role| &role.permissions)
                    .map(|perm| format!("{:?}", perm))
                    .collect(),
            }
        })
    }

    /// Create a new user (admin function)
    pub fn create_user(&mut self, username: &str, password: &str, roles: Vec<String>) -> QmsResult<TuiAuthResult> {
        // Check if current user has admin permissions
        if !self.has_permission("manage_users") && !self.has_permission("user_management") {
            return Ok(TuiAuthResult {
                success: false,
                message: "Insufficient permissions to create users".to_string(),
                session: None,
            });
        }

        // For now, create a simple role - in full implementation, this would use RoleManager
        let role_objects = vec![Role {
            name: roles.first().unwrap_or(&"QualityEngineer".to_string()).clone(),
            permissions: vec![Permission::ReadDocuments, Permission::WriteDocuments],
        }];

        // Create user through auth manager
        let create_result = {
            let auth_manager = self.auth_manager.lock().unwrap();
            auth_manager.add_user(username, password, Some(role_objects))
        };

        match create_result {
            Ok(_user) => {
                Ok(TuiAuthResult {
                    success: true,
                    message: format!("User '{}' created successfully", username),
                    session: None,
                })
            }
            Err(e) => {
                Ok(TuiAuthResult {
                    success: false,
                    message: format!("Failed to create user: {}", e),
                    session: None,
                })
            }
        }
    }
}

/// Session information for TUI display
#[derive(Debug, Clone)]
pub struct TuiSessionInfo {
    pub username: String,
    pub session_id: String,
    pub login_time: u64,
    pub last_activity: u64,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

impl TuiSessionInfo {
    /// Format login time for display
    pub fn format_login_time(&self) -> String {
        self.format_timestamp(self.login_time)
    }

    /// Format last activity time for display
    pub fn format_last_activity(&self) -> String {
        self.format_timestamp(self.last_activity)
    }

    /// Format timestamp for display
    fn format_timestamp(&self, timestamp: u64) -> String {
        use std::time::{UNIX_EPOCH, Duration};
        
        let duration = Duration::from_secs(timestamp);
        let datetime = UNIX_EPOCH + duration;
        
        // Simple formatting - in a full implementation, you'd use a proper date library
        format!("{:?}", datetime)
    }

    /// Get session duration in seconds
    pub fn session_duration(&self) -> u64 {
        self.last_activity - self.login_time
    }

    /// Check if session is recent (within last hour)
    pub fn is_recent_activity(&self) -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        if let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) {
            let now_secs = now.as_secs();
            now_secs - self.last_activity < 3600 // 1 hour
        } else {
            false
        }
    }
}
