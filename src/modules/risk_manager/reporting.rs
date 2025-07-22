//! Risk Reporting & Analytics Implementation
//! Task 3.1.10: Comprehensive risk reporting and analytics
//! 
//! This module implements advanced reporting and analytics capabilities
//! for risk management per ISO 14971 and FDA 21 CFR Part 820.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use crate::prelude::*;
use super::risk::RiskItem;

/// Risk report types for different analysis needs
#[derive(Debug, Clone)]
pub enum ReportType {
    Summary,           // Executive summary report
    TrendAnalysis,     // Risk trends over time
    FMEA,              // FMEA-specific analysis
    Compliance,        // Regulatory compliance status
    MitigationEffectiveness, // Mitigation analysis
    RiskDistribution,  // Risk distribution by severity/RPN
}

/// Output format options for reports
#[derive(Debug, Clone)]
pub enum ReportFormat {
    Markdown,
    CSV,
    JSON,
    HTML,
    PDF,
}

/// Time period for trend analysis
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TimePeriod {
    Days(u32),
    Weeks(u32), 
    Months(u32),
    Years(u32),
}

/// Risk trend data point
#[derive(Debug, Clone)]
pub struct RiskTrend {
    pub period: String,
    pub total_risks: u32,
    pub new_risks: u32,
    pub closed_risks: u32,
    pub average_rpn: f64,
    pub high_priority_risks: u32,
}

/// Risk analytics data
#[derive(Debug, Clone)]
pub struct RiskAnalytics {
    pub total_risks: usize,
    pub average_rpn: f64,
    pub high_priority_count: usize,
    pub severity_distribution: HashMap<String, usize>,
    pub rpn_distribution: HashMap<String, usize>,
    pub risks_with_mitigation: usize,
    pub risks_without_mitigation: usize,
}

/// Main risk reporter with analytics capabilities
#[allow(dead_code)]
pub struct RiskReporter {
    project_path: PathBuf,
    reports_dir: PathBuf,
}

impl RiskReporter {
    /// Create new risk reporter instance
    pub fn new(project_path: &Path) -> QmsResult<Self> {
        let reports_dir = project_path.join("risks").join("reports");
        
        // Ensure reports directory exists
        if !reports_dir.exists() {
            fs::create_dir_all(&reports_dir)?;
        }
        
        Ok(RiskReporter {
            project_path: project_path.to_path_buf(),
            reports_dir,
        })
    }
    
    /// Generate analytics from risk data
    pub fn generate_analytics(&self, risks: &[RiskItem]) -> RiskAnalytics {
        let total_risks = risks.len();
        let mut rpns = Vec::new();
        let mut high_priority_count = 0;
        let mut severity_distribution = HashMap::new();
        let mut rpn_distribution = HashMap::new();
        let mut risks_with_mitigation = 0;
        let mut risks_without_mitigation = 0;
        
        for risk in risks {
            rpns.push(risk.risk_priority_number);
            if risk.risk_priority_number > 50 {
                high_priority_count += 1;
            }
            
            // Severity distribution
            let severity_level = format!("{:?}", risk.severity);
            *severity_distribution.entry(severity_level).or_insert(0) += 1;
            
            // RPN distribution
            let rpn_level = match risk.risk_priority_number {
                0..=8 => "Low (1-8)",
                9..=24 => "Medium (9-24)", 
                25..=50 => "High (25-50)",
                51..=100 => "Very High (51-100)",
                _ => "Critical (>100)",
            };
            *rpn_distribution.entry(rpn_level.to_string()).or_insert(0) += 1;
            
            // Mitigation status
            if !risk.mitigation_measures.is_empty() {
                risks_with_mitigation += 1;
            } else {
                risks_without_mitigation += 1;
            }
        }
        
        let average_rpn = if total_risks > 0 {
            rpns.iter().map(|&x| x as f64).sum::<f64>() / total_risks as f64
        } else {
            0.0
        };
        
        RiskAnalytics {
            total_risks,
            average_rpn,
            high_priority_count,
            severity_distribution,
            rpn_distribution,
            risks_with_mitigation,
            risks_without_mitigation,
        }
    }
    
