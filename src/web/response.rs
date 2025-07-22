use std::collections::HashMap;
use std::fmt;

/// HTTP response representation
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: HttpStatus,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

/// HTTP status codes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HttpStatus {
    Ok = 200,
    Created = 201,
    NoContent = 204,
    MovedPermanently = 301,
    Found = 302,
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    Conflict = 409,
    InternalServerError = 500,
    NotImplemented = 501,
    ServiceUnavailable = 503,
}

impl HttpStatus {
    pub const fn code(&self) -> u16 {
        *self as u16
    }

    pub const fn reason_phrase(&self) -> &'static str {
        match self {
            HttpStatus::Ok => "OK",
            HttpStatus::Created => "Created",
            HttpStatus::NoContent => "No Content",
            HttpStatus::MovedPermanently => "Moved Permanently",
            HttpStatus::Found => "Found",
            HttpStatus::BadRequest => "Bad Request",
            HttpStatus::Unauthorized => "Unauthorized",
            HttpStatus::Forbidden => "Forbidden",
            HttpStatus::NotFound => "Not Found",
            HttpStatus::MethodNotAllowed => "Method Not Allowed",
            HttpStatus::Conflict => "Conflict",
            HttpStatus::InternalServerError => "Internal Server Error",
            HttpStatus::NotImplemented => "Not Implemented",
            HttpStatus::ServiceUnavailable => "Service Unavailable",
        }
    }
}

impl HttpResponse {
    pub fn new(status: HttpStatus) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Server".to_string(), "QMS/1.0.0".to_string());
        headers.insert("Content-Type".to_string(), "text/plain".to_string());
        headers.insert("Connection".to_string(), "close".to_string());
        
        // Add comprehensive security headers for medical device compliance
        Self::add_security_headers(&mut headers);

