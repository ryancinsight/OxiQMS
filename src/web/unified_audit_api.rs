// Unified Audit API - Web routes that delegate to CLI command infrastructure
// Follows SOLID, CUPID, GRASP, ACID, KISS, DRY, SOC, and YAGNI principles

use crate::prelude::*;
use crate::web::{
    HttpRequest, HttpResponse, 
    WebCommandBridge,
    unified_auth_context::UnifiedAuthContext,
    command_bridge::ClientInfo
};
use crate::web::response::HttpStatus;

/// Unified Audit API Handler - delegates to CLI command infrastructure
pub struct UnifiedAuditApiHandler {
    command_bridge: WebCommandBridge,
}

impl UnifiedAuditApiHandler {
    /// Create new unified audit API handler
    pub fn new() -> Self {
        Self {
            command_bridge: WebCommandBridge::new(),
        }
    }
    
    /// Handle GET /api/audit/logs - List audit logs
    pub fn handle_list_audit_logs(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("audit_list_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Build CLI arguments
        let args = vec![];
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "audit", "list", args)?;
        
        Ok(result.to_http_response())
    }
    
    /// Handle GET /api/audit/logs/{id} - Get specific audit log entry
    pub fn handle_get_audit_log(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("audit_get_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Extract audit log ID from path
        let log_id = self.extract_audit_id_from_path(&request.path())?;
        
        // Build CLI arguments
        let args = vec![log_id];
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "audit", "view", args)?;
        
        Ok(result.to_http_response())
    }
    
    /// Handle POST /api/audit/search - Search audit logs
    pub fn handle_search_audit_logs(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("audit_search_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Parse JSON body into CLI arguments
        let body = request.get_body_as_string()
            .map_err(|e| QmsError::parse_error(&format!("Failed to parse request body: {}", e)))?;
        let args = self.parse_search_audit_body(&body)?;
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "audit", "search", args)?;
        
        Ok(result.to_http_response())
    }
    
    /// Handle POST /api/audit/export - Export audit logs
    pub fn handle_export_audit_logs(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("audit_export_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Parse JSON body for export parameters
        let body = request.get_body_as_string()
            .map_err(|e| QmsError::parse_error(&format!("Failed to parse request body: {}", e)))?;
        let args = self.parse_export_audit_body(&body)?;
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "audit", "export", args)?;
        
        Ok(result.to_http_response())
    }
    
    /// Handle GET /api/audit/integrity - Check audit log integrity
    pub fn handle_check_audit_integrity(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("audit_integrity_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Build CLI arguments
        let args = vec![];
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "audit", "verify", args)?;
        
        Ok(result.to_http_response())
    }
    
    /// Handle GET /api/audit/stats - Get audit statistics
    pub fn handle_get_audit_stats(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("audit_stats_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Build CLI arguments
        let args = vec![];
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "audit", "stats", args)?;
        
        Ok(result.to_http_response())
    }
    
    /// Extract audit log ID from URL path
    fn extract_audit_id_from_path(&self, path: &str) -> QmsResult<String> {
        // Expected path format: /api/audit/logs/{id}
        let parts: Vec<&str> = path.split('/').collect();
        
        // Find the logs segment and get the next part
        for (i, part) in parts.iter().enumerate() {
            if *part == "logs" && i + 1 < parts.len() {
                return Ok(parts[i + 1].to_string());
            }
        }
        
        Err(QmsError::validation_error("Audit log ID not found in path"))
    }
    
    /// Parse JSON body for audit log search
    fn parse_search_audit_body(&self, body: &str) -> QmsResult<Vec<String>> {
        let mut args = Vec::new();
        
        // Extract search query
        if let Some(query) = self.extract_json_field(body, "query") {
            args.push("--query".to_string());
            args.push(query);
        }
        
        // Extract user filter
        if let Some(user) = self.extract_json_field(body, "user") {
            args.push("--user".to_string());
            args.push(user);
        }
        
        // Extract action filter
        if let Some(action) = self.extract_json_field(body, "action") {
            args.push("--action".to_string());
            args.push(action);
        }
        
        // Extract date range
        if let Some(start_date) = self.extract_json_field(body, "start_date") {
            args.push("--start".to_string());
            args.push(start_date);
        }
        
        if let Some(end_date) = self.extract_json_field(body, "end_date") {
            args.push("--end".to_string());
            args.push(end_date);
        }
        
        // Extract limit
        if let Some(limit) = self.extract_json_field(body, "limit") {
            args.push("--limit".to_string());
            args.push(limit);
        }
        
        Ok(args)
    }
    
    /// Parse JSON body for audit log export
    fn parse_export_audit_body(&self, body: &str) -> QmsResult<Vec<String>> {
        let mut args = Vec::new();
        
        // Extract format
        if let Some(format) = self.extract_json_field(body, "format") {
            args.push("--format".to_string());
            args.push(format);
        }
        
        // Extract output path
        if let Some(output) = self.extract_json_field(body, "output") {
            args.push("--output".to_string());
            args.push(output);
        }
        
        // Extract date range
        if let Some(start_date) = self.extract_json_field(body, "start_date") {
            args.push("--start".to_string());
            args.push(start_date);
        }
        
        if let Some(end_date) = self.extract_json_field(body, "end_date") {
            args.push("--end".to_string());
            args.push(end_date);
        }
        
        // Extract filters
        if let Some(user) = self.extract_json_field(body, "user") {
            args.push("--user".to_string());
            args.push(user);
        }
        
        if let Some(action) = self.extract_json_field(body, "action") {
            args.push("--action".to_string());
            args.push(action);
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
impl UnifiedAuditApiHandler {
    /// Static handler for listing audit logs
    pub fn static_handle_list_audit_logs(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_list_audit_logs(request)
    }
    
    /// Static handler for getting an audit log
    pub fn static_handle_get_audit_log(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_get_audit_log(request)
    }
    
    /// Static handler for searching audit logs
    pub fn static_handle_search_audit_logs(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_search_audit_logs(request)
    }
    
    /// Static handler for exporting audit logs
    pub fn static_handle_export_audit_logs(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_export_audit_logs(request)
    }
    
    /// Static handler for checking audit integrity
    pub fn static_handle_check_audit_integrity(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_check_audit_integrity(request)
    }
    
    /// Static handler for getting audit statistics
    pub fn static_handle_get_audit_stats(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_get_audit_stats(request)
    }
}

impl Default for UnifiedAuditApiHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Route registration helper for unified audit API
pub struct UnifiedAuditRoutes;

impl UnifiedAuditRoutes {

}
