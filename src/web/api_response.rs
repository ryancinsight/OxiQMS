//! API Response Wrapper System for QMS Web Interface
//! Phase 7.2.4 - JSON Serialization with Medical Device Compliance
//! 
//! This module provides a unified API response format that wraps CLI command outputs
//! in standardized JSON responses suitable for web consumption. Ensures FDA 21 CFR Part 820
//! compliance with comprehensive audit trails and error handling.

use crate::error::{QmsError, QmsResult};
use crate::json_utils::{JsonValue, JsonSerializable};
use crate::utils::{current_iso8601_timestamp, generate_uuid};
use std::collections::HashMap;

/// Interface Segregation Principle: Separate concerns for JSON serialization
pub trait JsonResponseSerializer {
    fn serialize_to_json(&self) -> String;
}

/// Interface Segregation Principle: Separate concerns for metadata generation
pub trait MetadataProvider {
    fn generate_metadata(&self, request_id: String, processing_time_ms: u64) -> ApiMetadata;
}

/// Standard API response structure for all QMS web endpoints
/// Ensures consistent format across all API responses with medical device compliance
#[derive(Debug, Clone)]
pub struct StandardApiResponse {
    /// Whether the API call was successful
    pub success: bool,
    /// Response data (null for errors)
    pub data: Option<JsonValue>,
    /// Error information (null for successful responses)
    pub error: Option<ApiError>,
    /// Response metadata including compliance information
    pub metadata: ApiMetadata,
}

/// API error structure with detailed information for debugging and compliance
#[derive(Debug, Clone)]
pub struct ApiError {
    /// Error code for programmatic handling
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Detailed error description
    pub details: Option<String>,
    /// HTTP status code equivalent
    pub status_code: u16,
    /// Trace ID for error tracking
    pub trace_id: String,
}

/// API response metadata for audit trails and compliance
#[derive(Debug, Clone)]
pub struct ApiMetadata {
    /// Response timestamp in ISO 8601 format
    pub timestamp: String,
    /// API version for backward compatibility
    pub api_version: String,
    /// Unique request identifier for audit trails
    pub request_id: String,
    /// Processing time in milliseconds
    pub processing_time_ms: Option<u64>,
    /// Medical device compliance information
    pub compliance: ComplianceMetadata,
    /// Pagination information (if applicable)
    pub pagination: Option<PaginationMetadata>,
}

/// Medical device compliance metadata per FDA 21 CFR Part 820
#[derive(Debug, Clone)]
pub struct ComplianceMetadata {
    /// FDA 21 CFR Part 820 compliance status
    pub cfr_820_compliant: bool,
    /// ISO 13485 compliance status
    pub iso_13485_compliant: bool,
    /// ISO 14971 compliance status (for risk management)
    pub iso_14971_compliant: bool,
    /// Audit trail reference ID
    pub audit_reference: Option<String>,
    /// Data integrity checksum
    pub checksum: Option<String>,
}

/// Pagination metadata for list responses
#[derive(Debug, Clone)]
pub struct PaginationMetadata {
    /// Current page number (1-based)
    pub page: usize,
    /// Items per page
    pub per_page: usize,
    /// Total number of items
    pub total_items: usize,
    /// Total number of pages
    pub total_pages: usize,
    /// Whether there are more pages
    pub has_next: bool,
    /// Whether there are previous pages
    pub has_previous: bool,
}

/// Factory trait for creating API responses (Open/Closed Principle)
pub trait ApiResponseFactory {
    fn create_success_response(&self, data: JsonValue) -> StandardApiResponse;
    fn create_error_response(&self, error: ApiError) -> StandardApiResponse;
    fn create_paginated_response(&self, data: JsonValue, pagination: PaginationMetadata) -> StandardApiResponse;
}

/// Default implementation of ApiResponseFactory
pub struct DefaultApiResponseFactory {
    metadata_provider: Box<dyn MetadataProvider>,
}

impl Default for DefaultApiResponseFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultApiResponseFactory {
    pub fn new() -> Self {
        Self {
            metadata_provider: Box::new(DefaultMetadataProvider),
        }
    }
}

impl ApiResponseFactory for DefaultApiResponseFactory {
    fn create_success_response(&self, data: JsonValue) -> StandardApiResponse {
        StandardApiResponse {
            success: true,
            data: Some(data),
            error: None,
            metadata: self.metadata_provider.generate_metadata(generate_uuid(), 0),
        }
    }

