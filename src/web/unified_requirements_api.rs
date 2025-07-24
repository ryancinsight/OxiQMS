// Unified Requirements API - Web routes that delegate to CLI command infrastructure
// Follows SOLID, CUPID, GRASP, ACID, KISS, DRY, SOC, and YAGNI principles

use crate::prelude::*;
use crate::web::{
    HttpRequest, HttpResponse, 
    WebCommandBridge,
    unified_auth_context::UnifiedAuthContext,
    command_bridge::ClientInfo
};
use crate::web::response::HttpStatus;

/// Unified Requirements API Handler - delegates to CLI command infrastructure
pub struct UnifiedRequirementsApiHandler {
    command_bridge: WebCommandBridge,
}

impl UnifiedRequirementsApiHandler {
    /// Create new unified requirements API handler
    pub fn new() -> Self {
        Self {
            command_bridge: WebCommandBridge::new(),
        }
    }
    
    /// Handle GET /api/requirements - List requirements
    pub fn handle_list_requirements(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("req_list_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Build CLI arguments
        let args = vec![];
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "req", "list", args)?;
        
        Ok(result.to_http_response())
    }
    
    /// Handle GET /api/requirements/{id} - Get specific requirement
    pub fn handle_get_requirement(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("req_get_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Extract requirement ID from path
        let req_id = self.extract_requirement_id_from_path(&request.path())?;
        
        // Build CLI arguments
        let args = vec![req_id];
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "req", "view", args)?;
        
        Ok(result.to_http_response())
    }
    
    /// Handle POST /api/requirements - Create new requirement
    pub fn handle_create_requirement(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("req_create_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Parse JSON body into CLI arguments
        let body = request.get_body_as_string()
            .map_err(|e| QmsError::parse_error(&format!("Failed to parse request body: {}", e)))?;
        let args = self.parse_create_requirement_body(&body)?;
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "req", "add", args)?;
        
        // Set HTTP status to Created for successful requirement creation
        let mut response = result.to_http_response();
        if result.success {
            response.status = HttpStatus::Created;
        }
        
        Ok(response)
    }
    
    /// Handle PUT /api/requirements/{id} - Update requirement
    pub fn handle_update_requirement(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("req_update_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Extract requirement ID from path
        let req_id = self.extract_requirement_id_from_path(&request.path())?;
        
        // Parse JSON body into CLI arguments
        let body = request.get_body_as_string()
            .map_err(|e| QmsError::parse_error(&format!("Failed to parse request body: {}", e)))?;
        let mut args = self.parse_update_requirement_body(&body)?;
        args.insert(0, req_id); // Add requirement ID as first argument
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "req", "update", args)?;
        
        Ok(result.to_http_response())
    }
    
    /// Handle DELETE /api/requirements/{id} - Delete requirement
    pub fn handle_delete_requirement(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("req_delete_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Extract requirement ID from path
        let req_id = self.extract_requirement_id_from_path(&request.path())?;
        
        // Build CLI arguments
        let args = vec![req_id];
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "req", "remove", args)?;
        
        Ok(result.to_http_response())
    }
    
    /// Handle POST /api/requirements/{id}/trace - Create traceability link
    pub fn handle_trace_requirement(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("req_trace_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Extract requirement ID from path
        let req_id = self.extract_requirement_id_from_path(&request.path())?;
        
        // Parse JSON body for traceability parameters
        let body = request.get_body_as_string()
            .map_err(|e| QmsError::parse_error(&format!("Failed to parse request body: {}", e)))?;
        let mut args = self.parse_trace_requirement_body(&body)?;
        args.insert(0, req_id); // Add requirement ID as first argument
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "req", "trace", args)?;
        
        Ok(result.to_http_response())
    }
    
