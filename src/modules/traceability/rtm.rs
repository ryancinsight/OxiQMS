#![allow(dead_code)]

use crate::prelude::*;
use crate::modules::traceability::links::{TraceabilityManager, TraceLinkType};
use crate::modules::traceability::requirement::{RequirementManager, Requirement, RequirementStatus};
use crate::modules::traceability::test_case::{TestCaseManager};
use crate::modules::audit_logger::functions::audit_log_create;
use crate::utils::current_timestamp;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Requirements Traceability Matrix (RTM) Generator
/// 
/// This module provides comprehensive RTM generation capabilities for regulatory compliance.
/// It builds on the traceability link management system to create detailed matrices showing
/// relationships between requirements, test cases, design elements, and verification status.
pub struct RTMGenerator {
    project_path: PathBuf,
    trace_manager: TraceabilityManager,
    req_manager: RequirementManager,
    test_manager: TestCaseManager,
}

/// RTM Entry representing a single row in the traceability matrix
#[derive(Debug, Clone)]
pub struct RTMEntry {
    pub requirement_id: String,
    pub requirement_title: String,
    pub requirement_description: String,
    pub requirement_category: String,
    pub requirement_priority: String,
    pub requirement_status: String,
    pub linked_test_cases: Vec<String>,
    pub linked_design_elements: Vec<String>,
    pub linked_risks: Vec<String>,
    pub linked_documents: Vec<String>,
    pub verification_status: String,
    pub verification_method: String,
    pub coverage_percentage: f64,
    pub last_verified_at: Option<String>,
    pub verification_notes: Option<String>,
}

/// RTM Configuration for filtering and formatting
#[derive(Debug, Clone)]
pub struct RTMConfig {
    pub include_categories: Option<Vec<String>>,
    pub include_priorities: Option<Vec<String>>, 
    pub include_statuses: Option<Vec<String>>,
    pub include_verification_statuses: Option<Vec<String>>,
    pub include_test_statuses: Option<Vec<String>>,
    pub show_descriptions: bool,
    pub show_verification_details: bool,
    pub show_coverage_metrics: bool,
    pub sort_by: RTMSortBy,
    pub group_by: Option<RTMGroupBy>,
}

/// RTM sorting options
#[derive(Debug, Clone)]
pub enum RTMSortBy {
    RequirementId,
    RequirementTitle,
    Priority,
    Status,
    VerificationStatus,
    CoveragePercentage,
}

/// RTM grouping options
#[derive(Debug, Clone)]
pub enum RTMGroupBy {
    Category,
    Priority,
    Status,
    VerificationStatus,
}

/// RTM output format options
#[derive(Debug, Clone)]
pub enum RTMFormat {
    CSV,
    JSON,
    HTML,
    PDF,
    Markdown,
}

/// RTM statistics and metrics
#[derive(Debug, Clone)]
pub struct RTMStats {
    pub total_requirements: usize,
    pub total_test_cases: usize,
    pub total_links: usize,
    pub requirements_with_tests: usize,
    pub requirements_without_tests: usize,
    pub test_cases_with_requirements: usize,
    pub orphaned_test_cases: usize,
    pub verification_coverage: f64,
    pub category_breakdown: HashMap<String, usize>,
    pub priority_breakdown: HashMap<String, usize>,
    pub status_breakdown: HashMap<String, usize>,
    pub verification_status_breakdown: HashMap<String, usize>,
}

impl RTMGenerator {
    /// Create a new RTM generator for the specified project
    pub fn new(project_path: &Path) -> QmsResult<Self> {
        let trace_manager = TraceabilityManager::new(project_path)?;
        let req_manager = RequirementManager::new(project_path)?;
        let test_manager = TestCaseManager::new(project_path)?;
        
        Ok(RTMGenerator {
            project_path: project_path.to_path_buf(),
            trace_manager,
            req_manager,
            test_manager,
        })
    }

