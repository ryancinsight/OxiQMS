// QMS Web API Router - Core HTTP request routing for /api/v1/* endpoints
// Medical Device Quality Management System - FDA 21 CFR Part 820 Compliant
// Implements RESTful API routing with comprehensive medical device compliance

use std::collections::HashMap;
use std::time::SystemTime;
use crate::web::request::HttpRequest;
use crate::web::response::{HttpResponse, HttpStatus};
use crate::web::error::{WebErrorHandler, WebLogConfig};
use crate::modules::audit_logger::{AuditSession, log_user_action};
use crate::models::AuditAction;
use crate::error::QmsError;

/// HTTP method enumeration for API routing
#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    OPTIONS,
    HEAD,
}

impl HttpMethod {
    pub fn from_str(method: &str) -> Result<HttpMethod, QmsError> {
        match method.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "DELETE" => Ok(HttpMethod::DELETE),
            "PATCH" => Ok(HttpMethod::PATCH),
            "OPTIONS" => Ok(HttpMethod::OPTIONS),
            "HEAD" => Ok(HttpMethod::HEAD),
            _ => Err(QmsError::InvalidOperation(format!("Unsupported HTTP method: {method}"))),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            HttpMethod::GET => "GET".to_string(),
            HttpMethod::POST => "POST".to_string(),
            HttpMethod::PUT => "PUT".to_string(),
            HttpMethod::DELETE => "DELETE".to_string(),
            HttpMethod::PATCH => "PATCH".to_string(),
            HttpMethod::OPTIONS => "OPTIONS".to_string(),
            HttpMethod::HEAD => "HEAD".to_string(),
        }
    }
}

/// Route definition for API endpoints
#[derive(Debug, Clone)]
pub struct Route {
    pub method: HttpMethod,
    pub path: String,
    pub handler_name: String,
    pub requires_auth: bool,
    pub allowed_roles: Vec<String>,
    pub rate_limit: Option<u32>,
    pub description: String,
}

/// API route handler function signature - simplified to avoid borrowing conflicts
pub type RouteHandler = Box<dyn Fn(&HttpRequest) -> Result<HttpResponse, QmsError> + Send + Sync>;

/// Request metrics for performance monitoring
#[derive(Debug, Clone)]
pub struct RequestMetrics {
    pub start_time: SystemTime,
    pub method: String,
    pub path: String,
    pub user_agent: String,
    pub remote_addr: String,
    pub content_length: usize,
    pub processing_time_ms: u64,
    pub status_code: u16,
    pub response_size: usize,
}

/// Core API Router for medical device QMS
pub struct ApiRouter {
    routes: Vec<Route>,
    handlers: HashMap<String, RouteHandler>,
    error_handler: WebErrorHandler,
    audit_session: Option<AuditSession>,
    base_path: String,
    version: String,
    request_metrics: Vec<RequestMetrics>,
    rate_limits: HashMap<String, (u32, SystemTime)>, // IP -> (count, window_start)
    cors_enabled: bool,
    allowed_origins: Vec<String>,
}

impl ApiRouter {
    /// Create new API router with medical device compliance
    pub fn new() -> Self {
        let config = WebLogConfig::default();
        let error_handler = WebErrorHandler::new(config);
        
        ApiRouter {
            routes: Vec::new(),
            handlers: HashMap::new(),
            error_handler,
            audit_session: None,
            base_path: "/api/v1".to_string(),
            version: "1.0.0".to_string(),
            request_metrics: Vec::new(),
            rate_limits: HashMap::new(),
            cors_enabled: true,
            allowed_origins: vec!["*".to_string()], // Configurable for production
        }
    }

    /// Initialize with audit session for medical device compliance
    pub fn with_audit_session(mut self, session: AuditSession) -> Self {
        self.audit_session = Some(session);
        self
    }

    /// Register API route with handler
    pub fn register_route(&mut self, route: Route, handler: RouteHandler) -> Result<(), QmsError> {
        // Validate route path starts with base path
        if !route.path.starts_with(&self.base_path) {
            return Err(QmsError::InvalidOperation(
                format!("Route path must start with {}: {}", self.base_path, route.path)
            ));
        }

        // Check for duplicate routes
        for existing_route in &self.routes {
            if existing_route.method == route.method && existing_route.path == route.path {
                return Err(QmsError::InvalidOperation(
                    format!("Route already exists: {} {}", route.method.to_string(), route.path)
                ));
            }
        }

        // Register handler
        self.handlers.insert(route.handler_name.clone(), handler);
        self.routes.push(route);

        Ok(())
    }

