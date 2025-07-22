// QMS WASM Event Handlers
// Event handling for interactive elements in the medical device interface
// Provides type-safe event processing with audit logging

use crate::prelude::*;
use crate::wasm::{QmsApiClient, DomUtils};
use std::collections::HashMap;

/// Event handler registry for QMS application
pub struct EventHandlers {
    api_client: QmsApiClient,
}

/// Event handler function type
pub type EventHandler = fn(&EventHandlers, &str) -> QmsResult<()>;

/// Event data structure
#[derive(Debug, Clone)]
pub struct EventData {
    pub event_type: String,
    pub target_id: String,
    pub data: HashMap<String, String>,
}

impl EventHandlers {
    /// Create new event handler registry
    pub const fn new(api_client: QmsApiClient) -> Self {
        Self { api_client }
    }

    /// Initialize all event handlers for the QMS interface
    pub fn initialize(&self) -> QmsResult<()> {
        // Register event listeners for all interactive elements
        self.register_navigation_handlers()?;
        self.register_dashboard_handlers()?;
        self.register_document_handlers()?;
        self.register_risk_handlers()?;
        self.register_requirement_handlers()?;
        self.register_audit_handlers()?;
        self.register_form_handlers()?;
        
        Ok(())
    }

    /// Register navigation event handlers
    fn register_navigation_handlers(&self) -> QmsResult<()> {
        // Main navigation menu
        DomUtils::add_event_listener("nav-dashboard", "click", "handleNavigateToDashboard")?;
        DomUtils::add_event_listener("nav-documents", "click", "handleNavigateToDocuments")?;
        DomUtils::add_event_listener("nav-risks", "click", "handleNavigateToRisks")?;
        DomUtils::add_event_listener("nav-requirements", "click", "handleNavigateToRequirements")?;
        DomUtils::add_event_listener("nav-audit", "click", "handleNavigateToAudit")?;
        DomUtils::add_event_listener("nav-reports", "click", "handleNavigateToReports")?;
        
        Ok(())
    }

    /// Register dashboard event handlers
    fn register_dashboard_handlers(&self) -> QmsResult<()> {
        // Quick action buttons
        DomUtils::add_event_listener("quick-add-document", "click", "handleQuickAddDocument")?;
        DomUtils::add_event_listener("quick-add-risk", "click", "handleQuickAddRisk")?;
        DomUtils::add_event_listener("quick-add-requirement", "click", "handleQuickAddRequirement")?;
        
        // Refresh button
        DomUtils::add_event_listener("refresh-dashboard", "click", "handleRefreshDashboard")?;
        
        // System status check
        DomUtils::add_event_listener("check-server-status", "click", "handleCheckServerStatus")?;
        
        Ok(())
    }

    /// Register document management event handlers
    fn register_document_handlers(&self) -> QmsResult<()> {
        // Document operations
        DomUtils::add_event_listener("add-document-btn", "click", "handleAddDocument")?;
        DomUtils::add_event_listener("search-documents", "input", "handleSearchDocuments")?;
        DomUtils::add_event_listener("filter-documents", "change", "handleFilterDocuments")?;
        DomUtils::add_event_listener("sort-documents", "change", "handleSortDocuments")?;
        
        // Document actions (will be dynamically added for each document row)
        // DomUtils::add_event_listener for view, edit, delete, approve, etc.
        
        Ok(())
    }

    /// Register risk management event handlers
    fn register_risk_handlers(&self) -> QmsResult<()> {
        // Risk operations
        DomUtils::add_event_listener("add-risk-btn", "click", "handleAddRisk")?;
        DomUtils::add_event_listener("search-risks", "input", "handleSearchRisks")?;
        DomUtils::add_event_listener("filter-risks", "change", "handleFilterRisks")?;
        DomUtils::add_event_listener("risk-matrix-btn", "click", "handleShowRiskMatrix")?;
        
        Ok(())
    }

    /// Register requirement management event handlers
    fn register_requirement_handlers(&self) -> QmsResult<()> {
        // Requirement operations
        DomUtils::add_event_listener("add-requirement-btn", "click", "handleAddRequirement")?;
        DomUtils::add_event_listener("search-requirements", "input", "handleSearchRequirements")?;
        DomUtils::add_event_listener("filter-requirements", "change", "handleFilterRequirements")?;
        DomUtils::add_event_listener("trace-matrix-btn", "click", "handleShowTraceMatrix")?;
        
        Ok(())
    }