    /// Generate a complete RTM with all entries
    pub fn generate_rtm(&mut self, config: &RTMConfig) -> QmsResult<Vec<RTMEntry>> {
        // Load all requirements
        let requirements = self.req_manager.list_requirements();
        
        // Load all traceability links
        let links = self.trace_manager.get_trace_links()?;
        
        // Build RTM entries
        let mut rtm_entries = Vec::new();
        
        for req in requirements {
            // Apply filters
            if !self.matches_filters(req, config) {
                continue;
            }
            
            // Find all links for this requirement
            let req_links = links.iter()
                .filter(|link| link.source_id == req.req_id || link.target_id == req.req_id)
                .collect::<Vec<_>>();
            
            // Build RTM entry
            let rtm_entry = RTMEntry {
                requirement_id: req.req_id.clone(),
                requirement_title: req.title.clone(),
                requirement_description: req.description.clone(),
                requirement_category: req.category.as_str().to_string(),
                requirement_priority: req.priority.as_str().to_string(),
                requirement_status: req.status.as_str().to_string(),
                linked_test_cases: req_links.iter()
                    .filter_map(|link| {
                        let target_id = if link.source_id == req.req_id {
                            &link.target_id
                        } else {
                            &link.source_id
                        };
                        if target_id.starts_with("TC-") {
                            Some(target_id.clone())
                        } else {
                            None
                        }
                    })
                    .collect(),
                linked_design_elements: req_links.iter()
                    .filter_map(|link| {
                        let target_id = if link.source_id == req.req_id {
                            &link.target_id
                        } else {
                            &link.source_id
                        };
                        if target_id.starts_with("DESIGN-") {
                            Some(target_id.clone())
                        } else {
                            None
                        }
                    })
                    .collect(),
                linked_risks: req_links.iter()
                    .filter_map(|link| {
                        let target_id = if link.source_id == req.req_id {
                            &link.target_id
                        } else {
                            &link.source_id
                        };
                        if target_id.starts_with("RISK-") {
                            Some(target_id.clone())
                        } else {
                            None
                        }
                    })
                    .collect(),
                linked_documents: req_links.iter()
                    .filter_map(|link| {
                        let target_id = if link.source_id == req.req_id {
                            &link.target_id
                        } else {
                            &link.source_id
                        };
                        if target_id.starts_with("DOC-") {
                            Some(target_id.clone())
                        } else {
                            None
                        }
                    })
                    .collect(),
                verification_status: if matches!(req.status, RequirementStatus::Verified | RequirementStatus::Validated) {
                    "Verified".to_string()
                } else {
                    "Not Verified".to_string()
                },
                verification_method: req.verification_method.as_str().to_string(),
                coverage_percentage: if req_links.iter().any(|link| {
                    let target_id = if link.source_id == req.req_id {
                        &link.target_id
                    } else {
                        &link.source_id
                    };
                    target_id.starts_with("TC-")
                }) {
                    100.0
                } else {
                    0.0
                },
                last_verified_at: None,
                verification_notes: None,
            };
            
            rtm_entries.push(rtm_entry);
        }
        
        // Sort entries according to configuration
        self.sort_rtm_entries(&mut rtm_entries, &config.sort_by);
        
        // Log RTM generation
        audit_log_create(
            "RTM",
            "GENERATE",
            &format!("Generated RTM with {} entries", rtm_entries.len())
        )?;
        
        Ok(rtm_entries)
    }

    /// Generate RTM and export to specified format
    pub fn generate_and_export(&mut self, config: &RTMConfig, format: RTMFormat, output_path: &Path) -> QmsResult<()> {
        let rtm_entries = self.generate_rtm(config)?;
        
        match format {
            RTMFormat::CSV => self.export_csv(&rtm_entries, output_path, config)?,
            RTMFormat::JSON => self.export_json(&rtm_entries, output_path, config)?,
            RTMFormat::HTML => self.export_html(&rtm_entries, output_path, config)?,
            RTMFormat::PDF => self.export_pdf(&rtm_entries, output_path, config)?,
            RTMFormat::Markdown => self.export_markdown(&rtm_entries, output_path, config)?,
        }
        
        // Log export
        audit_log_create(
            "RTM",
            "EXPORT",
            &format!("Exported RTM to {} in format {:?}", output_path.display(), format)
        )?;
        
        Ok(())
    }