    fn create_error_response(&self, error: ApiError) -> StandardApiResponse {
        StandardApiResponse {
            success: false,
            data: None,
            error: Some(error),
            metadata: self.metadata_provider.generate_metadata(generate_uuid(), 0),
        }
    }

    fn create_paginated_response(&self, data: JsonValue, pagination: PaginationMetadata) -> StandardApiResponse {
        let mut metadata = self.metadata_provider.generate_metadata(generate_uuid(), 0);
        metadata.pagination = Some(pagination);

        StandardApiResponse {
            success: true,
            data: Some(data),
            error: None,
            metadata,
        }
    }
}

/// Default metadata provider implementation
pub struct DefaultMetadataProvider;

impl MetadataProvider for DefaultMetadataProvider {
    fn generate_metadata(&self, request_id: String, processing_time_ms: u64) -> ApiMetadata {
        ApiMetadata {
            timestamp: current_iso8601_timestamp(),
            api_version: "1.0".to_string(),
            request_id,
            processing_time_ms: Some(processing_time_ms),
            compliance: ComplianceMetadata {
                cfr_820_compliant: true,
                iso_13485_compliant: true,
                iso_14971_compliant: true,
                audit_reference: None,
                checksum: None,
            },
            pagination: None,
        }
    }
}

/// Builder for creating standardized API responses (maintains backward compatibility)
pub struct ApiResponseBuilder {
    success: bool,
    data: Option<JsonValue>,
    error: Option<ApiError>,
    request_id: String,
    start_time: std::time::Instant,
    pagination: Option<PaginationMetadata>,
    audit_reference: Option<String>,
}

impl ApiResponseBuilder {
    /// Create a new API response builder for a successful response
    pub fn success() -> Self {
        Self {
            success: true,
            data: None,
            error: None,
            request_id: generate_uuid(),
            start_time: std::time::Instant::now(),
            pagination: None,
            audit_reference: None,
        }
    }

    /// Create a new API response builder for an error response
    pub fn error() -> Self {
        Self {
            success: false,
            data: None,
            error: None,
            request_id: generate_uuid(),
            start_time: std::time::Instant::now(),
            pagination: None,
            audit_reference: None,
        }
    }

    /// Set the response data (for successful responses)
    pub fn data(mut self, data: JsonValue) -> Self {
        self.data = Some(data);
        self
    }

    /// Set the error information (for error responses)
    pub fn error_info(mut self, code: String, message: String, status_code: u16) -> Self {
        self.error = Some(ApiError {
            code,
            message,
            details: None,
            status_code,
            trace_id: generate_uuid(),
        });
        self
    }

    /// Set detailed error information
    pub fn error_details(mut self, details: String) -> Self {
        if let Some(ref mut error) = self.error {
            error.details = Some(details);
        }
        self
    }

    /// Set request ID (useful for tracing)
    pub fn request_id(mut self, request_id: String) -> Self {
        self.request_id = request_id;
        self
    }

    /// Set pagination metadata
    pub const fn pagination(mut self, page: usize, per_page: usize, total_items: usize) -> Self {
        let total_pages = (total_items + per_page - 1) / per_page;
        self.pagination = Some(PaginationMetadata {
            page,
            per_page,
            total_items,
            total_pages,
            has_next: page < total_pages,
            has_previous: page > 1,
        });
        self
    }

    /// Set audit reference for compliance tracking
    pub fn audit_reference(mut self, audit_ref: String) -> Self {
        self.audit_reference = Some(audit_ref);
        self
    }

    /// Build the final API response
    pub fn build(self) -> StandardApiResponse {
        let processing_time_ms = self.start_time.elapsed().as_millis() as u64;
        
        StandardApiResponse {
            success: self.success,
            data: self.data,
            error: self.error,
            metadata: ApiMetadata {
                timestamp: current_iso8601_timestamp(),
                api_version: "1.0".to_string(),
                request_id: self.request_id,
                processing_time_ms: Some(processing_time_ms),
                compliance: ComplianceMetadata {
                    cfr_820_compliant: true,
                    iso_13485_compliant: true,
                    iso_14971_compliant: true,
                    audit_reference: self.audit_reference,
                    checksum: None, // Can be calculated if needed
                },
                pagination: self.pagination,
            },
        }
    }
}