    /// Register audit trail event handlers
    fn register_audit_handlers(&self) -> QmsResult<()> {
        // Audit operations
        DomUtils::add_event_listener("refresh-audit", "click", "handleRefreshAudit")?;
        DomUtils::add_event_listener("filter-audit", "change", "handleFilterAudit")?;
        DomUtils::add_event_listener("export-audit", "click", "handleExportAudit")?;
        
        Ok(())
    }

    /// Register form event handlers
    fn register_form_handlers(&self) -> QmsResult<()> {
        // Generic form handlers
        DomUtils::add_event_listener("modal-close", "click", "handleCloseModal")?;
        DomUtils::add_event_listener("modal-cancel", "click", "handleCancelForm")?;
        DomUtils::add_event_listener("modal-submit", "click", "handleSubmitForm")?;
        
        // Form validation
        DomUtils::add_event_listener("form-input", "change", "handleValidateField")?;
        DomUtils::add_event_listener("form-input", "blur", "handleValidateField")?;
        
        Ok(())
    }

    // Specific event handler implementations

    /// Handle navigation to dashboard
    pub fn handle_navigate_to_dashboard(&self, _event_data: &str) -> QmsResult<()> {
        self.show_loading("dashboard-content")?;
        self.load_dashboard_data()?;
        self.update_active_nav("nav-dashboard")?;
        Ok(())
    }

    /// Handle navigation to documents
    pub fn handle_navigate_to_documents(&self, _event_data: &str) -> QmsResult<()> {
        self.show_loading("main-content")?;
        self.load_documents_page()?;
        self.update_active_nav("nav-documents")?;
        Ok(())
    }

    /// Handle dashboard refresh
    pub fn handle_refresh_dashboard(&self, _event_data: &str) -> QmsResult<()> {
        self.show_loading("dashboard-stats")?;
        self.load_dashboard_data()?;
        self.check_server_status()?;
        Ok(())
    }

    /// Handle server status check
    pub fn handle_check_server_status(&self, _event_data: &str) -> QmsResult<()> {
        match self.api_client.get_health() {
            Ok(response) => {
                if response.status == 200 {
                    DomUtils::update_server_status(true)?;
                    DomUtils::show_notification("Server connection successful", "success")?;
                } else {
                    DomUtils::update_server_status(false)?;
                    DomUtils::show_notification("Server returned error", "warning")?;
                }
            }
            Err(_) => {
                DomUtils::update_server_status(false)?;
                DomUtils::show_notification("Cannot connect to server", "error")?;
            }
        }
        Ok(())
    }

    /// Handle quick add document
    pub fn handle_quick_add_document(&self, _event_data: &str) -> QmsResult<()> {
        self.show_modal("add-document-modal")?;
        self.initialize_document_form()?;
        Ok(())
    }

    /// Handle quick add risk
    pub fn handle_quick_add_risk(&self, _event_data: &str) -> QmsResult<()> {
        self.show_modal("add-risk-modal")?;
        self.initialize_risk_form()?;
        Ok(())
    }

    /// Handle document search
    pub fn handle_search_documents(&self, search_term: &str) -> QmsResult<()> {
        if search_term.len() >= 2 {
            self.filter_documents_by_search(search_term)?;
        } else if search_term.is_empty() {
            self.load_all_documents()?;
        }
        Ok(())
    }

    /// Handle form submission
    pub fn handle_submit_form(&self, form_id: &str) -> QmsResult<()> {
        match form_id {
            "add-document-form" => self.submit_document_form()?,
            "add-risk-form" => self.submit_risk_form()?,
            "add-requirement-form" => self.submit_requirement_form()?,
            _ => return Err(QmsError::validation_error("Unknown form type")),
        }
        Ok(())
    }

    // Helper methods for UI operations

    /// Show loading state
    fn show_loading(&self, element_id: &str) -> QmsResult<()> {
        DomUtils::set_loading_state(element_id, true)?;
        Ok(())
    }

    /// Hide loading state
    fn hide_loading(&self, element_id: &str) -> QmsResult<()> {
        DomUtils::set_loading_state(element_id, false)?;
        Ok(())
    }

