//! Authentication API Endpoints
//! 
//! This module provides REST API endpoints for the user-first authentication flow.
//! Handles initial admin setup, login, logout, and session management.
//! 
//! SOLID Principles:
//! - Single Responsibility: Handles only authentication-related API endpoints
//! - Open/Closed: Extensible for additional authentication methods
//! - Liskov Substitution: Uses standard HTTP request/response interfaces
//! - Interface Segregation: Focused on authentication operations only
//! - Dependency Inversion: Depends on authentication service abstractions

use crate::prelude::*;
use crate::web::{HttpRequest, HttpResponse};
use crate::web::response::HttpStatus;
use crate::modules::user_manager::{StartupAuthService, AdminSetupRequest, QmsFolderSetupRequest, FileBasedAuthService, UserSession, SessionType};
use crate::modules::audit_logger::audit_log_action;
use std::sync::Arc;
use std::path::PathBuf;

/// Authentication API handler using unified authentication service
pub struct AuthApiHandler {
    startup_service: Arc<StartupAuthService>,
    auth_service: Arc<FileBasedAuthService>,
}

impl AuthApiHandler {
    /// Create new authentication API handler using unified authentication service
    pub fn new() -> QmsResult<Self> {
        let startup_service = Arc::new(StartupAuthService::new()?);
        let auth_service = Arc::new(FileBasedAuthService::create_global()?);

        Ok(AuthApiHandler {
            startup_service,
            auth_service,
        })
    }
    
