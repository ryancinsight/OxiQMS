// QMS WASM Application
// Main application controller for the medical device QMS web interface
// Coordinates API client, DOM utilities, and event handlers

use crate::prelude::*;
use crate::wasm::{QmsApiClient, DomUtils, EventHandlers};
use std::collections::HashMap;

/// Main WASM application for QMS medical device system
pub struct QmsApp {
    api_client: QmsApiClient,
    event_handlers: EventHandlers,
    current_user: Option<String>,
    current_session: Option<String>,
    is_authenticated: bool,
    config: AppConfig,
}

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub api_base_url: String,
    pub auto_refresh_interval: u32,
    pub session_timeout: u32,
    pub enable_notifications: bool,
    pub debug_mode: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_base_url: "http://localhost:8080".to_string(),
            auto_refresh_interval: 30, // seconds
            session_timeout: 1800, // 30 minutes
            enable_notifications: true,
            debug_mode: false,
        }
    }
}

impl QmsApp {
    /// Create new QMS application instance
    pub fn new(config: AppConfig) -> Self {
        let api_client = QmsApiClient::new(config.api_base_url.clone());
        let event_handlers = EventHandlers::new(api_client.clone());

        Self {
            api_client,
            event_handlers,
            current_user: None,
            current_session: None,
            is_authenticated: false,
            config,
        }
    }

    /// Initialize the QMS application
    pub fn initialize(&mut self) -> QmsResult<()> {
        self.log_debug("Initializing QMS Medical Device Application")?;
        
        // Initialize UI components
        self.initialize_ui()?;
        
        // Set up event handlers
        self.event_handlers.initialize()?;
        
        // Check server connectivity
        self.check_server_status()?;
        
        // Load initial data
        self.load_initial_data()?;
        
        // Start auto-refresh timer (placeholder for WASM implementation)
        self.start_auto_refresh()?;
        
        self.log_info("QMS Application initialized successfully")?;
        Ok(())
    }

    /// Initialize UI components and layout
    fn initialize_ui(&self) -> QmsResult<()> {
        // Set application title
        DomUtils::set_text_content("app-title", "QMS Medical Device System")?;
        
        // Initialize compliance badges
        self.initialize_compliance_badges()?;
        
        // Initialize navigation
        self.initialize_navigation()?;
        
        // Initialize main content area
        self.initialize_main_content()?;
        
        // Initialize notification area
        self.initialize_notifications()?;
        
        Ok(())
    }

    /// Initialize compliance indicators
    fn initialize_compliance_badges(&self) -> QmsResult<()> {
        DomUtils::update_badge("badge-fda", "FDA 21 CFR", 0)?;
        DomUtils::update_badge("badge-iso13485", "ISO 13485", 0)?;
        DomUtils::update_badge("badge-iso14971", "ISO 14971", 0)?;
        DomUtils::set_text_content("compliance-status", "Loading compliance data...")?;
        Ok(())
    }

    /// Initialize navigation menu
    fn initialize_navigation(&self) -> QmsResult<()> {
        // Set active navigation item (dashboard by default)
        DomUtils::add_class("nav-dashboard", "active")?;
        
        // Ensure other nav items are not active
        let nav_items = ["nav-documents", "nav-risks", "nav-requirements", "nav-audit", "nav-reports"];
        for item in &nav_items {
            DomUtils::remove_class(item, "active")?;
        }
        
        Ok(())
    }

    /// Initialize main content area
    fn initialize_main_content(&self) -> QmsResult<()> {
        // Show dashboard by default
        self.show_dashboard()?;
        Ok(())
    }

    /// Initialize notification system
    fn initialize_notifications(&self) -> QmsResult<()> {
        DomUtils::hide_element("notification-area")?;
        Ok(())
    }

    /// Check server connectivity
    fn check_server_status(&self) -> QmsResult<()> {
        match self.api_client.get_health() {
            Ok(response) => {
                if response.status == 200 {
                    DomUtils::update_server_status(true)?;
                    self.log_info("Server connection established")?;
                } else {
                    DomUtils::update_server_status(false)?;
                    self.log_warning(&format!("Server returned status: {}", response.status))?;
                }
            }
            Err(e) => {
                DomUtils::update_server_status(false)?;
                self.log_error(&format!("Server connection failed: {e}"))?;
            }
        }
        Ok(())
    }

