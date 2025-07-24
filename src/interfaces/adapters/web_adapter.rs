//! Web Adapter for Unified Interface System
//! 
//! This module provides an adapter that bridges the existing web server system
//! to the new unified interface abstractions, enabling shared routing and
//! command handling between CLI and web interfaces.

use crate::prelude::*;
use crate::interfaces::{InterfaceContext, InterfaceManager, CommandResult, InterfaceType};
use crate::interfaces::routing::{CommandHandler, UnifiedRouter};
use crate::web::{HttpRequest, HttpResponse};
use crate::web::response::HttpStatus;
use crate::json_utils::{JsonValue, JsonSerializable};
use std::sync::Arc;
use std::collections::HashMap;

/// Web Router Adapter - Bridges web HTTP requests to unified command routing
pub struct WebRouterAdapter {
    interface_manager: InterfaceManager,
}

impl WebRouterAdapter {
    /// Create new web router adapter
    pub fn new(project_path: Option<std::path::PathBuf>) -> QmsResult<Self> {
        let interface_manager = crate::interfaces::InterfaceFactory::create_web_manager(project_path)?;
        
        Ok(Self {
            interface_manager,
        })
    }

    /// Route HTTP request to unified command system
    pub fn route_http_request(&self, request: &HttpRequest) -> HttpResponse {
        // Create web interface context
        let mut context = InterfaceContext::new(InterfaceType::Web);

        // Extract session information from request if available
        if let Some(session) = self.extract_session_from_request(request) {
            context = context.with_user_session(session);
        }

        // Parse the request path to extract command and arguments
        let (command, args) = self.parse_request_path(&request.uri);

        // Execute command through unified interface system
        match self.interface_manager.execute_command(&mut context, &command, &args) {
            Ok(result) => self.create_json_response(result),
            Err(e) => self.create_error_response(HttpStatus::InternalServerError, &format!("Command failed: {}", e)),
        }
    }

    /// Extract session from HTTP request
    fn extract_session_from_request(&self, request: &HttpRequest) -> Option<crate::modules::user_manager::UserSession> {
        // Extract session from cookies or headers
        // This would integrate with the existing UnifiedSessionAdapter
        None // Simplified for now
    }

    /// Parse HTTP request path to extract command and arguments
    fn parse_request_path(&self, uri: &str) -> (String, Vec<String>) {
        let path_parts: Vec<&str> = uri.trim_start_matches('/').split('/').collect();
        
        match path_parts.as_slice() {
            ["api", command, args @ ..] => {
                (format!("api/{}", command), args.iter().map(|s| s.to_string()).collect())
            }
            [command, args @ ..] => {
                (command.to_string(), args.iter().map(|s| s.to_string()).collect())
            }
            _ => ("help".to_string(), Vec::new()),
        }
    }

    /// Create JSON response from command result
    fn create_json_response(&self, result: CommandResult) -> HttpResponse {
        let mut json_obj = HashMap::new();
        json_obj.insert("success".to_string(), JsonValue::Bool(result.success));
        json_obj.insert("message".to_string(), JsonValue::String(result.message));
        
        if let Some(data) = result.data {
            json_obj.insert("data".to_string(), data);
        }
        
        if result.requires_user_input {
            json_obj.insert("requires_input".to_string(), JsonValue::Bool(true));
            if let Some(next_action) = result.next_action {
                json_obj.insert("next_action".to_string(), JsonValue::String(next_action));
            }
        }

        let json_response = JsonValue::Object(json_obj);
        
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Access-Control-Allow-Origin".to_string(), "*".to_string());

        HttpResponse {
            status: if result.success { HttpStatus::Ok } else { HttpStatus::BadRequest },
            headers,
            body: json_response.to_json().into_bytes(),
        }
    }

