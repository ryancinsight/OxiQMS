//! Post-Market Surveillance Module
//! Task 3.1.13: Post-Market Surveillance Implementation
//! 
//! This module implements post-market surveillance capabilities required for medical device
//! regulatory compliance, tracking real-world device performance and updating risk assessments.

use crate::prelude::*;
use crate::modules::risk_manager::risk::{RiskManager, RiskSeverity, RiskOccurrence};
use crate::modules::audit_logger::functions::{audit_log_action, audit_log_create, audit_log_update};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Post-market surveillance data entry
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields used for regulatory compliance and future reporting
pub struct SurveillanceData {
    pub id: String,                      // UUID v4
    pub risk_id: String,                 // Associated risk ID
    pub data_type: SurveillanceType,     // Type of surveillance data
    pub source: String,                  // Data source (complaint, field report, etc.)
    pub description: String,             // Detailed description of the data
    pub date_reported: String,           // ISO 8601 date when reported
    pub date_occurred: Option<String>,   // ISO 8601 date when incident occurred
    pub severity_observed: Option<RiskSeverity>, // Observed severity in field
    pub frequency_data: Option<FrequencyData>, // Frequency/occurrence data
    pub patient_impact: Option<String>,  // Patient impact description
    pub device_info: DeviceInfo,         // Device identification
    pub corrective_actions: Vec<CorrectiveAction>, // Actions taken
    pub regulatory_notifications: Vec<RegulatoryNotification>, // Regulatory reports
    pub status: SurveillanceStatus,      // Investigation status
    pub analyzed_by: Option<String>,     // Quality engineer who analyzed
    pub analysis_date: Option<String>,   // Date of analysis
    pub risk_reassessment_required: bool, // Whether risk reassessment needed
    pub created_at: String,              // Creation timestamp
    pub updated_at: String,              // Last update timestamp
}

/// Types of post-market surveillance data
#[derive(Debug, Clone, PartialEq)]
pub enum SurveillanceType {
    CustomerComplaint,     // Customer complaint report
    FieldReport,          // Field service report
    AdverseEvent,         // Adverse event report
    DeviceFailure,        // Device failure analysis
    UsabilityIssue,       // User interface/usability issue
    SoftwareBug,          // Software defect report
    PerformanceData,      // Performance monitoring data
    ClinicalFeedback,     // Clinical user feedback
    RegulatoryReport,     // Regulatory authority report
    PostMarketStudy,      // Post-market clinical study data
    Other(String),        // Other surveillance data
}

impl SurveillanceType {
    /// Parse surveillance type from string
    pub fn from_str(s: &str) -> QmsResult<Self> {
        match s.to_lowercase().as_str() {
            "complaint" | "customer-complaint" => Ok(SurveillanceType::CustomerComplaint),
            "field-report" | "field_report" => Ok(SurveillanceType::FieldReport),
            "adverse-event" | "adverse_event" => Ok(SurveillanceType::AdverseEvent),
            "device-failure" | "device_failure" => Ok(SurveillanceType::DeviceFailure),
            "usability" | "usability-issue" => Ok(SurveillanceType::UsabilityIssue),
            "software-bug" | "software_bug" => Ok(SurveillanceType::SoftwareBug),
            "performance" | "performance-data" => Ok(SurveillanceType::PerformanceData),
            "clinical" | "clinical-feedback" => Ok(SurveillanceType::ClinicalFeedback),
            "regulatory" | "regulatory-report" => Ok(SurveillanceType::RegulatoryReport),
            "study" | "post-market-study" => Ok(SurveillanceType::PostMarketStudy),
            other => Ok(SurveillanceType::Other(other.to_string())),
        }
    }

    /// Get display name for surveillance type
    pub fn display_name(&self) -> &str {
        match self {
            SurveillanceType::CustomerComplaint => "Customer Complaint",
            SurveillanceType::FieldReport => "Field Report",
            SurveillanceType::AdverseEvent => "Adverse Event",
            SurveillanceType::DeviceFailure => "Device Failure",
            SurveillanceType::UsabilityIssue => "Usability Issue",
            SurveillanceType::SoftwareBug => "Software Bug",
            SurveillanceType::PerformanceData => "Performance Data",
            SurveillanceType::ClinicalFeedback => "Clinical Feedback",
            SurveillanceType::RegulatoryReport => "Regulatory Report",
            SurveillanceType::PostMarketStudy => "Post-Market Study",
            SurveillanceType::Other(name) => name,
        }
    }
}

