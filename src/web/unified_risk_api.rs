// Unified Risk API - Web routes that delegate to CLI command infrastructure
// Follows SOLID, CUPID, GRASP, ACID, KISS, DRY, SOC, and YAGNI principles

use crate::prelude::*;
use crate::web::{
    HttpRequest, HttpResponse, 
    WebCommandBridge,
    unified_auth_context::UnifiedAuthContext,
    command_bridge::ClientInfo
};
use crate::web::response::HttpStatus;

/// Unified Risk API Handler - delegates to CLI command infrastructure
pub struct UnifiedRiskApiHandler {
    command_bridge: WebCommandBridge,
}

impl UnifiedRiskApiHandler {
    /// Create new unified risk API handler
    pub fn new() -> Self {
        Self {
            command_bridge: WebCommandBridge::new(),
        }
    }
    
    /// Handle GET /api/risks - List risks
    pub fn handle_list_risks(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("risk_list_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Build CLI arguments
        let args = vec![];
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "risk", "list", args)?;
        
        Ok(result.to_http_response())
    }
    
    /// Handle GET /api/risks/{id} - Get specific risk
    pub fn handle_get_risk(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("risk_get_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Extract risk ID from path
        let risk_id = self.extract_risk_id_from_path(&request.path())?;
        
        // Build CLI arguments
        let args = vec![risk_id];
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "risk", "view", args)?;
        
        Ok(result.to_http_response())
    }
    
    /// Handle POST /api/risks - Create new risk
    pub fn handle_create_risk(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("risk_create_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Parse JSON body into CLI arguments
        let body = request.get_body_as_string()
            .map_err(|e| QmsError::parse_error(&format!("Failed to parse request body: {}", e)))?;
        let args = self.parse_create_risk_body(&body)?;
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "risk", "add", args)?;
        
        // Set HTTP status to Created for successful risk creation
        let mut response = result.to_http_response();
        if result.success {
            response.status = HttpStatus::Created;
        }
        
        Ok(response)
    }
    
    /// Handle PUT /api/risks/{id} - Update risk
    pub fn handle_update_risk(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("risk_update_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Extract risk ID from path
        let risk_id = self.extract_risk_id_from_path(&request.path())?;
        
        // Parse JSON body into CLI arguments
        let body = request.get_body_as_string()
            .map_err(|e| QmsError::parse_error(&format!("Failed to parse request body: {}", e)))?;
        let mut args = self.parse_update_risk_body(&body)?;
        args.insert(0, risk_id); // Add risk ID as first argument
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "risk", "update", args)?;
        
        Ok(result.to_http_response())
    }
    
    /// Handle DELETE /api/risks/{id} - Delete risk
    pub fn handle_delete_risk(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("risk_delete_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Extract risk ID from path
        let risk_id = self.extract_risk_id_from_path(&request.path())?;
        
        // Build CLI arguments
        let args = vec![risk_id];
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "risk", "remove", args)?;
        
        Ok(result.to_http_response())
    }
    
    /// Handle POST /api/risks/{id}/assess - Assess risk
    pub fn handle_assess_risk(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("risk_assess_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Extract risk ID from path
        let risk_id = self.extract_risk_id_from_path(&request.path())?;
        
        // Parse JSON body for assessment parameters
        let body = request.get_body_as_string()
            .map_err(|e| QmsError::parse_error(&format!("Failed to parse request body: {}", e)))?;
        let mut args = self.parse_assess_risk_body(&body)?;
        args.insert(0, risk_id); // Add risk ID as first argument
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "risk", "assess", args)?;
        
        Ok(result.to_http_response())
    }
    
    /// Extract risk ID from URL path
    fn extract_risk_id_from_path(&self, path: &str) -> QmsResult<String> {
        // Expected path format: /api/risks/{id} or /api/v1/risks/{id}
        let parts: Vec<&str> = path.split('/').collect();
        
        // Find the risks segment and get the next part
        for (i, part) in parts.iter().enumerate() {
            if *part == "risks" && i + 1 < parts.len() {
                return Ok(parts[i + 1].to_string());
            }
        }
        
        Err(QmsError::validation_error("Risk ID not found in path"))
    }
    
    /// Parse JSON body for risk creation
    fn parse_create_risk_body(&self, body: &str) -> QmsResult<Vec<String>> {
        let mut args = Vec::new();
        
        // Extract title/name
        if let Some(title) = self.extract_json_field(body, "title") {
            args.push("--title".to_string());
            args.push(title);
        } else if let Some(name) = self.extract_json_field(body, "name") {
            args.push("--title".to_string());
            args.push(name);
        }
        
        // Extract description
        if let Some(description) = self.extract_json_field(body, "description") {
            args.push("--description".to_string());
            args.push(description);
        }
        
        // Extract severity
        if let Some(severity) = self.extract_json_field(body, "severity") {
            args.push("--severity".to_string());
            args.push(severity);
        }
        
        // Extract probability
        if let Some(probability) = self.extract_json_field(body, "probability") {
            args.push("--probability".to_string());
            args.push(probability);
        }
        
        Ok(args)
    }
    
    /// Parse JSON body for risk update
    fn parse_update_risk_body(&self, body: &str) -> QmsResult<Vec<String>> {
        // Similar to create, but for update operations
        self.parse_create_risk_body(body)
    }
    
    /// Parse JSON body for risk assessment
    fn parse_assess_risk_body(&self, body: &str) -> QmsResult<Vec<String>> {
        let mut args = Vec::new();
        
        // Extract severity score
        if let Some(severity) = self.extract_json_field(body, "severity") {
            args.push("--severity".to_string());
            args.push(severity);
        }
        
        // Extract probability score
        if let Some(probability) = self.extract_json_field(body, "probability") {
            args.push("--probability".to_string());
            args.push(probability);
        }
        
        // Extract detectability score
        if let Some(detectability) = self.extract_json_field(body, "detectability") {
            args.push("--detectability".to_string());
            args.push(detectability);
        }
        
        // Extract mitigation measures
        if let Some(mitigation) = self.extract_json_field(body, "mitigation") {
            args.push("--mitigation".to_string());
            args.push(mitigation);
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
impl UnifiedRiskApiHandler {
    /// Static handler for listing risks
    pub fn static_handle_list_risks(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_list_risks(request)
    }
    
    /// Static handler for getting a risk
    pub fn static_handle_get_risk(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_get_risk(request)
    }
    
    /// Static handler for creating a risk
    pub fn static_handle_create_risk(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_create_risk(request)
    }
    
    /// Static handler for updating a risk
    pub fn static_handle_update_risk(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_update_risk(request)
    }
    
    /// Static handler for deleting a risk
    pub fn static_handle_delete_risk(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_delete_risk(request)
    }
    
    /// Static handler for assessing a risk
    pub fn static_handle_assess_risk(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_assess_risk(request)
    }
}

impl Default for UnifiedRiskApiHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Route registration helper for unified risk API
pub struct UnifiedRiskRoutes;

impl UnifiedRiskRoutes {

}
