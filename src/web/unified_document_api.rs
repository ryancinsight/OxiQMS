// Unified Document API - Web routes that delegate to CLI command infrastructure
// Follows SOLID, CUPID, GRASP, ACID, KISS, DRY, SOC, and YAGNI principles

use crate::prelude::*;
use crate::web::{
    HttpRequest, HttpResponse, 
    WebCommandBridge, CommandArgumentParser,
    unified_auth_context::UnifiedAuthContext,
    command_bridge::ClientInfo
};
use crate::web::response::HttpStatus;
use std::collections::HashMap;

/// Unified Document API Handler - delegates to CLI command infrastructure
pub struct UnifiedDocumentApiHandler {
    command_bridge: WebCommandBridge,
}

impl UnifiedDocumentApiHandler {
    /// Create new unified document API handler
    pub fn new() -> Self {
        Self {
            command_bridge: WebCommandBridge::new(),
        }
    }
    
    /// Handle GET /api/documents - List documents
    pub fn handle_list_documents(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("doc_list_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Parse query parameters into CLI arguments
        let mut args = vec!["qms".to_string(), "doc".to_string(), "list".to_string()];
        
        // TODO: Extract query parameters for filtering
        // For now, we'll use the basic list command
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "doc", "list", args[3..].to_vec())?;
        
        Ok(result.to_http_response())
    }
    
    /// Handle GET /api/documents/{id} - Get specific document
    pub fn handle_get_document(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("doc_get_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Extract document ID from path
        let doc_id = self.extract_document_id_from_path(&request.path())?;
        
        // Build CLI arguments
        let args = vec![doc_id];
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "doc", "view", args)?;
        
        Ok(result.to_http_response())
    }
    
    /// Handle POST /api/documents - Create new document
    pub fn handle_create_document(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("doc_create_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Parse JSON body into CLI arguments
        let body = request.get_body_as_string()
            .map_err(|e| QmsError::parse_error(&format!("Failed to parse request body: {}", e)))?;
        let args = self.parse_create_document_body(&body)?;
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "doc", "add", args)?;
        
        // Set HTTP status to Created for successful document creation
        let mut response = result.to_http_response();
        if result.success {
            response.status = HttpStatus::Created;
        }
        
        Ok(response)
    }
    
    /// Handle PUT /api/documents/{id} - Update document
    pub fn handle_update_document(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("doc_update_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Extract document ID from path
        let doc_id = self.extract_document_id_from_path(&request.path())?;
        
        // Parse JSON body into CLI arguments
        let body = request.get_body_as_string()
            .map_err(|e| QmsError::parse_error(&format!("Failed to parse request body: {}", e)))?;
        let mut args = self.parse_update_document_body(&body)?;
        args.insert(0, doc_id); // Add document ID as first argument
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "doc", "update", args)?;
        
        Ok(result.to_http_response())
    }
    
    /// Handle DELETE /api/documents/{id} - Delete document
    pub fn handle_delete_document(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("doc_delete_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Extract document ID from path
        let doc_id = self.extract_document_id_from_path(&request.path())?;
        
        // Build CLI arguments
        let args = vec![doc_id];
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "doc", "remove", args)?;
        
        Ok(result.to_http_response())
    }
    
    /// Handle POST /api/documents/search - Search documents
    pub fn handle_search_documents(&self, request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Create authentication context
        let auth_context = UnifiedAuthContext::from_web_request(request)?;
        
        // Generate request ID and client info
        let request_id = format!("doc_search_{}", self.generate_request_id());
        let client_info = ClientInfo::from_request(request);
        
        // Create web command context
        let web_context = auth_context.to_web_command_context(request_id, client_info);
        
        // Parse JSON body into CLI arguments
        let body = request.get_body_as_string()
            .map_err(|e| QmsError::parse_error(&format!("Failed to parse request body: {}", e)))?;
        let args = self.parse_search_document_body(&body)?;
        
        // Execute command through bridge
        let result = self.command_bridge.execute_command(&web_context, "doc", "search", args)?;
        
        Ok(result.to_http_response())
    }
    
    /// Extract document ID from URL path
    fn extract_document_id_from_path(&self, path: &str) -> QmsResult<String> {
        // Expected path format: /api/documents/{id} or /api/v1/documents/{id}
        let parts: Vec<&str> = path.split('/').collect();
        
        // Find the documents segment and get the next part
        for (i, part) in parts.iter().enumerate() {
            if *part == "documents" && i + 1 < parts.len() {
                return Ok(parts[i + 1].to_string());
            }
        }
        
        Err(QmsError::validation_error("Document ID not found in path"))
    }
    
    /// Parse JSON body for document creation
    fn parse_create_document_body(&self, body: &str) -> QmsResult<Vec<String>> {
        // Simple JSON parsing - in a full implementation, use proper JSON parser
        let mut args = Vec::new();
        
        // Extract title
        if let Some(title) = self.extract_json_field(body, "title") {
            args.push("--title".to_string());
            args.push(title);
        }
        
        // Extract content
        if let Some(content) = self.extract_json_field(body, "content") {
            args.push("--content".to_string());
            args.push(content);
        }
        
        // Extract type
        if let Some(doc_type) = self.extract_json_field(body, "type") {
            args.push("--type".to_string());
            args.push(doc_type);
        }
        
        Ok(args)
    }
    
    /// Parse JSON body for document update
    fn parse_update_document_body(&self, body: &str) -> QmsResult<Vec<String>> {
        // Similar to create, but for update operations
        let mut args = Vec::new();
        
        // Extract title
        if let Some(title) = self.extract_json_field(body, "title") {
            args.push("--title".to_string());
            args.push(title);
        }
        
        // Extract content
        if let Some(content) = self.extract_json_field(body, "content") {
            args.push("--content".to_string());
            args.push(content);
        }
        
        Ok(args)
    }
    
    /// Parse JSON body for document search
    fn parse_search_document_body(&self, body: &str) -> QmsResult<Vec<String>> {
        let mut args = Vec::new();
        
        // Extract search query
        if let Some(query) = self.extract_json_field(body, "query") {
            args.push("--query".to_string());
            args.push(query);
        }
        
        // Extract type filter
        if let Some(doc_type) = self.extract_json_field(body, "type") {
            args.push("--type".to_string());
            args.push(doc_type);
        }
        
        // Extract author filter
        if let Some(author) = self.extract_json_field(body, "author") {
            args.push("--author".to_string());
            args.push(author);
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

impl Default for UnifiedDocumentApiHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Static handler functions for route registration
impl UnifiedDocumentApiHandler {
    /// Static handler for listing documents
    pub fn static_handle_list_documents(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_list_documents(request)
    }

    /// Static handler for getting a document
    pub fn static_handle_get_document(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_get_document(request)
    }

    /// Static handler for creating a document
    pub fn static_handle_create_document(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_create_document(request)
    }

    /// Static handler for updating a document
    pub fn static_handle_update_document(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_update_document(request)
    }

    /// Static handler for deleting a document
    pub fn static_handle_delete_document(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_delete_document(request)
    }

    /// Static handler for searching documents
    pub fn static_handle_search_documents(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let handler = Self::new();
        handler.handle_search_documents(request)
    }
}

/// Route registration helper for unified document API
pub struct UnifiedDocumentRoutes;

impl UnifiedDocumentRoutes {

}
