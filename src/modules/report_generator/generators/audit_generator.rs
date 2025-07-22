//! Audit Report Generator - Canonical Implementation
//! 
//! This module provides the canonical audit report generator implementation,
//! consolidating all audit reporting functionality following SOLID principles.
//! 
//! SOLID Principles Applied:
//! - Single Responsibility: Handles only audit report generation
//! - Open/Closed: Extensible through format strategies and data collectors
//! - Liskov Substitution: Can be used wherever report generators are expected
//! - Interface Segregation: Uses focused interfaces for different operations
//! - Dependency Inversion: Depends on abstractions, not concrete implementations

use crate::prelude::*;
use crate::models::AuditEntry;
use crate::json_utils::JsonSerializable;
use crate::modules::audit_logger::audit_log_action;
// KISS: Use public re-exports to avoid complex import issues
use crate::modules::report_generator::{
    ReportGenerator, OutputFormat, DataFilterCriteria, ReportContext,
    ValidationResult, DataCollector, DataFormatter, OutputDestination,
    ReportMetadata, DataCollectionStats
};
use super::super::template_method::*;
use super::super::strategies::*;
use std::fmt::Write;

/// Canonical Audit Report Generator
/// Single Responsibility Principle: Focused on audit report generation
pub struct AuditReportGenerator {
    project_path: std::path::PathBuf,
    data_collector: Box<dyn DataCollector<AuditEntry>>,
    formatter: Box<dyn DataFormatter<AuditEntry>>,
}

impl AuditReportGenerator {
    /// Create new audit report generator with dependency injection
    /// Dependency Inversion Principle: Accept abstractions as parameters
    pub fn new(project_path: &std::path::Path) -> Self {
        let project_path_buf = project_path.to_path_buf();
        Self {
            project_path: project_path_buf.clone(),
            data_collector: Box::new(FileAuditDataCollector::new(project_path_buf.clone())),
            formatter: Box::new(AuditDataFormatter::new()),
        }
    }
    
    /// Create with custom implementations for testing or advanced use cases
    pub fn with_dependencies(
        project_path: std::path::PathBuf,
        data_collector: Box<dyn DataCollector<AuditEntry>>,
        formatter: Box<dyn DataFormatter<AuditEntry>>,
    ) -> Self {
        Self {
            project_path,
            data_collector,
            formatter,
        }
    }
    
    /// Generate audit report in specified format (legacy API compatibility)
    pub fn generate_report(&self, format: &str, output_path: Option<&str>, last_n: Option<usize>) -> QmsResult<String> {
        // Convert string format to OutputFormat enum
        let output_format = OutputFormat::from_str(format)?;

        // Create destination
        let destination = match output_path {
            Some(path) => OutputDestination::File(std::path::PathBuf::from(path)),
            None => OutputDestination::String,
        };

        // Create context with filtering options
        let mut context = ReportContext::new(
            self.project_path.clone(),
            "system".to_string(), // Default user for legacy API
        );

        // Add limit filter if specified
        if let Some(limit) = last_n {
            context = context.with_metadata("limit".to_string(), limit.to_string());
        }

        // Generate report using Template Method pattern
        let report = self.generate_report_with_context(&output_format, &destination, &context)?;

        // Log audit action for compliance
        if let Some(path) = output_path {
            audit_log_action("AUDIT_REPORT_GENERATED", "Report", path)?;
        }

        Ok(report)
    }

    /// Generate report with context (new API)
    pub fn generate_report_with_context(&self, format: &OutputFormat, destination: &OutputDestination, context: &ReportContext) -> QmsResult<String> {
        // Use Template Method pattern
        BaseReportGenerator::generate_report(self, format, destination, context)
    }
}

impl BaseReportGenerator<AuditEntry> for AuditReportGenerator {
    /// Collect audit data from the audit log
    fn collect_report_data(&self, _context: &ReportContext) -> QmsResult<Vec<AuditEntry>> {
        self.data_collector.collect_data()
    }
    
    /// Format audit data for specific output format
    fn format_report_data(
        &self,
        data: &[AuditEntry],
        format: &OutputFormat,
        metadata: &ReportMetadata,
    ) -> QmsResult<String> {
        // Use the injected formatter
        let formatted_data = self.formatter.format_data(data, format)?;
        
        // Add format-specific wrapper using strategy pattern
        let strategy = FormatStrategyFactory::create_strategy(format)?;
        let header = strategy.get_header(metadata, "Audit Trail Report");
        let footer = strategy.get_footer(metadata);
        
        Ok(format!("{}{}{}", header, formatted_data, footer))
    }
    