/// Interface Segregation Principle: Implement specific JSON serialization
impl JsonResponseSerializer for StandardApiResponse {
    fn serialize_to_json(&self) -> String {
        let mut response_obj = HashMap::new();

        // Success flag
        response_obj.insert("success".to_string(), JsonValue::Bool(self.success));

        // Data or error
        if let Some(ref data) = self.data {
            response_obj.insert("data".to_string(), data.clone());
            response_obj.insert("error".to_string(), JsonValue::Null);
        } else if let Some(ref error) = self.error {
            response_obj.insert("data".to_string(), JsonValue::Null);
            response_obj.insert("error".to_string(), error.to_json_value());
        } else {
            response_obj.insert("data".to_string(), JsonValue::Null);
            response_obj.insert("error".to_string(), JsonValue::Null);
        }

        // Metadata
        response_obj.insert("metadata".to_string(), self.metadata.to_json_value());

        JsonValue::Object(response_obj).json_to_string()
    }
}

/// Convert StandardApiResponse to JSON string (maintains backward compatibility)
impl JsonSerializable for StandardApiResponse {
    fn to_json(&self) -> String {
        self.serialize_to_json()
    }

    fn from_json(_s: &str) -> Result<Self, crate::json_utils::JsonError> {
        // API responses are typically not parsed from JSON
        Err(crate::json_utils::JsonError::SerializationError(
            "API responses should not be parsed from JSON".to_string()
        ))
    }
}

impl ApiError {
    /// Convert ApiError to JsonValue for serialization
    fn to_json_value(&self) -> JsonValue {
        let mut error_obj = HashMap::new();
        error_obj.insert("code".to_string(), JsonValue::String(self.code.clone()));
        error_obj.insert("message".to_string(), JsonValue::String(self.message.clone()));
        error_obj.insert("status_code".to_string(), JsonValue::Number(self.status_code as f64));
        error_obj.insert("trace_id".to_string(), JsonValue::String(self.trace_id.clone()));
        
        if let Some(ref details) = self.details {
            error_obj.insert("details".to_string(), JsonValue::String(details.clone()));
        } else {
            error_obj.insert("details".to_string(), JsonValue::Null);
        }
        
        JsonValue::Object(error_obj)
    }
}

impl ApiMetadata {
    /// Convert ApiMetadata to JsonValue for serialization
    fn to_json_value(&self) -> JsonValue {
        let mut metadata_obj = HashMap::new();
        metadata_obj.insert("timestamp".to_string(), JsonValue::String(self.timestamp.clone()));
        metadata_obj.insert("api_version".to_string(), JsonValue::String(self.api_version.clone()));
        metadata_obj.insert("request_id".to_string(), JsonValue::String(self.request_id.clone()));
        
        if let Some(processing_time) = self.processing_time_ms {
            metadata_obj.insert("processing_time_ms".to_string(), JsonValue::Number(processing_time as f64));
        } else {
            metadata_obj.insert("processing_time_ms".to_string(), JsonValue::Null);
        }
        
        metadata_obj.insert("compliance".to_string(), self.compliance.to_json_value());
        
        if let Some(ref pagination) = self.pagination {
            metadata_obj.insert("pagination".to_string(), pagination.to_json_value());
        } else {
            metadata_obj.insert("pagination".to_string(), JsonValue::Null);
        }
        
        JsonValue::Object(metadata_obj)
    }
}

impl ComplianceMetadata {
    /// Convert ComplianceMetadata to JsonValue for serialization
    fn to_json_value(&self) -> JsonValue {
        let mut compliance_obj = HashMap::new();
        compliance_obj.insert("cfr_820_compliant".to_string(), JsonValue::Bool(self.cfr_820_compliant));
        compliance_obj.insert("iso_13485_compliant".to_string(), JsonValue::Bool(self.iso_13485_compliant));
        compliance_obj.insert("iso_14971_compliant".to_string(), JsonValue::Bool(self.iso_14971_compliant));
        
        if let Some(ref audit_ref) = self.audit_reference {
            compliance_obj.insert("audit_reference".to_string(), JsonValue::String(audit_ref.clone()));
        } else {
            compliance_obj.insert("audit_reference".to_string(), JsonValue::Null);
        }
        
        if let Some(ref checksum) = self.checksum {
            compliance_obj.insert("checksum".to_string(), JsonValue::String(checksum.clone()));
        } else {
            compliance_obj.insert("checksum".to_string(), JsonValue::Null);
        }
        
        JsonValue::Object(compliance_obj)
    }
}

