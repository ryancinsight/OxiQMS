//! CLI-to-API Bridge Module
//! Phase 7.2.4 Implementation - Examples and integration helpers
//! 
//! This module demonstrates how to use the API response wrapper system
//! to convert existing CLI command outputs to JSON API responses.

use crate::web::api_response::*;
use crate::error::QmsResult;
use crate::json_utils::JsonSerializable;

/// Generic CLI Command Bridge - Wraps any CLI command output as JSON
pub struct CliCommandBridge;

impl CliCommandBridge {
    /// Execute a CLI command and wrap result as JSON API response
    pub fn execute_command<F, T>(command_fn: F, operation: &str, entity_type: &str, entity_id: &str) -> Result<String, String>
    where
        F: FnOnce() -> QmsResult<T>,
        T: JsonSerializable,
    {
        let result = command_fn();
        wrap_command_output_with_audit(result, operation, entity_type, entity_id)
    }

    /// Execute a CLI command that returns a simple message
    pub fn execute_message_command<F>(command_fn: F) -> Result<String, String>
    where
        F: FnOnce() -> QmsResult<String>,
    {
        match command_fn() {
            Ok(message) => wrap_success_message(&message),
            Err(error) => wrap_error(error),
        }
    }

    /// Execute a CLI command that returns a list with pagination
    pub fn execute_list_command<F, T>(
        command_fn: F,
        page: usize,
        per_page: usize,
    ) -> Result<String, String>
    where
        F: FnOnce() -> QmsResult<Vec<T>>,
        T: JsonSerializable + Clone,
    {
        match command_fn() {
            Ok(items) => {
                let total_items = items.len();
                let start_idx = (page - 1) * per_page;
                let end_idx = std::cmp::min(start_idx + per_page, total_items);
                
                let page_items = if start_idx < total_items {
                    items[start_idx..end_idx].to_vec()
                } else {
                    vec![]
                };
                
                wrap_paginated_list(page_items, page, per_page, total_items)
            },
            Err(error) => wrap_error(error),
        }
    }
}

/// Document Operations Bridge - Simplified API for document operations
pub struct DocumentOperations;

impl DocumentOperations {
    /// List documents as JSON API response with pagination
    pub fn list_documents_json(page: Option<usize>, per_page: Option<usize>) -> Result<String, String> {
        let page = page.unwrap_or(1);
        let per_page = per_page.unwrap_or(20);
        
        // Mock document list for demonstration
        use crate::json_utils::JsonValue;
        use std::collections::HashMap;
        
        let mut doc1 = HashMap::new();
        doc1.insert("id".to_string(), JsonValue::String("DOC001".to_string()));
        doc1.insert("title".to_string(), JsonValue::String("System Requirements".to_string()));
        doc1.insert("type".to_string(), JsonValue::String("Requirement".to_string()));
        doc1.insert("status".to_string(), JsonValue::String("Approved".to_string()));
        
        let mut doc2 = HashMap::new();
        doc2.insert("id".to_string(), JsonValue::String("DOC002".to_string()));
        doc2.insert("title".to_string(), JsonValue::String("Risk Assessment".to_string()));
        doc2.insert("type".to_string(), JsonValue::String("Risk".to_string()));
        doc2.insert("status".to_string(), JsonValue::String("Draft".to_string()));
        
        let documents = vec![
            JsonValue::Object(doc1),
            JsonValue::Object(doc2),
        ];
        
        wrap_paginated_list(documents, page, per_page, 2)
    }

    /// Create document operation result as JSON
    pub fn create_document_json(title: &str, doc_type: &str) -> Result<String, String> {
        use crate::json_utils::JsonValue;
        use std::collections::HashMap;
        
        let mut doc = HashMap::new();
        doc.insert("id".to_string(), JsonValue::String("DOC003".to_string()));
        doc.insert("title".to_string(), JsonValue::String(title.to_string()));
        doc.insert("type".to_string(), JsonValue::String(doc_type.to_string()));
        doc.insert("status".to_string(), JsonValue::String("Draft".to_string()));
        doc.insert("created_at".to_string(), JsonValue::String("2024-01-20T10:30:00Z".to_string()));
        
        wrap_success(JsonValue::Object(doc))
    }