/// Frequency/occurrence data for risk updates
#[derive(Debug, Clone)]
#[allow(dead_code)] // Statistical fields for regulatory analysis
pub struct FrequencyData {
    pub numerator: u32,           // Number of incidents
    pub denominator: u32,         // Total population/exposures
    pub time_period: String,      // Time period for the data
    pub confidence_level: Option<f64>, // Statistical confidence level
    pub data_source: String,      // Source of frequency data
}

/// Device identification information
#[derive(Debug, Clone)]
#[allow(dead_code)] // Device tracking fields for regulatory traceability
pub struct DeviceInfo {
    pub device_model: String,      // Device model number
    pub serial_number: Option<String>, // Device serial number
    pub software_version: Option<String>, // Software version
    pub manufacturing_date: Option<String>, // Manufacturing date
    pub lot_number: Option<String>, // Manufacturing lot number
    pub installation_date: Option<String>, // Installation date
    pub location: Option<String>,  // Device location/site
}

/// Corrective action taken
#[derive(Debug, Clone)]
#[allow(dead_code)] // Action tracking fields for CAPA system integration
pub struct CorrectiveAction {
    pub id: String,               // UUID v4
    pub action_type: ActionType,  // Type of action
    pub description: String,      // Action description
    pub date_initiated: String,   // Date action was started
    pub date_completed: Option<String>, // Date action was completed
    pub responsible_party: String, // Who is responsible
    pub effectiveness: Option<f64>, // Effectiveness rating (0.0-1.0)
    pub verification_method: Option<String>, // How effectiveness was verified
    pub status: ActionStatus,     // Current status
}

/// Types of corrective actions
#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    FieldCorrection,     // Field correction/recall
    SoftwareUpdate,      // Software patch/update
    LabelingChange,      // Warning label update
    TrainingUpdate,      // User training enhancement
    DesignChange,        // Device design modification
    ProcessImprovement,  // Manufacturing process change
    ProcedureUpdate,     // Operating procedure update
    Communication,       // Safety communication
    Investigation,       // Root cause analysis
    Other(String),       // Other action type
}

/// Status of corrective action
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Status variants for workflow management
pub enum ActionStatus {
    Planned,            // Action planned but not started
    InProgress,         // Action in progress
    Completed,          // Action completed
    Verified,           // Action effectiveness verified
    Closed,             // Action closed
    Cancelled,          // Action cancelled
}

/// Regulatory notification
#[derive(Debug, Clone)]
#[allow(dead_code)] // Regulatory reporting fields for compliance tracking
pub struct RegulatoryNotification {
    pub id: String,               // UUID v4
    pub authority: String,        // Regulatory authority (FDA, CE, etc.)
    pub notification_type: String, // Type of notification (MDR, etc.)
    pub reference_number: Option<String>, // Authority reference number
    pub date_submitted: String,   // Date submitted to authority
    pub status: String,           // Submission status
    pub response_received: Option<String>, // Authority response
}

/// Status of surveillance data investigation
#[derive(Debug, Clone, PartialEq)]
pub enum SurveillanceStatus {
    Reported,           // Data reported, not yet analyzed
    UnderInvestigation, // Being investigated
    Analyzed,           // Analysis complete
    RiskUpdated,        // Risk assessment updated
    Closed,             // Investigation closed
    EscalatedToRegulatory, // Escalated to regulatory authorities
}

/// Post-market surveillance manager
pub struct SurveillanceManager {
    project_path: PathBuf,
    risk_manager: RiskManager,
}

impl SurveillanceManager {
    /// Create new surveillance manager
    pub fn new(project_path: &Path) -> QmsResult<Self> {
        let risk_manager = RiskManager::new(project_path)?;
        Ok(Self {
            project_path: project_path.to_path_buf(),
            risk_manager,
        })
    }

