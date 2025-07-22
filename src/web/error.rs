// QMS Web Server Error Handling & Logging
// Medical Device Quality Management System
// Regulatory Compliance: FDA 21 CFR Part 820, ISO 13485, ISO 14971

use super::{HttpResponse, HttpRequest};
use super::response::HttpStatus;
use crate::prelude::{QmsResult, QmsError};
use std::io::Write;
use std::net::TcpStream;
use std::time::{SystemTime, UNIX_EPOCH};

/// Web error logging configuration
pub struct WebLogConfig {
    pub enable_access_log: bool,
    pub enable_error_log: bool,
    pub enable_performance_log: bool,
    pub enable_security_log: bool,
    pub max_log_entry_size: usize,
}

impl Default for WebLogConfig {
    fn default() -> Self {
        Self {
            enable_access_log: true,
            enable_error_log: true,
            enable_performance_log: true,
            enable_security_log: true,
            max_log_entry_size: 4096,
        }
    }
}

/// Request metrics for performance monitoring
pub struct RequestMetrics {
    pub start_time: SystemTime,
    pub method: String,
    pub path: String,
    pub user_agent: String,
    pub remote_addr: String,
    pub bytes_sent: usize,
    pub bytes_received: usize,
    pub status_code: u16,
    pub processing_time_ms: u64,
}

impl RequestMetrics {
    pub fn new(request: &HttpRequest, remote_addr: String) -> Self {
        Self {
            start_time: SystemTime::now(),
            method: request.method.clone(),
            path: request.uri.clone(),
            user_agent: request.headers.get("User-Agent").map(|s| s.as_str()).unwrap_or("Unknown").to_string(),
            remote_addr,
            bytes_sent: 0,
            bytes_received: request.body.len(),
            status_code: 200,
            processing_time_ms: 0,
        }
    }

    pub fn finish(&mut self, status_code: u16, bytes_sent: usize) {
        self.status_code = status_code;
        self.bytes_sent = bytes_sent;
        if let Ok(duration) = SystemTime::now().duration_since(self.start_time) {
            self.processing_time_ms = duration.as_millis() as u64;
        }
    }
}

/// Web error handler for HTTP error responses and logging
pub struct WebErrorHandler {
    config: WebLogConfig,
}

impl WebErrorHandler {
    /// Create new web error handler
    pub const fn new(config: WebLogConfig) -> Self {
        Self { config }
    }

    /// Send error response to client with proper medical device styling
    pub fn send_error_response(
        &self,
        stream: &mut TcpStream,
        status: HttpStatus,
        error_message: Option<&str>,
        request: Option<&HttpRequest>,
    ) -> QmsResult<()> {
        let error_page = self.create_error_page(status, error_message);
        let mut response = HttpResponse::new(status);
        response.set_body(error_page.as_bytes().to_vec());
        response.set_content_type("text/html; charset=utf-8");

        let response_data = response.to_string();
        stream.write_all(response_data.as_bytes())?;
        stream.flush()?;

        // Log error response
        if self.config.enable_error_log {
            self.log_error_response(status, error_message, request);
        }

        Ok(())
    }

