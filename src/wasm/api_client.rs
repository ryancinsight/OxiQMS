// QMS WASM API Client
// HTTP client for consuming QMS REST API endpoints from WebAssembly
// Provides type-safe API access with error handling

use crate::prelude::*;
use std::collections::HashMap;

/// WASM-compatible API client for QMS REST endpoints
#[derive(Debug, Clone)]
pub struct QmsApiClient {
    base_url: String,
    auth_token: Option<String>,
    session_id: Option<String>,
}

/// HTTP methods supported by the API client
#[derive(Debug, Clone)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

/// API response structure
#[derive(Debug)]
pub struct ApiResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

/// API request structure
#[derive(Debug)]
pub struct ApiRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl QmsApiClient {
    /// Create new API client with base URL
    pub const fn new(base_url: String) -> Self {
        Self {
            base_url,
            auth_token: None,
            session_id: None,
        }
    }

    /// Set authentication token
    pub fn set_auth_token(&mut self, token: String) {
        self.auth_token = Some(token);
    }

    /// Set session ID
    pub fn set_session_id(&mut self, session_id: String) {
        self.session_id = Some(session_id);
    }

    /// Make authenticated API request
    pub fn request(&self, req: ApiRequest) -> QmsResult<ApiResponse> {
        // In WASM, this would use web_sys::fetch
        // For now, implement a stdlib-compatible version for compilation
        self.make_request_internal(req)
    }

    /// Make GET request
    pub fn get(&self, path: &str) -> QmsResult<ApiResponse> {
        let url = format!("{}{}", self.base_url, path);
        let request = ApiRequest {
            method: HttpMethod::Get,
            url,
            headers: std::collections::HashMap::new(),
            body: None,
        };
        self.request(request)
    }

    /// Make POST request
    pub fn post(&self, path: &str, body: Option<String>) -> QmsResult<ApiResponse> {
        let url = format!("{}{}", self.base_url, path);
        let request = ApiRequest {
            method: HttpMethod::Post,
            url,
            headers: std::collections::HashMap::new(),
            body,
        };
        self.request(request)
    }

    /// Make PUT request
    pub fn put(&self, path: &str, body: Option<String>) -> QmsResult<ApiResponse> {
        let url = format!("{}{}", self.base_url, path);
        let request = ApiRequest {
            method: HttpMethod::Put,
            url,
            headers: std::collections::HashMap::new(),
            body,
        };
        self.request(request)
    }

    /// Make DELETE request
    pub fn delete(&self, path: &str) -> QmsResult<ApiResponse> {
        let url = format!("{}{}", self.base_url, path);
        let request = ApiRequest {
            method: HttpMethod::Delete,
            url,
            headers: std::collections::HashMap::new(),
            body: None,
        };
        self.request(request)
    }

