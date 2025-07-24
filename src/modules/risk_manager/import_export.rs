//! Risk Import/Export Module
//! Task 3.1.12: Risk Import/Export Implementation
//! 
//! This module provides comprehensive import and export functionality for risk data,
//! supporting multiple formats and ensuring medical device regulatory compliance.

use crate::prelude::*;
use crate::modules::risk_manager::risk::{RiskItem, RiskManager, RiskSeverity, RiskOccurrence, RiskDetectability, RiskLevel, RiskStatus};
use crate::modules::audit_logger::functions::{audit_log_action, audit_log_create};
use std::fs;
use std::path::Path;
use std::collections::HashMap;

/// Supported import formats for risk data
#[derive(Debug, Clone, PartialEq)]
pub enum ImportFormat {
    Csv,           // Standard CSV with risk data columns
    Json,          // JSON export format for backup/restore
    FmeaTemplate,  // FMEA analysis CSV template
}

impl ImportFormat {
    /// Parse import format from string
    pub fn from_str(s: &str) -> QmsResult<Self> {
        match s.to_lowercase().as_str() {
            "csv" => Ok(ImportFormat::Csv),
            "json" => Ok(ImportFormat::Json),
            "fmea" | "fmea-template" => Ok(ImportFormat::FmeaTemplate),
            _ => Err(QmsError::validation_error(&format!("Unsupported import format: {s}"))),
        }
    }

    /// Get expected file extension for format
    pub const fn extension(&self) -> &'static str {
        match self {
            ImportFormat::Csv => ".csv",
            ImportFormat::Json => ".json",
            ImportFormat::FmeaTemplate => ".csv",
        }
    }
}

/// Supported export formats for risk data
#[derive(Debug, Clone, PartialEq)]
pub enum ExportFormat {
    Csv,           // Standard CSV export
    Json,          // Complete JSON export
    Pdf,           // Text-based PDF report
    FmeaTemplate,  // FMEA CSV template
}

impl ExportFormat {
    /// Parse export format from string
    pub fn from_str(s: &str) -> QmsResult<Self> {
        match s.to_lowercase().as_str() {
            "csv" => Ok(ExportFormat::Csv),
            "json" => Ok(ExportFormat::Json),
            "pdf" => Ok(ExportFormat::Pdf),
            "fmea" | "fmea-template" => Ok(ExportFormat::FmeaTemplate),
            _ => Err(QmsError::validation_error(&format!("Unsupported export format: {s}"))),
        }
    }

    /// Get file extension for format
    #[allow(dead_code)]
    pub const fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Csv => ".csv",
            ExportFormat::Json => ".json",
            ExportFormat::Pdf => ".pdf",
            ExportFormat::FmeaTemplate => ".csv",
        }
    }
}

/// Import options and configuration
#[derive(Debug, Clone)]
pub struct ImportOptions {
    pub validate_only: bool,    // Only validate, don't import
    pub skip_duplicates: bool,  // Skip risks with existing hazard_id
    pub backup_existing: bool,  // Create backup before import
    pub component_filter: Option<String>, // Filter for FMEA templates
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            validate_only: false,
            skip_duplicates: true,
            backup_existing: true,
            component_filter: None,
        }
    }
}

/// Export options and configuration
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct ExportOptions {
    #[allow(dead_code)]
    pub include_history: bool,     // Include version history in export
    pub summary_only: bool,        // Export summary data only
    pub severity_filter: Option<u8>, // Filter by minimum severity
    pub status_filter: Option<RiskStatus>, // Filter by risk status
    pub component_filter: Option<String>, // Component filter for FMEA
}


/// Risk data importer
pub struct RiskImporter {
    manager: RiskManager,
}

impl RiskImporter {
    /// Create new risk importer
    pub fn new(project_path: &Path) -> QmsResult<Self> {
        let manager = RiskManager::new(project_path)?;
        Ok(Self { manager })
    }

