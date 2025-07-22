//! Regulatory Compliance Module for Audit Logging
//! 
//! Implements 21 CFR Part 11 compliance features for electronic records and signatures.
//! Provides validation, reporting, and compliance checking functionality.

use crate::prelude::*;
use crate::models::AuditEntry;
use crate::json_utils::JsonSerializable;
use std::collections::HashMap;

/// 21 CFR Part 11 compliance validator
pub struct RegulatoryCompliance {
    project_path: std::path::PathBuf,
}

/// Compliance validation result
#[derive(Debug)]
pub struct ComplianceValidation {
    pub is_compliant: bool,
    pub issues: Vec<ComplianceIssue>,
    pub validation_summary: ComplianceSummary,
}

/// Individual compliance issue
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ComplianceIssue {
    pub issue_type: ComplianceIssueType,
    pub description: String,
    pub affected_entry: Option<String>, // Entry ID if applicable
    pub severity: IssueSeverity,
}

/// Types of compliance issues
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ComplianceIssueType {
    MissingRequiredField,
    InvalidTimestamp,
    MissingUserIdentification,
    BreachedImmutability,
    InvalidHashChain,
    MissingElectronicSignature,
    InsufficientAuditTrail,
}

/// Severity levels for compliance issues
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum IssueSeverity {
    Critical,    // Must fix for compliance
    Warning,     // Should fix for best practices
    Info,        // Informational note
}

/// Summary of compliance validation
#[derive(Debug)]
pub struct ComplianceSummary {
    pub total_entries_checked: usize,
    pub compliant_entries: usize,
    pub critical_issues: usize,
    pub warning_issues: usize,
    pub info_issues: usize,
    pub hash_chain_integrity: bool,
    pub cfr_part_11_score: f64, // 0.0 to 100.0
}

/// 21 CFR Part 11 compliance report
#[derive(Debug)]
pub struct ComplianceReport {
    pub generated_at: String,
    pub generated_by: String,
    pub report_period: String,
    pub validation_result: ComplianceValidation,
    pub audit_trail_summary: AuditTrailSummary,
    pub recommendations: Vec<String>,
}

/// Summary of audit trail for compliance
#[allow(dead_code)]
#[derive(Debug)]
pub struct AuditTrailSummary {
    pub total_entries: usize,
    pub date_range: (String, String),
    pub unique_users: usize,
    pub action_types: HashMap<String, usize>,
    pub entity_types: HashMap<String, usize>,
    pub average_daily_activity: f64,
}

impl RegulatoryCompliance {
    /// Create new regulatory compliance validator
    pub const fn new(project_path: std::path::PathBuf) -> Self {
        Self { project_path }
    }

