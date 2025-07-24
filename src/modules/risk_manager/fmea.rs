//! FMEA (Failure Mode & Effects Analysis) Module
//! Task 3.1.3: FMEA Implementation
//! 
//! This module implements comprehensive FMEA functionality following ISO 14971
//! requirements for medical device risk management.

#![allow(dead_code)] // Allow unused fields and variants for comprehensive data model

use crate::prelude::*;
use crate::modules::risk_manager::{RiskSeverity, RiskOccurrence, RiskDetectability};
use crate::utils::RiskCalculator; // REFACTORED: Use centralized risk calculator

/// FMEA Analysis entity containing all analysis data
#[derive(Debug, Clone)]
pub struct FMEAAnalysis {
    pub id: String,              // UUID v4
    pub project_id: String,      // Foreign key to Project
    pub name: String,            // FMEA analysis name
    pub component: String,       // System/component being analyzed
    pub function: String,        // Primary function of component
    pub description: String,     // Analysis description
    pub created_at: String,      // ISO 8601 timestamp
    pub updated_at: String,      // ISO 8601 timestamp
    pub created_by: String,      // User ID
    pub status: FMEAStatus,      // Analysis status
    pub failure_modes: Vec<FailureMode>, // List of failure modes
    pub scope: String,           // Analysis scope
    pub assumptions: Vec<String>, // Analysis assumptions
    pub team_members: Vec<String>, // Analysis team
    pub version: String,         // Analysis version
}

/// Individual failure mode within an FMEA analysis
#[derive(Debug, Clone)]
pub struct FailureMode {
    pub id: String,              // UUID v4
    pub fmea_id: String,         // Foreign key to FMEA
    pub mode_id: String,         // FM-001, FM-002, etc.
    pub description: String,     // Failure mode description
    pub function: String,        // Function that fails
    pub effects: Vec<FailureEffect>, // Effects of failure
    pub causes: Vec<FailureCause>,   // Potential causes
    pub current_controls: Vec<CurrentControl>, // Existing controls
    pub severity: RiskSeverity,  // Effect severity (1-5)
    pub occurrence: RiskOccurrence, // Occurrence likelihood (1-5)
    pub detectability: RiskDetectability, // Detection capability (1-5)
    pub rpn: u32,               // Risk Priority Number (calculated)
    pub criticality: u32,       // Criticality number (Severity × Occurrence)
    pub recommended_actions: Vec<RecommendedAction>, // Improvement actions
    pub responsibility: String,  // Assigned team member
    pub target_date: Option<String>, // Target completion date
    pub actions_taken: Vec<String>, // Completed actions
    pub residual_severity: RiskSeverity,     // Post-action severity
    pub residual_occurrence: RiskOccurrence, // Post-action occurrence
    pub residual_detectability: RiskDetectability, // Post-action detectability
    pub residual_rpn: u32,      // Residual RPN after actions
    pub status: FailureModeStatus, // Status of failure mode
    pub created_at: String,     // Creation timestamp
    pub updated_at: String,     // Last update timestamp
}

/// Effect of a failure mode
#[derive(Debug, Clone)]
pub struct FailureEffect {
    pub id: String,             // UUID v4
    pub description: String,    // Effect description
    pub impact_level: EffectImpactLevel, // Local, subsystem, system, end user
    pub detection_method: String, // How effect is detected
    pub consequences: Vec<String>, // Specific consequences
}

/// Potential cause of a failure mode
#[derive(Debug, Clone)]
pub struct FailureCause {
    pub id: String,             // UUID v4
    pub description: String,    // Cause description
    pub category: CauseCategory, // Design, manufacturing, usage, environmental
    pub probability: f32,       // Probability factor (0.0-1.0)
    pub mechanism: String,      // Failure mechanism
}

/// Current control measure
#[derive(Debug, Clone)]
pub struct CurrentControl {
    pub id: String,             // UUID v4
    pub description: String,    // Control description
    pub control_type: ControlType, // Prevention, detection, mitigation
    pub effectiveness: f32,     // Effectiveness rating (0.0-1.0)
    pub verification_method: String, // How control is verified
}

/// Recommended action for risk mitigation
#[derive(Debug, Clone)]
pub struct RecommendedAction {
    pub id: String,             // UUID v4
    pub description: String,    // Action description
    pub action_type: ActionType, // Design change, process improvement, etc.
    pub priority: ActionPriority, // High, medium, low
    pub estimated_effort: String, // Effort estimate
    pub cost_estimate: Option<f32>, // Cost estimate
    pub target_reduction: RPNReduction, // Expected RPN reduction
}