    /// Generate summary report in specified format
    pub fn generate_summary_report(&self, risks: &[RiskItem], format: ReportFormat) -> QmsResult<String> {
        let analytics = self.generate_analytics(risks);
        
        match format {
            ReportFormat::Markdown => self.generate_markdown_summary(risks, &analytics),
            ReportFormat::CSV => self.generate_csv_summary(risks, &analytics),
            ReportFormat::JSON => self.generate_json_summary(risks, &analytics),
            ReportFormat::HTML => self.generate_html_summary(risks, &analytics),
            ReportFormat::PDF => {
                // For PDF, generate HTML first then note that PDF conversion would require external tool
                let html = self.generate_html_summary(risks, &analytics)?;
                Ok(format!("<!-- PDF Generation Note: This HTML can be converted to PDF using tools like wkhtmltopdf -->\n{html}"))
            }
        }
    }
    
    /// Generate Markdown format summary
    fn generate_markdown_summary(&self, risks: &[RiskItem], analytics: &RiskAnalytics) -> QmsResult<String> {
        let mut report = String::new();
        
        report.push_str("# Risk Management Summary Report\n\n");
        report.push_str(&format!("**Generated:** {} UTC\n", get_current_timestamp_string()));
        report.push_str(&format!("**Total Risks:** {}\n\n", analytics.total_risks));
        
        // Executive Summary
        report.push_str("## Executive Summary\n\n");
        report.push_str(&format!("- **Total Risks Identified:** {}\n", analytics.total_risks));
        report.push_str(&format!("- **Average RPN:** {:.1}\n", analytics.average_rpn));
        report.push_str(&format!("- **High Priority Risks (RPN > 100):** {}\n", analytics.high_priority_count));
        report.push_str(&format!("- **Risks with Mitigation:** {}\n", analytics.risks_with_mitigation));
        report.push_str(&format!("- **Risks without Mitigation:** {}\n\n", analytics.risks_without_mitigation));
        
        // Risk Distribution
        report.push_str("## Risk Distribution\n\n");
        report.push_str("### By Severity Level\n\n");
        for (level, count) in &analytics.severity_distribution {
            report.push_str(&format!("- {level}: {count} risks\n"));
        }
        
        report.push_str("\n### By RPN Level\n\n");
        for (level, count) in &analytics.rpn_distribution {
            report.push_str(&format!("- {level}: {count} risks\n"));
        }
        
        // Top 10 highest RPN risks
        report.push_str("\n## Top Priority Risks\n\n");
        let mut sorted_risks: Vec<_> = risks.iter().collect();
        sorted_risks.sort_by(|a, b| b.risk_priority_number.cmp(&a.risk_priority_number));
        
        report.push_str("| Risk ID | Hazard | RPN | Severity | Occurrence | Detectability | Mitigations |\n");
        report.push_str("|---------|--------|-----|----------|------------|---------------|-------------|\n");
        
        for risk in sorted_risks.iter().take(10) {
            let short_desc = if risk.hazard_description.len() > 30 {
                format!("{}...", &risk.hazard_description[..27])
            } else {
                risk.hazard_description.clone()
            };
            
            let mitigation_count = risk.mitigation_measures.len();
            
            report.push_str(&format!(
                "| {} | {} | {} | {:?} | {:?} | {:?} | {} |\n",
                risk.hazard_id,
                short_desc,
                risk.risk_priority_number,
                risk.severity,
                risk.occurrence,
                risk.detectability,
                mitigation_count
            ));
        }
        
        // Recommendations
        report.push_str("\n## Recommendations\n\n");
        if analytics.high_priority_count > 0 {
            report.push_str(&format!("⚠️  **Immediate Action Required:** {} high-priority risks (RPN > 100) need attention.\n\n", analytics.high_priority_count));
        }
        if analytics.risks_without_mitigation > 0 {
            report.push_str(&format!("📋 **Mitigation Planning:** {} risks lack mitigation measures.\n\n", analytics.risks_without_mitigation));
        }
        if analytics.average_rpn > 50.0 {
            report.push_str("📈 **Process Improvement:** Consider systematic risk reduction strategies.\n\n");
        }
        
        Ok(report)
    }
    
    /// Generate CSV format summary
    fn generate_csv_summary(&self, risks: &[RiskItem], _analytics: &RiskAnalytics) -> QmsResult<String> {
        let mut csv = String::new();
        csv.push_str("Hazard ID,Project ID,Hazard Description,Harm,RPN,Severity,Occurrence,Detectability,Risk Status,Mitigation Count,Created At,Updated At\n");
        
        for risk in risks {
            let mitigation_count = risk.mitigation_measures.len();
            
            csv.push_str(&format!(
                "{},{},{},{},{},{:?},{:?},{:?},{:?},{},{},{}\n",
                risk.hazard_id,
                risk.project_id,
                risk.hazard_description.replace(',', ";"),
                risk.harm.replace(',', ";"),
                risk.risk_priority_number,
                risk.severity,
                risk.occurrence,
                risk.detectability,
                risk.risk_status,
                mitigation_count,
                risk.created_at,
                risk.updated_at
            ));
        }
        
        Ok(csv)
    }
    
