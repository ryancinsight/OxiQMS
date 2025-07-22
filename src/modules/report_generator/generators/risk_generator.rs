//! Risk Report Generator - Canonical Implementation
//! 
//! This module provides the canonical risk report generator implementation,
//! consolidating all risk reporting functionality following SOLID principles.
//! 
//! SOLID Principles Applied:
//! - Single Responsibility: Handles only risk report generation
//! - Open/Closed: Extensible through format strategies and data collectors
//! - Liskov Substitution: Can be used wherever report generators are expected
//! - Interface Segregation: Uses focused interfaces for different operations
//! - Dependency Inversion: Depends on abstractions, not concrete implementations

use crate::prelude::*;
use crate::modules::risk_manager::{RiskManager, risk::RiskIndexEntry};
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

/// Canonical Risk Report Generator
/// Single Responsibility Principle: Focused on risk report generation
pub struct RiskReportGenerator {
    project_path: std::path::PathBuf,
    data_collector: Box<dyn DataCollector<RiskIndexEntry>>,
    formatter: Box<dyn DataFormatter<RiskIndexEntry>>,
}

impl RiskReportGenerator {
    /// Create new risk report generator with default implementations
    pub fn new(project_path: &std::path::Path) -> Self {
        let project_path_buf = project_path.to_path_buf();
        Self {
            project_path: project_path_buf.clone(),
            data_collector: Box::new(RiskDataCollector::new(project_path_buf.clone())),
            formatter: Box::new(RiskDataFormatter::new()),
        }
    }
    
    /// Create with custom implementations for testing or advanced use cases
    pub fn with_dependencies(
        project_path: std::path::PathBuf,
        data_collector: Box<dyn DataCollector<RiskIndexEntry>>,
        formatter: Box<dyn DataFormatter<RiskIndexEntry>>,
    ) -> Self {
        Self {
            project_path,
            data_collector,
            formatter,
        }
    }
    
    /// Generate risk report in specified format (legacy API compatibility)
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
            audit_log_action("RISK_REPORT_GENERATED", "Report", path)?;
        }
        
        Ok(report)
    }
    
    /// Generate report with context (new API)
    pub fn generate_report_with_context(&self, format: &OutputFormat, destination: &OutputDestination, context: &ReportContext) -> QmsResult<String> {
        // Use Template Method pattern
        BaseReportGenerator::generate_report(self, format, destination, context)
    }
}

impl BaseReportGenerator<RiskIndexEntry> for RiskReportGenerator {
    /// Collect risk data for report
    fn collect_report_data(&self, _context: &ReportContext) -> QmsResult<Vec<RiskIndexEntry>> {
        self.data_collector.collect_data()
    }
    
    /// Format risk data for specific output format
    fn format_report_data(
        &self,
        data: &[RiskIndexEntry],
        format: &OutputFormat,
        metadata: &ReportMetadata,
    ) -> QmsResult<String> {
        // Use the injected formatter
        let formatted_data = self.formatter.format_data(data, format)?;
        
        // Add format-specific wrapper using strategy pattern
        let strategy = FormatStrategyFactory::create_strategy(format)?;
        let header = strategy.get_header(metadata, "Risk Management Report");
        let footer = strategy.get_footer(metadata);
        
        Ok(format!("{}{}{}", header, formatted_data, footer))
    }
    
    /// Get report type identifier
    fn get_report_type(&self) -> &'static str {
        "risk_management"
    }
    
    /// Get compliance standards for risk reports
    fn get_compliance_standards(&self) -> Vec<String> {
        vec![
            "ISO 14971".to_string(),
            "FDA 21 CFR Part 820".to_string(),
            "IEC 62304".to_string(),
        ]
    }
    
    /// Get supported output formats for risk reports
    fn get_supported_formats(&self) -> Vec<OutputFormat> {
        vec![
            OutputFormat::Markdown,
            OutputFormat::CSV,
            OutputFormat::JSON,
            OutputFormat::HTML,
        ]
    }
}

/// Risk data collector
/// Single Responsibility Principle: Handles only risk data collection
pub struct RiskDataCollector {
    project_path: std::path::PathBuf,
}

impl RiskDataCollector {
    pub fn new(project_path: std::path::PathBuf) -> Self {
        Self { project_path }
    }
}

impl DataCollector<RiskIndexEntry> for RiskDataCollector {
    fn collect_data(&self) -> QmsResult<Vec<RiskIndexEntry>> {
        let risk_manager = RiskManager::new(&self.project_path)?;
        risk_manager.list_risks(None)
    }
    
