// Role-based authorization strategy
// Implements UserAuthorizer interface using role-based access control (RBAC)

use crate::prelude::*;
use crate::models::{User, Role, Permission};
use crate::modules::user_manager::interfaces::UserAuthorizer;

/// Role-based authorization strategy
pub struct RoleBasedAuthorizationStrategy {
    strict_mode: bool, // If true, requires explicit permission grants
}

impl RoleBasedAuthorizationStrategy {
    /// Create new role-based authorization strategy
    pub fn new() -> Self {
        Self {
            strict_mode: true,
        }
    }
    
    /// Create with permissive mode (allows implicit permissions)
    pub fn with_permissive_mode() -> Self {
        Self {
            strict_mode: false,
        }
    }
    
    /// Check if role contains permission
    fn role_has_permission(&self, role: &Role, permission: &Permission) -> bool {
        role.permissions.contains(permission)
    }
    
    /// Check if user has administrator role
    fn is_administrator(&self, user: &User) -> bool {
        user.roles.iter().any(|role| role.name == "Administrator")
    }
}

impl Default for RoleBasedAuthorizationStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl UserAuthorizer for RoleBasedAuthorizationStrategy {
    fn has_permission(&self, user: &User, permission: &Permission) -> bool {
        // Administrators have all permissions in non-strict mode
        if !self.strict_mode && self.is_administrator(user) {
            return true;
        }
        
        // Check if any of the user's roles has the permission
        user.roles.iter().any(|role| self.role_has_permission(role, permission))
    }
    
    fn has_any_permission(&self, user: &User, permissions: &[Permission]) -> bool {
        permissions.iter().any(|permission| self.has_permission(user, permission))
    }
    
    fn has_all_permissions(&self, user: &User, permissions: &[Permission]) -> bool {
        permissions.iter().all(|permission| self.has_permission(user, permission))
    }
    
    fn get_user_permissions(&self, user: &User) -> Vec<Permission> {
        let mut permissions = Vec::new();
        
        for role in &user.roles {
            for permission in &role.permissions {
                if !permissions.contains(permission) {
                    permissions.push(permission.clone());
                }
            }
        }
        
        permissions
    }
}

/// Permission hierarchy for medical device compliance
pub struct MedicalDevicePermissionHierarchy;

impl MedicalDevicePermissionHierarchy {
    /// Get permission hierarchy - higher level permissions include lower level ones
    pub fn get_permission_hierarchy() -> Vec<(Permission, Vec<Permission>)> {
        vec![
            // Delete permissions include write and read
            (Permission::DeleteDocuments, vec![Permission::WriteDocuments, Permission::ReadDocuments]),
            (Permission::DeleteRisks, vec![Permission::WriteRisks, Permission::ReadRisks]),
            (Permission::DeleteTrace, vec![Permission::WriteTrace, Permission::ReadTrace]),
            
            // Write permissions include read
            (Permission::WriteDocuments, vec![Permission::ReadDocuments]),
            (Permission::WriteRisks, vec![Permission::ReadRisks]),
            (Permission::WriteTrace, vec![Permission::ReadTrace]),
            
            // Export permissions include read
            (Permission::ExportAudit, vec![Permission::ReadAudit]),
            
            // Generate reports includes read permissions for all modules
            (Permission::GenerateReports, vec![
                Permission::ReadDocuments,
                Permission::ReadRisks,
                Permission::ReadTrace,
                Permission::ReadAudit,
            ]),
            
            // Manage users is a standalone high-privilege permission
            (Permission::ManageUsers, vec![]),
        ]
    }
    
    /// Check if permission implies other permissions
    pub fn permission_implies(&self, granted: &Permission, required: &Permission) -> bool {
        if granted == required {
            return true;
        }
        
        for (high_perm, implied_perms) in Self::get_permission_hierarchy() {
            if granted == &high_perm && implied_perms.contains(required) {
                return true;
            }
        }
        
        false
    }
}

/// Enhanced authorization strategy with permission hierarchy
pub struct HierarchicalAuthorizationStrategy {
    base_strategy: RoleBasedAuthorizationStrategy,
    hierarchy: MedicalDevicePermissionHierarchy,
}

impl HierarchicalAuthorizationStrategy {
    /// Create new hierarchical authorization strategy
    pub fn new() -> Self {
        Self {
            base_strategy: RoleBasedAuthorizationStrategy::new(),
            hierarchy: MedicalDevicePermissionHierarchy,
        }
    }
}

impl Default for HierarchicalAuthorizationStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl UserAuthorizer for HierarchicalAuthorizationStrategy {
    fn has_permission(&self, user: &User, permission: &Permission) -> bool {
        // First check direct permission
        if self.base_strategy.has_permission(user, permission) {
            return true;
        }
        
        // Check if user has any higher-level permission that implies this one
        let user_permissions = self.base_strategy.get_user_permissions(user);
        user_permissions.iter().any(|granted| {
            self.hierarchy.permission_implies(granted, permission)
        })
    }
    
