// QMS User Management API Examples
// Integration examples and usage demonstrations for the user management REST API
// Shows how to set up the API and provides sample requests/responses

use crate::prelude::*;
use crate::web::{ApiRouter, UserApiHandler};
use std::path::Path;

/// Setup user management API with router
pub fn setup_user_api(project_path: &Path) -> QmsResult<ApiRouter> {
    let mut router = ApiRouter::new();
    let user_api = UserApiHandler::new(project_path)?;
    
    // Setup all user API routes
    user_api.setup_routes(&mut router)?;
    
    Ok(router)
}

/// Example JSON request for user login
pub fn example_login_request() -> String {
    "{
    \"username\": \"quality-engineer\",
    \"password\": \"securePassword123\"
}".to_string()
}

/// Example JSON response for successful login
pub fn example_login_response() -> String {
    "{
    \"success\": true,
    \"session_id\": \"ses-a1b2c3d4e5f6\",
    \"user_id\": \"quality-engineer\",
    \"username\": \"quality-engineer\",
    \"roles\": [\"QualityEngineer\"]
}".to_string()
}

/// Example JSON request for creating a new user
pub fn example_create_user_request() -> String {
    "{
    \"username\": \"new-engineer\",
    \"password\": \"strongPassword456\",
    \"roles\": [\"QualityEngineer\"]
}".to_string()
}

/// Example JSON response for user creation
pub fn example_create_user_response() -> String {
    "{
    \"success\": true,
    \"user\": {
        \"username\": \"new-engineer\",
        \"created_at\": 1738067400,
        \"roles\": [\"QualityEngineer\"]
    }
}".to_string()
}

/// Example JSON response for listing users
pub fn example_list_users_response() -> String {
    "{
    \"users\": [
        {
            \"username\": \"admin\",
            \"created_at\": 1738000000,
            \"last_login\": 1738067200,
            \"roles\": [\"Administrator\"]
        },
        {
            \"username\": \"quality-engineer\",
            \"created_at\": 1738010000,
            \"last_login\": 1738067100,
            \"roles\": [\"QualityEngineer\"]
        },
        {
            \"username\": \"auditor\",
            \"created_at\": 1738020000,
            \"last_login\": null,
            \"roles\": [\"Auditor\"]
        }
    ],
    \"total\": 3
}".to_string()
}

/// Example JSON response for session information
pub fn example_session_info_response() -> String {
    "{
    \"authenticated\": true,
    \"user_id\": \"quality-engineer\",
    \"session_id\": \"ses-a1b2c3d4e5f6\",
    \"permissions\": [\"ReadDocuments\", \"WriteDocuments\", \"ReadRisks\", \"WriteRisks\", \"ReadTrace\", \"WriteTrace\", \"ReadAudit\", \"GenerateReports\"],
    \"created_at\": 1738067400,
    \"last_accessed\": 1738067500
}".to_string()
}

/// Example JSON response for listing available roles
pub fn example_list_roles_response() -> String {
    "{
    \"roles\": [
        {
            \"name\": \"Administrator\",
            \"description\": \"Full system access including user management and system configuration\",
            \"permissions\": [\"ManageUsers\", \"ReadDocuments\", \"WriteDocuments\", \"DeleteDocuments\", \"ReadRisks\", \"WriteRisks\", \"DeleteRisks\", \"ReadTrace\", \"WriteTrace\", \"DeleteTrace\", \"ReadAudit\", \"ExportAudit\", \"GenerateReports\"]
        },
        {
            \"name\": \"QualityEngineer\",
            \"description\": \"Quality management functions including document control and risk management\",
            \"permissions\": [\"ReadDocuments\", \"WriteDocuments\", \"ReadRisks\", \"WriteRisks\", \"ReadTrace\", \"WriteTrace\", \"ReadAudit\", \"GenerateReports\"]
        },
        {
            \"name\": \"Auditor\",
            \"description\": \"Read-only access for audit and compliance activities\",
            \"permissions\": [\"ReadDocuments\", \"ReadRisks\", \"ReadTrace\", \"ReadAudit\", \"ExportAudit\"]
        }
    ]
}".to_string()
}

/// Example JSON request for assigning role to user
pub fn example_assign_role_request() -> String {
    "{
    \"role\": \"Administrator\"
}".to_string()
}

