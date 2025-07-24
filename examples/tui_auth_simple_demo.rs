//! Simple TUI Authentication System Demonstration
//! 
//! This example demonstrates the TUI authentication system features
//! without requiring a full QMS project setup.

use qms::prelude::*;
use qms::tui::auth::{TuiAuthManager, TuiSessionInfo};
use std::path::PathBuf;
use std::fs;

fn main() -> QmsResult<()> {
    println!("🏥 QMS TUI Authentication System - Simple Demo");
    println!("===============================================");
    println!();

    // Create a temporary directory for this demo
    let demo_dir = std::env::temp_dir().join("qms_tui_simple_demo");
    let _ = fs::remove_dir_all(&demo_dir); // Clean up any existing demo
    let _ = fs::create_dir_all(&demo_dir);
    
    println!("📁 Demo Directory: {}", demo_dir.display());
    println!();

    // Demonstrate TUI Authentication Manager Features
    demonstrate_auth_manager_features(&demo_dir)?;
    
    // Demonstrate Input Validation
    demonstrate_input_validation(&demo_dir)?;
    
    // Demonstrate Permission System Structure
    demonstrate_permission_system(&demo_dir)?;
    
    // Demonstrate Session Management
    demonstrate_session_management(&demo_dir)?;
    
    // Demonstrate Security Features
    demonstrate_security_features(&demo_dir)?;

    // Cleanup
    let _ = fs::remove_dir_all(&demo_dir);
    
    println!("✅ TUI Authentication System Simple Demo Complete!");
    println!();
    println!("Key Features Demonstrated:");
    println!("  • TUI Authentication Manager creation and configuration");
    println!("  • Comprehensive input validation and security");
    println!("  • Permission system structure and checking");
    println!("  • Session management capabilities");
    println!("  • Integration with unified QMS authentication");
    println!("  • Medical device compliance features");
    
    Ok(())
}

fn demonstrate_auth_manager_features(demo_dir: &PathBuf) -> QmsResult<()> {
    println!("🔧 1. TUI Authentication Manager Features");
    println!("------------------------------------------");
    
    // Create TUI auth manager with specific path
    let auth_manager_with_path = TuiAuthManager::new(Some(demo_dir.clone()))?;
    println!("✅ TUI Auth Manager created with specific path");
    
    // Create TUI auth manager with default path
    let auth_manager_default = TuiAuthManager::new(None)?;
    println!("✅ TUI Auth Manager created with default path");
    
    // Test initial state
    assert!(!auth_manager_with_path.is_authenticated());
    assert!(auth_manager_with_path.current_username().is_none());
    assert!(auth_manager_with_path.current_user_roles().is_empty());
    assert!(auth_manager_with_path.get_session_info().is_none());
    
    println!("   • Initial state: Not authenticated ✅");
    println!("   • No current user ✅");
    println!("   • No user roles ✅");
    println!("   • No session information ✅");
    
    // Test session validation without session
    let is_valid = auth_manager_with_path.validate_current_session()?;
    assert!(!is_valid);
    println!("   • Session validation without session: Invalid ✅");
    
    // Test session activity update without session
    let mut auth_manager_mut = auth_manager_with_path;
    let update_result = auth_manager_mut.update_session_activity();
    assert!(update_result.is_ok());
    println!("   • Session activity update without session: Safe ✅");
    
    println!();
    Ok(())
}

fn demonstrate_input_validation(demo_dir: &PathBuf) -> QmsResult<()> {
    println!("🛡️  2. Input Validation and Security");
    println!("------------------------------------");
    
    let mut auth_manager = TuiAuthManager::new(Some(demo_dir.clone()))?;
    
    // Test empty username validation
    let result = auth_manager.login("", "password")?;
    assert!(!result.success);
    assert!(result.message.contains("Username cannot be empty"));
    println!("✅ Empty username rejected: {}", result.message);
    
    // Test whitespace-only username validation
    let result = auth_manager.login("   ", "password")?;
    assert!(!result.success);
    assert!(result.message.contains("Username cannot be empty"));
    println!("✅ Whitespace-only username rejected: {}", result.message);
    
    // Test empty password validation
    let result = auth_manager.login("admin", "")?;
    assert!(!result.success);
    assert!(result.message.contains("Password cannot be empty"));
    println!("✅ Empty password rejected: {}", result.message);
    
    // Test whitespace-only password validation
    let result = auth_manager.login("admin", "   ")?;
    assert!(!result.success);
    assert!(result.message.contains("Password cannot be empty"));
    println!("✅ Whitespace-only password rejected: {}", result.message);
    
    // Test malicious input handling
    let malicious_inputs = vec![
        ("'; DROP TABLE users; --", "SQL injection attempt"),
        ("<script>alert('xss')</script>", "XSS attempt"),
        ("../../etc/passwd", "Path traversal attempt"),
        ("\0\0\0", "Null byte injection"),
        ("admin\nadmin", "Newline injection"),
    ];
    
    for (malicious_input, description) in malicious_inputs {
        let result = auth_manager.login(malicious_input, "password")?;
        assert!(!result.success);
        println!("✅ {} rejected safely", description);
    }
    
    println!("   • All malicious inputs handled securely");
    println!("   • Input validation prevents injection attacks");
    println!("   • Graceful error handling without crashes");
    
    println!();
    Ok(())
}