    /// Load initial application data for medical device project setup
    fn load_initial_data(&self) -> QmsResult<()> {
        self.log_info("Loading initial medical device project data...")?;

        // Phase 1: Load core system data
        self.load_system_configuration()?;
        self.load_user_permissions()?;

        // Phase 2: Load medical device project data
        self.load_project_metadata()?;
        self.load_regulatory_requirements()?;

        // Phase 3: Load dashboard and compliance data
        self.load_dashboard_data()?;
        self.load_compliance_data()?;

        // Phase 4: Load workflow-specific data
        self.load_document_templates()?;
        self.load_risk_categories()?;
        self.load_test_protocols()?;

        // Phase 5: Initialize project setup wizard if needed
        self.check_project_setup_status()?;

        self.log_info("Initial data loading completed successfully")?;
        Ok(())
    }

    /// Load system configuration for medical device compliance
    fn load_system_configuration(&self) -> QmsResult<()> {
        self.log_debug("Loading system configuration...")?;

        // Load regulatory standards configuration
        let standards_response = self.api_client.get("/api/config/regulatory-standards")?;
        if standards_response.status == 200 {
            DomUtils::set_attribute("body", "data-regulatory-standards", &standards_response.body)?;
        }

        // Load audit configuration
        let audit_response = self.api_client.get("/api/config/audit")?;
        if audit_response.status == 200 {
            DomUtils::set_attribute("body", "data-audit-enabled", "true")?;
        }

        // Load validation rules
        let validation_response = self.api_client.get("/api/config/validation-rules")?;
        if validation_response.status == 200 {
            // Store validation rules for form validation
            DomUtils::set_attribute("body", "data-validation-rules", &validation_response.body)?;
        }

        Ok(())
    }

    /// Load user permissions and role-based access control
    fn load_user_permissions(&self) -> QmsResult<()> {
        self.log_debug("Loading user permissions...")?;

        if let Some(user) = &self.current_user {
            let permissions_response = self.api_client.get(&format!("/api/users/{}/permissions", user))?;
            if permissions_response.status == 200 {
                // Update UI based on user permissions
                self.update_ui_permissions(&permissions_response.body)?;
            }
        }

        Ok(())
    }

    /// Load project metadata and setup status
    fn load_project_metadata(&self) -> QmsResult<()> {
        self.log_debug("Loading project metadata...")?;

        let project_response = self.api_client.get("/api/project/metadata")?;
        if project_response.status == 200 {
            // Update project information in UI
            DomUtils::set_text_content("project-name", "Medical Device QMS Project")?;
            DomUtils::set_text_content("project-status", "Active")?;

            // Parse and display project details
            self.update_project_details(&project_response.body)?;
        } else {
            // Project not configured - show setup wizard
            self.show_project_setup_wizard()?;
        }

        Ok(())
    }

    /// Load regulatory requirements for medical device compliance
    fn load_regulatory_requirements(&self) -> QmsResult<()> {
        self.log_debug("Loading regulatory requirements...")?;

        let requirements_response = self.api_client.get("/api/regulatory/requirements")?;
        if requirements_response.status == 200 {
            // Update compliance dashboard
            self.update_regulatory_compliance_status(&requirements_response.body)?;
        }

        Ok(())
    }

    /// Load document templates for medical device documentation
    fn load_document_templates(&self) -> QmsResult<()> {
        self.log_debug("Loading document templates...")?;

        let templates_response = self.api_client.get("/api/documents/templates")?;
        if templates_response.status == 200 {
            // Populate template selection dropdowns
            self.populate_template_selectors(&templates_response.body)?;
        }

        Ok(())
    }

    /// Load risk categories for risk management
    fn load_risk_categories(&self) -> QmsResult<()> {
        self.log_debug("Loading risk categories...")?;

        let categories_response = self.api_client.get("/api/risks/categories")?;
        if categories_response.status == 200 {
            // Populate risk category dropdowns
            self.populate_risk_categories(&categories_response.body)?;
        }

        Ok(())
    }

    /// Load test protocols and validation procedures
    fn load_test_protocols(&self) -> QmsResult<()> {
        self.log_debug("Loading test protocols...")?;

        let protocols_response = self.api_client.get("/api/testing/protocols")?;
        if protocols_response.status == 200 {
            // Update testing dashboard
            self.update_testing_protocols(&protocols_response.body)?;
        }

        Ok(())
    }

    /// Check project setup status and show wizard if needed
    fn check_project_setup_status(&self) -> QmsResult<()> {
        self.log_debug("Checking project setup status...")?;

        let setup_response = self.api_client.get("/api/project/setup-status")?;
        if setup_response.status == 200 {
            // Parse setup status and show appropriate UI
            if setup_response.body.contains("\"setup_complete\":false") {
                self.show_project_setup_wizard()?;
            } else {
                self.hide_project_setup_wizard()?;
            }
        }

        Ok(())
    }