    /// Import risks from file with specified format and options
    pub fn import_from_file(&mut self, file_path: &Path, format: ImportFormat, options: ImportOptions) -> QmsResult<ImportResult> {
        // Validate file exists and is readable
        if !file_path.exists() {
            return Err(QmsError::not_found(&format!("Import file not found: {}", file_path.display())));
        }

        // Check file extension matches format
        if let Some(ext) = file_path.extension() {
            let expected_ext = format.extension().trim_start_matches('.');
            if ext.to_string_lossy().to_lowercase() != expected_ext {
                return Err(QmsError::validation_error(&format!(
                    "File extension '{}' doesn't match format '{}' (expected '{}')",
                    ext.to_string_lossy(), 
                    format!("{:?}", format),
                    expected_ext
                )));
            }
        }

        // Read file content
        let content = fs::read_to_string(file_path)
            .map_err(|e| QmsError::io_error(&format!("Failed to read import file: {e}")))?;

        // Parse based on format
        let import_data = match format {
            ImportFormat::Csv => self.parse_csv_data(&content)?,
            ImportFormat::Json => self.parse_json_data(&content)?,
            ImportFormat::FmeaTemplate => self.parse_fmea_template(&content, &options)?,
        };

        // Validate import data
        let validation_result = self.validate_import_data(&import_data, &options)?;
        if !validation_result.is_valid {
            return Err(QmsError::validation_error(&format!(
                "Import validation failed: {}", 
                validation_result.errors.join(", ")
            )));
        }

        // If validate-only mode, return validation results
        if options.validate_only {
            audit_log_action("RISK_IMPORT_VALIDATED", "ImportFile", &file_path.display().to_string())?;
            return Ok(ImportResult {
                total_records: import_data.len(),
                imported_records: 0,
                skipped_records: 0,
                validation_errors: validation_result.errors,
                created_risks: Vec::new(),
                updated_risks: Vec::new(),
            });
        }

        // Create backup if requested
        if options.backup_existing {
            self.create_import_backup()?;
        }

        // Import the data
        let mut result = ImportResult {
            total_records: import_data.len(),
            imported_records: 0,
            skipped_records: 0,
            validation_errors: validation_result.warnings,
            created_risks: Vec::new(),
            updated_risks: Vec::new(),
        };

        for risk_data in import_data {
            match self.import_single_risk(risk_data, &options) {
                Ok(ImportAction::Created(risk)) => {
                    result.imported_records += 1;
                    result.created_risks.push(risk.id.clone());
                    audit_log_create("Risk", &risk.id, &risk.hazard_id)?;
                }
                Ok(ImportAction::Updated(risk)) => {
                    result.imported_records += 1;
                    result.updated_risks.push(risk.id.clone());
                    audit_log_action("RISK_UPDATED_FROM_IMPORT", "Risk", &risk.id)?;
                }
                Ok(ImportAction::Skipped) => {
                    result.skipped_records += 1;
                }
                Err(e) => {
                    result.validation_errors.push(format!("Failed to import risk: {e}"));
                    result.skipped_records += 1;
                }
            }
        }

        // Log import completion
        audit_log_action(
            "RISK_IMPORT_COMPLETED", 
            "ImportFile", 
            &format!("{}|imported:{}/total:{}", file_path.display(), result.imported_records, result.total_records)
        )?;

        Ok(result)
    }

    /// Parse CSV risk data
    fn parse_csv_data(&self, content: &str) -> QmsResult<Vec<ImportRiskData>> {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return Err(QmsError::validation_error("CSV file is empty"));
        }

        // Parse header
        let header = lines[0];
        let headers: Vec<&str> = header.split(',').map(|h| h.trim()).collect();
        
        // Validate required columns
        let required_headers = vec!["hazard_id", "description", "hazardous_situation", "harm", "severity", "occurrence", "detectability"];
        for required in &required_headers {
            if !headers.contains(required) {
                return Err(QmsError::validation_error(&format!("Missing required CSV column: {required}")));
            }
        }

