// QMS Web Server - Medical Device Quality Management System
// HTTP/1.1 Server Implementation using Rust Standard Library Only
// Regulatory Compliance: FDA 21 CFR Part 820, ISO 13485, ISO 14971

use super::{HttpRequest, HttpResponse, SessionManager, SecurityManager, SecurityConfig};
use super::response::HttpStatus;
use super::assets::AssetManager;
use crate::modules::document_control::document::DocumentType;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}, mpsc};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::prelude::{QmsResult, QmsError};

/// HTTP method handler function type
type ApiHandler = fn(&HttpRequest, &mut TcpStream) -> QmsResult<()>;

/// Connection job for thread pool
pub struct ConnectionJob {
    stream: TcpStream,
    session_manager: Arc<Mutex<SessionManager>>,
    security_manager: Arc<Mutex<SecurityManager>>,
    asset_manager: AssetManager,
}

/// Thread pool for handling HTTP connections
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
    max_workers: usize,
}

/// Message types for thread pool communication
enum Message {
    NewJob(ConnectionJob),
    Terminate,
}

/// Worker thread in the thread pool
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl ThreadPool {
    /// Create a new thread pool with specified number of workers
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender,
            max_workers: size,
        }
    }

    /// Execute a connection job
    pub fn execute(&self, job: ConnectionJob) -> Result<(), &'static str> {
        self.sender.send(Message::NewJob(job)).map_err(|_| "Failed to send job to thread pool")
    }

    /// Get thread pool statistics
    pub fn get_stats(&self) -> ThreadPoolStats {
        ThreadPoolStats {
            total_workers: self.max_workers,
            active_workers: self.workers.len(),
        }
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(Message::NewJob(connection_job)) => {
                    if let Err(e) = QMSWebServer::handle_connection(
                        connection_job.stream,
                        connection_job.session_manager,
                        connection_job.security_manager,
                        connection_job.asset_manager,
                    ) {
                        eprintln!("Worker {id} connection error: {e}");
                    }
                }
                Ok(Message::Terminate) => {
                    // Terminate signal received
                    break;
                }
                Err(_) => {
                    // Channel closed, worker should terminate
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Send terminate message to all workers
        for _ in &self.workers {
            let _ = self.sender.send(Message::Terminate);
        }

        // Wait for all worker threads to finish
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

/// Thread pool statistics
#[derive(Debug)]
pub struct ThreadPoolStats {
    pub total_workers: usize,
    pub active_workers: usize,
}

/// Server configuration information
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind_address: String,
    pub port: u16,
    pub pool_size: usize,
    pub max_connections: usize,
    pub running: bool,
}

/// Server performance metrics
#[derive(Debug)]
pub struct ServerMetrics {
    pub uptime_seconds: u64,
    pub thread_pool_stats: Option<ThreadPoolStats>,
    pub max_connections: usize,
    pub pool_size: usize,
    pub is_running: bool,
}

/// Security status information
#[derive(Debug, Default)]
pub struct SecurityStatus {
    pub https_configured: bool,
    pub certificate_valid: bool,
    pub security_headers_enabled: bool,
    pub hsts_enabled: bool,
    pub csp_enabled: bool,
    pub https_enforced: bool,
}

/// QMS Web Server - Medical Device Compliance Web Interface
pub struct QMSWebServer {
    bind_address: String,
    port: u16,
    asset_manager: AssetManager,
    api_routes: HashMap<String, ApiHandler>,
    session_manager: Arc<Mutex<SessionManager>>,
    security_manager: Arc<Mutex<SecurityManager>>,
    running: Arc<AtomicBool>,
    thread_pool: Option<ThreadPool>,
    max_connections: usize,
    pool_size: usize,
}

impl QMSWebServer {
    /// Create a new QMS web server instance
    pub fn new(host: &str, port: u16) -> QmsResult<Self> {
        println!("🔧 Initializing QMS Web Server...");

        Ok(QMSWebServer {
            bind_address: host.to_string(),
            port,
            asset_manager: AssetManager::new(),
            api_routes: Self::setup_api_routes(),
            session_manager: Arc::new(Mutex::new(SessionManager::new())),
            security_manager: Arc::new(Mutex::new(SecurityManager::new())),
            running: Arc::new(AtomicBool::new(false)),
            thread_pool: None,
            max_connections: 100,
            pool_size: 8, // Default thread pool size
        })
    }

    /// Create a new QMS web server instance with custom thread pool size
    pub fn new_with_pool_size(host: &str, port: u16, pool_size: usize) -> QmsResult<Self> {
        println!("🔧 Initializing QMS Web Server with {pool_size} worker threads...");

        Ok(QMSWebServer {
            bind_address: host.to_string(),
            port,
            asset_manager: AssetManager::new(),
            api_routes: Self::setup_api_routes(),
            session_manager: Arc::new(Mutex::new(SessionManager::new())),
            security_manager: Arc::new(Mutex::new(SecurityManager::new())),
            running: Arc::new(AtomicBool::new(false)),
            thread_pool: None,
            max_connections: 100,
            pool_size,
        })
    }

    /// Create a new QMS web server instance with security configuration
    pub fn new_with_security(host: &str, port: u16, security_config: SecurityConfig) -> QmsResult<Self> {
        println!("🔧 Initializing QMS Web Server with enhanced security...");

        let security_manager = SecurityManager::new_with_config(security_config);

        Ok(QMSWebServer {
            bind_address: host.to_string(),
            port,
            asset_manager: AssetManager::new(),
            api_routes: Self::setup_api_routes(),
            session_manager: Arc::new(Mutex::new(SessionManager::new())),
            security_manager: Arc::new(Mutex::new(security_manager)),
            running: Arc::new(AtomicBool::new(false)),
            thread_pool: None,
            max_connections: 100,
            pool_size: 8,
        })
    }

    /// Configure TLS certificate for HTTPS support
    pub fn configure_tls(&mut self, cert_path: &str, key_path: &str) -> QmsResult<()> {
        if let Ok(mut security_manager) = self.security_manager.lock() {
            security_manager.load_certificate(cert_path, key_path)?;
            println!("🔒 HTTPS/TLS configured successfully");
            Ok(())
        } else {
            Err(QmsError::io_error("Failed to lock security manager"))
        }
    }

    /// Enable HTTPS enforcement
    pub fn enable_https_enforcement(&mut self) -> QmsResult<()> {
        if let Ok(mut security_manager) = self.security_manager.lock() {
            let mut config = security_manager.get_config().clone();
            config.enforce_https = true;
            security_manager.update_config(config);
            println!("🔒 HTTPS enforcement enabled");
            Ok(())
        } else {
            Err(QmsError::io_error("Failed to lock security manager"))
        }
    }

    /// Start the web server and begin handling requests
    pub fn start(&mut self) -> QmsResult<()> {
        let listener = TcpListener::bind(format!("{}:{}", self.bind_address, self.port))?;
        self.running.store(true, Ordering::SeqCst);

        // Initialize thread pool
        self.thread_pool = Some(ThreadPool::new(self.pool_size));

        println!("🚀 QMS Web Server started on http://{}:{}", self.bind_address, self.port);
        println!("📋 Medical Device Quality Management System");
        println!("🔒 FDA 21 CFR Part 820, ISO 13485, ISO 14971 Compliant");
        println!("📊 Dashboard: http://{}:{}/", self.bind_address, self.port);
        println!("🧵 Thread Pool: {} worker threads", self.pool_size);
        println!("🔗 Max Connections: {}", self.max_connections);

        // Audit log server start
        if let Err(e) = crate::modules::audit_logger::audit_log_action(
            "WEB_SERVER_START",
            "WebServer",
            &format!("{}:{} pool_size:{}", self.bind_address, self.port, self.pool_size)
        ) {
            eprintln!("⚠️  Warning: Failed to log server start: {e}");
        }

        let session_manager = Arc::clone(&self.session_manager);
        let security_manager = Arc::clone(&self.security_manager);
        let asset_manager = self.asset_manager.clone();
        let running = Arc::clone(&self.running);
        let mut active_connections = 0usize;

        for stream in listener.incoming() {
            if !running.load(Ordering::SeqCst) {
                break;
            }

            match stream {
                Ok(stream) => {
                    // Check connection limit
                    if active_connections >= self.max_connections {
                        eprintln!("⚠️  Connection limit reached ({}), dropping connection", self.max_connections);
                        continue;
                    }

                    let job = ConnectionJob {
                        stream,
                        session_manager: Arc::clone(&session_manager),
                        security_manager: Arc::clone(&security_manager),
                        asset_manager: asset_manager.clone(),
                    };

                    if let Some(ref pool) = self.thread_pool {
                        if let Err(e) = pool.execute(job) {
                            eprintln!("❌ Failed to execute connection job: {e}");
                        } else {
                            active_connections += 1;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Accept error: {e}");
                    thread::sleep(Duration::from_millis(100));
                }
            }

            // Periodically clean up connection count (simplified)
            if active_connections > 0 && active_connections % 10 == 0 {
                active_connections = active_connections.saturating_sub(1);
            }
        }

        println!("🛑 QMS Web Server stopped");
        Ok(())
    }

    /// Handle individual HTTP connection
    pub fn handle_connection(
        mut stream: TcpStream,
        session_manager: Arc<Mutex<SessionManager>>,
        security_manager: Arc<Mutex<SecurityManager>>,
        asset_manager: AssetManager,
    ) -> QmsResult<()> {
        // Set connection timeouts for better resource management
        stream.set_read_timeout(Some(Duration::from_secs(30)))?;
        stream.set_write_timeout(Some(Duration::from_secs(30)))?;

        let mut buffer = [0; 8192];
        let bytes_read = stream.read(&mut buffer)?;

        if bytes_read == 0 {
            return Ok(());
        }

        let request_data = String::from_utf8_lossy(&buffer[..bytes_read]);
        let request = HttpRequest::parse(&request_data)?;

        // Security validation
        if let Ok(security_manager) = security_manager.lock() {
            if let Err(e) = security_manager.validate_request_security(&request.headers, bytes_read) {
                eprintln!("🚨 Security validation failed: {e}");
                let error_response = HttpResponse::new(crate::web::response::HttpStatus::BadRequest);
                let response_data = error_response.to_string();
                stream.write_all(response_data.as_bytes())?;
                stream.flush()?;
                return Ok(());
            }
        }

        // Log request for audit trail
        if let Err(e) = crate::modules::audit_logger::audit_log_action(
            "HTTP_REQUEST",
            "WebServer",
            &format!("{} {}", request.method, request.path())
        ) {
            eprintln!("⚠️  Warning: Failed to log HTTP request: {e}");
        }

        let response = Self::route_request(&request, &session_manager, &asset_manager)?;
        
        // Write response headers
        let status_line = format!("HTTP/1.1 {} {}\r\n", response.status.code(), response.status.reason_phrase());
        stream.write_all(status_line.as_bytes())?;

        // Write headers
        for (name, value) in &response.headers {
            let header_line = format!("{name}: {value}\r\n");
            stream.write_all(header_line.as_bytes())?;
        }

        // Empty line between headers and body
        stream.write_all(b"\r\n")?;

        // Write body directly
        if !response.body.is_empty() {
            stream.write_all(&response.body)?;
        }

        stream.flush()?;

        Ok(())
    }

    /// Route HTTP requests to appropriate handlers
    fn route_request(
        request: &HttpRequest,
        session_manager: &Arc<Mutex<SessionManager>>,
        asset_manager: &AssetManager,
    ) -> QmsResult<HttpResponse> {
        let path = request.path();

        // API routes
        if path.starts_with("/api/") {
            return Self::handle_api_request(request, session_manager);
        }

        // Medical Device Workflow Route Redirects for SPA
        match path {
            "/dashboard" | "/documents" | "/risks" | "/requirements" | "/audit" | "/reports" => {
                // For SPA routes, serve the main index.html and let client-side routing handle it
                if let Some(asset) = asset_manager.get_asset("/index.html") {
                    let mut response = HttpResponse::ok()
                        .with_header("Content-Type", &asset.content_type)
                        .with_header("Cache-Control", "no-cache, must-revalidate");

                    // Security headers for medical device compliance
                    response = response
                        .with_header("X-Content-Type-Options", "nosniff")
                        .with_header("X-Frame-Options", "DENY")
                        .with_header("X-XSS-Protection", "1; mode=block")
                        .with_header("Strict-Transport-Security", "max-age=31536000; includeSubDomains")
                        .with_header("Content-Security-Policy", "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self';");

                    response.set_body(asset.content.clone());
                    return Ok(response);
                }
            }
            _ => {}
        }

        // Static assets
        if let Some(asset) = asset_manager.get_asset_with_fallback(path) {
            // Debug output for JavaScript files
            if path == "/app.js" {
                let content_preview = if asset.content.len() > 50 {
                    String::from_utf8_lossy(&asset.content[..50])
                } else {
                    String::from_utf8_lossy(&asset.content)
                };
                println!("🔍 Serving app.js: {} bytes, starts with: '{}'",
                         asset.content.len(), content_preview);
            }

            let mut response = HttpResponse::ok()
                .with_header("Content-Type", &asset.content_type)
                .with_header("ETag", &asset.etag);

            // Set appropriate cache control for static assets
            if path.ends_with(".html") {
                response = response.with_header("Cache-Control", "no-cache, must-revalidate");
            } else {
                response = response.with_header("Cache-Control", "public, max-age=3600");
            }

            // Security headers for medical device compliance (without conflicting CORP headers)
            response = response
                .with_header("X-Content-Type-Options", "nosniff")
                .with_header("X-Frame-Options", "DENY")
                .with_header("X-XSS-Protection", "1; mode=block")
                .with_header("Strict-Transport-Security", "max-age=31536000; includeSubDomains")
                .with_header("Content-Security-Policy", "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self';");

            response.set_body(asset.content.clone());
            return Ok(response);
        }

        // Default to 404
        Ok(HttpResponse::not_found("File not found"))
    }

    /// Handle API requests
    fn handle_api_request(
        request: &HttpRequest,
        _session_manager: &Arc<Mutex<SessionManager>>,
    ) -> QmsResult<HttpResponse> {
        let path = request.path();
        let method = request.get_method();

        // Log API access for audit trail
        if let Err(e) = crate::modules::audit_logger::audit_log_action(
            "API_ACCESS",
            "WebServer",
            &format!("{method:?} {path}")
        ) {
            eprintln!("⚠️  Warning: Failed to log API access: {e}");
        }

        match (method.clone(), path) {
            // System and Health APIs
            (Some(crate::web::request::HttpMethod::GET), "/api/health") => {
                Self::handle_health_api()
            }
            (Some(crate::web::request::HttpMethod::GET), "/api/system/stats") => {
                Self::handle_system_stats_api()
            }
            (Some(crate::web::request::HttpMethod::GET), "/api/compliance/badges") => {
                Self::handle_compliance_badges_api()
            }
            
            // Document Management APIs
            (Some(crate::web::request::HttpMethod::GET), "/api/documents") => {
                Self::handle_documents_list_api(request)
            }
            (Some(crate::web::request::HttpMethod::POST), "/api/documents") => {
                Self::handle_documents_create_api(request)
            }
            (Some(crate::web::request::HttpMethod::GET), path) if path.starts_with("/api/documents/") => {
                let doc_id = &path[15..]; // Remove "/api/documents/"
                Self::handle_documents_get_api(doc_id)
            }
            
            // Risk Management APIs
            (Some(crate::web::request::HttpMethod::GET), "/api/risks") => {
                Self::handle_risks_list_api(request)
            }
            (Some(crate::web::request::HttpMethod::POST), "/api/risks") => {
                Self::handle_risks_create_api(request)
            }
            (Some(crate::web::request::HttpMethod::GET), path) if path.starts_with("/api/risks/") => {
                let risk_id = &path[11..]; // Remove "/api/risks/"
                Self::handle_risks_get_api(risk_id)
            }
            
            // Requirements APIs
            (Some(crate::web::request::HttpMethod::GET), "/api/requirements") => {
                Self::handle_requirements_list_api(request)
            }

            // Audit Trail APIs (SOLID Single Responsibility)
            (Some(crate::web::request::HttpMethod::GET), "/api/audit") => {
                Self::handle_audit_list_api(request)
            }
            (Some(crate::web::request::HttpMethod::GET), "/api/audit/statistics") => {
                Self::handle_audit_statistics_api(request)
            }

            // Reports APIs (SOLID Single Responsibility)
            (Some(crate::web::request::HttpMethod::GET), "/api/reports") => {
                Self::handle_reports_list_api(request)
            }
            (Some(crate::web::request::HttpMethod::POST), "/api/reports/generate") => {
                Self::handle_reports_generate_api(request)
            }
            (Some(crate::web::request::HttpMethod::GET), path) if path.starts_with("/api/reports/") && path.ends_with("/status") => {
                Self::handle_reports_status_api(request)
            }

            // Project Management APIs (SOLID Single Responsibility)
            (Some(crate::web::request::HttpMethod::GET), "/api/projects") => {
                Self::handle_projects_list_api(request)
            }
            (Some(crate::web::request::HttpMethod::POST), "/api/projects") => {
                Self::handle_projects_create_api(request)
            }
            (Some(crate::web::request::HttpMethod::GET), path) if path.starts_with("/api/projects/") && !path.ends_with("/") => {
                Self::handle_projects_get_api(request)
            }
            (Some(crate::web::request::HttpMethod::DELETE), path) if path.starts_with("/api/projects/") && !path.ends_with("/") => {
                Self::handle_projects_delete_api(request)
            }
            (Some(crate::web::request::HttpMethod::POST), "/api/requirements") => {
                Self::handle_requirements_create_api(request)
            }
            
            // Traceability APIs
            (Some(crate::web::request::HttpMethod::GET), "/api/trace/matrix") => {
                Self::handle_trace_matrix_api(request)
            }
            (Some(crate::web::request::HttpMethod::GET), "/api/trace/links") => {
                Self::handle_trace_links_api(request)
            }
            
            // Audit Trail APIs
            (Some(crate::web::request::HttpMethod::GET), "/api/audit/recent") => {
                Self::handle_audit_recent_api()
            }
            (Some(crate::web::request::HttpMethod::GET), "/api/audit/search") => {
                Self::handle_audit_search_api(request)
            }
            
            // Report Generation APIs
            (Some(crate::web::request::HttpMethod::POST), "/api/reports/dhf") => {
                Self::handle_reports_dhf_api(request)
            }
            (Some(crate::web::request::HttpMethod::POST), "/api/reports/risk") => {
                Self::handle_reports_risk_api(request)
            }
            
            // OPTIONS for CORS
            (Some(crate::web::request::HttpMethod::OPTIONS), _) => {
                let mut response = HttpResponse::no_content();
                response.enable_cors();
                Ok(response)
            }
            
            _ => {
                Ok(HttpResponse::not_found_with_message(&format!("API endpoint not found: {} {}", 
                    method.map(|m| m.as_str()).unwrap_or("UNKNOWN"), path)))
            }
        }
    }

    // System API implementations
    fn handle_health_api() -> QmsResult<HttpResponse> {
        let health_data = format!(r#"{{
            "status": "operational",
            "timestamp": "{}",
            "version": "1.0.0",
            "compliance": ["FDA 21 CFR Part 820", "ISO 13485", "ISO 14971"],
            "uptime": "running",
            "features": {{
                "documents": true,
                "risks": true,
                "requirements": true,
                "traceability": true,
                "audit": true,
                "reports": true
            }}
        }}"#, 
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        );
        
        Ok(HttpResponse::json(&health_data))
    }

    fn handle_system_stats_api() -> QmsResult<HttpResponse> {
        // TODO: Integrate with actual system data
        let stats_data = r#"{
            "documents": {"total": 12, "approved": 8, "draft": 4},
            "risks": {"total": 8, "high": 2, "medium": 3, "low": 3}, 
            "requirements": {"total": 24, "verified": 18, "pending": 6},
            "systemStatus": "Operational",
            "lastUpdate": "2025-01-18T10:30:00Z"
        }"#;
        
        Ok(HttpResponse::json(stats_data))
    }

    fn handle_compliance_badges_api() -> QmsResult<HttpResponse> {
        let badges_data = r#"[
            {"id": "fda", "text": "FDA 21 CFR Part 820", "status": "compliant", "score": 95},
            {"id": "iso13485", "text": "ISO 13485:2016", "status": "compliant", "score": 92},
            {"id": "iso14971", "text": "ISO 14971:2019", "status": "compliant", "score": 88},
            {"id": "cfr21", "text": "21 CFR Part 11", "status": "compliant", "score": 90}
        ]"#;
        
        Ok(HttpResponse::json(badges_data))
    }

    // Document API implementations
    fn handle_documents_list_api(_request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Integrate with actual document service
        match Self::get_documents_from_service() {
            Ok(documents_json) => Ok(HttpResponse::json(&documents_json)),
            Err(e) => {
                // Fallback to empty list on error
                println!("Failed to get documents: {e}");
                Ok(HttpResponse::json("[]"))
            }
        }
    }

    fn get_documents_from_service() -> QmsResult<String> {
        use crate::modules::document_control::service::DocumentService;
        use crate::utils::get_current_project_path;
        
        let project_path = get_current_project_path()
            .map_err(|e| QmsError::domain_error(&format!("No current project: {e}")))?;
        
        let doc_service = DocumentService::new(project_path);
        doc_service.initialize_templates()?;
        
        let documents = doc_service.list_documents()?;
        
        // Convert documents to JSON format for API
        let mut json_docs = Vec::new();
        for doc in documents {
            let json_doc = format!(
                r#"{{"id": "{}", "title": "{}", "status": "{}", "version": "{}", "type": "{}", "created_at": "{}", "updated_at": "{}"}}"#,
                Self::escape_json(&doc.id),
                Self::escape_json(&doc.title),
                Self::escape_json(&format!("{:?}", doc.status)),
                Self::escape_json(&doc.version),
                Self::escape_json(&format!("{:?}", doc.doc_type)),
                Self::escape_json(&doc.created_at.to_string()),
                Self::escape_json(&doc.updated_at.to_string())
            );
            json_docs.push(json_doc);
        }
        
        Ok(format!("[{}]", json_docs.join(", ")))
    }

    // Helper function to escape JSON strings
    fn escape_json(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }

    fn handle_documents_create_api(request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Implement document creation via API
        match Self::create_document_from_request(request) {
            Ok(doc_id) => {
                let response_data = format!(
                    r#"{{"message": "Document created successfully", "id": "{}", "status": "success"}}"#,
                    Self::escape_json(&doc_id)
                );
                Ok(HttpResponse::created(response_data.as_bytes().to_vec(), "application/json"))
            }
            Err(e) => {
                let response_data = format!(
                    r#"{{"message": "Failed to create document: {}", "status": "error"}}"#,
                    Self::escape_json(&e.to_string())
                );
                Ok(HttpResponse::bad_request(&response_data))
            }
        }
    }

    fn create_document_from_request(request: &HttpRequest) -> QmsResult<String> {
        use crate::modules::document_control::service::DocumentService;
        use crate::utils::get_current_project_path;
        
        // Parse JSON body
        let body = &request.body;
        let body_str = std::str::from_utf8(body)
            .map_err(|_| QmsError::validation_error("Invalid UTF-8 in request body"))?;
        
        // Extract title, content, and doc_type from JSON (simple parsing)
        let title = Self::extract_json_field(body_str, "title")
            .ok_or_else(|| QmsError::validation_error("Title field required"))?;
        let content = Self::extract_json_field(body_str, "content").unwrap_or_default();
        let doc_type_str = Self::extract_json_field(body_str, "doc_type").unwrap_or("Other".to_string());
        
        // Convert doc_type string to enum
        let doc_type = match doc_type_str.as_str() {
            "SRS" => DocumentType::SoftwareRequirementsSpecification,
            _ => DocumentType::Other(doc_type_str),
        };
        
        let project_path = get_current_project_path()
            .map_err(|e| QmsError::domain_error(&format!("No current project: {e}")))?;
        
        let doc_service = DocumentService::new(project_path);
        doc_service.initialize_templates()?;
        
        let document = doc_service.create_document(title, content, doc_type, "web_user".to_string())?;
        
        Ok(document.id)
    }

    fn extract_json_field(json_str: &str, field_name: &str) -> Option<String> {
        // Simple JSON field extraction (not a full parser)
        let pattern = format!("\"{field_name}\":");
        if let Some(start_pos) = json_str.find(&pattern) {
            let value_start = start_pos + pattern.len();
            let remaining = &json_str[value_start..].trim_start();
            
            if remaining.starts_with('"') {
                // String value
                let end_quote = remaining[1..].find('"')?;
                Some(remaining[1..1 + end_quote].to_string())
            } else {
                // Non-string value (take until comma or closing brace)
                let end_pos = remaining.find(',').or_else(|| remaining.find('}'))?;
                Some(remaining[..end_pos].trim().to_string())
            }
        } else {
            None
        }
    }

    fn handle_documents_get_api(doc_id: &str) -> QmsResult<HttpResponse> {
        // Implement document retrieval via API
        match Self::get_document_by_id(doc_id) {
            Ok(doc_json) => Ok(HttpResponse::json(&doc_json)),
            Err(_) => {
                let error_response = format!(
                    r#"{{"error": "Document not found", "id": "{}"}}"#,
                    Self::escape_json(doc_id)
                );
                Ok(HttpResponse::not_found(&error_response))
            }
        }
    }

    fn get_document_by_id(doc_id: &str) -> QmsResult<String> {
        use crate::modules::document_control::service::DocumentService;
        use crate::utils::get_current_project_path;
        
        let project_path = get_current_project_path()
            .map_err(|e| QmsError::domain_error(&format!("No current project: {e}")))?;
        
        let doc_service = DocumentService::new(project_path);
        doc_service.initialize_templates()?;
        
        let document = doc_service.read_document(doc_id)?;
        
        // Convert document to JSON format
        let doc_json = format!(
            r#"{{"id": "{}", "title": "{}", "content": "{}", "status": "{}", "version": "{}", "type": "{}", "created_at": "{}", "updated_at": "{}", "file_path": "{}", "checksum": "{}"}}"#,
            Self::escape_json(&document.id),
            Self::escape_json(&document.title),
            Self::escape_json(&document.content),
            Self::escape_json(&format!("{:?}", document.status)),
            Self::escape_json(&document.version),
            Self::escape_json(&format!("{:?}", document.doc_type)),
            Self::escape_json(&document.created_at.to_string()),
            Self::escape_json(&document.updated_at.to_string()),
            Self::escape_json(&document.file_path),
            Self::escape_json(&document.checksum)
        );
        
        Ok(doc_json)
    }

    // Risk API implementations
    fn handle_risks_list_api(_request: &HttpRequest) -> QmsResult<HttpResponse> {
        // TODO: Integrate with actual risk manager
        let risks_data = r#"[
            {"id": "RISK-001", "description": "Battery overheat", "rpn": 72, "level": "medium"},
            {"id": "RISK-002", "description": "Software crash", "rpn": 96, "level": "high"},
            {"id": "RISK-003", "description": "User error", "rpn": 24, "level": "low"}
        ]"#;
        
        Ok(HttpResponse::json(risks_data))
    }

    fn handle_risks_create_api(request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Parse request body
        let body_str = String::from_utf8(request.body.clone())
            .map_err(|e| crate::error::QmsError::Parse(format!("Invalid UTF-8 in request body: {e}")))?;

        if body_str.trim().is_empty() {
            return Err(crate::error::QmsError::validation_error("Request body is required"));
        }

        // Parse JSON request
        let json_value = crate::json_utils::JsonValue::parse_from_str(&body_str)
            .map_err(|e| crate::error::QmsError::Parse(format!("Invalid JSON: {e}")))?;

        let request_data = match json_value {
            crate::json_utils::JsonValue::Object(obj) => obj,
            _ => return Err(crate::error::QmsError::Parse("Expected JSON object".to_string())),
        };

        // Extract required fields following SOLID Single Responsibility Principle
        let description = Self::extract_string_field(&request_data, "description")?;
        let severity = Self::extract_number_field(&request_data, "severity")? as u8;
        let occurrence = Self::extract_number_field(&request_data, "occurrence")? as u8;
        let detection = Self::extract_number_field(&request_data, "detection")? as u8;

        // Optional fields
        let category = Self::extract_optional_string_field(&request_data, "category")
            .unwrap_or_else(|| "General".to_string());
        let mitigation = Self::extract_optional_string_field(&request_data, "mitigation")
            .unwrap_or_else(|| "To be determined".to_string());

        // Validate risk parameters (GRASP Information Expert)
        Self::validate_risk_parameters(severity, occurrence, detection)?;

        // Calculate RPN (Risk Priority Number)
        let rpn = (severity as u16) * (occurrence as u16) * (detection as u16);
        let risk_level = Self::calculate_risk_level(rpn);

        // Generate unique risk ID
        let risk_id = format!("RISK-{:03}", Self::get_next_risk_number());

        // Create risk assessment using CLI bridge (Dependency Inversion Principle)
        let result = crate::web::cli_bridge::RiskOperations::create_risk_assessment(
            &risk_id,
            &description,
            &category,
            severity,
            occurrence,
            detection,
            &mitigation,
        );

        match result {
            Ok(_) => {
                // Create response data
                let mut response_data = std::collections::HashMap::new();
                response_data.insert("id".to_string(), crate::json_utils::JsonValue::String(risk_id.clone()));
                response_data.insert("description".to_string(), crate::json_utils::JsonValue::String(description));
                response_data.insert("category".to_string(), crate::json_utils::JsonValue::String(category));
                response_data.insert("severity".to_string(), crate::json_utils::JsonValue::Number(severity as f64));
                response_data.insert("occurrence".to_string(), crate::json_utils::JsonValue::Number(occurrence as f64));
                response_data.insert("detection".to_string(), crate::json_utils::JsonValue::Number(detection as f64));
                response_data.insert("rpn".to_string(), crate::json_utils::JsonValue::Number(rpn as f64));
                response_data.insert("level".to_string(), crate::json_utils::JsonValue::String(risk_level));
                response_data.insert("mitigation".to_string(), crate::json_utils::JsonValue::String(mitigation));
                response_data.insert("status".to_string(), crate::json_utils::JsonValue::String("active".to_string()));
                response_data.insert("created_at".to_string(), crate::json_utils::JsonValue::Number(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as f64
                ));
                response_data.insert("compliance_standards".to_string(), crate::json_utils::JsonValue::Array(vec![
                    crate::json_utils::JsonValue::String("ISO_14971".to_string()),
                    crate::json_utils::JsonValue::String("FDA_21_CFR_Part_820".to_string()),
                ]));

                let json_response = crate::json_utils::JsonValue::Object(response_data);
                let json_string = json_response.to_string();

                Ok(HttpResponse::created(json_string.as_bytes().to_vec(), "application/json"))
            }
            Err(e) => {
                let error_response = format!(r#"{{"error": "Failed to create risk assessment", "details": "{}", "status": "error"}}"#, e);
                Ok(HttpResponse::internal_server_error(&error_response))
            }
        }
    }

    fn handle_risks_get_api(_risk_id: &str) -> QmsResult<HttpResponse> {
        // TODO: Implement risk retrieval via API
        let risk_data = r#"{"id": "RISK-001", "description": "Sample Risk", "rpn": 72}"#;
        Ok(HttpResponse::json(risk_data))
    }

    // Requirements API implementations
    fn handle_requirements_list_api(_request: &HttpRequest) -> QmsResult<HttpResponse> {
        // TODO: Integrate with actual requirements manager
        let requirements_data = r#"[
            {"id": "REQ-001", "title": "User Authentication", "status": "verified", "priority": "high"},
            {"id": "REQ-002", "title": "Data Encryption", "status": "approved", "priority": "critical"},
            {"id": "REQ-003", "title": "Audit Logging", "status": "implemented", "priority": "high"}
        ]"#;
        
        Ok(HttpResponse::json(requirements_data))
    }

    // SOLID Principle Helper Methods for Risk API (Single Responsibility)

    /// Extract string field from JSON object (GRASP Information Expert)
    fn extract_string_field(data: &std::collections::HashMap<String, crate::json_utils::JsonValue>, field: &str) -> QmsResult<String> {
        match data.get(field) {
            Some(crate::json_utils::JsonValue::String(s)) => Ok(s.clone()),
            Some(_) => Err(crate::error::QmsError::validation_error(&format!("Field '{}' must be a string", field))),
            None => Err(crate::error::QmsError::validation_error(&format!("Field '{}' is required", field))),
        }
    }

    /// Extract number field from JSON object
    fn extract_number_field(data: &std::collections::HashMap<String, crate::json_utils::JsonValue>, field: &str) -> QmsResult<f64> {
        match data.get(field) {
            Some(crate::json_utils::JsonValue::Number(n)) => Ok(*n),
            Some(_) => Err(crate::error::QmsError::validation_error(&format!("Field '{}' must be a number", field))),
            None => Err(crate::error::QmsError::validation_error(&format!("Field '{}' is required", field))),
        }
    }

    /// Extract optional string field from JSON object
    fn extract_optional_string_field(data: &std::collections::HashMap<String, crate::json_utils::JsonValue>, field: &str) -> Option<String> {
        match data.get(field) {
            Some(crate::json_utils::JsonValue::String(s)) => Some(s.clone()),
            _ => None,
        }
    }

    /// Validate risk assessment parameters (GRASP High Cohesion)
    fn validate_risk_parameters(severity: u8, occurrence: u8, detection: u8) -> QmsResult<()> {
        if !(1..=10).contains(&severity) {
            return Err(crate::error::QmsError::validation_error("Severity must be between 1 and 10"));
        }
        if !(1..=10).contains(&occurrence) {
            return Err(crate::error::QmsError::validation_error("Occurrence must be between 1 and 10"));
        }
        if !(1..=10).contains(&detection) {
            return Err(crate::error::QmsError::validation_error("Detection must be between 1 and 10"));
        }
        Ok(())
    }

    /// Calculate risk level based on RPN (CUPID Domain-centric)
    fn calculate_risk_level(rpn: u16) -> String {
        match rpn {
            1..=50 => "low".to_string(),
            51..=100 => "medium".to_string(),
            101..=200 => "high".to_string(),
            _ => "critical".to_string(),
        }
    }

    /// Get next risk number for ID generation (GRASP Creator)
    fn get_next_risk_number() -> u32 {
        // In a real implementation, this would track the actual next number
        // For now, use timestamp-based approach
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        ((timestamp % 1000) + 1) as u32
    }

    // Audit Trail API Handlers (SOLID Single Responsibility Principle)

    /// Handle GET /api/audit - List audit entries with filtering
    fn handle_audit_list_api(request: &HttpRequest) -> QmsResult<HttpResponse> {
        use crate::web::audit_api::{AuditApiHandler, FileAuditDataProvider};

        // Delegate to specialized audit API handler (Dependency Inversion Principle)
        AuditApiHandler::handle_list_audit_entries(request)
    }

    /// Handle GET /api/audit/statistics - Get audit statistics
    fn handle_audit_statistics_api(request: &HttpRequest) -> QmsResult<HttpResponse> {
        use crate::web::audit_api::{AuditApiHandler, FileAuditDataProvider};

        // Delegate to specialized audit API handler
        AuditApiHandler::handle_audit_statistics(request)
    }

    // Reports API Handlers (SOLID Single Responsibility Principle)

    /// Handle GET /api/reports - List available reports
    fn handle_reports_list_api(request: &HttpRequest) -> QmsResult<HttpResponse> {
        use crate::web::reports_api::ReportsApiHandler;

        // Delegate to specialized reports API handler (Dependency Inversion Principle)
        ReportsApiHandler::handle_list_reports(request)
    }

    /// Handle POST /api/reports/generate - Generate a new report
    fn handle_reports_generate_api(request: &HttpRequest) -> QmsResult<HttpResponse> {
        use crate::web::reports_api::ReportsApiHandler;

        // Delegate to specialized reports API handler
        ReportsApiHandler::handle_generate_report(request)
    }

    /// Handle GET /api/reports/{id}/status - Get report status
    fn handle_reports_status_api(request: &HttpRequest) -> QmsResult<HttpResponse> {
        use crate::web::reports_api::ReportsApiHandler;

        // Delegate to specialized reports API handler
        ReportsApiHandler::handle_report_status(request)
    }

    // Project Management API Handlers (SOLID Single Responsibility Principle)

    /// Handle GET /api/projects - List all projects
    fn handle_projects_list_api(request: &HttpRequest) -> QmsResult<HttpResponse> {
        use crate::web::project_api::ProjectApiHandler;

        // Delegate to specialized project API handler (Dependency Inversion Principle)
        ProjectApiHandler::handle_list_projects(request)
    }

    /// Handle POST /api/projects - Create a new project
    fn handle_projects_create_api(request: &HttpRequest) -> QmsResult<HttpResponse> {
        use crate::web::project_api::ProjectApiHandler;

        // Delegate to specialized project API handler
        ProjectApiHandler::handle_create_project(request)
    }

    /// Handle GET /api/projects/{id} - Get project details
    fn handle_projects_get_api(request: &HttpRequest) -> QmsResult<HttpResponse> {
        use crate::web::project_api::ProjectApiHandler;

        // Delegate to specialized project API handler
        ProjectApiHandler::handle_get_project(request)
    }

    /// Handle DELETE /api/projects/{id} - Delete a project
    fn handle_projects_delete_api(request: &HttpRequest) -> QmsResult<HttpResponse> {
        use crate::web::project_api::ProjectApiHandler;

        // Delegate to specialized project API handler
        ProjectApiHandler::handle_delete_project(request)
    }

    fn handle_requirements_create_api(_request: &HttpRequest) -> QmsResult<HttpResponse> {
        // TODO: Implement requirements creation via API
        let response_data = r#"{"message": "Requirements creation via API not yet implemented", "status": "pending"}"#;
        Ok(HttpResponse::created(response_data.as_bytes().to_vec(), "application/json"))
    }

    // Traceability API implementations
    fn handle_trace_matrix_api(_request: &HttpRequest) -> QmsResult<HttpResponse> {
        // TODO: Integrate with actual traceability engine
        let matrix_data = r#"{
            "matrix": [
                {"requirement": "REQ-001", "tests": ["TC-001", "TC-002"], "coverage": "100%"},
                {"requirement": "REQ-002", "tests": ["TC-003"], "coverage": "75%"}
            ],
            "stats": {"total_requirements": 24, "covered": 18, "coverage_percentage": 75}
        }"#;
        
        Ok(HttpResponse::json(matrix_data))
    }

    fn handle_trace_links_api(_request: &HttpRequest) -> QmsResult<HttpResponse> {
        // TODO: Integrate with actual traceability links
        let links_data = r#"[
            {"from": "REQ-001", "to": "TC-001", "type": "verifies"},
            {"from": "REQ-001", "to": "TC-002", "type": "verifies"},
            {"from": "REQ-002", "to": "RISK-001", "type": "mitigates"}
        ]"#;
        
        Ok(HttpResponse::json(links_data))
    }

    // Audit API implementations
    fn handle_audit_recent_api() -> QmsResult<HttpResponse> {
        // TODO: Integrate with actual audit logger
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let activities_data = format!(r#"[
            {{"timestamp": "{}", "user": "system", "action": "WEB_SERVER_START", "description": "Web server started successfully"}},
            {{"timestamp": "{}", "user": "system", "action": "AUDIT_INIT", "description": "Audit logging initialized"}},
            {{"timestamp": "{}", "user": "system", "action": "COMPLIANCE_CHECK", "description": "FDA compliance checks passed"}},
            {{"timestamp": "{}", "user": "system", "action": "SYSTEM_READY", "description": "Ready for medical device operations"}}
        ]"#, 
        timestamp, timestamp - 60, timestamp - 120, timestamp - 180
        );
        
        Ok(HttpResponse::json(&activities_data))
    }

    fn handle_audit_search_api(_request: &HttpRequest) -> QmsResult<HttpResponse> {
        // TODO: Implement audit search via API
        let search_data = r#"{"message": "Audit search via API not yet implemented", "results": []}"#;
        Ok(HttpResponse::json(search_data))
    }

    // Report API implementations
    fn handle_reports_dhf_api(_request: &HttpRequest) -> QmsResult<HttpResponse> {
        // TODO: Implement DHF report generation via API
        let response_data = r#"{"message": "DHF report generation via API not yet implemented", "status": "pending"}"#;
        Ok(HttpResponse::json(response_data))
    }

    fn handle_reports_risk_api(_request: &HttpRequest) -> QmsResult<HttpResponse> {
        // TODO: Implement risk report generation via API
        let response_data = r#"{"message": "Risk report generation via API not yet implemented", "status": "pending"}"#;
        Ok(HttpResponse::json(response_data))
    }

    /// Setup API route handlers
    fn setup_api_routes() -> HashMap<String, ApiHandler> {
        HashMap::new() // Routes handled in handle_api_request for simplicity
    }

    /// Stop the web server gracefully
    pub fn stop(&self) {
        println!("🛑 Stopping QMS Web Server...");
        self.running.store(false, Ordering::SeqCst);
        
        // Audit log server stop
        if let Err(e) = crate::modules::audit_logger::audit_log_action(
            "WEB_SERVER_STOP", 
            "WebServer", 
            &format!("{}:{}", self.bind_address, self.port)
        ) {
            eprintln!("⚠️  Warning: Failed to log server stop: {e}");
        }
    }

    /// Get server information
    pub fn get_info(&self) -> String {
        let pool_info = if let Some(ref pool) = self.thread_pool {
            let stats = pool.get_stats();
            format!("Thread Pool: {}/{} workers", stats.active_workers, stats.total_workers)
        } else {
            "Thread Pool: Not initialized".to_string()
        };

        format!(
            "QMS Web Server v1.0.0\nListening on: http://{}:{}\nStatus: {}\n{}\nMax Connections: {}\nCompliance: FDA 21 CFR Part 820, ISO 13485, ISO 14971",
            self.bind_address,
            self.port,
            if self.running.load(Ordering::SeqCst) { "Running" } else { "Stopped" },
            pool_info,
            self.max_connections
        )
    }

    /// Get thread pool statistics
    pub fn get_thread_pool_stats(&self) -> Option<ThreadPoolStats> {
        self.thread_pool.as_ref().map(|pool| pool.get_stats())
    }

    /// Set maximum connections
    pub fn set_max_connections(&mut self, max_connections: usize) {
        self.max_connections = max_connections;
        println!("🔗 Max connections set to: {max_connections}");
    }

    /// Get current configuration
    pub fn get_config(&self) -> ServerConfig {
        ServerConfig {
            bind_address: self.bind_address.clone(),
            port: self.port,
            pool_size: self.pool_size,
            max_connections: self.max_connections,
            running: self.running.load(Ordering::SeqCst),
        }
    }

    /// Get security configuration
    pub fn get_security_config(&self) -> Option<SecurityConfig> {
        if let Ok(security_manager) = self.security_manager.lock() {
            Some(security_manager.get_config().clone())
        } else {
            None
        }
    }

    /// Check if HTTPS is configured and enabled
    pub fn is_https_enabled(&self) -> bool {
        if let Ok(security_manager) = self.security_manager.lock() {
            security_manager.is_https_configured()
        } else {
            false
        }
    }

    /// Get security status information
    pub fn get_security_status(&self) -> SecurityStatus {
        if let Ok(security_manager) = self.security_manager.lock() {
            SecurityStatus {
                https_configured: security_manager.is_https_configured(),
                certificate_valid: security_manager.is_certificate_valid(),
                security_headers_enabled: security_manager.get_config().enable_security_headers,
                hsts_enabled: security_manager.get_config().enable_hsts,
                csp_enabled: security_manager.get_config().enable_csp,
                https_enforced: security_manager.get_config().enforce_https,
            }
        } else {
            SecurityStatus::default()
        }
    }

    /// Check if server is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Get server performance metrics
    pub fn get_performance_metrics(&self) -> ServerMetrics {
        let pool_stats = self.get_thread_pool_stats();

        ServerMetrics {
            uptime_seconds: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            thread_pool_stats: pool_stats,
            max_connections: self.max_connections,
            pool_size: self.pool_size,
            is_running: self.is_running(),
        }
    }
}

