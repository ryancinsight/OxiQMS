//! Template Method Pattern for Report Generation
//! 
//! This module implements the Template Method pattern to eliminate code duplication
//! between different report generators while following SOLID principles.
//! 
//! SOLID Principles Applied:
//! - Single Responsibility: Base class handles common report generation workflow
//! - Open/Closed: New report types can be added by extending the base class
//! - Liskov Substitution: All report generators can be used interchangeably
//! - Template Method Pattern: Defines the skeleton of report generation algorithm

use crate::prelude::*;
use super::super::interfaces::report_interfaces::*;
use crate::modules::audit_logger::audit_log_action;
use std::fmt::Write;

/// Template Method Pattern: Base report generator with common workflow
/// Single Responsibility Principle: Handles the common report generation workflow
pub trait BaseReportGenerator<T: Clone> {
    /// Template method defining the report generation algorithm
    /// This method should not be overridden by subclasses
    fn generate_report(
        &self,
        format: &OutputFormat,
        destination: &OutputDestination,
        context: &ReportContext,
    ) -> QmsResult<String> {
        // Step 1: Collect data (implemented by subclasses)
        let raw_data = self.collect_report_data(context)?;
        
        // Step 2: Filter data (common implementation with customization hooks)
        let filtered_data = self.filter_report_data(&raw_data, context)?;
        
        // Step 3: Validate data (common implementation)
        self.validate_report_data(&filtered_data)?;
        
        // Step 4: Generate metadata (common implementation with customization)
        let metadata = self.generate_report_metadata(context);
        
        // Step 5: Format data (implemented by subclasses)
        let formatted_content = self.format_report_data(&filtered_data, format, &metadata)?;
        
        // Step 6: Post-process content (hook for subclasses)
        let final_content = self.post_process_content(&formatted_content, format)?;
        
        // Step 7: Write output (common implementation)
        self.write_report_output(&final_content, destination, context)?;
        
        // Step 8: Log audit trail (common implementation)
        self.log_report_generation(format, destination, context)?;
        
        Ok(final_content)
    }
    
    // Abstract methods that must be implemented by subclasses
    
    /// Collect raw data for the report (must be implemented by subclasses)
    fn collect_report_data(&self, context: &ReportContext) -> QmsResult<Vec<T>>;
    
    /// Format data for specific output format (must be implemented by subclasses)
    fn format_report_data(
        &self,
        data: &[T],
        format: &OutputFormat,
        metadata: &ReportMetadata,
    ) -> QmsResult<String>;
    
    /// Get report type identifier (must be implemented by subclasses)
    fn get_report_type(&self) -> &'static str;
    
    // Hook methods with default implementations (can be overridden by subclasses)
    
    /// Filter collected data (default implementation, can be overridden)
    fn filter_report_data(&self, data: &[T], _context: &ReportContext) -> QmsResult<Vec<T>> {
        // Default: no filtering
        Ok(data.to_vec())
    }
    
    /// Validate report data (default implementation, can be overridden)
    fn validate_report_data(&self, _data: &[T]) -> QmsResult<()> {
        // Default implementation allows empty data - reports can be generated with no data
        // Subclasses can override this to add specific validation requirements
        Ok(())
    }
    
    /// Generate report metadata (default implementation, can be overridden)
    fn generate_report_metadata(&self, context: &ReportContext) -> ReportMetadata {
        ReportMetadata {
            report_type: self.get_report_type().to_string(),
            version: "1.0".to_string(),
            generated_at: crate::utils::current_iso8601_timestamp(),
            generated_by: context.user_id.clone(),
            project_path: context.project_path.display().to_string(),
            compliance_standards: self.get_compliance_standards(),
        }
    }
    
    /// Post-process formatted content (hook for subclasses)
    fn post_process_content(&self, content: &str, _format: &OutputFormat) -> QmsResult<String> {
        // Default: no post-processing
        Ok(content.to_string())
    }
    
    /// Get compliance standards for this report type (can be overridden)
    fn get_compliance_standards(&self) -> Vec<String> {
        vec![
            "ISO 13485".to_string(),
            "FDA 21 CFR Part 820".to_string(),
            "FDA 21 CFR Part 11".to_string(),
        ]
    }
    