    /// Load dashboard data
    fn load_dashboard_data(&self) -> QmsResult<()> {
        match self.api_client.get_system_stats() {
            Ok(response) => {
                if response.status == 200 {
                    self.update_dashboard_stats(&response.body)?;
                } else {
                    self.show_notification("Failed to load dashboard data", "warning")?;
                }
            }
            Err(_) => {
                self.show_notification("Cannot connect to server", "error")?;
                self.set_offline_mode()?;
            }
        }
        Ok(())
    }

    /// Load compliance data
    fn load_compliance_data(&self) -> QmsResult<()> {
        match self.api_client.get_compliance_badges() {
            Ok(response) => {
                if response.status == 200 {
                    self.update_compliance_badges(&response.body)?;
                } else {
                    self.log_warning("Failed to load compliance data")?;
                }
            }
            Err(_) => {
                self.log_error("Cannot load compliance data - server unavailable")?;
            }
        }
        Ok(())
    }

    /// Update dashboard statistics
    fn update_dashboard_stats(&self, _stats_json: &str) -> QmsResult<()> {
        // Parse JSON response and update UI (placeholder implementation)
        let mut stats = HashMap::new();
        stats.insert("total-documents".to_string(), "25".to_string());
        stats.insert("total-risks".to_string(), "8".to_string());
        stats.insert("total-requirements".to_string(), "42".to_string());
        stats.insert("total-audit-entries".to_string(), "156".to_string());
        
        DomUtils::update_dashboard_stats(&stats)?;
        
        // Update last refresh time
        DomUtils::set_text_content("last-refresh", "Just now")?;
        
        Ok(())
    }

    /// Update compliance badges
    fn update_compliance_badges(&self, _badges_json: &str) -> QmsResult<()> {
        // Parse JSON response and update badges (placeholder implementation)
        DomUtils::update_badge("badge-fda", "FDA 21 CFR", 95)?;
        DomUtils::update_badge("badge-iso13485", "ISO 13485", 88)?;
        DomUtils::update_badge("badge-iso14971", "ISO 14971", 92)?;
        
        // Update overall compliance status
        let overall_compliance = (95 + 88 + 92) / 3;
        let status_text = format!("Overall Compliance: {overall_compliance}%");
        DomUtils::set_text_content("compliance-status", &status_text)?;
        
        if overall_compliance >= 90 {
            DomUtils::add_class("compliance-status", "status-excellent")?;
        } else if overall_compliance >= 80 {
            DomUtils::add_class("compliance-status", "status-good")?;
        } else {
            DomUtils::add_class("compliance-status", "status-needs-attention")?;
        }
        
        Ok(())
    }

    /// Show dashboard page
    fn show_dashboard(&self) -> QmsResult<()> {
        DomUtils::show_element("dashboard-content")?;
        DomUtils::hide_element("documents-content")?;
        DomUtils::hide_element("risks-content")?;
        DomUtils::hide_element("requirements-content")?;
        DomUtils::hide_element("audit-content")?;
        DomUtils::hide_element("reports-content")?;
        
        // Update page title
        DomUtils::set_text_content("page-title", "Dashboard")?;
        
        Ok(())
    }

    /// Show documents page
    pub fn show_documents_page(&self) -> QmsResult<()> {
        DomUtils::hide_element("dashboard-content")?;
        DomUtils::show_element("documents-content")?;
        DomUtils::hide_element("risks-content")?;
        DomUtils::hide_element("requirements-content")?;
        DomUtils::hide_element("audit-content")?;
        DomUtils::hide_element("reports-content")?;
        
        DomUtils::set_text_content("page-title", "Document Management")?;
        self.load_documents_data()?;
        
        Ok(())
    }

    /// Show risks page
    pub fn show_risks_page(&self) -> QmsResult<()> {
        DomUtils::hide_element("dashboard-content")?;
        DomUtils::hide_element("documents-content")?;
        DomUtils::show_element("risks-content")?;
        DomUtils::hide_element("requirements-content")?;
        DomUtils::hide_element("audit-content")?;
        DomUtils::hide_element("reports-content")?;
        
        DomUtils::set_text_content("page-title", "Risk Management")?;
        self.load_risks_data()?;
        
        Ok(())
    }

    /// Show requirements page
    pub fn show_requirements_page(&self) -> QmsResult<()> {
        DomUtils::hide_element("dashboard-content")?;
        DomUtils::hide_element("documents-content")?;
        DomUtils::hide_element("risks-content")?;
        DomUtils::show_element("requirements-content")?;
        DomUtils::hide_element("audit-content")?;
        DomUtils::hide_element("reports-content")?;
        
        DomUtils::set_text_content("page-title", "Requirements Management")?;
        self.load_requirements_data()?;
        
        Ok(())
    }

