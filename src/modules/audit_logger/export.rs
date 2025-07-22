//! Audit Export and Reporting Module
//! 
//! Provides comprehensive audit log export capabilities in multiple formats
//! for regulatory compliance, documentation, and analysis purposes.
//! Supports PDF, CSV, JSON, and XML export formats with filtering options.

use crate::prelude::*;
use crate::modules::audit_logger::{AuditSearchEngine, AuditSearchCriteria};
use crate::json_utils::JsonSerializable;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Export format enumeration
#[derive(Debug, Clone)]
pub enum ExportFormat {
    PDF,
    CSV,
    JSON,
    XML,
}

impl ExportFormat {
    /// Parse format from string
    pub fn from_string(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "pdf" => Ok(Self::PDF),
            "csv" => Ok(Self::CSV),
            "json" => Ok(Self::JSON),
            "xml" => Ok(Self::XML),
            _ => Err(format!("Unsupported export format: {s}")),
        }
    }
    
    /// Get file extension for format
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::PDF => "pdf",
            Self::CSV => "csv", 
            Self::JSON => "json",
            Self::XML => "xml",
        }
    }
}

/// Audit export options
#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub output_path: PathBuf,
    pub filter: Option<String>,
    pub include_headers: bool,
    pub include_metadata: bool,
    pub max_entries: Option<usize>,
}

impl ExportOptions {
    /// Create new export options
    pub const fn new(format: ExportFormat, output_path: PathBuf) -> Self {
        Self {
            format,
            output_path,
            filter: None,
            include_headers: true,
            include_metadata: true,
            max_entries: None,
        }
    }
    
    /// Set filter criteria
    pub fn with_filter(mut self, filter: String) -> Self {
        self.filter = Some(filter);
        self
    }
    
    /// Set maximum entries
    pub const fn with_max_entries(mut self, max_entries: usize) -> Self {
        self.max_entries = Some(max_entries);
        self
    }
}

/// Export statistics
#[derive(Debug, Clone)]
pub struct ExportStats {
    pub total_entries: usize,
    pub exported_entries: usize,
    pub filtered_entries: usize,
    pub file_size: u64,
    pub processing_time_ms: u64,
    pub export_format: String,
}

/// Audit export engine
pub struct AuditExportEngine {
    project_path: PathBuf,
}

impl AuditExportEngine {
    /// Create new export engine
    pub const fn new(project_path: PathBuf) -> Self {
        Self { project_path }
    }
    
    /// Export audit logs with specified options
    pub fn export_audit_logs(&self, options: &ExportOptions) -> QmsResult<ExportStats> {
        let start_time = std::time::Instant::now();
        
        // Search audit logs based on filter
        let search_criteria = self.build_search_criteria(options)?;
        let search_engine = AuditSearchEngine::new(self.project_path.clone());
        let search_results = search_engine.search(&search_criteria)?;
        
        // Apply entry limit if specified
        let entries_to_export = if let Some(max_entries) = options.max_entries {
            search_results.entries.into_iter().take(max_entries).collect()
        } else {
            search_results.entries
        };
        
        // Export based on format
        let file_size = match options.format {
            ExportFormat::PDF => self.export_pdf(&entries_to_export, options)?,
            ExportFormat::CSV => self.export_csv(&entries_to_export, options)?,
            ExportFormat::JSON => self.export_json(&entries_to_export, options)?,
            ExportFormat::XML => self.export_xml(&entries_to_export, options)?,
        };
        
        let processing_time = start_time.elapsed().as_millis() as u64;
        
        Ok(ExportStats {
            total_entries: search_results.total_matches,
            exported_entries: entries_to_export.len(),
            filtered_entries: search_results.total_matches - entries_to_export.len(),
            file_size,
            processing_time_ms: processing_time,
            export_format: format!("{:?}", options.format),
        })
    }
    