    fn filter_data(&self, data: &[RiskIndexEntry], criteria: &DataFilterCriteria) -> QmsResult<Vec<RiskIndexEntry>> {
        let mut filtered = data.to_vec();
        
        // Apply status filter if specified (convert to string for comparison)
        if let Some(status_filter) = &criteria.status_filter {
            filtered.retain(|entry| status_filter.contains(&format!("{:?}", entry.status)));
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
            data_sources: vec!["risk_index.json".to_string()],
        })
    }
}

/// Risk data formatter
/// Single Responsibility Principle: Handles only risk data formatting
pub struct RiskDataFormatter;

impl RiskDataFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl DataFormatter<RiskIndexEntry> for RiskDataFormatter {
    fn format_data(&self, data: &[RiskIndexEntry], format: &OutputFormat) -> QmsResult<String> {
        match format {
            OutputFormat::Markdown => self.format_as_markdown(data),
            OutputFormat::CSV => self.format_as_csv(data),
            OutputFormat::JSON => self.format_as_json(data),
            OutputFormat::HTML => self.format_as_html(data),
            _ => Err(QmsError::validation_error(&format!("Unsupported format: {:?}", format))),
        }
    }
    
    fn validate_data(&self, data: &[RiskIndexEntry]) -> QmsResult<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        if data.is_empty() {
            warnings.push("No risks found".to_string());
        }
        