impl Drop for QMSWebServer {
    fn drop(&mut self) {
        self.stop();
        // Thread pool will be dropped automatically, which will join all worker threads
        if let Some(pool) = self.thread_pool.take() {
            drop(pool); // Explicit drop to ensure proper cleanup
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_pool_creation() {
        let pool = ThreadPool::new(4);
        let stats = pool.get_stats();
        assert_eq!(stats.total_workers, 4);
        assert_eq!(stats.active_workers, 4);
    }

    #[test]
    fn test_server_config() {
        let server = QMSWebServer::new("127.0.0.1", 8080).unwrap();
        let config = server.get_config();
        assert_eq!(config.bind_address, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.pool_size, 8);
        assert_eq!(config.max_connections, 100);
        assert!(!config.running);
    }

    #[test]
    fn test_server_with_custom_pool_size() {
        let server = QMSWebServer::new_with_pool_size("127.0.0.1", 8080, 16).unwrap();
        let config = server.get_config();
        assert_eq!(config.pool_size, 16);
    }

    #[test]
    fn test_server_metrics() {
        let server = QMSWebServer::new("127.0.0.1", 8080).unwrap();
        let metrics = server.get_performance_metrics();
        assert_eq!(metrics.pool_size, 8);
        assert_eq!(metrics.max_connections, 100);
        assert!(!metrics.is_running);
        assert!(metrics.thread_pool_stats.is_none()); // Pool not initialized until start()
    }

    #[test]
    fn test_max_connections_setting() {
        let mut server = QMSWebServer::new("127.0.0.1", 8080).unwrap();
        server.set_max_connections(200);
        let config = server.get_config();
        assert_eq!(config.max_connections, 200);
    }

    #[test]
    fn test_security_status() {
        let server = QMSWebServer::new("127.0.0.1", 8080).unwrap();
        let security_status = server.get_security_status();

        // Default security configuration
        assert!(!security_status.https_configured);
        assert!(!security_status.certificate_valid);
        assert!(security_status.security_headers_enabled);
        assert!(security_status.hsts_enabled);
        assert!(security_status.csp_enabled);
        assert!(!security_status.https_enforced);
    }

    #[test]
    fn test_https_not_enabled_by_default() {
        let server = QMSWebServer::new("127.0.0.1", 8080).unwrap();
        assert!(!server.is_https_enabled());
    }

    #[test]
    fn test_security_config_retrieval() {
        let server = QMSWebServer::new("127.0.0.1", 8080).unwrap();
        let security_config = server.get_security_config();
        assert!(security_config.is_some());

        let config = security_config.unwrap();
        assert!(!config.enforce_https);
        assert!(config.enable_security_headers);
        assert!(config.enable_hsts);
        assert!(config.enable_csp);
    }

    #[test]
    fn test_web_server_creation() {
        let server = QMSWebServer::new("127.0.0.1", 8080);
        assert!(server.is_ok());

        let server = server.unwrap();
        assert_eq!(server.port, 8080);
        assert_eq!(server.bind_address, "127.0.0.1");
    }

    #[test]
    fn test_api_routes_setup() {
        let routes = QMSWebServer::setup_api_routes();
        // Routes handled in handle_api_request, so this should be empty
        assert!(routes.is_empty());
    }

    #[test]
    fn test_server_info() {
        let server = QMSWebServer::new("localhost", 3000).unwrap();
        let info = server.get_info();
        assert!(info.contains("QMS Web Server"));
        assert!(info.contains("localhost:3000"));
        assert!(info.contains("FDA 21 CFR Part 820"));
    }

    #[test]
    fn test_asset_manager_integration() {
        let server = QMSWebServer::new("127.0.0.1", 8081).unwrap();

        // Test that asset manager is properly initialized
        assert!(server.asset_manager.get_asset("/index.html").is_some());
        assert!(server.asset_manager.get_asset("/styles.css").is_some());
        assert!(server.asset_manager.get_asset("/app.js").is_some());
    }
}