    /// Internal request implementation (enhanced mock for development)
    fn make_request_internal(&self, req: ApiRequest) -> QmsResult<ApiResponse> {
        // This is a comprehensive mock implementation for development and testing
        // In actual WASM, this would use web_sys::fetch or similar
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        if let Some(ref token) = self.auth_token {
            headers.insert("Authorization".to_string(), format!("Bearer {token}"));
        }
        
        if let Some(ref session) = self.session_id {
            headers.insert("X-Session-ID".to_string(), session.clone());
        }

        // Log request for debugging
        println!("[WASM API] {} {}", req.method.as_str(), req.url);
        if let Some(ref body) = req.body {
            println!("[WASM API] Body: {body}");
        }

        // Comprehensive mock responses based on URL and method
        let response = if req.url.ends_with("/api/health") {
            ApiResponse {
                status: 200,
                headers: headers.clone(),
                body: r#"{"status":"healthy","version":"1.0.0","uptime":3600,"database":"connected","compliance":"medical_device"}"#.to_string(),
            }
        } else if req.url.ends_with("/api/system/stats") {
            ApiResponse {
                status: 200,
                headers: headers.clone(),
                body: r#"{"documents":25,"risks":8,"requirements":42,"audit_entries":156,"active_users":3,"compliance_score":92,"last_updated":"2025-01-19T15:00:00Z"}"#.to_string(),
            }
        } else if req.url.ends_with("/api/compliance/badges") {
            ApiResponse {
                status: 200,
                headers: headers.clone(),
                body: r#"{"fda_21_cfr":{"score":95,"status":"compliant","last_audit":"2025-01-15"},"iso_13485":{"score":88,"status":"compliant","last_audit":"2025-01-10"},"iso_14971":{"score":92,"status":"compliant","last_audit":"2025-01-12"},"overall":{"score":91,"status":"excellent","last_updated":"2025-01-19"}}"#.to_string(),
            }
        } else if req.url.ends_with("/api/documents") && matches!(req.method, HttpMethod::Get) {
            ApiResponse {
                status: 200,
                headers: headers.clone(),
                body: r#"[{"id":"DOC-001","title":"Software Requirements Specification","type":"SRS","status":"Approved","version":"2.1.0","last_modified":"2025-01-15T14:30:00Z","author":"admin","checksum":"sha256:abc123"},{"id":"DOC-002","title":"System Design Document","type":"SDD","status":"Draft","version":"1.0.0","last_modified":"2025-01-10T10:00:00Z","author":"engineer1","checksum":"sha256:def456"},{"id":"DOC-003","title":"Test Protocol","type":"Test","status":"InReview","version":"1.5.0","last_modified":"2025-01-12T16:45:00Z","author":"tester1","checksum":"sha256:ghi789"}]"#.to_string(),
            }
        } else if req.url.ends_with("/api/documents") && matches!(req.method, HttpMethod::Post) {
            ApiResponse {
                status: 201,
                headers: headers.clone(),
                body: r#"{"id":"DOC-new","title":"New Document","type":"SRS","status":"Draft","version":"1.0.0","created_at":"2025-01-19T15:00:00Z","author":"current_user","message":"Document created successfully"}"#.to_string(),
            }
        } else if req.url.ends_with("/api/risks") && matches!(req.method, HttpMethod::Get) {
            ApiResponse {
                status: 200,
                headers: headers.clone(),
                body: r#"[{"id":"RISK-001","hazard_id":"HAZ-001","hazard_description":"Power failure during operation","hazardous_situation":"System loses power during critical procedure","harm":"Potential patient injury","severity":5,"occurrence":3,"detectability":2,"rpn":30,"risk_level":"ALARP","status":"Mitigated","created_at":"2025-01-01T00:00:00Z","mitigation_measures":[{"measure":"UPS backup system","effectiveness":0.8,"status":"Implemented"}]},{"id":"RISK-002","hazard_id":"HAZ-002","hazard_description":"Software crash","hazardous_situation":"Application crashes during data entry","harm":"Data loss or corruption","severity":4,"occurrence":4,"detectability":3,"rpn":48,"risk_level":"ALARP","status":"Assessed","created_at":"2025-01-05T00:00:00Z","mitigation_measures":[]},{"id":"RISK-003","hazard_id":"HAZ-003","hazard_description":"Data corruption","hazardous_situation":"Database corruption during backup","harm":"Loss of audit trail","severity":5,"occurrence":2,"detectability":4,"rpn":40,"risk_level":"ALARP","status":"Identified","created_at":"2025-01-08T00:00:00Z","mitigation_measures":[]}]"#.to_string(),
            }
        } else if req.url.ends_with("/api/risks") && matches!(req.method, HttpMethod::Post) {
            ApiResponse {
                status: 201,
                headers: headers.clone(),
                body: r#"{"id":"RISK-new","hazard_id":"HAZ-004","hazard_description":"New hazard","status":"Identified","rpn":0,"risk_level":"Unassessed","created_at":"2025-01-19T15:00:00Z","message":"Risk created successfully"}"#.to_string(),
            }
        } else if req.url.ends_with("/api/requirements") && matches!(req.method, HttpMethod::Get) {
            ApiResponse {
                status: 200,
                headers: headers.clone(),
                body: r#"[{"id":"REQ-001","req_id":"REQ-001","title":"User Authentication","description":"System shall provide secure user authentication with password hashing","category":"Security","priority":"Critical","status":"Verified","verification_method":"Test","created_at":"2025-01-01T00:00:00Z","linked_tests":["TC-001","TC-002"],"acceptance_criteria":"Authentication successful with valid credentials, failure with invalid credentials"},{"id":"REQ-002","req_id":"REQ-002","title":"Data Backup","description":"System shall automatically backup all data every 24 hours","category":"Reliability","priority":"High","status":"Implemented","verification_method":"Analysis","created_at":"2025-01-02T00:00:00Z","linked_tests":["TC-003"],"acceptance_criteria":"Backup completes successfully, data can be restored"},{"id":"REQ-003","req_id":"REQ-003","title":"Audit Logging","description":"System shall log all user actions with timestamp and user identification","category":"Regulatory","priority":"Critical","status":"Verified","verification_method":"Test","created_at":"2025-01-03T00:00:00Z","linked_tests":["TC-004","TC-005"],"acceptance_criteria":"All actions logged, logs immutable, user identification present"}]"#.to_string(),
            }
        } else if req.url.ends_with("/api/requirements") && matches!(req.method, HttpMethod::Post) {
            ApiResponse {
                status: 201,
                headers: headers.clone(),
                body: r#"{"id":"REQ-new","req_id":"REQ-004","title":"New Requirement","category":"Functional","priority":"Medium","status":"Draft","created_at":"2025-01-19T15:00:00Z","message":"Requirement created successfully"}"#.to_string(),
            }
        } else if req.url.contains("/api/audit") {
            ApiResponse {
                status: 200,
                headers: headers.clone(),
                body: r#"[{"id":"audit-001","timestamp":"2025-01-19T14:30:15Z","user":"admin","action":"CREATE","entity_type":"Document","entity_id":"DOC-003","details":"Created new test protocol document","session_id":"sess-123","checksum":"sha256:abc123"},{"id":"audit-002","timestamp":"2025-01-19T14:25:42Z","user":"engineer1","action":"UPDATE","entity_type":"Risk","entity_id":"RISK-001","details":"Updated mitigation measure effectiveness to 0.8","old_value":"0.6","new_value":"0.8","session_id":"sess-456","checksum":"sha256:def456"},{"id":"audit-003","timestamp":"2025-01-19T14:20:18Z","user":"admin","action":"APPROVE","entity_type":"Document","entity_id":"DOC-001","details":"Approved SRS document version 2.1.0","signature":"electronic_signature_hash","session_id":"sess-123","checksum":"sha256:ghi789"}]"#.to_string(),
            }
        } else if req.url.ends_with("/api/auth/login") && matches!(req.method, HttpMethod::Post) {
            if req.body.is_some() {
                ApiResponse {
                    status: 200,
                    headers: headers.clone(),
                    body: r#"{"success":true,"user_id":"user-001","username":"admin","session_id":"sess-12345","token":"jwt-token-example","expires_in":3600,"permissions":["read_documents","write_documents","read_risks","write_risks","read_audit","manage_users"],"message":"Login successful"}"#.to_string(),
                }
            } else {
                ApiResponse {
                    status: 400,
                    headers: headers.clone(),
                    body: r#"{"success":false,"error":"Missing credentials","message":"Username and password are required"}"#.to_string(),
                }
            }
        } else if req.url.ends_with("/api/auth/logout") && matches!(req.method, HttpMethod::Post) {
            ApiResponse {
                status: 200,
                headers: headers.clone(),
                body: r#"{"success":true,"message":"Logged out successfully","timestamp":"2025-01-19T15:00:00Z"}"#.to_string(),
            }
        } else if req.url.contains("/api/documents/") && matches!(req.method, HttpMethod::Get) {
            // Individual document endpoint
            ApiResponse {
                status: 200,
                headers: headers.clone(),
                body: r#"{"id":"DOC-001","title":"Software Requirements Specification","content":"Requirements content here","type":"SRS","status":"Approved","version":"2.1.0","author":"admin","created_at":"2024-12-01T00:00:00Z","updated_at":"2025-01-15T14:30:00Z","checksum":"sha256:abc123","regulatory_mapping":["FDA_21_CFR_820_30","ISO_13485_7_3"]}"#.to_string(),
            }
        } else if req.url.contains("/api/risks/") && matches!(req.method, HttpMethod::Get) {
            // Individual risk endpoint
            ApiResponse {
                status: 200,
                headers: headers.clone(),
                body: r#"{"id":"RISK-001","hazard_description":"Power failure during operation","severity":5,"occurrence":3,"detectability":2,"rpn":30,"risk_level":"ALARP","status":"Mitigated","mitigation_measures":[{"measure":"Install UPS backup system","status":"Implemented"}],"residual_risk":{"severity":5,"occurrence":1,"detectability":2,"rpn":10}}"#.to_string(),
            }
        } else if req.url.contains("/api/requirements/") && matches!(req.method, HttpMethod::Get) {
            // Individual requirement endpoint
            ApiResponse {
                status: 200,
                headers: headers.clone(),
                body: r#"{"id":"REQ-001","title":"User Authentication","description":"Secure user authentication","category":"Security","priority":"Critical","status":"Verified","verification_method":"Test","acceptance_criteria":"User authentication requirements","linked_tests":["TC-001","TC-002"],"regulatory_references":["FDA_21_CFR_11_10","ISO_13485_4_2"]}"#.to_string(),
            }
        } else {
            // Default 404 response
            ApiResponse {
                status: 404,
                headers: headers.clone(),
                body: format!(r#"{{"error":"Endpoint not found","method":"{}","path":"{}","timestamp":"2025-01-19T15:00:00Z","request_id":"req-12345"}}"#, req.method.as_str(), req.url),
            }
        };

        Ok(response)
    }

    // QMS-specific API methods

    /// Get system health status
    pub fn get_health(&self) -> QmsResult<ApiResponse> {
        let req = ApiRequest {
            method: HttpMethod::Get,
            url: format!("{}/api/health", self.base_url),
            headers: HashMap::new(),
            body: None,
        };
        self.request(req)
    }

    /// Get system statistics
    pub fn get_system_stats(&self) -> QmsResult<ApiResponse> {
        let req = ApiRequest {
            method: HttpMethod::Get,
            url: format!("{}/api/system/stats", self.base_url),
            headers: HashMap::new(),
            body: None,
        };
        self.request(req)
    }

    /// Get compliance badges
    pub fn get_compliance_badges(&self) -> QmsResult<ApiResponse> {
        let req = ApiRequest {
            method: HttpMethod::Get,
            url: format!("{}/api/compliance/badges", self.base_url),
            headers: HashMap::new(),
            body: None,
        };
        self.request(req)
    }

    /// List documents
    pub fn list_documents(&self) -> QmsResult<ApiResponse> {
        let req = ApiRequest {
            method: HttpMethod::Get,
            url: format!("{}/api/documents", self.base_url),
            headers: HashMap::new(),
            body: None,
        };
        self.request(req)
    }

    /// Get document by ID
    pub fn get_document(&self, doc_id: &str) -> QmsResult<ApiResponse> {
        let req = ApiRequest {
            method: HttpMethod::Get,
            url: format!("{}/api/documents/{}", self.base_url, doc_id),
            headers: HashMap::new(),
            body: None,
        };
        self.request(req)
    }

    /// Create new document
    pub fn create_document(&self, title: &str, content: &str, doc_type: &str) -> QmsResult<ApiResponse> {
        let body = format!(
            r#"{{"title":"{title}","content":"{content}","doc_type":"{doc_type}"}}"#
        );
        
        let req = ApiRequest {
            method: HttpMethod::Post,
            url: format!("{}/api/documents", self.base_url),
            headers: HashMap::new(),
            body: Some(body),
        };
        self.request(req)
    }

    /// List risks
    pub fn list_risks(&self) -> QmsResult<ApiResponse> {
        let req = ApiRequest {
            method: HttpMethod::Get,
            url: format!("{}/api/risks", self.base_url),
            headers: HashMap::new(),
            body: None,
        };
        self.request(req)
    }

    /// Get risk by ID
    pub fn get_risk(&self, risk_id: &str) -> QmsResult<ApiResponse> {
        let req = ApiRequest {
            method: HttpMethod::Get,
            url: format!("{}/api/risks/{}", self.base_url, risk_id),
            headers: HashMap::new(),
            body: None,
        };
        self.request(req)
    }

    /// Create new risk
    pub fn create_risk(&self, hazard: &str, situation: &str, harm: &str) -> QmsResult<ApiResponse> {
        let body = format!(
            r#"{{"hazard_description":"{hazard}","hazardous_situation":"{situation}","harm":"{harm}"}}"#
        );
        
        let req = ApiRequest {
            method: HttpMethod::Post,
            url: format!("{}/api/risks", self.base_url),
            headers: HashMap::new(),
            body: Some(body),
        };
        self.request(req)
    }

    /// List requirements
    pub fn list_requirements(&self) -> QmsResult<ApiResponse> {
        let req = ApiRequest {
            method: HttpMethod::Get,
            url: format!("{}/api/requirements", self.base_url),
            headers: HashMap::new(),
            body: None,
        };
        self.request(req)
    }

    /// Get requirement by ID
    pub fn get_requirement(&self, req_id: &str) -> QmsResult<ApiResponse> {
        let req = ApiRequest {
            method: HttpMethod::Get,
            url: format!("{}/api/requirements/{}", self.base_url, req_id),
            headers: HashMap::new(),
            body: None,
        };
        self.request(req)
    }

    /// Create new requirement
    pub fn create_requirement(&self, title: &str, description: &str, category: &str) -> QmsResult<ApiResponse> {
        let body = format!(
            r#"{{"title":"{title}","description":"{description}","category":"{category}"}}"#
        );
        
        let req = ApiRequest {
            method: HttpMethod::Post,
            url: format!("{}/api/requirements", self.base_url),
            headers: HashMap::new(),
            body: Some(body),
        };
        self.request(req)
    }

    /// Get audit entries
    pub fn get_audit_entries(&self, limit: Option<u32>) -> QmsResult<ApiResponse> {
        let url = if let Some(limit) = limit {
            format!("{}/api/audit?limit={}", self.base_url, limit)
        } else {
            format!("{}/api/audit", self.base_url)
        };
        
        let req = ApiRequest {
            method: HttpMethod::Get,
            url,
            headers: HashMap::new(),
            body: None,
        };
        self.request(req)
    }

    /// User authentication
    pub fn login(&mut self, username: &str, password: &str) -> QmsResult<ApiResponse> {
        let body = format!(
            r#"{{"username":"{username}","password":"{password}"}}"#
        );
        
        let req = ApiRequest {
            method: HttpMethod::Post,
            url: format!("{}/api/auth/login", self.base_url),
            headers: HashMap::new(),
            body: Some(body),
        };
        
        let response = self.request(req)?;
        
        // Extract token and session from response (in real implementation)
        // For now, set mock values
        if response.status == 200 {
            self.auth_token = Some("mock_token".to_string());
            self.session_id = Some("mock_session".to_string());
        }
        
        Ok(response)
    }

    /// User logout
    pub fn logout(&mut self) -> QmsResult<ApiResponse> {
        let req = ApiRequest {
            method: HttpMethod::Post,
            url: format!("{}/api/auth/logout", self.base_url),
            headers: HashMap::new(),
            body: None,
        };
        
        let response = self.request(req)?;
        
        // Clear authentication
        self.auth_token = None;
        self.session_id = None;
        
        Ok(response)
    }
}

impl HttpMethod {
    pub const fn as_str(&self) -> &str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_client_creation() {
        let client = QmsApiClient::new("http://localhost:8080".to_string());
        assert_eq!(client.base_url, "http://localhost:8080");
        assert!(client.auth_token.is_none());
        assert!(client.session_id.is_none());
    }