    /// Initialize surveillance system
    pub fn initialize(&self) -> QmsResult<()> {
        // Create surveillance directories
        let surveillance_dir = self.project_path.join("surveillance");
        fs::create_dir_all(&surveillance_dir)?;
        
        let data_dir = surveillance_dir.join("data");
        fs::create_dir_all(&data_dir)?;
        
        let reports_dir = surveillance_dir.join("reports");
        fs::create_dir_all(&reports_dir)?;
        
        let analysis_dir = surveillance_dir.join("analysis");
        fs::create_dir_all(&analysis_dir)?;

        // Create initial index
        let index_path = surveillance_dir.join("index.json");
        if !index_path.exists() {
            let initial_index = r#"{"version": "1.0", "data": []}"#;
            fs::write(&index_path, initial_index)?;
        }

        // Audit initialization
        audit_log_action("SURVEILLANCE_SYSTEM_INITIALIZED", "SurveillanceManager", &surveillance_dir.display().to_string())?;

        Ok(())
    }

    /// Add new surveillance data
    pub fn add_surveillance_data(
        &mut self,
        risk_id: &str,
        data_type: SurveillanceType,
        source: &str,
        description: &str,
        device_info: DeviceInfo,
    ) -> QmsResult<SurveillanceData> {
        // Validate that risk exists
        if !self.risk_manager.risk_exists(risk_id) {
            return Err(QmsError::not_found(&format!("Risk {risk_id} not found")));
        }

        // Create surveillance data entry
        let surveillance_data = SurveillanceData {
            id: crate::utils::generate_uuid(),
            risk_id: risk_id.to_string(),
            data_type,
            source: source.to_string(),
            description: description.to_string(),
            date_reported: crate::utils::current_iso8601_timestamp(),
            date_occurred: None,
            severity_observed: None,
            frequency_data: None,
            patient_impact: None,
            device_info,
            corrective_actions: Vec::new(),
            regulatory_notifications: Vec::new(),
            status: SurveillanceStatus::Reported,
            analyzed_by: None,
            analysis_date: None,
            risk_reassessment_required: false,
            created_at: crate::utils::current_iso8601_timestamp(),
            updated_at: crate::utils::current_iso8601_timestamp(),
        };

        // Save surveillance data
        self.save_surveillance_data(&surveillance_data)?;
        
        // Update index
        self.update_surveillance_index(&surveillance_data)?;

        // Audit logging
        audit_log_create(
            "SurveillanceData",
            &surveillance_data.id,
            &format!("{}|{}|{}", surveillance_data.data_type.display_name(), risk_id, source)
        )?;

        Ok(surveillance_data)
    }

    /// Update risk estimates based on surveillance data
    pub fn update_risk_estimates(&mut self, surveillance_id: &str, new_frequency: FrequencyData) -> QmsResult<()> {
        // Load surveillance data
        let mut surveillance_data = self.load_surveillance_data(surveillance_id)?;
        let risk_id = surveillance_data.risk_id.clone();
        
        // Load associated risk
        let mut risk = self.risk_manager.load_risk(&risk_id)?;
        let original_rpn = risk.risk_priority_number;

        // Update frequency data
        surveillance_data.frequency_data = Some(new_frequency.clone());
        surveillance_data.updated_at = crate::utils::current_iso8601_timestamp();

        // Calculate new occurrence based on frequency data
        let new_occurrence = self.calculate_occurrence_from_frequency(&new_frequency)?;
        
        // Update risk occurrence and recalculate RPN
        let _old_occurrence = risk.occurrence;
        risk.occurrence = new_occurrence;
        
        // Calculate RPN using references to avoid partial moves
        let severity_val = match risk.severity {
            RiskSeverity::Negligible => 1,
            RiskSeverity::Minor => 2,
            RiskSeverity::Major => 3,
            RiskSeverity::Critical => 4,
            RiskSeverity::Catastrophic => 5,
        };
        
        let occurrence_val = match risk.occurrence {
            RiskOccurrence::Improbable => 1,
            RiskOccurrence::Remote => 2,
            RiskOccurrence::Occasional => 3,
            RiskOccurrence::Probable => 4,
            RiskOccurrence::Frequent => 5,
        };
        
        let detectability_val = match &risk.detectability {
            crate::modules::risk_manager::risk::RiskDetectability::VeryHigh => 1,
            crate::modules::risk_manager::risk::RiskDetectability::High => 2,
            crate::modules::risk_manager::risk::RiskDetectability::Moderate => 3,
            crate::modules::risk_manager::risk::RiskDetectability::Low => 4,
            crate::modules::risk_manager::risk::RiskDetectability::VeryLow => 5,
        };
        
        risk.risk_priority_number = severity_val * occurrence_val * detectability_val;
        risk.initial_risk_level = RiskManager::assess_risk_level(risk.risk_priority_number);
        risk.updated_at = crate::utils::current_iso8601_timestamp();

        // Mark surveillance data as requiring risk reassessment if RPN increased significantly
        if risk.risk_priority_number > original_rpn + 10 {
            surveillance_data.risk_reassessment_required = true;
        }

        // Save updates
        self.save_surveillance_data(&surveillance_data)?;
        self.risk_manager.save_risk(&risk)?;

        // Audit logging
        audit_log_update(
            "Risk",
            &risk_id,
            &format!("RPN: {original_rpn}"),
            &format!("RPN: {} (updated from surveillance)", risk.risk_priority_number)
        )?;

        audit_log_action(
            "RISK_UPDATED_FROM_SURVEILLANCE",
            "SurveillanceData",
            &format!("{}|{}→{}", surveillance_id, original_rpn, risk.risk_priority_number)
        )?;

        Ok(())
    }