    /// Convenience method to add a route with default settings
    pub fn add_route(&mut self, method: String, path: String, handler: RouteHandler) -> Result<(), QmsError> {
        let http_method = HttpMethod::from_str(&method)?;
        let route = Route {
            method: http_method,
            path,
            handler_name: format!("handler_{}", self.routes.len()),
            requires_auth: false,
            allowed_roles: vec![],
            rate_limit: None,
            description: "Auto-generated route".to_string(),
        };
        self.register_route(route, handler)
    }

    /// Create simple error response
    fn create_error_response(&self, status: HttpStatus, message: &str) -> HttpResponse {
        let mut response = HttpResponse::new(status);
        let error_json = format!(r#"{{"error": "{}", "status": {}}}"#, message, status as u16);
        response.set_body(error_json.as_bytes().to_vec());
        response.set_content_type("application/json");
        response
    }

    /// Route incoming HTTP request to appropriate handler
    pub fn route_request(&mut self, request: &HttpRequest, remote_addr: &str) -> HttpResponse {
        let start_time = SystemTime::now();
        
        // Parse HTTP method
        let method = match HttpMethod::from_str(&request.method) {
            Ok(m) => m,
            Err(_) => {
                return self.create_error_response(HttpStatus::MethodNotAllowed, "HTTP method not supported");
            }
        };

        // Handle CORS preflight requests
        if method == HttpMethod::OPTIONS && self.cors_enabled {
            return self.handle_cors_preflight();
        }

        // Check rate limiting
        if let Err(response) = self.check_rate_limit(remote_addr) {
            return response;
        }

        // Find matching route
        let matching_route = self.find_matching_route(&method, &request.uri);
        
        let response = match matching_route {
            Some(route) => {
                // Log API request for medical device audit trail
                self.log_api_request(request, remote_addr, route);
                
                // Check authentication if required
                if route.requires_auth {
                    if let Err(auth_response) = self.check_authentication(request) {
                        return auth_response;
                    }
                }

                // Execute route handler
                match self.handlers.get(&route.handler_name) {
                    Some(handler) => {
                        match handler(request) {
                            Ok(mut response) => {
                                // Add CORS headers if enabled
                                if self.cors_enabled {
                                    self.add_cors_headers(&mut response);
                                }
                                response
                            },
                            Err(error) => {
                                self.create_error_response(
                                    HttpStatus::InternalServerError,
                                    &format!("Handler error: {error}")
                                )
                            }
                        }
                    },
                    None => {
                        self.create_error_response(
                            HttpStatus::InternalServerError,
                            "Handler not found"
                        )
                    }
                }
            },
            None => {
                // Check if path starts with API base but no route found
                if request.uri.starts_with(&self.base_path) {
                    self.create_error_response(
                        HttpStatus::NotFound,
                        "API endpoint not found"
                    )
                } else {
                    // Non-API request, return 404
                    self.create_error_response(
                        HttpStatus::NotFound,
                        "Resource not found"
                    )
                }
            }
        };

        // Record request metrics
        self.record_request_metrics(request, remote_addr, start_time, &response);

        response
    }

    /// Find matching route for method and path
    fn find_matching_route(&self, method: &HttpMethod, path: &str) -> Option<&Route> {
        for route in &self.routes {
            if route.method == *method {
                // Exact path match
                if route.path == path {
                    return Some(route);
                }
                
                // Path parameter matching (e.g., /api/v1/documents/{id})
                if self.path_matches_pattern(&route.path, path) {
                    return Some(route);
                }
            }
        }
        None
    }

    /// Check if request path matches route pattern with parameters
    fn path_matches_pattern(&self, pattern: &str, path: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split('/').collect();
        let path_parts: Vec<&str> = path.split('/').collect();
        
        if pattern_parts.len() != path_parts.len() {
            return false;
        }

        for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
            if pattern_part.starts_with('{') && pattern_part.ends_with('}') {
                // Parameter placeholder - matches any value
                continue;
            } else if pattern_part != path_part {
                return false;
            }
        }
        
        true
    }

    /// Extract path parameters from URL
    pub fn extract_path_params(&self, pattern: &str, path: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let pattern_parts: Vec<&str> = pattern.split('/').collect();
        let path_parts: Vec<&str> = path.split('/').collect();
        
        for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
            if pattern_part.starts_with('{') && pattern_part.ends_with('}') {
                let param_name = &pattern_part[1..pattern_part.len()-1];
                params.insert(param_name.to_string(), path_part.to_string());
            }
        }
        