    /// Get report type identifier
    fn get_report_type(&self) -> &'static str {
        "audit_trail"
    }
    
    /// Filter audit data based on context
    fn filter_report_data(&self, data: &[AuditEntry], context: &ReportContext) -> QmsResult<Vec<AuditEntry>> {
        let mut filtered_data = data.to_vec();
        
        // Apply date range filter if specified
        if let Some(start_date) = context.metadata.get("start_date") {
            filtered_data.retain(|entry| entry.timestamp >= *start_date);
        }
        
        if let Some(end_date) = context.metadata.get("end_date") {
            filtered_data.retain(|entry| entry.timestamp <= *end_date);
        }
        
        // Apply user filter if specified
        if let Some(user_filter) = context.metadata.get("user_filter") {
            filtered_data.retain(|entry| entry.user_id == *user_filter);
        }
        
        // Apply entity type filter if specified
        if let Some(entity_type) = context.metadata.get("entity_type") {
            filtered_data.retain(|entry| entry.entity_type == *entity_type);
        }
        
        // Apply limit if specified
        if let Some(limit_str) = context.metadata.get("limit") {
            if let Ok(limit) = limit_str.parse::<usize>() {
                filtered_data.truncate(limit);
            }
        }
        
        Ok(filtered_data)
    }
    
    /// Get compliance standards for audit reports
    fn get_compliance_standards(&self) -> Vec<String> {
        vec![
            "FDA 21 CFR Part 11".to_string(),
            "ISO 13485".to_string(),
            "ISO 27001".to_string(),
        ]
    }
    
    /// Get supported output formats for audit reports
    fn get_supported_formats(&self) -> Vec<OutputFormat> {
        vec![
            OutputFormat::Markdown,
            OutputFormat::CSV,
            OutputFormat::JSON,
            OutputFormat::HTML,
        ]
    }
}

/// File-based audit data collector
/// Single Responsibility Principle: Handles only audit data collection from files
pub struct FileAuditDataCollector {
    project_path: std::path::PathBuf,
}

impl FileAuditDataCollector {
    pub fn new(project_path: std::path::PathBuf) -> Self {
        Self { project_path }
    }
    
    /// Get audit log file path
    fn get_audit_log_path(&self) -> std::path::PathBuf {
        self.project_path.join("audit").join("audit.log")
    }
}

impl DataCollector<AuditEntry> for FileAuditDataCollector {
    fn collect_data(&self) -> QmsResult<Vec<AuditEntry>> {
        let log_path = self.get_audit_log_path();
        
        if !log_path.exists() {
            return Ok(Vec::new());
        }
        
        let content = std::fs::read_to_string(log_path)?;
        let mut entries = Vec::new();
        
        for line in content.lines() {
            if !line.trim().is_empty() {
                match AuditEntry::from_json(line) {
                    Ok(entry) => entries.push(entry),
                    Err(e) => {
                        eprintln!("Warning: Failed to parse audit entry: {}", e);
                    }
                }
            }
        }
        
        // Sort by timestamp (most recent first)
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(entries)
    }
    
    fn filter_data(&self, data: &[AuditEntry], criteria: &DataFilterCriteria) -> QmsResult<Vec<AuditEntry>> {
        let mut filtered = data.to_vec();
        
        // Apply date range filter
        if let Some((start, end)) = &criteria.date_range {
            filtered.retain(|entry| entry.timestamp >= *start && entry.timestamp <= *end);
        }
        
        // Apply entity type filter
        if let Some(entity_types) = &criteria.entity_types {
            filtered.retain(|entry| entity_types.contains(&entry.entity_type));
        }
        
        // Apply user filter
        if let Some(users) = &criteria.user_filter {
            filtered.retain(|entry| users.contains(&entry.user_id));
        }
        
        // Apply limit
        if let Some(limit) = criteria.limit {
            filtered.truncate(limit);
        }
        
        Ok(filtered)
    }
    
    fn get_collection_stats(&self) -> QmsResult<DataCollectionStats> {
        let start_time = crate::utils::current_timestamp();
        let data = self.collect_data()?;
        let end_time = crate::utils::current_timestamp();
        
        Ok(DataCollectionStats {
            total_items: data.len(),
            filtered_items: data.len(), // No filtering applied in basic collection
            collection_time_ms: end_time - start_time,
            data_sources: vec!["audit.log".to_string()],
        })
    }
}

/// Audit data formatter
/// Single Responsibility Principle: Handles only audit data formatting
pub struct AuditDataFormatter;

impl AuditDataFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl DataFormatter<AuditEntry> for AuditDataFormatter {
    fn format_data(&self, data: &[AuditEntry], format: &OutputFormat) -> QmsResult<String> {
        match format {
            OutputFormat::Markdown => self.format_as_markdown(data),
            OutputFormat::CSV => self.format_as_csv(data),
            OutputFormat::JSON => self.format_as_json(data),
            OutputFormat::HTML => self.format_as_html(data),
            _ => Err(QmsError::validation_error(&format!("Unsupported format: {:?}", format))),
        }
    }
    