    /// Generate RTM statistics and metrics
    pub fn generate_stats(&mut self) -> QmsResult<RTMStats> {
        let requirements = self.req_manager.list_requirements();
        let test_cases = self.test_manager.list_test_cases();
        let links = self.trace_manager.get_trace_links()?;
        
        let mut stats = RTMStats {
            total_requirements: requirements.len(),
            total_test_cases: test_cases.len(),
            total_links: links.len(),
            requirements_with_tests: 0,
            requirements_without_tests: 0,
            test_cases_with_requirements: 0,
            orphaned_test_cases: 0,
            verification_coverage: 0.0,
            category_breakdown: HashMap::new(),
            priority_breakdown: HashMap::new(),
            status_breakdown: HashMap::new(),
            verification_status_breakdown: HashMap::new(),
        };
        
        // Count requirements with/without tests
        for req in &requirements {
            let has_test = links.iter().any(|link| {
                (link.source_id == req.req_id && link.target_id.starts_with("TC-") && 
                 matches!(link.link_type, TraceLinkType::Verifies)) ||
                (link.target_id == req.req_id && link.source_id.starts_with("TC-") && 
                 matches!(link.link_type, TraceLinkType::Verifies))
            });
            
            if has_test {
                stats.requirements_with_tests += 1;
            } else {
                stats.requirements_without_tests += 1;
            }
            
            // Category breakdown
            *stats.category_breakdown.entry(req.category.as_str().to_string()).or_insert(0) += 1;
            
            // Priority breakdown
            *stats.priority_breakdown.entry(req.priority.as_str().to_string()).or_insert(0) += 1;
            
            // Status breakdown
            *stats.status_breakdown.entry(req.status.as_str().to_string()).or_insert(0) += 1;
            
            // Verification status breakdown
            let verification_status = if matches!(req.status, RequirementStatus::Verified | RequirementStatus::Validated) {
                "Verified".to_string()
            } else {
                "Not Verified".to_string()
            };
            *stats.verification_status_breakdown.entry(verification_status).or_insert(0) += 1;
        }
        
        // Count test cases with/without requirements
        for test in &test_cases {
            let has_requirement = links.iter().any(|link| {
                (link.source_id == test.test_id && link.target_id.starts_with("REQ-") && 
                 matches!(link.link_type, TraceLinkType::Verifies)) ||
                (link.target_id == test.test_id && link.source_id.starts_with("REQ-") && 
                 matches!(link.link_type, TraceLinkType::Verifies))
            });
            
            if has_requirement {
                stats.test_cases_with_requirements += 1;
            } else {
                stats.orphaned_test_cases += 1;
            }
        }
        
        // Calculate verification coverage
        if stats.total_requirements > 0 {
            stats.verification_coverage = (stats.requirements_with_tests as f64 / stats.total_requirements as f64) * 100.0;
        }
        
        Ok(stats)
    }

    /// Check if requirement matches the configured filters
    fn matches_filters(&self, req: &Requirement, config: &RTMConfig) -> bool {
        // Category filter
        if let Some(ref categories) = config.include_categories {
            if !categories.contains(&req.category.as_str().to_string()) {
                return false;
            }
        }
        
        // Priority filter
        if let Some(ref priorities) = config.include_priorities {
            if !priorities.contains(&req.priority.as_str().to_string()) {
                return false;
            }
        }
        
        // Status filter
        if let Some(ref statuses) = config.include_statuses {
            if !statuses.contains(&req.status.as_str().to_string()) {
                return false;
            }
        }
        
        // Verification status filter
        if let Some(ref verification_statuses) = config.include_verification_statuses {
            let verification_status = if matches!(req.status, RequirementStatus::Verified | RequirementStatus::Validated) {
                "Verified"
            } else {
                "Not Verified"
            };
            if !verification_statuses.contains(&verification_status.to_string()) {
                return false;
            }
        }
        
        true
    }

    /// Sort RTM entries according to specified criteria
    fn sort_rtm_entries(&self, entries: &mut Vec<RTMEntry>, sort_by: &RTMSortBy) {
        match sort_by {
            RTMSortBy::RequirementId => {
                entries.sort_by(|a, b| a.requirement_id.cmp(&b.requirement_id));
            }
            RTMSortBy::RequirementTitle => {
                entries.sort_by(|a, b| a.requirement_title.cmp(&b.requirement_title));
            }
            RTMSortBy::Priority => {
                entries.sort_by(|a, b| {
                    let priority_order = |p: &str| match p {
                        "Critical" => 0,
                        "High" => 1,
                        "Medium" => 2,
                        "Low" => 3,
                        _ => 4,
                    };
                    priority_order(&a.requirement_priority).cmp(&priority_order(&b.requirement_priority))
                });
            }
            RTMSortBy::Status => {
                entries.sort_by(|a, b| a.requirement_status.cmp(&b.requirement_status));
            }
            RTMSortBy::VerificationStatus => {
                entries.sort_by(|a, b| a.verification_status.cmp(&b.verification_status));
            }
            RTMSortBy::CoveragePercentage => {
                entries.sort_by(|a, b| b.coverage_percentage.partial_cmp(&a.coverage_percentage).unwrap_or(std::cmp::Ordering::Equal));
            }
        }
    }

