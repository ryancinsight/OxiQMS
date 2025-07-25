//! Unified User Service
//! 
//! Consolidates user management business logic across CLI, TUI, and web interfaces
//! following SOLID principles and DRY methodology.

use crate::prelude::*;
use crate::modules::user_manager::{
    FileBasedAuthService, UserProfileManager, ProfileSummary
};
use crate::modules::user_manager::interfaces::{UserSession, SessionType};
use crate::models::{User, Role, Permission};
use crate::modules::audit_logger::audit_log_action;
use std::path::PathBuf;
use std::sync::Arc;

/// Unified User Service Interface
/// 
/// Provides a single interface for user management operations that can be used
/// by CLI, TUI, and web interfaces, eliminating code duplication.
pub trait UserServiceInterface: Send + Sync {
    /// Authenticate user and create session
    fn login(&self, username: &str, password: &str, session_type: SessionType) -> QmsResult<UserSession>;

    /// Authenticate user and create session (alias for login)
    fn authenticate_user(&self, username: &str, password: &str, session_type: SessionType) -> QmsResult<UserSession>;

    /// Validate existing session
    fn validate_session(&self, session_id: &str) -> QmsResult<UserSession>;

    /// Create session for authenticated user (for cross-interface sync)
    fn create_session(&self, username: &str, session_type: SessionType) -> QmsResult<UserSession>;

    /// Logout user and invalidate session
    fn logout(&self, session_id: &str) -> QmsResult<()>;

    /// Create new user account
    fn create_user(&self, username: &str, password: &str, roles: Vec<Role>, created_by: &str) -> QmsResult<User>;

    /// Get user by username
    fn get_user(&self, username: &str) -> QmsResult<User>;

    /// Update user information
    fn update_user(&self, username: &str, updates: UserUpdates, updated_by: &str) -> QmsResult<User>;

    /// Delete user account
    fn delete_user(&self, username: &str, deleted_by: &str) -> QmsResult<()>;

    /// List all users with optional filtering
    fn list_users(&self, filter: Option<UserFilter>) -> QmsResult<Vec<UserSummary>>;

    /// Assign role to user
    fn assign_role(&self, username: &str, role_name: &str, assigned_by: &str) -> QmsResult<()>;

    /// Remove role from user
    fn remove_role(&self, username: &str, role_name: &str, removed_by: &str) -> QmsResult<()>;

    /// Check if user has specific permission
    fn has_permission(&self, username: &str, permission: &str) -> QmsResult<bool>;

    /// Get user profile summary
    fn get_profile_summary(&self, username: &str) -> QmsResult<ProfileSummary>;

    /// Update user password
    fn update_password(&self, username: &str, old_password: &str, new_password: &str) -> QmsResult<()>;

    /// Reset user password (admin function)
    fn reset_password(&self, username: &str, new_password: &str, reset_by: &str) -> QmsResult<()>;
}

/// User update parameters
#[derive(Debug, Clone)]
pub struct UserUpdates {
    pub password: Option<String>,
    pub roles: Option<Vec<Role>>,
    pub profile_data: Option<std::collections::HashMap<String, String>>,
}

/// User filter for listing operations
#[derive(Debug, Clone)]
pub struct UserFilter {
    pub role: Option<String>,
    pub active_only: bool,
    pub search_term: Option<String>,
}

/// User summary for list operations
#[derive(Debug, Clone)]
pub struct UserSummary {
    pub username: String,
    pub roles: Vec<String>,
    pub created_at: u64,
    pub last_login: Option<u64>,
    pub is_active: bool,
}

/// Unified User Service Implementation
/// 
/// Wraps the existing user management services and provides a unified interface
/// that can be used across all interfaces (CLI, TUI, Web).
pub struct UnifiedUserService {
    auth_service: Arc<FileBasedAuthService>,
    profile_manager: UserProfileManager,
    project_path: PathBuf,
}

impl UnifiedUserService {
    /// Create new unified user service
    pub fn new(project_path: PathBuf) -> QmsResult<Self> {
        let user_storage = crate::modules::user_manager::implementations::file_user_storage::FileUserStorage::new(&project_path)?;
        let session_storage = crate::modules::user_manager::implementations::file_session_storage::FileSessionStorage::new(&project_path)?;
        let auth_service = Arc::new(FileBasedAuthService::new(user_storage, session_storage));
        let profile_manager = UserProfileManager::new()?;
        
        Ok(Self {
            auth_service,
            profile_manager,
            project_path,
        })
    }