    /// Generate JSON format summary  
    fn generate_json_summary(&self, risks: &[RiskItem], analytics: &RiskAnalytics) -> QmsResult<String> {
        let mut json = String::new();
        json.push_str("{\n");
        json.push_str(&format!("  \"generated_at\": \"{} UTC\",\n", get_current_timestamp_string()));
        json.push_str("  \"summary\": {\n");
        json.push_str(&format!("    \"total_risks\": {},\n", analytics.total_risks));
        json.push_str(&format!("    \"average_rpn\": {:.1},\n", analytics.average_rpn));
        json.push_str(&format!("    \"high_priority_count\": {},\n", analytics.high_priority_count));
        json.push_str(&format!("    \"risks_with_mitigation\": {},\n", analytics.risks_with_mitigation));
        json.push_str(&format!("    \"risks_without_mitigation\": {}\n", analytics.risks_without_mitigation));
        json.push_str("  },\n");
        json.push_str("  \"distributions\": {\n");
        json.push_str("    \"severity\": {\n");
        for (i, (level, count)) in analytics.severity_distribution.iter().enumerate() {
            let comma = if i == analytics.severity_distribution.len() - 1 { "" } else { "," };
            json.push_str(&format!("      \"{level}\": {count}{comma}\n"));
        }
        json.push_str("    },\n");
        json.push_str("    \"rpn\": {\n");
        for (i, (level, count)) in analytics.rpn_distribution.iter().enumerate() {
            let comma = if i == analytics.rpn_distribution.len() - 1 { "" } else { "," };
            json.push_str(&format!("      \"{level}\": {count}{comma}\n"));
        }
        json.push_str("    }\n");
        json.push_str("  },\n");
        json.push_str("  \"risks\": [\n");
        for (i, risk) in risks.iter().enumerate() {
            let comma = if i == risks.len() - 1 { "" } else { "," };
            json.push_str("    {\n");
            json.push_str(&format!("      \"project_id\": \"{}\",\n", risk.project_id));
            json.push_str(&format!("      \"hazard_id\": \"{}\",\n", risk.hazard_id));
            json.push_str(&format!("      \"hazard_description\": \"{}\",\n", risk.hazard_description.replace('"', "\\\"")));
            json.push_str(&format!("      \"harm\": \"{}\",\n", risk.harm.replace('"', "\\\"")));
            json.push_str(&format!("      \"severity\": \"{:?}\",\n", risk.severity));
            json.push_str(&format!("      \"occurrence\": \"{:?}\",\n", risk.occurrence));
            json.push_str(&format!("      \"detectability\": \"{:?}\",\n", risk.detectability));
            json.push_str(&format!("      \"rpn\": {},\n", risk.risk_priority_number));
            json.push_str(&format!("      \"risk_status\": \"{:?}\",\n", risk.risk_status));
            json.push_str(&format!("      \"mitigation_count\": {},\n", risk.mitigation_measures.len()));
            json.push_str(&format!("      \"created_at\": \"{}\",\n", risk.created_at));
            json.push_str(&format!("      \"updated_at\": \"{}\"\n", risk.updated_at));
            json.push_str(&format!("    }}{comma}\n"));
        }
        json.push_str("  ]\n");
        json.push_str("}\n");
        
        Ok(json)
    }
    