    fn validate_data(&self, data: &[AuditEntry]) -> QmsResult<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        if data.is_empty() {
            warnings.push("No audit entries found".to_string());
        }
        
        // Validate each entry
        for (i, entry) in data.iter().enumerate() {
            if entry.id.is_empty() {
                errors.push(format!("Entry {} has empty ID", i));
            }
            
            if entry.user_id.is_empty() {
                errors.push(format!("Entry {} has empty user ID", i));
            }
            
            if entry.timestamp.is_empty() {
                errors.push(format!("Entry {} has empty timestamp", i));
            }
        }
        
        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        })
    }
    
    fn supported_formats(&self) -> Vec<OutputFormat> {
        vec![
            OutputFormat::Markdown,
            OutputFormat::CSV,
            OutputFormat::JSON,
            OutputFormat::HTML,
        ]
    }
}

impl AuditDataFormatter {
    /// Format audit data as Markdown
    fn format_as_markdown(&self, data: &[AuditEntry]) -> QmsResult<String> {
        let mut report = String::new();
        
        writeln!(report, "## Audit Trail Entries").unwrap();
        writeln!(report).unwrap();
        writeln!(report, "Total Entries: {}", data.len()).unwrap();
        writeln!(report).unwrap();
        
        if data.is_empty() {
            writeln!(report, "*No audit entries found.*").unwrap();
            return Ok(report);
        }
        
        writeln!(report, "| Timestamp | User | Action | Entity | Details |").unwrap();
        writeln!(report, "|-----------|------|--------|--------|---------|").unwrap();
        
        for entry in data {
            let details = entry.details.as_deref().unwrap_or("N/A");
            let truncated_details = ReportFormatter::truncate_string(details, 50);
            
            writeln!(
                report,
                "| {} | {} | {:?} | {} {} | {} |",
                ReportFormatter::format_timestamp(&entry.timestamp),
                entry.user_id,
                entry.action,
                entry.entity_type,
                entry.entity_id,
                truncated_details
            ).unwrap();
        }
        
        Ok(report)
    }
    
    /// Format audit data as CSV
    fn format_as_csv(&self, data: &[AuditEntry]) -> QmsResult<String> {
        let mut csv = String::new();
        
        // CSV header
        writeln!(csv, "Timestamp,User ID,Session ID,Action,Entity Type,Entity ID,Details,IP Address,Checksum").unwrap();
        
        // CSV data rows
        for entry in data {
            writeln!(
                csv,
                "{},{},{},{:?},{},{},{},{},{}",
                ReportFormatter::escape_csv_field(&entry.timestamp),
                ReportFormatter::escape_csv_field(&entry.user_id),
                ReportFormatter::escape_csv_field(&entry.session_id.as_deref().unwrap_or("N/A")),
                entry.action,
                ReportFormatter::escape_csv_field(&entry.entity_type),
                ReportFormatter::escape_csv_field(&entry.entity_id),
                ReportFormatter::escape_csv_field(&entry.details.as_deref().unwrap_or("N/A")),
                ReportFormatter::escape_csv_field(&entry.ip_address.as_deref().unwrap_or("N/A")),
                ReportFormatter::escape_csv_field(&entry.checksum)
            ).unwrap();
        }
        
        Ok(csv)
    }
    
    /// Format audit data as JSON
    fn format_as_json(&self, data: &[AuditEntry]) -> QmsResult<String> {
        let mut json = String::new();
        
        writeln!(json, "[").unwrap();
        
        for (i, entry) in data.iter().enumerate() {
            if i > 0 {
                writeln!(json, ",").unwrap();
            }
            
            writeln!(json, "  {{").unwrap();
            writeln!(json, "    \"id\": \"{}\",", ReportFormatter::escape_json_string(&entry.id)).unwrap();
            writeln!(json, "    \"timestamp\": \"{}\",", ReportFormatter::escape_json_string(&entry.timestamp)).unwrap();
            writeln!(json, "    \"user_id\": \"{}\",", ReportFormatter::escape_json_string(&entry.user_id)).unwrap();
            writeln!(json, "    \"session_id\": {},", 
                if let Some(ref session) = entry.session_id {
                    format!("\"{}\"", ReportFormatter::escape_json_string(session))
                } else {
                    "null".to_string()
                }
            ).unwrap();
            writeln!(json, "    \"action\": \"{:?}\",", entry.action).unwrap();
            writeln!(json, "    \"entity_type\": \"{}\",", ReportFormatter::escape_json_string(&entry.entity_type)).unwrap();
            writeln!(json, "    \"entity_id\": \"{}\",", ReportFormatter::escape_json_string(&entry.entity_id)).unwrap();
            writeln!(json, "    \"details\": {},", 
                if let Some(ref details) = entry.details {
                    format!("\"{}\"", ReportFormatter::escape_json_string(details))
                } else {
                    "null".to_string()
                }
            ).unwrap();
            writeln!(json, "    \"ip_address\": {},", 
                if let Some(ref ip) = entry.ip_address {
                    format!("\"{}\"", ReportFormatter::escape_json_string(ip))
                } else {
                    "null".to_string()
                }
            ).unwrap();
            writeln!(json, "    \"checksum\": \"{}\"", ReportFormatter::escape_json_string(&entry.checksum)).unwrap();
            write!(json, "  }}").unwrap();
        }
        
        writeln!(json, "").unwrap();
        writeln!(json, "]").unwrap();
        
        Ok(json)
    }
    