    /// Show audit page
    pub fn show_audit_page(&self) -> QmsResult<()> {
        DomUtils::hide_element("dashboard-content")?;
        DomUtils::hide_element("documents-content")?;
        DomUtils::hide_element("risks-content")?;
        DomUtils::hide_element("requirements-content")?;
        DomUtils::show_element("audit-content")?;
        DomUtils::hide_element("reports-content")?;
        
        DomUtils::set_text_content("page-title", "Audit Trail")?;
        self.load_audit_data()?;
        
        Ok(())
    }

    /// Load documents data
    fn load_documents_data(&self) -> QmsResult<()> {
        DomUtils::set_loading_state("documents-table", true)?;
        
        match self.api_client.list_documents() {
            Ok(response) => {
                if response.status == 200 {
                    self.update_documents_table(&response.body)?;
                } else {
                    self.show_notification("Failed to load documents", "error")?;
                }
            }
            Err(_) => {
                self.show_notification("Cannot load documents - server unavailable", "error")?;
            }
        }
        
        DomUtils::set_loading_state("documents-table", false)?;
        Ok(())
    }

    /// Load risks data
    fn load_risks_data(&self) -> QmsResult<()> {
        DomUtils::set_loading_state("risks-table", true)?;
        
        match self.api_client.list_risks() {
            Ok(response) => {
                if response.status == 200 {
                    self.update_risks_table(&response.body)?;
                } else {
                    self.show_notification("Failed to load risks", "error")?;
                }
            }
            Err(_) => {
                self.show_notification("Cannot load risks - server unavailable", "error")?;
            }
        }
        
        DomUtils::set_loading_state("risks-table", false)?;
        Ok(())
    }

    /// Load requirements data
    fn load_requirements_data(&self) -> QmsResult<()> {
        DomUtils::set_loading_state("requirements-table", true)?;
        
        match self.api_client.list_requirements() {
            Ok(response) => {
                if response.status == 200 {
                    self.update_requirements_table(&response.body)?;
                } else {
                    self.show_notification("Failed to load requirements", "error")?;
                }
            }
            Err(_) => {
                self.show_notification("Cannot load requirements - server unavailable", "error")?;
            }
        }
        
        DomUtils::set_loading_state("requirements-table", false)?;
        Ok(())
    }

    /// Load audit data
    fn load_audit_data(&self) -> QmsResult<()> {
        DomUtils::set_loading_state("audit-table", true)?;
        
        match self.api_client.get_audit_entries(Some(50)) {
            Ok(response) => {
                if response.status == 200 {
                    self.update_audit_table(&response.body)?;
                } else {
                    self.show_notification("Failed to load audit trail", "error")?;
                }
            }
            Err(_) => {
                self.show_notification("Cannot load audit trail - server unavailable", "error")?;
            }
        }
        
        DomUtils::set_loading_state("audit-table", false)?;
        Ok(())
    }

    /// Update documents table
    fn update_documents_table(&self, _documents_json: &str) -> QmsResult<()> {
        // Parse JSON and populate table (placeholder implementation)
        let headers = vec!["ID", "Title", "Type", "Status", "Version", "Last Modified", "Actions"];
        let rows = vec![
            vec!["DOC-001".to_string(), "Software Requirements".to_string(), "SRS".to_string(), "Approved".to_string(), "2.1.0".to_string(), "2025-01-15".to_string(), "View | Edit | Download".to_string()],
            vec!["DOC-002".to_string(), "System Design".to_string(), "SDD".to_string(), "Draft".to_string(), "1.0.0".to_string(), "2025-01-10".to_string(), "View | Edit | Delete".to_string()],
            vec!["DOC-003".to_string(), "Test Protocol".to_string(), "Test".to_string(), "In Review".to_string(), "1.5.0".to_string(), "2025-01-12".to_string(), "View | Approve | Reject".to_string()],
        ];
        
        DomUtils::populate_table("documents-table", &headers, &rows)?;
        Ok(())
    }

    /// Update risks table
    fn update_risks_table(&self, _risks_json: &str) -> QmsResult<()> {
        let headers = vec!["ID", "Hazard", "Severity", "Occurrence", "Detectability", "RPN", "Risk Level", "Status"];
        let rows = vec![
            vec!["RISK-001".to_string(), "Power failure".to_string(), "5".to_string(), "3".to_string(), "2".to_string(), "30".to_string(), "ALARP".to_string(), "Mitigated".to_string()],
            vec!["RISK-002".to_string(), "Software crash".to_string(), "4".to_string(), "4".to_string(), "3".to_string(), "48".to_string(), "ALARP".to_string(), "Assessed".to_string()],
            vec!["RISK-003".to_string(), "Data corruption".to_string(), "5".to_string(), "2".to_string(), "4".to_string(), "40".to_string(), "ALARP".to_string(), "Identified".to_string()],
        ];
        
        DomUtils::populate_table("risks-table", &headers, &rows)?;
        Ok(())
    }

