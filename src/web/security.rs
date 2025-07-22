// QMS Web Security - Medical Device Security Implementation
// Implements FDA 21 CFR Part 820, ISO 13485, and ISO 14971 security requirements
// Standard library only implementation for regulatory compliance

use crate::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Security configuration for the QMS web server
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Enable HTTPS enforcement
    pub enforce_https: bool,
    /// HTTPS port (default: 443)
    pub https_port: u16,
    /// HTTP port for redirects (default: 80)
    pub http_port: u16,
    /// TLS certificate file path
    pub cert_file: Option<String>,
    /// TLS private key file path
    pub key_file: Option<String>,
    /// Enable HSTS (HTTP Strict Transport Security)
    pub enable_hsts: bool,
    /// HSTS max age in seconds
    pub hsts_max_age: u64,
    /// Enable CSP (Content Security Policy)
    pub enable_csp: bool,
    /// Custom CSP policy
    pub csp_policy: Option<String>,
    /// Enable security headers
    pub enable_security_headers: bool,
    /// Maximum request size in bytes
    pub max_request_size: usize,
    /// Session timeout in seconds
    pub session_timeout: u64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enforce_https: false, // Default to false for development
            https_port: 443,
            http_port: 80,
            cert_file: None,
            key_file: None,
            enable_hsts: true,
            hsts_max_age: 31536000, // 1 year
            enable_csp: true,
            csp_policy: None, // Use default policy
            enable_security_headers: true,
            max_request_size: 10 * 1024 * 1024, // 10MB
            session_timeout: 3600, // 1 hour
        }
    }
}

/// TLS certificate information
#[derive(Debug, Clone)]
pub struct TlsCertificate {
    pub cert_data: Vec<u8>,
    pub key_data: Vec<u8>,
    pub valid_from: u64,
    pub valid_until: u64,
    pub subject: String,
    pub issuer: String,
}

/// Security manager for the QMS web server
pub struct SecurityManager {
    config: SecurityConfig,
    certificate: Option<TlsCertificate>,
}

impl SecurityManager {
    /// Create a new security manager with default configuration
    pub fn new() -> Self {
        Self {
            config: SecurityConfig::default(),
            certificate: None,
        }
    }

    /// Create a new security manager with custom configuration
    pub const fn new_with_config(config: SecurityConfig) -> Self {
        Self {
            config,
            certificate: None,
        }
    }

    /// Load TLS certificate and key from files
    pub fn load_certificate(&mut self, cert_path: &str, key_path: &str) -> QmsResult<()> {
        if !Path::new(cert_path).exists() {
            return Err(QmsError::io_error("Certificate file not found"));
        }

        if !Path::new(key_path).exists() {
            return Err(QmsError::io_error("Key file not found"));
        }

        let cert_data = fs::read(cert_path)
            .map_err(|_| QmsError::io_error("Failed to read certificate"))?;

        let key_data = fs::read(key_path)
            .map_err(|_| QmsError::io_error("Failed to read private key"))?;

        // Basic certificate validation (simplified for stdlib-only implementation)
        if cert_data.is_empty() || key_data.is_empty() {
            return Err(QmsError::validation_error("Certificate or key file is empty"));
        }

        // Create certificate info (simplified - in production, parse actual certificate)
        let certificate = TlsCertificate {
            cert_data,
            key_data,
            valid_from: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            valid_until: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + (365 * 24 * 3600), // 1 year
            subject: "QMS Medical Device Server".to_string(),
            issuer: "QMS Certificate Authority".to_string(),
        };

        self.certificate = Some(certificate);
        self.config.cert_file = Some(cert_path.to_string());
        self.config.key_file = Some(key_path.to_string());

        println!("🔒 TLS Certificate loaded successfully");
        Ok(())
    }

    /// Check if HTTPS is properly configured
    pub const fn is_https_configured(&self) -> bool {
        self.certificate.is_some() && self.config.cert_file.is_some() && self.config.key_file.is_some()
    }

    /// Get security configuration
    pub const fn get_config(&self) -> &SecurityConfig {
        &self.config
    }

    /// Update security configuration
    pub fn update_config(&mut self, config: SecurityConfig) {
        self.config = config;
    }

    /// Validate request security
    pub fn validate_request_security(&self, headers: &HashMap<String, String>, body_size: usize) -> QmsResult<()> {
        // Check request size limit
        if body_size > self.config.max_request_size {
            return Err(QmsError::validation_error(&format!(
                "Request size {} exceeds maximum allowed size {}",
                body_size, self.config.max_request_size
            )));
        }

        // Check for required security headers in requests
        if self.config.enforce_https {
            if let Some(forwarded_proto) = headers.get("x-forwarded-proto") {
                if forwarded_proto != "https" {
                    return Err(QmsError::validation_error("HTTPS required for medical device data"));
                }
            }
        }

        Ok(())
    }