    /// Build search criteria from export options
    fn build_search_criteria(&self, options: &ExportOptions) -> QmsResult<AuditSearchCriteria> {
        let mut criteria = AuditSearchCriteria::new();
        
        if let Some(filter) = &options.filter {
            // Parse filter string (simple implementation)
            // Format: "user:john,action:create,date:2024-01-01"
            for part in filter.split(',') {
                if let Some((key, value)) = part.split_once(':') {
                    match key.trim() {
                        "user" => criteria.user_filter = Some(value.trim().to_string()),
                        "action" => criteria.action_filter = Some(value.trim().to_string()),
                        "entity_type" => criteria.entity_type_filter = Some(value.trim().to_string()),
                        "entity_id" => criteria.entity_id_filter = Some(value.trim().to_string()),
                        _ => {
                            // Ignore unknown filter keys
                        }
                    }
                }
            }
        }
        
        Ok(criteria)
    }
    
    /// Export to PDF format (simplified text-based PDF)
    fn export_pdf(&self, entries: &[crate::models::AuditEntry], options: &ExportOptions) -> QmsResult<u64> {
        let mut content = String::new();
        
        if options.include_headers {
            content.push_str("AUDIT LOG REPORT\n");
            content.push_str("================\n\n");
            content.push_str(&format!("Generated: {}\n", crate::utils::current_iso8601_timestamp()));
            content.push_str(&format!("Total Entries: {}\n", entries.len()));
            content.push_str(&format!("Project: {}\n\n", self.project_path.display()));
        }
        
        for (i, entry) in entries.iter().enumerate() {
            content.push_str(&format!("Entry #{}\n", i + 1));
            content.push_str(&format!("ID: {}\n", entry.id));
            content.push_str(&format!("Timestamp: {}\n", entry.timestamp));
            content.push_str(&format!("User: {}\n", entry.user_id));
            content.push_str(&format!("Action: {:?}\n", entry.action));
            content.push_str(&format!("Entity: {} ({})\n", entry.entity_type, entry.entity_id));
            content.push_str(&format!("Details: {}\n", entry.details.as_ref().unwrap_or(&"N/A".to_string())));
            content.push_str(&format!("Checksum: {}\n", entry.checksum));
            content.push_str("----------------------------------------\n\n");
        }
        
        // Write to file (simplified PDF - in real implementation would use PDF library)
        fs::write(&options.output_path, content.as_bytes())
            .map_err(|e| QmsError::io_error(&e.to_string()))?;
        
        Ok(content.len() as u64)
    }
    