    /// Export RTM as CSV
    fn export_csv(&self, entries: &[RTMEntry], output_path: &Path, config: &RTMConfig) -> QmsResult<()> {
        let mut csv_content = String::new();
        
        // CSV header
        csv_content.push_str("Requirement ID,Title,");
        if config.show_descriptions {
            csv_content.push_str("Description,");
        }
        csv_content.push_str("Category,Priority,Status,");
        csv_content.push_str("Test Cases,Design Elements,Risks,Documents,");
        csv_content.push_str("Verification Status,Verification Method,");
        if config.show_coverage_metrics {
            csv_content.push_str("Coverage %,");
        }
        if config.show_verification_details {
            csv_content.push_str("Last Verified,Notes,");
        }
        csv_content.push('\n');
        
        // CSV rows
        for entry in entries {
            csv_content.push_str(&format!("\"{}\",\"{}\"", 
                entry.requirement_id, 
                entry.requirement_title.replace("\"", "\"\"")
            ));
            
            if config.show_descriptions {
                csv_content.push_str(&format!(",\"{}\"", 
                    entry.requirement_description.replace("\"", "\"\"")
                ));
            }
            
            csv_content.push_str(&format!(",\"{}\",\"{}\",\"{}\"", 
                entry.requirement_category, 
                entry.requirement_priority, 
                entry.requirement_status
            ));
            
            csv_content.push_str(&format!(",\"{}\",\"{}\",\"{}\",\"{}\"", 
                entry.linked_test_cases.join("; "), 
                entry.linked_design_elements.join("; "), 
                entry.linked_risks.join("; "), 
                entry.linked_documents.join("; ")
            ));
            
            csv_content.push_str(&format!(",\"{}\",\"{}\"", 
                entry.verification_status, 
                entry.verification_method
            ));
            
            if config.show_coverage_metrics {
                csv_content.push_str(&format!(",{:.1}", entry.coverage_percentage));
            }
            
            if config.show_verification_details {
                csv_content.push_str(&format!(",\"{}\",\"{}\"", 
                    entry.last_verified_at.as_deref().unwrap_or(""),
                    entry.verification_notes.as_deref().unwrap_or("")
                ));
            }
            
            csv_content.push('\n');
        }
        
        fs::write(output_path, csv_content)?;
        Ok(())
    }

    /// Export RTM as JSON
    fn export_json(&self, entries: &[RTMEntry], output_path: &Path, _config: &RTMConfig) -> QmsResult<()> {
        let mut json_content = String::new();
        json_content.push_str("{\n");
        json_content.push_str("  \"version\": \"1.0\",\n");
        json_content.push_str(&format!("  \"generated_at\": \"{}\",\n", current_timestamp()));
        json_content.push_str(&format!("  \"total_entries\": {},\n", entries.len()));
        json_content.push_str("  \"entries\": [\n");
        
        for (i, entry) in entries.iter().enumerate() {
            json_content.push_str("    {\n");
            json_content.push_str(&format!("      \"requirement_id\": \"{}\",\n", entry.requirement_id));
            json_content.push_str(&format!("      \"requirement_title\": \"{}\",\n", entry.requirement_title.replace("\"", "\\\"")));
            json_content.push_str(&format!("      \"requirement_description\": \"{}\",\n", entry.requirement_description.replace("\"", "\\\"")));
            json_content.push_str(&format!("      \"requirement_category\": \"{}\",\n", entry.requirement_category));
            json_content.push_str(&format!("      \"requirement_priority\": \"{}\",\n", entry.requirement_priority));
            json_content.push_str(&format!("      \"requirement_status\": \"{}\",\n", entry.requirement_status));
            json_content.push_str("      \"linked_test_cases\": [");
            for (j, tc) in entry.linked_test_cases.iter().enumerate() {
                if j > 0 { json_content.push_str(", "); }
                json_content.push_str(&format!("\"{tc}\""));
            }
            json_content.push_str("],\n");
            json_content.push_str("      \"linked_design_elements\": [");
            for (j, de) in entry.linked_design_elements.iter().enumerate() {
                if j > 0 { json_content.push_str(", "); }
                json_content.push_str(&format!("\"{de}\""));
            }
            json_content.push_str("],\n");
            json_content.push_str("      \"linked_risks\": [");
            for (j, risk) in entry.linked_risks.iter().enumerate() {
                if j > 0 { json_content.push_str(", "); }
                json_content.push_str(&format!("\"{risk}\""));
            }
            json_content.push_str("],\n");
            json_content.push_str("      \"linked_documents\": [");
            for (j, doc) in entry.linked_documents.iter().enumerate() {
                if j > 0 { json_content.push_str(", "); }
                json_content.push_str(&format!("\"{doc}\""));
            }
            json_content.push_str("],\n");
            json_content.push_str(&format!("      \"verification_status\": \"{}\",\n", entry.verification_status));
            json_content.push_str(&format!("      \"verification_method\": \"{}\",\n", entry.verification_method));
            json_content.push_str(&format!("      \"coverage_percentage\": {:.1}\n", entry.coverage_percentage));
            json_content.push_str("    }");
            
            if i < entries.len() - 1 {
                json_content.push(',');
            }
            json_content.push('\n');
        }
        
        json_content.push_str("  ]\n");
        json_content.push_str("}\n");
        
        fs::write(output_path, json_content)?;
        Ok(())
    }