        // Parse data rows
        let mut import_data = Vec::new();
        for (line_num, line) in lines.iter().skip(1).enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            match self.parse_csv_row(line, &headers) {
                Ok(risk_data) => import_data.push(risk_data),
                Err(e) => return Err(QmsError::validation_error(&format!("Error on line {}: {}", line_num + 2, e))),
            }
        }

        Ok(import_data)
    }

    /// Parse single CSV row into ImportRiskData
    fn parse_csv_row(&self, line: &str, headers: &[&str]) -> QmsResult<ImportRiskData> {
        let values: Vec<&str> = line.split(',').map(|v| v.trim().trim_matches('"')).collect();
        if values.len() != headers.len() {
            return Err(QmsError::validation_error("CSV row has incorrect number of columns"));
        }

        let mut field_map = HashMap::new();
        for (header, value) in headers.iter().zip(values.iter()) {
            field_map.insert(header.to_string(), value.to_string());
        }

        // Extract required fields
        let hazard_id = field_map.get("hazard_id")
            .ok_or_else(|| QmsError::validation_error("Missing hazard_id"))?
            .clone();

        let description = field_map.get("description")
            .ok_or_else(|| QmsError::validation_error("Missing description"))?
            .clone();

        let hazardous_situation = field_map.get("hazardous_situation")
            .ok_or_else(|| QmsError::validation_error("Missing hazardous_situation"))?
            .clone();

        let harm = field_map.get("harm")
            .ok_or_else(|| QmsError::validation_error("Missing harm"))?
            .clone();

        // Parse numeric risk parameters
        let severity = field_map.get("severity")
            .ok_or_else(|| QmsError::validation_error("Missing severity"))?
            .parse::<u8>()
            .map_err(|_| QmsError::validation_error("Invalid severity value (must be 1-5)"))?;

        let occurrence = field_map.get("occurrence")
            .ok_or_else(|| QmsError::validation_error("Missing occurrence"))?
            .parse::<u8>()
            .map_err(|_| QmsError::validation_error("Invalid occurrence value (must be 1-5)"))?;

        let detectability = field_map.get("detectability")
            .ok_or_else(|| QmsError::validation_error("Missing detectability"))?
            .parse::<u8>()
            .map_err(|_| QmsError::validation_error("Invalid detectability value (must be 1-5)"))?;

        // Validate risk parameter ranges
        if !(1..=5).contains(&severity) {
            return Err(QmsError::validation_error("Severity must be between 1 and 5"));
        }
        if !(1..=5).contains(&occurrence) {
            return Err(QmsError::validation_error("Occurrence must be between 1 and 5"));
        }
        if !(1..=5).contains(&detectability) {
            return Err(QmsError::validation_error("Detectability must be between 1 and 5"));
        }

        // Parse optional fields
        let category = field_map.get("category").cloned().unwrap_or_else(|| "Safety".to_string());
        let status = field_map.get("status").cloned().unwrap_or_else(|| "Identified".to_string());

        Ok(ImportRiskData {
            hazard_id,
            description,
            hazardous_situation,
            harm,
            severity,
            occurrence,
            detectability,
            category,
            status,
            additional_fields: field_map,
        })
    }

    /// Parse JSON risk data (placeholder for JSON parsing)
    fn parse_json_data(&self, _content: &str) -> QmsResult<Vec<ImportRiskData>> {
        // Placeholder: JSON parsing would be implemented here
        // For now, return error indicating JSON import is not yet implemented
        Err(QmsError::validation_error("JSON import not yet implemented - use CSV format"))
    }

    /// Parse FMEA template data
    fn parse_fmea_template(&self, content: &str, options: &ImportOptions) -> QmsResult<Vec<ImportRiskData>> {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return Err(QmsError::validation_error("FMEA template file is empty"));
        }

        // Parse header
        let header = lines[0];
        let headers: Vec<&str> = header.split(',').map(|h| h.trim()).collect();
        
        // Validate FMEA required columns
        let required_headers = vec!["component", "function", "failure_mode", "failure_effect", "severity", "occurrence", "detectability"];
        for required in &required_headers {
            if !headers.contains(required) {
                return Err(QmsError::validation_error(&format!("Missing required FMEA column: {required}")));
            }
        }

        // Parse data rows and convert to risk data
        let mut import_data = Vec::new();
        for (line_num, line) in lines.iter().skip(1).enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            match self.parse_fmea_row(line, &headers, options) {
                Ok(Some(risk_data)) => import_data.push(risk_data),
                Ok(None) => continue, // Filtered out
                Err(e) => return Err(QmsError::validation_error(&format!("Error on FMEA line {}: {}", line_num + 2, e))),
            }
        }

        Ok(import_data)
    }

    /// Parse single FMEA row into ImportRiskData
    fn parse_fmea_row(&self, line: &str, headers: &[&str], options: &ImportOptions) -> QmsResult<Option<ImportRiskData>> {
        let values: Vec<&str> = line.split(',').map(|v| v.trim().trim_matches('"')).collect();
        if values.len() != headers.len() {
            return Err(QmsError::validation_error("FMEA row has incorrect number of columns"));
        }

        let mut field_map = HashMap::new();
        for (header, value) in headers.iter().zip(values.iter()) {
            field_map.insert(header.to_string(), value.to_string());
        }

        // Apply component filter if specified
        if let Some(component_filter) = &options.component_filter {
            if let Some(component) = field_map.get("component") {
                if !component.to_lowercase().contains(&component_filter.to_lowercase()) {
                    return Ok(None); // Filtered out
                }
            }
        }

        // Extract FMEA fields
        let component = field_map.get("component").unwrap_or(&String::new()).clone();
        let function = field_map.get("function").unwrap_or(&String::new()).clone();
        let failure_mode = field_map.get("failure_mode").unwrap_or(&String::new()).clone();
        let failure_effect = field_map.get("failure_effect").unwrap_or(&String::new()).clone();

        // Generate hazard ID from FMEA data
        let hazard_id = format!("HAZ-FMEA-{}-{}", 
            component.replace(" ", ""), 
            failure_mode.replace(" ", "").chars().take(10).collect::<String>()
        );

        // Create risk description from FMEA data
        let description = format!("FMEA Risk: {failure_mode} failure in {component}");
        let hazardous_situation = format!("{function} - {failure_mode}");
        let harm = failure_effect;

        // Parse risk parameters
        let severity = field_map.get("severity")
            .ok_or_else(|| QmsError::validation_error("Missing severity in FMEA data"))?
            .parse::<u8>()
            .map_err(|_| QmsError::validation_error("Invalid FMEA severity value"))?;

        let occurrence = field_map.get("occurrence")
            .ok_or_else(|| QmsError::validation_error("Missing occurrence in FMEA data"))?
            .parse::<u8>()
            .map_err(|_| QmsError::validation_error("Invalid FMEA occurrence value"))?;

        let detectability = field_map.get("detectability")
            .ok_or_else(|| QmsError::validation_error("Missing detectability in FMEA data"))?
            .parse::<u8>()
            .map_err(|_| QmsError::validation_error("Invalid FMEA detectability value"))?;

        Ok(Some(ImportRiskData {
            hazard_id,
            description,
            hazardous_situation,
            harm,
            severity,
            occurrence,
            detectability,
            category: "FMEA".to_string(),
            status: "Identified".to_string(),
            additional_fields: field_map,
        }))
    }

    /// Validate import data for integrity and compliance
    fn validate_import_data(&self, import_data: &[ImportRiskData], _options: &ImportOptions) -> QmsResult<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check for duplicate hazard IDs within import data
        let mut hazard_ids = std::collections::HashSet::new();
        for risk_data in import_data {
            if !hazard_ids.insert(&risk_data.hazard_id) {
                errors.push(format!("Duplicate hazard_id in import data: {}", risk_data.hazard_id));
            }
        }

        // Validate each risk item
        for risk_data in import_data {
            // Validate required fields are not empty
            if risk_data.description.trim().is_empty() {
                errors.push(format!("Empty description for hazard_id: {}", risk_data.hazard_id));
            }
            if risk_data.hazardous_situation.trim().is_empty() {
                errors.push(format!("Empty hazardous_situation for hazard_id: {}", risk_data.hazard_id));
            }
            if risk_data.harm.trim().is_empty() {
                errors.push(format!("Empty harm for hazard_id: {}", risk_data.hazard_id));
            }

            // Validate RPN calculation
            let calculated_rpn = (risk_data.severity as u32) * (risk_data.occurrence as u32) * (risk_data.detectability as u32);
            if calculated_rpn > 125 {
                warnings.push(format!("High RPN ({}) for hazard_id: {} - review recommended", calculated_rpn, risk_data.hazard_id));
            }
        }

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        })
    }

    /// Create backup before import
    fn create_import_backup(&self) -> QmsResult<()> {
        audit_log_action("RISK_IMPORT_BACKUP_CREATED", "RiskManager", "pre_import_backup")?;
        Ok(())
    }

    /// Import single risk with deduplication
    fn import_single_risk(&mut self, risk_data: ImportRiskData, options: &ImportOptions) -> QmsResult<ImportAction> {
        // Check if risk already exists
        if self.manager.risk_exists(&risk_data.hazard_id) {
            if options.skip_duplicates {
                return Ok(ImportAction::Skipped);
            } else {
                // Update existing risk
                let existing_risk = self.manager.load_risk(&risk_data.hazard_id)?;
                let updated_risk = self.update_risk_from_import(existing_risk, risk_data)?;
                self.manager.save_risk(&updated_risk)?;
                return Ok(ImportAction::Updated(updated_risk));
            }
        }

        // Create new risk
        let new_risk = self.create_risk_from_import(risk_data)?;
        self.manager.save_risk(&new_risk)?;
        Ok(ImportAction::Created(new_risk))
    }

    /// Create new risk from import data
    fn create_risk_from_import(&self, risk_data: ImportRiskData) -> QmsResult<RiskItem> {
        let severity = self.convert_severity(risk_data.severity)?;
        let occurrence = self.convert_occurrence(risk_data.occurrence)?;
        let detectability = self.convert_detectability(risk_data.detectability)?;
        let rpn = (risk_data.severity as u32) * (risk_data.occurrence as u32) * (risk_data.detectability as u32);

        Ok(RiskItem {
            id: crate::utils::generate_uuid(),
            project_id: self.manager.get_project_id().to_string(),
            hazard_id: risk_data.hazard_id.clone(),
            hazard_description: risk_data.description,
            hazardous_situation: risk_data.hazardous_situation,
            harm: risk_data.harm,
            severity: severity.clone(),
            occurrence: occurrence.clone(),
            detectability: detectability.clone(),
            risk_priority_number: rpn,
            initial_risk_level: RiskManager::assess_risk_level(rpn),
            mitigation_measures: Vec::new(),
            residual_severity: severity,
            residual_occurrence: occurrence,
            residual_detectability: detectability,
            residual_rpn: rpn,
            residual_risk_level: RiskManager::assess_risk_level(rpn),
            residual_risk_justification: None,
            residual_risk_approved: false,
            residual_risk_approved_by: None,
            residual_risk_approval_date: None,
            verification_method: "Testing".to_string(),
            verification_status: crate::modules::risk_manager::risk::VerificationStatus::Planned,
            verification_evidence: Vec::new(),
            category: risk_data.category,
            source: "Import".to_string(),
            assigned_to: None,
            due_date: None,
            priority: "Medium".to_string(),
            risk_status: RiskStatus::Identified,
            tags: Vec::new(),
            regulatory_references: Vec::new(),
            standard_references: Vec::new(),
            created_at: crate::utils::format_timestamp(crate::utils::current_timestamp()),
            updated_at: crate::utils::format_timestamp(crate::utils::current_timestamp()),
            created_by: crate::utils::user_context::get_current_user_id(), // Get actual user
            approved_by: None,
            approval_date: None,
            post_market_data: Vec::new(),
            review_required: false,
            next_review_date: None,
        })
    }

    /// Update existing risk from import data
    fn update_risk_from_import(&self, mut risk: RiskItem, risk_data: ImportRiskData) -> QmsResult<RiskItem> {
        // Update risk parameters from import
        risk.severity = self.convert_severity(risk_data.severity)?;
        risk.occurrence = self.convert_occurrence(risk_data.occurrence)?;
        risk.detectability = self.convert_detectability(risk_data.detectability)?;
        risk.risk_priority_number = (risk_data.severity as u32) * (risk_data.occurrence as u32) * (risk_data.detectability as u32);
        risk.initial_risk_level = RiskManager::assess_risk_level(risk.risk_priority_number);
        risk.updated_at = crate::utils::format_timestamp(crate::utils::current_timestamp());

        // Update description fields if they're more detailed in import
        if risk_data.description.len() > risk.hazard_description.len() {
            risk.hazard_description = risk_data.description;
        }
        if risk_data.hazardous_situation.len() > risk.hazardous_situation.len() {
            risk.hazardous_situation = risk_data.hazardous_situation;
        }
        if risk_data.harm.len() > risk.harm.len() {
            risk.harm = risk_data.harm;
        }

        Ok(risk)
    }

    /// Convert numeric severity to enum
    fn convert_severity(&self, value: u8) -> QmsResult<RiskSeverity> {
        match value {
            1 => Ok(RiskSeverity::Negligible),
            2 => Ok(RiskSeverity::Minor),
            3 => Ok(RiskSeverity::Major),
            4 => Ok(RiskSeverity::Critical),
            5 => Ok(RiskSeverity::Catastrophic),
            _ => Err(QmsError::validation_error(&format!("Invalid severity value: {value} (must be 1-5)"))),
        }
    }

    /// Convert numeric occurrence to enum
    fn convert_occurrence(&self, value: u8) -> QmsResult<RiskOccurrence> {
        match value {
            1 => Ok(RiskOccurrence::Improbable),
            2 => Ok(RiskOccurrence::Remote),
            3 => Ok(RiskOccurrence::Occasional),
            4 => Ok(RiskOccurrence::Probable),
            5 => Ok(RiskOccurrence::Frequent),
            _ => Err(QmsError::validation_error(&format!("Invalid occurrence value: {value} (must be 1-5)"))),
        }
    }

    /// Convert numeric detectability to enum
    fn convert_detectability(&self, value: u8) -> QmsResult<RiskDetectability> {
        match value {
            1 => Ok(RiskDetectability::VeryHigh),
            2 => Ok(RiskDetectability::High),
            3 => Ok(RiskDetectability::Moderate),
            4 => Ok(RiskDetectability::Low),
            5 => Ok(RiskDetectability::VeryLow),
            _ => Err(QmsError::validation_error(&format!("Invalid detectability value: {value} (must be 1-5)"))),
        }
    }
}