/// Expected RPN reduction from an action
#[derive(Debug, Clone)]
pub struct RPNReduction {
    pub severity_reduction: u8,     // Expected severity reduction
    pub occurrence_reduction: u8,   // Expected occurrence reduction
    pub detectability_reduction: u8, // Expected detectability reduction
    pub expected_rpn: u32,          // Expected final RPN
}

/// FMEA analysis status
#[derive(Debug, Clone)]
pub enum FMEAStatus {
    Draft,          // Being created
    InProgress,     // Under analysis
    ReviewPending,  // Awaiting review
    Approved,       // Completed and approved
    Archived,       // Historical record
}

/// Failure mode status
#[derive(Debug, Clone)]
pub enum FailureModeStatus {
    Identified,     // Failure mode identified
    Analyzed,       // Analysis completed
    ActionRequired, // Actions needed
    ActionInProgress, // Actions being implemented
    ActionCompleted, // Actions completed
    Verified,       // Effectiveness verified
    Closed,         // No further action needed
}

/// Effect impact level
#[derive(Debug, Clone)]
pub enum EffectImpactLevel {
    Local,          // Component level only
    Subsystem,      // Affects subsystem
    System,         // Affects entire system
    EndUser,        // Affects end user/patient
}

/// Cause category
#[derive(Debug, Clone)]
pub enum CauseCategory {
    Design,         // Design deficiency
    Manufacturing,  // Manufacturing issue
    Usage,          // User error/misuse
    Environmental,  // Environmental factor
    Maintenance,    // Maintenance related
    Software,       // Software defect
    Hardware,       // Hardware failure
}

/// Control type
#[derive(Debug, Clone)]
pub enum ControlType {
    Prevention,     // Prevents failure occurrence
    Detection,      // Detects failure when it occurs
    Mitigation,     // Reduces failure effects
}

/// Action type
#[derive(Debug, Clone)]
pub enum ActionType {
    DesignChange,   // Design modification
    ProcessImprovement, // Process enhancement
    TestingEnhancement, // Improved testing
    Documentation,  // Documentation update
    Training,       // User/operator training
    Monitoring,     // Enhanced monitoring
}

/// Action priority
#[derive(Debug, Clone)]
pub enum ActionPriority {
    Critical,       // Immediate action required
    High,           // High priority
    Medium,         // Medium priority
    Low,            // Low priority
}

/// FMEA Manager for handling FMEA operations
pub struct FMEAManager {
    project_path: PathBuf,
}

impl FMEAManager {
    /// Create new FMEA manager for a project
    pub fn new(project_path: &Path) -> QmsResult<Self> {
        Ok(Self {
            project_path: project_path.to_path_buf(),
        })
    }
    
    /// Initialize FMEA system for project
    pub fn initialize(&self) -> QmsResult<()> {
        let fmea_dir = self.project_path.join("fmea");
        std::fs::create_dir_all(&fmea_dir)?;
        
        // Create subdirectories
        std::fs::create_dir_all(fmea_dir.join("analyses"))?;
        std::fs::create_dir_all(fmea_dir.join("reports"))?;
        std::fs::create_dir_all(fmea_dir.join("templates"))?;
        std::fs::create_dir_all(fmea_dir.join("exports"))?;
        
        // Create FMEA index file
        let index_path = fmea_dir.join("index.json");
        if !index_path.exists() {
            let index_data = r#"{"version": "1.0", "data": []}"#;
            std::fs::write(&index_path, index_data)?;
        }
        
        Ok(())
    }
    
    /// Create new FMEA analysis
    pub fn create_fmea_analysis(&self, component: &str, function: &str, name: &str) -> QmsResult<FMEAAnalysis> {
        let id = crate::utils::generate_uuid();
        let current_time = crate::utils::current_timestamp();
        
        let analysis = FMEAAnalysis {
            id: id.clone(),
            project_id: crate::utils::user_context::get_current_project_id(), // Get actual project ID
            name: name.to_string(),
            component: component.to_string(),
            function: function.to_string(),
            description: format!("FMEA analysis for {component} - {function}"),
            created_at: current_time.to_string(),
            updated_at: current_time.to_string(),
            created_by: crate::utils::user_context::get_current_user_id(), // Get actual user
            status: FMEAStatus::Draft,
            failure_modes: Vec::new(),
            scope: format!("Analysis of {component} component focusing on {function} function"),
            assumptions: Vec::new(),
            team_members: Vec::new(),
            version: "1.0".to_string(),
        };
        
        // Save to file
        self.save_fmea_analysis(&analysis)?;
        
        // Log audit entry
        crate::audit::log_audit(&format!("FMEA_CREATED: {id}: Created FMEA analysis for component: {component}, function: {function}"));
        
        Ok(analysis)
    }
    