    /// Format audit data as HTML
    fn format_as_html(&self, data: &[AuditEntry]) -> QmsResult<String> {
        let mut html = String::new();
        
        writeln!(html, "<h2>Audit Trail Entries</h2>").unwrap();
        writeln!(html, "<p><strong>Total Entries:</strong> {}</p>", data.len()).unwrap();
        
        if data.is_empty() {
            writeln!(html, "<p><em>No audit entries found.</em></p>").unwrap();
            return Ok(html);
        }
        
        writeln!(html, "<table>").unwrap();
        writeln!(html, "<thead>").unwrap();
        writeln!(html, "<tr>").unwrap();
        writeln!(html, "<th>Timestamp</th>").unwrap();
        writeln!(html, "<th>User</th>").unwrap();
        writeln!(html, "<th>Action</th>").unwrap();
        writeln!(html, "<th>Entity</th>").unwrap();
        writeln!(html, "<th>Details</th>").unwrap();
        writeln!(html, "</tr>").unwrap();
        writeln!(html, "</thead>").unwrap();
        writeln!(html, "<tbody>").unwrap();
        
        for entry in data {
            writeln!(html, "<tr>").unwrap();
            writeln!(html, "<td>{}</td>", ReportFormatter::escape_html(&entry.timestamp)).unwrap();
            writeln!(html, "<td>{}</td>", ReportFormatter::escape_html(&entry.user_id)).unwrap();
            writeln!(html, "<td>{:?}</td>", entry.action).unwrap();
            writeln!(html, "<td>{} {}</td>", 
                ReportFormatter::escape_html(&entry.entity_type),
                ReportFormatter::escape_html(&entry.entity_id)
            ).unwrap();
            writeln!(html, "<td>{}</td>", 
                ReportFormatter::escape_html(&entry.details.as_deref().unwrap_or("N/A"))
            ).unwrap();
            writeln!(html, "</tr>").unwrap();
        }
        
        writeln!(html, "</tbody>").unwrap();
        writeln!(html, "</table>").unwrap();
        
        Ok(html)
    }
}

impl Default for AuditDataFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::models::AuditAction;

    #[test]
    fn test_audit_generator_creation() {
        let temp_dir = tempdir().unwrap();
        let generator = AuditReportGenerator::new(temp_dir.path());
        
        assert_eq!(generator.get_report_type(), "audit_trail");
        assert!(generator.get_supported_formats().contains(&OutputFormat::Markdown));
    }

    #[test]
    fn test_legacy_api_compatibility() {
        let temp_dir = tempdir().unwrap();
        let generator = AuditReportGenerator::new(temp_dir.path());

        // Test legacy API - these should work even with empty data
        let result = generator.generate_report("md", None, Some(10));
        assert!(result.is_ok(), "Markdown generation failed: {:?}", result.err());

        let result = generator.generate_report("csv", None, None);
        assert!(result.is_ok(), "CSV generation failed: {:?}", result.err());

        let result = generator.generate_report("json", None, None);
        assert!(result.is_ok(), "JSON generation failed: {:?}", result.err());
    }

    #[test]
    fn test_audit_data_formatter() {
        let formatter = AuditDataFormatter::new();
        
        let sample_entry = AuditEntry {
            id: "test-id".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            user_id: "test-user".to_string(),
            session_id: Some("test-session".to_string()),
            action: AuditAction::Create,
            entity_type: "Document".to_string(),
            entity_id: "DOC-001".to_string(),
            old_value: None,
            new_value: None,
            details: Some("Test entry".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            signature: None,
            checksum: "test-checksum".to_string(),
            previous_hash: None,
        };
        
        let data = vec![sample_entry];
        
        // Test all formats
        assert!(formatter.format_as_markdown(&data).is_ok());
        assert!(formatter.format_as_csv(&data).is_ok());
        assert!(formatter.format_as_json(&data).is_ok());
        assert!(formatter.format_as_html(&data).is_ok());
    }
}
