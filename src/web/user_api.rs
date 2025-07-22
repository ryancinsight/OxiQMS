// QMS User Management API
// REST endpoints for user authentication, session management, and role-based access control
// Integrates with existing Phase 2 user management functionality
// Uses stdlib only for JSON serialization and HTTP handling

use crate::prelude::*;
use crate::web::{HttpRequest, HttpResponse, ApiRouter, SessionManager};
use crate::modules::user_manager::FileAuthManager;
use crate::modules::user_manager::roles::RoleManager;
use crate::models::{Role, Permission};
use std::path::Path;

/// User API handler providing REST endpoints for user management and authentication
pub struct UserApiHandler {
    auth_manager: std::sync::Arc<std::sync::Mutex<FileAuthManager>>,
    role_manager: RoleManager,
    session_manager: SessionManager,
}

impl UserApiHandler {
    /// Create new user API handler
    pub fn new(project_path: &Path) -> QmsResult<Self> {
        let auth_manager = std::sync::Arc::new(std::sync::Mutex::new(FileAuthManager::from_project_path(project_path)?));
        let role_manager = RoleManager::new(project_path)?;
        let session_manager = SessionManager::new();
        
        Ok(UserApiHandler {
            auth_manager,
            role_manager,
            session_manager,
        })
    }

    /// Helper method to create JSON response (DRY principle)
    fn json_response(&self, json_content: &str) -> HttpResponse {
        HttpResponse::ok_with_string(json_content, "application/json")
    }