    /// Add failure mode to FMEA analysis
    pub fn add_failure_mode(&self, fmea_id: &str, description: &str, function: &str) -> QmsResult<FailureMode> {
        let mut analysis = self.load_fmea_analysis(fmea_id)?;
        
        // Generate failure mode ID
        let mode_id = format!("FM-{:03}", analysis.failure_modes.len() + 1);
        let id = crate::utils::generate_uuid();
        let current_time = crate::utils::current_timestamp();
        
        let failure_mode = FailureMode {
            id: id.clone(),
            fmea_id: fmea_id.to_string(),
            mode_id: mode_id.clone(),
            description: description.to_string(),
            function: function.to_string(),
            effects: Vec::new(),
            causes: Vec::new(),
            current_controls: Vec::new(),
            severity: RiskSeverity::Minor,        // Default values
            occurrence: RiskOccurrence::Remote,    // Will be assessed later
            detectability: RiskDetectability::Moderate,
            rpn: 0,                               // Will be calculated
            criticality: 0,                       // Will be calculated
            recommended_actions: Vec::new(),
            responsibility: String::new(),
            target_date: None,
            actions_taken: Vec::new(),
            residual_severity: RiskSeverity::Minor,
            residual_occurrence: RiskOccurrence::Remote,
            residual_detectability: RiskDetectability::Moderate,
            residual_rpn: 0,
            status: FailureModeStatus::Identified,
            created_at: current_time.to_string(),
            updated_at: current_time.to_string(),
        };
        
        // Calculate initial RPN
        let mut updated_failure_mode = failure_mode;
        updated_failure_mode.rpn = self.calculate_rpn(
            &updated_failure_mode.severity,
            &updated_failure_mode.occurrence,
            &updated_failure_mode.detectability
        );
        updated_failure_mode.criticality = self.calculate_criticality(
            &updated_failure_mode.severity,
            &updated_failure_mode.occurrence
        );
        
        // Add to analysis
        analysis.failure_modes.push(updated_failure_mode.clone());
        analysis.updated_at = crate::utils::current_timestamp().to_string();
        
        // Save updated analysis
        self.save_fmea_analysis(&analysis)?;
        
        // Log audit entry
        crate::audit::log_audit(&format!("FAILURE_MODE_ADDED: {id}: Added failure mode {mode_id} to FMEA {fmea_id}"));
        
        Ok(updated_failure_mode)
    }
    
    /// Calculate Risk Priority Number (RPN)
    /// REFACTORED: Delegates to centralized RiskCalculator to eliminate DRY violation
    pub fn calculate_rpn(&self, severity: &RiskSeverity, occurrence: &RiskOccurrence,
                        detectability: &RiskDetectability) -> u32 {
        RiskCalculator::calculate_rpn(severity, occurrence, detectability)
    }
    
    /// Calculate Criticality Number (Severity × Occurrence)
    pub fn calculate_criticality(&self, severity: &RiskSeverity, occurrence: &RiskOccurrence) -> u32 {
        (severity.clone() as u32) * (occurrence.clone() as u32)
    }
    
    /// Load FMEA analysis from file
    pub fn load_fmea_analysis(&self, fmea_id: &str) -> QmsResult<FMEAAnalysis> {
        let file_path = self.project_path.join("fmea").join("analyses").join(format!("{fmea_id}.json"));
        
        if !file_path.exists() {
            return Err(QmsError::not_found(&format!("FMEA analysis not found: {fmea_id}")));
        }
        
        let content = std::fs::read_to_string(&file_path)?;
        let mut analysis = self.parse_fmea_json(&content)?;
        
        // TODO: Load failure modes from separate files in future enhancement
        // For now, initialize empty failure modes list
        analysis.failure_modes = Vec::new();
        
        Ok(analysis)
    }
    
