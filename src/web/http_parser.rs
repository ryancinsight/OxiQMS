// HTTP Request/Response Parser - Phase 7.1.2
// Medical Device QMS - FDA 21 CFR Part 820, ISO 13485, ISO 14971 Compliant
// Uses stdlib only - no external dependencies

use std::collections::HashMap;
use std::io::Read;
use std::str::FromStr;
use crate::prelude::*;

/// HTTP method enumeration supporting common REST API operations
#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    OPTIONS,
    HEAD,
    PATCH,
}

impl HttpMethod {
    pub const fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::PATCH => "PATCH",
        }
    }
}

impl FromStr for HttpMethod {
    type Err = QmsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "DELETE" => Ok(HttpMethod::DELETE),
            "OPTIONS" => Ok(HttpMethod::OPTIONS),
            "HEAD" => Ok(HttpMethod::HEAD),
            "PATCH" => Ok(HttpMethod::PATCH),
            _ => Err(QmsError::parse_error(&format!("Invalid HTTP method: {s}"))),
        }
    }
}

/// HTTP status codes for medical device API responses
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HttpStatus {
    Ok = 200,
    Created = 201,
    NoContent = 204,
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    Conflict = 409,
    UnprocessableEntity = 422,
    InternalServerError = 500,
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
            HttpStatus::BadRequest => "Bad Request",
            HttpStatus::Unauthorized => "Unauthorized",
            HttpStatus::Forbidden => "Forbidden",
            HttpStatus::NotFound => "Not Found",
            HttpStatus::MethodNotAllowed => "Method Not Allowed",
            HttpStatus::Conflict => "Conflict",
            HttpStatus::UnprocessableEntity => "Unprocessable Entity",
            HttpStatus::InternalServerError => "Internal Server Error",
            HttpStatus::ServiceUnavailable => "Service Unavailable",
        }
    }
}

/// HTTP request structure for medical device API
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub body: Vec<u8>,
    pub content_length: usize,
}

impl HttpRequest {
    pub fn new(method: HttpMethod, path: String) -> Self {
        Self {
            method,
            path,
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            query_params: HashMap::new(),
            body: Vec::new(),
            content_length: 0,
        }
    }

    pub fn get_header(&self, name: &str) -> Option<&String> {
        self.headers.get(&name.to_lowercase())
    }

    pub fn get_query_param(&self, name: &str) -> Option<&String> {
        self.query_params.get(name)
    }

    pub fn get_content_type(&self) -> Option<&String> {
        self.get_header("content-type")
    }

    pub fn body_as_string(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }

    pub fn is_json(&self) -> bool {
        self.get_content_type()
            .map(|ct| ct.contains("application/json"))
            .unwrap_or(false)
    }

    pub fn is_form_data(&self) -> bool {
        self.get_content_type()
            .map(|ct| ct.contains("application/x-www-form-urlencoded"))
            .unwrap_or(false)
    }
}

/// HTTP response structure for medical device API
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: HttpStatus,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn new(status: HttpStatus) -> Self {
        let mut response = Self {
            status,
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
        };
        
        // Add default security headers for medical device compliance
        response.add_security_headers();
        response
    }

    pub fn ok() -> Self {
        Self::new(HttpStatus::Ok)
    }

    pub fn created() -> Self {
        Self::new(HttpStatus::Created)
    }

    pub fn bad_request() -> Self {
        Self::new(HttpStatus::BadRequest)
    }

    pub fn not_found() -> Self {
        Self::new(HttpStatus::NotFound)
    }

    pub fn internal_server_error() -> Self {
        Self::new(HttpStatus::InternalServerError)
    }

    pub fn set_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_string(), value.to_string());
    }

    pub fn set_json_body(&mut self, json: &str) {
        self.set_header("Content-Type", "application/json; charset=utf-8");
        self.body = json.as_bytes().to_vec();
        self.set_header("Content-Length", &self.body.len().to_string());
    }

    pub fn set_text_body(&mut self, text: &str) {
        self.set_header("Content-Type", "text/plain; charset=utf-8");
        self.body = text.as_bytes().to_vec();
        self.set_header("Content-Length", &self.body.len().to_string());
    }

    pub fn set_html_body(&mut self, html: &str) {
        self.set_header("Content-Type", "text/html; charset=utf-8");
        self.body = html.as_bytes().to_vec();
        self.set_header("Content-Length", &self.body.len().to_string());
    }

    /// Add security headers for medical device compliance
    fn add_security_headers(&mut self) {
        // Content Security Policy for medical device security
        self.set_header("Content-Security-Policy", 
            "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'");
        
        // Security headers per FDA guidance
        self.set_header("X-Content-Type-Options", "nosniff");
        self.set_header("X-Frame-Options", "DENY");
        self.set_header("X-XSS-Protection", "1; mode=block");
        self.set_header("Strict-Transport-Security", "max-age=31536000; includeSubDomains");
        self.set_header("Referrer-Policy", "strict-origin-when-cross-origin");
        self.set_header("Permissions-Policy", "geolocation=(), microphone=(), camera=()");
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut response = Vec::new();
        
        // Status line
        let status_line = format!("{} {} {}\r\n", 
            self.version, self.status.code(), self.status.reason_phrase());
        response.extend_from_slice(status_line.as_bytes());
        
        // Headers
        for (name, value) in &self.headers {
            let header_line = format!("{name}: {value}\r\n");
            response.extend_from_slice(header_line.as_bytes());
        }
        
        // Empty line before body
        response.extend_from_slice(b"\r\n");
        
        // Body
        response.extend_from_slice(&self.body);
        
        response
    }
}