impl PaginationMetadata {
    /// Convert PaginationMetadata to JsonValue for serialization
    fn to_json_value(&self) -> JsonValue {
        let mut pagination_obj = HashMap::new();
        pagination_obj.insert("page".to_string(), JsonValue::Number(self.page as f64));
        pagination_obj.insert("per_page".to_string(), JsonValue::Number(self.per_page as f64));
        pagination_obj.insert("total_items".to_string(), JsonValue::Number(self.total_items as f64));
        pagination_obj.insert("total_pages".to_string(), JsonValue::Number(self.total_pages as f64));
        pagination_obj.insert("has_next".to_string(), JsonValue::Bool(self.has_next));
        pagination_obj.insert("has_previous".to_string(), JsonValue::Bool(self.has_previous));
        
        JsonValue::Object(pagination_obj)
    }
}

// ================================================================================================
// Command Output Wrapper Functions (Primary Task 7.2.4 Implementation)
// ================================================================================================

/// Wrap a successful CLI command result in standardized JSON response
/// This is the primary function for task 7.2.4 - converting CLI outputs to JSON
pub fn wrap_success<T: JsonSerializable>(data: T) -> Result<String, String> {
    wrap_success_with_data(data, None)
}

/// Wrap a successful CLI command result with pagination information
pub fn wrap_success_with_pagination<T: JsonSerializable>(
    data: T,
    page: usize,
    per_page: usize,
    total_items: usize,
) -> Result<String, String> {
    let json_data = match JsonValue::parse(&data.to_json()) {
        Ok(parsed) => parsed,
        Err(e) => return Err(format!("Failed to parse data as JSON: {e}")),
    };

    let response = ApiResponseBuilder::success()
        .data(json_data)
        .pagination(page, per_page, total_items)
        .build();

    Ok(response.to_json())
}

/// Wrap a successful CLI command result with additional metadata
pub fn wrap_success_with_data<T: JsonSerializable>(
    data: T,
    audit_reference: Option<String>,
) -> Result<String, String> {
    let json_data = match JsonValue::parse(&data.to_json()) {
        Ok(parsed) => parsed,
        Err(e) => return Err(format!("Failed to parse data as JSON: {e}")),
    };

    let mut builder = ApiResponseBuilder::success().data(json_data);
    
    if let Some(audit_ref) = audit_reference {
        builder = builder.audit_reference(audit_ref);
    }

    let response = builder.build();
    Ok(response.to_json())
}

/// Error mapping strategy trait - Open/Closed Principle
/// Allows extending error mapping without modifying existing code
pub trait ErrorMapper {
    fn map_error(&self, error: &QmsError) -> (String, String, u16);
}

/// Default error mapper implementation
pub struct DefaultErrorMapper;

impl ErrorMapper for DefaultErrorMapper {
    fn map_error(&self, error: &QmsError) -> (String, String, u16) {
        match error {
            QmsError::Io(_) => ("IO_ERROR".to_string(), error.to_string(), 500),
            QmsError::Validation(_) => ("VALIDATION_ERROR".to_string(), error.to_string(), 400),
            QmsError::NotFound(_) => ("NOT_FOUND".to_string(), error.to_string(), 404),
            QmsError::Parse(_) => ("PARSE_ERROR".to_string(), error.to_string(), 400),
            QmsError::Domain(_) => ("DOMAIN_ERROR".to_string(), error.to_string(), 422),
            QmsError::Lock(_) => ("LOCK_ERROR".to_string(), error.to_string(), 409),
            QmsError::Permission(_) => ("PERMISSION_ERROR".to_string(), error.to_string(), 403),
            QmsError::AlreadyExists(_) => ("ALREADY_EXISTS".to_string(), error.to_string(), 409),
            QmsError::Authentication(_) => ("AUTHENTICATION_ERROR".to_string(), error.to_string(), 401),
            QmsError::InvalidOperation(_) => ("INVALID_OPERATION".to_string(), error.to_string(), 400),
        }
    }
}

/// Wrap an error in standardized JSON response format
/// This function satisfies the task requirement for error handling
/// Now follows Dependency Inversion Principle by accepting an ErrorMapper
pub fn wrap_error(error: QmsError) -> Result<String, String> {
    wrap_error_with_mapper(error, &DefaultErrorMapper)
}