    /// Save FMEA analysis to file
    pub fn save_fmea_analysis(&self, analysis: &FMEAAnalysis) -> QmsResult<()> {
        let analyses_dir = self.project_path.join("fmea").join("analyses");
        std::fs::create_dir_all(&analyses_dir)?;
        
        let file_path = analyses_dir.join(format!("{}.json", analysis.id));
        let json_content = self.serialize_fmea_to_json(analysis)?;
        
        // Use direct write for now to avoid Windows temp file issues
        std::fs::write(&file_path, &json_content)
            .map_err(|e| QmsError::io_error(&format!("Failed to write FMEA file: {e}")))?;
        
        // Update index
        self.update_fmea_index(analysis)?;
        
        Ok(())
    }
    
    /// List all FMEA analyses
    pub fn list_fmea_analyses(&self) -> QmsResult<Vec<FMEAAnalysis>> {
        let index_path = self.project_path.join("fmea").join("index.json");
        
        if !index_path.exists() {
            return Ok(Vec::new());
        }
        
        let content = std::fs::read_to_string(&index_path)?;
        let fmea_ids = self.parse_fmea_index(&content)?;
        
        let mut analyses = Vec::new();
        for fmea_id in fmea_ids {
            match self.load_fmea_analysis(&fmea_id) {
                Ok(analysis) => analyses.push(analysis),
                Err(_) => continue, // Skip missing analyses
            }
        }
        
        Ok(analyses)
    }
    