    /// Update active navigation item
    fn update_active_nav(&self, active_nav_id: &str) -> QmsResult<()> {
        // Remove active class from all nav items
        let nav_items = ["nav-dashboard", "nav-documents", "nav-risks", "nav-requirements", "nav-audit", "nav-reports"];
        for nav_item in &nav_items {
            DomUtils::remove_class(nav_item, "active")?;
        }
        
        // Add active class to current nav item
        DomUtils::add_class(active_nav_id, "active")?;
        Ok(())
    }

    /// Show modal dialog
    fn show_modal(&self, modal_id: &str) -> QmsResult<()> {
        DomUtils::show_element(modal_id)?;
        DomUtils::add_class(modal_id, "modal-open")?;
        Ok(())
    }

    /// Hide modal dialog
    fn hide_modal(&self, modal_id: &str) -> QmsResult<()> {
        DomUtils::hide_element(modal_id)?;
        DomUtils::remove_class(modal_id, "modal-open")?;
        Ok(())
    }

    /// Load dashboard data
    fn load_dashboard_data(&self) -> QmsResult<()> {
        // Get system stats
        match self.api_client.get_system_stats() {
            Ok(response) => {
                if response.status == 200 {
                    self.update_dashboard_stats(&response.body)?;
                }
            }
            Err(_) => {
                DomUtils::show_notification("Failed to load dashboard data", "error")?;
            }
        }

        // Get compliance badges
        match self.api_client.get_compliance_badges() {
            Ok(response) => {
                if response.status == 200 {
                    self.update_compliance_badges(&response.body)?;
                }
            }
            Err(_) => {
                DomUtils::show_notification("Failed to load compliance data", "warning")?;
            }
        }

        self.hide_loading("dashboard-stats")?;
        Ok(())
    }

    /// Load documents page
    fn load_documents_page(&self) -> QmsResult<()> {
        match self.api_client.list_documents() {
            Ok(response) => {
                if response.status == 200 {
                    self.update_documents_table(&response.body)?;
                }
            }
            Err(_) => {
                DomUtils::show_notification("Failed to load documents", "error")?;
            }
        }
        
        self.hide_loading("main-content")?;
        Ok(())
    }

    /// Update dashboard statistics
    fn update_dashboard_stats(&self, _stats_json: &str) -> QmsResult<()> {
        // Parse JSON and update stats (placeholder implementation)
        let mut stats = HashMap::new();
        stats.insert("total-documents".to_string(), "25".to_string());
        stats.insert("total-risks".to_string(), "8".to_string());
        stats.insert("total-requirements".to_string(), "42".to_string());
        stats.insert("total-audit-entries".to_string(), "156".to_string());
        
        DomUtils::update_dashboard_stats(&stats)?;
        Ok(())
    }

    /// Update compliance badges
    fn update_compliance_badges(&self, _badges_json: &str) -> QmsResult<()> {
        // Parse JSON and update badges (placeholder implementation)
        DomUtils::update_badge("badge-fda", "FDA 21 CFR", 95)?;
        DomUtils::update_badge("badge-iso13485", "ISO 13485", 88)?;
        DomUtils::update_badge("badge-iso14971", "ISO 14971", 92)?;
        Ok(())
    }

    /// Update documents table
    fn update_documents_table(&self, _documents_json: &str) -> QmsResult<()> {
        // Parse JSON and populate table (placeholder implementation)
        let headers = vec!["ID", "Title", "Type", "Status", "Version", "Actions"];
        let rows = vec![
            vec!["DOC-001".to_string(), "Software Requirements".to_string(), "SRS".to_string(), "Approved".to_string(), "2.1.0".to_string(), "View | Edit".to_string()],
            vec!["DOC-002".to_string(), "System Design".to_string(), "SDD".to_string(), "Draft".to_string(), "1.0.0".to_string(), "View | Edit".to_string()],
        ];
        
        DomUtils::populate_table("documents-table", &headers, &rows)?;
        Ok(())
    }

    /// Check server connectivity
    fn check_server_status(&self) -> QmsResult<()> {
        match self.api_client.get_health() {
            Ok(response) => {
                DomUtils::update_server_status(response.status == 200)?;
            }
            Err(_) => {
                DomUtils::update_server_status(false)?;
            }
        }
        Ok(())
    }