    #[test]
    fn test_set_auth_token() {
        let mut client = QmsApiClient::new("http://localhost:8080".to_string());
        client.set_auth_token("test_token".to_string());
        assert_eq!(client.auth_token, Some("test_token".to_string()));
    }

    #[test]
    fn test_http_method_as_str() {
        assert_eq!(HttpMethod::Get.as_str(), "GET");
        assert_eq!(HttpMethod::Post.as_str(), "POST");
        assert_eq!(HttpMethod::Put.as_str(), "PUT");
        assert_eq!(HttpMethod::Delete.as_str(), "DELETE");
    }

    #[test]
    fn test_get_health_request() {
        let client = QmsApiClient::new("http://localhost:8080".to_string());
        let response = client.get_health().unwrap();
        assert_eq!(response.status, 200);
        assert!(response.body.contains("healthy"));
        assert!(response.body.contains("medical_device"));
    }

    #[test]
    fn test_get_system_stats() {
        let client = QmsApiClient::new("http://localhost:8080".to_string());
        let response = client.get_system_stats().unwrap();
        assert_eq!(response.status, 200);
        assert!(response.body.contains("documents"));
        assert!(response.body.contains("risks"));
        assert!(response.body.contains("requirements"));
        assert!(response.body.contains("compliance_score"));
    }