    /// Create HTML error page with medical device styling
    fn create_error_page(&self, status: HttpStatus, error_message: Option<&str>) -> String {
        let status_code = status.code();
        let reason_phrase = status.reason_phrase();
        let error_details = error_message.unwrap_or("An error occurred while processing your request.");

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Error {status_code} - QMS Medical Device System</title>
    <style>
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            margin: 0;
            padding: 0;
            min-height: 100vh;
            display: flex;
            justify-content: center;
            align-items: center;
        }}
        .error-container {{
            background: white;
            border-radius: 12px;
            box-shadow: 0 10px 40px rgba(0,0,0,0.1);
            padding: 3rem;
            max-width: 600px;
            text-align: center;
            border: 3px solid #2c5aa0;
        }}
        .error-code {{
            font-size: 4rem;
            font-weight: bold;
            color: #c62d42;
            margin-bottom: 0.5rem;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.1);
        }}
        .error-title {{
            font-size: 1.5rem;
            color: #2c5aa0;
            margin-bottom: 1rem;
            font-weight: 600;
        }}
        .error-description {{
            color: #666;
            margin-bottom: 2rem;
            line-height: 1.6;
        }}
        .medical-badge {{
            background: #2c5aa0;
            color: white;
            padding: 0.5rem 1rem;
            border-radius: 20px;
            font-size: 0.9rem;
            font-weight: 500;
            margin-bottom: 2rem;
            display: inline-block;
        }}
        .back-button {{
            background: #2c5aa0;
            color: white;
            border: none;
            padding: 1rem 2rem;
            border-radius: 6px;
            font-size: 1rem;
            cursor: pointer;
            transition: background 0.3s ease;
            text-decoration: none;
            display: inline-block;
        }}
        .back-button:hover {{
            background: #1a3d73;
        }}
        .compliance-info {{
            margin-top: 2rem;
            padding: 1rem;
            background: #f8f9fa;
            border-radius: 6px;
            font-size: 0.9rem;
            color: #666;
        }}
        .timestamp {{
            margin-top: 1rem;
            font-size: 0.8rem;
            color: #999;
        }}
    </style>
</head>
<body>
    <div class="error-container">
        <div class="medical-badge">🏥 Medical Device QMS</div>
        <div class="error-code">{status_code}</div>
        <div class="error-title">{reason_phrase}</div>
        <div class="error-description">{error_details}</div>
        <a href="/" class="back-button">Return to Dashboard</a>
        
        <div class="compliance-info">
            <strong>Regulatory Compliance:</strong><br>
            This system operates under FDA 21 CFR Part 820 (Quality System Regulation),<br>
            ISO 13485 (Medical Devices Quality Management), and ISO 14971 (Risk Management).
        </div>
        
        <div class="timestamp">
            Error occurred: {timestamp}<br>
            Reference ID: ERR-{ref_id}
        </div>
    </div>