    /// Delete document operation result as JSON  
    pub fn delete_document_json(doc_id: &str) -> Result<String, String> {
        // Mock successful deletion
        wrap_success_message(&format!("Document {doc_id} deleted successfully"))
    }
}

/// Risk Operations Bridge - Simplified API for risk operations
pub struct RiskOperations;

impl RiskOperations {
    /// List risks as JSON API response with pagination
    pub fn list_risks_json(page: Option<usize>, per_page: Option<usize>) -> Result<String, String> {
        let page = page.unwrap_or(1);
        let per_page = per_page.unwrap_or(20);
        
        // Mock risk list for demonstration
        use crate::json_utils::JsonValue;
        use std::collections::HashMap;
        
        let mut risk1 = HashMap::new();
        risk1.insert("id".to_string(), JsonValue::String("RISK001".to_string()));
        risk1.insert("hazard".to_string(), JsonValue::String("Software failure".to_string()));
        risk1.insert("severity".to_string(), JsonValue::String("Major".to_string()));
        risk1.insert("occurrence".to_string(), JsonValue::String("Remote".to_string()));
        risk1.insert("risk_score".to_string(), JsonValue::Number(6.0));
        
        let mut risk2 = HashMap::new();
        risk2.insert("id".to_string(), JsonValue::String("RISK002".to_string()));
        risk2.insert("hazard".to_string(), JsonValue::String("Data corruption".to_string()));
        risk2.insert("severity".to_string(), JsonValue::String("Critical".to_string()));
        risk2.insert("occurrence".to_string(), JsonValue::String("Improbable".to_string()));
        risk2.insert("risk_score".to_string(), JsonValue::Number(4.0));
        
        let risks = vec![
            JsonValue::Object(risk1),
            JsonValue::Object(risk2),
        ];
        
        wrap_paginated_list(risks, page, per_page, 2)
    }

    /// Create risk operation result as JSON
    pub fn create_risk_json(hazard: &str, situation: &str, harm: &str) -> Result<String, String> {
        use crate::json_utils::JsonValue;
        use std::collections::HashMap;
        
        let mut risk = HashMap::new();
        risk.insert("id".to_string(), JsonValue::String("RISK003".to_string()));
        risk.insert("hazard".to_string(), JsonValue::String(hazard.to_string()));
        risk.insert("situation".to_string(), JsonValue::String(situation.to_string()));
        risk.insert("harm".to_string(), JsonValue::String(harm.to_string()));
        risk.insert("status".to_string(), JsonValue::String("Created".to_string()));
        risk.insert("created_at".to_string(), JsonValue::String("2024-01-20T10:30:00Z".to_string()));
        
        wrap_success(JsonValue::Object(risk))
    }

    /// Create risk assessment with detailed parameters (SOLID Single Responsibility)
    pub fn create_risk_assessment(
        risk_id: &str,
        description: &str,
        category: &str,
        severity: u8,
        occurrence: u8,
        detection: u8,
        mitigation: &str,
    ) -> Result<String, String> {
        use crate::json_utils::JsonValue;
        use std::collections::HashMap;

        // Calculate RPN
        let rpn = (severity as u16) * (occurrence as u16) * (detection as u16);
        let risk_level = match rpn {
            1..=50 => "low",
            51..=100 => "medium",
            101..=200 => "high",
            _ => "critical",
        };

        let mut risk = HashMap::new();
        risk.insert("id".to_string(), JsonValue::String(risk_id.to_string()));
        risk.insert("description".to_string(), JsonValue::String(description.to_string()));
        risk.insert("category".to_string(), JsonValue::String(category.to_string()));
        risk.insert("severity".to_string(), JsonValue::Number(severity as f64));
        risk.insert("occurrence".to_string(), JsonValue::Number(occurrence as f64));
        risk.insert("detection".to_string(), JsonValue::Number(detection as f64));
        risk.insert("rpn".to_string(), JsonValue::Number(rpn as f64));
        risk.insert("level".to_string(), JsonValue::String(risk_level.to_string()));
        risk.insert("mitigation".to_string(), JsonValue::String(mitigation.to_string()));
        risk.insert("status".to_string(), JsonValue::String("active".to_string()));
        risk.insert("created_at".to_string(), JsonValue::Number(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as f64
        ));

        wrap_success(JsonValue::Object(risk))
    }
}