    #[test]
    fn test_get_compliance_badges() {
        let client = QmsApiClient::new("http://localhost:8080".to_string());
        let response = client.get_compliance_badges().unwrap();
        assert_eq!(response.status, 200);
        assert!(response.body.contains("fda_21_cfr"));
        assert!(response.body.contains("iso_13485"));
        assert!(response.body.contains("iso_14971"));
        assert!(response.body.contains("overall"));
    }

    #[test]
    fn test_list_documents() {
        let client = QmsApiClient::new("http://localhost:8080".to_string());
        let response = client.list_documents().unwrap();
        assert_eq!(response.status, 200);
        assert!(response.body.contains("DOC-001"));
        assert!(response.body.contains("Software Requirements"));
        assert!(response.body.contains("SRS"));
    }

    #[test]
    fn test_create_document() {
        let client = QmsApiClient::new("http://localhost:8080".to_string());
        let response = client.create_document("Test Document", "Test content", "SRS").unwrap();
        assert_eq!(response.status, 201);
        assert!(response.body.contains("DOC-new"));
        assert!(response.body.contains("created successfully"));
    }

    #[test]
    fn test_list_risks() {
        let client = QmsApiClient::new("http://localhost:8080".to_string());
        let response = client.list_risks().unwrap();
        assert_eq!(response.status, 200);
        assert!(response.body.contains("RISK-001"));
        assert!(response.body.contains("Power failure"));
        assert!(response.body.contains("rpn"));
        assert!(response.body.contains("ALARP"));
    }