    /// Export to CSV format
    fn export_csv(&self, entries: &[crate::models::AuditEntry], options: &ExportOptions) -> QmsResult<u64> {
        let mut content = String::new();
        
        if options.include_headers {
            content.push_str("ID,Timestamp,User,Action,Entity Type,Entity ID,Old Value,New Value,Details,Checksum,Signature\n");
        }
        
        for entry in entries {
            let old_value = entry.old_value.as_ref().unwrap_or(&"".to_string()).replace('"', "\"\"");
            let new_value = entry.new_value.as_ref().unwrap_or(&"".to_string()).replace('"', "\"\"");
            let details = entry.details.as_ref().unwrap_or(&"".to_string()).replace('"', "\"\"");
            let signature = entry.signature.as_ref().map(|s| format!("{}@{}", s.user_id, s.timestamp)).unwrap_or_else(|| "".to_string());
            
            content.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",\"{:?}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                entry.id,
                entry.timestamp,
                entry.user_id,
                entry.action,
                entry.entity_type,
                entry.entity_id,
                old_value,
                new_value,
                details,
                entry.checksum,
                signature
            ));
        }
        
        fs::write(&options.output_path, content.as_bytes())
            .map_err(|e| QmsError::io_error(&e.to_string()))?;
        
        Ok(content.len() as u64)
    }
    
    /// Export to JSON format
    fn export_json(&self, entries: &[crate::models::AuditEntry], options: &ExportOptions) -> QmsResult<u64> {
        let mut json_content = String::new();
        
        json_content.push_str("{\n");
        
        if options.include_metadata {
            json_content.push_str("  \"metadata\": {\n");
            json_content.push_str(&format!("    \"generated_at\": \"{}\",\n", crate::utils::current_iso8601_timestamp()));
            json_content.push_str(&format!("    \"total_entries\": {},\n", entries.len()));
            json_content.push_str(&format!("    \"project_path\": \"{}\",\n", self.project_path.display()));
            json_content.push_str("    \"export_format\": \"JSON\"\n");
            json_content.push_str("  },\n");
        }
        
        json_content.push_str("  \"audit_entries\": [\n");
        
        for (i, entry) in entries.iter().enumerate() {
            json_content.push_str("    ");
            json_content.push_str(&entry.to_json());
            if i < entries.len() - 1 {
                json_content.push(',');
            }
            json_content.push('\n');
        }
        
        json_content.push_str("  ]\n");
        json_content.push_str("}\n");
        
        fs::write(&options.output_path, json_content.as_bytes())
            .map_err(|e| QmsError::io_error(&e.to_string()))?;
        
        Ok(json_content.len() as u64)
    }
    
    /// Export to XML format
    fn export_xml(&self, entries: &[crate::models::AuditEntry], options: &ExportOptions) -> QmsResult<u64> {
        let mut xml_content = String::new();
        
        xml_content.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml_content.push_str("<audit_report>\n");
        
        if options.include_metadata {
            xml_content.push_str("  <metadata>\n");
            xml_content.push_str(&format!("    <generated_at>{}</generated_at>\n", crate::utils::current_iso8601_timestamp()));
            xml_content.push_str(&format!("    <total_entries>{}</total_entries>\n", entries.len()));
            xml_content.push_str(&format!("    <project_path>{}</project_path>\n", self.project_path.display()));
            xml_content.push_str("    <export_format>XML</export_format>\n");
            xml_content.push_str("  </metadata>\n");
        }
        
        xml_content.push_str("  <audit_entries>\n");
        
        for entry in entries {
            xml_content.push_str("    <entry>\n");
            xml_content.push_str(&format!("      <id>{}</id>\n", entry.id));
            xml_content.push_str(&format!("      <timestamp>{}</timestamp>\n", entry.timestamp));
            xml_content.push_str(&format!("      <user_id>{}</user_id>\n", entry.user_id));
            xml_content.push_str(&format!("      <action>{:?}</action>\n", entry.action));
            xml_content.push_str(&format!("      <entity_type>{}</entity_type>\n", entry.entity_type));
            xml_content.push_str(&format!("      <entity_id>{}</entity_id>\n", entry.entity_id));
            xml_content.push_str(&format!("      <details><![CDATA[{}]]></details>\n", entry.details.as_ref().unwrap_or(&"".to_string())));
            if let Some(old_value) = &entry.old_value {
                xml_content.push_str(&format!("      <old_value><![CDATA[{old_value}]]></old_value>\n"));
            }
            if let Some(new_value) = &entry.new_value {
                xml_content.push_str(&format!("      <new_value><![CDATA[{new_value}]]></new_value>\n"));
            }
            if let Some(signature) = &entry.signature {
                xml_content.push_str(&format!("      <signature>{}@{}</signature>\n", signature.user_id, signature.timestamp));
            }
            xml_content.push_str("    </entry>\n");
        }
        
        xml_content.push_str("  </audit_entries>\n");
        xml_content.push_str("</audit_report>\n");
        
        fs::write(&options.output_path, xml_content.as_bytes())
            .map_err(|e| QmsError::io_error(&e.to_string()))?;
        
        Ok(xml_content.len() as u64)
    }
    
    /// Generate activity summary report
    pub fn generate_activity_summary(&self, options: &ExportOptions) -> QmsResult<ExportStats> {
        let search_criteria = AuditSearchCriteria::new();
        let search_engine = AuditSearchEngine::new(self.project_path.clone());
        let search_results = search_engine.search(&search_criteria)?;
        
        // Calculate activity statistics
        let mut user_activity: HashMap<String, usize> = HashMap::new();
        let mut action_counts: HashMap<String, usize> = HashMap::new();
        let mut entity_counts: HashMap<String, usize> = HashMap::new();
        
        for entry in &search_results.entries {
            *user_activity.entry(entry.user_id.clone()).or_insert(0) += 1;
            *action_counts.entry(format!("{:?}", entry.action)).or_insert(0) += 1;
            *entity_counts.entry(entry.entity_type.clone()).or_insert(0) += 1;
        }
        
        // Generate summary content
        let mut content = String::new();
        content.push_str("AUDIT ACTIVITY SUMMARY REPORT\n");
        content.push_str("==============================\n\n");
        content.push_str(&format!("Generated: {}\n", crate::utils::current_iso8601_timestamp()));
        content.push_str(&format!("Total Audit Entries: {}\n\n", search_results.entries.len()));
        
        content.push_str("USER ACTIVITY:\n");
        for (user, count) in user_activity {
            content.push_str(&format!("  {user}: {count} actions\n"));
        }
        content.push('\n');
        
        content.push_str("ACTION BREAKDOWN:\n");
        for (action, count) in action_counts {
            content.push_str(&format!("  {action}: {count} times\n"));
        }
        content.push('\n');
        
        content.push_str("ENTITY TYPE DISTRIBUTION:\n");
        for (entity_type, count) in entity_counts {
            content.push_str(&format!("  {entity_type}: {count} entries\n"));
        }
        
        fs::write(&options.output_path, content.as_bytes())
            .map_err(|e| QmsError::io_error(&e.to_string()))?;
        
        Ok(ExportStats {
            total_entries: search_results.entries.len(),
            exported_entries: search_results.entries.len(),
            filtered_entries: 0,
            file_size: content.len() as u64,
            processing_time_ms: 0,
            export_format: "Summary Report".to_string(),
        })
    }
    
    /// Generate compliance report
    pub fn generate_compliance_report(&self, options: &ExportOptions) -> QmsResult<ExportStats> {
        // Use regulatory compliance module
        let compliance = crate::modules::audit_logger::RegulatoryCompliance::new(self.project_path.clone());
        let compliance_report = compliance.generate_compliance_report("30-day")?;
        let formatted_report = crate::modules::audit_logger::format_compliance_report(&compliance_report);
        
        fs::write(&options.output_path, formatted_report.as_bytes())
            .map_err(|e| QmsError::io_error(&e.to_string()))?;
        
        Ok(ExportStats {
            total_entries: compliance_report.audit_trail_summary.total_entries,
            exported_entries: compliance_report.audit_trail_summary.total_entries,
            filtered_entries: 0,
            file_size: formatted_report.len() as u64,
            processing_time_ms: 0,
            export_format: "Compliance Report".to_string(),
        })
    }
}

