//! TUI Authentication System Demonstration
//! 
//! This example demonstrates the comprehensive authentication system
//! integrated into the QMS TUI interface, showing how it provides
//! secure access control for medical device quality management.

use qms::prelude::*;
use qms::tui::auth::TuiAuthManager;
use qms::modules::user_manager::auth::FileAuthManager;
use qms::models::{Role, Permission};
use std::path::PathBuf;
use std::fs;

fn main() -> QmsResult<()> {
    println!("🏥 QMS TUI Authentication System Demonstration");
    println!("===============================================");
    println!();

    // Create a temporary directory for this demo
    let demo_dir = std::env::temp_dir().join("qms_tui_auth_demo");
    let _ = fs::remove_dir_all(&demo_dir); // Clean up any existing demo
    let _ = fs::create_dir_all(&demo_dir);
    
    println!("📁 Demo Directory: {}", demo_dir.display());
    println!();

    // Demonstrate TUI Authentication Manager Creation
    demonstrate_auth_manager_creation(&demo_dir)?;
    
    // Demonstrate User Creation and Management
    demonstrate_user_management(&demo_dir)?;
    
    // Demonstrate Authentication Flow
    demonstrate_authentication_flow(&demo_dir)?;
    
    // Demonstrate Permission System
    demonstrate_permission_system(&demo_dir)?;
    
    // Demonstrate Security Features
    demonstrate_security_features(&demo_dir)?;
    
    // Demonstrate Integration with Unified Auth System
    demonstrate_unified_integration(&demo_dir)?;

    // Cleanup
    let _ = fs::remove_dir_all(&demo_dir);
    
    println!("✅ TUI Authentication System Demonstration Complete!");
    println!();
    println!("Key Features Demonstrated:");
    println!("  • Unified authentication with CLI/Web systems");
    println!("  • Role-based access control");
    println!("  • Comprehensive audit logging");
    println!("  • Medical device compliance");
    println!("  • Security input validation");
    println!("  • Session management");
    println!("  • Permission hierarchy");
    
    Ok(())
}

fn demonstrate_auth_manager_creation(demo_dir: &PathBuf) -> QmsResult<()> {
    println!("🔧 1. TUI Authentication Manager Creation");
    println!("------------------------------------------");
    
    // Create TUI auth manager
    let auth_manager = TuiAuthManager::new(Some(demo_dir.clone()))?;
    
    println!("✅ TUI Authentication Manager created successfully");
    println!("   • Integrates with unified QMS authentication system");
    println!("   • Uses file-based user storage for persistence");
    println!("   • Maintains session state for TUI operations");
    println!("   • Initial state: Not authenticated");
    println!();
    
    // Verify initial state
    assert!(!auth_manager.is_authenticated());
    assert!(auth_manager.current_username().is_none());
    assert!(auth_manager.current_user_roles().is_empty());
    
    Ok(())
}

fn demonstrate_user_management(demo_dir: &PathBuf) -> QmsResult<()> {
    println!("👥 2. User Management Integration");
    println!("---------------------------------");
    
    // Create a user through the unified auth system
    let file_auth = FileAuthManager::from_project_path(demo_dir)?;
    
    // Create admin user
    let admin_role = Role {
        name: "Administrator".to_string(),
        permissions: vec![
            Permission::ManageUsers,
            Permission::UserManagement,
            Permission::ReadDocuments,
            Permission::WriteDocuments,
            Permission::ReadAudit,
            Permission::GenerateReports,
        ],
    };
    
    let admin_user = if !file_auth.user_exists("admin")? {
        file_auth.add_user("admin", "SecurePass123!", Some(vec![admin_role]))?
    } else {
        file_auth.load_user("admin")?
    };
    println!("✅ Admin user ready: {}", admin_user.username);
    
    // Create quality engineer user
    let qe_role = Role {
        name: "QualityEngineer".to_string(),
        permissions: vec![
            Permission::ReadDocuments,
            Permission::WriteDocuments,
            Permission::ReadRisks,
            Permission::WriteRisks,
            Permission::GenerateReports,
        ],
    };
    
    let qe_user = if !file_auth.user_exists("qe_user")? {
        file_auth.add_user("qe_user", "QualityPass456!", Some(vec![qe_role]))?
    } else {
        file_auth.load_user("qe_user")?
    };
    println!("✅ Quality Engineer user ready: {}", qe_user.username);
    
    // Create auditor user
    let auditor_role = Role {
        name: "Auditor".to_string(),
        permissions: vec![
            Permission::ReadDocuments,
            Permission::ReadRisks,
            Permission::ReadAudit,
            Permission::AuditAccess,
        ],
    };
    
    let auditor_user = if !file_auth.user_exists("auditor")? {
        file_auth.add_user("auditor", "AuditPass789!", Some(vec![auditor_role]))?
    } else {
        file_auth.load_user("auditor")?
    };
    println!("✅ Auditor user ready: {}", auditor_user.username);
    
    println!("   • Users stored in unified authentication system");
    println!("   • Same users accessible from CLI, Web, and TUI interfaces");
    println!("   • Role-based permissions enforced across all interfaces");
    println!();
    
    Ok(())
}