    #[test]
    fn test_create_risk() {
        let client = QmsApiClient::new("http://localhost:8080".to_string());
        let response = client.create_risk("Test hazard", "Test situation", "Test harm").unwrap();
        assert_eq!(response.status, 201);
        assert!(response.body.contains("RISK-new"));
        assert!(response.body.contains("created successfully"));
    }

    #[test]
    fn test_list_requirements() {
        let client = QmsApiClient::new("http://localhost:8080".to_string());
        let response = client.list_requirements().unwrap();
        assert_eq!(response.status, 200);
        assert!(response.body.contains("REQ-001"));
        assert!(response.body.contains("User Authentication"));
        assert!(response.body.contains("Security"));
        assert!(response.body.contains("Critical"));
    }

    #[test]
    fn test_create_requirement() {
        let client = QmsApiClient::new("http://localhost:8080".to_string());
        let response = client.create_requirement("Test Requirement", "Test description", "Functional").unwrap();
        assert_eq!(response.status, 201);
        assert!(response.body.contains("REQ-new"));
        assert!(response.body.contains("created successfully"));
    }

    #[test]
    fn test_get_audit_entries() {
        let client = QmsApiClient::new("http://localhost:8080".to_string());
        let response = client.get_audit_entries(Some(10)).unwrap();
        assert_eq!(response.status, 200);
        assert!(response.body.contains("audit-001"));
        assert!(response.body.contains("CREATE"));
        assert!(response.body.contains("admin"));
        assert!(response.body.contains("timestamp"));
    }