    /// Generate FMEA table in CSV format
    pub fn generate_fmea_table(&self, fmea_id: &str) -> QmsResult<String> {
        let analysis = self.load_fmea_analysis(fmea_id)?;
        
        let mut csv = String::new();
        csv.push_str("Component,Function,Failure Mode,Effects,Causes,Current Controls,");
        csv.push_str("Severity,Occurrence,Detectability,RPN,Criticality,");
        csv.push_str("Recommended Actions,Responsibility,Target Date,Status\n");
        
        for fm in &analysis.failure_modes {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:?}\n",
                escape_csv(&analysis.component),
                escape_csv(&fm.function),
                escape_csv(&fm.description),
                escape_csv(&self.format_effects(&fm.effects)),
                escape_csv(&self.format_causes(&fm.causes)),
                escape_csv(&self.format_controls(&fm.current_controls)),
                fm.severity.clone() as u8,
                fm.occurrence.clone() as u8,
                fm.detectability.clone() as u8,
                fm.rpn,
                fm.criticality,
                escape_csv(&self.format_actions(&fm.recommended_actions)),
                escape_csv(&fm.responsibility),
                escape_csv(fm.target_date.as_deref().unwrap_or("")),
                fm.status
            ));
        }
        
        Ok(csv)
    }
    
    /// Export FMEA table to file
    pub fn export_fmea_table(&self, fmea_id: &str, format: &str, output_path: &Path) -> QmsResult<()> {
        match format.to_lowercase().as_str() {
            "csv" => {
                let csv_content = self.generate_fmea_table(fmea_id)?;
                std::fs::write(output_path, csv_content)?;
            }
            "json" => {
                let analysis = self.load_fmea_analysis(fmea_id)?;
                let json_content = self.serialize_fmea_to_json(&analysis)?;
                std::fs::write(output_path, json_content)?;
            }
            _ => {
                return Err(QmsError::validation_error(&format!("Unsupported export format: {format}")));
            }
        }
        
        Ok(())
    }
    
    // Helper methods for serialization
    fn serialize_fmea_to_json(&self, analysis: &FMEAAnalysis) -> QmsResult<String> {
        // Basic JSON serialization - will be enhanced in future tasks
        let mut json = String::new();
        json.push_str("{\n");
        json.push_str("  \"version\": \"1.0\",\n");
        json.push_str(&format!("  \"id\": \"{}\",\n", escape_json(&analysis.id)));
        json.push_str(&format!("  \"project_id\": \"{}\",\n", escape_json(&analysis.project_id)));
        json.push_str(&format!("  \"name\": \"{}\",\n", escape_json(&analysis.name)));
        json.push_str(&format!("  \"component\": \"{}\",\n", escape_json(&analysis.component)));
        json.push_str(&format!("  \"function\": \"{}\",\n", escape_json(&analysis.function)));
        json.push_str(&format!("  \"description\": \"{}\",\n", escape_json(&analysis.description)));
        json.push_str(&format!("  \"created_at\": \"{}\",\n", escape_json(&analysis.created_at)));
        json.push_str(&format!("  \"updated_at\": \"{}\",\n", escape_json(&analysis.updated_at)));
        json.push_str(&format!("  \"created_by\": \"{}\",\n", escape_json(&analysis.created_by)));
        json.push_str(&format!("  \"status\": \"{:?}\",\n", analysis.status));
        json.push_str(&format!("  \"scope\": \"{}\",\n", escape_json(&analysis.scope)));
        json.push_str(&format!("  \"version\": \"{}\",\n", escape_json(&analysis.version)));
        json.push_str(&format!("  \"failure_mode_count\": {}\n", analysis.failure_modes.len()));
        json.push('}');
        Ok(json)
    }
    
    fn parse_fmea_json(&self, content: &str) -> QmsResult<FMEAAnalysis> {
        // Manual JSON parsing for FMEA analysis
        let lines: Vec<&str> = content.lines().collect();
        
        let mut id = String::new();
        let mut project_id = String::new();
        let mut name = String::new();
        let mut component = String::new();
        let mut function = String::new();
        let mut description = String::new();
        let mut created_at = String::new();
        let mut updated_at = String::new();
        let mut created_by = String::new();
        let mut status = FMEAStatus::Draft;
        let mut scope = String::new();
        let mut version = String::new();
        
        for line in lines {
            let trimmed = line.trim();
            if trimmed.contains("\"id\":") {
                id = extract_json_string_value(trimmed)?;
            } else if trimmed.contains("\"project_id\":") {
                project_id = extract_json_string_value(trimmed)?;
            } else if trimmed.contains("\"name\":") {
                name = extract_json_string_value(trimmed)?;
            } else if trimmed.contains("\"component\":") {
                component = extract_json_string_value(trimmed)?;
            } else if trimmed.contains("\"function\":") {
                function = extract_json_string_value(trimmed)?;
            } else if trimmed.contains("\"description\":") {
                description = extract_json_string_value(trimmed)?;
            } else if trimmed.contains("\"created_at\":") {
                created_at = extract_json_string_value(trimmed)?;
            } else if trimmed.contains("\"updated_at\":") {
                updated_at = extract_json_string_value(trimmed)?;
            } else if trimmed.contains("\"created_by\":") {
                created_by = extract_json_string_value(trimmed)?;
            } else if trimmed.contains("\"status\":") {
                let status_str = extract_json_string_value(trimmed)?;
                status = match status_str.as_str() {
                    "Draft" => FMEAStatus::Draft,
                    "InProgress" => FMEAStatus::InProgress,
                    "ReviewPending" => FMEAStatus::ReviewPending,
                    "Approved" => FMEAStatus::Approved,
                    "Archived" => FMEAStatus::Archived,
                    _ => FMEAStatus::Draft,
                };
            } else if trimmed.contains("\"scope\":") {
                scope = extract_json_string_value(trimmed)?;
            } else if trimmed.contains("\"version\":") && !trimmed.contains("failure_mode_count") {
                version = extract_json_string_value(trimmed)?;
            }
        }
        
        if id.is_empty() {
            return Err(QmsError::validation_error("Missing required field: id"));
        }
        
        Ok(FMEAAnalysis {
            id,
            project_id,
            name,
            component,
            function,
            description,
            created_at,
            updated_at,
            created_by,
            status,
            failure_modes: Vec::new(), // Failure modes loaded separately
            scope,
            assumptions: Vec::new(),
            team_members: Vec::new(),
            version,
        })
    }
    
    fn parse_fmea_index(&self, content: &str) -> QmsResult<Vec<String>> {
        // Parse FMEA index file to extract analysis IDs
        let lines: Vec<&str> = content.lines().collect();
        let mut ids = Vec::new();
        let mut in_data_array = false;
        
        for line in lines {
            let trimmed = line.trim();
            if trimmed.contains("\"data\":") {
                in_data_array = true;
                continue;
            }
            
            if in_data_array && trimmed.starts_with('"') && trimmed.ends_with('"') {
                let id = trimmed.trim_matches('"').trim_matches(',');
                if !id.is_empty() {
                    ids.push(id.to_string());
                }
            }
            
            if trimmed == "]" {
                in_data_array = false;
            }
        }
        
        Ok(ids)
    }
    
    fn update_fmea_index(&self, analysis: &FMEAAnalysis) -> QmsResult<()> {
        let index_path = self.project_path.join("fmea").join("index.json");
        
        // Read existing index
        let mut existing_ids = Vec::new();
        if index_path.exists() {
            let content = std::fs::read_to_string(&index_path)?;
            existing_ids = self.parse_fmea_index(&content)?;
        }
        
        // Add new analysis ID if not already present
        if !existing_ids.contains(&analysis.id) {
            existing_ids.push(analysis.id.clone());
        }
        
        // Write updated index
        let mut index_json = String::new();
        index_json.push_str("{\n");
        index_json.push_str("  \"version\": \"1.0\",\n");
        index_json.push_str("  \"data\": [\n");
        
        for (i, id) in existing_ids.iter().enumerate() {
            if i > 0 {
                index_json.push_str(",\n");
            }
            index_json.push_str(&format!("    \"{id}\""));
        }
        
        index_json.push_str("\n  ]\n");
        index_json.push('}');
        
        // Use direct write for now to avoid Windows temp file issues
        std::fs::write(&index_path, &index_json)
            .map_err(|e| QmsError::io_error(&format!("Failed to write FMEA index: {e}")))?;
        Ok(())
    }
    
    // Helper formatting methods
    fn format_effects(&self, effects: &[FailureEffect]) -> String {
        effects.iter().map(|e| e.description.clone()).collect::<Vec<_>>().join("; ")
    }
    
    fn format_causes(&self, causes: &[FailureCause]) -> String {
        causes.iter().map(|c| c.description.clone()).collect::<Vec<_>>().join("; ")
    }
    
    fn format_controls(&self, controls: &[CurrentControl]) -> String {
        controls.iter().map(|c| c.description.clone()).collect::<Vec<_>>().join("; ")
    }
    
    fn format_actions(&self, actions: &[RecommendedAction]) -> String {
        actions.iter().map(|a| a.description.clone()).collect::<Vec<_>>().join("; ")
    }
}