/// Requirement Operations Bridge - Simplified API for requirement operations  
pub struct RequirementOperations;

impl RequirementOperations {
    /// List requirements as JSON API response with pagination
    pub fn list_requirements_json(page: Option<usize>, per_page: Option<usize>) -> Result<String, String> {
        let page = page.unwrap_or(1);
        let per_page = per_page.unwrap_or(20);
        
        // Mock requirement list for demonstration
        use crate::json_utils::JsonValue;
        use std::collections::HashMap;
        
        let mut req1 = HashMap::new();
        req1.insert("id".to_string(), JsonValue::String("REQ001".to_string()));
        req1.insert("title".to_string(), JsonValue::String("User Authentication".to_string()));
        req1.insert("category".to_string(), JsonValue::String("Security".to_string()));
        req1.insert("priority".to_string(), JsonValue::String("High".to_string()));
        req1.insert("status".to_string(), JsonValue::String("Verified".to_string()));
        
        let mut req2 = HashMap::new();
        req2.insert("id".to_string(), JsonValue::String("REQ002".to_string()));
        req2.insert("title".to_string(), JsonValue::String("Audit Logging".to_string()));
        req2.insert("category".to_string(), JsonValue::String("Regulatory".to_string()));
        req2.insert("priority".to_string(), JsonValue::String("Critical".to_string()));
        req2.insert("status".to_string(), JsonValue::String("Implemented".to_string()));
        
        let requirements = vec![
            JsonValue::Object(req1),
            JsonValue::Object(req2),
        ];
        
        wrap_paginated_list(requirements, page, per_page, 2)
    }

    /// Create requirement operation result as JSON
    pub fn create_requirement_json(title: &str, description: &str, category: &str, priority: &str) -> Result<String, String> {
        use crate::json_utils::JsonValue;
        use std::collections::HashMap;
        
        let mut req = HashMap::new();
        req.insert("id".to_string(), JsonValue::String("REQ003".to_string()));
        req.insert("title".to_string(), JsonValue::String(title.to_string()));
        req.insert("description".to_string(), JsonValue::String(description.to_string()));
        req.insert("category".to_string(), JsonValue::String(category.to_string()));
        req.insert("priority".to_string(), JsonValue::String(priority.to_string()));
        req.insert("status".to_string(), JsonValue::String("Created".to_string()));
        req.insert("created_at".to_string(), JsonValue::String("2024-01-20T10:30:00Z".to_string()));
        
        wrap_success(JsonValue::Object(req))
    }
}

/// Health Check API - System status and compliance information
pub struct HealthApiBridge;

