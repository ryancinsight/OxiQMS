//! Report Generator Interface Segregation
//! 
//! This module implements Interface Segregation Principle (ISP) for report generation,
//! separating concerns into focused, cohesive interfaces following SOLID principles.
//! 
//! SOLID Principles Applied:
//! - Interface Segregation: Separate interfaces for different report generation concerns
//! - Single Responsibility: Each interface handles one specific aspect of reporting
//! - Dependency Inversion: High-level modules depend on abstractions
//! - Open/Closed: New implementations can be added without modification

use crate::prelude::*;

/// Interface Segregation Principle: Focused interface for data collection
pub trait DataCollector<T> {
    /// Collect raw data for report generation
    fn collect_data(&self) -> QmsResult<Vec<T>>;
    
    /// Filter collected data based on criteria
    fn filter_data(&self, data: &[T], criteria: &DataFilterCriteria) -> QmsResult<Vec<T>>;
    
    /// Get data collection statistics
    fn get_collection_stats(&self) -> QmsResult<DataCollectionStats>;
}

/// Interface Segregation Principle: Focused interface for data formatting
pub trait DataFormatter<T> {
    /// Format data for specific output type
    fn format_data(&self, data: &[T], format: &OutputFormat) -> QmsResult<String>;
    
    /// Validate data before formatting
    fn validate_data(&self, data: &[T]) -> QmsResult<ValidationResult>;
    
    /// Get supported output formats
    fn supported_formats(&self) -> Vec<OutputFormat>;
}

/// Interface Segregation Principle: Focused interface for report output
pub trait ReportOutputHandler {
    /// Write report to specified destination
    fn write_report(&self, content: &str, destination: &OutputDestination) -> QmsResult<()>;
    
    /// Validate output destination
    fn validate_destination(&self, destination: &OutputDestination) -> QmsResult<()>;
    
    /// Get output statistics
    fn get_output_stats(&self) -> QmsResult<OutputStats>;
}

/// Interface Segregation Principle: Focused interface for report metadata
pub trait ReportMetadataProvider {
    /// Get report metadata
    fn get_metadata(&self) -> ReportMetadata;
    
    /// Get report header information
    fn get_header(&self) -> String;
    
    /// Get report footer information
    fn get_footer(&self) -> String;
    
    /// Get compliance information
    fn get_compliance_info(&self) -> ComplianceInfo;
}

/// Interface Segregation Principle: Focused interface for report validation
pub trait ReportValidator {
    /// Validate report content
    fn validate_report(&self, content: &str, format: &OutputFormat) -> QmsResult<ValidationResult>;
    
    /// Validate compliance requirements
    fn validate_compliance(&self, content: &str, requirements: &[ComplianceRequirement]) -> QmsResult<ComplianceValidationResult>;
    
    /// Get validation rules
    fn get_validation_rules(&self) -> Vec<ValidationRule>;
}

/// Output format enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OutputFormat {
    Markdown,
    CSV,
    JSON,
    HTML,
    PDF,
    XML,
}

impl OutputFormat {
    /// Get file extension for format
    pub fn file_extension(&self) -> &'static str {
        match self {
            OutputFormat::Markdown => "md",
            OutputFormat::CSV => "csv",
            OutputFormat::JSON => "json",
            OutputFormat::HTML => "html",
            OutputFormat::PDF => "pdf",
            OutputFormat::XML => "xml",
        }
    }
    
    /// Get MIME type for format
    pub fn mime_type(&self) -> &'static str {
        match self {
            OutputFormat::Markdown => "text/markdown",
            OutputFormat::CSV => "text/csv",
            OutputFormat::JSON => "application/json",
            OutputFormat::HTML => "text/html",
            OutputFormat::PDF => "application/pdf",
            OutputFormat::XML => "application/xml",
        }
    }
    
    /// Parse format from string
    pub fn from_str(s: &str) -> QmsResult<Self> {
        match s.to_lowercase().as_str() {
            "md" | "markdown" => Ok(OutputFormat::Markdown),
            "csv" => Ok(OutputFormat::CSV),
            "json" => Ok(OutputFormat::JSON),
            "html" => Ok(OutputFormat::HTML),
            "pdf" => Ok(OutputFormat::PDF),
            "xml" => Ok(OutputFormat::XML),
            _ => Err(QmsError::validation_error(&format!("Unsupported format: {}", s))),
        }
    }
}

/// Output destination options
#[derive(Debug, Clone)]
pub enum OutputDestination {
    File(std::path::PathBuf),
    String,
    Stream(String), // Stream identifier
    Memory,
}