    /// Helper method to convert request body to string (DRY principle)
    fn body_to_string<'a>(&self, body: &'a [u8]) -> Result<&'a str, HttpResponse> {
        match std::str::from_utf8(body) {
            Ok(s) => Ok(s),
            Err(_) => Err(HttpResponse::bad_request("Invalid UTF-8 in request body")),
        }
    }

    /// Setup user API routes on the router
    pub fn setup_routes(&self, _router: &mut ApiRouter) -> QmsResult<()> {
        // Note: Due to Rust's ownership model and lifetime constraints,
        // user API routes will be handled directly in the main router dispatch logic
        // This avoids the complex closure lifetime issues while maintaining functionality

        // For now, we'll document the supported endpoints:
        // POST /api/v1/auth/login - User login
        // POST /api/v1/auth/logout - User logout
        // GET /api/v1/auth/session - Session info
        // POST /api/v1/auth/validate - Validate session
        
        // GET /api/v1/users - List users
        // POST /api/v1/users - Create user
        // GET /api/v1/users/{id} - Get user
        // PUT /api/v1/users/{id} - Update user
        // DELETE /api/v1/users/{id} - Delete user
        // GET /api/v1/roles - List roles
        // GET /api/v1/users/{id}/roles - Get user roles
        // POST /api/v1/users/{id}/roles - Assign role
        // DELETE /api/v1/users/{id}/roles/{role} - Remove role
        // GET /api/v1/users/{id}/permissions - Get user permissions
        
        Ok(())
    }
    
    /// Handle user login authentication
    fn handle_login(&mut self, request: &HttpRequest) -> HttpResponse {
        // Validate request method
        if request.method != "POST" {
            return HttpResponse::method_not_allowed(&["POST"]);
        }
        
        // Parse JSON request body
        if request.body.is_empty() {
            return HttpResponse::bad_request("Request body required");
        }
        let body_str = match self.body_to_string(&request.body) {
            Ok(s) => s,
            Err(response) => return response,
        };

        let username = match self.extract_json_string(body_str, "username") {
            Ok(username) => username,
            Err(_) => {
                return HttpResponse::bad_request("Username required");
            }
        };
        
        let password = match self.extract_json_string(body_str, "password") {
            Ok(password) => password,
            Err(_) => {
                return HttpResponse::bad_request("Password required");
            }
        };
        
        // Authenticate user
        let login_result = {
            let mut auth_manager = self.auth_manager.lock().unwrap();
            auth_manager.login(&username, &password)
        };
        
        match login_result {
            Ok(user_session) => {
                // Create web session
                let web_session = self.session_manager.create_session(request);
                let session_id = web_session.id.clone();
                
                // Convert user roles to permission strings
                let permissions: Vec<String> = user_session.roles.iter()
                    .flat_map(|role| &role.permissions)
                    .map(|perm| self.permission_to_string(perm).to_string())
                    .collect();
                
                // Authenticate the web session
                match self.session_manager.authenticate_session(&session_id, &username, permissions) {
                    Ok(authenticated_session) => {
                        let response_json = format!(
                            "{{\"success\": true, \"session_id\": \"{}\", \"user_id\": \"{}\", \"username\": \"{}\", \"roles\": [{}]}}",
                            authenticated_session.id,
                            username,
                            username,
                            self.format_roles(&user_session.roles)
                        );
                        self.json_response(&response_json)
                    }
                    Err(e) => {
                        HttpResponse::internal_error(&format!("Session creation failed: {e}"))
                    }
                }
            }
            Err(e) => {
                HttpResponse::unauthorized(&format!("Authentication failed: {e}"))
            }
        }
    }
    
    /// Handle user logout
    fn handle_logout(&self, request: &HttpRequest) -> HttpResponse {
        // Validate request method
        if request.method != "POST" {
            return HttpResponse::method_not_allowed(&["POST"]);
        }
        
        // Get session from request header or cookie
        let session_id = match self.extract_session_id(request) {
            Some(session_id) => session_id,
            None => {
                return HttpResponse::unauthorized("No active session found");
            }
        };
        
        // Remove session
        let success = self.session_manager.remove_session(&session_id);
        
        if success {
            let response_json = "{\"success\": true, \"message\": \"Logged out successfully\"}";
            self.json_response(response_json)
        } else {
            HttpResponse::not_found("Session not found")
        }
    }
    
    /// Handle session information request
    fn handle_session_info(&self, request: &HttpRequest) -> HttpResponse {
        // Validate request method
        if request.method != "GET" {
            return HttpResponse::method_not_allowed(&["GET"]);
        }
        
        // Get session from request
        let session_id = match self.extract_session_id(request) {
            Some(session_id) => session_id,
            None => {
                return HttpResponse::unauthorized("No active session found");
            }
        };
        
        // Get session details
        match self.session_manager.get_session(&session_id) {
            Some(session) => {
                if session.is_authenticated() {
                    let response_json = format!(
                        "{{\"authenticated\": true, \"user_id\": \"{}\", \"session_id\": \"{}\", \"permissions\": [{}], \"created_at\": {}, \"last_accessed\": {}}}",
                        session.user_id.as_ref().unwrap_or(&"unknown".to_string()),
                        session.id,
                        self.format_permissions(&session.permissions),
                        session.created_at,
                        session.last_accessed
                    );
                    self.json_response(&response_json)
                } else {
                    let response_json = "{\"authenticated\": false, \"message\": \"Session not authenticated\"}";
                    self.json_response(response_json)
                }
            }
            None => {
                HttpResponse::not_found("Session not found")
            }
        }
    }
    
    /// Handle session validation request
    fn handle_validate_session(&self, request: &HttpRequest) -> HttpResponse {
        // Validate request method
        if request.method != "POST" {
            return HttpResponse::method_not_allowed(&["POST"]);
        }
        
        // Parse JSON request body for session_id
        if request.body.is_empty() {
            return HttpResponse::bad_request("Request body required");
        }
        let body_str = match self.body_to_string(&request.body) {
            Ok(s) => s,
            Err(response) => return response,
        };

        let session_id = match self.extract_json_string(body_str, "session_id") {
            Ok(session_id) => session_id,
            Err(_) => {
                return HttpResponse::bad_request("Session ID required");
            }
        };
        
        // Validate session
        match self.session_manager.get_session(&session_id) {
            Some(session) => {
                if session.is_authenticated() {
                    let response_json = format!(
                        "{{\"valid\": true, \"user_id\": \"{}\", \"permissions\": [{}]}}",
                        session.user_id.as_ref().unwrap_or(&"unknown".to_string()),
                        self.format_permissions(&session.permissions)
                    );
                    self.json_response(&response_json)
                } else {
                    let response_json = "{\"valid\": false, \"message\": \"Session not authenticated\"}";
                    self.json_response(response_json)
                }
            }
            None => {
                let response_json = "{\"valid\": false, \"message\": \"Session not found\"}";
                self.json_response(response_json)
            }
        }
    }
    
    /// Handle list users request
    fn handle_list_users(&self, request: &HttpRequest) -> HttpResponse {
        // Validate request method
        if request.method != "GET" {
            return HttpResponse::method_not_allowed(&["GET"]);
        }
        
        // Check authentication and permissions
        if let Err(response) = self.check_authentication_and_permission(request, "ManageUsers") {
            return response;
        }
        
        // Get all users
        let users_result = {
            let auth_manager = self.auth_manager.lock().unwrap();
            auth_manager.list_users()
        };
        
        match users_result {
            Ok(users) => {
                let users_json: Vec<String> = users.iter().map(|user| {
                    format!(
                        "{{\"username\": \"{}\", \"created_at\": {}, \"last_login\": {}, \"roles\": [{}]}}",
                        user.username,
                        user.created_at,
                        user.last_login.map(|t| t.to_string()).unwrap_or_else(|| "null".to_string()),
                        self.format_roles(&user.roles)
                    )
                }).collect();
                
                let response_json = format!("{{\"users\": [{}], \"total\": {}}}", users_json.join(","), users.len());
                self.json_response(&response_json)
            }
            Err(e) => {
                HttpResponse::internal_error(&format!("Failed to list users: {e}"))
            }
        }
    }
    
    /// Handle create user request
    fn handle_create_user(&self, request: &HttpRequest) -> HttpResponse {
        // Validate request method
        if request.method != "POST" {
            return HttpResponse::method_not_allowed(&["POST"]);
        }
        
        // Check authentication and permissions
        if let Err(response) = self.check_authentication_and_permission(request, "ManageUsers") {
            return response;
        }
        
        // Parse JSON request body
        if request.body.is_empty() {
            return HttpResponse::bad_request("Request body required");
        }
        let body_str = match self.body_to_string(&request.body) {
            Ok(s) => s,
            Err(response) => return response,
        };

        let username = match self.extract_json_string(body_str, "username") {
            Ok(username) => username,
            Err(_) => {
                return HttpResponse::bad_request("Username required");
            }
        };
        
        let password = match self.extract_json_string(body_str, "password") {
            Ok(password) => password,
            Err(_) => {
                return HttpResponse::bad_request("Password required");
            }
        };

        // Extract optional roles
        let roles = self.extract_json_roles(body_str).ok();
        
        // Create user
        let create_result = {
            let auth_manager = self.auth_manager.lock().unwrap();
            auth_manager.add_user(&username, &password, roles)
        };
        
        match create_result {
            Ok(user) => {
                let response_json = format!(
                    "{{\"success\": true, \"user\": {{\"username\": \"{}\", \"created_at\": {}, \"roles\": [{}]}}}}",
                    user.username,
                    user.created_at,
                    self.format_roles(&user.roles)
                );
                HttpResponse::created(response_json.as_bytes().to_vec(), "application/json")
            }
            Err(e) => {
                HttpResponse::bad_request(&format!("Failed to create user: {e}"))
            }
        }
    }
    
    /// Handle get user request
    fn handle_get_user(&self, request: &HttpRequest) -> HttpResponse {
        // Validate request method
        if request.method != "GET" {
            return HttpResponse::method_not_allowed(&["GET"]);
        }
        
        // Check authentication
        if let Err(response) = self.check_authentication(request) {
            return response;
        }
        
        // Extract user ID from path
        let username = match self.extract_path_parameter(request, "id") {
            Some(username) => username,
            None => {
                return HttpResponse::bad_request("User ID required in path");
            }
        };
        
        // Get user details
        let user_result = {
            let auth_manager = self.auth_manager.lock().unwrap();
            auth_manager.load_user(&username)
        };
        
        match user_result {
            Ok(user) => {
                let response_json = format!(
                    "{{\"username\": \"{}\", \"created_at\": {}, \"last_login\": {}, \"roles\": [{}]}}",
                    user.username,
                    user.created_at,
                    user.last_login.map(|t| t.to_string()).unwrap_or_else(|| "null".to_string()),
                    self.format_roles(&user.roles)
                );
                self.json_response(&response_json)
            }
            Err(_) => {
                HttpResponse::not_found("User not found")
            }
        }
    }
    
    /// Handle update user request
    fn handle_update_user(&self, request: &HttpRequest) -> HttpResponse {
        // Validate request method
        if request.method != "PUT" {
            return HttpResponse::method_not_allowed(&["PUT"]);
        }
        
        // Check authentication and permissions
        if let Err(response) = self.check_authentication_and_permission(request, "ManageUsers") {
            return response;
        }
        
        // Extract user ID from path
        let username = match self.extract_path_parameter(request, "id") {
            Some(username) => username,
            None => {
                return HttpResponse::bad_request("User ID required in path");
            }
        };
        
        // Parse JSON request body
        if request.body.is_empty() {
            return HttpResponse::bad_request("Request body required");
        }
        let body_str = match self.body_to_string(&request.body) {
            Ok(s) => s,
            Err(response) => return response,
        };
        
        // Load existing user
        let mut user = {
            let auth_manager = self.auth_manager.lock().unwrap();
            match auth_manager.load_user(&username) {
                Ok(user) => user,
                Err(_) => {
                    return HttpResponse::not_found("User not found");
                }
            }
        };
        
        // Update password if provided
        if let Ok(new_password) = self.extract_json_string(body_str, "password") {
            if new_password.len() >= 8 {
                user.password_hash = FileAuthManager::hash_password(&new_password);
            } else {
                return HttpResponse::bad_request("Password must be at least 8 characters");
            }
        }
        
        // Update roles if provided
        if let Ok(roles) = self.extract_json_roles(body_str) {
            user.roles = roles;
        }
        
        // Save updated user
        let save_result = {
            let auth_manager = self.auth_manager.lock().unwrap();
            auth_manager.save_user(&user)
        };
        
        match save_result {
            Ok(_) => {
                let response_json = format!(
                    "{{\"success\": true, \"user\": {{\"username\": \"{}\", \"created_at\": {}, \"roles\": [{}]}}}}",
                    user.username,
                    user.created_at,
                    self.format_roles(&user.roles)
                );
                self.json_response(&response_json)
            }
            Err(e) => {
                HttpResponse::internal_error(&format!("Failed to update user: {e}"))
            }
        }
    }
    
    /// Handle delete user request
    fn handle_delete_user(&self, request: &HttpRequest) -> HttpResponse {
        // Validate request method
        if request.method != "DELETE" {
            return HttpResponse::method_not_allowed(&["DELETE"]);
        }
        
        // Check authentication and permissions
        if let Err(response) = self.check_authentication_and_permission(request, "ManageUsers") {
            return response;
        }
        
        // Extract user ID from path
        let username = match self.extract_path_parameter(request, "id") {
            Some(username) => username,
            None => {
                return HttpResponse::bad_request("User ID required in path");
            }
        };
        
        // Note: For medical device compliance, we typically don't delete users
        // Instead, we mark them as inactive or archived
        let response_json = format!(
            "{{\"success\": false, \"message\": \"User deletion not allowed for regulatory compliance. User '{username}' should be deactivated instead.\"}}"
        );
        HttpResponse::forbidden(&response_json)
    }
    
    /// Handle list roles request
    fn handle_list_roles(&self, request: &HttpRequest) -> HttpResponse {
        // Validate request method
        if request.method != "GET" {
            return HttpResponse::method_not_allowed(&["GET"]);
        }
        
        // Check authentication
        if let Err(response) = self.check_authentication(request) {
            return response;
        }
        
        // Get available roles
        let roles = self.role_manager.get_available_roles();
        let role_descriptions = self.role_manager.get_role_descriptions();
        
        let roles_json: Vec<String> = roles.iter().map(|role| {
            let default_desc = "No description".to_string();
            let description = role_descriptions.get(&role.name).unwrap_or(&default_desc);
            format!(
                "{{\"name\": \"{}\", \"description\": \"{}\", \"permissions\": [{}]}}",
                role.name,
                description,
                self.format_permissions_enum(&role.permissions)
            )
        }).collect();
        
        let response_json = format!("{{\"roles\": [{}]}}", roles_json.join(","));
        self.json_response(&response_json)
    }
    
    /// Handle get user roles request
    fn handle_get_user_roles(&self, request: &HttpRequest) -> HttpResponse {
        // Validate request method
        if request.method != "GET" {
            return HttpResponse::method_not_allowed(&["GET"]);
        }
        
        // Check authentication
        if let Err(response) = self.check_authentication(request) {
            return response;
        }
        
        // Extract user ID from path
        let username = match self.extract_path_parameter(request, "id") {
            Some(username) => username,
            None => {
                return HttpResponse::bad_request("User ID required in path");
            }
        };
        
        // Get user roles
        match self.role_manager.get_user_roles(&username) {
            Ok(roles) => {
                let response_json = format!(
                    "{{\"username\": \"{}\", \"roles\": [{}]}}",
                    username,
                    self.format_roles(&roles)
                );
                self.json_response(&response_json)
            }
            Err(_) => {
                HttpResponse::not_found("User not found")
            }
        }
    }
    
    /// Handle assign role request
    fn handle_assign_role(&self, request: &HttpRequest) -> HttpResponse {
        // Validate request method
        if request.method != "POST" {
            return HttpResponse::method_not_allowed(&["POST"]);
        }
        
        // Check authentication and permissions
        if let Err(response) = self.check_authentication_and_permission(request, "ManageUsers") {
            return response;
        }
        
        // Extract user ID from path
        let username = match self.extract_path_parameter(request, "id") {
            Some(username) => username,
            None => {
                return HttpResponse::bad_request("User ID required in path");
            }
        };
        
        // Parse JSON request body
        if request.body.is_empty() {
            return HttpResponse::bad_request("Request body required");
        }
        let body_str = match self.body_to_string(&request.body) {
            Ok(s) => s,
            Err(response) => return response,
        };

        let role_name = match self.extract_json_string(body_str, "role") {
            Ok(role_name) => role_name,
            Err(_) => {
                return HttpResponse::bad_request("Role name required");
            }
        };
        
        // Assign role to user
        match self.role_manager.assign_role(&username, &role_name) {
            Ok(_) => {
                let response_json = format!(
                    "{{\"success\": true, \"message\": \"Role '{role_name}' assigned to user '{username}'\"}}"
                );
                self.json_response(&response_json)
            }
            Err(e) => {
                HttpResponse::bad_request(&format!("Failed to assign role: {e}"))
            }
        }
    }
    
    /// Handle remove role request
    fn handle_remove_role(&self, request: &HttpRequest) -> HttpResponse {
        // Validate request method
        if request.method != "DELETE" {
            return HttpResponse::method_not_allowed(&["DELETE"]);
        }
        
        // Check authentication and permissions
        if let Err(response) = self.check_authentication_and_permission(request, "ManageUsers") {
            return response;
        }
        
        // Extract user ID and role from path
        let username = match self.extract_path_parameter(request, "id") {
            Some(username) => username,
            None => {
                return HttpResponse::bad_request("User ID required in path");
            }
        };
        
        let role_name = match self.extract_path_parameter(request, "role") {
            Some(role_name) => role_name,
            None => {
                return HttpResponse::bad_request("Role name required in path");
            }
        };
        
        // Remove role from user
        match self.role_manager.remove_role(&username, &role_name) {
            Ok(_) => {
                let response_json = format!(
                    "{{\"success\": true, \"message\": \"Role '{role_name}' removed from user '{username}'\"}}"
                );
                self.json_response(&response_json)
            }
            Err(e) => {
                HttpResponse::bad_request(&format!("Failed to remove role: {e}"))
            }
        }
    }
    
    /// Handle get user permissions request
    fn handle_get_user_permissions(&self, request: &HttpRequest) -> HttpResponse {
        // Validate request method
        if request.method != "GET" {
            return HttpResponse::method_not_allowed(&["GET"]);
        }
        
        // Check authentication
        if let Err(response) = self.check_authentication(request) {
            return response;
        }
        
        // Extract user ID from path
        let username = match self.extract_path_parameter(request, "id") {
            Some(username) => username,
            None => {
                return HttpResponse::bad_request("User ID required in path");
            }
        };
        
        // Get user permissions
        match self.role_manager.get_user_permissions(&username) {
            Ok(permissions) => {
                let response_json = format!(
                    "{{\"username\": \"{}\", \"permissions\": [{}]}}",
                    username,
                    self.format_permissions_enum(&permissions)
                );
                self.json_response(&response_json)
            }
            Err(_) => {
                HttpResponse::not_found("User not found")
            }
        }
    }
    
    // Helper methods
    
    /// Check if request is authenticated
    fn check_authentication(&self, request: &HttpRequest) -> Result<(), HttpResponse> {
        let session_id = match self.extract_session_id(request) {
            Some(session_id) => session_id,
            None => {
                return Err(HttpResponse::unauthorized("Authentication required"));
            }
        };
        
        match self.session_manager.get_session(&session_id) {
            Some(session) => {
                if session.is_authenticated() {
                    Ok(())
                } else {
                    Err(HttpResponse::unauthorized("Session not authenticated"))
                }
            }
            None => {
                Err(HttpResponse::unauthorized("Invalid session"))
            }
        }
    }
    
    /// Check if request is authenticated and has specific permission
    fn check_authentication_and_permission(&self, request: &HttpRequest, permission: &str) -> Result<(), HttpResponse> {
        let session_id = match self.extract_session_id(request) {
            Some(session_id) => session_id,
            None => {
                return Err(HttpResponse::unauthorized("Authentication required"));
            }
        };
        
        match self.session_manager.get_session(&session_id) {
            Some(session) => {
                if session.is_authenticated() {
                    if session.has_permission(permission) {
                        Ok(())
                    } else {
                        Err(HttpResponse::forbidden("Insufficient permissions"))
                    }
                } else {
                    Err(HttpResponse::unauthorized("Session not authenticated"))
                }
            }
            None => {
                Err(HttpResponse::unauthorized("Invalid session"))
            }
        }
    }
    
    /// Extract session ID from request headers or cookies
    fn extract_session_id(&self, request: &HttpRequest) -> Option<String> {
        // Try Authorization header first
        if let Some(auth_header) = request.headers.get("authorization") {
            if auth_header.starts_with("Bearer ") {
                return Some(auth_header[7..].to_string());
            }
        }
        
        // Try Cookie header
        if let Some(cookie_header) = request.headers.get("cookie") {
            for cookie in cookie_header.split(';') {
                let cookie = cookie.trim();
                if cookie.starts_with("session_id=") {
                    return Some(cookie[11..].to_string());
                }
            }
        }
        
        None
    }
    
    /// Extract path parameter from request
    fn extract_path_parameter(&self, request: &HttpRequest, _param_name: &str) -> Option<String> {
        // Simple path parameter extraction
        // In a full implementation, this would use proper URL routing
        request.path().split('/').collect::<Vec<&str>>().get(4).map(|path_parts| path_parts.to_string())
    }
    
    /// Extract string value from JSON
    fn extract_json_string(&self, json: &str, key: &str) -> QmsResult<String> {
        let key_pattern = format!("\"{key}\":");
        if let Some(start) = json.find(&key_pattern) {
            let value_start = start + key_pattern.len();
            let remaining = &json[value_start..].trim_start();
            
            if remaining.starts_with('"') {
                let end_quote = remaining[1..].find('"').unwrap_or(0) + 1;
                Ok(remaining[1..end_quote].to_string())
            } else {
                Err(QmsError::validation_error("Invalid JSON string value"))
            }
        } else {
            Err(QmsError::validation_error("Key not found in JSON"))
        }
    }
    
    /// Extract roles from JSON
    fn extract_json_roles(&self, json: &str) -> QmsResult<Vec<Role>> {
        // Simple role extraction - in a full implementation this would be more robust
        if let Ok(roles_str) = self.extract_json_string(json, "roles") {
            // Parse roles array (simplified)
            let mut roles = Vec::new();
            if roles_str == "Administrator" {
                roles.push(self.get_admin_role());
            } else if roles_str == "QualityEngineer" {
                roles.push(self.get_quality_engineer_role());
            } else {
                roles.push(self.get_quality_engineer_role()); // Default
            }
            Ok(roles)
        } else {
            Ok(vec![self.get_quality_engineer_role()]) // Default role
        }
    }
    
    /// Format roles for JSON output
    fn format_roles(&self, roles: &[Role]) -> String {
        roles.iter().map(|role| format!("\"{}\"", role.name)).collect::<Vec<_>>().join(",")
    }
    
    /// Format permissions (strings) for JSON output
    fn format_permissions(&self, permissions: &[String]) -> String {
        permissions.iter().map(|perm| format!("\"{perm}\"")).collect::<Vec<_>>().join(",")
    }
    
    /// Format permissions (enums) for JSON output
    fn format_permissions_enum(&self, permissions: &[Permission]) -> String {
        permissions.iter().map(|perm| format!("\"{}\"", self.permission_to_string(perm))).collect::<Vec<_>>().join(",")
    }
    
    /// Convert permission enum to string
    const fn permission_to_string(&self, permission: &Permission) -> &str {
        match permission {
            Permission::ManageUsers => "ManageUsers",
            Permission::ReadDocuments => "ReadDocuments",
            Permission::WriteDocuments => "WriteDocuments",
            Permission::DeleteDocuments => "DeleteDocuments",
            Permission::ReadRisks => "ReadRisks",
            Permission::WriteRisks => "WriteRisks",
            Permission::DeleteRisks => "DeleteRisks",
            Permission::ReadTrace => "ReadTrace",
            Permission::WriteTrace => "WriteTrace",
            Permission::DeleteTrace => "DeleteTrace",
            Permission::ReadAudit => "ReadAudit",
            Permission::ExportAudit => "ExportAudit",
            Permission::GenerateReports => "GenerateReports",
        }
    }
    
    /// Get admin role
    fn get_admin_role(&self) -> Role {
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
    
    fn create_test_request(method: &str, path: &str, body: Option<String>) -> HttpRequest {
        HttpRequest {
            method: method.to_string(),
            uri: path.to_string(),
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            body: body.map(|s| s.into_bytes()).unwrap_or_default(),
            query_params: HashMap::new(),
            timestamp: 0,
        }
    }
    
    #[test]
    fn test_user_api_creation() {
        let temp_dir = tempdir().unwrap();
        init_audit_for_test(temp_dir.path());
        
        let user_api = UserApiHandler::new(temp_dir.path());
        assert!(user_api.is_ok());
    }
    
    #[test]
    fn test_login_endpoint() {
        let temp_dir = tempdir().unwrap();
        init_audit_for_test(temp_dir.path());
        
        let mut user_api = UserApiHandler::new(temp_dir.path()).unwrap();
        
        // Create test user first
        {
            let auth_manager = user_api.auth_manager.lock().unwrap();
            let _ = auth_manager.add_user("testuser", "password123", None);
        }
        
        // Test login
        let login_body = "{\"username\": \"testuser\", \"password\": \"password123\"}".to_string();
        let request = create_test_request("POST", "/api/v1/auth/login", Some(login_body));
        
        let response = user_api.handle_login(&request);
        assert_eq!(response.status, crate::web::response::HttpStatus::Ok);
        let body_str = String::from_utf8(response.body).unwrap();
        assert!(body_str.contains("success"));
    }
    
    #[test]
    fn test_invalid_login() {
        let temp_dir = tempdir().unwrap();
        init_audit_for_test(temp_dir.path());
        
        let mut user_api = UserApiHandler::new(temp_dir.path()).unwrap();
        
        // Test login with invalid credentials
        let login_body = "{\"username\": \"invalid\", \"password\": \"wrong\"}".to_string();
        let request = create_test_request("POST", "/api/v1/auth/login", Some(login_body));
        
        let response = user_api.handle_login(&request);
        assert_eq!(response.status, crate::web::response::HttpStatus::Unauthorized);
    }
    
    #[test]
    fn test_json_extraction() {
        let temp_dir = tempdir().unwrap();
        init_audit_for_test(temp_dir.path());
        
        let user_api = UserApiHandler::new(temp_dir.path()).unwrap();
        
        let json = "{\"username\": \"testuser\", \"password\": \"secret123\"}";
        
        let username = user_api.extract_json_string(json, "username").unwrap();
        assert_eq!(username, "testuser");
        
        let password = user_api.extract_json_string(json, "password").unwrap();
        assert_eq!(password, "secret123");
    }
    
    #[test]
    fn test_permission_formatting() {
        let temp_dir = tempdir().unwrap();
        init_audit_for_test(temp_dir.path());
        
        let user_api = UserApiHandler::new(temp_dir.path()).unwrap();
        
        let permissions = vec![Permission::ReadDocuments, Permission::WriteDocuments];
        let formatted = user_api.format_permissions_enum(&permissions);
        assert!(formatted.contains("ReadDocuments"));
        assert!(formatted.contains("WriteDocuments"));
    }
    
    #[test]
    fn test_role_formatting() {
        let temp_dir = tempdir().unwrap();
        init_audit_for_test(temp_dir.path());
        
        let user_api = UserApiHandler::new(temp_dir.path()).unwrap();
        
        let roles = vec![user_api.get_admin_role()];
        let formatted = user_api.format_roles(&roles);
        assert!(formatted.contains("Administrator"));
    }
    
    #[test]
    fn test_list_roles_endpoint() {
        let temp_dir = tempdir().unwrap();
        init_audit_for_test(temp_dir.path());
        
        let mut user_api = UserApiHandler::new(temp_dir.path()).unwrap();
        
        let request = create_test_request("GET", "/api/v1/roles", None);
        // Note: This test would require authentication in a real scenario
        // For now, we're testing the basic structure
        let response = user_api.handle_list_roles(&request);
        // Would expect 401 without authentication
        assert_eq!(response.status, crate::web::response::HttpStatus::Unauthorized);
    }
}