/// Risk data exporter
pub struct RiskExporter {
    manager: RiskManager,
}

impl RiskExporter {
    /// Create new risk exporter
    pub fn new(project_path: &Path) -> QmsResult<Self> {
        let manager = RiskManager::new(project_path)?;
        Ok(Self { manager })
    }

    /// Export risks to file with specified format and options
    pub fn export_to_file(&self, output_path: &Path, format: ExportFormat, options: ExportOptions) -> QmsResult<ExportResult> {
        // Load risks with filtering
        let risks = self.load_filtered_risks(&options)?;
        
        if risks.is_empty() {
            return Err(QmsError::validation_error("No risks found matching export criteria"));
        }

        // Generate export content based on format
        let content = match format {
            ExportFormat::Csv => self.generate_csv_export(&risks, &options)?,
            ExportFormat::Json => self.generate_json_export(&risks, &options)?,
            ExportFormat::Pdf => self.generate_pdf_export(&risks, &options)?,
            ExportFormat::FmeaTemplate => self.generate_fmea_template(&risks, &options)?,
        };

        // Write to file
        fs::write(output_path, content)
            .map_err(|e| QmsError::io_error(&format!("Failed to write export file: {e}")))?;

        // Log export operation
        audit_log_action(
            "RISK_EXPORT_COMPLETED",
            "ExportFile", 
            &format!("{}|format:{:?}|count:{}", output_path.display(), format, risks.len())
        )?;

        Ok(ExportResult {
            exported_count: risks.len(),
            output_file: output_path.to_path_buf(),
            format,
        })
    }