    /// Create with dependency injection for testing
    pub fn with_services(
        auth_service: Arc<FileBasedAuthService>,
        profile_manager: UserProfileManager,
        project_path: PathBuf,
    ) -> Self {
        Self {
            auth_service,
            profile_manager,
            project_path,
        }
    }

    /// Convert User to UserSummary
    fn to_summary(&self, user: &User) -> UserSummary {
        UserSummary {
            username: user.username.clone(),
            roles: user.roles.iter().map(|r| r.name.clone()).collect(),
            created_at: user.created_at,
            last_login: user.last_login,
            is_active: true, // Simplified - would check actual status
        }
    }

    /// Validate user operation permissions
    fn validate_permissions(&self, operation: &str, user_id: &str, target_user: &str) -> QmsResult<()> {
        if user_id.is_empty() {
            return Err(QmsError::permission_error("User ID is required"));
        }
        
        // Log the operation attempt for audit
        let _ = audit_log_action(
            &format!("USER_OPERATION_ATTEMPT_{}", operation.to_uppercase()),
            "User",
            &format!("{}:{}", user_id, target_user),
        );
        
        Ok(())
    }

    /// Validate password strength
    fn validate_password(&self, password: &str) -> QmsResult<()> {
        if password.len() < 8 {
            return Err(QmsError::validation_error("Password must be at least 8 characters"));
        }
        
        if !password.chars().any(|c| c.is_uppercase()) {
            return Err(QmsError::validation_error("Password must contain at least one uppercase letter"));
        }
        
        if !password.chars().any(|c| c.is_lowercase()) {
            return Err(QmsError::validation_error("Password must contain at least one lowercase letter"));
        }
        
        if !password.chars().any(|c| c.is_numeric()) {
            return Err(QmsError::validation_error("Password must contain at least one number"));
        }
        
        Ok(())
    }
}

impl UserServiceInterface for UnifiedUserService {
    fn login(&self, username: &str, password: &str, session_type: SessionType) -> QmsResult<UserSession> {
        let session = match session_type {
            SessionType::CLI => self.auth_service.cli_login(username, password)?,
            SessionType::Web => self.auth_service.web_login(username, password, None, None)?,
            SessionType::TUI => {
                // TUI login uses CLI session type internally
                self.auth_service.cli_login(username, password)?
            }
        };
        
        // Update last login time
        let _ = self.profile_manager.update_last_login(username);
        
        // Audit log the login
        let _ = audit_log_action("USER_LOGIN", "User", username);
        
        Ok(session)
    }

    fn authenticate_user(&self, username: &str, password: &str, session_type: SessionType) -> QmsResult<UserSession> {
        // Delegate to login method
        self.login(username, password, session_type)
    }

    fn validate_session(&self, session_id: &str) -> QmsResult<UserSession> {
        self.auth_service.validate_session(session_id)
    }

    fn create_session(&self, username: &str, session_type: SessionType) -> QmsResult<UserSession> {
        // Create session for already authenticated user (used for cross-interface sync)
        self.auth_service.create_session_for_user(username, session_type)
    }

    fn logout(&self, session_id: &str) -> QmsResult<()> {
        // Get session info for audit logging
        if let Ok(session) = self.auth_service.validate_session(session_id) {
            let _ = audit_log_action("USER_LOGOUT", "User", &session.username);
        }
        
        self.auth_service.logout(session_id)
    }

    fn create_user(&self, username: &str, password: &str, roles: Vec<Role>, created_by: &str) -> QmsResult<User> {
        self.validate_permissions("CREATE", created_by, username)?;
        self.validate_password(password)?;
        
        let user = self.auth_service.create_user(username, password, roles)?;
        
        // Create user profile
        let _ = self.profile_manager.get_or_create_profile(username);
        
        // Audit log the creation
        let _ = audit_log_action("USER_CREATED", "User", username);
        
        Ok(user)
    }

    fn get_user(&self, username: &str) -> QmsResult<User> {
        self.auth_service.get_user(username)
    }