/// Example JSON response for successful role assignment
pub fn example_assign_role_response() -> String {
    "{
    \"success\": true,
    \"message\": \"Role 'Administrator' assigned to user 'quality-engineer'\"
}".to_string()
}

/// Example JSON response for user permissions
pub fn example_user_permissions_response() -> String {
    "{
    \"username\": \"quality-engineer\",
    \"permissions\": [\"ReadDocuments\", \"WriteDocuments\", \"ReadRisks\", \"WriteRisks\", \"ReadTrace\", \"WriteTrace\", \"ReadAudit\", \"GenerateReports\"]
}".to_string()
}

/// Example usage of the User Management API endpoints
pub fn demonstrate_api_usage() {
    println!("=== QMS User Management API Usage Examples ===");
    
    println!("\n1. User Authentication:");
    println!("POST /api/v1/auth/login");
    println!("Request: {}", example_login_request());
    println!("Response: {}", example_login_response());
    
    println!("\n2. Session Information:");
    println!("GET /api/v1/auth/session");
    println!("Headers: Authorization: Bearer ses-a1b2c3d4e5f6");
    println!("Response: {}", example_session_info_response());
    
    println!("\n3. Create New User:");
    println!("POST /api/v1/users");
    println!("Headers: Authorization: Bearer ses-admin-token");
    println!("Request: {}", example_create_user_request());
    println!("Response: {}", example_create_user_response());
    
    println!("\n4. List All Users:");
    println!("GET /api/v1/users");
    println!("Headers: Authorization: Bearer ses-admin-token");
    println!("Response: {}", example_list_users_response());
    
    println!("\n5. List Available Roles:");
    println!("GET /api/v1/roles");
    println!("Headers: Authorization: Bearer ses-a1b2c3d4e5f6");
    println!("Response: {}", example_list_roles_response());
    
    println!("\n6. Assign Role to User:");
    println!("POST /api/v1/users/quality-engineer/roles");
    println!("Headers: Authorization: Bearer ses-admin-token");
    println!("Request: {}", example_assign_role_request());
    println!("Response: {}", example_assign_role_response());
    
    println!("\n7. Get User Permissions:");
    println!("GET /api/v1/users/quality-engineer/permissions");
    println!("Headers: Authorization: Bearer ses-a1b2c3d4e5f6");
    println!("Response: {}", example_user_permissions_response());
    
    println!("\n8. User Logout:");
    println!("POST /api/v1/auth/logout");
    println!("Headers: Authorization: Bearer ses-a1b2c3d4e5f6");
    println!("Response: {{\"success\": true, \"message\": \"Logged out successfully\"}}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::modules::audit_logger::{AuditConfig, initialize_audit_system};

    fn init_audit_for_test(path: &std::path::Path) {
        let config = AuditConfig {
            project_path: path.to_string_lossy().to_string(),
            retention_days: 30,
            daily_rotation: false,
            max_file_size_mb: 10,
            require_checksums: false,
        };
        let _ = initialize_audit_system(config);
    }

    #[test]
    fn test_api_setup() {
        let temp_dir = tempdir().unwrap();
        init_audit_for_test(temp_dir.path());
        
        let result = setup_user_api(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_example_json_parsing() {
        let login_json = example_login_request();
        assert!(login_json.contains("quality-engineer"));
        assert!(login_json.contains("securePassword123"));
        
        let response_json = example_login_response();
        assert!(response_json.contains("success"));
        assert!(response_json.contains("session_id"));
        
        let users_json = example_list_users_response();
        assert!(users_json.contains("users"));
        assert!(users_json.contains("total"));
    }

    #[test]
    fn test_role_examples() {
        let roles_json = example_list_roles_response();
        assert!(roles_json.contains("Administrator"));
        assert!(roles_json.contains("QualityEngineer"));
        assert!(roles_json.contains("Auditor"));
        assert!(roles_json.contains("permissions"));
    }

    #[test]
    fn test_session_examples() {
        let session_json = example_session_info_response();
        assert!(session_json.contains("authenticated"));
        assert!(session_json.contains("permissions"));
        assert!(session_json.contains("ReadDocuments"));
    }

    #[test]
    fn test_user_creation_examples() {
        let create_json = example_create_user_request();
        assert!(create_json.contains("new-engineer"));
        assert!(create_json.contains("strongPassword456"));
        
        let response_json = example_create_user_response();
        assert!(response_json.contains("success"));
        assert!(response_json.contains("created_at"));
    }
}