    fn has_any_permission(&self, user: &User, permissions: &[Permission]) -> bool {
        permissions.iter().any(|permission| self.has_permission(user, permission))
    }
    
    fn has_all_permissions(&self, user: &User, permissions: &[Permission]) -> bool {
        permissions.iter().all(|permission| self.has_permission(user, permission))
    }
    
    fn get_user_permissions(&self, user: &User) -> Vec<Permission> {
        let mut all_permissions = Vec::new();
        let direct_permissions = self.base_strategy.get_user_permissions(user);
        
        // Add direct permissions
        for permission in &direct_permissions {
            if !all_permissions.contains(permission) {
                all_permissions.push(permission.clone());
            }
        }
        
        // Add implied permissions
        for granted in &direct_permissions {
            for (high_perm, implied_perms) in MedicalDevicePermissionHierarchy::get_permission_hierarchy() {
                if granted == &high_perm {
                    for implied in implied_perms {
                        if !all_permissions.contains(&implied) {
                            all_permissions.push(implied);
                        }
                    }
                }
            }
        }
        
        all_permissions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_user_with_roles(roles: Vec<Role>) -> User {
        User {
            username: "testuser".to_string(),
            password_hash: "hash".to_string(),
            roles,
            created_at: 0,
            last_login: None,
        }
    }
    
    fn create_admin_role() -> Role {
        Role {
            name: "Administrator".to_string(),
            permissions: vec![
                Permission::ManageUsers,
                Permission::DeleteDocuments,
                Permission::DeleteRisks,
                Permission::DeleteTrace,
                Permission::ExportAudit,
                Permission::GenerateReports,
            ],
        }
    }
    
    fn create_quality_engineer_role() -> Role {
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
    
    #[test]
    fn test_role_based_authorization() {
        let auth_strategy = RoleBasedAuthorizationStrategy::new();
        let user = create_test_user_with_roles(vec![create_quality_engineer_role()]);
        
        // Test permissions user should have
        assert!(auth_strategy.has_permission(&user, &Permission::ReadDocuments));
        assert!(auth_strategy.has_permission(&user, &Permission::WriteDocuments));
        assert!(auth_strategy.has_permission(&user, &Permission::GenerateReports));
        
        // Test permissions user should not have
        assert!(!auth_strategy.has_permission(&user, &Permission::ManageUsers));
        assert!(!auth_strategy.has_permission(&user, &Permission::DeleteDocuments));
    }
    
    #[test]
    fn test_hierarchical_authorization() {
        let auth_strategy = HierarchicalAuthorizationStrategy::new();
        let user = create_test_user_with_roles(vec![create_admin_role()]);
        
        // Test that delete permission implies write and read
        assert!(auth_strategy.has_permission(&user, &Permission::DeleteDocuments));
        assert!(auth_strategy.has_permission(&user, &Permission::WriteDocuments));
        assert!(auth_strategy.has_permission(&user, &Permission::ReadDocuments));
        
        // Test that generate reports implies read permissions
        assert!(auth_strategy.has_permission(&user, &Permission::GenerateReports));
        assert!(auth_strategy.has_permission(&user, &Permission::ReadAudit));
    }
    
    #[test]
    fn test_permission_hierarchy() {
        let hierarchy = MedicalDevicePermissionHierarchy;
        
        // Test delete implies write and read
        assert!(hierarchy.permission_implies(&Permission::DeleteDocuments, &Permission::WriteDocuments));
        assert!(hierarchy.permission_implies(&Permission::DeleteDocuments, &Permission::ReadDocuments));
        assert!(hierarchy.permission_implies(&Permission::WriteDocuments, &Permission::ReadDocuments));
        
        // Test that lower permissions don't imply higher ones
        assert!(!hierarchy.permission_implies(&Permission::ReadDocuments, &Permission::WriteDocuments));
        assert!(!hierarchy.permission_implies(&Permission::WriteDocuments, &Permission::DeleteDocuments));
    }
    
    #[test]
    fn test_get_all_user_permissions() {
        let auth_strategy = HierarchicalAuthorizationStrategy::new();
        let user = create_test_user_with_roles(vec![create_admin_role()]);
        
        let permissions = auth_strategy.get_user_permissions(&user);
        
        // Should include both direct and implied permissions
        assert!(permissions.contains(&Permission::DeleteDocuments));
        assert!(permissions.contains(&Permission::WriteDocuments));
        assert!(permissions.contains(&Permission::ReadDocuments));
        assert!(permissions.contains(&Permission::ManageUsers));
    }
}