    /// Extract requirement ID from URL path
    fn extract_requirement_id_from_path(&self, path: &str) -> QmsResult<String> {
        // Expected path format: /api/requirements/{id} or /api/v1/requirements/{id}
        let parts: Vec<&str> = path.split('/').collect();
        
        // Find the requirements segment and get the next part
        for (i, part) in parts.iter().enumerate() {
            if *part == "requirements" && i + 1 < parts.len() {
                return Ok(parts[i + 1].to_string());
            }
        }
        
        Err(QmsError::validation_error("Requirement ID not found in path"))
    }
    
    /// Parse JSON body for requirement creation
    fn parse_create_requirement_body(&self, body: &str) -> QmsResult<Vec<String>> {
        let mut args = Vec::new();
        
        // Extract title
        if let Some(title) = self.extract_json_field(body, "title") {
            args.push("--title".to_string());
            args.push(title);
        }
        
        // Extract description
        if let Some(description) = self.extract_json_field(body, "description") {
            args.push("--description".to_string());
            args.push(description);
        }
        
        // Extract type
        if let Some(req_type) = self.extract_json_field(body, "type") {
            args.push("--type".to_string());
            args.push(req_type);
        }
        
        // Extract priority
        if let Some(priority) = self.extract_json_field(body, "priority") {
            args.push("--priority".to_string());
            args.push(priority);
        }
        
        // Extract category
        if let Some(category) = self.extract_json_field(body, "category") {
            args.push("--category".to_string());
            args.push(category);
        }
        
        Ok(args)
    }
    
    /// Parse JSON body for requirement update
    fn parse_update_requirement_body(&self, body: &str) -> QmsResult<Vec<String>> {
        // Similar to create, but for update operations
        self.parse_create_requirement_body(body)
    }
    
    /// Parse JSON body for requirement traceability
    fn parse_trace_requirement_body(&self, body: &str) -> QmsResult<Vec<String>> {
        let mut args = Vec::new();
        
        // Extract target ID (what this requirement traces to)
        if let Some(target_id) = self.extract_json_field(body, "target_id") {
            args.push("--target".to_string());
            args.push(target_id);
        }
        
        // Extract trace type
        if let Some(trace_type) = self.extract_json_field(body, "trace_type") {
            args.push("--type".to_string());
            args.push(trace_type);
        }
        
        // Extract relationship description
        if let Some(relationship) = self.extract_json_field(body, "relationship") {
            args.push("--relationship".to_string());
            args.push(relationship);
        }
        
        Ok(args)
    }
    
    /// Simple JSON field extraction (basic implementation)
    fn extract_json_field(&self, json: &str, field: &str) -> Option<String> {
        // Look for "field": "value" pattern
        let pattern = format!("\"{}\":", field);
        if let Some(start) = json.find(&pattern) {
            let after_colon = &json[start + pattern.len()..];
            if let Some(quote_start) = after_colon.find('"') {
                let value_start = quote_start + 1;
                if let Some(quote_end) = after_colon[value_start..].find('"') {
                    return Some(after_colon[value_start..value_start + quote_end].to_string());
                }
            }
        }
        None
    }
    
    /// Generate unique request ID
    fn generate_request_id(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{:x}", timestamp)
    }
}

/// Static handler functions for route registration
impl UnifiedRequirementsApiHandler {
    /// Static handler for listing requirements
    pub fn static_handle_list_requirements(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_list_requirements(request)
    }
    
    /// Static handler for getting a requirement
    pub fn static_handle_get_requirement(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_get_requirement(request)
    }
    
    /// Static handler for creating a requirement
    pub fn static_handle_create_requirement(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_create_requirement(request)
    }
    
    /// Static handler for updating a requirement
    pub fn static_handle_update_requirement(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_update_requirement(request)
    }
    
    /// Static handler for deleting a requirement
    pub fn static_handle_delete_requirement(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_delete_requirement(request)
    }
    
    /// Static handler for tracing a requirement
    pub fn static_handle_trace_requirement(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_trace_requirement(request)
    }
}

impl Default for UnifiedRequirementsApiHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Route registration helper for unified requirements API
pub struct UnifiedRequirementsRoutes;

impl UnifiedRequirementsRoutes {

}