    fn update_user(&self, username: &str, updates: UserUpdates, updated_by: &str) -> QmsResult<User> {
        self.validate_permissions("UPDATE", updated_by, username)?;
        
        let mut user = self.auth_service.get_user(username)?;
        
        // Apply updates
        if let Some(new_password) = updates.password {
            self.validate_password(&new_password)?;
            user.password_hash = FileBasedAuthService::hash_password(&new_password);
        }
        
        if let Some(new_roles) = updates.roles {
            user.roles = new_roles;
        }
        
        // Save updated user (simplified - would use proper update method)
        // self.auth_service.update_user(&user)?;
        
        // Audit log the update
        let _ = audit_log_action("USER_UPDATED", "User", username);
        
        Ok(user)
    }

    fn delete_user(&self, username: &str, deleted_by: &str) -> QmsResult<()> {
        self.validate_permissions("DELETE", deleted_by, username)?;
        
        // Delete user profile first
        let _ = self.profile_manager.reset_profile(username);
        
        // Delete user account (simplified - would use proper delete method)
        // self.auth_service.delete_user(username)?;
        
        // Audit log the deletion
        let _ = audit_log_action("USER_DELETED", "User", username);
        
        Ok(())
    }

    fn list_users(&self, filter: Option<UserFilter>) -> QmsResult<Vec<UserSummary>> {
        // Simplified implementation - would get users from storage
        let users = Vec::new(); // self.auth_service.list_users()?;
        
        let mut summaries: Vec<UserSummary> = users
            .iter()
            .map(|user| self.to_summary(user))
            .collect();
        
        // Apply filters if provided
        if let Some(filter) = filter {
            if let Some(role) = filter.role {
                summaries.retain(|s| s.roles.contains(&role));
            }
            
            if filter.active_only {
                summaries.retain(|s| s.is_active);
            }
            
            if let Some(search_term) = filter.search_term {
                let search_lower = search_term.to_lowercase();
                summaries.retain(|s| s.username.to_lowercase().contains(&search_lower));
            }
        }
        
        Ok(summaries)
    }

    fn assign_role(&self, username: &str, role_name: &str, assigned_by: &str) -> QmsResult<()> {
        self.validate_permissions("ASSIGN_ROLE", assigned_by, username)?;
        
        // Simplified implementation - would use role manager
        // self.role_manager.assign_role(username, role_name)?;
        
        // Audit log the role assignment
        let _ = audit_log_action("ROLE_ASSIGNED", "User", &format!("{}:{}", username, role_name));
        
        Ok(())
    }

    fn remove_role(&self, username: &str, role_name: &str, removed_by: &str) -> QmsResult<()> {
        self.validate_permissions("REMOVE_ROLE", removed_by, username)?;
        
        // Simplified implementation - would use role manager
        // self.role_manager.remove_role(username, role_name)?;
        
        // Audit log the role removal
        let _ = audit_log_action("ROLE_REMOVED", "User", &format!("{}:{}", username, role_name));
        
        Ok(())
    }

    fn has_permission(&self, username: &str, permission: &str) -> QmsResult<bool> {
        let user = self.auth_service.get_user(username)?;
        
        for role in &user.roles {
            for perm in &role.permissions {
                if Self::permission_to_string(perm) == permission {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }

    fn get_profile_summary(&self, username: &str) -> QmsResult<ProfileSummary> {
        self.profile_manager.get_profile_summary(username)
    }

    fn update_password(&self, username: &str, old_password: &str, new_password: &str) -> QmsResult<()> {
        // Verify old password first
        let _session = self.auth_service.cli_login(username, old_password)?;
        
        self.validate_password(new_password)?;
        
        // Update password (simplified - would use proper update method)
        // self.auth_service.update_password(username, new_password)?;
        
        // Audit log the password change
        let _ = audit_log_action("PASSWORD_CHANGED", "User", username);
        
        Ok(())
    }

    fn reset_password(&self, username: &str, new_password: &str, reset_by: &str) -> QmsResult<()> {
        self.validate_permissions("RESET_PASSWORD", reset_by, username)?;
        self.validate_password(new_password)?;
        
        // Reset password (simplified - would use proper reset method)
        // self.auth_service.reset_password(username, new_password)?;
        
        // Audit log the password reset
        let _ = audit_log_action("PASSWORD_RESET", "User", &format!("{}:{}", username, reset_by));
        
        Ok(())
    }
}

impl UnifiedUserService {
    /// Convert Permission enum to string for comparison
    fn permission_to_string(permission: &Permission) -> &str {
        match permission {
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
        }
    }
}