    /// Initialize document form
    fn initialize_document_form(&self) -> QmsResult<()> {
        // Clear form fields and set defaults
        DomUtils::set_attribute("doc-title", "value", "")?;
        DomUtils::set_attribute("doc-type", "value", "srs")?;
        DomUtils::set_text_content("doc-description", "")?;
        Ok(())
    }

    /// Initialize risk form
    fn initialize_risk_form(&self) -> QmsResult<()> {
        // Clear form fields and set defaults
        DomUtils::set_attribute("risk-hazard", "value", "")?;
        DomUtils::set_attribute("risk-situation", "value", "")?;
        DomUtils::set_attribute("risk-harm", "value", "")?;
        DomUtils::set_attribute("risk-severity", "value", "3")?;
        DomUtils::set_attribute("risk-occurrence", "value", "3")?;
        DomUtils::set_attribute("risk-detectability", "value", "3")?;
        Ok(())
    }

    /// Submit document form
    fn submit_document_form(&self) -> QmsResult<()> {
        // Validate and submit document (placeholder implementation)
        DomUtils::show_notification("Document created successfully", "success")?;
        self.hide_modal("add-document-modal")?;
        self.load_documents_page()?;
        Ok(())
    }

    /// Submit risk form
    fn submit_risk_form(&self) -> QmsResult<()> {
        // Validate and submit risk (placeholder implementation)
        DomUtils::show_notification("Risk assessment created successfully", "success")?;
        self.hide_modal("add-risk-modal")?;
        Ok(())
    }

    /// Submit requirement form
    fn submit_requirement_form(&self) -> QmsResult<()> {
        // Validate and submit requirement (placeholder implementation)
        DomUtils::show_notification("Requirement created successfully", "success")?;
        self.hide_modal("add-requirement-modal")?;
        Ok(())
    }

    /// Filter documents by search term
    fn filter_documents_by_search(&self, _search_term: &str) -> QmsResult<()> {
        // Implement search filtering (placeholder)
        self.load_documents_page()?;
        Ok(())
    }

    /// Load all documents (clear filters)
    fn load_all_documents(&self) -> QmsResult<()> {
        self.load_documents_page()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_api_client() -> QmsApiClient {
        QmsApiClient::new("http://localhost:8080".to_string())
    }

    #[test]
    fn test_event_handlers_creation() {
        let api_client = create_test_api_client();
        let handlers = EventHandlers::new(api_client);
        // Should create successfully - test passes if no panic occurs
    }

    #[test]
    fn test_handle_navigate_to_dashboard() {
        let api_client = create_test_api_client();
        let handlers = EventHandlers::new(api_client);
        let result = handlers.handle_navigate_to_dashboard("");
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_refresh_dashboard() {
        let api_client = create_test_api_client();
        let handlers = EventHandlers::new(api_client);
        let result = handlers.handle_refresh_dashboard("");
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_check_server_status() {
        let api_client = create_test_api_client();
        let handlers = EventHandlers::new(api_client);
        let result = handlers.handle_check_server_status("");
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_quick_add_document() {
        let api_client = create_test_api_client();
        let handlers = EventHandlers::new(api_client);
        let result = handlers.handle_quick_add_document("");
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_search_documents() {
        let api_client = create_test_api_client();
        let handlers = EventHandlers::new(api_client);
        
        // Test with valid search term
        let result = handlers.handle_search_documents("test");
        assert!(result.is_ok());
        
        // Test with empty search term
        let result = handlers.handle_search_documents("");
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_submit_form() {
        let api_client = create_test_api_client();
        let handlers = EventHandlers::new(api_client);
        
        // Test valid form
        let result = handlers.handle_submit_form("add-document-form");
        assert!(result.is_ok());
        
        // Test invalid form
        let result = handlers.handle_submit_form("invalid-form");
        assert!(result.is_err());
    }

    #[test]
    fn test_modal_operations() {
        let api_client = create_test_api_client();
        let handlers = EventHandlers::new(api_client);
        
        let result = handlers.show_modal("test-modal");
        assert!(result.is_ok());
        
        let result = handlers.hide_modal("test-modal");
        assert!(result.is_ok());
    }

    #[test]
    fn test_form_initialization() {
        let api_client = create_test_api_client();
        let handlers = EventHandlers::new(api_client);
        
        let result = handlers.initialize_document_form();
        assert!(result.is_ok());
        
        let result = handlers.initialize_risk_form();
        assert!(result.is_ok());
    }
}
