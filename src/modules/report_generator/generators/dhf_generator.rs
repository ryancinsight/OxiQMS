//! DHF Report Generator - Canonical Implementation
//! 
//! This module provides the canonical Design History File (DHF) report generator,
//! consolidating all DHF reporting functionality following SOLID principles.
//! 
//! SOLID Principles Applied:
//! - Single Responsibility: Handles only DHF report generation
//! - Open/Closed: Extensible through format strategies and data collectors
//! - Liskov Substitution: Can be used wherever report generators are expected
//! - Interface Segregation: Uses focused interfaces for different operations
//! - Dependency Inversion: Depends on abstractions, not concrete implementations

use crate::prelude::*;
use crate::modules::document_control::service::{DocumentService, DocumentIndexEntry};
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

/// Canonical DHF Report Generator
/// Single Responsibility Principle: Focused on DHF report generation
pub struct DHFReportGenerator {
    project_path: std::path::PathBuf,
    data_collector: Box<dyn DataCollector<DocumentIndexEntry>>,
    formatter: Box<dyn DataFormatter<DocumentIndexEntry>>,
}

impl DHFReportGenerator {
    /// Create new DHF report generator with default implementations
    pub fn new(project_path: &std::path::Path) -> Self {
        let project_path_buf = project_path.to_path_buf();
        Self {
            project_path: project_path_buf.clone(),
            data_collector: Box::new(DocumentDataCollector::new(project_path_buf.clone())),
            formatter: Box::new(DHFDataFormatter::new()),
        }
    }
    
    /// Create with custom implementations for testing or advanced use cases
    pub fn with_dependencies(
        project_path: std::path::PathBuf,
        data_collector: Box<dyn DataCollector<DocumentIndexEntry>>,
        formatter: Box<dyn DataFormatter<DocumentIndexEntry>>,
    ) -> Self {
        Self {
            project_path,
            data_collector,
            formatter,
        }
    }
    
    /// Generate DHF report in specified format (legacy API compatibility)
    pub fn generate_report(&self, format: &str, output_path: Option<&str>) -> QmsResult<String> {
        // Convert string format to OutputFormat enum
        let output_format = OutputFormat::from_str(format)?;
        
        // Create destination
        let destination = match output_path {
            Some(path) => OutputDestination::File(std::path::PathBuf::from(path)),
            None => OutputDestination::String,
        };
        
        // Create context
        let context = ReportContext::new(
            self.project_path.clone(),
            "system".to_string(), // Default user for legacy API
        );
        
        // Generate report using Template Method pattern
        let report = self.generate_report_with_context(&output_format, &destination, &context)?;
        
        // Log audit action for compliance
        if let Some(path) = output_path {
            audit_log_action("DHF_REPORT_GENERATED", "Report", path)?;
        }
        
        Ok(report)
    }
    
    /// Generate report with context (new API)
    pub fn generate_report_with_context(&self, format: &OutputFormat, destination: &OutputDestination, context: &ReportContext) -> QmsResult<String> {
        // Use Template Method pattern
        BaseReportGenerator::generate_report(self, format, destination, context)
    }
}

impl BaseReportGenerator<DocumentIndexEntry> for DHFReportGenerator {
    /// Collect document data for DHF report
    fn collect_report_data(&self, _context: &ReportContext) -> QmsResult<Vec<DocumentIndexEntry>> {
        self.data_collector.collect_data()
    }
    
    /// Format document data for specific output format
    fn format_report_data(
        &self,
        data: &[DocumentIndexEntry],
        format: &OutputFormat,
        metadata: &ReportMetadata,
    ) -> QmsResult<String> {
        // Use the injected formatter
        let formatted_data = self.formatter.format_data(data, format)?;
        
        // Add format-specific wrapper using strategy pattern
        let strategy = FormatStrategyFactory::create_strategy(format)?;
        let header = strategy.get_header(metadata, "Design History File (DHF) Report");
        let footer = strategy.get_footer(metadata);
        
        Ok(format!("{}{}{}", header, formatted_data, footer))
    }
    
    /// Get report type identifier
    fn get_report_type(&self) -> &'static str {
        "dhf"
    }
    
    /// Get compliance standards for DHF reports
    fn get_compliance_standards(&self) -> Vec<String> {
        vec![
            "FDA 21 CFR Part 820.30".to_string(),
            "ISO 13485".to_string(),
            "IEC 62304".to_string(),
        ]
    }
    
    /// Get supported output formats for DHF reports
    fn get_supported_formats(&self) -> Vec<OutputFormat> {
        vec![
            OutputFormat::Markdown,
            OutputFormat::CSV,
            OutputFormat::JSON,
            OutputFormat::HTML,
        ]
    }
}

/// Document data collector for DHF reports
/// Single Responsibility Principle: Handles only document data collection
pub struct DocumentDataCollector {
    project_path: std::path::PathBuf,
}

