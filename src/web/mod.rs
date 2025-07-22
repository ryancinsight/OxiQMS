// QMS Web Module
// Provides HTTP server and web interface functionality using stdlib only

#[allow(dead_code)]
pub mod server;
#[allow(dead_code)]
pub mod request;
#[allow(dead_code)]
pub mod response;
#[allow(dead_code)]
pub mod session;
#[allow(dead_code)]
pub mod assets;
#[allow(dead_code)]
pub mod security;
#[allow(dead_code)]
pub mod error;
#[allow(dead_code)]
pub mod router;
#[allow(dead_code)]
pub mod document_api;
#[allow(dead_code)]
pub mod document_api_example;
#[allow(dead_code)]
pub mod user_api;
#[allow(dead_code)]
pub mod user_api_example;
#[allow(dead_code)]
pub mod api_response;
#[allow(dead_code)]
pub mod cli_bridge;
#[allow(dead_code)]
pub mod http_parser;
#[allow(dead_code)]
pub mod audit_api;
#[allow(dead_code)]
pub mod reports_api;
#[allow(dead_code)]
pub mod project_api;

pub use request::HttpRequest;
pub use response::HttpResponse;
pub use session::SessionManager;
pub use security::{SecurityManager, SecurityConfig};

// Error handling types - will be used in the next phase
#[allow(unused_imports)]
pub use error::{WebErrorHandler, WebLogConfig, RequestMetrics};

// API Router types - core HTTP request routing for /api/v1/* endpoints
#[allow(unused_imports)]
pub use router::{ApiRouter, Route, HttpMethod, RouteHandler};

// Document API types - REST endpoints for document CRUD operations
#[allow(unused_imports)]
pub use document_api::DocumentApiHandler;

// User API types - REST endpoints for user management and authentication
#[allow(unused_imports)]
pub use user_api::UserApiHandler;

// API Response types - Standardized JSON response wrapper system for CLI-to-API bridging
#[allow(unused_imports)]
pub use api_response::{
    StandardApiResponse, ApiResponseBuilder, ApiError, ApiMetadata, ComplianceMetadata, PaginationMetadata,
    wrap_success, wrap_error, wrap_command_output, wrap_success_with_pagination, wrap_paginated_list,
    wrap_success_message, wrap_empty_success, invalid_request_error, unauthorized_error, 
    forbidden_error, not_found_error, internal_server_error
};

// CLI Bridge types - Convert CLI commands to JSON API responses with medical device compliance
#[allow(unused_imports)]
pub use cli_bridge::{
    DocumentOperations, RiskOperations, RequirementOperations, CliCommandBridge, HealthApiBridge
};

// HTTP Parser types - Parse HTTP requests and build responses with medical device security
#[allow(unused_imports)]
pub use http_parser::{
    HttpParser, HttpMethod as ParserHttpMethod, HttpStatus, HttpRequest as ParsedHttpRequest,
    HttpResponse as ParsedHttpResponse, ParserConfig
};

// Audit API types - REST endpoints for audit trail management and compliance
#[allow(unused_imports)]
pub use audit_api::{AuditApiHandler, AuditDataProvider, FileAuditDataProvider};

// Reports API types - REST endpoints for medical device compliance report generation
#[allow(unused_imports)]
pub use reports_api::{ReportsApiHandler, ReportProvider, MedicalDeviceReportProvider, ReportData, ReportMetadata, ReportStatus};

// Project API types - REST endpoints for QMS project management
#[allow(unused_imports)]
pub use project_api::{ProjectApiHandler, ProjectProvider, MedicalDeviceProjectProvider};