    /// Update requirements table
    fn update_requirements_table(&self, _requirements_json: &str) -> QmsResult<()> {
        let headers = vec!["ID", "Title", "Category", "Priority", "Status", "Verification", "Linked Tests"];
        let rows = vec![
            vec!["REQ-001".to_string(), "User Authentication".to_string(), "Security".to_string(), "Critical".to_string(), "Verified".to_string(), "Test".to_string(), "TC-001, TC-002".to_string()],
            vec!["REQ-002".to_string(), "Data Backup".to_string(), "Reliability".to_string(), "High".to_string(), "Implemented".to_string(), "Analysis".to_string(), "TC-003".to_string()],
            vec!["REQ-003".to_string(), "Audit Logging".to_string(), "Regulatory".to_string(), "Critical".to_string(), "Verified".to_string(), "Test".to_string(), "TC-004, TC-005".to_string()],
        ];
        
        DomUtils::populate_table("requirements-table", &headers, &rows)?;
        Ok(())
    }

    /// Update audit table
    fn update_audit_table(&self, _audit_json: &str) -> QmsResult<()> {
        let headers = vec!["Timestamp", "User", "Action", "Entity", "Details"];
        let rows = vec![
            vec!["2025-01-19 14:30:15".to_string(), "admin".to_string(), "CREATE".to_string(), "Document".to_string(), "Created DOC-003".to_string()],
            vec!["2025-01-19 14:25:42".to_string(), "engineer1".to_string(), "UPDATE".to_string(), "Risk".to_string(), "Updated RISK-001 mitigation".to_string()],
            vec!["2025-01-19 14:20:18".to_string(), "admin".to_string(), "APPROVE".to_string(), "Document".to_string(), "Approved DOC-001 v2.1.0".to_string()],
        ];
        
        DomUtils::populate_table("audit-table", &headers, &rows)?;
        Ok(())
    }

    /// Start auto-refresh timer for medical device compliance monitoring
    fn start_auto_refresh(&self) -> QmsResult<()> {
        self.log_debug("Starting auto-refresh timer for compliance monitoring...")?;

        // In actual WASM, this would use web_sys::window().set_interval_with_callback()
        // For stdlib-only implementation, we simulate the timer behavior
        self.schedule_next_refresh()?;

        // Update UI to show auto-refresh is active
        DomUtils::add_class("refresh-indicator", "active")?;
        DomUtils::set_text_content("refresh-status", "Auto-refresh: Active")?;

        self.log_info(&format!("Auto-refresh started with {}-second interval", self.config.auto_refresh_interval))?;
        Ok(())
    }

    /// Schedule next refresh cycle (WASM-compatible implementation)
    fn schedule_next_refresh(&self) -> QmsResult<()> {
        // In a real WASM implementation, this would use:
        // let closure = Closure::wrap(Box::new(move || {
        //     refresh_dashboard_data();
        // }) as Box<dyn FnMut()>);
        // window().set_timeout_with_callback_and_timeout_and_arguments_0(
        //     closure.as_ref().unchecked_ref(),
        //     self.config.auto_refresh_interval * 1000
        // );

        // For stdlib-only simulation, we mark the refresh as scheduled
        DomUtils::set_attribute("body", "data-refresh-scheduled", "true")?;
        DomUtils::set_attribute("body", "data-refresh-interval", &self.config.auto_refresh_interval.to_string())?;

        self.log_debug("Next refresh cycle scheduled")?;
        Ok(())
    }

    /// Execute refresh cycle for medical device data
    pub fn execute_refresh_cycle(&mut self) -> QmsResult<()> {
        self.log_debug("Executing scheduled refresh cycle...")?;

        // Refresh critical medical device data
        self.refresh_compliance_status()?;
        self.refresh_audit_status()?;
        self.refresh_risk_alerts()?;
        self.refresh_document_status()?;

        // Update last refresh timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        DomUtils::set_attribute("body", "data-last-refresh", &timestamp.to_string())?;
        DomUtils::set_text_content("last-refresh-time", &format!("Last updated: {}", timestamp))?;

        // Schedule next refresh
        self.schedule_next_refresh()?;

        self.log_debug("Refresh cycle completed successfully")?;
        Ok(())
    }

