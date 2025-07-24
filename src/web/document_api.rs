// QMS Document Management API - REST endpoints for document CRUD operations
// Medical Device Quality Management System - FDA 21 CFR Part 820 Compliant
// Task 7.2.2 - CRUD operations for documents with approval workflow

use std::collections::HashMap;
use crate::web::request::HttpRequest;
use crate::web::response::{HttpResponse, HttpStatus};

use crate::modules::document_control::service::DocumentService;
use crate::modules::document_control::document::{Document, DocumentType, DocumentStatus};
use crate::modules::document_control::version::VersionChangeType;
use crate::error::{QmsError, QmsResult};
use crate::json_utils::JsonValue;

/// Document API handlers for RESTful document management
pub struct DocumentApiHandler {
    service: DocumentService,
}

impl DocumentApiHandler {
    /// Create new document API handler
    pub const fn new(project_path: std::path::PathBuf) -> Self {
        Self {
            service: DocumentService::new(project_path),
        }
    }

    /// Helper method to extract document ID from path (DRY principle)
    fn extract_document_id(uri: &str) -> QmsResult<&str> {
        let path_parts: Vec<&str> = uri.split('/').collect();
        path_parts.get(4)
            .ok_or_else(|| QmsError::validation_error("Document ID is required")).copied()
    }



    /// Handle GET /api/v1/documents - List documents with filtering
    fn handle_list_documents(request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Extract query parameters for filtering
        let type_filter = request.query_params.get("type");
        let status_filter = request.query_params.get("status");
        let limit = request.query_params.get("limit")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(50);
        let offset = request.query_params.get("offset")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);

        // Get project path from router (simplified for now)
        let project_path = std::env::current_dir()
            .map_err(QmsError::Io)?
            .join("qms_projects")
            .join("default");

        let service = DocumentService::new(project_path);
        
        // Get documents list
        let documents = service.list_documents()?;
        
        // Apply filters
        let filtered_docs: Vec<_> = documents.into_iter()
            .filter(|doc| {
                if let Some(type_filter) = type_filter {
                    if doc.doc_type != *type_filter {
                        return false;
                    }
                }
                if let Some(status_filter) = status_filter {
                    if doc.status != *status_filter {
                        return false;
                    }
                }
                true
            })
            .skip(offset)
            .take(limit)
            .collect();

        // Convert to JSON response
        let mut response_data = HashMap::new();
        response_data.insert("documents".to_string(), Self::serialize_document_list(&filtered_docs)?);
        response_data.insert("total".to_string(), JsonValue::Number(filtered_docs.len() as f64));
        response_data.insert("offset".to_string(), JsonValue::Number(offset as f64));
        response_data.insert("limit".to_string(), JsonValue::Number(limit as f64));