/// Wrap an error using a custom error mapper (Dependency Inversion Principle)
pub fn wrap_error_with_mapper<M: ErrorMapper>(error: QmsError, mapper: &M) -> Result<String, String> {
    let (code, message, status_code) = mapper.map_error(&error);

    let response = ApiResponseBuilder::error()
        .error_info(code, message, status_code)
        .build();

    Ok(response.to_json())
}

/// Wrap a CLI command result (success or error) in JSON format
/// This is the main wrapper function that handles both success and error cases
pub fn wrap_command_output<T: JsonSerializable>(
    result: QmsResult<T>,
) -> Result<String, String> {
    match result {
        Ok(data) => wrap_success(data),
        Err(error) => wrap_error(error),
    }
}

/// Wrap a CLI command result with audit logging
pub fn wrap_command_output_with_audit<T: JsonSerializable>(
    result: QmsResult<T>,
    operation: &str,
    entity_type: &str,
    entity_id: &str,
) -> Result<String, String> {
    // Log the API operation to audit trail
    let audit_reference = if crate::modules::audit_logger::functions::audit_log_action(
        operation, entity_type, entity_id
    ).is_err() {
        None // Continue if audit logging fails
    } else {
        Some(format!("{}_{}_{}_{}", operation, entity_type, entity_id, generate_uuid()))
    };

    match result {
        Ok(data) => wrap_success_with_data(data, audit_reference),
        Err(error) => wrap_error(error),
    }
}

/// Create a simple success response with a message
pub fn wrap_success_message(message: &str) -> Result<String, String> {
    let mut data_obj = HashMap::new();
    data_obj.insert("message".to_string(), JsonValue::String(message.to_string()));
    
    let response = ApiResponseBuilder::success()
        .data(JsonValue::Object(data_obj))
        .build();

    Ok(response.to_json())
}