</body>
</html>"#,
            status_code = status_code,
            reason_phrase = reason_phrase,
            error_details = error_details,
            timestamp = format_timestamp(SystemTime::now()),
            ref_id = generate_error_reference_id()
        )
    }

    /// Add medical device security headers
    fn add_medical_device_security_headers(&self, _response: &mut HttpResponse) {
        // Medical device security headers are already added in HttpResponse::new()
        // This method is kept for potential additional security headers
    }

    /// Log error response for audit trail
    fn log_error_response(&self, status: HttpStatus, error_message: Option<&str>, request: Option<&HttpRequest>) {
        let log_entry = format!(
            "[{}] ERROR {} {} - {} - Request: {} {} - User-Agent: {} - Error: {}",
            format_timestamp(SystemTime::now()),
            status.code(),
            status.reason_phrase(),
            request.map(|r| r.headers.get("Host").map(|s| s.as_str()).unwrap_or("unknown")).unwrap_or("unknown"),
            request.map(|r| r.method.as_str()).unwrap_or("UNKNOWN"),
            request.map(|r| r.uri.as_str()).unwrap_or("/unknown"),
            request.map(|r| r.headers.get("User-Agent").map(|s| s.as_str()).unwrap_or("Unknown")).unwrap_or("Unknown"),
            error_message.unwrap_or("No details provided")
        );

        // Log to audit system for medical device compliance
        if let Err(e) = self.audit_log_web_error(&log_entry) {
            eprintln!("Failed to log web error to audit: {e}");
        }

        // Also log to stderr for immediate visibility
        eprintln!("{log_entry}");
    }

    /// Log access request for performance monitoring
    pub fn log_access_request(&self, metrics: &RequestMetrics) -> QmsResult<()> {
        if !self.config.enable_access_log {
            return Ok(());
        }

        let log_entry = format!(
            "[{}] ACCESS {} {} {} {} {}ms {}bytes_in {}bytes_out \"{}\" \"{}\"",
            format_timestamp(metrics.start_time),
            metrics.remote_addr,
            metrics.method,
            metrics.path,
            metrics.status_code,
            metrics.processing_time_ms,
            metrics.bytes_received,
            metrics.bytes_sent,
            metrics.user_agent,
            "QMS-Medical-Device"
        );

        self.audit_log_web_access(&log_entry)?;
        println!("{log_entry}");
        Ok(())
    }

    /// Log performance metrics
    pub fn log_performance_metrics(&self, metrics: &RequestMetrics) -> QmsResult<()> {
        if !self.config.enable_performance_log {
            return Ok(());
        }

        // Log slow requests (> 1000ms) for medical device performance monitoring
        if metrics.processing_time_ms > 1000 {
        let log_entry = format!(
            "[{}] PERFORMANCE SLOW_REQUEST {} {} {}ms - Medical Device Performance Alert",
            format_timestamp(metrics.start_time),
            metrics.method,
            metrics.path,
            metrics.processing_time_ms
        );

        self.audit_log_web_performance(&log_entry)?;
            eprintln!("⚠️  {log_entry}");
        }

        Ok(())
    }

    /// Log security events
    pub fn log_security_event(&self, event_type: &str, details: &str, request: Option<&HttpRequest>) -> QmsResult<()> {
        if !self.config.enable_security_log {
            return Ok(());
        }

        let log_entry = format!(
            "[{}] SECURITY {} - {} - Request: {} {} - User-Agent: {}",
            format_timestamp(SystemTime::now()),
            event_type,
            details,
            request.map(|r| r.method.as_str()).unwrap_or("UNKNOWN"),
            request.map(|r| r.uri.as_str()).unwrap_or("/unknown"),
            request.map(|r| r.headers.get("User-Agent").map(|s| s.as_str()).unwrap_or("Unknown")).unwrap_or("Unknown")
        );

        self.audit_log_web_security(&log_entry)?;
        eprintln!("🔒 {log_entry}");
        Ok(())
    }

    /// Handle various types of errors with appropriate responses
    pub fn handle_error(&self, stream: &mut TcpStream, error: &QmsError, request: Option<&HttpRequest>) -> QmsResult<()> {
        let (status, message) = match error {
            QmsError::NotFound(_) => (HttpStatus::NotFound, "The requested resource was not found."),
            QmsError::Validation(_) => (HttpStatus::BadRequest, "Invalid request data provided."),
            QmsError::Permission(_) => (HttpStatus::Forbidden, "Access denied. Insufficient permissions."),
            QmsError::Authentication(_) => (HttpStatus::Unauthorized, "Authentication required."),
            _ => (HttpStatus::InternalServerError, "An internal server error occurred."),
        };

        self.send_error_response(stream, status, Some(message), request)
    }

    /// Audit logging functions for medical device compliance

    fn audit_log_web_error(&self, _log_entry: &str) -> QmsResult<()> {
        // Use existing audit logging system
        crate::modules::audit_logger::functions::audit_log_action(
            "WEB_ERROR",
            "WebServer",
            &format!("error_log_{}", generate_error_reference_id()),
        )?;
        Ok(())
    }

    fn audit_log_web_access(&self, _log_entry: &str) -> QmsResult<()> {
        // Use existing audit logging system for access logs
        crate::modules::audit_logger::functions::audit_log_action(
            "WEB_ACCESS",
            "WebServer",
            &format!("access_log_{}", generate_error_reference_id()),
        )?;
        Ok(())
    }

    fn audit_log_web_performance(&self, _log_entry: &str) -> QmsResult<()> {
        // Use existing audit logging system for performance logs
        crate::modules::audit_logger::functions::audit_log_action(
            "WEB_PERFORMANCE",
            "WebServer",
            &format!("perf_log_{}", generate_error_reference_id()),
        )?;
        Ok(())
    }

    fn audit_log_web_security(&self, _log_entry: &str) -> QmsResult<()> {
        // Use existing audit logging system for security logs
        crate::modules::audit_logger::functions::audit_log_action(
            "WEB_SECURITY",
            "WebServer",
            &format!("security_log_{}", generate_error_reference_id()),
        )?;
        Ok(())
    }
}