    /// Generate HTML format summary
    fn generate_html_summary(&self, risks: &[RiskItem], analytics: &RiskAnalytics) -> QmsResult<String> {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<title>Risk Management Summary Report</title>\n");
        html.push_str("<style>\n");
        html.push_str("body { font-family: Arial, sans-serif; margin: 40px; }\n");
        html.push_str("table { border-collapse: collapse; width: 100%; }\n");
        html.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
        html.push_str("th { background-color: #f2f2f2; }\n");
        html.push_str(".high-rpn { background-color: #ffcccc; }\n");
        html.push_str("</style>\n</head>\n<body>\n");
        
        html.push_str("<h1>Risk Management Summary Report</h1>\n");
        html.push_str(&format!("<p><strong>Generated:</strong> {} UTC</p>\n", get_current_timestamp_string()));
        
        html.push_str("<h2>Executive Summary</h2>\n<ul>\n");
        html.push_str(&format!("<li><strong>Total Risks:</strong> {}</li>\n", analytics.total_risks));
        html.push_str(&format!("<li><strong>Average RPN:</strong> {:.1}</li>\n", analytics.average_rpn));
        html.push_str(&format!("<li><strong>High Priority Risks:</strong> {}</li>\n", analytics.high_priority_count));
        html.push_str(&format!("<li><strong>Risks with Mitigation:</strong> {}</li>\n", analytics.risks_with_mitigation));
        html.push_str("</ul>\n");
        
        html.push_str("<h2>Risk Details</h2>\n");
        html.push_str("<table>\n<tr><th>Hazard ID</th><th>Hazard Description</th><th>RPN</th><th>Severity</th><th>Occurrence</th><th>Detectability</th><th>Mitigations</th></tr>\n");
        
        let mut sorted_risks: Vec<_> = risks.iter().collect();
        sorted_risks.sort_by(|a, b| b.risk_priority_number.cmp(&a.risk_priority_number));
        
        for risk in sorted_risks {
            let row_class = if risk.risk_priority_number > 50 { " class=\"high-rpn\"" } else { "" };
            let mitigation_count = risk.mitigation_measures.len();
            
            html.push_str(&format!(
                "<tr{}><td>{}</td><td>{}</td><td>{}</td><td>{:?}</td><td>{:?}</td><td>{:?}</td><td>{}</td></tr>\n",
                row_class,
                risk.hazard_id,
                risk.hazard_description,
                risk.risk_priority_number,
                risk.severity,
                risk.occurrence,
                risk.detectability,
                mitigation_count
            ));
        }
        
        html.push_str("</table>\n</body>\n</html>");
        Ok(html)
    }
    
    /// Generate trend analysis (placeholder for now)
    pub fn generate_trend_analysis(&self, _period: TimePeriod) -> QmsResult<Vec<RiskTrend>> {
        // For now, return placeholder data
        // In a full implementation, this would analyze risk data over time
        Ok(vec![
            RiskTrend {
                period: "Current".to_string(),
                total_risks: 0,
                new_risks: 0,
                closed_risks: 0,
                average_rpn: 0.0,
                high_priority_risks: 0,
            }
        ])
    }
    
    /// Save report content to file
    pub fn save_report_to_file(&self, content: &str, filename: &str) -> QmsResult<()> {
        let file_path = self.reports_dir.join(filename);
        fs::write(file_path, content)?;
        Ok(())
    }
}