impl DocumentDataCollector {
    pub fn new(project_path: std::path::PathBuf) -> Self {
        Self { project_path }
    }
}

impl DataCollector<DocumentIndexEntry> for DocumentDataCollector {
    fn collect_data(&self) -> QmsResult<Vec<DocumentIndexEntry>> {
        let document_service = DocumentService::new(self.project_path.clone());
        document_service.list_documents()
    }
    
    fn filter_data(&self, data: &[DocumentIndexEntry], criteria: &DataFilterCriteria) -> QmsResult<Vec<DocumentIndexEntry>> {
        let mut filtered = data.to_vec();
        
        // Apply status filter if specified
        if let Some(status_filter) = &criteria.status_filter {
            filtered.retain(|entry| status_filter.contains(&entry.status));
        }
        
        // Apply entity type filter (document type)
        if let Some(entity_types) = &criteria.entity_types {
            filtered.retain(|entry| entity_types.contains(&entry.doc_type));
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
            filtered_items: data.len(),
            collection_time_ms: end_time - start_time,
            data_sources: vec!["document_index.json".to_string()],
        })
    }
}

/// DHF data formatter
/// Single Responsibility Principle: Handles only DHF data formatting
pub struct DHFDataFormatter;

impl DHFDataFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl DataFormatter<DocumentIndexEntry> for DHFDataFormatter {
    fn format_data(&self, data: &[DocumentIndexEntry], format: &OutputFormat) -> QmsResult<String> {
        match format {
            OutputFormat::Markdown => self.format_as_markdown(data),
            OutputFormat::CSV => self.format_as_csv(data),
            OutputFormat::JSON => self.format_as_json(data),
            OutputFormat::HTML => self.format_as_html(data),
            _ => Err(QmsError::validation_error(&format!("Unsupported format: {:?}", format))),
        }
    }
    
    fn validate_data(&self, data: &[DocumentIndexEntry]) -> QmsResult<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        if data.is_empty() {
            warnings.push("No documents found".to_string());
        }
        