/// HTTP request parser configuration for medical device security
#[derive(Debug, Clone)]
pub struct ParserConfig {
    pub max_request_size: usize,
    pub max_header_count: usize,
    pub max_header_size: usize,
    pub max_uri_length: usize,
    pub timeout_seconds: u64,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_request_size: 1024 * 1024, // 1MB max request
            max_header_count: 100,         // Max 100 headers
            max_header_size: 8192,         // 8KB max header
            max_uri_length: 2048,          // 2KB max URI
            timeout_seconds: 30,           // 30 second timeout
        }
    }
}

/// HTTP request parser for medical device API server
pub struct HttpParser {
    config: ParserConfig,
}

impl Default for HttpParser {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpParser {
    pub fn new() -> Self {
        Self {
            config: ParserConfig::default(),
        }
    }

    pub const fn with_config(config: ParserConfig) -> Self {
        Self { config }
    }

    /// Parse HTTP request from reader with medical device security validation
    pub fn parse_request<R: Read>(&self, mut reader: R) -> QmsResult<HttpRequest> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        
        if buffer.len() > self.config.max_request_size {
            return Err(QmsError::validation_error("Request size exceeds maximum limit"));
        }

        let request_str = String::from_utf8_lossy(&buffer);
        let mut lines = request_str.lines();
        
        // Parse request line
        let request_line = lines.next()
            .ok_or_else(|| QmsError::parse_error("Missing request line"))?;
        
        let (method, path, version) = self.parse_request_line(request_line)?;
        
        // Parse headers
        let mut headers = HashMap::new();
        let mut header_count = 0;
        let mut body_start = 0;
        
        for line in lines {
            if line.is_empty() {
                body_start = request_str.find("\r\n\r\n")
                    .or_else(|| request_str.find("\n\n"))
                    .map(|pos| pos + if request_str.contains("\r\n\r\n") { 4 } else { 2 })
                    .unwrap_or(buffer.len());
                break;
            }
            
            header_count += 1;
            if header_count > self.config.max_header_count {
                return Err(QmsError::validation_error("Too many headers"));
            }
            
            if line.len() > self.config.max_header_size {
                return Err(QmsError::validation_error("Header size exceeds maximum"));
            }
            
            let (name, value) = self.parse_header(line)?;
            headers.insert(name.to_lowercase(), value);
        }
        
        // Parse query parameters
        let (path_clean, query_params) = self.parse_query_string(&path)?;
        
        // Extract body
        let body = if body_start < buffer.len() {
            buffer[body_start..].to_vec()
        } else {
            Vec::new()
        };
        
        // Validate content length
        let content_length = headers.get("content-length")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(body.len());
        
        if content_length != body.len() {
            return Err(QmsError::validation_error("Content-Length mismatch"));
        }
        
        Ok(HttpRequest {
            method,
            path: path_clean,
            version,
            headers,
            query_params,
            body,
            content_length,
        })
    }