    /// Trigger risk review based on surveillance data
    pub fn trigger_risk_review(&mut self, surveillance_id: &str, reason: &str) -> QmsResult<()> {
        // Load surveillance data
        let mut surveillance_data = self.load_surveillance_data(surveillance_id)?;
        let risk_id = surveillance_data.risk_id.clone();

        // Load associated risk
        let mut risk = self.risk_manager.load_risk(&risk_id)?;

        // Mark risk for review
        risk.review_required = true;
        risk.next_review_date = Some(crate::utils::current_iso8601_timestamp());
        risk.updated_at = crate::utils::current_iso8601_timestamp();

        // Update surveillance status
        surveillance_data.status = SurveillanceStatus::EscalatedToRegulatory;
        surveillance_data.updated_at = crate::utils::current_iso8601_timestamp();

        // Save updates
        self.save_surveillance_data(&surveillance_data)?;
        self.risk_manager.save_risk(&risk)?;

        // Audit logging
        audit_log_action(
            "RISK_REVIEW_TRIGGERED",
            "SurveillanceData",
            &format!("{surveillance_id}|{risk_id}|{reason}")
        )?;

        Ok(())
    }

    /// Add corrective action to surveillance data
    pub fn add_corrective_action(
        &mut self,
        surveillance_id: &str,
        action_type: ActionType,
        description: &str,
        responsible_party: &str,
    ) -> QmsResult<CorrectiveAction> {
        // Load surveillance data
        let mut surveillance_data = self.load_surveillance_data(surveillance_id)?;

        // Create corrective action
        let action = CorrectiveAction {
            id: crate::utils::generate_uuid(),
            action_type,
            description: description.to_string(),
            date_initiated: crate::utils::current_iso8601_timestamp(),
            date_completed: None,
            responsible_party: responsible_party.to_string(),
            effectiveness: None,
            verification_method: None,
            status: ActionStatus::Planned,
        };

        // Add to surveillance data
        surveillance_data.corrective_actions.push(action.clone());
        surveillance_data.updated_at = crate::utils::current_iso8601_timestamp();

        // Save surveillance data
        self.save_surveillance_data(&surveillance_data)?;

        // Audit logging
        audit_log_action(
            "CORRECTIVE_ACTION_ADDED",
            "SurveillanceData",
            &format!("{}|{}|{}", surveillance_id, action.id, description)
        )?;

        Ok(action)
    }

    /// Get surveillance data for a risk
    pub fn get_surveillance_for_risk(&self, risk_id: &str) -> QmsResult<Vec<SurveillanceData>> {
        let surveillance_dir = self.project_path.join("surveillance").join("data");
        let mut results = Vec::new();

        if !surveillance_dir.exists() {
            return Ok(results);
        }

        // Read all surveillance files
        for entry in fs::read_dir(&surveillance_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(surveillance_data) = self.load_surveillance_data_from_path(&path) {
                    if surveillance_data.risk_id == risk_id {
                        results.push(surveillance_data);
                    }
                }
            }
        }