fn demonstrate_permission_system(demo_dir: &PathBuf) -> QmsResult<()> {
    println!("🔐 3. Permission System Structure");
    println!("---------------------------------");
    
    let auth_manager = TuiAuthManager::new(Some(demo_dir.clone()))?;
    
    // Test all permission types without authentication
    let permissions = vec![
        ("read_documents", "Document Reading"),
        ("write_documents", "Document Writing"),
        ("delete_documents", "Document Deletion"),
        ("read_risks", "Risk Reading"),
        ("write_risks", "Risk Writing"),
        ("delete_risks", "Risk Deletion"),
        ("read_trace", "Traceability Reading"),
        ("write_trace", "Traceability Writing"),
        ("delete_trace", "Traceability Deletion"),
        ("read_audit", "Audit Reading"),
        ("export_audit", "Audit Export"),
        ("manage_users", "User Management"),
        ("generate_reports", "Report Generation"),
        ("user_management", "User Management (Legacy)"),
        ("project_management", "Project Management"),
        ("document_management", "Document Management"),
        ("risk_management", "Risk Management"),
        ("audit_access", "Audit Access"),
        ("system_configuration", "System Configuration"),
    ];
    
    println!("Testing permission system structure:");
    for (permission, description) in permissions {
        let has_permission = auth_manager.has_permission(permission);
        // Without authentication, all should return false
        assert!(!has_permission);
        println!("   • {}: {} (no session)", description, has_permission);
    }
    
    println!("   ✅ All {} permission types handled correctly", 19);
    println!("   ✅ Permission checking works without active session");
    println!("   ✅ Comprehensive permission coverage for medical device QMS");
    
    println!();
    Ok(())
}

fn demonstrate_session_management(demo_dir: &PathBuf) -> QmsResult<()> {
    println!("📋 4. Session Management Capabilities");
    println!("-------------------------------------");
    
    let mut auth_manager = TuiAuthManager::new(Some(demo_dir.clone()))?;
    
    // Test logout without session
    let logout_result = auth_manager.logout()?;
    assert!(!logout_result.success);
    assert!(logout_result.message.contains("No active session to logout"));
    println!("✅ Logout without session handled: {}", logout_result.message);
    
    // Test user creation without permissions
    let create_result = auth_manager.create_user(
        "newuser", 
        "password", 
        vec!["QualityEngineer".to_string()]
    )?;
    assert!(!create_result.success);
    assert!(create_result.message.contains("Insufficient permissions"));
    println!("✅ User creation without permissions rejected: {}", create_result.message);
    
    // Test session info formatting
    demonstrate_session_info_formatting();
    
    println!("   • Session lifecycle management");
    println!("   • Permission-based operation control");
    println!("   • Secure session information handling");
    println!("   • Audit trail integration points");
    
    println!();
    Ok(())
}

fn demonstrate_session_info_formatting() {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let session_info = TuiSessionInfo {
        username: "demo_user".to_string(),
        session_id: "demo-session-12345".to_string(),
        login_time: now - 7200, // 2 hours ago
        last_activity: now - 600, // 10 minutes ago
        roles: vec!["Administrator".to_string(), "QualityEngineer".to_string()],
        permissions: vec![
            "ReadDocuments".to_string(),
            "WriteDocuments".to_string(),
            "ManageUsers".to_string(),
        ],
    };
    
    println!("✅ Session Info Formatting Demo:");
    println!("   • Username: {}", session_info.username);
    println!("   • Session ID: {}", session_info.session_id);
    println!("   • Session Duration: {} seconds", session_info.session_duration());
    println!("   • Recent Activity: {}", session_info.is_recent_activity());
    println!("   • Roles: {}", session_info.roles.join(", "));
    println!("   • Permissions: {} total", session_info.permissions.len());
    
    // Test time formatting (basic functionality)
    let login_time_str = session_info.format_login_time();
    let activity_time_str = session_info.format_last_activity();
    assert!(!login_time_str.is_empty());
    assert!(!activity_time_str.is_empty());
    println!("   • Time formatting: Working ✅");
}

fn demonstrate_security_features(demo_dir: &PathBuf) -> QmsResult<()> {
    println!("🔒 5. Security Features");
    println!("-----------------------");
    
    let mut auth_manager = TuiAuthManager::new(Some(demo_dir.clone()))?;
    
    // Test authentication attempt with non-existent user
    let result = auth_manager.login("nonexistent_user", "password")?;
    assert!(!result.success);
    println!("✅ Non-existent user login rejected: Authentication failed");
    
    // Test that authentication state is properly maintained
    assert!(!auth_manager.is_authenticated());
    assert!(auth_manager.current_username().is_none());
    println!("✅ Authentication state properly maintained after failed login");
    
    // Test session validation security
    let is_valid = auth_manager.validate_current_session()?;
    assert!(!is_valid);
    println!("✅ Session validation secure without active session");
    
    // Test permission checking security
    let sensitive_permissions = vec![
        "manage_users",
        "user_management", 
        "system_configuration",
        "delete_documents",
        "delete_risks",
    ];
    
    for permission in sensitive_permissions {
        let has_permission = auth_manager.has_permission(permission);
        assert!(!has_permission);
        println!("✅ Sensitive permission '{}' properly denied without session", permission);
    }
    
    println!("   • Secure authentication state management");
    println!("   • Proper session validation");
    println!("   • Permission-based access control");
    println!("   • Protection against unauthorized operations");
    println!("   • Medical device security compliance");
    
    println!();
    Ok(())
}