        // Validate each entry
        for (i, entry) in data.iter().enumerate() {
            if entry.id.is_empty() {
                errors.push(format!("Document {} has empty ID", i));
            }
            
            if entry.title.is_empty() {
                errors.push(format!("Document {} has empty title", i));
            }
            
            if entry.version.is_empty() {
                errors.push(format!("Document {} has empty version", i));
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

impl DHFDataFormatter {
    /// Format document data as Markdown
    fn format_as_markdown(&self, data: &[DocumentIndexEntry]) -> QmsResult<String> {
        let mut report = String::new();
        
        writeln!(report, "## Document Summary").unwrap();
        writeln!(report).unwrap();
        writeln!(report, "Total Documents: {}", data.len()).unwrap();
        writeln!(report).unwrap();
        
        if data.is_empty() {
            writeln!(report, "*No documents found.*").unwrap();
            return Ok(report);
        }
        
        // Group by status for summary
        let mut status_counts = std::collections::HashMap::new();
        for doc in data {
            *status_counts.entry(&doc.status).or_insert(0) += 1;
        }
        
        writeln!(report, "### Status Summary").unwrap();
        for (status, count) in &status_counts {
            writeln!(report, "- **{}**: {}", status, count).unwrap();
        }
        writeln!(report).unwrap();
        
        writeln!(report, "## Document Details").unwrap();
        writeln!(report).unwrap();
        writeln!(report, "| ID | Title | Version | Type | Status | Path |").unwrap();
        writeln!(report, "|----|-------|---------|------|--------|------|").unwrap();
        
        for doc in data {
            writeln!(
                report,
                "| {} | {} | {} | {} | {} | {} |",
                doc.id,
                ReportFormatter::truncate_string(&doc.title, 30),
                doc.version,
                doc.doc_type,
                doc.status,
                ReportFormatter::truncate_string(&doc.file_path, 40)
            ).unwrap();
        }
        
        Ok(report)
    }
    
    /// Format document data as CSV
    fn format_as_csv(&self, data: &[DocumentIndexEntry]) -> QmsResult<String> {
        let mut csv = String::new();
        
        // CSV header
        writeln!(csv, "ID,Title,Version,Type,Status,Path,Created,Modified").unwrap();
        
        // CSV data rows
        for doc in data {
            writeln!(
                csv,
                "{},{},{},{},{},{},{},{}",
                ReportFormatter::escape_csv_field(&doc.id),
                ReportFormatter::escape_csv_field(&doc.title),
                ReportFormatter::escape_csv_field(&doc.version),
                ReportFormatter::escape_csv_field(&doc.doc_type),
                ReportFormatter::escape_csv_field(&doc.status),
                ReportFormatter::escape_csv_field(&doc.file_path),
                ReportFormatter::escape_csv_field(&doc.created_at),
                ReportFormatter::escape_csv_field(&doc.updated_at)
            ).unwrap();
        }
        
        Ok(csv)
    }
    
    /// Format document data as JSON
    fn format_as_json(&self, data: &[DocumentIndexEntry]) -> QmsResult<String> {
        let mut json = String::new();
        
        writeln!(json, "{{").unwrap();
        writeln!(json, "  \"total_documents\": {},", data.len()).unwrap();
        writeln!(json, "  \"documents\": [").unwrap();
        
        for (i, doc) in data.iter().enumerate() {
            if i > 0 {
                writeln!(json, ",").unwrap();
            }
            
            writeln!(json, "    {{").unwrap();
            writeln!(json, "      \"id\": \"{}\",", ReportFormatter::escape_json_string(&doc.id)).unwrap();
            writeln!(json, "      \"title\": \"{}\",", ReportFormatter::escape_json_string(&doc.title)).unwrap();
            writeln!(json, "      \"version\": \"{}\",", ReportFormatter::escape_json_string(&doc.version)).unwrap();
            writeln!(json, "      \"type\": \"{}\",", ReportFormatter::escape_json_string(&doc.doc_type)).unwrap();
            writeln!(json, "      \"status\": \"{}\",", ReportFormatter::escape_json_string(&doc.status)).unwrap();
            writeln!(json, "      \"path\": \"{}\",", ReportFormatter::escape_json_string(&doc.file_path)).unwrap();
            writeln!(json, "      \"created_at\": \"{}\",", ReportFormatter::escape_json_string(&doc.created_at)).unwrap();
            writeln!(json, "      \"modified_at\": \"{}\"", ReportFormatter::escape_json_string(&doc.updated_at)).unwrap();
            write!(json, "    }}").unwrap();
        }
        
        writeln!(json, "").unwrap();
        writeln!(json, "  ]").unwrap();
        writeln!(json, "}}").unwrap();
        
        Ok(json)
    }
    
    /// Format document data as HTML
    fn format_as_html(&self, data: &[DocumentIndexEntry]) -> QmsResult<String> {
        let mut html = String::new();
        
        writeln!(html, "<h2>Document Summary</h2>").unwrap();
        writeln!(html, "<p><strong>Total Documents:</strong> {}</p>", data.len()).unwrap();
        
        if data.is_empty() {
            writeln!(html, "<p><em>No documents found.</em></p>").unwrap();
            return Ok(html);
        }
        
        writeln!(html, "<table>").unwrap();
        writeln!(html, "<thead>").unwrap();
        writeln!(html, "<tr>").unwrap();
        writeln!(html, "<th>ID</th>").unwrap();
        writeln!(html, "<th>Title</th>").unwrap();
        writeln!(html, "<th>Version</th>").unwrap();
        writeln!(html, "<th>Type</th>").unwrap();
        writeln!(html, "<th>Status</th>").unwrap();
        writeln!(html, "<th>Path</th>").unwrap();
        writeln!(html, "</tr>").unwrap();
        writeln!(html, "</thead>").unwrap();
        writeln!(html, "<tbody>").unwrap();
        
        for doc in data {
            writeln!(html, "<tr>").unwrap();
            writeln!(html, "<td>{}</td>", ReportFormatter::escape_html(&doc.id)).unwrap();
            writeln!(html, "<td>{}</td>", ReportFormatter::escape_html(&doc.title)).unwrap();
            writeln!(html, "<td>{}</td>", ReportFormatter::escape_html(&doc.version)).unwrap();
            writeln!(html, "<td>{}</td>", ReportFormatter::escape_html(&doc.doc_type)).unwrap();
            writeln!(html, "<td>{}</td>", ReportFormatter::escape_html(&doc.status)).unwrap();
            writeln!(html, "<td>{}</td>", ReportFormatter::escape_html(&doc.file_path)).unwrap();
            writeln!(html, "</tr>").unwrap();
        }
        
        writeln!(html, "</tbody>").unwrap();
        writeln!(html, "</table>").unwrap();
        
        Ok(html)
    }
}

impl Default for DHFDataFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_dhf_generator_creation() {
        let temp_dir = tempdir().unwrap();
        let generator = DHFReportGenerator::new(temp_dir.path());
        
        assert_eq!(generator.get_report_type(), "dhf");
        assert!(generator.get_supported_formats().contains(&OutputFormat::Markdown));
    }

    #[test]
    fn test_legacy_api_compatibility() {
        let temp_dir = tempdir().unwrap();
        let generator = DHFReportGenerator::new(temp_dir.path());

        // Test legacy API - these should work even with empty data
        let result = generator.generate_report("md", None);
        assert!(result.is_ok(), "Markdown generation failed: {:?}", result.err());

        let result = generator.generate_report("csv", None);
        assert!(result.is_ok(), "CSV generation failed: {:?}", result.err());

        let result = generator.generate_report("json", None);
        assert!(result.is_ok(), "JSON generation failed: {:?}", result.err());
    }
}