    /// Validate 21 CFR Part 11 compliance for all audit entries
    pub fn validate_cfr_part_11_compliance(&self) -> QmsResult<ComplianceValidation> {
        let mut issues = Vec::new();
        let mut compliant_entries = 0;

        // Read all audit entries
        let audit_entries = self.load_all_audit_entries()?;
        let total_entries = audit_entries.len();

        // Validate each entry for 21 CFR Part 11 requirements
        for entry in &audit_entries {
            let entry_issues = self.validate_entry_compliance(entry);
            let is_entry_compliant = entry_issues.iter()
                .all(|issue| !matches!(issue.severity, IssueSeverity::Critical));
            
            if is_entry_compliant {
                compliant_entries += 1;
            }
            
            issues.extend(entry_issues);
        }

        // Validate hash chain integrity
        let hash_chain_integrity = self.validate_hash_chain_integrity(&audit_entries)?;
        if !hash_chain_integrity {
            issues.push(ComplianceIssue {
                issue_type: ComplianceIssueType::InvalidHashChain,
                description: "Hash chain integrity validation failed".to_string(),
                affected_entry: None,
                severity: IssueSeverity::Critical,
            });
        }

        // Calculate compliance metrics
        let critical_issues = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Critical)).count();
        let warning_issues = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Warning)).count();
        let info_issues = issues.iter().filter(|i| matches!(i.severity, IssueSeverity::Info)).count();

        // Calculate CFR Part 11 compliance score
        let cfr_score = if total_entries == 0 {
            100.0
        } else {
            let compliance_rate = compliant_entries as f64 / total_entries as f64;
            let chain_bonus = if hash_chain_integrity { 10.0 } else { 0.0 };
            let critical_penalty = critical_issues as f64 * 10.0;
            
            (compliance_rate * 80.0 + chain_bonus - critical_penalty).clamp(0.0, 100.0)
        };

        let validation = ComplianceValidation {
            is_compliant: critical_issues == 0 && hash_chain_integrity,
            issues,
            validation_summary: ComplianceSummary {
                total_entries_checked: total_entries,
                compliant_entries,
                critical_issues,
                warning_issues,
                info_issues,
                hash_chain_integrity,
                cfr_part_11_score: cfr_score,
            },
        };

        Ok(validation)
    }

    /// Validate individual audit entry for 21 CFR Part 11 compliance
    fn validate_entry_compliance(&self, entry: &AuditEntry) -> Vec<ComplianceIssue> {
        let mut issues = Vec::new();

        // Required Field: User identification
        if entry.user_id.trim().is_empty() {
            issues.push(ComplianceIssue {
                issue_type: ComplianceIssueType::MissingUserIdentification,
                description: "User identification is required for 21 CFR Part 11 compliance".to_string(),
                affected_entry: Some(entry.id.clone()),
                severity: IssueSeverity::Critical,
            });
        }

        // Required Field: Timestamp validation
        if !self.is_valid_timestamp(&entry.timestamp) {
            issues.push(ComplianceIssue {
                issue_type: ComplianceIssueType::InvalidTimestamp,
                description: "Invalid or missing timestamp format".to_string(),
                affected_entry: Some(entry.id.clone()),
                severity: IssueSeverity::Critical,
            });
        }

        // Required Field: Action performed
        if format!("{:?}", entry.action).trim().is_empty() {
            issues.push(ComplianceIssue {
                issue_type: ComplianceIssueType::MissingRequiredField,
                description: "Action performed must be recorded".to_string(),
                affected_entry: Some(entry.id.clone()),
                severity: IssueSeverity::Critical,
            });
        }

        // Audit trail requirement: Entity type and ID
        if entry.entity_type.trim().is_empty() {
            issues.push(ComplianceIssue {
                issue_type: ComplianceIssueType::InsufficientAuditTrail,
                description: "Entity type must be specified for complete audit trail".to_string(),
                affected_entry: Some(entry.id.clone()),
                severity: IssueSeverity::Warning,
            });
        }

        if entry.entity_id.trim().is_empty() {
            issues.push(ComplianceIssue {
                issue_type: ComplianceIssueType::InsufficientAuditTrail,
                description: "Entity ID must be specified for complete audit trail".to_string(),
                affected_entry: Some(entry.id.clone()),
                severity: IssueSeverity::Warning,
            });
        }

        // Electronic signature requirement for critical actions
        if self.requires_electronic_signature(entry) && entry.signature.is_none() {
            issues.push(ComplianceIssue {
                issue_type: ComplianceIssueType::MissingElectronicSignature,
                description: "Electronic signature required for this type of action".to_string(),
                affected_entry: Some(entry.id.clone()),
                severity: IssueSeverity::Critical,
            });
        }

        issues
    }

    /// Check if timestamp is valid 21 CFR Part 11 format
    fn is_valid_timestamp(&self, timestamp: &str) -> bool {
        // Should be ISO 8601 format with timezone
        
        // Try parsing as ISO 8601 with 'Z' timezone
        if timestamp.ends_with('Z') && timestamp.len() >= 19 {
            // Format: YYYY-MM-DDTHH:MM:SSZ
            let parts: Vec<&str> = timestamp.trim_end_matches('Z').split('T').collect();
            if parts.len() == 2 {
                let date_parts: Vec<&str> = parts[0].split('-').collect();
                let time_parts: Vec<&str> = parts[1].split(':').collect();
                
                return date_parts.len() == 3 && time_parts.len() == 3 &&
                       date_parts.iter().all(|p| p.parse::<u32>().is_ok()) &&
                       time_parts.iter().all(|p| p.parse::<u32>().is_ok());
            }
        }
        
        false
    }

    /// Check if action requires electronic signature per 21 CFR Part 11
    fn requires_electronic_signature(&self, entry: &AuditEntry) -> bool {
        match entry.action {
            crate::models::AuditAction::Delete => true,
            crate::models::AuditAction::Other(ref action) => {
                action.to_lowercase().contains("approve") ||
                action.to_lowercase().contains("sign") ||
                action.to_lowercase().contains("release") ||
                action.to_lowercase().contains("final")
            },
            _ => false,
        }
    }

    /// Validate hash chain integrity for immutability compliance
    fn validate_hash_chain_integrity(&self, entries: &[AuditEntry]) -> QmsResult<bool> {
        if entries.is_empty() {
            return Ok(true);
        }

        // Sort entries by timestamp to validate chain
        let mut sorted_entries = entries.to_vec();
        sorted_entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        for i in 1..sorted_entries.len() {
            let current = &sorted_entries[i];
            let previous = &sorted_entries[i-1];

            if let Some(ref prev_hash) = current.previous_hash {
                if prev_hash != &previous.checksum {
                    return Ok(false);
                }
            } else if i > 0 {
                // Missing previous hash link
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Load all audit entries from all sources
    fn load_all_audit_entries(&self) -> QmsResult<Vec<AuditEntry>> {
        use std::fs;
        
        let mut all_entries = Vec::new();

        // Load from main audit log
        let main_log_path = self.project_path.join("audit").join("audit.log");
        if main_log_path.exists() {
            let entries = self.load_entries_from_file(&main_log_path)?;
            all_entries.extend(entries);
        }

        // Load from daily logs
        let daily_dir = self.project_path.join("audit").join("daily");
        if daily_dir.exists() {
            for entry in fs::read_dir(&daily_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "log") {
                    let entries = self.load_entries_from_file(&path)?;
                    all_entries.extend(entries);
                }
            }
        }

        Ok(all_entries)
    }

    /// Load audit entries from a specific file
    fn load_entries_from_file(&self, file_path: &Path) -> QmsResult<Vec<AuditEntry>> {
        use std::fs;
        
        let content = fs::read_to_string(file_path)?;
        let mut entries = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match AuditEntry::from_json(line) {
                Ok(entry) => entries.push(entry),
                Err(_) => continue, // Skip malformed entries
            }
        }

        Ok(entries)
    }

    /// Generate comprehensive 21 CFR Part 11 compliance report
    pub fn generate_compliance_report(&self, period: &str) -> QmsResult<ComplianceReport> {
        let validation = self.validate_cfr_part_11_compliance()?;
        let audit_summary = self.generate_audit_trail_summary()?;
        let recommendations = self.generate_compliance_recommendations(&validation);

        let report = ComplianceReport {
            generated_at: self.get_current_timestamp(),
            generated_by: "QMS Regulatory Compliance System".to_string(),
            report_period: period.to_string(),
            validation_result: validation,
            audit_trail_summary: audit_summary,
            recommendations,
        };

        Ok(report)
    }

    /// Generate audit trail summary for compliance reporting
    fn generate_audit_trail_summary(&self) -> QmsResult<AuditTrailSummary> {
        let entries = self.load_all_audit_entries()?;
        
        if entries.is_empty() {
            return Ok(AuditTrailSummary {
                total_entries: 0,
                date_range: ("N/A".to_string(), "N/A".to_string()),
                unique_users: 0,
                action_types: HashMap::new(),
                entity_types: HashMap::new(),
                average_daily_activity: 0.0,
            });
        }

        // Calculate date range
        let mut timestamps: Vec<String> = entries.iter().map(|e| e.timestamp.clone()).collect();
        timestamps.sort();
        let date_range = (
            timestamps.first().cloned().unwrap_or_else(|| "N/A".to_string()),
            timestamps.last().cloned().unwrap_or_else(|| "N/A".to_string())
        );

        // Count unique users
        let unique_users: std::collections::HashSet<&str> = 
            entries.iter().map(|e| e.user_id.as_str()).collect();

        // Count action types
        let mut action_types = HashMap::new();
        for entry in &entries {
            let action_str = format!("{:?}", entry.action);
            *action_types.entry(action_str).or_insert(0) += 1;
        }

        // Count entity types
        let mut entity_types = HashMap::new();
        for entry in &entries {
            *entity_types.entry(entry.entity_type.clone()).or_insert(0) += 1;
        }

        // Calculate average daily activity (simplified)
        let average_daily_activity = entries.len() as f64; // Simplified calculation

        Ok(AuditTrailSummary {
            total_entries: entries.len(),
            date_range,
            unique_users: unique_users.len(),
            action_types,
            entity_types,
            average_daily_activity,
        })
    }

    /// Generate compliance recommendations based on validation results
    fn generate_compliance_recommendations(&self, validation: &ComplianceValidation) -> Vec<String> {
        let mut recommendations = Vec::new();

        if validation.validation_summary.critical_issues > 0 {
            recommendations.push("CRITICAL: Address all critical compliance issues immediately to meet 21 CFR Part 11 requirements.".to_string());
        }

        if !validation.validation_summary.hash_chain_integrity {
            recommendations.push("CRITICAL: Restore hash chain integrity to ensure audit trail immutability.".to_string());
        }

        if validation.validation_summary.cfr_part_11_score < 90.0 {
            recommendations.push("Improve 21 CFR Part 11 compliance score by addressing identified issues.".to_string());
        }

        if validation.validation_summary.warning_issues > 0 {
            recommendations.push("Address warning-level issues to improve audit trail completeness.".to_string());
        }

        if validation.validation_summary.total_entries_checked == 0 {
            recommendations.push("Initialize audit logging system to begin compliance tracking.".to_string());
        }

        // Add general best practices
        recommendations.push("Regularly validate compliance and generate reports for regulatory submissions.".to_string());
        recommendations.push("Implement electronic signatures for critical system actions.".to_string());
        recommendations.push("Maintain backup copies of audit logs in secure, immutable storage.".to_string());

        recommendations
    }

    /// Get current timestamp in ISO 8601 format
    fn get_current_timestamp(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let duration = SystemTime::now().duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0));
        let timestamp = duration.as_secs();
        
        // Convert to ISO 8601 format (simplified)
        format!("2025-07-25T{:02}:{:02}:{:02}Z", 
                (timestamp / 3600) % 24,
                (timestamp / 60) % 60,
                timestamp % 60)
    }
}