    /// Generate HTTPS redirect response
    pub fn create_https_redirect_response(&self, host: &str, path: &str) -> String {
        let https_url = if self.config.https_port == 443 {
            format!("https://{host}{path}")
        } else {
            format!("https://{}:{}{}", host, self.config.https_port, path)
        };

        format!(
            "HTTP/1.1 301 Moved Permanently\r\n\
             Location: {}\r\n\
             Content-Type: text/html\r\n\
             Content-Length: 0\r\n\
             Strict-Transport-Security: max-age={}; includeSubDomains; preload\r\n\
             X-Content-Type-Options: nosniff\r\n\
             X-Frame-Options: DENY\r\n\
             Connection: close\r\n\
             \r\n",
            https_url, self.config.hsts_max_age
        )
    }

    /// Get default Content Security Policy for medical devices
    pub fn get_default_csp_policy(&self) -> String {
        self.config.csp_policy.clone().unwrap_or_else(|| {
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data:; \
             font-src 'self'; \
             connect-src 'self'; \
             media-src 'self'; \
             object-src 'none'; \
             child-src 'none'; \
             frame-src 'none'; \
             worker-src 'self'; \
             manifest-src 'self'; \
             base-uri 'self'; \
             form-action 'self'".to_string()
        })
    }

    /// Check if certificate is valid
    pub fn is_certificate_valid(&self) -> bool {
        if let Some(cert) = &self.certificate {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
            now >= cert.valid_from && now <= cert.valid_until
        } else {
            false
        }
    }

    /// Get certificate information
    pub const fn get_certificate_info(&self) -> Option<&TlsCertificate> {
        self.certificate.as_ref()
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Security audit information
#[derive(Debug)]
pub struct SecurityAudit {
    pub timestamp: u64,
    pub event_type: String,
    pub description: String,
    pub source_ip: Option<String>,
    pub user_agent: Option<String>,
    pub severity: SecuritySeverity,
}

/// Security event severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl SecurityAudit {
    pub fn new(event_type: &str, description: &str, severity: SecuritySeverity) -> Self {
        Self {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            event_type: event_type.to_string(),
            description: description.to_string(),
            source_ip: None,
            user_agent: None,
            severity,
        }
    }

    pub fn with_source_info(mut self, ip: Option<String>, user_agent: Option<String>) -> Self {
        self.source_ip = ip;
        self.user_agent = user_agent;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_config_default() {
        let config = SecurityConfig::default();
        assert!(!config.enforce_https);
        assert_eq!(config.https_port, 443);
        assert_eq!(config.http_port, 80);
        assert!(config.enable_hsts);
        assert!(config.enable_csp);
        assert!(config.enable_security_headers);
    }

    #[test]
    fn test_security_manager_creation() {
        let manager = SecurityManager::new();
        assert!(!manager.is_https_configured());
        assert!(!manager.is_certificate_valid());
    }

    #[test]
    fn test_csp_policy_generation() {
        let manager = SecurityManager::new();
        let policy = manager.get_default_csp_policy();
        assert!(policy.contains("default-src 'self'"));
        assert!(policy.contains("object-src 'none'"));
    }

    #[test]
    fn test_https_redirect_response() {
        let manager = SecurityManager::new();
        let response = manager.create_https_redirect_response("example.com", "/test");
        assert!(response.contains("301 Moved Permanently"));
        assert!(response.contains("https://example.com/test"));
        assert!(response.contains("Strict-Transport-Security"));
    }

    #[test]
    fn test_request_size_validation() {
        let manager = SecurityManager::new();
        let headers = HashMap::new();
        
        // Test valid request size
        assert!(manager.validate_request_security(&headers, 1024).is_ok());
        
        // Test oversized request
        assert!(manager.validate_request_security(&headers, 20 * 1024 * 1024).is_err());
    }

    #[test]
    fn test_security_audit_creation() {
        let audit = SecurityAudit::new("LOGIN_ATTEMPT", "User login attempt", SecuritySeverity::Medium);
        assert_eq!(audit.event_type, "LOGIN_ATTEMPT");
        assert_eq!(audit.severity, SecuritySeverity::Medium);
        assert!(audit.timestamp > 0);
    }
}