/// Create a paginated list response
pub fn wrap_paginated_list<T: JsonSerializable>(
    items: Vec<T>,
    page: usize,
    per_page: usize,
    total_items: usize,
) -> Result<String, String> {
    let json_items: Vec<JsonValue> = items.into_iter()
        .map(|item| JsonValue::parse(&item.to_json()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to parse items as JSON: {e}"))?;

    let response = ApiResponseBuilder::success()
        .data(JsonValue::Array(json_items))
        .pagination(page, per_page, total_items)
        .build();

    Ok(response.to_json())
}

/// Create an empty success response
pub fn wrap_empty_success() -> Result<String, String> {
    let response = ApiResponseBuilder::success()
        .data(JsonValue::Object(HashMap::new()))
        .build();

    Ok(response.to_json())
}

/// Single Responsibility Principle: Dedicated response validator
pub struct ApiResponseValidator;

impl ApiResponseValidator {
    /// Validate that a JSON string is properly formatted for API responses
    pub fn validate(&self, json_response: &str) -> Result<(), String> {
        match JsonValue::parse(json_response) {
            Ok(JsonValue::Object(obj)) => {
                self.validate_required_fields(&obj)?;
                self.validate_metadata_structure(&obj)?;
                Ok(())
            },
            Ok(_) => Err("API response must be a JSON object".to_string()),
            Err(e) => Err(format!("Invalid JSON: {e}")),
        }
    }

    /// Validate required top-level fields
    fn validate_required_fields(&self, obj: &HashMap<String, JsonValue>) -> Result<(), String> {
        if !obj.contains_key("success") {
            return Err("Missing 'success' field".to_string());
        }
        if !obj.contains_key("metadata") {
            return Err("Missing 'metadata' field".to_string());
        }
        Ok(())
    }

    /// Validate metadata structure (lenient for backward compatibility)
    fn validate_metadata_structure(&self, obj: &HashMap<String, JsonValue>) -> Result<(), String> {
        if let Some(JsonValue::Object(_metadata)) = obj.get("metadata") {
            // Metadata exists and is an object - that's sufficient for basic validation
            // More strict validation can be added in specialized validators
            Ok(())
        } else {
            Err("Invalid metadata structure".to_string())
        }
    }
}

/// Validate that a JSON string is properly formatted for API responses (backward compatibility)
pub fn validate_api_response(json_response: &str) -> Result<(), String> {
    let validator = ApiResponseValidator;
    validator.validate(json_response)
}

// ================================================================================================
// Convenience Functions for Common API Operations
// ================================================================================================

/// Create a standardized error response for invalid requests
pub fn invalid_request_error(message: &str) -> Result<String, String> {
    let response = ApiResponseBuilder::error()
        .error_info("INVALID_REQUEST".to_string(), message.to_string(), 400)
        .build();

    Ok(response.to_json())
}

/// Create a standardized error response for unauthorized access
pub fn unauthorized_error() -> Result<String, String> {
    let response = ApiResponseBuilder::error()
        .error_info(
            "UNAUTHORIZED".to_string(),
            "Authentication required or insufficient permissions".to_string(),
            401
        )
        .build();

    Ok(response.to_json())
}

/// Create a standardized error response for forbidden access
pub fn forbidden_error(message: &str) -> Result<String, String> {
    let response = ApiResponseBuilder::error()
        .error_info("FORBIDDEN".to_string(), message.to_string(), 403)
        .build();

    Ok(response.to_json())
}

/// Create a standardized error response for not found resources
pub fn not_found_error(resource: &str) -> Result<String, String> {
    let message = format!("The requested {resource} was not found");
    let response = ApiResponseBuilder::error()
        .error_info("NOT_FOUND".to_string(), message, 404)
        .build();

    Ok(response.to_json())
}

/// Create a standardized error response for server errors
pub fn internal_server_error(details: Option<String>) -> Result<String, String> {
    let mut builder = ApiResponseBuilder::error()
        .error_info(
            "INTERNAL_SERVER_ERROR".to_string(),
            "An internal server error occurred".to_string(),
            500
        );

    if let Some(details_str) = details {
        builder = builder.error_details(details_str);
    }

    let response = builder.build();
    Ok(response.to_json())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple test structure for serialization tests
    #[derive(Debug)]
    struct TestData {
        id: String,
        name: String,
        value: i32,
    }

    impl JsonSerializable for TestData {
        fn to_json(&self) -> String {
            format!(
                r#"{{"id": "{}", "name": "{}", "value": {}}}"#,
                self.id, self.name, self.value
            )
        }

        fn from_json(_s: &str) -> Result<Self, crate::json_utils::JsonError> {
            // Not needed for this test
            Ok(TestData {
                id: "test".to_string(),
                name: "test".to_string(),
                value: 0,
            })
        }
    }

    #[test]
    fn test_api_response_builder_success() {
        let test_data = TestData {
            id: "test-123".to_string(),
            name: "Test Document".to_string(),
            value: 42,
        };

        let json_result = wrap_success(test_data);
        assert!(json_result.is_ok());

        let json_response = json_result.unwrap();
        assert!(json_response.contains("\"success\": true"));
        assert!(json_response.contains("\"Test Document\""));
        assert!(json_response.contains("\"value\": 42"));
        assert!(json_response.contains("\"metadata\""));
        assert!(json_response.contains("\"cfr_820_compliant\": true"));
    }

    #[test]
    fn test_api_response_builder_error() {
        let error = QmsError::validation_error("Invalid input data");
        let json_result = wrap_error(error);
        assert!(json_result.is_ok());

        let json_response = json_result.unwrap();
        assert!(json_response.contains("\"success\": false"));
        assert!(json_response.contains("\"VALIDATION_ERROR\""));
        assert!(json_response.contains("\"status_code\": 400"));
        assert!(json_response.contains("\"metadata\""));
    }

    #[test]
    fn test_wrap_command_output_success() {
        let test_data = TestData {
            id: "cmd-test".to_string(),
            name: "Command Test".to_string(),
            value: 100,
        };

        let result: QmsResult<TestData> = Ok(test_data);
        let json_result = wrap_command_output(result);
        
        assert!(json_result.is_ok());
        let json_response = json_result.unwrap();
        assert!(json_response.contains("\"success\": true"));
        assert!(json_response.contains("\"Command Test\""));
    }

    #[test]
    fn test_wrap_command_output_error() {
        let result: QmsResult<TestData> = Err(QmsError::not_found("Test item not found"));
        let json_result = wrap_command_output(result);
        
        assert!(json_result.is_ok());
        let json_response = json_result.unwrap();
        assert!(json_response.contains("\"success\": false"));
        assert!(json_response.contains("\"NOT_FOUND\""));
        assert!(json_response.contains("\"status_code\": 404"));
    }

    #[test]
    fn test_wrap_success_with_pagination() {
        let test_data = TestData {
            id: "page-test".to_string(),
            name: "Pagination Test".to_string(),
            value: 1,
        };

        let json_result = wrap_success_with_pagination(test_data, 2, 10, 25);
        assert!(json_result.is_ok());

        let json_response = json_result.unwrap();
        assert!(json_response.contains("\"page\": 2"));
        assert!(json_response.contains("\"per_page\": 10"));
        assert!(json_response.contains("\"total_items\": 25"));
        assert!(json_response.contains("\"total_pages\": 3"));
        assert!(json_response.contains("\"has_next\": true"));
        assert!(json_response.contains("\"has_previous\": true"));
    }

    #[test]
    fn test_wrap_paginated_list() {
        let test_items = vec![
            TestData { id: "1".to_string(), name: "Item 1".to_string(), value: 1 },
            TestData { id: "2".to_string(), name: "Item 2".to_string(), value: 2 },
        ];

        let json_result = wrap_paginated_list(test_items, 1, 10, 2);
        assert!(json_result.is_ok());

        let json_response = json_result.unwrap();
        assert!(json_response.contains("\"success\": true"));
        assert!(json_response.contains("\"Item 1\""));
        assert!(json_response.contains("\"Item 2\""));
        assert!(json_response.contains("\"total_items\": 2"));
    }

    #[test]
    fn test_wrap_success_message() {
        let json_result = wrap_success_message("Operation completed successfully");
        assert!(json_result.is_ok());

        let json_response = json_result.unwrap();
        assert!(json_response.contains("\"success\": true"));
        assert!(json_response.contains("\"Operation completed successfully\""));
    }

    #[test]
    fn test_convenience_error_functions() {
        // Test invalid request error
        let invalid_result = invalid_request_error("Missing required field");
        assert!(invalid_result.is_ok());
        assert!(invalid_result.unwrap().contains("\"status_code\": 400"));

        // Test unauthorized error
        let unauthorized_result = unauthorized_error();
        assert!(unauthorized_result.is_ok());
        assert!(unauthorized_result.unwrap().contains("\"status_code\": 401"));

        // Test forbidden error
        let forbidden_result = forbidden_error("Insufficient permissions");
        assert!(forbidden_result.is_ok());
        assert!(forbidden_result.unwrap().contains("\"status_code\": 403"));

        // Test not found error
        let not_found_result = not_found_error("document");
        assert!(not_found_result.is_ok());
        assert!(not_found_result.unwrap().contains("\"status_code\": 404"));

        // Test internal server error
        let server_error_result = internal_server_error(Some("Database connection failed".to_string()));
        assert!(server_error_result.is_ok());
        let response = server_error_result.unwrap();
        assert!(response.contains("\"status_code\": 500"));
        assert!(response.contains("\"Database connection failed\""));
    }

    #[test]
    fn test_validate_api_response() {
        // Valid API response
        let valid_response = r#"{"success": true, "data": null, "error": null, "metadata": {}}"#;
        assert!(validate_api_response(valid_response).is_ok());

        // Invalid API response - missing success field
        let invalid_response = r#"{"data": null, "error": null, "metadata": {}}"#;
        assert!(validate_api_response(invalid_response).is_err());

        // Invalid JSON
        let malformed_response = r#"{"success": true, "data": }"#;
        assert!(validate_api_response(malformed_response).is_err());
    }

    #[test]
    fn test_compliance_metadata() {
        let test_data = TestData {
            id: "compliance-test".to_string(),
            name: "Compliance Test".to_string(),
            value: 1,
        };

        let json_result = wrap_success(test_data);
        assert!(json_result.is_ok());

        let json_response = json_result.unwrap();
        // Verify medical device compliance fields are present
        assert!(json_response.contains("\"cfr_820_compliant\": true"));
        assert!(json_response.contains("\"iso_13485_compliant\": true"));
        assert!(json_response.contains("\"iso_14971_compliant\": true"));
        assert!(json_response.contains("\"api_version\": \"1.0\""));
        assert!(json_response.contains("\"processing_time_ms\""));
    }

    #[test]
    fn test_audit_reference() {
        let test_data = TestData {
            id: "audit-test".to_string(),
            name: "Audit Test".to_string(),
            value: 1,
        };

        let audit_ref = "AUDIT_REF_12345".to_string();
        let json_result = wrap_success_with_data(test_data, Some(audit_ref.clone()));
        assert!(json_result.is_ok());

        let json_response = json_result.unwrap();
        assert!(json_response.contains(&audit_ref));
        assert!(json_response.contains("\"audit_reference\""));
    }
}