        params
    }

    /// Check rate limiting for medical device security
    fn check_rate_limit(&mut self, remote_addr: &str) -> Result<(), HttpResponse> {
        let rate_limit = 100; // requests per minute
        let window_duration = std::time::Duration::from_secs(60);
        
        let now = SystemTime::now();
        
        let (count, window_start) = self.rate_limits
            .get(remote_addr)
            .cloned()
            .unwrap_or((0, now));
        
        // Reset window if expired
        if now.duration_since(window_start).unwrap_or_default() > window_duration {
            self.rate_limits.insert(remote_addr.to_string(), (1, now));
            return Ok(());
        }
        
        // Check rate limit
        if count >= rate_limit {
            let response = self.create_error_response(
                HttpStatus::ServiceUnavailable, // Using 503 instead of 429
                "Rate limit exceeded. Please wait before making more requests."
            );
            return Err(response);
        }
        
        // Increment counter
        self.rate_limits.insert(remote_addr.to_string(), (count + 1, window_start));
        Ok(())
    }

    /// Handle CORS preflight requests
    fn handle_cors_preflight(&self) -> HttpResponse {
        let mut response = HttpResponse::new(HttpStatus::NoContent);
        response.add_header("Access-Control-Allow-Origin", "*");
        response.add_header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, PATCH, OPTIONS");
        response.add_header("Access-Control-Allow-Headers", "Content-Type, Authorization, X-Requested-With");
        response.add_header("Access-Control-Max-Age", "86400");
        response
    }

    /// Add CORS headers to response
    fn add_cors_headers(&self, response: &mut HttpResponse) {
        if self.cors_enabled {
            response.add_header("Access-Control-Allow-Origin", "*");
            response.add_header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, PATCH, OPTIONS");
            response.add_header("Access-Control-Allow-Headers", "Content-Type, Authorization, X-Requested-With");
        }
    }

    /// Check authentication for protected routes
    fn check_authentication(&self, request: &HttpRequest) -> Result<(), HttpResponse> {
        // Check for Authorization header
        if let Some(auth_header) = request.headers.get("Authorization") {
            if auth_header.starts_with("Bearer ") {
                // TODO: Implement JWT token validation
                // For now, accept any Bearer token
                return Ok(());
            }
        }

        // Check for session cookie
        if let Some(cookie_header) = request.headers.get("Cookie") {
            if cookie_header.contains("session_id=") {
                // TODO: Implement session validation
                // For now, accept any session cookie
                return Ok(());
            }
        }

        // No valid authentication found
        let response = self.create_error_response(
            HttpStatus::Unauthorized,
            "Authentication required. Please provide a valid Bearer token or session cookie."
        );
        Err(response)
    }

    /// Log API request for medical device audit trail
    fn log_api_request(&self, request: &HttpRequest, remote_addr: &str, route: &Route) {
        if let Some(ref session) = self.audit_session {
            let action = format!("API_REQUEST: {} {}", request.method, request.uri);
            let details = format!(
                "Route: {}, Remote: {}, User-Agent: {}, Auth Required: {}",
                route.description,
                remote_addr,
                request.headers.get("User-Agent").unwrap_or(&"Unknown".to_string()),
                route.requires_auth
            );
            
            // Log with audit system
            let _ = log_user_action(
                &session.user_id,
                AuditAction::Other(action),
                "web_api",
                &request.uri,
                Some(&details)
            );
        }
    }

    /// Record request metrics for performance monitoring
    fn record_request_metrics(&mut self, request: &HttpRequest, remote_addr: &str, start_time: SystemTime, response: &HttpResponse) {
        let processing_time = start_time.elapsed()
            .unwrap_or_default()
            .as_millis() as u64;

        let metrics = RequestMetrics {
            start_time,
            method: request.method.clone(),
            path: request.uri.clone(),
            user_agent: request.headers.get("User-Agent").unwrap_or(&"Unknown".to_string()).clone(),
            remote_addr: remote_addr.to_string(),
            content_length: request.body.len(),
            processing_time_ms: processing_time,
            status_code: response.status.code(),
            response_size: response.body.len(),
        };

        self.request_metrics.push(metrics);

        // Keep only last 1000 metrics to prevent memory issues
        if self.request_metrics.len() > 1000 {
            self.request_metrics.remove(0);
        }
    }

    /// Get API router statistics
    pub fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        
        stats.insert("total_routes".to_string(), self.routes.len().to_string());
        stats.insert("total_requests".to_string(), self.request_metrics.len().to_string());
        stats.insert("version".to_string(), self.version.clone());
        stats.insert("base_path".to_string(), self.base_path.clone());

        if !self.request_metrics.is_empty() {
            let avg_processing_time: u64 = self.request_metrics
                .iter()
                .map(|m| m.processing_time_ms)
                .sum::<u64>() / self.request_metrics.len() as u64;
            
            stats.insert("avg_processing_time_ms".to_string(), avg_processing_time.to_string());
        }

        stats
    }

    /// List all registered routes
    pub const fn list_routes(&self) -> &Vec<Route> {
        &self.routes
    }

    /// Get recent request metrics
    pub fn get_recent_metrics(&self, limit: usize) -> Vec<&RequestMetrics> {
        let start_idx = if self.request_metrics.len() > limit {
            self.request_metrics.len() - limit
        } else {
            0
        };
        
        self.request_metrics[start_idx..].iter().collect()
    }
}

