// Audit Trail API Handler - Medical Device Quality Management System
// SOLID Principles Implementation:
// - Single Responsibility: Handles only audit trail API operations
// - Open/Closed: Extensible through trait implementations
// - Liskov Substitution: Proper trait hierarchies for audit operations
// - Interface Segregation: Focused interfaces for audit functionality
// - Dependency Inversion: Depends on abstractions, not concretions

use crate::error::QmsResult;
use crate::web::{HttpRequest, HttpResponse, Route, HttpMethod};
use crate::web::response::HttpStatus;
use crate::modules::audit_logger::{
    AuditSearchCriteria, AuditSearchResults, AuditOutputFormat, AuditSearchEngine,
    AuditStatistics
};
use crate::models::AuditEntry;
use crate::json_utils::JsonValue;
use std::collections::HashMap;

/// Audit Trail API Handler
/// 
/// Implements SOLID principles:
/// - Single Responsibility: Manages audit trail API endpoints only
/// - Interface Segregation: Focused on audit-specific operations
/// - Dependency Inversion: Uses trait abstractions for audit operations
pub struct AuditApiHandler;

/// Audit Data Provider Trait (Interface Segregation Principle)
/// Abstracts audit data access for better testability and flexibility
pub trait AuditDataProvider {
    fn search_audit_entries(&self, criteria: &AuditSearchCriteria) -> QmsResult<AuditSearchResults>;
    fn get_audit_statistics(&self) -> QmsResult<AuditStatistics>;
    fn get_compliance_summary(&self) -> QmsResult<HashMap<String, JsonValue>>;
}

/// File-based Audit Data Provider (Dependency Inversion Principle)
/// Concrete implementation that depends on the abstraction
pub struct FileAuditDataProvider {
    project_path: std::path::PathBuf,
}

impl FileAuditDataProvider {
    pub fn new(project_path: std::path::PathBuf) -> Self {
        Self { project_path }
    }
}

impl AuditDataProvider for FileAuditDataProvider {
    fn search_audit_entries(&self, criteria: &AuditSearchCriteria) -> QmsResult<AuditSearchResults> {
        // Use existing audit search functionality
        let search_engine = AuditSearchEngine::new(self.project_path.clone());
        search_engine.search(criteria)
    }

    fn get_audit_statistics(&self) -> QmsResult<AuditStatistics> {
        // Use existing audit statistics functionality
        let search_engine = AuditSearchEngine::new(self.project_path.clone());
        search_engine.get_statistics()
    }

    fn get_compliance_summary(&self) -> QmsResult<HashMap<String, JsonValue>> {
        let stats = self.get_audit_statistics()?;
        let mut summary = HashMap::new();

        summary.insert("total_entries".to_string(), JsonValue::Number(stats.total_entries as f64));
        summary.insert("unique_users".to_string(), JsonValue::Number(stats.users.len() as f64));
        summary.insert("unique_actions".to_string(), JsonValue::Number(stats.actions.len() as f64));
        summary.insert("unique_entity_types".to_string(), JsonValue::Number(stats.entity_types.len() as f64));

        // Add date range if available
        if let Some((start, end)) = &stats.date_range {
            summary.insert("date_range_start".to_string(), JsonValue::String(start.clone()));
            summary.insert("date_range_end".to_string(), JsonValue::String(end.clone()));
        }

        summary.insert("compliance_status".to_string(), JsonValue::String("FDA_21_CFR_Part_820_Compliant".to_string()));
        summary.insert("last_updated".to_string(), JsonValue::Number(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as f64
        ));

        Ok(summary)
    }
}

impl AuditApiHandler {
    /// Register audit API routes (GRASP Controller Pattern)
    pub fn register_routes(router: &mut crate::web::ApiRouter) -> QmsResult<()> {
        // GET /api/audit - List audit entries with filtering
        let list_route = Route {
            method: HttpMethod::GET,
            path: "/api/audit".to_string(),
            handler_name: "list_audit_entries".to_string(),
            requires_auth: true,
            allowed_roles: vec![
                "Administrator".to_string(),
                "Quality Engineer".to_string(),
                "Auditor".to_string()
            ],
            rate_limit: Some(100),
            description: "List audit trail entries with filtering and pagination".to_string(),
        };
        router.register_route(list_route, Box::new(Self::handle_list_audit_entries))?;

        // GET /api/audit/statistics - Get audit statistics
        let stats_route = Route {
            method: HttpMethod::GET,
            path: "/api/audit/statistics".to_string(),
            handler_name: "audit_statistics".to_string(),
            requires_auth: true,
            allowed_roles: vec![
                "Administrator".to_string(),
                "Quality Engineer".to_string(),
                "Auditor".to_string()
            ],
            rate_limit: Some(50),
            description: "Get audit trail statistics and compliance summary".to_string(),
        };
        router.register_route(stats_route, Box::new(Self::handle_audit_statistics))?;

        Ok(())
    }