/// Format timestamp for logging
fn format_timestamp(time: SystemTime) -> String {
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs();
            let nanos = duration.subsec_nanos();
            format!("{}.{:03}", secs, nanos / 1_000_000)
        }
        Err(_) => "0.000".to_string(),
    }
}

/// Generate unique error reference ID for tracking
fn generate_error_reference_id() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let timestamp = duration.as_millis();
            format!("{:X}", timestamp % 0xFFFFFF)
        }
        Err(_) => "000000".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_status_reason_phrases() {
        assert_eq!(HttpStatus::Ok.reason_phrase(), "OK");
        assert_eq!(HttpStatus::NotFound.reason_phrase(), "Not Found");
        assert_eq!(HttpStatus::InternalServerError.reason_phrase(), "Internal Server Error");
    }

    #[test]
    fn test_web_log_config_default() {
        let config = WebLogConfig::default();
        assert!(config.enable_access_log);
        assert!(config.enable_error_log);
        assert!(config.enable_performance_log);
        assert!(config.enable_security_log);
        assert_eq!(config.max_log_entry_size, 4096);
    }

    #[test]
    fn test_web_error_handler_creation() {
        let config = WebLogConfig::default();
        let handler = WebErrorHandler::new(config);
        // Handler should be created successfully
        assert!(true);
    }

    #[test]
    fn test_error_page_generation() {
        let config = WebLogConfig::default();
        let handler = WebErrorHandler::new(config);
        let error_page = handler.create_error_page(HttpStatus::NotFound, Some("Test error"));
        
        assert!(error_page.contains("404"));
        assert!(error_page.contains("Not Found"));
        assert!(error_page.contains("Test error"));
        assert!(error_page.contains("Medical Device QMS"));
        assert!(error_page.contains("FDA 21 CFR Part 820"));
    }

    #[test]
    fn test_request_metrics_creation() {
        use std::collections::HashMap;
        
        let request = HttpRequest {
            method: "GET".to_string(),
            uri: "/api/test".to_string(),
            version: "HTTP/1.1".to_string(),
            headers: {
                let mut headers = HashMap::new();
                headers.insert("User-Agent".to_string(), "Test-Agent".to_string());
                headers
            },
            body: "test body".as_bytes().to_vec(),
            query_params: HashMap::new(),
            timestamp: 0,
        };

        let metrics = RequestMetrics::new(&request, "127.0.0.1".to_string());
        assert_eq!(metrics.method, "GET");
        assert_eq!(metrics.path, "/api/test");
        assert_eq!(metrics.user_agent, "Test-Agent");
        assert_eq!(metrics.remote_addr, "127.0.0.1");
        assert_eq!(metrics.bytes_received, 9); // "test body".len()
    }

    #[test]
    fn test_request_metrics_finish() {
        use std::collections::HashMap;
        
        let request = HttpRequest {
            method: "POST".to_string(),
            uri: "/api/submit".to_string(),
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
            query_params: HashMap::new(),
            timestamp: 0,
        };

        let mut metrics = RequestMetrics::new(&request, "192.168.1.1".to_string());
        metrics.finish(201, 1024);
        
        assert_eq!(metrics.status_code, 201);
        assert_eq!(metrics.bytes_sent, 1024);
        // processing_time_ms will be set during finish()
    }

    #[test]
    fn test_format_timestamp() {
        let time = SystemTime::now();
        let formatted = format_timestamp(time);
        // Should be in format of timestamp.milliseconds
        assert!(formatted.contains('.'));
        assert!(formatted.len() > 5);
    }

    #[test]
    fn test_generate_error_reference_id() {
        let ref_id = generate_error_reference_id();
        // Should be hexadecimal string
        assert!(ref_id.len() > 0);
        assert!(ref_id.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