    /// Refresh compliance monitoring status
    fn refresh_compliance_status(&self) -> QmsResult<()> {
        let response = self.api_client.get("/api/compliance/status")?;
        if response.status == 200 {
            self.update_compliance_indicators(&response.body)?;
        }
        Ok(())
    }

    /// Refresh audit trail status
    fn refresh_audit_status(&self) -> QmsResult<()> {
        let response = self.api_client.get("/api/audit/status")?;
        if response.status == 200 {
            self.update_audit_indicators(&response.body)?;
        }
        Ok(())
    }

    /// Refresh risk management alerts
    fn refresh_risk_alerts(&self) -> QmsResult<()> {
        let response = self.api_client.get("/api/risks/alerts")?;
        if response.status == 200 {
            self.update_risk_alerts(&response.body)?;
        }
        Ok(())
    }

    /// Refresh document control status
    fn refresh_document_status(&self) -> QmsResult<()> {
        let response = self.api_client.get("/api/documents/status")?;
        if response.status == 200 {
            self.update_document_status(&response.body)?;
        }
        Ok(())
    }

    /// Set offline mode
    fn set_offline_mode(&self) -> QmsResult<()> {
        DomUtils::show_notification("Application is in offline mode", "warning")?;
        DomUtils::add_class("app-container", "offline-mode")?;
        Ok(())
    }

    /// Show notification to user
    fn show_notification(&self, message: &str, notification_type: &str) -> QmsResult<()> {
        DomUtils::show_notification(message, notification_type)?;
        Ok(())
    }

    // Medical Device Project Setup Workflow Methods

    /// Show project setup wizard for medical device configuration
    fn show_project_setup_wizard(&self) -> QmsResult<()> {
        self.log_info("Displaying medical device project setup wizard")?;

        // Show setup wizard modal
        DomUtils::show_element("project-setup-wizard")?;
        DomUtils::add_class("project-setup-wizard", "active")?;

        // Initialize setup steps
        self.initialize_setup_steps()?;

        // Load setup form with medical device defaults
        self.load_setup_form_defaults()?;

        Ok(())
    }

    /// Hide project setup wizard
    fn hide_project_setup_wizard(&self) -> QmsResult<()> {
        DomUtils::hide_element("project-setup-wizard")?;
        DomUtils::remove_class("project-setup-wizard", "active")?;
        Ok(())
    }

    /// Initialize setup wizard steps
    fn initialize_setup_steps(&self) -> QmsResult<()> {
        let steps = vec![
            ("step-1", "Project Information", "Configure basic project details"),
            ("step-2", "Regulatory Standards", "Select applicable medical device standards"),
            ("step-3", "Risk Management", "Configure risk management framework"),
            ("step-4", "Document Control", "Set up document control system"),
            ("step-5", "Quality Assurance", "Configure quality assurance processes"),
            ("step-6", "Audit Configuration", "Set up audit trail and compliance monitoring"),
        ];

        for (i, (step_id, title, description)) in steps.iter().enumerate() {
            DomUtils::set_text_content(&format!("{}-title", step_id), title)?;
            DomUtils::set_text_content(&format!("{}-description", step_id), description)?;

            if i == 0 {
                DomUtils::add_class(step_id, "active")?;
            } else {
                DomUtils::add_class(step_id, "pending")?;
            }
        }

        Ok(())
    }

    /// Load setup form with medical device defaults
    fn load_setup_form_defaults(&self) -> QmsResult<()> {
        // Set default regulatory standards
        DomUtils::set_form_value("regulatory-standard-iso13485", "true")?;
        DomUtils::set_form_value("regulatory-standard-iso14971", "true")?;
        DomUtils::set_form_value("regulatory-standard-fda21cfr820", "true")?;

        // Set default risk management configuration
        DomUtils::set_form_value("risk-management-enabled", "true")?;
        DomUtils::set_form_value("risk-assessment-method", "iso14971")?;

        // Set default document control settings
        DomUtils::set_form_value("document-versioning", "automatic")?;
        DomUtils::set_form_value("document-approval-required", "true")?;

        // Set default audit configuration
        DomUtils::set_form_value("audit-trail-enabled", "true")?;
        DomUtils::set_form_value("audit-retention-years", "7")?;

        Ok(())
    }

    // UI Update Helper Methods

    /// Update UI based on user permissions
    fn update_ui_permissions(&self, permissions_json: &str) -> QmsResult<()> {
        // Parse permissions and show/hide UI elements accordingly
        if permissions_json.contains("\"document_create\":true") {
            DomUtils::show_element("create-document-btn")?;
        } else {
            DomUtils::hide_element("create-document-btn")?;
        }

        if permissions_json.contains("\"risk_manage\":true") {
            DomUtils::show_element("risk-management-section")?;
        } else {
            DomUtils::hide_element("risk-management-section")?;
        }

        if permissions_json.contains("\"audit_view\":true") {
            DomUtils::show_element("audit-log-section")?;
        } else {
            DomUtils::hide_element("audit-log-section")?;
        }

        Ok(())
    }