        // Validate each entry
        for (i, entry) in data.iter().enumerate() {
            if entry.id.is_empty() {
                errors.push(format!("Risk {} has empty ID", i));
            }
            
            if entry.description.is_empty() {
                errors.push(format!("Risk {} has empty description", i));
            }

            if entry.rpn > 125 {
                errors.push(format!("Risk {} has invalid RPN: {}", i, entry.rpn));
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

impl RiskDataFormatter {
    /// Format risk data as Markdown
    fn format_as_markdown(&self, data: &[RiskIndexEntry]) -> QmsResult<String> {
        let mut report = String::new();
        
        writeln!(report, "## Risk Summary").unwrap();
        writeln!(report).unwrap();
        writeln!(report, "Total Risks: {}", data.len()).unwrap();
        writeln!(report).unwrap();
        
        if data.is_empty() {
            writeln!(report, "*No risks found.*").unwrap();
            return Ok(report);
        }
        
        // Calculate risk statistics based on RPN
        let high_risks = data.iter().filter(|r| r.rpn >= 50).count();
        let medium_risks = data.iter().filter(|r| r.rpn >= 15 && r.rpn < 50).count();
        let low_risks = data.iter().filter(|r| r.rpn < 15).count();
        
        writeln!(report, "### Risk Distribution").unwrap();
        writeln!(report, "- **High Risk (RPN ≥50)**: {}", high_risks).unwrap();
        writeln!(report, "- **Medium Risk (RPN 15-49)**: {}", medium_risks).unwrap();
        writeln!(report, "- **Low Risk (RPN <15)**: {}", low_risks).unwrap();
        writeln!(report).unwrap();
        
        writeln!(report, "## Risk Details").unwrap();
        writeln!(report).unwrap();
        writeln!(report, "| ID | Description | Severity | RPN | Status |").unwrap();
        writeln!(report, "|----|-------------|----------|-----|--------|").unwrap();

        for risk in data {
            let severity_label = match risk.rpn {
                rpn if rpn >= 50 => "🔴 High",
                rpn if rpn >= 15 => "🟡 Medium",
                _ => "🟢 Low",
            };

            writeln!(
                report,
                "| {} | {} | {} | {} | {:?} |",
                risk.id,
                ReportFormatter::truncate_string(&risk.description, 30),
                severity_label,
                risk.rpn,
                risk.status
            ).unwrap();
        }
        
        Ok(report)
    }
    
    /// Format risk data as CSV
    fn format_as_csv(&self, data: &[RiskIndexEntry]) -> QmsResult<String> {
        let mut csv = String::new();
        
        // CSV header
        writeln!(csv, "ID,Hazard ID,Description,Severity,RPN,Status,Created,Modified").unwrap();

        // CSV data rows
        for risk in data {
            writeln!(
                csv,
                "{},{},{},{:?},{},{:?},{},{}",
                ReportFormatter::escape_csv_field(&risk.id),
                ReportFormatter::escape_csv_field(&risk.hazard_id),
                ReportFormatter::escape_csv_field(&risk.description),
                risk.severity,
                risk.rpn,
                risk.status,
                ReportFormatter::escape_csv_field(&risk.created_at),
                ReportFormatter::escape_csv_field(&risk.updated_at)
            ).unwrap();
        }
        
        Ok(csv)
    }
    
    /// Format risk data as JSON
    fn format_as_json(&self, data: &[RiskIndexEntry]) -> QmsResult<String> {
        let mut json = String::new();
        
        writeln!(json, "{{").unwrap();
        writeln!(json, "  \"total_risks\": {},", data.len()).unwrap();
        writeln!(json, "  \"risks\": [").unwrap();
        
        for (i, risk) in data.iter().enumerate() {
            if i > 0 {
                writeln!(json, ",").unwrap();
            }
            
            writeln!(json, "    {{").unwrap();
            writeln!(json, "      \"id\": \"{}\",", ReportFormatter::escape_json_string(&risk.id)).unwrap();
            writeln!(json, "      \"hazard_id\": \"{}\",", ReportFormatter::escape_json_string(&risk.hazard_id)).unwrap();
            writeln!(json, "      \"description\": \"{}\",", ReportFormatter::escape_json_string(&risk.description)).unwrap();
            writeln!(json, "      \"severity\": \"{:?}\",", risk.severity).unwrap();
            writeln!(json, "      \"rpn\": {},", risk.rpn).unwrap();
            writeln!(json, "      \"status\": \"{:?}\",", risk.status).unwrap();
            writeln!(json, "      \"created_at\": \"{}\",", ReportFormatter::escape_json_string(&risk.created_at)).unwrap();
            writeln!(json, "      \"modified_at\": \"{}\"", ReportFormatter::escape_json_string(&risk.updated_at)).unwrap();
            write!(json, "    }}").unwrap();
        }
        
        writeln!(json, "").unwrap();
        writeln!(json, "  ]").unwrap();
        writeln!(json, "}}").unwrap();
        
        Ok(json)
    }
    
    /// Format risk data as HTML
    fn format_as_html(&self, data: &[RiskIndexEntry]) -> QmsResult<String> {
        let mut html = String::new();
        
        writeln!(html, "<h2>Risk Summary</h2>").unwrap();
        writeln!(html, "<p><strong>Total Risks:</strong> {}</p>", data.len()).unwrap();
        
        if data.is_empty() {
            writeln!(html, "<p><em>No risks found.</em></p>").unwrap();
            return Ok(html);
        }
        
        writeln!(html, "<table>").unwrap();
        writeln!(html, "<thead>").unwrap();
        writeln!(html, "<tr>").unwrap();
        writeln!(html, "<th>ID</th>").unwrap();
        writeln!(html, "<th>Description</th>").unwrap();
        writeln!(html, "<th>Severity</th>").unwrap();
        writeln!(html, "<th>RPN</th>").unwrap();
        writeln!(html, "<th>Status</th>").unwrap();
        writeln!(html, "</tr>").unwrap();
        writeln!(html, "</thead>").unwrap();
        writeln!(html, "<tbody>").unwrap();

        for risk in data {
            let severity_class = match risk.rpn {
                rpn if rpn >= 50 => "high-risk",
                rpn if rpn >= 15 => "medium-risk",
                _ => "low-risk",
            };

            writeln!(html, "<tr class=\"{}\">", severity_class).unwrap();
            writeln!(html, "<td>{}</td>", ReportFormatter::escape_html(&risk.id)).unwrap();
            writeln!(html, "<td>{}</td>", ReportFormatter::escape_html(&risk.description)).unwrap();
            writeln!(html, "<td>{:?}</td>", risk.severity).unwrap();
            writeln!(html, "<td>{}</td>", risk.rpn).unwrap();
            writeln!(html, "<td>{:?}</td>", risk.status).unwrap();
            writeln!(html, "</tr>").unwrap();
        }
        
        writeln!(html, "</tbody>").unwrap();
        writeln!(html, "</table>").unwrap();
        
        Ok(html)
    }
}

impl Default for RiskDataFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_risk_generator_creation() {
        let temp_dir = tempdir().unwrap();
        let generator = RiskReportGenerator::new(temp_dir.path());
        
        assert_eq!(generator.get_report_type(), "risk_management");
        assert!(generator.get_supported_formats().contains(&OutputFormat::Markdown));
    }

    #[test]
    fn test_legacy_api_compatibility() {
        let temp_dir = tempdir().unwrap();
        let generator = RiskReportGenerator::new(temp_dir.path());

        // Test legacy API - these should work even with empty data
        let result = generator.generate_report("md", None);
        assert!(result.is_ok(), "Markdown generation failed: {:?}", result.err());

        let result = generator.generate_report("csv", None);
        assert!(result.is_ok(), "CSV generation failed: {:?}", result.err());

        let result = generator.generate_report("json", None);
        assert!(result.is_ok(), "JSON generation failed: {:?}", result.err());
    }
}