        Self {
            status,
            headers,
            body: Vec::new(),
        }
    }

    pub fn ok() -> Self {
        Self::new(HttpStatus::Ok)
    }

    pub fn ok_with_content(body: Vec<u8>, content_type: &str) -> Self {
        let mut response = Self::new(HttpStatus::Ok);
        response.set_body(body);
        response.set_content_type(content_type);
        response
    }

    pub fn not_found_empty() -> Self {
        Self::new(HttpStatus::NotFound)
    }

    pub fn not_found_with_message(message: &str) -> Self {
        let mut response = Self::new(HttpStatus::NotFound);
        response.set_body(message.as_bytes().to_vec());
        response.set_content_type("text/plain");
        response
    }

    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.add_header(name, value);
        self
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.set_body(body);
        self
    }

    pub fn json(json_string: &str) -> Self {
        let mut response = Self::new(HttpStatus::Ok);
        response.set_body(json_string.as_bytes().to_vec());
        response.set_content_type("application/json");
        response
    }

    pub fn html(html_string: &str) -> Self {
        let mut response = Self::new(HttpStatus::Ok);
        response.set_body(html_string.as_bytes().to_vec());
        response.set_content_type("text/html; charset=utf-8");
        response
    }

    pub fn created(body: Vec<u8>, content_type: &str) -> Self {
        let mut response = Self::new(HttpStatus::Created);
        response.set_body(body);
        response.set_content_type(content_type);
        response
    }

    /// Add comprehensive security headers for medical device compliance
    /// Implements FDA 21 CFR Part 820, ISO 13485, and ISO 14971 security requirements
    pub fn add_security_headers(headers: &mut HashMap<String, String>) {
        // Content Security Policy - Strict policy for medical device security
        headers.insert(
            "Content-Security-Policy".to_string(),
            "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; media-src 'self'; object-src 'none'; child-src 'none'; frame-src 'none'; worker-src 'self'; manifest-src 'self'".to_string()
        );

        // HTTP Strict Transport Security - Force HTTPS for medical device security
        headers.insert(
            "Strict-Transport-Security".to_string(),
            "max-age=31536000; includeSubDomains; preload".to_string()
        );

        // X-Content-Type-Options - Prevent MIME type sniffing
        headers.insert("X-Content-Type-Options".to_string(), "nosniff".to_string());

        // X-Frame-Options - Prevent clickjacking attacks
        headers.insert("X-Frame-Options".to_string(), "DENY".to_string());

        // X-XSS-Protection - Enable XSS filtering
        headers.insert("X-XSS-Protection".to_string(), "1; mode=block".to_string());

        // Referrer Policy - Control referrer information
        headers.insert("Referrer-Policy".to_string(), "strict-origin-when-cross-origin".to_string());

        // Permissions Policy - Control browser features
        headers.insert(
            "Permissions-Policy".to_string(),
            "geolocation=(), microphone=(), camera=(), payment=(), usb=(), magnetometer=(), gyroscope=(), accelerometer=()".to_string()
        );

        // Cache Control - Prevent caching of sensitive medical data
        headers.insert("Cache-Control".to_string(), "no-cache, no-store, must-revalidate, private".to_string());
        headers.insert("Pragma".to_string(), "no-cache".to_string());
        headers.insert("Expires".to_string(), "0".to_string());

        // Cross-Origin Policies for enhanced security (relaxed for static assets)
        headers.insert("Cross-Origin-Opener-Policy".to_string(), "same-origin".to_string());
        headers.insert("Cross-Origin-Resource-Policy".to_string(), "cross-origin".to_string());
        headers.insert("Cross-Origin-Embedder-Policy".to_string(), "require-corp".to_string());

        // X-Permitted-Cross-Domain-Policies - Restrict cross-domain policies
        headers.insert("X-Permitted-Cross-Domain-Policies".to_string(), "none".to_string());
    }

    /// Add security headers specifically for static assets
    pub fn add_static_asset_security_headers(&mut self) {
        // Allow caching for static assets but with security
        self.headers.insert("Cache-Control".to_string(), "public, max-age=3600, immutable".to_string());

        // Ensure security headers are still present
        self.headers.insert("X-Content-Type-Options".to_string(), "nosniff".to_string());
        self.headers.insert("X-Frame-Options".to_string(), "DENY".to_string());
    }

    /// Add security headers for API responses
    pub fn add_api_security_headers(&mut self) {
        // Strict no-cache for API responses containing medical data
        self.headers.insert("Cache-Control".to_string(), "no-cache, no-store, must-revalidate, private".to_string());
        self.headers.insert("Pragma".to_string(), "no-cache".to_string());
        self.headers.insert("Expires".to_string(), "0".to_string());

        // Additional API security
        self.headers.insert("X-Content-Type-Options".to_string(), "nosniff".to_string());
        self.headers.insert("X-Frame-Options".to_string(), "DENY".to_string());
    }

    /// Check if the response should be served over HTTPS only
    pub const fn requires_https(&self) -> bool {
        // Medical device data should always be served over HTTPS
        match self.status {
            HttpStatus::Ok | HttpStatus::Created | HttpStatus::NoContent => true,
            _ => false,
        }
    }

    pub fn no_content() -> Self {
        let mut response = Self::new(HttpStatus::NoContent);
        response.headers.remove("Content-Type");
        response
    }

    pub fn bad_request(message: &str) -> Self {
        let mut response = Self::new(HttpStatus::BadRequest);
        response.set_body(message.as_bytes().to_vec());
        response.set_content_type("text/plain");
        response
    }

    pub fn unauthorized(message: &str) -> Self {
        let mut response = Self::new(HttpStatus::Unauthorized);
        response.set_body(message.as_bytes().to_vec());
        response.set_content_type("text/plain");
        response.add_header("WWW-Authenticate", "Bearer");
        response
    }

    pub fn forbidden(message: &str) -> Self {
        let mut response = Self::new(HttpStatus::Forbidden);
        response.set_body(message.as_bytes().to_vec());
        response.set_content_type("text/plain");
        response
    }

    pub fn method_not_allowed(allowed_methods: &[&str]) -> Self {
        let mut response = Self::new(HttpStatus::MethodNotAllowed);
        response.set_body("Method Not Allowed".as_bytes().to_vec());
        response.set_content_type("text/plain");
        response.add_header("Allow", &allowed_methods.join(", "));
        response
    }

    pub fn internal_server_error(message: &str) -> Self {
        let mut response = Self::new(HttpStatus::InternalServerError);
        response.set_body(message.as_bytes().to_vec());
        response.set_content_type("text/plain");
        response
    }

    /// Alias for internal_server_error for backward compatibility
    pub fn internal_error(message: &str) -> Self {
        Self::internal_server_error(message)
    }

    /// Create OK response with JSON content
    pub fn ok_json(json_content: &str) -> Self {
        let mut response = Self::new(HttpStatus::Ok);
        response.set_body(json_content.as_bytes().to_vec());
        response.set_content_type("application/json");
        response
    }

    /// Create not found response with message
    pub fn not_found_message(message: &str) -> Self {
        let mut response = Self::new(HttpStatus::NotFound);
        response.set_body(message.as_bytes().to_vec());
        response.set_content_type("text/plain");
        response
    }

    /// Create OK response with string content and content type
    pub fn ok_with_string(content: &str, content_type: &str) -> Self {
        let mut response = Self::new(HttpStatus::Ok);
        response.set_body(content.as_bytes().to_vec());
        response.set_content_type(content_type);
        response
    }

    /// Create not found response with message
    pub fn not_found(message: &str) -> Self {
        let mut response = Self::new(HttpStatus::NotFound);
        response.set_body(message.as_bytes().to_vec());
        response.set_content_type("text/plain");
        response
    }

    /// Create created response with string content and content type
    pub fn created_with_string(content: &str, content_type: &str) -> Self {
        let mut response = Self::new(HttpStatus::Created);
        response.set_body(content.as_bytes().to_vec());
        response.set_content_type(content_type);
        response
    }

    /// Create method not allowed response with custom message
    pub fn method_not_allowed_with_message(message: &str) -> Self {
        let mut response = Self::new(HttpStatus::MethodNotAllowed);
        response.set_body(message.as_bytes().to_vec());
        response.set_content_type("text/plain");
        response
    }

    pub fn redirect(location: &str) -> Self {
        let mut response = Self::new(HttpStatus::Found);
        response.add_header("Location", location);
        response
    }

    pub fn set_body(&mut self, body: Vec<u8>) {
        self.body = body;
        self.set_content_length(self.body.len());
    }

    pub fn set_content_type(&mut self, content_type: &str) {
        self.headers.insert("Content-Type".to_string(), content_type.to_string());
    }

    pub fn set_content_length(&mut self, length: usize) {
        self.headers.insert("Content-Length".to_string(), length.to_string());
    }

    pub fn add_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_string(), value.to_string());
    }

    pub fn remove_header(&mut self, name: &str) {
        self.headers.remove(name);
    }

    pub fn set_cookie(&mut self, name: &str, value: &str, max_age: Option<u64>) {
        let mut cookie = format!("{name}={value}");
        
        // Security attributes for medical device compliance
        cookie.push_str("; HttpOnly");
        cookie.push_str("; Secure");
        cookie.push_str("; SameSite=Strict");
        cookie.push_str("; Path=/");
        
        if let Some(age) = max_age {
            cookie.push_str(&format!("; Max-Age={age}"));
        }

        self.headers.insert("Set-Cookie".to_string(), cookie);
    }

    pub fn clear_cookie(&mut self, name: &str) {
        let cookie = format!("{name}=; Max-Age=0; Path=/");
        self.headers.insert("Set-Cookie".to_string(), cookie);
    }

    pub fn enable_cors(&mut self) {
        self.add_header("Access-Control-Allow-Origin", "*");
        self.add_header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS");
        self.add_header("Access-Control-Allow-Headers", "Content-Type, Authorization");
        self.add_header("Access-Control-Max-Age", "86400");
    }

    pub fn set_cache_control(&mut self, directive: &str) {
        self.add_header("Cache-Control", directive);
    }

    pub fn set_etag(&mut self, etag: &str) {
        self.add_header("ETag", &format!("\"{etag}\""));
    }

    pub fn compress_gzip(&mut self) {
        // Placeholder for gzip compression
        // In a real implementation, you would compress self.body here
        // and add the Content-Encoding header
        if self.body.len() > 1024 {
            self.add_header("Content-Encoding", "gzip");
            // TODO: Implement actual gzip compression using stdlib
        }
    }

    pub fn get_body_as_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }

    pub fn is_success(&self) -> bool {
        let code = self.status.code();
        (200..300).contains(&code)
    }

    pub fn is_redirect(&self) -> bool {
        let code = self.status.code();
        (300..400).contains(&code)
    }

    pub fn is_client_error(&self) -> bool {
        let code = self.status.code();
        (400..500).contains(&code)
    }

    pub fn is_server_error(&self) -> bool {
        let code = self.status.code();
        (500..600).contains(&code)
    }
}