    /// Handle startup state check
    /// GET /api/auth/startup-state
    pub fn handle_startup_state(&self, _request: &HttpRequest) -> HttpResponse {
        match self.startup_service.check_startup_state() {
            Ok(state) => {
                let json = format!(
                    r#"{{
  "success": true,
  "requires_admin_setup": {},
  "has_existing_users": {},
  "system_initialized": {},
  "message": "{}"
}}"#,
                    state.requires_admin_setup,
                    state.has_existing_users,
                    state.system_initialized,
                    state.message
                );
                
                HttpResponse::json(&json)
            }
            Err(e) => {
                eprintln!("Error checking startup state: {e}");
                HttpResponse::internal_server_error("Failed to check startup state")
            }
        }
    }
    
    /// Handle admin setup
    /// POST /api/auth/setup-admin
    pub fn handle_admin_setup(&self, request: &HttpRequest) -> HttpResponse {
        // Parse request body
        let body = match request.get_body() {
            Some(body) => body,
            None => return HttpResponse::bad_request("Request body required"),
        };
        
        let username = match Self::extract_json_string(&body, "username") {
            Ok(username) => username,
            Err(_) => return HttpResponse::bad_request("Username required"),
        };

        let password = match Self::extract_json_string(&body, "password") {
            Ok(password) => password,
            Err(_) => return HttpResponse::bad_request("Password required"),
        };

        let confirm_password = match Self::extract_json_string(&body, "confirm_password") {
            Ok(confirm_password) => confirm_password,
            Err(_) => return HttpResponse::bad_request("Password confirmation required"),
        };

        let qms_folder_path = Self::extract_json_string(&body, "qms_folder_path")
            .ok()
            .map(PathBuf::from);
        
        let setup_request = AdminSetupRequest {
            username: username.clone(),
            password: password.clone(),
            confirm_password,
            qms_folder_path,
        };

        match self.startup_service.create_initial_admin(&setup_request) {
            Ok(profile) => {
                // Create session for the new admin using unified auth service
                match self.auth_service.web_login(&username, &password,
                                                 request.get_header("x-forwarded-for").map(|s| s.to_string()),
                                                 request.get_header("user-agent").map(|s| s.to_string())) {
                    Ok(session) => {
                        let json = format!(
                            r#"{{
  "success": true,
  "message": "Admin user created successfully",
  "user": {{
    "username": "{}",
    "qms_folder_path": "{}",
    "session_id": "{}"
  }}
}}"#,
                            profile.username,
                            profile.qms_folder_path.display(),
                            session.session_id
                        );
                
                        let mut response = HttpResponse::json(&json);
                        response.add_header("Set-Cookie", &format!("session_id={}; HttpOnly; Path=/", session.session_id));
                        response
                    }
                    Err(e) => {
                        let json = format!(
                            r#"{{
  "success": false,
  "message": "Failed to create session: {}"
}}"#,
                            e.to_string()
                        );
                        HttpResponse::new_with_body(HttpStatus::BadRequest, json)
                    }
                }
            }
            Err(e) => {
                let json = format!(
                    r#"{{
  "success": false,
  "message": "{}"
}}"#,
                    e.to_string()
                );
                
                HttpResponse::new_with_body(HttpStatus::BadRequest, json)
            }
        }
    }
    
    /// Handle user login
    /// POST /api/auth/login
    pub fn handle_login(&self, request: &HttpRequest) -> HttpResponse {
        let body = match request.get_body() {
            Some(body) => body,
            None => return HttpResponse::bad_request("Request body required"),
        };
        
        let username = match Self::extract_json_string(&body, "username") {
            Ok(username) => username,
            Err(_) => return HttpResponse::bad_request("Username required"),
        };

        let password = match Self::extract_json_string(&body, "password") {
            Ok(password) => password,
            Err(_) => return HttpResponse::bad_request("Password required"),
        };
        
        // Use unified authentication service for web login
        match self.auth_service.web_login(&username, &password,
                                         request.get_header("x-forwarded-for").map(|s| s.to_string()),
                                         request.get_header("user-agent").map(|s| s.to_string())) {
            Ok(session) => {
                println!("🔐 Created session: {}", session.session_id);
                println!("✅ Session authenticated: {} for user: {}", session.session_id, username);

                // Update user profile last login
                if let Ok(profile_manager) = crate::modules::user_manager::UserProfileManager::new() {
                    let _ = profile_manager.update_last_login(&username);
                }

                let json = format!(
                    r#"{{
  "success": true,
  "message": "Login successful",
  "user": {{
    "username": "{}",
    "session_id": "{}"
  }}
}}"#,
                    username,
                    session.session_id
                );

                let mut response = HttpResponse::json(&json);
                response.add_header("Set-Cookie", &format!("session_id={}; HttpOnly; Path=/", session.session_id));
                response
            }
            Err(_) => {
                HttpResponse::unauthorized("Invalid credentials")
            }
        }
    }
    
    /// Handle user logout
    /// POST /api/auth/logout
    pub fn handle_logout(&self, request: &HttpRequest) -> HttpResponse {
        if let Some(session_id) = self.extract_session_id(request) {
            // Use unified authentication service for logout
            match self.auth_service.logout(&session_id) {
                Ok(()) => {
                    let json = r#"{"success": true, "message": "Logged out successfully"}"#;
                    let mut response = HttpResponse::json(json);
                    response.add_header("Set-Cookie", "session_id=; HttpOnly; Path=/; Max-Age=0");
                    response
                }
                Err(_) => {
                    // Even if logout fails, clear the cookie
                    let json = r#"{"success": true, "message": "Logged out successfully"}"#;
                    let mut response = HttpResponse::json(json);
                    response.add_header("Set-Cookie", "session_id=; HttpOnly; Path=/; Max-Age=0");
                    response
                }
            }
        } else {
            HttpResponse::bad_request("No active session")
        }
    }
    
    /// Handle session validation
    /// GET /api/auth/session
    pub fn handle_session_check(&self, request: &HttpRequest) -> HttpResponse {
        if let Some(session_id) = self.extract_session_id(request) {
            // Use unified authentication service for session validation
            match self.auth_service.validate_session(&session_id) {
                Ok(session) => {
                    if session.is_authenticated() {
                        let json = format!(
                            r#"{{
  "success": true,
  "authenticated": true,
  "user": {{
    "username": "{}",
    "session_id": "{}"
  }}
}}"#,
                            session.username,
                            session_id
                        );
                        return HttpResponse::json(&json);
                    }
                }
                Err(_) => {
                    // Session not found or expired
                }
            }
        }

        let json = r#"{"success": true, "authenticated": false}"#;
        HttpResponse::json(json)
    }
    
    /// Handle QMS folder setup
    /// POST /api/auth/setup-qms-folder
    pub fn handle_qms_folder_setup(&self, request: &HttpRequest) -> HttpResponse {
        // Check authentication
        if let Err(response) = self.check_authentication(request) {
            return response;
        }
        
        let body = match request.get_body() {
            Some(body) => body,
            None => return HttpResponse::bad_request("Request body required"),
        };
        
        let username = match self.get_authenticated_username(request) {
            Some(username) => username,
            None => return HttpResponse::unauthorized("Authentication required"),
        };
        
        let qms_folder_path = match Self::extract_json_string(&body, "qms_folder_path") {
            Ok(path) => PathBuf::from(path),
            Err(_) => return HttpResponse::bad_request("QMS folder path required"),
        };

        let use_default = Self::extract_json_string(&body, "use_default")
            .map(|s| s == "true")
            .unwrap_or(false);
        
        let setup_request = QmsFolderSetupRequest {
            username: username.clone(),
            qms_folder_path,
            use_default,
        };
        
        match self.startup_service.setup_qms_folder(&setup_request) {
            Ok(profile) => {
                let json = format!(
                    r#"{{
  "success": true,
  "message": "QMS folder setup completed",
  "qms_folder_path": "{}"
}}"#,
                    profile.qms_folder_path.display()
                );
                HttpResponse::json(&json)
            }
            Err(e) => {
                let json = format!(
                    r#"{{
  "success": false,
  "message": "{}"
}}"#,
                    e.to_string()
                );
                HttpResponse::new_with_body(HttpStatus::BadRequest, json)
            }
        }
    }
    
    /// Get default QMS folder path
    /// GET /api/auth/default-qms-path
    pub fn handle_default_qms_path(&self, _request: &HttpRequest) -> HttpResponse {
        match StartupAuthService::get_default_qms_path() {
            Ok(path) => {
                let json = format!(
                    r#"{{
  "success": true,
  "default_path": "{}"
}}"#,
                    path.display()
                );
                HttpResponse::json(&json)
            }
            Err(e) => {
                let json = format!(
                    r#"{{
  "success": false,
  "message": "{}"
}}"#,
                    e.to_string()
                );
                HttpResponse::internal_server_error(&json)
            }
        }
    }
    
    // Helper methods
    
    /// Extract session ID from request
    fn extract_session_id(&self, request: &HttpRequest) -> Option<String> {
        // Check cookie header
        if let Some(cookie_header) = request.headers.get("Cookie") {
            for cookie in cookie_header.split(';') {
                let cookie = cookie.trim();
                if let Some(session_id) = cookie.strip_prefix("session_id=") {
                    return Some(session_id.to_string());
                }
            }
        }
        None
    }
    
    /// Check if request is authenticated
    fn check_authentication(&self, request: &HttpRequest) -> Result<(), HttpResponse> {
        if let Some(session_id) = self.extract_session_id(request) {
            match self.auth_service.validate_session(&session_id) {
                Ok(session) => {
                    if session.is_authenticated() {
                        return Ok(());
                    }
                }
                Err(_) => {
                    // Session not found or expired
                }
            }
        }
        Err(HttpResponse::unauthorized("Authentication required"))
    }
    
    /// Get authenticated username from request
    fn get_authenticated_username(&self, request: &HttpRequest) -> Option<String> {
        if let Some(session_id) = self.extract_session_id(request) {
            match self.auth_service.validate_session(&session_id) {
                Ok(session) => {
                    if session.is_authenticated() {
                        return Some(session.username);
                    }
                }
                Err(_) => {
                    // Session not found or expired
                }
            }
        }
        None
    }
    
    /// Extract string value from JSON
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
    

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::audit_logger::{AuditConfig, initialize_audit_system};
    use std::env;
    
    fn init_audit_for_test() {
        let temp_dir = env::temp_dir().join("qms_auth_api_test");
        let _ = std::fs::create_dir_all(&temp_dir);
        
        let audit_dir = temp_dir.join("audit");
        let _ = std::fs::create_dir_all(&audit_dir);

        let config = AuditConfig {
            project_path: temp_dir.to_string_lossy().to_string(),
            retention_days: 30,
            daily_rotation: false,
            max_file_size_mb: 10,
            require_checksums: false,
        };

        let _ = initialize_audit_system(config);
    }
    
    #[test]
    fn test_auth_api_handler_creation() {
        init_audit_for_test();
        let handler = AuthApiHandler::new().unwrap();

        // Test startup state check
        let request = HttpRequest::new_with_params("GET", "/api/auth/startup-state", std::collections::HashMap::new(), None);
        let response = handler.handle_startup_state(&request);

        // Should indicate admin setup is required
        let body_str = String::from_utf8(response.body).unwrap();
        assert!(body_str.contains("requires_admin_setup"));
    }
}