    /// Create error response
    fn create_error_response(&self, status: HttpStatus, message: &str) -> HttpResponse {
        let mut json_obj = HashMap::new();
        json_obj.insert("success".to_string(), JsonValue::Bool(false));
        json_obj.insert("error".to_string(), JsonValue::String(message.to_string()));

        let json_response = JsonValue::Object(json_obj);

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Access-Control-Allow-Origin".to_string(), "*".to_string());

        HttpResponse {
            status,
            headers,
            body: json_response.to_json().into_bytes(),
        }
    }
}

/// Web Interface Manager - Manages web interface using unified abstractions
pub struct WebInterfaceManager {
    interface_manager: InterfaceManager,
    router_adapter: WebRouterAdapter,
}

impl WebInterfaceManager {
    /// Create new web interface manager
    pub fn new(project_path: Option<std::path::PathBuf>) -> QmsResult<Self> {
        let interface_manager = crate::interfaces::InterfaceFactory::create_web_manager(project_path.clone())?;
        let router_adapter = WebRouterAdapter::new(project_path)?;

        Ok(Self {
            interface_manager,
            router_adapter,
        })
    }

    /// Handle HTTP request through unified interface system
    pub fn handle_request(&self, request: &HttpRequest) -> HttpResponse {
        self.router_adapter.route_http_request(request)
    }

    /// Handle authentication request
    pub fn handle_auth_request(&self, request: &HttpRequest) -> HttpResponse {
        // Parse authentication data from request body
        let (username, password) = match self.parse_auth_request(request) {
            Ok((u, p)) => (u, p),
            Err(e) => return self.router_adapter.create_error_response(
                HttpStatus::BadRequest, 
                &format!("Invalid auth request: {}", e)
            ),
        };

        // Create web interface context
        let mut context = InterfaceContext::new(InterfaceType::Web);

        // Attempt authentication
        match self.interface_manager.authenticate(&mut context, &username, &password) {
            Ok(()) => {
                // Create success response with session information
                let mut json_obj = HashMap::new();
                json_obj.insert("success".to_string(), JsonValue::Bool(true));
                json_obj.insert("message".to_string(), JsonValue::String("Authentication successful".to_string()));
                
                if let Some(session) = &context.user_session {
                    json_obj.insert("session_id".to_string(), JsonValue::String(session.session_id.clone()));
                    json_obj.insert("username".to_string(), JsonValue::String(session.username.clone()));
                }

                let json_response = JsonValue::Object(json_obj);

                let mut headers = HashMap::new();
                headers.insert("Content-Type".to_string(), "application/json".to_string());
                headers.insert("Access-Control-Allow-Origin".to_string(), "*".to_string());
                // Set session cookie
                headers.insert("Set-Cookie".to_string(), format!("session_id={}; HttpOnly; Path=/",
                    context.user_session.as_ref().map(|s| &s.session_id).unwrap_or(&"".to_string())));

                HttpResponse {
                    status: HttpStatus::Ok,
                    headers,
                    body: json_response.to_json().into_bytes(),
                }
            }
            Err(e) => self.router_adapter.create_error_response(
                HttpStatus::Unauthorized, 
                &format!("Authentication failed: {}", e)
            ),
        }
    }

    /// Parse authentication request
    fn parse_auth_request(&self, request: &HttpRequest) -> QmsResult<(String, String)> {
        let body = String::from_utf8_lossy(&request.body);
        
        // Try to parse as JSON
        if let Ok(json_value) = JsonValue::parse(&body) {
            if let JsonValue::Object(obj) = json_value {
                let username = obj.get("username")
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| QmsError::validation_error("Missing username"))?;
                
                let password = obj.get("password")
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| QmsError::validation_error("Missing password"))?;
                
                return Ok((username.clone(), password.clone()));
            }
        }

        // Try to parse as form data
        if body.contains("username=") && body.contains("password=") {
            let mut username = String::new();
            let mut password = String::new();
            
            for pair in body.split('&') {
                if let Some((key, value)) = pair.split_once('=') {
                    match key {
                        "username" => username = urlencoding::decode(value).unwrap_or_default().to_string(),
                        "password" => password = urlencoding::decode(value).unwrap_or_default().to_string(),
                        _ => {}
                    }
                }
            }
            
            if !username.is_empty() && !password.is_empty() {
                return Ok((username, password));
            }
        }

        Err(QmsError::validation_error("Invalid authentication request format"))
    }
}