/// Format export statistics for display
pub fn format_export_stats(stats: &ExportStats) -> String {
    format!(
        "Export completed successfully!\n\
        Total entries: {}\n\
        Exported entries: {}\n\
        Filtered entries: {}\n\
        File size: {} bytes\n\
        Processing time: {} ms\n\
        Format: {}",
        stats.total_entries,
        stats.exported_entries,
        stats.filtered_entries,
        stats.file_size,
        stats.processing_time_ms,
        stats.export_format
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_export_format_parsing() {
        assert!(matches!(ExportFormat::from_string("pdf"), Ok(ExportFormat::PDF)));
        assert!(matches!(ExportFormat::from_string("CSV"), Ok(ExportFormat::CSV)));
        assert!(matches!(ExportFormat::from_string("json"), Ok(ExportFormat::JSON)));
        assert!(matches!(ExportFormat::from_string("XML"), Ok(ExportFormat::XML)));
        assert!(ExportFormat::from_string("invalid").is_err());
    }

    #[test]
    fn test_export_options_builder() {
        let options = ExportOptions::new(
            ExportFormat::CSV,
            PathBuf::from("test.csv")
        )
        .with_filter("user:john".to_string())
        .with_max_entries(100);
        
        assert!(matches!(options.format, ExportFormat::CSV));
        assert_eq!(options.filter, Some("user:john".to_string()));
        assert_eq!(options.max_entries, Some(100));
    }

    #[test]
    fn test_export_engine_creation() {
        let engine = AuditExportEngine::new(PathBuf::from("/tmp/test"));
        assert_eq!(engine.project_path, PathBuf::from("/tmp/test"));
    }
}