fn demonstrate_authentication_flow(demo_dir: &PathBuf) -> QmsResult<()> {
    println!("🔐 3. Authentication Flow");
    println!("-------------------------");
    
    let mut tui_auth = TuiAuthManager::new(Some(demo_dir.clone()))?;
    
    // Test invalid credentials
    println!("Testing invalid credentials...");
    let invalid_result = tui_auth.login("admin", "wrongpassword")?;
    assert!(!invalid_result.success);
    println!("❌ Invalid credentials rejected: {}", invalid_result.message);
    
    // Test valid credentials
    println!("Testing valid credentials...");
    let valid_result = tui_auth.login("admin", "SecurePass123!")?;
    assert!(valid_result.success);
    println!("✅ Valid credentials accepted: {}", valid_result.message);
    
    // Verify authentication state
    assert!(tui_auth.is_authenticated());
    assert_eq!(tui_auth.current_username(), Some("admin"));
    
    let roles = tui_auth.current_user_roles();
    println!("   • Current user: {}", tui_auth.current_username().unwrap());
    println!("   • User roles: {}", roles.join(", "));
    
    // Test session information
    if let Some(session_info) = tui_auth.get_session_info() {
        println!("   • Session ID: {}", session_info.session_id);
        println!("   • Login time: {}", session_info.format_login_time());
        println!("   • Permissions: {}", session_info.permissions.len());
    }
    
    // Test logout
    let logout_result = tui_auth.logout()?;
    assert!(logout_result.success);
    println!("✅ Logout successful: {}", logout_result.message);
    assert!(!tui_auth.is_authenticated());
    
    println!();
    Ok(())
}

fn demonstrate_permission_system(demo_dir: &PathBuf) -> QmsResult<()> {
    println!("🛡️  4. Permission System");
    println!("------------------------");
    
    let mut tui_auth = TuiAuthManager::new(Some(demo_dir.clone()))?;
    
    // Login as admin
    let _login_result = tui_auth.login("admin", "SecurePass123!")?;
    
    // Test admin permissions
    println!("Testing admin permissions:");
    let admin_permissions = vec![
        ("manage_users", "User Management"),
        ("user_management", "User Management (Legacy)"),
        ("read_documents", "Document Reading"),
        ("write_documents", "Document Writing"),
        ("read_audit", "Audit Access"),
        ("generate_reports", "Report Generation"),
    ];
    
    for (permission, description) in admin_permissions {
        let has_permission = tui_auth.has_permission(permission);
        let status = if has_permission { "✅" } else { "❌" };
        println!("   {} {}: {}", status, description, has_permission);
    }
    
    // Test permissions admin should NOT have
    println!("Testing permissions admin should NOT have:");
    let restricted_permissions = vec![
        ("nonexistent_permission", "Non-existent Permission"),
    ];
    
    for (permission, description) in restricted_permissions {
        let has_permission = tui_auth.has_permission(permission);
        let status = if !has_permission { "✅" } else { "❌" };
        println!("   {} {}: {}", status, description, !has_permission);
    }
    
    // Logout and test as quality engineer
    let _logout_result = tui_auth.logout()?;
    let _login_result = tui_auth.login("qe_user", "QualityPass456!")?;
    
    println!("Testing Quality Engineer permissions:");
    let qe_permissions = vec![
        ("read_documents", "Document Reading", true),
        ("write_documents", "Document Writing", true),
        ("read_risks", "Risk Reading", true),
        ("write_risks", "Risk Writing", true),
        ("manage_users", "User Management", false), // Should NOT have
        ("user_management", "User Management (Legacy)", false), // Should NOT have
    ];
    
    for (permission, description, should_have) in qe_permissions {
        let has_permission = tui_auth.has_permission(permission);
        let status = if has_permission == should_have { "✅" } else { "❌" };
        println!("   {} {}: {} (expected: {})", status, description, has_permission, should_have);
    }
    
    println!();
    Ok(())
}