/// Web Command Bridge - Bridges web API endpoints to CLI commands
pub struct WebCommandBridge {
    cli_manager: crate::interfaces::adapters::cli_adapter::CliInterfaceManager,
}

impl WebCommandBridge {
    /// Create new web command bridge
    pub fn new(project_path: Option<std::path::PathBuf>) -> QmsResult<Self> {
        let cli_manager = crate::interfaces::adapters::cli_adapter::CliInterfaceManager::new(project_path)?;
        
        Ok(Self {
            cli_manager,
        })
    }

    /// Execute CLI command from web request
    pub fn execute_cli_command(&mut self, command: &str, args: &[String]) -> QmsResult<CommandResult> {
        self.cli_manager.execute_command(command, args)
    }

    /// Bridge web API call to CLI command
    pub fn bridge_api_call(&mut self, api_path: &str, request: &HttpRequest) -> HttpResponse {
        // Map API paths to CLI commands
        let (command, args) = self.map_api_to_cli_command(api_path, request);

        // Execute CLI command
        match self.execute_cli_command(&command, &args) {
            Ok(result) => self.create_api_response(result),
            Err(e) => self.create_error_response(&format!("Command failed: {}", e)),
        }
    }

    /// Map API path to CLI command
    fn map_api_to_cli_command(&self, api_path: &str, request: &HttpRequest) -> (String, Vec<String>) {
        match api_path {
            "/api/projects" => ("project".to_string(), vec!["list".to_string()]),
            "/api/risks" => ("risk".to_string(), vec!["list".to_string()]),
            "/api/documents" => ("doc".to_string(), vec!["list".to_string()]),
            "/api/requirements" => ("req".to_string(), vec!["list".to_string()]),
            "/api/audit" => ("audit".to_string(), vec!["stats".to_string()]),
            "/api/health" => ("version".to_string(), Vec::new()),
            _ => ("help".to_string(), Vec::new()),
        }
    }

    /// Create API response from command result
    fn create_api_response(&self, result: CommandResult) -> HttpResponse {
        let mut json_obj = HashMap::new();
        json_obj.insert("success".to_string(), JsonValue::Bool(result.success));
        json_obj.insert("message".to_string(), JsonValue::String(result.message));
        
        if let Some(data) = result.data {
            json_obj.insert("data".to_string(), data);
        }

        let json_response = JsonValue::Object(json_obj);

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Access-Control-Allow-Origin".to_string(), "*".to_string());

        HttpResponse {
            status: if result.success { HttpStatus::Ok } else { HttpStatus::BadRequest },
            headers,
            body: json_response.to_json().into_bytes(),
        }
    }

    /// Create error response
    fn create_error_response(&self, message: &str) -> HttpResponse {
        let mut json_obj = HashMap::new();
        json_obj.insert("success".to_string(), JsonValue::Bool(false));
        json_obj.insert("error".to_string(), JsonValue::String(message.to_string()));

        let json_response = JsonValue::Object(json_obj);

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Access-Control-Allow-Origin".to_string(), "*".to_string());

        HttpResponse {
            status: HttpStatus::InternalServerError,
            headers,
            body: json_response.to_json().into_bytes(),
        }
    }
}

/// URL encoding utility (simplified implementation)
mod urlencoding {
    use std::borrow::Cow;

    pub fn decode(input: &str) -> Option<Cow<str>> {
        // Simplified URL decoding - in production would use proper URL decoding
        Some(Cow::Borrowed(input))
    }
}