impl HealthApiBridge {
    /// Get system health status as JSON API response
    pub fn get_health_status() -> Result<String, String> {
        use crate::json_utils::JsonValue;
        use std::collections::HashMap;

        let mut health_data = HashMap::new();
        health_data.insert("status".to_string(), JsonValue::String("healthy".to_string()));
        health_data.insert("version".to_string(), JsonValue::String("1.0.0".to_string()));
        health_data.insert("api_version".to_string(), JsonValue::String("1.0".to_string()));
        
        // Medical device compliance status
        let mut compliance = HashMap::new();
        compliance.insert("fda_21_cfr_820".to_string(), JsonValue::Bool(true));
        compliance.insert("iso_13485".to_string(), JsonValue::Bool(true));
        compliance.insert("iso_14971".to_string(), JsonValue::Bool(true));
        compliance.insert("cfr_part_11".to_string(), JsonValue::Bool(true));
        health_data.insert("compliance".to_string(), JsonValue::Object(compliance));
        
        // System status
        let mut system = HashMap::new();
        system.insert("audit_logging".to_string(), JsonValue::String("active".to_string()));
        system.insert("user_management".to_string(), JsonValue::String("active".to_string()));
        system.insert("document_control".to_string(), JsonValue::String("active".to_string()));
        system.insert("risk_management".to_string(), JsonValue::String("active".to_string()));
        system.insert("traceability".to_string(), JsonValue::String("active".to_string()));
        health_data.insert("systems".to_string(), JsonValue::Object(system));
        
        let response = ApiResponseBuilder::success()
            .data(JsonValue::Object(health_data))
            .build();

        Ok(response.to_json())
    }