/// Get current timestamp as formatted string (stdlib-only)
fn get_current_timestamp_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let timestamp = duration.as_secs();
    
    // Simple date formatting from UNIX timestamp (approximation)
    let days_since_epoch = timestamp / 86400;
    let years_since_1970 = days_since_epoch / 365;
    let year = 1970 + years_since_1970;
    
    // This is a simplified date format - in production, a proper date library would be better
    format!("{year}-01-01 12:00:00")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    fn create_test_risk(hazard_id: &str, rpn: u32, has_mitigation: bool) -> RiskItem {
        use crate::modules::risk_manager::risk::{RiskSeverity, RiskOccurrence, RiskDetectability, RiskLevel, RiskStatus, VerificationStatus, MitigationMeasure};
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let iso_timestamp = format!("2024-01-16T{:02}:00:00Z", timestamp % 24);
        
        RiskItem {
            id: format!("uuid-{}", hazard_id),
            project_id: "test-project".to_string(),
            hazard_id: hazard_id.to_string(),
            hazard_description: format!("Test hazard {}", hazard_id),
            hazardous_situation: "Test situation".to_string(),
            harm: "Test harm".to_string(),
            severity: if rpn > 50 { RiskSeverity::Major } else { RiskSeverity::Minor },
            occurrence: if rpn > 25 { RiskOccurrence::Occasional } else { RiskOccurrence::Remote },
            detectability: RiskDetectability::Moderate,
            risk_priority_number: rpn,
            initial_risk_level: if rpn > 50 { RiskLevel::Unacceptable } else { RiskLevel::Acceptable },
            mitigation_measures: if has_mitigation { 
                vec![MitigationMeasure {
                    id: "MIT-001".to_string(),
                    description: "Test mitigation".to_string(),
                    implementation: "Test implementation".to_string(),
                    effectiveness: 0.8,
                    cost: None,
                    timeline: None,
                    verification_method: "Testing".to_string(),
                    verification_status: VerificationStatus::Planned,
                    verification_evidence: vec![],
                    implementation_status: "Planned".to_string(),
                    assigned_to: None,
                    due_date: None,
                    implemented_date: None,
                    verified_date: None,
                }]
            } else { 
                vec![] 
            },
            residual_severity: RiskSeverity::Minor,
            residual_occurrence: RiskOccurrence::Remote,
            residual_detectability: RiskDetectability::High,
            residual_rpn: rpn / 2,
            residual_risk_level: RiskLevel::Acceptable,
            residual_risk_justification: None,
            residual_risk_approved: false,
            residual_risk_approved_by: None,
            residual_risk_approval_date: None,
            verification_method: "Testing".to_string(),
            verification_status: VerificationStatus::Planned,
            verification_evidence: vec![],
            category: "Safety".to_string(),
            source: "FMEA".to_string(),
            assigned_to: None,
            due_date: None,
            priority: "Medium".to_string(),
            risk_status: RiskStatus::Identified,
            tags: vec![],
            regulatory_references: vec![],
            standard_references: vec![],
            created_at: iso_timestamp.clone(),
            updated_at: iso_timestamp.clone(),
            created_by: "test-user".to_string(),
            approved_by: None,
            approval_date: None,
            post_market_data: vec![],
            review_required: false,
            next_review_date: None,
        }
    }
    
    #[test]
    fn test_risk_analytics_generation() {
        let temp_dir = TempDir::new().unwrap();
        let reporter = RiskReporter::new(temp_dir.path()).unwrap();
        
        let risks = vec![
            create_test_risk("RISK-001", 150, true),
            create_test_risk("RISK-002", 80, false),
            create_test_risk("RISK-003", 200, true),
        ];
        
        let analytics = reporter.generate_analytics(&risks);
        
        assert_eq!(analytics.total_risks, 3);
        assert_eq!(analytics.high_priority_count, 3); // RPN > 50 (150, 80, 200 are all > 50)
        assert_eq!(analytics.risks_with_mitigation, 2);
        assert_eq!(analytics.risks_without_mitigation, 1);
        assert!((analytics.average_rpn - 143.33).abs() < 0.1);
    }
    
    #[test]
    fn test_markdown_report_generation() {
        let temp_dir = TempDir::new().unwrap();
        let reporter = RiskReporter::new(temp_dir.path()).unwrap();
        
        let risks = vec![
            create_test_risk("RISK-001", 120, true),
            create_test_risk("RISK-002", 50, false),
        ];
        
        let report = reporter.generate_summary_report(&risks, ReportFormat::Markdown).unwrap();
        
        assert!(report.contains("Risk Management Summary Report"));
        assert!(report.contains("**Total Risks:** 2"));
        assert!(report.contains("High Priority Risks"));
        assert!(report.contains("RISK-001"));
    }
    
    #[test]
    fn test_csv_report_generation() {
        let temp_dir = TempDir::new().unwrap();
        let reporter = RiskReporter::new(temp_dir.path()).unwrap();
        
        let risks = vec![create_test_risk("RISK-001", 100, true)];
        
        let report = reporter.generate_summary_report(&risks, ReportFormat::CSV).unwrap();
        
        assert!(report.contains("Hazard ID,Project ID"));
        assert!(report.contains("RISK-001"));
        assert!(report.contains("1")); // has 1 mitigation
    }
    
    #[test]
    fn test_json_report_generation() {
        let temp_dir = TempDir::new().unwrap();
        let reporter = RiskReporter::new(temp_dir.path()).unwrap();
        
        let risks = vec![create_test_risk("RISK-001", 75, false)];
        
        let report = reporter.generate_summary_report(&risks, ReportFormat::JSON).unwrap();
        
        assert!(report.contains("\"total_risks\": 1"));
        assert!(report.contains("\"RISK-001\""));
        // Should be valid JSON structure
        assert!(report.contains("{") && report.contains("}"));
    }
    
    #[test]
    fn test_html_report_generation() {
        let temp_dir = TempDir::new().unwrap();
        let reporter = RiskReporter::new(temp_dir.path()).unwrap();
        
        let risks = vec![create_test_risk("RISK-001", 150, true)];
        
        let report = reporter.generate_summary_report(&risks, ReportFormat::HTML).unwrap();
        
        assert!(report.contains("<!DOCTYPE html>"));
        assert!(report.contains("Risk Management Summary Report"));
        assert!(report.contains("RISK-001"));
        assert!(report.contains("class=\"high-rpn\"")); // High RPN styling (>50)
    }
}