    /// Get supported output formats (can be overridden)
    fn get_supported_formats(&self) -> Vec<OutputFormat> {
        vec![
            OutputFormat::Markdown,
            OutputFormat::CSV,
            OutputFormat::JSON,
            OutputFormat::HTML,
        ]
    }
    
    // Common implementation methods (used by template method)
    
    /// Write report output to destination
    fn write_report_output(
        &self,
        content: &str,
        destination: &OutputDestination,
        _context: &ReportContext,
    ) -> QmsResult<()> {
        match destination {
            OutputDestination::File(path) => {
                // Ensure directory exists
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(path, content)?;
            }
            OutputDestination::String => {
                // Content is already in string format, nothing to do
            }
            OutputDestination::Stream(_stream_id) => {
                // For now, treat as string output
                // In a full implementation, this would write to a specific stream
            }
            OutputDestination::Memory => {
                // Content is kept in memory, nothing to do
            }
        }
        Ok(())
    }
    
    /// Log report generation for audit trail
    fn log_report_generation(
        &self,
        format: &OutputFormat,
        destination: &OutputDestination,
        context: &ReportContext,
    ) -> QmsResult<()> {
        let destination_str = match destination {
            OutputDestination::File(path) => path.display().to_string(),
            OutputDestination::String => "string".to_string(),
            OutputDestination::Stream(id) => format!("stream:{}", id),
            OutputDestination::Memory => "memory".to_string(),
        };
        
        let details = format!(
            "Report type: {}, Format: {:?}, Destination: {}, User: {}",
            self.get_report_type(),
            format,
            destination_str,
            context.user_id
        );
        
        audit_log_action("REPORT_GENERATED", "Report", &details)?;
        Ok(())
    }
}

/// Common report formatting utilities
/// Single Responsibility Principle: Handles common formatting operations
pub struct ReportFormatter;

impl ReportFormatter {
    /// Generate common report header
    pub fn generate_header(metadata: &ReportMetadata, title: &str) -> String {
        format!(
            "# {}\n\n\
            **Report Type:** {}\n\
            **Generated:** {}\n\
            **Generated By:** {}\n\
            **Project:** {}\n\
            **Version:** {}\n\
            **Compliance Standards:** {}\n\n",
            title,
            metadata.report_type,
            metadata.generated_at,
            metadata.generated_by,
            metadata.project_path,
            metadata.version,
            metadata.compliance_standards.join(", ")
        )
    }
    
    /// Generate common report footer
    pub fn generate_footer(metadata: &ReportMetadata) -> String {
        format!(
            "\n---\n\
            *This report was generated automatically by the QMS system.*\n\
            *Report ID: {}*\n\
            *Generated: {}*\n",
            crate::utils::generate_uuid(),
            metadata.generated_at
        )
    }
    
    /// Escape CSV field content
    pub fn escape_csv_field(field: &str) -> String {
        if field.contains(',') || field.contains('"') || field.contains('\n') {
            format!("\"{}\"", field.replace('"', "\"\""))
        } else {
            field.to_string()
        }
    }
    
    /// Escape JSON string content
    pub fn escape_json_string(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }
    
    /// Escape HTML content
    pub fn escape_html(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }
    
    /// Format timestamp for display
    pub fn format_timestamp(timestamp: &str) -> String {
        // Simple timestamp formatting - in a full implementation,
        // this would handle various timestamp formats and localization
        timestamp.to_string()
    }
    
    /// Generate table of contents for markdown
    pub fn generate_toc(sections: &[String]) -> String {
        let mut toc = String::from("## Table of Contents\n\n");
        for (i, section) in sections.iter().enumerate() {
            writeln!(toc, "{}. [{}](#{}) ", i + 1, section, section.to_lowercase().replace(' ', "-")).unwrap();
        }
        toc.push('\n');
        toc
    }
    
    /// Generate summary statistics section
    pub fn generate_summary_stats(stats: &std::collections::HashMap<String, usize>) -> String {
        let mut summary = String::from("## Summary Statistics\n\n");
        
        for (key, value) in stats {
            writeln!(summary, "- **{}:** {}", key, value).unwrap();
        }
        
        summary.push('\n');
        summary
    }
    