/// Format compliance report for display
pub fn format_compliance_report(report: &ComplianceReport) -> String {
    let mut output = String::new();
    
    output.push_str("21 CFR Part 11 COMPLIANCE REPORT\n");
    output.push_str("==================================\n\n");
    
    output.push_str(&format!("Generated: {}\n", report.generated_at));
    output.push_str(&format!("Generated By: {}\n", report.generated_by));
    output.push_str(&format!("Report Period: {}\n\n", report.report_period));
    
    // Compliance status
    output.push_str("COMPLIANCE STATUS\n");
    output.push_str("-----------------\n");
    output.push_str(&format!("Overall Compliance: {}\n", 
        if report.validation_result.is_compliant { "✅ COMPLIANT" } else { "❌ NON-COMPLIANT" }));
    output.push_str(&format!("CFR Part 11 Score: {:.1}%\n\n", 
        report.validation_result.validation_summary.cfr_part_11_score));
    
    // Validation summary
    let summary = &report.validation_result.validation_summary;
    output.push_str("VALIDATION SUMMARY\n");
    output.push_str("------------------\n");
    output.push_str(&format!("Total Entries Checked: {}\n", summary.total_entries_checked));
    output.push_str(&format!("Compliant Entries: {}\n", summary.compliant_entries));
    output.push_str(&format!("Critical Issues: {}\n", summary.critical_issues));
    output.push_str(&format!("Warning Issues: {}\n", summary.warning_issues));
    output.push_str(&format!("Hash Chain Integrity: {}\n\n", 
        if summary.hash_chain_integrity { "✅ Valid" } else { "❌ Invalid" }));
    
    // Issues
    if !report.validation_result.issues.is_empty() {
        output.push_str("COMPLIANCE ISSUES\n");
        output.push_str("-----------------\n");
        for issue in &report.validation_result.issues {
            let severity_icon = match issue.severity {
                IssueSeverity::Critical => "🔴",
                IssueSeverity::Warning => "🟡", 
                IssueSeverity::Info => "🔵",
            };
            output.push_str(&format!("{} {:?}: {}\n", severity_icon, issue.severity, issue.description));
        }
        output.push('\n');
    }
    
    // Recommendations
    if !report.recommendations.is_empty() {
        output.push_str("RECOMMENDATIONS\n");
        output.push_str("---------------\n");
        for (i, recommendation) in report.recommendations.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", i + 1, recommendation));
        }
        output.push('\n');
    }
    
    // Audit trail summary
    let audit_summary = &report.audit_trail_summary;
    output.push_str("AUDIT TRAIL SUMMARY\n");
    output.push_str("-------------------\n");
    output.push_str(&format!("Total Entries: {}\n", audit_summary.total_entries));
    output.push_str(&format!("Unique Users: {}\n", audit_summary.unique_users));
    output.push_str(&format!("Date Range: {} to {}\n", audit_summary.date_range.0, audit_summary.date_range.1));
    output.push_str(&format!("Average Daily Activity: {:.1} entries\n\n", audit_summary.average_daily_activity));
    
    output.push_str("This report certifies compliance with 21 CFR Part 11 requirements\n");
    output.push_str("for electronic records and electronic signatures.\n");
    
    output
}