    #[test]
    fn test_get_individual_document() {
        let client = QmsApiClient::new("http://localhost:8080".to_string());
        let response = client.get_document("DOC-001").unwrap();
        assert_eq!(response.status, 200);
        assert!(response.body.contains("Software Requirements Specification"));
        assert!(response.body.contains("content"));
        assert!(response.body.contains("regulatory_mapping"));
    }

    #[test]
    fn test_get_individual_risk() {
        let client = QmsApiClient::new("http://localhost:8080".to_string());
        let response = client.get_risk("RISK-001").unwrap();
        assert_eq!(response.status, 200);
        assert!(response.body.contains("Power failure"));
        assert!(response.body.contains("mitigation_measures"));
        assert!(response.body.contains("residual_risk"));
    }

    #[test]
    fn test_get_individual_requirement() {
        let client = QmsApiClient::new("http://localhost:8080".to_string());
        let response = client.get_requirement("REQ-001").unwrap();
        assert_eq!(response.status, 200);
        assert!(response.body.contains("User Authentication"));
        assert!(response.body.contains("acceptance_criteria"));
        assert!(response.body.contains("linked_tests"));
        assert!(response.body.contains("regulatory_references"));
    }

    #[test]
    fn test_404_response() {
        let client = QmsApiClient::new("http://localhost:8080".to_string());
        let req = ApiRequest {
            method: HttpMethod::Get,
            url: "http://localhost:8080/api/nonexistent".to_string(),
            headers: HashMap::new(),
            body: None,
        };
        let response = client.request(req).unwrap();
        assert_eq!(response.status, 404);
        assert!(response.body.contains("Endpoint not found"));
    }

    #[test]
    fn test_login_sets_auth_token() {
        let mut client = QmsApiClient::new("http://localhost:8080".to_string());
        let response = client.login("test_user", "test_pass").unwrap();
        assert_eq!(response.status, 200);
        assert!(client.auth_token.is_some());
        assert!(client.session_id.is_some());
    }

    #[test]
    fn test_logout_clears_auth() {
        let mut client = QmsApiClient::new("http://localhost:8080".to_string());
        client.set_auth_token("test_token".to_string());
        client.set_session_id("test_session".to_string());
        
        let response = client.logout().unwrap();
        assert_eq!(response.status, 200);
        assert!(client.auth_token.is_none());
        assert!(client.session_id.is_none());
    }
}