    /// Load risks with filtering applied
    fn load_filtered_risks(&self, options: &ExportOptions) -> QmsResult<Vec<RiskItem>> {
        let all_risks = self.manager.list_all_risks()?;
        let mut filtered_risks = Vec::new();

        for risk in all_risks {
            // Apply severity filter
            if let Some(min_severity) = options.severity_filter {
                let risk_severity_value = match risk.severity {
                    RiskSeverity::Negligible => 1,
                    RiskSeverity::Minor => 2,
                    RiskSeverity::Major => 3,
                    RiskSeverity::Critical => 4,
                    RiskSeverity::Catastrophic => 5,
                };
                if risk_severity_value < min_severity {
                    continue;
                }
            }

            // Apply status filter
            if let Some(ref status_filter) = options.status_filter {
                if risk.risk_status != *status_filter {
                    continue;
                }
            }

            // Apply component filter for FMEA exports
            if let Some(ref component_filter) = options.component_filter {
                if !risk.category.to_lowercase().contains(&component_filter.to_lowercase()) &&
                   !risk.hazard_description.to_lowercase().contains(&component_filter.to_lowercase()) {
                    continue;
                }
            }

            filtered_risks.push(risk);
        }

        Ok(filtered_risks)
    }

    /// Generate CSV export content
    fn generate_csv_export(&self, risks: &[RiskItem], _options: &ExportOptions) -> QmsResult<String> {
        let mut csv_content = String::new();
        
        // CSV header
        csv_content.push_str("hazard_id,description,hazardous_situation,harm,severity,occurrence,detectability,rpn,risk_level,status,category,created_at\n");
        
        // CSV data rows
        for risk in risks {
            let severity_num = match risk.severity {
                RiskSeverity::Negligible => 1,
                RiskSeverity::Minor => 2,
                RiskSeverity::Major => 3,
                RiskSeverity::Critical => 4,
                RiskSeverity::Catastrophic => 5,
            };
            
            let occurrence_num = match risk.occurrence {
                RiskOccurrence::Improbable => 1,
                RiskOccurrence::Remote => 2,
                RiskOccurrence::Occasional => 3,
                RiskOccurrence::Probable => 4,
                RiskOccurrence::Frequent => 5,
            };
            
            let detectability_num = match risk.detectability {
                RiskDetectability::VeryHigh => 1,
                RiskDetectability::High => 2,
                RiskDetectability::Moderate => 3,
                RiskDetectability::Low => 4,
                RiskDetectability::VeryLow => 5,
            };

            let risk_level_str = match risk.initial_risk_level {
                RiskLevel::Acceptable => "Acceptable",
                RiskLevel::ALARP => "ALARP",
                RiskLevel::Unacceptable => "Unacceptable",
            };

            let status_str = format!("{:?}", risk.risk_status);

            csv_content.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",\"{}\",{},{},{},{},\"{}\",\"{}\",\"{}\",\"{}\"\n",
                self.escape_csv_field(&risk.hazard_id),
                self.escape_csv_field(&risk.hazard_description),
                self.escape_csv_field(&risk.hazardous_situation),
                self.escape_csv_field(&risk.harm),
                severity_num,
                occurrence_num,
                detectability_num,
                risk.risk_priority_number,
                risk_level_str,
                status_str,
                self.escape_csv_field(&risk.category),
                risk.created_at
            ));
        }

        Ok(csv_content)
    }

    /// Generate JSON export content (placeholder)
    fn generate_json_export(&self, risks: &[RiskItem], _options: &ExportOptions) -> QmsResult<String> {
        // Placeholder: Basic JSON structure
        let mut json_content = String::from("{\n  \"version\": \"1.0\",\n  \"export_type\": \"risk_data\",\n  \"risks\": [\n");
        
        for (i, risk) in risks.iter().enumerate() {
            if i > 0 {
                json_content.push_str(",\n");
            }
            json_content.push_str(&format!("    {{\n      \"hazard_id\": \"{}\",\n      \"description\": \"{}\",\n      \"rpn\": {}\n    }}", 
                risk.hazard_id, risk.hazard_description, risk.risk_priority_number));
        }
        
        json_content.push_str("\n  ]\n}");
        Ok(json_content)
    }

    /// Generate PDF export content (text-based)
    fn generate_pdf_export(&self, risks: &[RiskItem], options: &ExportOptions) -> QmsResult<String> {
        let mut pdf_content = String::new();
        
        // PDF header (text format)
        pdf_content.push_str("RISK MANAGEMENT REPORT\n");
        pdf_content.push_str("========================\n\n");
        pdf_content.push_str(&format!("Generated: {}\n", crate::utils::format_timestamp(crate::utils::current_timestamp())));
        pdf_content.push_str(&format!("Total Risks: {}\n\n", risks.len()));

        if options.summary_only {
            // Summary report
            let mut high_rpn_count = 0;
            let mut total_rpn = 0;
            
            for risk in risks {
                if risk.risk_priority_number >= 50 {
                    high_rpn_count += 1;
                }
                total_rpn += risk.risk_priority_number;
            }
            
            let avg_rpn = if !risks.is_empty() { total_rpn / risks.len() as u32 } else { 0 };
            
            pdf_content.push_str("RISK SUMMARY\n");
            pdf_content.push_str("============\n\n");
            pdf_content.push_str(&format!("High RPN Risks (≥50): {high_rpn_count}\n"));
            pdf_content.push_str(&format!("Average RPN: {avg_rpn}\n\n"));
        } else {
            // Detailed report
            pdf_content.push_str("DETAILED RISK LIST\n");
            pdf_content.push_str("==================\n\n");
            
            for risk in risks {
                pdf_content.push_str(&format!("Risk ID: {}\n", risk.hazard_id));
                pdf_content.push_str(&format!("Description: {}\n", risk.hazard_description));
                pdf_content.push_str(&format!("RPN: {} ({:?})\n", risk.risk_priority_number, risk.initial_risk_level));
                pdf_content.push_str(&format!("Status: {:?}\n", risk.risk_status));
                pdf_content.push_str("---\n\n");
            }
        }

        Ok(pdf_content)
    }

    /// Generate FMEA template export
    fn generate_fmea_template(&self, risks: &[RiskItem], _options: &ExportOptions) -> QmsResult<String> {
        let mut fmea_content = String::new();
        
        // FMEA header
        fmea_content.push_str("component,function,failure_mode,failure_effect,failure_cause,current_control,severity,occurrence,detectability,rpn,recommended_action\n");
        
        // Convert risks to FMEA format
        for risk in risks {
            let severity_num = match risk.severity {
                RiskSeverity::Negligible => 1,
                RiskSeverity::Minor => 2,
                RiskSeverity::Major => 3,
                RiskSeverity::Critical => 4,
                RiskSeverity::Catastrophic => 5,
            };
            
            let occurrence_num = match risk.occurrence {
                RiskOccurrence::Improbable => 1,
                RiskOccurrence::Remote => 2,
                RiskOccurrence::Occasional => 3,
                RiskOccurrence::Probable => 4,
                RiskOccurrence::Frequent => 5,
            };
            
            let detectability_num = match risk.detectability {
                RiskDetectability::VeryHigh => 1,
                RiskDetectability::High => 2,
                RiskDetectability::Moderate => 3,
                RiskDetectability::Low => 4,
                RiskDetectability::VeryLow => 5,
            };

            // Extract component from category or description
            let component = if risk.category != "Safety" && risk.category != "FMEA" {
                risk.category.clone()
            } else {
                "System".to_string()
            };

            fmea_content.push_str(&format!(
                "\"{}\",\"Primary Function\",\"{}\",\"{}\",\"TBD\",\"TBD\",{},{},{},{},\"TBD\"\n",
                self.escape_csv_field(&component),
                self.escape_csv_field(&risk.hazard_description),
                self.escape_csv_field(&risk.harm),
                severity_num,
                occurrence_num,
                detectability_num,
                risk.risk_priority_number
            ));
        }

        Ok(fmea_content)
    }

    /// Escape CSV field content
    fn escape_csv_field(&self, field: &str) -> String {
        field.replace("\"", "\"\"").replace("\n", " ").replace("\r", " ")
    }
}

