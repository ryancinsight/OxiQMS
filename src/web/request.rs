use std::collections::HashMap;

/// HTTP request representation
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub uri: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub query_params: HashMap<String, String>,
    pub timestamp: u64,
}

/// HTTP methods
#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    PATCH,
}

impl HttpMethod {
    pub fn from_str(method: &str) -> Option<HttpMethod> {
        match method.to_uppercase().as_str() {
            "GET" => Some(HttpMethod::GET),
            "POST" => Some(HttpMethod::POST),
            "PUT" => Some(HttpMethod::PUT),
            "DELETE" => Some(HttpMethod::DELETE),
            "HEAD" => Some(HttpMethod::HEAD),
            "OPTIONS" => Some(HttpMethod::OPTIONS),
            "PATCH" => Some(HttpMethod::PATCH),
            _ => None,
        }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::PATCH => "PATCH",
        }
    }
}

impl HttpRequest {
    pub fn new(method: String, uri: String) -> Self {
        Self {
            method,
            uri,
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
            query_params: HashMap::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn parse(request_data: &str) -> Result<Self, crate::error::QmsError> {
        let lines: Vec<&str> = request_data.lines().collect();
        if lines.is_empty() {
            return Err(crate::error::QmsError::validation_error("Empty request"));
        }

        // Parse request line (GET /path HTTP/1.1)
        let request_line_parts: Vec<&str> = lines[0].split_whitespace().collect();
        if request_line_parts.len() != 3 {
            return Err(crate::error::QmsError::validation_error("Invalid request line"));
        }

        let method = request_line_parts[0].to_string();
        let uri_with_query = request_line_parts[1];
        let version = request_line_parts[2].to_string();

        // Split URI and query parameters
        let (uri, query_string) = if let Some(pos) = uri_with_query.find('?') {
            (uri_with_query[..pos].to_string(), Some(&uri_with_query[pos + 1..]))
        } else {
            (uri_with_query.to_string(), None)
        };

        // Parse query parameters
        let mut query_params = HashMap::new();
        if let Some(query) = query_string {
            for param in query.split('&') {
                if let Some(eq_pos) = param.find('=') {
                    let key = param[..eq_pos].to_string();
                    let value = param[eq_pos + 1..].to_string();
                    query_params.insert(key, value);
                }
            }
        }

        // Parse headers
        let mut headers = HashMap::new();
        let mut i = 1;
        while i < lines.len() && !lines[i].is_empty() {
            if let Some(colon_pos) = lines[i].find(':') {
                let name = lines[i][..colon_pos].trim().to_lowercase();
                let value = lines[i][colon_pos + 1..].trim().to_string();
                headers.insert(name, value);
            }
            i += 1;
        }

        // Parse body (everything after the empty line)
        let body = if i + 1 < lines.len() {
            lines[i + 1..].join("\n").into_bytes()
        } else {
            Vec::new()
        };

        Ok(Self {
            method,
            uri,
            version,
            headers,
            body,
            query_params,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }

    pub fn path(&self) -> &str {
        &self.uri
    }

    pub fn get_method(&self) -> Option<HttpMethod> {
        HttpMethod::from_str(&self.method)
    }

    pub fn get_header(&self, name: &str) -> Option<&String> {
        self.headers.get(&name.to_lowercase())
    }

    pub fn get_query_param(&self, name: &str) -> Option<&String> {
        self.query_params.get(name)
    }

    pub fn get_body_as_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }

    pub fn get_content_length(&self) -> usize {
        self.get_header("content-length")
            .and_then(|len| len.parse().ok())
            .unwrap_or(0)
    }

    pub fn get_content_type(&self) -> Option<&String> {
        self.get_header("content-type")
    }

    pub fn get_user_agent(&self) -> Option<&String> {
        self.get_header("user-agent")
    }

    pub fn get_authorization(&self) -> Option<&String> {
        self.get_header("authorization")
    }

    pub fn get_session_cookie(&self) -> Option<String> {
        if let Some(cookie_header) = self.get_header("cookie") {
            for cookie in cookie_header.split(';') {
                let cookie = cookie.trim();
                if cookie.starts_with("qms_session=") {
                    return Some(cookie[12..].to_string());
                }
            }
        }
        None
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

    pub fn is_multipart(&self) -> bool {
        self.get_content_type()
            .map(|ct| ct.contains("multipart/form-data"))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method_parsing() {
        assert_eq!(HttpMethod::from_str("GET"), Some(HttpMethod::GET));
        assert_eq!(HttpMethod::from_str("post"), Some(HttpMethod::POST));
        assert_eq!(HttpMethod::from_str("PUT"), Some(HttpMethod::PUT));
        assert_eq!(HttpMethod::from_str("delete"), Some(HttpMethod::DELETE));
        assert_eq!(HttpMethod::from_str("INVALID"), None);
    }

    #[test]
    fn test_http_method_as_str() {
        assert_eq!(HttpMethod::GET.as_str(), "GET");
        assert_eq!(HttpMethod::POST.as_str(), "POST");
        assert_eq!(HttpMethod::PUT.as_str(), "PUT");
        assert_eq!(HttpMethod::DELETE.as_str(), "DELETE");
    }

    #[test]
    fn test_http_request_creation() {
        let request = HttpRequest::new("GET".to_string(), "/api/docs".to_string());
        assert_eq!(request.method, "GET");
        assert_eq!(request.uri, "/api/docs");
        assert_eq!(request.version, "HTTP/1.1");
        assert!(request.headers.is_empty());
        assert!(request.body.is_empty());
    }

    #[test]
    fn test_header_access() {
        let mut request = HttpRequest::new("GET".to_string(), "/".to_string());
        request.headers.insert("content-type".to_string(), "application/json".to_string());
        request.headers.insert("user-agent".to_string(), "QMS/1.0".to_string());

        assert_eq!(request.get_header("content-type"), Some(&"application/json".to_string()));
        assert_eq!(request.get_header("CONTENT-TYPE"), Some(&"application/json".to_string()));
        assert_eq!(request.get_header("user-agent"), Some(&"QMS/1.0".to_string()));
        assert_eq!(request.get_header("missing"), None);
    }

    #[test]
    fn test_content_type_detection() {
        let mut request = HttpRequest::new("POST".to_string(), "/api/data".to_string());
        
        request.headers.insert("content-type".to_string(), "application/json".to_string());
        assert!(request.is_json());
        assert!(!request.is_form_data());

        request.headers.insert("content-type".to_string(), "application/x-www-form-urlencoded".to_string());
        assert!(!request.is_json());
        assert!(request.is_form_data());

        request.headers.insert("content-type".to_string(), "multipart/form-data; boundary=something".to_string());
        assert!(request.is_multipart());
    }

    #[test]
    fn test_session_cookie_extraction() {
        let mut request = HttpRequest::new("GET".to_string(), "/".to_string());
        
        // No cookie header
        assert_eq!(request.get_session_cookie(), None);
        
        // Cookie header with session
        request.headers.insert("cookie".to_string(), "qms_session=abc123; other=value".to_string());
        assert_eq!(request.get_session_cookie(), Some("abc123".to_string()));
        
        // Cookie header without session
        request.headers.insert("cookie".to_string(), "other=value; another=test".to_string());
        assert_eq!(request.get_session_cookie(), None);
    }

    #[test]
    fn test_body_handling() {
        let mut request = HttpRequest::new("POST".to_string(), "/api/data".to_string());
        request.body = "test data".as_bytes().to_vec();
        
        assert_eq!(request.get_body_as_string().unwrap(), "test data");
        assert_eq!(request.body.len(), 9);
    }
}
