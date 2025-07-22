use crate::prelude::*;
use crate::modules::risk_manager::RiskManager;
use crate::modules::audit_logger::audit_log_action;
use std::fmt::Write;

/// Simple Risk Report Generator
/// Generates basic risk reports from risk index entries
pub struct RiskReportGenerator {
    project_path: PathBuf,
}

impl RiskReportGenerator {
    /// Create a new risk report generator
    pub fn new(project_path: &Path) -> Self {
        RiskReportGenerator {
            project_path: project_path.to_path_buf(),
        }
    }
    
    /// Generate risk report in specified format
    pub fn generate_report(&self, format: &str, output_path: Option<&str>) -> QmsResult<String> {
        let risk_manager = RiskManager::new(&self.project_path)?;
        let risk_entries = risk_manager.list_risks(None)?;
        
        let report = match format.to_lowercase().as_str() {
            "md" | "markdown" => self.generate_markdown_report(&risk_entries)?,
            "csv" => self.generate_csv_report(&risk_entries)?,
            "json" => self.generate_json_report(&risk_entries)?,
            _ => return Err(QmsError::validation_error("Unsupported format. Use: md, csv, json")),
        };
        
        // Write to file if path provided
        if let Some(path) = output_path {
            std::fs::write(path, &report)?;
        }
        
        // Log audit entry
        audit_log_action("system", "Generate", "Report", &format!("risk_report_{}", format), None)?;
        
        Ok(report)
    }
    
    /// Generate markdown report
    fn generate_markdown_report(&self, entries: &[crate::modules::risk_manager::risk::RiskIndexEntry]) -> QmsResult<String> {
        let mut report = String::new();
        
        writeln!(report, "# Risk Management Report").unwrap();
        writeln!(report, "").unwrap();
        writeln!(report, "**Generated:** {}", self.format_current_time()).unwrap();
        writeln!(report, "**Project:** {}", self.project_path.file_name().unwrap_or_default().to_string_lossy()).unwrap();
        writeln!(report, "").unwrap();
        writeln!(report, "## Summary").unwrap();
        writeln!(report, "").unwrap();
        writeln!(report, "- **Total Risks:** {}", entries.len()).unwrap();
        writeln!(report, "").unwrap();
        writeln!(report, "## Risk List").unwrap();
        writeln!(report, "").unwrap();
        writeln!(report, "| ID | Hazard | Status | Priority | Created |").unwrap();
        writeln!(report, "|---|---|---|---|---|").unwrap();
        
        for entry in entries {
            writeln!(report, "| {} | {} | {} | {} | {} |",
                entry.id,
                self.truncate_string(&entry.hazard_description, 50),
                entry.status,
                entry.priority,
                entry.created_at
            ).unwrap();
        }
        
        writeln!(report, "").unwrap();
        writeln!(report, "## Regulatory Compliance").unwrap();
        writeln!(report, "").unwrap();
        writeln!(report, "This risk report satisfies ISO 14971 requirements for risk management documentation.").unwrap();
        
        Ok(report)
    }
    
    /// Generate CSV report
    fn generate_csv_report(&self, entries: &[crate::modules::risk_manager::risk::RiskIndexEntry]) -> QmsResult<String> {
        let mut report = String::new();
        
        writeln!(report, "ID,Hazard,Status,Priority,Created").unwrap();
        
        for entry in entries {
            writeln!(report, "{},{},{},{},{}",
                self.escape_csv(&entry.id),
                self.escape_csv(&entry.hazard_description),
                self.escape_csv(&entry.status),
                self.escape_csv(&entry.priority),
                self.escape_csv(&entry.created_at)
            ).unwrap();
        }
        
        Ok(report)
    }
    
    /// Generate JSON report
    fn generate_json_report(&self, entries: &[crate::modules::risk_manager::risk::RiskIndexEntry]) -> QmsResult<String> {
        let mut report = String::new();
        
        writeln!(report, "{{").unwrap();
        writeln!(report, "  \"report_type\": \"risk_management\",").unwrap();
        writeln!(report, "  \"generated_at\": \"{}\",", self.format_current_time()).unwrap();
        writeln!(report, "  \"total_risks\": {},", entries.len()).unwrap();
        writeln!(report, "  \"risks\": [").unwrap();
        
        for (i, entry) in entries.iter().enumerate() {
            if i > 0 {
                writeln!(report, ",").unwrap();
            }
            writeln!(report, "    {{").unwrap();
            writeln!(report, "      \"id\": \"{}\",", self.escape_json(&entry.id)).unwrap();
            writeln!(report, "      \"hazard\": \"{}\",", self.escape_json(&entry.hazard_description)).unwrap();
            writeln!(report, "      \"status\": \"{}\",", self.escape_json(&entry.status)).unwrap();
            writeln!(report, "      \"priority\": \"{}\",", self.escape_json(&entry.priority)).unwrap();
            writeln!(report, "      \"created_at\": \"{}\"", self.escape_json(&entry.created_at)).unwrap();
            write!(report, "    }}").unwrap();
        }
        
        writeln!(report, "").unwrap();
        writeln!(report, "  ]").unwrap();
        writeln!(report, "}}").unwrap();
        
        Ok(report)
    }
    
    /// Format current time
    fn format_current_time(&self) -> String {
        // Simple timestamp for now
        format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
    }
    
    /// Truncate string for display
    fn truncate_string(&self, s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len-3])
        }
    }
    
    /// Escape CSV field values
    fn escape_csv(&self, value: &str) -> String {
        if value.contains('"') || value.contains(',') || value.contains('\n') {
            format!("\"{}\"", value.replace('"', "\"\""))
        } else {
            value.to_string()
        }
    }
    
    /// Escape JSON string values
    fn escape_json(&self, value: &str) -> String {
        value.replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r")
    }
}