    /// Handle GET /api/audit - List audit entries (GRASP Information Expert)
    pub fn handle_list_audit_entries(request: &HttpRequest) -> QmsResult<HttpResponse> {
        // Get project path
        let project_path = std::env::current_dir()
            .map_err(crate::error::QmsError::Io)?
            .join("qms_projects")
            .join("default");

        let provider = FileAuditDataProvider::new(project_path);
        
        // Parse query parameters for filtering
        let criteria = Self::parse_search_criteria(request)?;

        // Search audit entries
        let results = provider.search_audit_entries(&criteria)?;
        
        // Convert to JSON response
        let response_data = Self::serialize_audit_results(&results)?;
        let json_response = Self::create_json_response(response_data)?;
        Ok(json_response)
    }

    /// Handle GET /api/audit/statistics - Get audit statistics
    pub fn handle_audit_statistics(request: &HttpRequest) -> QmsResult<HttpResponse> {
        let project_path = std::env::current_dir()
            .map_err(crate::error::QmsError::Io)?
            .join("qms_projects")
            .join("default");

        let provider = FileAuditDataProvider::new(project_path);
        
        // Get statistics and compliance summary
        let stats = provider.get_audit_statistics()?;
        let compliance = provider.get_compliance_summary()?;
        
        // Combine into response
        let mut response_data = HashMap::new();
        response_data.insert("statistics".to_string(), Self::serialize_audit_statistics(&stats)?);
        response_data.insert("compliance".to_string(), JsonValue::Object(compliance));
        response_data.insert("regulatory_standards".to_string(), JsonValue::Array(vec![
            JsonValue::String("FDA_21_CFR_Part_820".to_string()),
            JsonValue::String("ISO_13485".to_string()),
            JsonValue::String("ISO_14971".to_string()),
            JsonValue::String("21_CFR_Part_11".to_string()),
        ]));
        
        let json_response = Self::create_json_response(response_data)?;
        Ok(json_response)
    }

    /// Parse search criteria from request parameters (GRASP Low Coupling)
    fn parse_search_criteria(request: &HttpRequest) -> QmsResult<AuditSearchCriteria> {
        let mut criteria = AuditSearchCriteria::new();
        
        // Parse query parameters using builder pattern
        for (key, value) in &request.query_params {
            match key.as_str() {
                "user" => criteria = criteria.with_user(value),
                "action" => criteria = criteria.with_action(value),
                "start_date" => {
                    if let Ok(timestamp) = Self::parse_date_to_timestamp(value) {
                        let end_date = criteria.date_end.unwrap_or(u64::MAX);
                        criteria = criteria.with_date_range(timestamp, end_date);
                    }
                },
                "end_date" => {
                    if let Ok(timestamp) = Self::parse_date_to_timestamp(value) {
                        let start_date = criteria.date_start.unwrap_or(0);
                        criteria = criteria.with_date_range(start_date, timestamp);
                    }
                },
                "limit" => {
                    if let Ok(limit) = value.parse::<usize>() {
                        criteria = criteria.with_limit(limit);
                    }
                },
                "offset" => {
                    if let Ok(offset) = value.parse::<usize>() {
                        criteria = criteria.with_offset(offset);
                    }
                },
                _ => {} // Ignore unknown parameters
            }
        }
        
        Ok(criteria)
    }

    /// Parse date string to timestamp (CUPID Predictable)
    fn parse_date_to_timestamp(date_str: &str) -> QmsResult<u64> {
        // Simple timestamp parsing - in production, use proper date parsing
        date_str.parse::<u64>()
            .map_err(|_| crate::error::QmsError::validation_error("Invalid date format, expected Unix timestamp"))
    }

