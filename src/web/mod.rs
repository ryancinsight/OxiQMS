// QMS Web Module
// Provides HTTP server and web interface functionality using stdlib only

#[allow(dead_code)]
pub mod server;
#[allow(dead_code)]
pub mod request;
#[allow(dead_code)]
pub mod response;
#[allow(dead_code)]
pub mod assets;
#[allow(dead_code)]
pub mod security;
#[allow(dead_code)]
pub mod error;
#[allow(dead_code)]
pub mod document_api;
#[allow(dead_code)]
// Note: Removed unused user_api module (YAGNI Applied)
#[allow(dead_code)]
pub mod api_response;
#[allow(dead_code)]
pub mod cli_bridge;
#[allow(dead_code)]
pub mod command_bridge;
#[allow(dead_code)]
pub mod json_response_adapters;
#[allow(dead_code)]
pub mod unified_auth_context;
#[allow(dead_code)]
pub mod unified_document_api;
#[allow(dead_code)]
pub mod unified_risk_api;
#[allow(dead_code)]
pub mod unified_requirements_api;
#[allow(dead_code)]
pub mod unified_audit_api;
#[allow(dead_code)]
pub mod http_parser;
#[allow(dead_code)]
pub mod audit_api;
#[allow(dead_code)]
pub mod reports_api;
#[allow(dead_code)]
pub mod project_api;
#[allow(dead_code)]
pub mod auth_api;
#[allow(dead_code)]
pub mod unified_session_adapter;

pub use request::HttpRequest;
pub use response::HttpResponse;
pub use security::{SecurityManager, SecurityConfig};
// Note: Removed unused SessionManager - using UnifiedSessionAdapter instead (SOLID/YAGNI Applied)

// Error handling types - will be used in the next phase
#[allow(unused_imports)]
pub use error::{WebErrorHandler, WebLogConfig, RequestMetrics};



// Document API types - REST endpoints for document CRUD operations
#[allow(unused_imports)]
pub use document_api::DocumentApiHandler;

// Note: Removed unused UserApiHandler - using unified authentication system instead (YAGNI Applied)

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

// Command Bridge types - Unified CLI-Web command execution bridge
#[allow(unused_imports)]
pub use command_bridge::{
    WebCommandBridge, WebCommandContext, CommandResult, JsonValue, CommandArgumentParser
};

// JSON Response Adapters - Convert CLI outputs to structured JSON for web API
#[allow(unused_imports)]
pub use json_response_adapters::{
    ResponseAdapter, ResponseAdapterRegistry, DocumentResponseAdapter, RiskResponseAdapter,
    RequirementsResponseAdapter, AuditResponseAdapter
};

// Unified Authentication Context - Shared authentication context for CLI and Web
#[allow(unused_imports)]
pub use unified_auth_context::{
    UnifiedAuthContext, AuthContextType, AuthContextManager, AuthenticatedCommand
};

// Unified Document API - Web routes that delegate to CLI command infrastructure
#[allow(unused_imports)]
pub use unified_document_api::{
    UnifiedDocumentApiHandler, UnifiedDocumentRoutes
};

// Unified Risk API - Web routes that delegate to CLI command infrastructure
#[allow(unused_imports)]
pub use unified_risk_api::{
    UnifiedRiskApiHandler, UnifiedRiskRoutes
};

// Unified Requirements API - Web routes that delegate to CLI command infrastructure
#[allow(unused_imports)]
pub use unified_requirements_api::{
    UnifiedRequirementsApiHandler, UnifiedRequirementsRoutes
};

// Unified Audit API - Web routes that delegate to CLI command infrastructure
#[allow(unused_imports)]
pub use unified_audit_api::{
    UnifiedAuditApiHandler, UnifiedAuditRoutes
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

// Authentication API types - REST endpoints for user-first authentication flow
#[allow(unused_imports)]
pub use auth_api::AuthApiHandler;

// Unified Session Adapter - Bridges web server with unified authentication service
#[allow(unused_imports)]
pub use unified_session_adapter::UnifiedSessionAdapter;