impl Default for ApiRouter {
    fn default() -> Self {
        Self::new()
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_request(method: &str, uri: &str) -> HttpRequest {
        HttpRequest {
            method: method.to_string(),
            uri: uri.to_string(),
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
            query_params: HashMap::new(),
            timestamp: 0,
        }
    }

    fn test_handler(_request: &HttpRequest) -> Result<HttpResponse, QmsError> {
        Ok(HttpResponse {
            status: crate::web::response::HttpStatus::Ok,
            headers: HashMap::new(),
            body: b"OK".to_vec(),
        })
    }

    #[test]
    fn test_router_creation() {
        let router = ApiRouter::new();
        assert_eq!(router.base_path, "/api/v1");
        assert_eq!(router.version, "1.0.0");
        assert!(router.cors_enabled);
        assert_eq!(router.routes.len(), 0);
    }

    #[test]
    fn test_http_method_parsing() {
        assert_eq!(HttpMethod::from_str("GET").unwrap(), HttpMethod::GET);
        assert_eq!(HttpMethod::from_str("post").unwrap(), HttpMethod::POST);
        assert!(HttpMethod::from_str("INVALID").is_err());
    }

    #[test]
    fn test_route_registration() {
        let mut router = ApiRouter::new();
        let route = Route {
            method: HttpMethod::GET,
            path: "/api/v1/test".to_string(),
            handler_name: "test_handler".to_string(),
            requires_auth: false,
            allowed_roles: vec![],
            rate_limit: None,
            description: "Test endpoint".to_string(),
        };

        assert!(router.register_route(route, Box::new(test_handler)).is_ok());
        assert_eq!(router.routes.len(), 1);
    }

    #[test]
    fn test_path_parameter_extraction() {
        let router = ApiRouter::new();
        let params = router.extract_path_params(
            "/api/v1/documents/{id}/versions/{version}",
            "/api/v1/documents/123/versions/2"
        );
        
        assert_eq!(params.get("id"), Some(&"123".to_string()));
        assert_eq!(params.get("version"), Some(&"2".to_string()));
    }

    #[test]
    fn test_path_pattern_matching() {
        let router = ApiRouter::new();
        
        assert!(router.path_matches_pattern("/api/v1/documents/{id}", "/api/v1/documents/123"));
        assert!(!router.path_matches_pattern("/api/v1/documents/{id}", "/api/v1/documents"));
        assert!(router.path_matches_pattern("/api/v1/test", "/api/v1/test"));
    }

    #[test]
    fn test_cors_preflight_handling() {
        let router = ApiRouter::new();
        let response = router.handle_cors_preflight();
        
        assert_eq!(response.status.code(), 204);
        assert!(response.headers.contains_key("Access-Control-Allow-Origin"));
        assert!(response.headers.contains_key("Access-Control-Allow-Methods"));
    }

    #[test]
    fn test_stats_generation() {
        let router = ApiRouter::new();
        let stats = router.get_stats();
        
        assert_eq!(stats.get("total_routes"), Some(&"0".to_string()));
        assert_eq!(stats.get("version"), Some(&"1.0.0".to_string()));
        assert_eq!(stats.get("base_path"), Some(&"/api/v1".to_string()));
    }

    #[test]
    fn test_route_not_found() {
        let mut router = ApiRouter::new();
        let request = create_test_request("GET", "/api/v1/nonexistent");
        let response = router.route_request(&request, "127.0.0.1");
        
        assert_eq!(response.status.code(), 404);
    }

    #[test]
    fn test_method_not_allowed() {
        let mut router = ApiRouter::new();
        let request = create_test_request("INVALID", "/api/v1/test");
        let response = router.route_request(&request, "127.0.0.1");
        
        assert_eq!(response.status.code(), 405);
    }
}