    /// Get system statistics as JSON API response
    pub fn get_system_stats() -> Result<String, String> {
        use crate::json_utils::JsonValue;
        use std::collections::HashMap;

        let mut stats_data = HashMap::new();
        
        // Mock statistics - in real implementation, these would come from actual system data
        let mut documents = HashMap::new();
        documents.insert("total".to_string(), JsonValue::Number(42.0));
        documents.insert("approved".to_string(), JsonValue::Number(38.0));
        documents.insert("draft".to_string(), JsonValue::Number(4.0));
        stats_data.insert("documents".to_string(), JsonValue::Object(documents));
        
        let mut risks = HashMap::new();
        risks.insert("total".to_string(), JsonValue::Number(15.0));
        risks.insert("high".to_string(), JsonValue::Number(3.0));
        risks.insert("medium".to_string(), JsonValue::Number(7.0));
        risks.insert("low".to_string(), JsonValue::Number(5.0));
        stats_data.insert("risks".to_string(), JsonValue::Object(risks));
        
        let mut requirements = HashMap::new();
        requirements.insert("total".to_string(), JsonValue::Number(128.0));
        requirements.insert("verified".to_string(), JsonValue::Number(115.0));
        requirements.insert("pending".to_string(), JsonValue::Number(13.0));
        stats_data.insert("requirements".to_string(), JsonValue::Object(requirements));
        
        let response = ApiResponseBuilder::success()
            .data(JsonValue::Object(stats_data))
            .build();

        Ok(response.to_json())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::QmsError;

    #[test]
    fn test_health_api_bridge() {
        let health_result = HealthApiBridge::get_health_status();
        assert!(health_result.is_ok());
        
        let health_json = health_result.unwrap();
        assert!(health_json.contains("\"success\": true"));
        assert!(health_json.contains("\"status\": \"healthy\""));
        assert!(health_json.contains("\"fda_21_cfr_820\": true"));
        assert!(health_json.contains("\"api_version\": \"1.0\""));
    }

    #[test]
    fn test_system_stats_bridge() {
        let stats_result = HealthApiBridge::get_system_stats();
        assert!(stats_result.is_ok());
        
        let stats_json = stats_result.unwrap();
        assert!(stats_json.contains("\"success\": true"));
        assert!(stats_json.contains("\"documents\""));
        assert!(stats_json.contains("\"risks\""));
        assert!(stats_json.contains("\"requirements\""));
        assert!(stats_json.contains("\"total\": 42"));
    }

    #[test]
    fn test_cli_command_bridge_message() {
        let message_result = CliCommandBridge::execute_message_command(|| {
            Ok("Command executed successfully".to_string())
        });
        
        assert!(message_result.is_ok());
        let json_response = message_result.unwrap();
        assert!(json_response.contains("\"success\": true"));
        assert!(json_response.contains("\"Command executed successfully\""));
    }

    #[test]
    fn test_cli_command_bridge_error() {
        let error_result = CliCommandBridge::execute_message_command(|| {
            Err(QmsError::validation_error("Test validation error"))
        });
        
        assert!(error_result.is_ok());
        let json_response = error_result.unwrap();
        assert!(json_response.contains("\"success\": false"));
        assert!(json_response.contains("\"VALIDATION_ERROR\""));
        assert!(json_response.contains("\"Validation error: Test validation error\""));
    }

    #[test]
    fn test_document_operations() {
        // Test document list
        let list_result = DocumentOperations::list_documents_json(Some(1), Some(10));
        assert!(list_result.is_ok());
        let list_json = list_result.unwrap();
        assert!(list_json.contains("\"success\": true"));
        assert!(list_json.contains("\"DOC001\""));
        assert!(list_json.contains("\"System Requirements\""));

        // Test document creation
        let create_result = DocumentOperations::create_document_json("Test Document", "Procedure");
        assert!(create_result.is_ok());
        let create_json = create_result.unwrap();
        assert!(create_json.contains("\"success\": true"));
        assert!(create_json.contains("\"Test Document\""));
        assert!(create_json.contains("\"Procedure\""));

        // Test document deletion
        let delete_result = DocumentOperations::delete_document_json("DOC001");
        assert!(delete_result.is_ok());
        let delete_json = delete_result.unwrap();
        assert!(delete_json.contains("\"success\": true"));
        assert!(delete_json.contains("\"Document DOC001 deleted successfully\""));
    }

    #[test]
    fn test_risk_operations() {
        // Test risk list
        let list_result = RiskOperations::list_risks_json(Some(1), Some(10));
        assert!(list_result.is_ok());
        let list_json = list_result.unwrap();
        assert!(list_json.contains("\"success\": true"));
        assert!(list_json.contains("\"RISK001\""));
        assert!(list_json.contains("\"Software failure\""));

        // Test risk creation
        let create_result = RiskOperations::create_risk_json("Test Hazard", "Test Situation", "Test Harm");
        assert!(create_result.is_ok());
        let create_json = create_result.unwrap();
        assert!(create_json.contains("\"success\": true"));
        assert!(create_json.contains("\"Test Hazard\""));
    }

    #[test]
    fn test_requirement_operations() {
        // Test requirement list
        let list_result = RequirementOperations::list_requirements_json(Some(1), Some(10));
        assert!(list_result.is_ok());
        let list_json = list_result.unwrap();
        assert!(list_json.contains("\"success\": true"));
        assert!(list_json.contains("\"REQ001\""));
        assert!(list_json.contains("\"User Authentication\""));

        // Test requirement creation
        let create_result = RequirementOperations::create_requirement_json(
            "Test Requirement", 
            "Test Description", 
            "Functional", 
            "Medium"
        );
        assert!(create_result.is_ok());
        let create_json = create_result.unwrap();
        assert!(create_json.contains("\"success\": true"));
        assert!(create_json.contains("\"Test Requirement\""));
    }

    #[test]
    fn test_api_bridge_integration() {
        // Test that all bridge components work together
        let health = HealthApiBridge::get_health_status().unwrap();
        let stats = HealthApiBridge::get_system_stats().unwrap();
        let docs = DocumentOperations::list_documents_json(Some(1), Some(5)).unwrap();
        let risks = RiskOperations::list_risks_json(Some(1), Some(5)).unwrap();
        let reqs = RequirementOperations::list_requirements_json(Some(1), Some(5)).unwrap();
        
        // All should be valid JSON with success: true
        for response in [&health, &stats, &docs, &risks, &reqs] {
            assert!(response.contains("\"success\": true"));
            assert!(response.contains("\"compliance\"") || response.contains("\"data\""));
        }
    }
}