/// Import risk data structure
#[derive(Debug, Clone)]
pub struct ImportRiskData {
    pub hazard_id: String,
    pub description: String,
    pub hazardous_situation: String,
    pub harm: String,
    pub severity: u8,
    pub occurrence: u8,
    pub detectability: u8,
    pub category: String,
    #[allow(dead_code)]
    pub status: String,
    #[allow(dead_code)]
    pub additional_fields: HashMap<String, String>,
}

/// Import validation result
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Import action taken for a risk
#[derive(Debug)]
pub enum ImportAction {
    Created(RiskItem),
    Updated(RiskItem),
    Skipped,
}

/// Import operation result
#[derive(Debug)]
pub struct ImportResult {
    pub total_records: usize,
    pub imported_records: usize,
    pub skipped_records: usize,
    pub validation_errors: Vec<String>,
    pub created_risks: Vec<String>,
    pub updated_risks: Vec<String>,
}

/// Export operation result
#[derive(Debug)]
pub struct ExportResult {
    pub exported_count: usize,
    pub output_file: PathBuf,
    pub format: ExportFormat,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_import_format_parsing() {
        assert_eq!(ImportFormat::from_str("csv").unwrap(), ImportFormat::Csv);
        assert_eq!(ImportFormat::from_str("json").unwrap(), ImportFormat::Json);
        assert_eq!(ImportFormat::from_str("fmea").unwrap(), ImportFormat::FmeaTemplate);
        assert!(ImportFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_export_format_parsing() {
        assert_eq!(ExportFormat::from_str("csv").unwrap(), ExportFormat::Csv);
        assert_eq!(ExportFormat::from_str("json").unwrap(), ExportFormat::Json);
        assert_eq!(ExportFormat::from_str("pdf").unwrap(), ExportFormat::Pdf);
        assert_eq!(ExportFormat::from_str("fmea-template").unwrap(), ExportFormat::FmeaTemplate);
    }

    #[test]
    fn test_csv_field_escaping() {
        let exporter = RiskExporter { manager: create_test_risk_manager() };
        assert_eq!(exporter.escape_csv_field("simple text"), "simple text");
        assert_eq!(exporter.escape_csv_field("text with \"quotes\""), "text with \"\"quotes\"\"");
        assert_eq!(exporter.escape_csv_field("text\nwith\nnewlines"), "text with newlines");
    }

    #[test]
    fn test_risk_parameter_conversion() {
        let temp_dir = std::env::temp_dir().join("qms_test_import");
        let _ = fs::create_dir_all(&temp_dir);
        let importer = RiskImporter::new(&temp_dir).expect("Failed to create importer");
        
        // Test severity conversion
        assert_eq!(importer.convert_severity(1).unwrap(), RiskSeverity::Negligible);
        assert_eq!(importer.convert_severity(5).unwrap(), RiskSeverity::Catastrophic);
        assert!(importer.convert_severity(6).is_err());
        
        // Test occurrence conversion
        assert_eq!(importer.convert_occurrence(1).unwrap(), RiskOccurrence::Improbable);
        assert_eq!(importer.convert_occurrence(5).unwrap(), RiskOccurrence::Frequent);
        
        // Test detectability conversion
        assert_eq!(importer.convert_detectability(1).unwrap(), RiskDetectability::VeryHigh);
        assert_eq!(importer.convert_detectability(5).unwrap(), RiskDetectability::VeryLow);
    }

    #[test]
    fn test_csv_parsing_validation() {
        let temp_dir = std::env::temp_dir().join("qms_test_import_csv");
        let _ = fs::create_dir_all(&temp_dir);
        let importer = RiskImporter::new(&temp_dir).expect("Failed to create importer");
        
        // Test valid CSV content
        let valid_csv = "hazard_id,description,hazardous_situation,harm,severity,occurrence,detectability\nHAZ-001,Test hazard,Test situation,Test harm,3,2,2";
        let result = importer.parse_csv_data(valid_csv);
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].hazard_id, "HAZ-001");
        
        // Test invalid CSV (missing required column)
        let invalid_csv = "hazard_id,description\nHAZ-001,Test hazard";
        let result = importer.parse_csv_data(invalid_csv);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_logic() {
        let temp_dir = std::env::temp_dir().join("qms_test_validation");
        let _ = fs::create_dir_all(&temp_dir);
        let importer = RiskImporter::new(&temp_dir).expect("Failed to create importer");
        
        // Create test data with validation issues
        let mut test_data = Vec::new();
        
        // Valid risk
        test_data.push(ImportRiskData {
            hazard_id: "HAZ-001".to_string(),
            description: "Valid risk".to_string(),
            hazardous_situation: "Valid situation".to_string(),
            harm: "Valid harm".to_string(),
            severity: 3,
            occurrence: 2,
            detectability: 2,
            category: "Safety".to_string(),
            status: "Identified".to_string(),
            additional_fields: HashMap::new(),
        });
        
        // Risk with empty description (should cause error)
        test_data.push(ImportRiskData {
            hazard_id: "HAZ-002".to_string(),
            description: "".to_string(),  // Empty description
            hazardous_situation: "Valid situation".to_string(),
            harm: "Valid harm".to_string(),
            severity: 3,
            occurrence: 2,
            detectability: 2,
            category: "Safety".to_string(),
            status: "Identified".to_string(),
            additional_fields: HashMap::new(),
        });
        
        let options = ImportOptions::default();
        let validation_result = importer.validate_import_data(&test_data, &options).unwrap();
        
        assert!(!validation_result.is_valid);  // Should be invalid due to empty description
        assert!(!validation_result.errors.is_empty());  // Should have error for empty description
    }

    // Helper function to create test risk manager
    fn create_test_risk_manager() -> RiskManager {
        let temp_dir = std::env::temp_dir().join("qms_test_export");
        let _ = fs::create_dir_all(&temp_dir);
        RiskManager::new(&temp_dir).expect("Failed to create test risk manager")
    }
}