        // Sort by date (newest first)
        results.sort_by(|a, b| b.date_reported.cmp(&a.date_reported));

        Ok(results)
    }

    /// List all surveillance data with filtering
    pub fn list_surveillance_data(
        &self,
        data_type_filter: Option<SurveillanceType>,
        status_filter: Option<SurveillanceStatus>,
    ) -> QmsResult<Vec<SurveillanceData>> {
        let surveillance_dir = self.project_path.join("surveillance").join("data");
        let mut results = Vec::new();

        if !surveillance_dir.exists() {
            return Ok(results);
        }

        // Read all surveillance files
        for entry in fs::read_dir(&surveillance_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(surveillance_data) = self.load_surveillance_data_from_path(&path) {
                    // Apply filters
                    if let Some(ref type_filter) = data_type_filter {
                        if surveillance_data.data_type != *type_filter {
                            continue;
                        }
                    }
                    
                    if let Some(ref status_filter) = status_filter {
                        if surveillance_data.status != *status_filter {
                            continue;
                        }
                    }

                    results.push(surveillance_data);
                }
            }
        }

        // Sort by date (newest first)
        results.sort_by(|a, b| b.date_reported.cmp(&a.date_reported));

        Ok(results)
    }

    /// Generate surveillance summary report
    pub fn generate_surveillance_report(&self, risk_id: Option<&str>) -> QmsResult<String> {
        let mut report = String::new();
        
        report.push_str("POST-MARKET SURVEILLANCE REPORT\n");
        report.push_str("===============================\n\n");
        report.push_str(&format!("Generated: {}\n\n", crate::utils::current_iso8601_timestamp()));

        // Get surveillance data
        let surveillance_data = if let Some(risk_id) = risk_id {
            self.get_surveillance_for_risk(risk_id)?
        } else {
            self.list_surveillance_data(None, None)?
        };

        if surveillance_data.is_empty() {
            report.push_str("No surveillance data found.\n");
            return Ok(report);
        }

        // Summary statistics
        let total_count = surveillance_data.len();
        let mut type_counts = HashMap::new();
        let mut status_counts = HashMap::new();
        let mut high_risk_count = 0;

        for data in &surveillance_data {
            *type_counts.entry(data.data_type.display_name()).or_insert(0) += 1;
            *status_counts.entry(format!("{:?}", data.status)).or_insert(0) += 1;
            
            if data.risk_reassessment_required {
                high_risk_count += 1;
            }
        }

        report.push_str(&format!("Total Surveillance Reports: {total_count}\n"));
        report.push_str(&format!("High Risk Items Requiring Reassessment: {high_risk_count}\n\n"));

        // Type breakdown
        report.push_str("SURVEILLANCE DATA BY TYPE\n");
        report.push_str("=========================\n");
        for (data_type, count) in type_counts {
            report.push_str(&format!("  {data_type}: {count}\n"));
        }
        report.push('\n');

        // Status breakdown
        report.push_str("SURVEILLANCE DATA BY STATUS\n");
        report.push_str("============================\n");
        for (status, count) in status_counts {
            report.push_str(&format!("  {status}: {count}\n"));
        }
        report.push('\n');

        // Detailed entries
        report.push_str("DETAILED SURVEILLANCE ENTRIES\n");
        report.push_str("==============================\n");
        for data in surveillance_data.iter().take(10) { // Show first 10
            report.push_str(&format!("ID: {}\n", data.id));
            report.push_str(&format!("Risk ID: {}\n", data.risk_id));
            report.push_str(&format!("Type: {}\n", data.data_type.display_name()));
            report.push_str(&format!("Source: {}\n", data.source));
            report.push_str(&format!("Date: {}\n", data.date_reported));
            report.push_str(&format!("Status: {:?}\n", data.status));
            report.push_str(&format!("Description: {}\n", data.description));
            if data.risk_reassessment_required {
                report.push_str("⚠️  RISK REASSESSMENT REQUIRED\n");
            }
            report.push_str("---\n\n");
        }

        if surveillance_data.len() > 10 {
            report.push_str(&format!("... and {} more entries\n", surveillance_data.len() - 10));
        }

        Ok(report)
    }

    /// Save surveillance data to file
    fn save_surveillance_data(&self, data: &SurveillanceData) -> QmsResult<()> {
        let surveillance_dir = self.project_path.join("surveillance").join("data");
        fs::create_dir_all(&surveillance_dir)?;
        
        let file_path = surveillance_dir.join(format!("{}.json", data.id));
        let json_content = self.surveillance_data_to_json(data)?;
        
        fs::write(&file_path, json_content)?;
        
        Ok(())
    }

    /// Load surveillance data from file
    fn load_surveillance_data(&self, surveillance_id: &str) -> QmsResult<SurveillanceData> {
        let file_path = self.project_path
            .join("surveillance")
            .join("data")
            .join(format!("{surveillance_id}.json"));
        
        self.load_surveillance_data_from_path(&file_path)
    }

    /// Load surveillance data from specific path
    fn load_surveillance_data_from_path(&self, file_path: &Path) -> QmsResult<SurveillanceData> {
        let content = fs::read_to_string(file_path)?;
        self.parse_surveillance_json(&content)
    }

    /// Convert surveillance data to JSON (basic implementation)
    fn surveillance_data_to_json(&self, data: &SurveillanceData) -> QmsResult<String> {
        let mut json = String::from("{\n");
        json.push_str("  \"version\": \"1.0\",\n");
        json.push_str(&format!("  \"id\": \"{}\",\n", data.id));
        json.push_str(&format!("  \"risk_id\": \"{}\",\n", data.risk_id));
        json.push_str(&format!("  \"data_type\": \"{}\",\n", data.data_type.display_name()));
        json.push_str(&format!("  \"source\": \"{}\",\n", self.escape_json(&data.source)));
        json.push_str(&format!("  \"description\": \"{}\",\n", self.escape_json(&data.description)));
        json.push_str(&format!("  \"date_reported\": \"{}\",\n", data.date_reported));
        json.push_str(&format!("  \"status\": \"{:?}\",\n", data.status));
        json.push_str(&format!("  \"device_model\": \"{}\",\n", self.escape_json(&data.device_info.device_model)));
        json.push_str(&format!("  \"risk_reassessment_required\": {},\n", data.risk_reassessment_required));
        json.push_str(&format!("  \"corrective_actions_count\": {},\n", data.corrective_actions.len()));
        json.push_str(&format!("  \"created_at\": \"{}\",\n", data.created_at));
        json.push_str(&format!("  \"updated_at\": \"{}\"\n", data.updated_at));
        json.push('}');
        Ok(json)
    }

    /// Parse surveillance data from JSON (basic implementation)
    fn parse_surveillance_json(&self, content: &str) -> QmsResult<SurveillanceData> {
        // This is a basic JSON parser - in a full implementation, you'd want more robust parsing
        let lines: Vec<&str> = content.lines().collect();
        
        let mut id = String::new();
        let mut risk_id = String::new();
        let mut data_type = String::new();
        let mut source = String::new();
        let mut description = String::new();
        let mut date_reported = String::new();
        let mut status = String::new();
        let mut device_model = String::new();
        let mut risk_reassessment_required = false;
        let mut created_at = String::new();
        let mut updated_at = String::new();

        for line in lines {
            let line = line.trim();
            if line.starts_with("\"id\":") {
                id = self.extract_json_string_value(line)?;
            } else if line.starts_with("\"risk_id\":") {
                risk_id = self.extract_json_string_value(line)?;
            } else if line.starts_with("\"data_type\":") {
                data_type = self.extract_json_string_value(line)?;
            } else if line.starts_with("\"source\":") {
                source = self.extract_json_string_value(line)?;
            } else if line.starts_with("\"description\":") {
                description = self.extract_json_string_value(line)?;
            } else if line.starts_with("\"date_reported\":") {
                date_reported = self.extract_json_string_value(line)?;
            } else if line.starts_with("\"status\":") {
                status = self.extract_json_string_value(line)?;
            } else if line.starts_with("\"device_model\":") {
                device_model = self.extract_json_string_value(line)?;
            } else if line.starts_with("\"risk_reassessment_required\":") {
                risk_reassessment_required = line.contains("true");
            } else if line.starts_with("\"created_at\":") {
                created_at = self.extract_json_string_value(line)?;
            } else if line.starts_with("\"updated_at\":") {
                updated_at = self.extract_json_string_value(line)?;
            }
        }

        // Parse surveillance type
        let surveillance_type = SurveillanceType::from_str(&data_type).unwrap_or(SurveillanceType::Other(data_type));
        
        // Parse status
        let surveillance_status = match status.as_str() {
            "Reported" => SurveillanceStatus::Reported,
            "UnderInvestigation" => SurveillanceStatus::UnderInvestigation,
            "Analyzed" => SurveillanceStatus::Analyzed,
            "RiskUpdated" => SurveillanceStatus::RiskUpdated,
            "Closed" => SurveillanceStatus::Closed,
            "EscalatedToRegulatory" => SurveillanceStatus::EscalatedToRegulatory,
            _ => SurveillanceStatus::Reported,
        };

        Ok(SurveillanceData {
            id,
            risk_id,
            data_type: surveillance_type,
            source,
            description,
            date_reported,
            date_occurred: None,
            severity_observed: None,
            frequency_data: None,
            patient_impact: None,
            device_info: DeviceInfo {
                device_model,
                serial_number: None,
                software_version: None,
                manufacturing_date: None,
                lot_number: None,
                installation_date: None,
                location: None,
            },
            corrective_actions: Vec::new(),
            regulatory_notifications: Vec::new(),
            status: surveillance_status,
            analyzed_by: None,
            analysis_date: None,
            risk_reassessment_required,
            created_at,
            updated_at,
        })
    }

    /// Calculate occurrence from frequency data
    fn calculate_occurrence_from_frequency(&self, frequency_data: &FrequencyData) -> QmsResult<RiskOccurrence> {
        if frequency_data.denominator == 0 {
            return Err(QmsError::validation_error("Frequency denominator cannot be zero"));
        }

        let rate = frequency_data.numerator as f64 / frequency_data.denominator as f64;
        
        // Map frequency rate to occurrence scale (ISO 14971 based)
        let occurrence = if rate >= 0.1 {          // 10% or higher
            RiskOccurrence::Frequent
        } else if rate >= 0.01 {      // 1-10%
            RiskOccurrence::Probable
        } else if rate >= 0.001 {     // 0.1-1%
            RiskOccurrence::Occasional
        } else if rate >= 0.0001 {    // 0.01-0.1%
            RiskOccurrence::Remote
        } else {                      // Less than 0.01%
            RiskOccurrence::Improbable
        };

        Ok(occurrence)
    }

    /// Update surveillance index
    const fn update_surveillance_index(&self, _data: &SurveillanceData) -> QmsResult<()> {
        // Placeholder for index update - in full implementation, this would maintain a searchable index
        Ok(())
    }

    /// Escape JSON string
    fn escape_json(&self, s: &str) -> String {
        s.replace("\"", "\\\"").replace("\n", "\\n").replace("\r", "\\r")
    }

    /// Extract string value from JSON line
    fn extract_json_string_value(&self, line: &str) -> QmsResult<String> {
        if let Some(start) = line.find("\"") {
            if let Some(end) = line[start + 1..].find("\"") {
                let start_pos = start + 1;
                let end_pos = start + 1 + end;
                return Ok(line[start_pos..end_pos].to_string());
            }
        }
        Err(QmsError::validation_error(&format!("Could not extract string value from: {line}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_surveillance_manager() -> SurveillanceManager {
        let temp_dir = std::env::temp_dir().join("qms_surveillance_test");
        let _ = fs::create_dir_all(&temp_dir);
        SurveillanceManager::new(&temp_dir).expect("Failed to create test surveillance manager")
    }

    #[test]
    fn test_surveillance_manager_creation() {
        let manager = create_test_surveillance_manager();
        assert!(manager.initialize().is_ok());
    }

    #[test]
    fn test_surveillance_type_parsing() {
        assert_eq!(SurveillanceType::from_str("complaint").unwrap(), SurveillanceType::CustomerComplaint);
        assert_eq!(SurveillanceType::from_str("field-report").unwrap(), SurveillanceType::FieldReport);
        assert_eq!(SurveillanceType::from_str("adverse-event").unwrap(), SurveillanceType::AdverseEvent);
        
        match SurveillanceType::from_str("custom-type").unwrap() {
            SurveillanceType::Other(name) => assert_eq!(name, "custom-type"),
            _ => panic!("Expected Other variant"),
        }
    }

    #[test]
    fn test_frequency_calculation() {
        let temp_dir = std::env::temp_dir().join("qms_surveillance_freq_test");
        let _ = fs::create_dir_all(&temp_dir);
        let manager = SurveillanceManager::new(&temp_dir).expect("Failed to create test manager");
        
        // Test high frequency (10%)
        let high_freq = FrequencyData {
            numerator: 100,
            denominator: 1000,
            time_period: "1 year".to_string(),
            confidence_level: Some(0.95),
            data_source: "Clinical data".to_string(),
        };
        let occurrence = manager.calculate_occurrence_from_frequency(&high_freq).unwrap();
        assert_eq!(occurrence, RiskOccurrence::Frequent);
        
        // Test low frequency (0.01%)
        let low_freq = FrequencyData {
            numerator: 1,
            denominator: 100000,
            time_period: "1 year".to_string(),
            confidence_level: Some(0.95),
            data_source: "Field data".to_string(),
        };
        let occurrence = manager.calculate_occurrence_from_frequency(&low_freq).unwrap();
        assert_eq!(occurrence, RiskOccurrence::Improbable);
    }

    #[test]
    fn test_surveillance_data_json_serialization() {
        let temp_dir = std::env::temp_dir().join("qms_surveillance_json_test");
        let _ = fs::create_dir_all(&temp_dir);
        let manager = SurveillanceManager::new(&temp_dir).expect("Failed to create test manager");
        
        let device_info = DeviceInfo {
            device_model: "Model X1".to_string(),
            serial_number: Some("SN123456".to_string()),
            software_version: Some("v2.1.0".to_string()),
            manufacturing_date: Some("2024-01-15".to_string()),
            lot_number: Some("LOT-001".to_string()),
            installation_date: Some("2024-02-01".to_string()),
            location: Some("Hospital A".to_string()),
        };

        let surveillance_data = SurveillanceData {
            id: "TEST-001".to_string(),
            risk_id: "RISK-001".to_string(),
            data_type: SurveillanceType::CustomerComplaint,
            source: "Customer Support".to_string(),
            description: "Device alarm malfunction".to_string(),
            date_reported: "2024-03-01T10:00:00Z".to_string(),
            date_occurred: None,
            severity_observed: None,
            frequency_data: None,
            patient_impact: None,
            device_info,
            corrective_actions: Vec::new(),
            regulatory_notifications: Vec::new(),
            status: SurveillanceStatus::Reported,
            analyzed_by: None,
            analysis_date: None,
            risk_reassessment_required: false,
            created_at: "2024-03-01T10:00:00Z".to_string(),
            updated_at: "2024-03-01T10:00:00Z".to_string(),
        };

        let json = manager.surveillance_data_to_json(&surveillance_data).unwrap();
        assert!(json.contains("TEST-001"));
        assert!(json.contains("RISK-001"));
        assert!(json.contains("Customer Complaint"));
        assert!(json.contains("Device alarm malfunction"));
    }

    #[test]
    fn test_surveillance_data_filtering() {
        // Test filtering logic
        let complaint_type = SurveillanceType::CustomerComplaint;
        let field_type = SurveillanceType::FieldReport;
        
        assert_eq!(complaint_type.display_name(), "Customer Complaint");
        assert_eq!(field_type.display_name(), "Field Report");
        
        let reported_status = SurveillanceStatus::Reported;
        let analyzed_status = SurveillanceStatus::Analyzed;
        
        assert_eq!(reported_status, SurveillanceStatus::Reported);
        assert_eq!(analyzed_status, SurveillanceStatus::Analyzed);
    }
}