// Helper functions for CSV and JSON escaping
fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\")
         .replace('"', "\\\"")
         .replace('\n', "\\n")
         .replace('\r', "\\r")
         .replace('\t', "\\t")
}

// Helper function to extract string value from JSON line
fn extract_json_string_value(line: &str) -> QmsResult<String> {
    // Find the part after the colon
    if let Some(colon_pos) = line.find(':') {
        let value_part = line[colon_pos + 1..].trim();
        
        // Handle quoted strings - find the actual content between quotes
        if let Some(first_quote) = value_part.find('"') {
            let after_first_quote = &value_part[first_quote + 1..];
            if let Some(last_quote) = after_first_quote.find('"') {
                let content = &after_first_quote[..last_quote];
                return Ok(content.to_string());
            }
        }
        
        // Fallback - just clean up the value
        let value = value_part.trim_matches(|c| c == '"' || c == ',' || c == ' ');
        Ok(value.to_string())
    } else {
        Err(QmsError::validation_error("Invalid JSON line format"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    
    #[test]
    fn test_fmea_manager_creation() {
        let temp_path = Path::new("/tmp/test_fmea");
        let manager = FMEAManager::new(temp_path);
        assert!(manager.is_ok());
    }
    
    #[test]
    fn test_rpn_calculation() {
        let manager = FMEAManager::new(Path::new("/tmp")).unwrap();
        let rpn = manager.calculate_rpn(&RiskSeverity::Critical, &RiskOccurrence::Probable, &RiskDetectability::Low);
        assert_eq!(rpn, 4 * 4 * 4); // 64
    }
    
    #[test]
    fn test_criticality_calculation() {
        let manager = FMEAManager::new(Path::new("/tmp")).unwrap();
        let criticality = manager.calculate_criticality(&RiskSeverity::Major, &RiskOccurrence::Occasional);
        assert_eq!(criticality, 3 * 3); // 9
    }
    
    #[test]
    fn test_csv_escaping() {
        assert_eq!(escape_csv("simple"), "simple");
        assert_eq!(escape_csv("with,comma"), "\"with,comma\"");
        assert_eq!(escape_csv("with\"quote"), "\"with\"\"quote\"");
    }
    
    #[test]
    fn test_json_escaping() {
        assert_eq!(escape_json("simple"), "simple");
        assert_eq!(escape_json("with\"quote"), "with\\\"quote");
        assert_eq!(escape_json("with\newline"), "with\\newline");
    }
}