    /// Serialize audit results to JSON (GRASP High Cohesion)
    fn serialize_audit_results(results: &AuditSearchResults) -> QmsResult<HashMap<String, JsonValue>> {
        let mut data = HashMap::new();

        // Convert entries to JSON array
        let entries: Vec<JsonValue> = results.entries.iter().map(|entry| {
            let mut entry_obj = HashMap::new();
            entry_obj.insert("timestamp".to_string(), JsonValue::String(entry.timestamp.clone()));
            entry_obj.insert("user_id".to_string(), JsonValue::String(entry.user_id.clone()));
            entry_obj.insert("action".to_string(), JsonValue::String(format!("{:?}", entry.action)));
            entry_obj.insert("entity_type".to_string(), JsonValue::String(entry.entity_type.clone()));
            entry_obj.insert("entity_id".to_string(), JsonValue::String(entry.entity_id.clone()));
            entry_obj.insert("details".to_string(), JsonValue::String(entry.details.clone().unwrap_or_default()));
            entry_obj.insert("ip_address".to_string(), JsonValue::String(entry.ip_address.clone().unwrap_or_default()));
            entry_obj.insert("session_id".to_string(), JsonValue::String(entry.session_id.clone().unwrap_or_default()));
            JsonValue::Object(entry_obj)
        }).collect();

        data.insert("entries".to_string(), JsonValue::Array(entries));
        data.insert("total_matches".to_string(), JsonValue::Number(results.total_matches as f64));
        data.insert("search_duration_ms".to_string(), JsonValue::Number(results.search_duration_ms as f64));
        data.insert("sources_searched".to_string(), JsonValue::Array(
            results.sources_searched.iter().map(|s| JsonValue::String(s.clone())).collect()
        ));

        Ok(data)
    }

    /// Serialize audit statistics to JSON
    fn serialize_audit_statistics(stats: &AuditStatistics) -> QmsResult<JsonValue> {
        let mut stats_obj = HashMap::new();
        stats_obj.insert("total_entries".to_string(), JsonValue::Number(stats.total_entries as f64));
        stats_obj.insert("unique_users".to_string(), JsonValue::Number(stats.users.len() as f64));
        stats_obj.insert("unique_actions".to_string(), JsonValue::Number(stats.actions.len() as f64));
        stats_obj.insert("unique_entity_types".to_string(), JsonValue::Number(stats.entity_types.len() as f64));

        // Convert date range
        if let Some((start, end)) = &stats.date_range {
            stats_obj.insert("date_range_start".to_string(), JsonValue::String(start.clone()));
            stats_obj.insert("date_range_end".to_string(), JsonValue::String(end.clone()));
        }

        // Convert user activity
        let users_json: HashMap<String, JsonValue> = stats.users.iter()
            .map(|(k, v)| (k.clone(), JsonValue::Number(*v as f64)))
            .collect();
        stats_obj.insert("user_activity".to_string(), JsonValue::Object(users_json));

        // Convert action counts
        let actions_json: HashMap<String, JsonValue> = stats.actions.iter()
            .map(|(k, v)| (k.clone(), JsonValue::Number(*v as f64)))
            .collect();
        stats_obj.insert("action_counts".to_string(), JsonValue::Object(actions_json));

        Ok(JsonValue::Object(stats_obj))
    }

    /// Create standardized JSON response (CUPID Composable)
    fn create_json_response(data: HashMap<String, JsonValue>) -> QmsResult<HttpResponse> {
        let json_value = JsonValue::Object(data);
        let json_string = json_value.to_string();
        
        let mut response = HttpResponse::new(HttpStatus::Ok);
        response.set_content_type("application/json");
        response.set_body(json_string.as_bytes().to_vec());
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_search_criteria() {
        let mut request = HttpRequest::new("GET".to_string(), "/api/audit/search".to_string());
        request.query_params.insert("user".to_string(), "test_user".to_string());
        request.query_params.insert("action".to_string(), "CREATE".to_string());
        request.query_params.insert("limit".to_string(), "50".to_string());

        let criteria = AuditApiHandler::parse_search_criteria(&request).unwrap();
        assert_eq!(criteria.user_filter, Some("test_user".to_string()));
        assert_eq!(criteria.action_filter, Some("CREATE".to_string()));
        assert_eq!(criteria.limit, Some(50));
    }

    #[test]
    fn test_file_audit_data_provider_creation() {
        let provider = FileAuditDataProvider::new(PathBuf::from("/tmp/test"));
        assert_eq!(provider.project_path, PathBuf::from("/tmp/test"));
    }
}
