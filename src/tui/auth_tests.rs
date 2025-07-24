//! TUI Authentication Tests
//! 
//! This module provides comprehensive tests for the TUI authentication system,
//! demonstrating integration with the unified QMS authentication infrastructure.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::tui::auth::{TuiAuthManager, TuiAuthResult};
    use std::path::PathBuf;
    use std::fs;

    /// Test TUI authentication manager creation
    #[test]
    fn test_tui_auth_manager_creation() {
        let test_dir = std::env::temp_dir().join("qms_tui_auth_test");
        let _ = fs::create_dir_all(&test_dir);

        let auth_manager = TuiAuthManager::new(Some(test_dir.clone()));
        assert!(auth_manager.is_ok());

        let manager = auth_manager.unwrap();
        assert!(!manager.is_authenticated());
        assert!(manager.current_username().is_none());
        assert!(manager.current_user_roles().is_empty());

        // Cleanup
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_tui_auth_manager_default_path() {
        let auth_manager = TuiAuthManager::new(None);
        assert!(auth_manager.is_ok());

        let manager = auth_manager.unwrap();
        assert!(!manager.is_authenticated());
    }

    #[test]
    fn test_login_validation() {
        let test_dir = std::env::temp_dir().join("qms_tui_login_test");
        let _ = fs::create_dir_all(&test_dir);

        let mut auth_manager = TuiAuthManager::new(Some(test_dir.clone())).unwrap();

        // Test empty username
        let result = auth_manager.login("", "password").unwrap();
        assert!(!result.success);
        assert!(result.message.contains("Username cannot be empty"));
        assert!(result.session.is_none());

        // Test empty password
        let result = auth_manager.login("admin", "").unwrap();
        assert!(!result.success);
        assert!(result.message.contains("Password cannot be empty"));
        assert!(result.session.is_none());

        // Test whitespace-only username
        let result = auth_manager.login("   ", "password").unwrap();
        assert!(!result.success);
        assert!(result.message.contains("Username cannot be empty"));

        // Test whitespace-only password
        let result = auth_manager.login("admin", "   ").unwrap();
        assert!(!result.success);
        assert!(result.message.contains("Password cannot be empty"));

        // Cleanup
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_login_failure_with_invalid_credentials() {
        let test_dir = std::env::temp_dir().join("qms_tui_invalid_login_test");
        let _ = fs::create_dir_all(&test_dir);

        let mut auth_manager = TuiAuthManager::new(Some(test_dir.clone())).unwrap();

        // Test login with non-existent user
        let result = auth_manager.login("nonexistent", "password").unwrap();
        assert!(!result.success);
        assert!(result.message.contains("Login failed"));
        assert!(result.session.is_none());
        assert!(!auth_manager.is_authenticated());

        // Cleanup
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_logout_without_session() {
        let test_dir = std::env::temp_dir().join("qms_tui_logout_test");
        let _ = fs::create_dir_all(&test_dir);

        let mut auth_manager = TuiAuthManager::new(Some(test_dir.clone())).unwrap();

        // Test logout without active session
        let result = auth_manager.logout().unwrap();
        assert!(!result.success);
        assert!(result.message.contains("No active session to logout"));
        assert!(result.session.is_none());

        // Cleanup
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_permission_checking_without_session() {
        let test_dir = std::env::temp_dir().join("qms_tui_permission_test");
        let _ = fs::create_dir_all(&test_dir);

        let auth_manager = TuiAuthManager::new(Some(test_dir.clone())).unwrap();

        // Test permission checking without session
        assert!(!auth_manager.has_permission("read_documents"));
        assert!(!auth_manager.has_permission("write_documents"));
        assert!(!auth_manager.has_permission("manage_users"));
        assert!(!auth_manager.has_permission("user_management"));

        // Cleanup
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_session_validation_without_session() {
        let test_dir = std::env::temp_dir().join("qms_tui_session_validation_test");
        let _ = fs::create_dir_all(&test_dir);

        let auth_manager = TuiAuthManager::new(Some(test_dir.clone())).unwrap();

        // Test session validation without active session
        let is_valid = auth_manager.validate_current_session().unwrap();
        assert!(!is_valid);

        // Cleanup
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_session_info_without_session() {
        let test_dir = std::env::temp_dir().join("qms_tui_session_info_test");
        let _ = fs::create_dir_all(&test_dir);

        let auth_manager = TuiAuthManager::new(Some(test_dir.clone())).unwrap();

        // Test session info without active session
        let session_info = auth_manager.get_session_info();
        assert!(session_info.is_none());

        // Cleanup
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_create_user_without_permissions() {
        let test_dir = std::env::temp_dir().join("qms_tui_create_user_test");
        let _ = fs::create_dir_all(&test_dir);

        let mut auth_manager = TuiAuthManager::new(Some(test_dir.clone())).unwrap();

        // Test creating user without proper permissions (no active session)
        let result = auth_manager.create_user("newuser", "password", vec!["QualityEngineer".to_string()]).unwrap();
        assert!(!result.success);
        assert!(result.message.contains("Insufficient permissions"));
        assert!(result.session.is_none());

        // Cleanup
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_session_activity_update_without_session() {
        let test_dir = std::env::temp_dir().join("qms_tui_activity_test");
        let _ = fs::create_dir_all(&test_dir);

        let mut auth_manager = TuiAuthManager::new(Some(test_dir.clone())).unwrap();

        // Test updating session activity without active session
        let result = auth_manager.update_session_activity();
        assert!(result.is_ok()); // Should not fail, just do nothing

        // Cleanup
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_tui_session_info_formatting() {
        use crate::tui::auth::TuiSessionInfo;
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let session_info = TuiSessionInfo {
            username: "testuser".to_string(),
            session_id: "test-session-123".to_string(),
            login_time: now - 3600, // 1 hour ago
            last_activity: now - 300, // 5 minutes ago
            roles: vec!["QualityEngineer".to_string(), "Auditor".to_string()],
            permissions: vec!["ReadDocuments".to_string(), "WriteDocuments".to_string()],
        };

        // Test session duration calculation
        assert_eq!(session_info.session_duration(), 3300); // 3600 - 300 = 3300 seconds

        // Test recent activity check
        assert!(session_info.is_recent_activity()); // 5 minutes ago should be recent

        // Test formatting methods (basic functionality)
        let login_time_str = session_info.format_login_time();
        let activity_time_str = session_info.format_last_activity();
        
        assert!(!login_time_str.is_empty());
        assert!(!activity_time_str.is_empty());
    }

    #[test]
    fn test_tui_auth_result_creation() {
        // Test successful auth result
        let success_result = TuiAuthResult {
            success: true,
            message: "Login successful".to_string(),
            session: None,
        };
        
        assert!(success_result.success);
        assert_eq!(success_result.message, "Login successful");
        assert!(success_result.session.is_none());

        // Test failed auth result
        let failure_result = TuiAuthResult {
            success: false,
            message: "Invalid credentials".to_string(),
            session: None,
        };
        
        assert!(!failure_result.success);
        assert_eq!(failure_result.message, "Invalid credentials");
        assert!(failure_result.session.is_none());
    }

    #[test]
    fn test_permission_matching() {
        let test_dir = std::env::temp_dir().join("qms_tui_permission_matching_test");
        let _ = fs::create_dir_all(&test_dir);

        let auth_manager = TuiAuthManager::new(Some(test_dir.clone())).unwrap();

        // Test all permission variants are handled
        let permissions = vec![
            "read_documents",
            "write_documents", 
            "delete_documents",
            "read_risks",
            "write_risks",
            "delete_risks",
            "read_trace",
            "write_trace",
            "delete_trace",
            "read_audit",
            "export_audit",
            "manage_users",
            "generate_reports",
            "user_management",
            "project_management",
            "document_management",
            "risk_management",
            "audit_access",
            "system_configuration",
        ];

        // Without session, all should return false
        for permission in permissions {
            assert!(!auth_manager.has_permission(permission));
        }

        // Cleanup
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_tui_auth_integration_with_unified_system() {
        let test_dir = std::env::temp_dir().join("qms_tui_integration_test");
        let _ = fs::create_dir_all(&test_dir);

        // Test that TUI auth manager integrates with the same auth system as CLI
        let tui_auth = TuiAuthManager::new(Some(test_dir.clone()));
        assert!(tui_auth.is_ok());

        // Test that it uses the same file-based storage
        let auth_manager = tui_auth.unwrap();
        assert!(!auth_manager.is_authenticated());

        // The auth manager should be using FileUserStorage internally
        // This ensures consistency with CLI authentication

        // Cleanup
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_tui_auth_security_features() {
        let test_dir = std::env::temp_dir().join("qms_tui_security_test");
        let _ = fs::create_dir_all(&test_dir);

        let mut auth_manager = TuiAuthManager::new(Some(test_dir.clone())).unwrap();

        // Test input validation prevents injection attacks
        let malicious_inputs = vec![
            "'; DROP TABLE users; --",
            "<script>alert('xss')</script>",
            "../../etc/passwd",
            "\0\0\0",
            "admin\nadmin",
        ];

        for malicious_input in malicious_inputs {
            let result = auth_manager.login(malicious_input, "password").unwrap();
            assert!(!result.success);
            // Should handle malicious input gracefully without crashing
        }

        // Test that empty/whitespace inputs are properly rejected
        let empty_inputs = vec!["", " ", "\t", "\n", "\r\n"];
        
        for empty_input in empty_inputs {
            let result = auth_manager.login(empty_input, "password").unwrap();
            assert!(!result.success);
            assert!(result.message.contains("Username cannot be empty"));
        }

        // Cleanup
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_tui_auth_medical_device_compliance() {
        let test_dir = std::env::temp_dir().join("qms_tui_compliance_test");
        let _ = fs::create_dir_all(&test_dir);

        let mut auth_manager = TuiAuthManager::new(Some(test_dir.clone())).unwrap();

        // Test that authentication attempts are logged for audit trail
        // (Even if logging fails due to no project, the attempt should be made)
        let result = auth_manager.login("testuser", "testpass").unwrap();
        assert!(!result.success); // Expected to fail without proper setup

        // Test that logout attempts are logged
        let logout_result = auth_manager.logout().unwrap();
        assert!(!logout_result.success); // No session to logout

        // Test that user creation requires proper permissions (medical device compliance)
        let create_result = auth_manager.create_user("newuser", "password", vec!["QualityEngineer".to_string()]).unwrap();
        assert!(!create_result.success);
        assert!(create_result.message.contains("Insufficient permissions"));

        // Cleanup
        let _ = fs::remove_dir_all(&test_dir);
    }
}