/// Data filter criteria
#[derive(Debug, Clone)]
pub struct DataFilterCriteria {
    pub date_range: Option<(String, String)>,
    pub entity_types: Option<Vec<String>>,
    pub status_filter: Option<Vec<String>>,
    pub user_filter: Option<Vec<String>>,
    pub limit: Option<usize>,
    pub custom_filters: std::collections::HashMap<String, String>,
}

impl Default for DataFilterCriteria {
    fn default() -> Self {
        Self {
            date_range: None,
            entity_types: None,
            status_filter: None,
            user_filter: None,
            limit: None,
            custom_filters: std::collections::HashMap::new(),
        }
    }
}

/// Data collection statistics
#[derive(Debug, Clone)]
pub struct DataCollectionStats {
    pub total_items: usize,
    pub filtered_items: usize,
    pub collection_time_ms: u64,
    pub data_sources: Vec<String>,
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Output statistics
#[derive(Debug, Clone)]
pub struct OutputStats {
    pub bytes_written: usize,
    pub write_time_ms: u64,
    pub destination: String,
    pub format: OutputFormat,
}

/// Report metadata
#[derive(Debug, Clone)]
pub struct ReportMetadata {
    pub report_type: String,
    pub version: String,
    pub generated_at: String,
    pub generated_by: String,
    pub project_path: String,
    pub compliance_standards: Vec<String>,
}

/// Compliance information
#[derive(Debug, Clone)]
pub struct ComplianceInfo {
    pub standards: Vec<String>,
    pub requirements: Vec<String>,
    pub certifications: Vec<String>,
    pub audit_trail: bool,
}

/// Compliance requirement
#[derive(Debug, Clone)]
pub struct ComplianceRequirement {
    pub standard: String,
    pub requirement_id: String,
    pub description: String,
    pub mandatory: bool,
}

/// Compliance validation result
#[derive(Debug, Clone)]
pub struct ComplianceValidationResult {
    pub is_compliant: bool,
    pub violations: Vec<ComplianceViolation>,
    pub compliance_score: f64,
}

/// Compliance violation
#[derive(Debug, Clone)]
pub struct ComplianceViolation {
    pub requirement: ComplianceRequirement,
    pub violation_type: String,
    pub description: String,
    pub severity: ViolationSeverity,
}

/// Violation severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Validation rule
#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub name: String,
    pub description: String,
    pub rule_type: ValidationRuleType,
    pub parameters: std::collections::HashMap<String, String>,
}

/// Validation rule types
#[derive(Debug, Clone)]
pub enum ValidationRuleType {
    Required,
    Format,
    Length,
    Pattern,
    Custom(String),
}

/// Report generation context
#[derive(Debug, Clone)]
pub struct ReportContext {
    pub project_path: std::path::PathBuf,
    pub user_id: String,
    pub session_id: Option<String>,
    pub request_id: String,
    pub metadata: std::collections::HashMap<String, String>,
}

impl ReportContext {
    /// Create new report context
    pub fn new(project_path: std::path::PathBuf, user_id: String) -> Self {
        Self {
            project_path,
            user_id,
            session_id: None,
            request_id: crate::utils::generate_uuid(),
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Add metadata to context
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Set session ID
    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_parsing() {
        assert_eq!(OutputFormat::from_str("md").unwrap(), OutputFormat::Markdown);
        assert_eq!(OutputFormat::from_str("CSV").unwrap(), OutputFormat::CSV);
        assert_eq!(OutputFormat::from_str("json").unwrap(), OutputFormat::JSON);
        
        assert!(OutputFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_output_format_properties() {
        assert_eq!(OutputFormat::Markdown.file_extension(), "md");
        assert_eq!(OutputFormat::CSV.mime_type(), "text/csv");
        assert_eq!(OutputFormat::JSON.file_extension(), "json");
    }

    #[test]
    fn test_data_filter_criteria_default() {
        let criteria = DataFilterCriteria::default();
        assert!(criteria.date_range.is_none());
        assert!(criteria.entity_types.is_none());
        assert!(criteria.limit.is_none());
    }

    #[test]
    fn test_report_context_creation() {
        let context = ReportContext::new(
            std::path::PathBuf::from("/test/path"),
            "test-user".to_string()
        )
        .with_metadata("source".to_string(), "unit-test".to_string())
        .with_session("test-session".to_string());
        
        assert_eq!(context.user_id, "test-user");
        assert_eq!(context.session_id, Some("test-session".to_string()));
        assert_eq!(context.metadata.get("source"), Some(&"unit-test".to_string()));
    }

    #[test]
    fn test_validation_result() {
        let result = ValidationResult {
            is_valid: false,
            errors: vec!["Error 1".to_string()],
            warnings: vec!["Warning 1".to_string()],
        };
        
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.warnings.len(), 1);
    }
}