    /// Export RTM as HTML
    fn export_html(&self, entries: &[RTMEntry], output_path: &Path, config: &RTMConfig) -> QmsResult<()> {
        let mut html_content = String::new();
        
        // HTML header
        html_content.push_str("<!DOCTYPE html>\n");
        html_content.push_str("<html>\n");
        html_content.push_str("<head>\n");
        html_content.push_str("  <title>Requirements Traceability Matrix</title>\n");
        html_content.push_str("  <style>\n");
        html_content.push_str("    body { font-family: Arial, sans-serif; margin: 20px; }\n");
        html_content.push_str("    table { border-collapse: collapse; width: 100%; }\n");
        html_content.push_str("    th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
        html_content.push_str("    th { background-color: #f2f2f2; }\n");
        html_content.push_str("    tr:nth-child(even) { background-color: #f9f9f9; }\n");
        html_content.push_str("    .verified { color: green; }\n");
        html_content.push_str("    .not-verified { color: red; }\n");
        html_content.push_str("    .partially-verified { color: orange; }\n");
        html_content.push_str("  </style>\n");
        html_content.push_str("</head>\n");
        html_content.push_str("<body>\n");
        html_content.push_str("  <h1>Requirements Traceability Matrix</h1>\n");
        html_content.push_str(&format!("  <p>Generated: {}</p>\n", current_timestamp()));
        html_content.push_str(&format!("  <p>Total Requirements: {}</p>\n", entries.len()));
        html_content.push_str("  <table>\n");
        html_content.push_str("    <tr>\n");
        html_content.push_str("      <th>Requirement ID</th>\n");
        html_content.push_str("      <th>Title</th>\n");
        if config.show_descriptions {
            html_content.push_str("      <th>Description</th>\n");
        }
        html_content.push_str("      <th>Category</th>\n");
        html_content.push_str("      <th>Priority</th>\n");
        html_content.push_str("      <th>Status</th>\n");
        html_content.push_str("      <th>Test Cases</th>\n");
        html_content.push_str("      <th>Design Elements</th>\n");
        html_content.push_str("      <th>Risks</th>\n");
        html_content.push_str("      <th>Documents</th>\n");
        html_content.push_str("      <th>Verification Status</th>\n");
        html_content.push_str("      <th>Verification Method</th>\n");
        if config.show_coverage_metrics {
            html_content.push_str("      <th>Coverage %</th>\n");
        }
        html_content.push_str("    </tr>\n");
        
        // HTML rows
        for entry in entries {
            html_content.push_str("    <tr>\n");
            html_content.push_str(&format!("      <td>{}</td>\n", entry.requirement_id));
            html_content.push_str(&format!("      <td>{}</td>\n", entry.requirement_title));
            if config.show_descriptions {
                html_content.push_str(&format!("      <td>{}</td>\n", entry.requirement_description));
            }
            html_content.push_str(&format!("      <td>{}</td>\n", entry.requirement_category));
            html_content.push_str(&format!("      <td>{}</td>\n", entry.requirement_priority));
            html_content.push_str(&format!("      <td>{}</td>\n", entry.requirement_status));
            html_content.push_str(&format!("      <td>{}</td>\n", entry.linked_test_cases.join(", ")));
            html_content.push_str(&format!("      <td>{}</td>\n", entry.linked_design_elements.join(", ")));
            html_content.push_str(&format!("      <td>{}</td>\n", entry.linked_risks.join(", ")));
            html_content.push_str(&format!("      <td>{}</td>\n", entry.linked_documents.join(", ")));
            
            let status_class = match entry.verification_status.as_str() {
                "Verified" => "verified",
                "Not Verified" => "not-verified",
                "Partially Verified" => "partially-verified",
                _ => "",
            };
            html_content.push_str(&format!("      <td class=\"{}\">{}</td>\n", status_class, entry.verification_status));
            html_content.push_str(&format!("      <td>{}</td>\n", entry.verification_method));
            if config.show_coverage_metrics {
                html_content.push_str(&format!("      <td>{:.1}%</td>\n", entry.coverage_percentage));
            }
            html_content.push_str("    </tr>\n");
        }
        
        html_content.push_str("  </table>\n");
        html_content.push_str("</body>\n");
        html_content.push_str("</html>\n");
        
        fs::write(output_path, html_content)?;
        Ok(())
    }

    /// Export RTM as PDF (text-based format for stdlib-only implementation)
    fn export_pdf(&self, entries: &[RTMEntry], output_path: &Path, config: &RTMConfig) -> QmsResult<()> {
        let mut content = String::new();
        let line_width = 80;

        // PDF Header
        content.push_str(&"=".repeat(line_width));
        content.push('\n');
        content.push_str(&Self::center_text("REQUIREMENTS TRACEABILITY MATRIX", line_width));
        content.push('\n');
        content.push_str(&"=".repeat(line_width));
        content.push('\n');
        content.push('\n');

        // Metadata
        content.push_str(&format!("Generated: {}\n", crate::utils::current_timestamp_string()));
        content.push_str(&format!("Total Entries: {}\n", entries.len()));
        content.push_str("Project: Current Project\n"); // RTMConfig doesn't have project_id field
        content.push('\n');
        content.push_str(&"-".repeat(line_width));
        content.push('\n');
        content.push('\n');

        // RTM Entries
        for (index, entry) in entries.iter().enumerate() {
            content.push_str(&format!("Entry {}: {}\n", index + 1, entry.requirement_id));
            content.push_str(&"-".repeat(40));
            content.push('\n');

            content.push_str(&format!("Requirement: {}\n", entry.requirement_description));
            content.push_str(&format!("Category: {}\n", entry.requirement_category));
            content.push_str(&format!("Priority: {}\n", entry.requirement_priority));
            content.push_str(&format!("Status: {}\n", entry.requirement_status));

            if !entry.linked_test_cases.is_empty() {
                content.push_str("Linked Test Cases:\n");
                for test_case in &entry.linked_test_cases {
                    content.push_str(&format!("  - {}\n", test_case));
                }
            }

            if !entry.linked_design_elements.is_empty() {
                content.push_str("Linked Design Elements:\n");
                for design in &entry.linked_design_elements {
                    content.push_str(&format!("  - {}\n", design));
                }
            }

            if !entry.verification_method.is_empty() {
                content.push_str(&format!("Verification Method: {}\n", entry.verification_method));
                content.push_str(&format!("Verification Status: {}\n", entry.verification_status));
            }

            content.push('\n');
        }

        // Summary
        content.push_str(&"=".repeat(line_width));
        content.push('\n');
        content.push_str(&Self::center_text("SUMMARY", line_width));
        content.push('\n');
        content.push_str(&"=".repeat(line_width));
        content.push('\n');
        content.push('\n');

        let total_requirements = entries.len();
        let tested_requirements = entries.iter().filter(|e| !e.linked_test_cases.is_empty()).count();
        let verified_requirements = entries.iter().filter(|e| !e.verification_method.is_empty()).count();

        content.push_str(&format!("Total Requirements: {}\n", total_requirements));
        content.push_str(&format!("Requirements with Tests: {} ({:.1}%)\n",
            tested_requirements,
            if total_requirements > 0 { (tested_requirements as f64 / total_requirements as f64) * 100.0 } else { 0.0 }
        ));
        content.push_str(&format!("Requirements with Verification: {} ({:.1}%)\n",
            verified_requirements,
            if total_requirements > 0 { (verified_requirements as f64 / total_requirements as f64) * 100.0 } else { 0.0 }
        ));

        content.push('\n');
        content.push_str("Note: This is a text-based PDF format. For graphical PDF output,\n");
        content.push_str("convert this file using external tools like pandoc or wkhtmltopdf.\n");

        // Write to file
        std::fs::write(output_path, content)?;

        Ok(())
    }

    /// Center text within a given width
    fn center_text(text: &str, width: usize) -> String {
        if text.len() >= width {
            return text.to_string();
        }

        let padding = (width - text.len()) / 2;
        format!("{}{}", " ".repeat(padding), text)
    }

    /// Export RTM as Markdown
    fn export_markdown(&self, entries: &[RTMEntry], output_path: &Path, config: &RTMConfig) -> QmsResult<()> {
        let mut md_content = String::new();
        
        // Markdown header
        md_content.push_str("# Requirements Traceability Matrix\n\n");
        md_content.push_str(&format!("**Generated:** {}\n", current_timestamp()));
        md_content.push_str(&format!("**Total Requirements:** {}\n\n", entries.len()));
        
        // Markdown table header
        md_content.push_str("| Requirement ID | Title |");
        if config.show_descriptions {
            md_content.push_str(" Description |");
        }
        md_content.push_str(" Category | Priority | Status | Test Cases | Design Elements | Risks | Documents | Verification Status | Verification Method |");
        if config.show_coverage_metrics {
            md_content.push_str(" Coverage % |");
        }
        md_content.push('\n');
        
        // Table separator
        md_content.push_str("|---|---|");
        if config.show_descriptions {
            md_content.push_str("---|");
        }
        md_content.push_str("---|---|---|---|---|---|---|---|---|");
        if config.show_coverage_metrics {
            md_content.push_str("---|");
        }
        md_content.push('\n');
        
        // Markdown rows
        for entry in entries {
            md_content.push_str(&format!("| {} | {} |", 
                entry.requirement_id, 
                entry.requirement_title.replace("|", "\\|")
            ));
            
            if config.show_descriptions {
                md_content.push_str(&format!(" {} |", 
                    entry.requirement_description.replace("|", "\\|")
                ));
            }
            
            md_content.push_str(&format!(" {} | {} | {} | {} | {} | {} | {} | {} | {} |", 
                entry.requirement_category, 
                entry.requirement_priority, 
                entry.requirement_status,
                entry.linked_test_cases.join(", "),
                entry.linked_design_elements.join(", "),
                entry.linked_risks.join(", "),
                entry.linked_documents.join(", "),
                entry.verification_status,
                entry.verification_method
            ));
            
            if config.show_coverage_metrics {
                md_content.push_str(&format!(" {:.1}% |", entry.coverage_percentage));
            }
            
            md_content.push('\n');
        }
        
        fs::write(output_path, md_content)?;
        Ok(())
    }
}

impl Default for RTMConfig {
    fn default() -> Self {
        RTMConfig {
            include_categories: None,
            include_priorities: None,
            include_statuses: None,
            include_verification_statuses: None,
            include_test_statuses: None,
            show_descriptions: false,
            show_verification_details: false,
            show_coverage_metrics: true,
            sort_by: RTMSortBy::RequirementId,
            group_by: None,
        }
    }
}

impl RTMFormat {
    /// Parse format string into RTMFormat enum
    pub fn from_str(s: &str) -> QmsResult<Self> {
        match s.to_lowercase().as_str() {
            "csv" => Ok(RTMFormat::CSV),
            "json" => Ok(RTMFormat::JSON),
            "html" => Ok(RTMFormat::HTML),
            "pdf" => Ok(RTMFormat::PDF),
            "markdown" | "md" => Ok(RTMFormat::Markdown),
            _ => Err(QmsError::validation_error(&format!("Invalid RTM format: {s}"))),
        }
    }
}

impl RTMSortBy {
    /// Parse sort string into RTMSortBy enum
    pub fn from_str(s: &str) -> QmsResult<Self> {
        match s.to_lowercase().as_str() {
            "id" | "requirement_id" => Ok(RTMSortBy::RequirementId),
            "title" | "requirement_title" => Ok(RTMSortBy::RequirementTitle),
            "priority" => Ok(RTMSortBy::Priority),
            "status" => Ok(RTMSortBy::Status),
            "verification_status" => Ok(RTMSortBy::VerificationStatus),
            "coverage" | "coverage_percentage" => Ok(RTMSortBy::CoveragePercentage),
            _ => Err(QmsError::validation_error(&format!("Invalid RTM sort criteria: {s}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_rtm_generator_creation() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Create necessary directories
        fs::create_dir_all(project_path.join("trace")).unwrap();
        fs::create_dir_all(project_path.join("audit")).unwrap();
        
        // Create empty files
        fs::write(project_path.join("trace/requirements.json"), "[]").unwrap();
        fs::write(project_path.join("trace/test_cases.json"), "[]").unwrap();
        fs::write(project_path.join("trace/links.json"), "[]").unwrap();
        fs::write(project_path.join("audit/audit.log"), "").unwrap();
        
        let result = RTMGenerator::new(project_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rtm_config_default() {
        let config = RTMConfig::default();
        assert!(config.include_categories.is_none());
        assert!(config.include_priorities.is_none());
        assert!(config.include_statuses.is_none());
        assert!(config.show_descriptions == false);
        assert!(config.show_coverage_metrics == true);
        assert!(matches!(config.sort_by, RTMSortBy::RequirementId));
    }

    #[test]
    fn test_rtm_format_parsing() {
        assert!(matches!(RTMFormat::from_str("csv").unwrap(), RTMFormat::CSV));
        assert!(matches!(RTMFormat::from_str("JSON").unwrap(), RTMFormat::JSON));
        assert!(matches!(RTMFormat::from_str("html").unwrap(), RTMFormat::HTML));
        assert!(matches!(RTMFormat::from_str("markdown").unwrap(), RTMFormat::Markdown));
        assert!(matches!(RTMFormat::from_str("md").unwrap(), RTMFormat::Markdown));
        assert!(RTMFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_rtm_sort_parsing() {
        assert!(matches!(RTMSortBy::from_str("id").unwrap(), RTMSortBy::RequirementId));
        assert!(matches!(RTMSortBy::from_str("title").unwrap(), RTMSortBy::RequirementTitle));
        assert!(matches!(RTMSortBy::from_str("priority").unwrap(), RTMSortBy::Priority));
        assert!(matches!(RTMSortBy::from_str("status").unwrap(), RTMSortBy::Status));
        assert!(matches!(RTMSortBy::from_str("verification_status").unwrap(), RTMSortBy::VerificationStatus));
        assert!(matches!(RTMSortBy::from_str("coverage").unwrap(), RTMSortBy::CoveragePercentage));
        assert!(RTMSortBy::from_str("invalid").is_err());
    }

    #[test]
    fn test_rtm_stats_structure() {
        let stats = RTMStats {
            total_requirements: 10,
            total_test_cases: 8,
            total_links: 15,
            requirements_with_tests: 7,
            requirements_without_tests: 3,
            test_cases_with_requirements: 8,
            orphaned_test_cases: 0,
            verification_coverage: 70.0,
            category_breakdown: HashMap::new(),
            priority_breakdown: HashMap::new(),
            status_breakdown: HashMap::new(),
            verification_status_breakdown: HashMap::new(),
        };
        
        assert_eq!(stats.total_requirements, 10);
        assert_eq!(stats.verification_coverage, 70.0);
        assert_eq!(stats.orphaned_test_cases, 0);
    }

    #[test]
    fn test_rtm_entry_structure() {
        let entry = RTMEntry {
            requirement_id: "REQ-001".to_string(),
            requirement_title: "Test Requirement".to_string(),
            requirement_description: "Test Description".to_string(),
            requirement_category: "Functional".to_string(),
            requirement_priority: "High".to_string(),
            requirement_status: "Approved".to_string(),
            linked_test_cases: vec!["TC-001".to_string()],
            linked_design_elements: vec!["DESIGN-001".to_string()],
            linked_risks: vec![],
            linked_documents: vec![],
            verification_status: "Verified".to_string(),
            verification_method: "Test".to_string(),
            coverage_percentage: 100.0,
            last_verified_at: None,
            verification_notes: None,
        };
        
        assert_eq!(entry.requirement_id, "REQ-001");
        assert_eq!(entry.coverage_percentage, 100.0);
        assert_eq!(entry.linked_test_cases.len(), 1);
    }
}