impl fmt::Display for HttpResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Status line
        write!(f, "HTTP/1.1 {} {}\r\n", self.status.code(), self.status.reason_phrase())?;

        // Headers
        for (name, value) in &self.headers {
            write!(f, "{name}: {value}\r\n")?;
        }

        // Empty line between headers and body
        write!(f, "\r\n")?;

        // Body (if text-like content type)
        if let Some(content_type) = self.headers.get("Content-Type") {
            if content_type.starts_with("text/") || 
               content_type.starts_with("application/json") ||
               content_type.starts_with("application/xml") {
                if let Ok(body_str) = String::from_utf8(self.body.clone()) {
                    write!(f, "{body_str}")?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_status_codes() {
        assert_eq!(HttpStatus::Ok.code(), 200);
        assert_eq!(HttpStatus::Created.code(), 201);
        assert_eq!(HttpStatus::BadRequest.code(), 400);
        assert_eq!(HttpStatus::NotFound.code(), 404);
        assert_eq!(HttpStatus::InternalServerError.code(), 500);
    }

    #[test]
    fn test_http_status_reason_phrases() {
        assert_eq!(HttpStatus::Ok.reason_phrase(), "OK");
        assert_eq!(HttpStatus::Created.reason_phrase(), "Created");
        assert_eq!(HttpStatus::BadRequest.reason_phrase(), "Bad Request");
        assert_eq!(HttpStatus::NotFound.reason_phrase(), "Not Found");
        assert_eq!(HttpStatus::InternalServerError.reason_phrase(), "Internal Server Error");
    }

    #[test]
    fn test_response_creation() {
        let response = HttpResponse::new(HttpStatus::Ok);
        assert_eq!(response.status.code(), 200);
        assert!(response.headers.contains_key("Server"));
        assert!(response.headers.contains_key("Content-Type"));
        assert!(response.headers.contains_key("X-Content-Type-Options"));
        assert!(response.body.is_empty());
    }

    #[test]
    fn test_ok_response() {
        let body = "Hello, World!".as_bytes().to_vec();
        let response = HttpResponse::ok_with_content(body.clone(), "text/plain");

        assert_eq!(response.status.code(), 200);
        assert_eq!(response.body, body);
        assert_eq!(response.headers.get("Content-Type"), Some(&"text/plain".to_string()));
        assert_eq!(response.headers.get("Content-Length"), Some(&"13".to_string()));
    }

    #[test]
    fn test_json_response() {
        let json = r#"{"message": "Hello, World!"}"#;
        let response = HttpResponse::json(json);
        
        assert_eq!(response.status.code(), 200);
        assert_eq!(response.get_body_as_string().unwrap(), json);
        assert_eq!(response.headers.get("Content-Type"), Some(&"application/json".to_string()));
    }

    #[test]
    fn test_error_responses() {
        let bad_request = HttpResponse::bad_request("Invalid input");
        assert_eq!(bad_request.status.code(), 400);
        assert_eq!(bad_request.get_body_as_string().unwrap(), "Invalid input");

        let not_found = HttpResponse::not_found_with_message("Resource not found");
        assert_eq!(not_found.status.code(), 404);
        assert_eq!(not_found.get_body_as_string().unwrap(), "Resource not found");

        let server_error = HttpResponse::internal_server_error("Server error");
        assert_eq!(server_error.status.code(), 500);
        assert_eq!(server_error.get_body_as_string().unwrap(), "Server error");
    }

    #[test]
    fn test_redirect_response() {
        let response = HttpResponse::redirect("/new-location");
        assert_eq!(response.status.code(), 302);
        assert_eq!(response.headers.get("Location"), Some(&"/new-location".to_string()));
    }

    #[test]
    fn test_cookie_handling() {
        let mut response = HttpResponse::new(HttpStatus::Ok);
        
        response.set_cookie("session", "abc123", Some(3600));
        let cookie = response.headers.get("Set-Cookie").unwrap();
        assert!(cookie.contains("session=abc123"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("Max-Age=3600"));

        response.clear_cookie("session");
        let clear_cookie = response.headers.get("Set-Cookie").unwrap();
        assert!(clear_cookie.contains("session="));
        assert!(clear_cookie.contains("Max-Age=0"));
    }

    #[test]
    fn test_cors_headers() {
        let mut response = HttpResponse::new(HttpStatus::Ok);
        response.enable_cors();
        
        assert_eq!(response.headers.get("Access-Control-Allow-Origin"), Some(&"*".to_string()));
        assert!(response.headers.contains_key("Access-Control-Allow-Methods"));
        assert!(response.headers.contains_key("Access-Control-Allow-Headers"));
    }

    #[test]
    fn test_response_status_checks() {
        assert!(HttpResponse::new(HttpStatus::Ok).is_success());
        assert!(!HttpResponse::new(HttpStatus::Ok).is_client_error());
        
        assert!(HttpResponse::new(HttpStatus::Found).is_redirect());
        assert!(!HttpResponse::new(HttpStatus::Found).is_success());
        
        assert!(HttpResponse::new(HttpStatus::BadRequest).is_client_error());
        assert!(!HttpResponse::new(HttpStatus::BadRequest).is_success());
        
        assert!(HttpResponse::new(HttpStatus::InternalServerError).is_server_error());
        assert!(!HttpResponse::new(HttpStatus::InternalServerError).is_success());
    }

    #[test]
    fn test_response_to_string() {
        let response = HttpResponse::ok_with_content("Hello".as_bytes().to_vec(), "text/plain");
        let response_str = response.to_string();

        assert!(response_str.starts_with("HTTP/1.1 200 OK"));
        assert!(response_str.contains("Content-Type: text/plain"));
        assert!(response_str.contains("Content-Length: 5"));
        assert!(response_str.ends_with("Hello"));
    }

    #[test]
    fn test_security_headers() {
        let response = HttpResponse::new(HttpStatus::Ok);

        // Test basic security headers
        assert_eq!(response.headers.get("X-Content-Type-Options"), Some(&"nosniff".to_string()));
        assert_eq!(response.headers.get("X-Frame-Options"), Some(&"DENY".to_string()));
        assert_eq!(response.headers.get("X-XSS-Protection"), Some(&"1; mode=block".to_string()));
        assert!(response.headers.contains_key("Cache-Control"));

        // Test enhanced security headers
        assert!(response.headers.contains_key("Content-Security-Policy"));
        assert!(response.headers.contains_key("Strict-Transport-Security"));
        assert!(response.headers.contains_key("Referrer-Policy"));
        assert!(response.headers.contains_key("Permissions-Policy"));
        assert!(response.headers.contains_key("Cross-Origin-Embedder-Policy"));
        assert!(response.headers.contains_key("Cross-Origin-Opener-Policy"));
        assert!(response.headers.contains_key("Cross-Origin-Resource-Policy"));
        assert!(response.headers.contains_key("X-Permitted-Cross-Domain-Policies"));
    }

    #[test]
    fn test_csp_header_content() {
        let response = HttpResponse::new(HttpStatus::Ok);
        let csp = response.headers.get("Content-Security-Policy").unwrap();

        assert!(csp.contains("default-src 'self'"));
        assert!(csp.contains("object-src 'none'"));
        assert!(csp.contains("frame-src 'none'"));
    }

    #[test]
    fn test_hsts_header_content() {
        let response = HttpResponse::new(HttpStatus::Ok);
        let hsts = response.headers.get("Strict-Transport-Security").unwrap();

        assert!(hsts.contains("max-age=31536000"));
        assert!(hsts.contains("includeSubDomains"));
        assert!(hsts.contains("preload"));
    }

    #[test]
    fn test_static_asset_security_headers() {
        let mut response = HttpResponse::new(HttpStatus::Ok);
        response.add_static_asset_security_headers();

        // Should allow caching for static assets
        let cache_control = response.headers.get("Cache-Control").unwrap();
        assert!(cache_control.contains("public"));
        assert!(cache_control.contains("max-age=3600"));

        // But still maintain security headers
        assert_eq!(response.headers.get("X-Content-Type-Options"), Some(&"nosniff".to_string()));
        assert_eq!(response.headers.get("X-Frame-Options"), Some(&"DENY".to_string()));
    }

    #[test]
    fn test_api_security_headers() {
        let mut response = HttpResponse::new(HttpStatus::Ok);
        response.add_api_security_headers();

        // Should have strict no-cache for API responses
        let cache_control = response.headers.get("Cache-Control").unwrap();
        assert!(cache_control.contains("no-cache"));
        assert!(cache_control.contains("no-store"));
        assert!(cache_control.contains("must-revalidate"));
        assert!(cache_control.contains("private"));
    }

    #[test]
    fn test_requires_https() {
        let ok_response = HttpResponse::new(HttpStatus::Ok);
        let created_response = HttpResponse::new(HttpStatus::Created);
        let not_found_response = HttpResponse::new(HttpStatus::NotFound);

        assert!(ok_response.requires_https());
        assert!(created_response.requires_https());
        assert!(!not_found_response.requires_https());
    }
}