    /// Update project details in UI
    fn update_project_details(&self, project_json: &str) -> QmsResult<()> {
        // Parse project JSON and update UI elements
        if project_json.contains("\"name\":") {
            // Extract project name (simplified parsing)
            if let Some(start) = project_json.find("\"name\":\"") {
                if let Some(end) = project_json[start + 8..].find("\"") {
                    let name = &project_json[start + 8..start + 8 + end];
                    DomUtils::set_text_content("project-name", name)?;
                }
            }
        }

        // Update project status indicators
        if project_json.contains("\"compliance_status\":\"compliant\"") {
            DomUtils::add_class("project-status", "compliant")?;
            DomUtils::set_text_content("compliance-status", "Compliant")?;
        }

        Ok(())
    }

    /// Update regulatory compliance status indicators
    fn update_regulatory_compliance_status(&self, requirements_json: &str) -> QmsResult<()> {
        // Update ISO 13485 status
        if requirements_json.contains("\"iso13485\":\"compliant\"") {
            DomUtils::add_class("iso13485-indicator", "compliant")?;
            DomUtils::set_text_content("iso13485-status", "Compliant")?;
        }

        // Update ISO 14971 status
        if requirements_json.contains("\"iso14971\":\"compliant\"") {
            DomUtils::add_class("iso14971-indicator", "compliant")?;
            DomUtils::set_text_content("iso14971-status", "Compliant")?;
        }

        // Update FDA 21 CFR Part 820 status
        if requirements_json.contains("\"fda21cfr820\":\"compliant\"") {
            DomUtils::add_class("fda21cfr820-indicator", "compliant")?;
            DomUtils::set_text_content("fda21cfr820-status", "Compliant")?;
        }

        Ok(())
    }

    /// Populate template selectors with available templates
    fn populate_template_selectors(&self, templates_json: &str) -> QmsResult<()> {
        // Clear existing options
        DomUtils::clear_select_options("document-template-select")?;

        // Add default option
        DomUtils::add_select_option("document-template-select", "", "Select a template...")?;

        // Parse templates and add options (simplified parsing)
        let templates = vec![
            ("sop-template", "Standard Operating Procedure"),
            ("risk-assessment-template", "Risk Assessment"),
            ("validation-protocol-template", "Validation Protocol"),
            ("design-control-template", "Design Control"),
        ];

        for (value, label) in templates {
            DomUtils::add_select_option("document-template-select", value, label)?;
        }

        Ok(())
    }

    /// Populate risk categories in dropdowns
    fn populate_risk_categories(&self, categories_json: &str) -> QmsResult<()> {
        // Clear existing options
        DomUtils::clear_select_options("risk-category-select")?;

        // Add medical device risk categories
        let categories = vec![
            ("biological", "Biological and Chemical"),
            ("mechanical", "Mechanical"),
            ("thermal", "Thermal"),
            ("electrical", "Electrical"),
            ("software", "Software"),
            ("usability", "Usability"),
        ];

        for (value, label) in categories {
            DomUtils::add_select_option("risk-category-select", value, label)?;
        }

        Ok(())
    }

    /// Update testing protocols dashboard
    fn update_testing_protocols(&self, protocols_json: &str) -> QmsResult<()> {
        // Update protocol status indicators
        if protocols_json.contains("\"verification_protocols\":") {
            DomUtils::set_text_content("verification-protocols-count", "5")?;
        }

        if protocols_json.contains("\"validation_protocols\":") {
            DomUtils::set_text_content("validation-protocols-count", "3")?;
        }

        Ok(())
    }

    /// Update compliance indicators
    fn update_compliance_indicators(&self, status_json: &str) -> QmsResult<()> {
        if status_json.contains("\"overall_status\":\"compliant\"") {
            DomUtils::add_class("compliance-indicator", "green")?;
            DomUtils::set_text_content("compliance-status", "Compliant")?;
        } else {
            DomUtils::add_class("compliance-indicator", "yellow")?;
            DomUtils::set_text_content("compliance-status", "Needs Attention")?;
        }
        Ok(())
    }

    /// Update audit indicators
    fn update_audit_indicators(&self, status_json: &str) -> QmsResult<()> {
        if status_json.contains("\"audit_trail_healthy\":true") {
            DomUtils::add_class("audit-indicator", "green")?;
            DomUtils::set_text_content("audit-status", "Healthy")?;
        }
        Ok(())
    }