    /// Truncate string to specified length with ellipsis
    pub fn truncate_string(s: &str, max_length: usize) -> String {
        if s.len() <= max_length {
            s.to_string()
        } else {
            format!("{}...", &s[..max_length.saturating_sub(3)])
        }
    }
}

/// Report generation statistics
#[derive(Debug, Clone)]
pub struct ReportGenerationStats {
    pub data_collection_time_ms: u64,
    pub formatting_time_ms: u64,
    pub total_time_ms: u64,
    pub data_items_processed: usize,
    pub output_size_bytes: usize,
}

impl ReportGenerationStats {
    /// Create new stats tracker
    pub fn new() -> Self {
        Self {
            data_collection_time_ms: 0,
            formatting_time_ms: 0,
            total_time_ms: 0,
            data_items_processed: 0,
            output_size_bytes: 0,
        }
    }
    
    /// Convert to JSON for logging
    pub fn to_json(&self) -> String {
        format!(
            r#"{{
    "data_collection_time_ms": {},
    "formatting_time_ms": {},
    "total_time_ms": {},
    "data_items_processed": {},
    "output_size_bytes": {}
}}"#,
            self.data_collection_time_ms,
            self.formatting_time_ms,
            self.total_time_ms,
            self.data_items_processed,
            self.output_size_bytes
        )
    }
}

impl Default for ReportGenerationStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_formatter_header() {
        let metadata = ReportMetadata {
            report_type: "Test Report".to_string(),
            version: "1.0".to_string(),
            generated_at: "2024-01-01T00:00:00Z".to_string(),
            generated_by: "test-user".to_string(),
            project_path: "/test/path".to_string(),
            compliance_standards: vec!["ISO 13485".to_string()],
        };
        
        let header = ReportFormatter::generate_header(&metadata, "Test Report Title");
        
        assert!(header.contains("# Test Report Title"));
        assert!(header.contains("**Report Type:** Test Report"));
        assert!(header.contains("**Generated By:** test-user"));
        assert!(header.contains("ISO 13485"));
    }

    #[test]
    fn test_csv_escaping() {
        assert_eq!(ReportFormatter::escape_csv_field("simple"), "simple");
        assert_eq!(ReportFormatter::escape_csv_field("with,comma"), "\"with,comma\"");
        assert_eq!(ReportFormatter::escape_csv_field("with\"quote"), "\"with\"\"quote\"");
    }

    #[test]
    fn test_json_escaping() {
        assert_eq!(ReportFormatter::escape_json_string("simple"), "simple");
        assert_eq!(ReportFormatter::escape_json_string("with\"quote"), "with\\\"quote");
        assert_eq!(ReportFormatter::escape_json_string("with\nnewline"), "with\\nnewline");
    }

    #[test]
    fn test_html_escaping() {
        assert_eq!(ReportFormatter::escape_html("simple"), "simple");
        assert_eq!(ReportFormatter::escape_html("<tag>"), "&lt;tag&gt;");
        assert_eq!(ReportFormatter::escape_html("A & B"), "A &amp; B");
    }

    #[test]
    fn test_string_truncation() {
        assert_eq!(ReportFormatter::truncate_string("short", 10), "short");
        assert_eq!(ReportFormatter::truncate_string("this is a very long string", 10), "this is...");
    }

    #[test]
    fn test_toc_generation() {
        let sections = vec!["Introduction".to_string(), "Data Analysis".to_string()];
        let toc = ReportFormatter::generate_toc(&sections);
        
        assert!(toc.contains("## Table of Contents"));
        assert!(toc.contains("1. [Introduction](#introduction)"));
        assert!(toc.contains("2. [Data Analysis](#data-analysis)"));
    }

    #[test]
    fn test_stats_json() {
        let stats = ReportGenerationStats {
            data_collection_time_ms: 100,
            formatting_time_ms: 50,
            total_time_ms: 150,
            data_items_processed: 25,
            output_size_bytes: 1024,
        };
        
        let json = stats.to_json();
        assert!(json.contains("\"data_collection_time_ms\": 100"));
        assert!(json.contains("\"total_time_ms\": 150"));
        assert!(json.contains("\"data_items_processed\": 25"));
    }
}