    /// Parse request line (GET /path HTTP/1.1)
    fn parse_request_line(&self, line: &str) -> QmsResult<(HttpMethod, String, String)> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(QmsError::parse_error("Invalid request line format"));
        }
        
        let method = HttpMethod::from_str(parts[0])?;
        let path = parts[1].to_string();
        let version = parts[2].to_string();
        
        if path.len() > self.config.max_uri_length {
            return Err(QmsError::validation_error("URI length exceeds maximum"));
        }
        
        if !version.starts_with("HTTP/") {
            return Err(QmsError::parse_error("Invalid HTTP version"));
        }
        
        Ok((method, path, version))
    }

    /// Parse HTTP header (Name: Value)
    fn parse_header(&self, line: &str) -> QmsResult<(String, String)> {
        let colon_pos = line.find(':')
            .ok_or_else(|| QmsError::parse_error("Invalid header format"))?;
        
        let name = line[..colon_pos].trim().to_string();
        let value = line[colon_pos + 1..].trim().to_string();
        
        if name.is_empty() {
            return Err(QmsError::parse_error("Empty header name"));
        }
        
        Ok((name, value))
    }

    /// Parse query string (?key=value&key2=value2)
    fn parse_query_string(&self, path: &str) -> QmsResult<(String, HashMap<String, String>)> {
        let mut query_params = HashMap::new();
        
        if let Some(query_start) = path.find('?') {
            let path_clean = path[..query_start].to_string();
            let query_string = &path[query_start + 1..];
            
            for pair in query_string.split('&') {
                if let Some(eq_pos) = pair.find('=') {
                    let key = self.url_decode(&pair[..eq_pos])?;
                    let value = self.url_decode(&pair[eq_pos + 1..])?;
                    query_params.insert(key, value);
                } else if !pair.is_empty() {
                    let key = self.url_decode(pair)?;
                    query_params.insert(key, String::new());
                }
            }
            
            Ok((path_clean, query_params))
        } else {
            Ok((path.to_string(), query_params))
        }
    }

    /// Simple URL decoding for query parameters
    fn url_decode(&self, s: &str) -> QmsResult<String> {
        let mut result = String::new();
        let mut chars = s.chars();
        
        while let Some(c) = chars.next() {
            match c {
                '%' => {
                    let hex1 = chars.next()
                        .ok_or_else(|| QmsError::parse_error("Incomplete URL encoding"))?;
                    let hex2 = chars.next()
                        .ok_or_else(|| QmsError::parse_error("Incomplete URL encoding"))?;
                    
                    let hex_str = format!("{hex1}{hex2}");
                    let byte = u8::from_str_radix(&hex_str, 16)
                        .map_err(|_| QmsError::parse_error("Invalid URL encoding"))?;
                    
                    result.push(byte as char);
                },
                '+' => result.push(' '),
                _ => result.push(c),
            }
        }
        
        Ok(result)
    }

    /// Parse form data from request body
    pub fn parse_form_data(&self, request: &HttpRequest) -> QmsResult<HashMap<String, String>> {
        if !request.is_form_data() {
            return Err(QmsError::validation_error("Request is not form data"));
        }
        
        let body_str = request.body_as_string();
        let mut form_data = HashMap::new();
        
        for pair in body_str.split('&') {
            if let Some(eq_pos) = pair.find('=') {
                let key = self.url_decode(&pair[..eq_pos])?;
                let value = self.url_decode(&pair[eq_pos + 1..])?;
                form_data.insert(key, value);
            } else if !pair.is_empty() {
                let key = self.url_decode(pair)?;
                form_data.insert(key, String::new());
            }
        }
        
        Ok(form_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_http_method_parsing() {
        assert_eq!(HttpMethod::from_str("GET").unwrap(), HttpMethod::GET);
        assert_eq!(HttpMethod::from_str("post").unwrap(), HttpMethod::POST);
        assert_eq!(HttpMethod::from_str("PUT").unwrap(), HttpMethod::PUT);
        assert!(HttpMethod::from_str("INVALID").is_err());
    }

    #[test]
    fn test_http_method_as_str() {
        assert_eq!(HttpMethod::GET.as_str(), "GET");
        assert_eq!(HttpMethod::POST.as_str(), "POST");
        assert_eq!(HttpMethod::DELETE.as_str(), "DELETE");
    }

    #[test]
    fn test_http_status_codes() {
        assert_eq!(HttpStatus::Ok.code(), 200);
        assert_eq!(HttpStatus::NotFound.code(), 404);
        assert_eq!(HttpStatus::InternalServerError.code(), 500);
        
        assert_eq!(HttpStatus::Ok.reason_phrase(), "OK");
        assert_eq!(HttpStatus::NotFound.reason_phrase(), "Not Found");
        assert_eq!(HttpStatus::InternalServerError.reason_phrase(), "Internal Server Error");
    }

    #[test]
    fn test_http_request_creation() {
        let request = HttpRequest::new(HttpMethod::GET, "/api/health".to_string());
        assert_eq!(request.method, HttpMethod::GET);
        assert_eq!(request.path, "/api/health");
        assert_eq!(request.version, "HTTP/1.1");
        assert!(request.headers.is_empty());
        assert!(request.query_params.is_empty());
        assert!(request.body.is_empty());
    }

    #[test]
    fn test_http_response_creation() {
        let mut response = HttpResponse::ok();
        assert_eq!(response.status, HttpStatus::Ok);
        assert_eq!(response.version, "HTTP/1.1");
        assert!(!response.headers.is_empty()); // Should have security headers
        
        response.set_json_body(r#"{"status": "ok"}"#);
        assert_eq!(response.headers.get("Content-Type").unwrap(), "application/json; charset=utf-8");
        assert_eq!(response.headers.get("Content-Length").unwrap(), "16");
    }

    #[test]
    fn test_security_headers() {
        let response = HttpResponse::ok();
        assert!(response.headers.contains_key("Content-Security-Policy"));
        assert!(response.headers.contains_key("X-Content-Type-Options"));
        assert!(response.headers.contains_key("X-Frame-Options"));
        assert!(response.headers.contains_key("X-XSS-Protection"));
        assert!(response.headers.contains_key("Strict-Transport-Security"));
    }

    #[test]
    fn test_parse_simple_get_request() {
        let parser = HttpParser::new();
        let request_data = "GET /api/health HTTP/1.1\r\nHost: localhost:8080\r\nUser-Agent: QMS-Client/1.0\r\n\r\n";
        let cursor = Cursor::new(request_data.as_bytes());
        
        let request = parser.parse_request(cursor).unwrap();
        assert_eq!(request.method, HttpMethod::GET);
        assert_eq!(request.path, "/api/health");
        assert_eq!(request.version, "HTTP/1.1");
        assert_eq!(request.headers.get("host").unwrap(), "localhost:8080");
        assert_eq!(request.headers.get("user-agent").unwrap(), "QMS-Client/1.0");
    }

    #[test]
    fn test_parse_request_with_query_params() {
        let parser = HttpParser::new();
        let request_data = "GET /api/documents?status=active&limit=10 HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let cursor = Cursor::new(request_data.as_bytes());
        
        let request = parser.parse_request(cursor).unwrap();
        assert_eq!(request.path, "/api/documents");
        assert_eq!(request.query_params.get("status").unwrap(), "active");
        assert_eq!(request.query_params.get("limit").unwrap(), "10");
    }

    #[test]
    fn test_parse_post_request_with_body() {
        let parser = HttpParser::new();
        let body = r#"{"title": "Test Document", "content": "Sample content"}"#;
        let request_data = format!(
            "POST /api/documents HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(), body
        );
        let cursor = Cursor::new(request_data.as_bytes());
        
        let request = parser.parse_request(cursor).unwrap();
        assert_eq!(request.method, HttpMethod::POST);
        assert_eq!(request.path, "/api/documents");
        assert!(request.is_json());
        assert_eq!(request.body_as_string(), body);
        assert_eq!(request.content_length, body.len());
    }

    #[test]
    fn test_parse_form_data() {
        let parser = HttpParser::new();
        let mut request = HttpRequest::new(HttpMethod::POST, "/submit".to_string());
        request.headers.insert("content-type".to_string(), "application/x-www-form-urlencoded".to_string());
        request.body = "name=John+Doe&email=john%40example.com&age=30".as_bytes().to_vec();
        
        let form_data = parser.parse_form_data(&request).unwrap();
        assert_eq!(form_data.get("name").unwrap(), "John Doe");
        assert_eq!(form_data.get("email").unwrap(), "john@example.com");
        assert_eq!(form_data.get("age").unwrap(), "30");
    }

    #[test]
    fn test_url_decoding() {
        let parser = HttpParser::new();
        assert_eq!(parser.url_decode("hello%20world").unwrap(), "hello world");
        assert_eq!(parser.url_decode("user%40example.com").unwrap(), "user@example.com");
        assert_eq!(parser.url_decode("test+string").unwrap(), "test string");
    }

    #[test]
    fn test_request_size_validation() {
        let mut config = ParserConfig::default();
        config.max_request_size = 50; // Very small limit for testing
        let parser = HttpParser::with_config(config);
        
        let large_request = "GET / HTTP/1.1\r\n".repeat(10);
        let cursor = Cursor::new(large_request.as_bytes());
        
        assert!(parser.parse_request(cursor).is_err());
    }

    #[test]
    fn test_invalid_request_line() {
        let parser = HttpParser::new();
        let request_data = "INVALID REQUEST\r\n\r\n";
        let cursor = Cursor::new(request_data.as_bytes());
        
        assert!(parser.parse_request(cursor).is_err());
    }

    #[test]
    fn test_response_to_bytes() {
        let mut response = HttpResponse::ok();
        response.set_json_body(r#"{"message": "Success"}"#);
        
        let bytes = response.to_bytes();
        let response_str = String::from_utf8_lossy(&bytes);
        
        assert!(response_str.contains("HTTP/1.1 200 OK"));
        assert!(response_str.contains("Content-Type: application/json"));
        assert!(response_str.contains(r#"{"message": "Success"}"#));
    }
}