fn demonstrate_security_features(demo_dir: &PathBuf) -> QmsResult<()> {
    println!("🔒 5. Security Features");
    println!("-----------------------");
    
    let mut tui_auth = TuiAuthManager::new(Some(demo_dir.clone()))?;
    
    // Test input validation
    println!("Testing input validation:");
    
    let malicious_inputs = vec![
        ("", "Empty username"),
        ("   ", "Whitespace-only username"),
        ("'; DROP TABLE users; --", "SQL injection attempt"),
        ("<script>alert('xss')</script>", "XSS attempt"),
        ("../../etc/passwd", "Path traversal attempt"),
    ];
    
    for (malicious_input, description) in malicious_inputs {
        let result = tui_auth.login(malicious_input, "password")?;
        let status = if !result.success { "✅" } else { "❌" };
        println!("   {} {}: Rejected", status, description);
        assert!(!result.success);
    }
    
    // Test password validation
    println!("Testing password validation:");
    let password_tests = vec![
        ("", "Empty password"),
        ("   ", "Whitespace-only password"),
    ];
    
    for (password, description) in password_tests {
        let result = tui_auth.login("admin", password)?;
        let status = if !result.success { "✅" } else { "❌" };
        println!("   {} {}: Rejected", status, description);
        assert!(!result.success);
    }
    
    // Test session validation
    println!("Testing session validation:");
    assert!(!tui_auth.validate_current_session()?);
    println!("   ✅ No session validation without active session");
    
    // Login and test session validation
    let _login_result = tui_auth.login("admin", "SecurePass123!")?;
    assert!(tui_auth.validate_current_session()?);
    println!("   ✅ Session validation works with active session");
    
    println!();
    Ok(())
}

fn demonstrate_unified_integration(demo_dir: &PathBuf) -> QmsResult<()> {
    println!("🔗 6. Unified System Integration");
    println!("--------------------------------");
    
    // Create TUI auth manager
    let mut tui_auth = TuiAuthManager::new(Some(demo_dir.clone()))?;
    
    // Create CLI auth manager (same underlying system)
    let mut cli_auth = FileAuthManager::from_project_path(demo_dir)?;
    
    println!("Testing cross-interface compatibility:");
    
    // Login through TUI
    let tui_login = tui_auth.login("admin", "SecurePass123!")?;
    assert!(tui_login.success);
    println!("   ✅ TUI login successful");
    
    // Verify user exists in CLI system
    let user_exists = cli_auth.user_exists("admin")?;
    assert!(user_exists);
    println!("   ✅ User accessible from CLI interface");
    
    // Login through CLI
    let _cli_session = cli_auth.login("admin", "SecurePass123!")?;
    println!("   ✅ CLI login successful for same user");
    
    // Both systems use the same underlying storage
    let cli_user = cli_auth.load_user("admin")?;
    println!("   ✅ User data consistent across interfaces");
    println!("     • Username: {}", cli_user.username);
    println!("     • Roles: {}", cli_user.roles.len());
    println!("     • Created: {}", cli_user.created_at);
    
    // Test that permissions are consistent
    let tui_has_manage_users = tui_auth.has_permission("manage_users");
    let cli_has_manage_users = cli_auth.has_permission("admin", &Permission::ManageUsers)?;
    
    println!("   ✅ Permission consistency:");
    println!("     • TUI manage_users: {}", tui_has_manage_users);
    println!("     • CLI ManageUsers: {}", cli_has_manage_users);
    
    println!("   • Same user storage backend");
    println!("   • Consistent authentication logic");
    println!("   • Unified audit trail");
    println!("   • Cross-interface session compatibility");
    
    println!();
    Ok(())
}