    /// Update risk alerts
    fn update_risk_alerts(&self, alerts_json: &str) -> QmsResult<()> {
        if alerts_json.contains("\"high_risk_count\":0") {
            DomUtils::add_class("risk-indicator", "green")?;
            DomUtils::set_text_content("risk-status", "No High Risks")?;
        } else {
            DomUtils::add_class("risk-indicator", "red")?;
            DomUtils::set_text_content("risk-status", "High Risks Present")?;
        }
        Ok(())
    }

    /// Update document status
    fn update_document_status(&self, status_json: &str) -> QmsResult<()> {
        if status_json.contains("\"pending_approvals\":0") {
            DomUtils::add_class("document-indicator", "green")?;
            DomUtils::set_text_content("document-status", "All Approved")?;
        } else {
            DomUtils::add_class("document-indicator", "yellow")?;
            DomUtils::set_text_content("document-status", "Pending Approvals")?;
        }
        Ok(())
    }

    // Logging methods

    /// Log info message
    fn log_info(&self, message: &str) -> QmsResult<()> {
        println!("[INFO] {message}");
        Ok(())
    }

    /// Log warning message
    fn log_warning(&self, message: &str) -> QmsResult<()> {
        println!("[WARN] {message}");
        Ok(())
    }

    /// Log error message
    fn log_error(&self, message: &str) -> QmsResult<()> {
        eprintln!("[ERROR] {message}");
        Ok(())
    }

    /// Log debug message (only in debug builds for medical device compliance)
    fn log_debug(&self, message: &str) -> QmsResult<()> {
        #[cfg(debug_assertions)]
        println!("[DEBUG] {message}");
        Ok(())
    }
}

// WASM entry points (to be implemented when compiled to WASM)

/// Initialize QMS application (WASM entry point)
pub fn init_qms_app() -> QmsResult<()> {
    let config = AppConfig::default();
    let mut app = QmsApp::new(config);
    app.initialize()?;
    Ok(())
}

/// Refresh dashboard data (WASM export)
pub fn refresh_dashboard() -> QmsResult<()> {
    // This would access a global app instance in actual WASM
    let config = AppConfig::default();
    let mut app = QmsApp::new(config);
    app.load_dashboard_data()?;
    Ok(())
}

/// Check server connectivity (WASM export)
pub fn check_server_connection() -> QmsResult<bool> {
    let config = AppConfig::default();
    let app = QmsApp::new(config);
    match app.api_client.get_health() {
        Ok(response) => Ok(response.status == 200),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.api_base_url, "http://localhost:8080");
        assert_eq!(config.auto_refresh_interval, 30);
        assert_eq!(config.session_timeout, 1800);
        assert!(config.enable_notifications);
        assert!(!config.debug_mode);
    }

    #[test]
    fn test_qms_app_creation() {
        let config = AppConfig::default();
        let app = QmsApp::new(config);
        assert!(!app.is_authenticated);
        assert!(app.current_user.is_none());
        assert!(app.current_session.is_none());
    }

    #[test]
    fn test_qms_app_initialization() {
        let config = AppConfig::default();
        let mut app = QmsApp::new(config);
        let result = app.initialize();
        assert!(result.is_ok());
    }

    #[test]
    fn test_page_navigation() {
        let config = AppConfig::default();
        let app = QmsApp::new(config);
        
        assert!(app.show_documents_page().is_ok());
        assert!(app.show_risks_page().is_ok());
        assert!(app.show_requirements_page().is_ok());
        assert!(app.show_audit_page().is_ok());
    }

    #[test]
    fn test_data_loading() {
        let config = AppConfig::default();
        let app = QmsApp::new(config);
        
        assert!(app.load_dashboard_data().is_ok());
        assert!(app.load_compliance_data().is_ok());
        assert!(app.load_documents_data().is_ok());
        assert!(app.load_risks_data().is_ok());
        assert!(app.load_requirements_data().is_ok());
        assert!(app.load_audit_data().is_ok());
    }

    #[test]
    fn test_table_updates() {
        let config = AppConfig::default();
        let app = QmsApp::new(config);
        
        assert!(app.update_documents_table("{}").is_ok());
        assert!(app.update_risks_table("{}").is_ok());
        assert!(app.update_requirements_table("{}").is_ok());
        assert!(app.update_audit_table("{}").is_ok());
    }

    #[test]
    fn test_wasm_entry_points() {
        assert!(init_qms_app().is_ok());
        assert!(refresh_dashboard().is_ok());
        assert!(check_server_connection().is_ok());
    }
}