        let json_response = Self::create_json_response(response_data)?;
        Ok(json_response)
    }

    /// Handle GET /api/v1/documents/{id} - Get specific document
    fn handle_get_document(request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Extract document ID from path parameters
        let document_id = Self::extract_document_id(&request.uri)?;


        // Get project path
        let project_path = std::env::current_dir()
            .map_err(QmsError::Io)?
            .join("qms_projects")
            .join("default");

        let service = DocumentService::new(project_path);
        
        // Get document
        let document = service.read_document(document_id)?;
        
        // Convert to JSON response
        let response_data = Self::serialize_document(&document)?;
        let json_response = Self::create_json_response(response_data)?;
        Ok(json_response)
    }

    /// Handle POST /api/v1/documents - Create new document
    fn handle_create_document(request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Parse request body as JSON
        let body_str = String::from_utf8(request.body.clone())
            .map_err(|e| QmsError::Parse(format!("Invalid UTF-8 in request body: {e}")))?;
        
        let request_data = Self::parse_json_request(&body_str)?;
        
        // Extract document fields
        let title = Self::get_string_field(&request_data, "title")?;
        let content = Self::get_string_field(&request_data, "content")?;
        let doc_type_str = Self::get_string_field(&request_data, "type")?;
        let doc_type = DocumentType::from_str(&doc_type_str);
        
        // Get created_by from authentication context (simplified for now)
        let created_by = request.headers.get("X-User-ID")
            .unwrap_or(&"system".to_string())
            .clone();

        // Get project path
        let project_path = std::env::current_dir()
            .map_err(QmsError::Io)?
            .join("qms_projects")
            .join("default");

        let service = DocumentService::new(project_path);
        
        // Create document
        let document = service.create_document(title, content, doc_type, created_by)?;
        
        // Convert to JSON response
        let response_data = Self::serialize_document(&document)?;
        let mut response = Self::create_json_response(response_data)?;
        response.status = HttpStatus::Created;
        Ok(response)
    }

    /// Handle PUT /api/v1/documents/{id} - Update document
    fn handle_update_document(request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Extract document ID from path parameters
        let document_id = Self::extract_document_id(&request.uri)?;

        // Parse request body as JSON
        let body_str = String::from_utf8(request.body.clone())
            .map_err(|e| QmsError::Parse(format!("Invalid UTF-8 in request body: {e}")))?;
        
        let request_data = Self::parse_json_request(&body_str)?;
        
        // Get updated_by from authentication context
        let updated_by = request.headers.get("X-User-ID")
            .unwrap_or(&"system".to_string())
            .clone();

        // Get project path
        let project_path = std::env::current_dir()
            .map_err(QmsError::Io)?
            .join("qms_projects")
            .join("default");

        let service = DocumentService::new(project_path);
        
        // Get existing document
        let mut document = service.read_document(document_id)?;
        
        // Update fields if provided
        if let Some(title) = Self::get_optional_string_field(&request_data, "title") {
            document.title = title;
        }
        if let Some(content) = Self::get_optional_string_field(&request_data, "content") {
            document.content = content;
        }
        if let Some(doc_type_str) = Self::get_optional_string_field(&request_data, "type") {
            document.doc_type = DocumentType::from_str(&doc_type_str);
        }
        
        // Update document
        let updated_document = service.update_document(
            &document.id,
            Some(document.title),
            Some(document.content),
            Some(VersionChangeType::Minor),
            Some("Updated via API".to_string()),
            updated_by
        )?;
        
        // Convert to JSON response
        let response_data = Self::serialize_document(&updated_document)?;
        let json_response = Self::create_json_response(response_data)?;
        Ok(json_response)
    }

    /// Handle DELETE /api/v1/documents/{id} - Delete document
    fn handle_delete_document(request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Extract document ID from path parameters
        let document_id = Self::extract_document_id(&request.uri)?;

        // Get deleted_by from authentication context
        let deleted_by = request.headers.get("X-User-ID")
            .unwrap_or(&"system".to_string())
            .clone();

        // Get project path
        let project_path = std::env::current_dir()
            .map_err(QmsError::Io)?
            .join("qms_projects")
            .join("default");

        let service = DocumentService::new(project_path);
        
        // Delete document
        service.delete_document(document_id, deleted_by)?;
        
        // Return success response
        let mut response_data = HashMap::new();
        response_data.insert("message".to_string(), JsonValue::String("Document deleted successfully".to_string()));
        response_data.insert("document_id".to_string(), JsonValue::String(document_id.to_string()));
        
        let json_response = Self::create_json_response(response_data)?;
        Ok(json_response)
    }

    /// Handle POST /api/v1/documents/{id}/approve - Approve document
    fn handle_approve_document(request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Extract document ID from path parameters
        let document_id = Self::extract_document_id(&request.uri)?;

        // Get approved_by from authentication context
        let approved_by = request.headers.get("X-User-ID")
            .unwrap_or(&"system".to_string())
            .clone();

        // Get project path
        let project_path = std::env::current_dir()
            .map_err(QmsError::Io)?
            .join("qms_projects")
            .join("default");

        let service = DocumentService::new(project_path);
        
        // Get and update document status
        let mut document = service.read_document(document_id)?;
        document.status = DocumentStatus::Approved;
        document.approved_by = Some(approved_by.clone());
        
        let updated_document = service.update_document(
            &document.id,
            Some(document.title),
            Some(document.content),
            Some(VersionChangeType::Minor),
            Some("Approved via API".to_string()),
            approved_by
        )?;
        
        // Convert to JSON response
        let response_data = Self::serialize_document(&updated_document)?;
        let json_response = Self::create_json_response(response_data)?;
        Ok(json_response)
    }

    /// Handle POST /api/v1/documents/{id}/checkout - Checkout document
    fn handle_checkout_document(request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Extract document ID from path parameters
        let document_id = Self::extract_document_id(&request.uri)?;

        // Get user_id from authentication context
        let user_id = request.headers.get("X-User-ID")
            .unwrap_or(&"system".to_string())
            .clone();

        // Parse request body for optional reason
        let body_str = String::from_utf8(request.body.clone()).unwrap_or_default();
        let request_data = if body_str.is_empty() {
            HashMap::new()
        } else {
            Self::parse_json_request(&body_str)?
        };
        let reason = Self::get_optional_string_field(&request_data, "reason");

        // Get project path
        let project_path = std::env::current_dir()
            .map_err(QmsError::Io)?
            .join("qms_projects")
            .join("default");

        let service = DocumentService::new(project_path);
        
        // Checkout document
        let lock = service.checkout_document(document_id, &user_id, reason)?;
        
        // Convert to JSON response
        let mut response_data = HashMap::new();
        response_data.insert("message".to_string(), JsonValue::String("Document checked out successfully".to_string()));
        response_data.insert("document_id".to_string(), JsonValue::String(lock.document_id));
        response_data.insert("locked_by".to_string(), JsonValue::String(lock.user_id));
        response_data.insert("locked_at".to_string(), JsonValue::String(lock.locked_at));
        if let Some(reason) = lock.lock_reason {
            response_data.insert("reason".to_string(), JsonValue::String(reason));
        }
        
        let json_response = Self::create_json_response(response_data)?;
        Ok(json_response)
    }

    /// Handle POST /api/v1/documents/{id}/checkin - Checkin document
    fn handle_checkin_document(request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Extract document ID from path parameters
        let document_id = Self::extract_document_id(&request.uri)?;

        // Get user_id from authentication context
        let user_id = request.headers.get("X-User-ID")
            .unwrap_or(&"system".to_string())
            .clone();

        // Parse request body for optional comment
        let body_str = String::from_utf8(request.body.clone()).unwrap_or_default();
        let request_data = if body_str.is_empty() {
            HashMap::new()
        } else {
            Self::parse_json_request(&body_str)?
        };
        let comment = Self::get_optional_string_field(&request_data, "comment");

        // Get project path
        let project_path = std::env::current_dir()
            .map_err(QmsError::Io)?
            .join("qms_projects")
            .join("default");

        let service = DocumentService::new(project_path);
        
        // Checkin document
        service.checkin_document(document_id, &user_id, None, comment.as_deref())?;
        
        // Convert to JSON response
        let mut response_data = HashMap::new();
        response_data.insert("message".to_string(), JsonValue::String("Document checked in successfully".to_string()));
        response_data.insert("document_id".to_string(), JsonValue::String(document_id.to_string()));
        
        let json_response = Self::create_json_response(response_data)?;
        Ok(json_response)
    }

    /// Helper functions for JSON serialization

    fn serialize_document_list(documents: &[crate::modules::document_control::service::DocumentIndexEntry]) -> QmsResult<JsonValue> {
        let mut doc_array = Vec::new();
        for doc in documents {
            let mut doc_obj = HashMap::new();
            doc_obj.insert("id".to_string(), JsonValue::String(doc.id.clone()));
            doc_obj.insert("title".to_string(), JsonValue::String(doc.title.clone()));
            doc_obj.insert("type".to_string(), JsonValue::String(doc.doc_type.clone()));
            doc_obj.insert("version".to_string(), JsonValue::String(doc.version.clone()));
            doc_obj.insert("status".to_string(), JsonValue::String(doc.status.clone()));
            doc_obj.insert("created_at".to_string(), JsonValue::String(doc.created_at.clone()));
            doc_obj.insert("updated_at".to_string(), JsonValue::String(doc.updated_at.clone()));
            doc_obj.insert("author".to_string(), JsonValue::String(doc.author.clone()));
            doc_array.push(JsonValue::Object(doc_obj));
        }
        Ok(JsonValue::Array(doc_array))
    }

    fn serialize_document(document: &Document) -> QmsResult<HashMap<String, JsonValue>> {
        let mut response_data = HashMap::new();
        response_data.insert("id".to_string(), JsonValue::String(document.id.clone()));
        response_data.insert("project_id".to_string(), JsonValue::String(document.project_id.clone()));
        response_data.insert("title".to_string(), JsonValue::String(document.title.clone()));
        response_data.insert("content".to_string(), JsonValue::String(document.content.clone()));
        response_data.insert("type".to_string(), JsonValue::String(document.doc_type.to_string()));
        response_data.insert("version".to_string(), JsonValue::String(document.version.clone()));
        response_data.insert("status".to_string(), JsonValue::String(document.status.to_string()));
        response_data.insert("created_at".to_string(), JsonValue::String(document.created_at.clone()));
        response_data.insert("updated_at".to_string(), JsonValue::String(document.updated_at.clone()));
        response_data.insert("created_by".to_string(), JsonValue::String(document.created_by.clone()));
        if let Some(approved_by) = &document.approved_by {
            response_data.insert("approved_by".to_string(), JsonValue::String(approved_by.clone()));
        }
        response_data.insert("file_path".to_string(), JsonValue::String(document.file_path.clone()));
        response_data.insert("checksum".to_string(), JsonValue::String(document.checksum.clone()));
        response_data.insert("locked".to_string(), JsonValue::boolean(document.locked));
        if let Some(locked_by) = &document.locked_by {
            response_data.insert("locked_by".to_string(), JsonValue::String(locked_by.clone()));
        }
        if let Some(locked_at) = &document.locked_at {
            response_data.insert("locked_at".to_string(), JsonValue::String(locked_at.clone()));
        }
        
        // Serialize tags
        let tags: Vec<JsonValue> = document.tags.iter()
            .map(|tag| JsonValue::String(tag.clone()))
            .collect();
        response_data.insert("tags".to_string(), JsonValue::Array(tags));
        
        Ok(response_data)
    }

    fn create_json_response(data: HashMap<String, JsonValue>) -> QmsResult<HttpResponse> {
        let json_value = JsonValue::Object(data);
        let json_string = json_value.to_string();
        
        let mut response = HttpResponse::new(HttpStatus::Ok);
        response.set_content_type("application/json");
        response.set_body(json_string.as_bytes().to_vec());
        Ok(response)
    }

    fn parse_json_request(body: &str) -> QmsResult<HashMap<String, JsonValue>> {
        let json_value = JsonValue::parse_from_str(body)
            .map_err(|e| QmsError::Parse(format!("Invalid JSON: {e}")))?;
        
        match json_value {
            JsonValue::Object(obj) => Ok(obj),
            _ => Err(QmsError::Parse("Expected JSON object".to_string())),
        }
    }

    fn get_string_field(data: &HashMap<String, JsonValue>, field: &str) -> QmsResult<String> {
        match data.get(field) {
            Some(JsonValue::String(s)) => Ok(s.clone()),
            Some(_) => Err(QmsError::Parse(format!("Field '{field}' must be a string"))),
            None => Err(QmsError::Parse(format!("Required field '{field}' is missing"))),
        }
    }

    fn get_optional_string_field(data: &HashMap<String, JsonValue>, field: &str) -> Option<String> {
        match data.get(field) {
            Some(JsonValue::String(s)) => Some(s.clone()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_request(method: &str, uri: &str, body: &str) -> HttpRequest {
        HttpRequest {
            method: method.to_string(),
            uri: uri.to_string(),
            version: "HTTP/1.1".to_string(),
            headers: {
                let mut headers = HashMap::new();
                headers.insert("Content-Type".to_string(), "application/json".to_string());
                headers.insert("X-User-ID".to_string(), "test-user".to_string());
                headers
            },
            body: body.as_bytes().to_vec(),
            query_params: HashMap::new(),
            timestamp: 0,
        }
    }

    #[test]
    fn test_json_serialization() {
        let mut data = HashMap::new();
        data.insert("message".to_string(), JsonValue::String("test".to_string()));
        data.insert("success".to_string(), JsonValue::boolean(true));
        
        let response = DocumentApiHandler::create_json_response(data);
        assert!(response.is_ok());
        
        let http_response = response.unwrap();
        assert_eq!(http_response.status.code(), 200);
        assert!(http_response.headers.get("Content-Type").unwrap().contains("application/json"));
    }

    #[test]
    fn test_json_parsing() {
        let json_body = r#"{"title": "Test Document", "content": "Test content", "type": "SRS"}"#;
        let parsed = DocumentApiHandler::parse_json_request(json_body);
        assert!(parsed.is_ok());
        
        let data = parsed.unwrap();
        assert_eq!(DocumentApiHandler::get_string_field(&data, "title").unwrap(), "Test Document");
        assert_eq!(DocumentApiHandler::get_string_field(&data, "type").unwrap(), "SRS");
    }

    #[test]
    fn test_field_extraction() {
        let mut data = HashMap::new();
        data.insert("title".to_string(), JsonValue::String("Test".to_string()));
        data.insert("optional".to_string(), JsonValue::String("Value".to_string()));
        
        assert_eq!(DocumentApiHandler::get_string_field(&data, "title").unwrap(), "Test");
        assert_eq!(DocumentApiHandler::get_optional_string_field(&data, "optional"), Some("Value".to_string()));
        assert_eq!(DocumentApiHandler::get_optional_string_field(&data, "missing"), None);
        assert!(DocumentApiHandler::get_string_field(&data, "missing").is_err());
    }

    #[test]
    fn test_document_type_conversion() {
        assert_eq!(DocumentType::from_str("SRS").to_string(), "SRS");
        assert_eq!(DocumentType::from_str("SDD").to_string(), "SDD");
        assert_eq!(DocumentType::from_str("Custom").to_string(), "Custom");
    }

    #[test]
    fn test_request_creation() {
        let request = create_test_request("POST", "/api/v1/documents", r#"{"title": "Test"}"#);
        assert_eq!(request.method, "POST");
        assert_eq!(request.uri, "/api/v1/documents");
        assert_eq!(request.headers.get("X-User-ID").unwrap(), "test-user");
    }
}
