use crate::prelude::*;
use crate::modules::traceability::links::TraceabilityManager;
use crate::audit::log_audit;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Helper function to get current timestamp as RFC3339 string
fn get_current_timestamp() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs();
            let nanos = duration.subsec_nanos();
            format!("{secs}.{nanos:09}Z")
        }
        Err(_) => "1970-01-01T00:00:00Z".to_string(),
    }
}

/// Verification methods as per FDA 21 CFR 820 and IEC 62304
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationMethod {
    Test,           // Testing and verification through test execution
    Analysis,       // Analysis and mathematical verification
    Inspection,     // Visual inspection and code review
    Demonstration,  // Demonstration of functionality
}

impl VerificationMethod {
    /// Convert string to verification method
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "test" => Some(VerificationMethod::Test),
            "analysis" => Some(VerificationMethod::Analysis),
            "inspection" => Some(VerificationMethod::Inspection),
            "demonstration" => Some(VerificationMethod::Demonstration),
            _ => None,
        }
    }
    
    /// Convert verification method to string
    pub fn to_string(&self) -> String {
        match self {
            VerificationMethod::Test => "Test".to_string(),
            VerificationMethod::Analysis => "Analysis".to_string(),
            VerificationMethod::Inspection => "Inspection".to_string(),
            VerificationMethod::Demonstration => "Demonstration".to_string(),
        }
    }
    
    /// Get all available verification methods
    #[allow(dead_code)]
    pub fn all() -> Vec<Self> {
        vec![
            VerificationMethod::Test,
            VerificationMethod::Analysis,
            VerificationMethod::Inspection,
            VerificationMethod::Demonstration,
        ]
    }
}

/// Verification status for requirements
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationStatus {
    NotVerified,      // No verification performed
    PartiallyVerified, // Some verification evidence exists
    FullyVerified,    // Complete verification with all evidence
}

impl VerificationStatus {
    /// Convert string to verification status
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "not_verified" | "not verified" | "none" => Some(VerificationStatus::NotVerified),
            "partially_verified" | "partially verified" | "partial" => Some(VerificationStatus::PartiallyVerified),
            "fully_verified" | "fully verified" | "complete" => Some(VerificationStatus::FullyVerified),
            _ => None,
        }
    }
    
    /// Convert verification status to string
    pub fn to_string(&self) -> String {
        match self {
            VerificationStatus::NotVerified => "Not Verified".to_string(),
            VerificationStatus::PartiallyVerified => "Partially Verified".to_string(),
            VerificationStatus::FullyVerified => "Fully Verified".to_string(),
        }
    }
    
    /// Get status color for display
    pub const fn color(&self) -> &'static str {
        match self {
            VerificationStatus::NotVerified => "🔴",
            VerificationStatus::PartiallyVerified => "🟡",
            VerificationStatus::FullyVerified => "🟢",
        }
    }
}

/// Verification evidence linking requirements to evidence
#[derive(Debug, Clone)]
pub struct VerificationEvidence {
    pub evidence_id: String,
    pub evidence_type: String,
    #[allow(dead_code)]
    pub evidence_path: Option<String>,
    pub description: String,
    pub created_at: String,
    pub created_by: String,
    pub method: VerificationMethod,
    pub status: VerificationStatus,
    pub notes: String,
}

impl Default for VerificationEvidence {
    fn default() -> Self {
        Self {
            evidence_id: String::new(),
            evidence_type: "Test Result".to_string(),
            evidence_path: None,
            description: String::new(),
            created_at: get_current_timestamp(),
            created_by: "System".to_string(),
            method: VerificationMethod::Test,
            status: VerificationStatus::NotVerified,
            notes: String::new(),
        }
    }
}

/// Verification record for a requirement
#[derive(Debug, Clone)]
pub struct RequirementVerification {
    pub requirement_id: String,
    pub verification_method: VerificationMethod,
    pub verification_status: VerificationStatus,
    pub evidence: Vec<VerificationEvidence>,
    pub verified_at: Option<String>,
    pub verified_by: Option<String>,
    pub verification_notes: String,
    pub acceptance_criteria: Vec<String>,
    pub test_results: Vec<String>,
}

impl Default for RequirementVerification {
    fn default() -> Self {
        Self {
            requirement_id: String::new(),
            verification_method: VerificationMethod::Test,
            verification_status: VerificationStatus::NotVerified,
            evidence: Vec::new(),
            verified_at: None,
            verified_by: None,
            verification_notes: String::new(),
            acceptance_criteria: Vec::new(),
            test_results: Vec::new(),
        }
    }
}

/// Requirement verification manager
pub struct RequirementVerificationManager {
    verifications: HashMap<String, RequirementVerification>,
    storage_path: String,
}

impl RequirementVerificationManager {
    /// Create new verification manager
    pub fn new(storage_path: &str) -> Self {
        Self {
            verifications: HashMap::new(),
            storage_path: storage_path.to_string(),
        }
    }
    
    /// Load verification data from storage
    pub fn load(&mut self) -> QmsResult<()> {
        let path = Path::new(&self.storage_path);
        if !path.exists() {
            return Ok(());
        }
        
        let content = fs::read_to_string(path)?;
        if content.trim().is_empty() {
            return Ok(());
        }
        
        // Parse JSON manually for verification data
        let json_data = self.parse_verification_json(&content)?;
        self.verifications = json_data;
        
        log_audit("verification_data_loaded");
        Ok(())
    }
    
    /// Save verification data to storage
    pub fn save(&self) -> QmsResult<()> {
        let json_content = self.serialize_verifications_to_json()?;
        fs::write(&self.storage_path, json_content)?;
        log_audit("verification_data_saved");
        Ok(())
    }
    
    /// Add verification for a requirement
    pub fn add_verification(&mut self, requirement_id: &str, method: VerificationMethod, evidence_id: &str) -> QmsResult<()> {
        let mut verification = self.verifications.get(requirement_id).cloned()
            .unwrap_or_else(|| RequirementVerification {
                requirement_id: requirement_id.to_string(),
                ..Default::default()
            });
        
        verification.verification_method = method.clone();
        
        // Add evidence
        let evidence = VerificationEvidence {
            evidence_id: evidence_id.to_string(),
            evidence_type: "Test Result".to_string(),
            evidence_path: None,
            description: format!("Verification evidence for requirement {requirement_id}"),
            created_at: get_current_timestamp(),
            created_by: "System".to_string(),
            method: method.clone(),
            status: VerificationStatus::PartiallyVerified,
            notes: String::new(),
        };
        
        verification.evidence.push(evidence);
        verification.verification_status = VerificationStatus::PartiallyVerified;
        verification.verified_at = Some(get_current_timestamp());
        verification.verified_by = Some("System".to_string());
        
        self.verifications.insert(requirement_id.to_string(), verification);
        
        log_audit("verification_added");
        Ok(())
    }
    
    /// Update verification status
    pub fn update_verification_status(&mut self, requirement_id: &str, status: VerificationStatus) -> QmsResult<()> {
        if let Some(verification) = self.verifications.get_mut(requirement_id) {
            verification.verification_status = status.clone();
            verification.verified_at = Some(get_current_timestamp());
            log_audit("verification_status_updated");
        }
        Ok(())
    }
    
    /// Add evidence to existing verification
    #[allow(dead_code)]
    pub fn add_evidence(&mut self, requirement_id: &str, evidence: VerificationEvidence) -> QmsResult<()> {
        if let Some(verification) = self.verifications.get_mut(requirement_id) {
            verification.evidence.push(evidence);
            log_audit("verification_evidence_added");
        }
        Ok(())
    }
    
    /// Get verification for a requirement
    pub fn get_verification(&self, requirement_id: &str) -> Option<&RequirementVerification> {
        self.verifications.get(requirement_id)
    }
    
    /// Get all verifications
    #[allow(dead_code)]
    pub const fn get_all_verifications(&self) -> &HashMap<String, RequirementVerification> {
        &self.verifications
    }
    
    /// Get verification statistics
    pub fn get_verification_statistics(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        let total = self.verifications.len();
        
        let mut not_verified = 0;
        let mut partially_verified = 0;
        let mut fully_verified = 0;
        
        for verification in self.verifications.values() {
            match verification.verification_status {
                VerificationStatus::NotVerified => not_verified += 1,
                VerificationStatus::PartiallyVerified => partially_verified += 1,
                VerificationStatus::FullyVerified => fully_verified += 1,
            }
        }
        
        stats.insert("total".to_string(), total);
        stats.insert("not_verified".to_string(), not_verified);
        stats.insert("partially_verified".to_string(), partially_verified);
        stats.insert("fully_verified".to_string(), fully_verified);
        
        stats
    }
    
    /// Generate verification report
    pub fn generate_verification_report(&self, _tm: &TraceabilityManager) -> QmsResult<String> {
        let mut report = String::new();
        
        report.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        report.push_str("                               REQUIREMENT VERIFICATION REPORT\n");
        report.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        report.push_str(&format!("Generated: {}\n", get_current_timestamp()));
        report.push_str(&format!("Total Requirements: {}\n", self.verifications.len()));
        report.push('\n');
        
        let stats = self.get_verification_statistics();
        report.push_str("📊 VERIFICATION STATISTICS\n");
        report.push_str("────────────────────────────────────────────────────────────────────────────────────────────────────\n");
        report.push_str(&format!("   Total Verifications: {}\n", stats.get("total").unwrap_or(&0)));
        report.push_str(&format!("   🔴 Not Verified: {}\n", stats.get("not_verified").unwrap_or(&0)));
        report.push_str(&format!("   🟡 Partially Verified: {}\n", stats.get("partially_verified").unwrap_or(&0)));
        report.push_str(&format!("   🟢 Fully Verified: {}\n", stats.get("fully_verified").unwrap_or(&0)));
        report.push('\n');
        
        // Detailed verification listing
        report.push_str("📋 DETAILED VERIFICATION STATUS\n");
        report.push_str("────────────────────────────────────────────────────────────────────────────────────────────────────\n");
        
        for (req_id, verification) in &self.verifications {
            report.push_str(&format!("\n{} {} - {}\n", 
                verification.verification_status.color(),
                req_id,
                verification.verification_status.to_string()));
            report.push_str(&format!("   Method: {}\n", verification.verification_method.to_string()));
            report.push_str(&format!("   Evidence Count: {}\n", verification.evidence.len()));
            
            if let Some(verified_at) = &verification.verified_at {
                report.push_str(&format!("   Verified: {verified_at}\n"));
            }
            
            if let Some(verified_by) = &verification.verified_by {
                report.push_str(&format!("   Verified By: {verified_by}\n"));
            }
            
            if !verification.verification_notes.is_empty() {
                report.push_str(&format!("   Notes: {}\n", verification.verification_notes));
            }
            
            if !verification.evidence.is_empty() {
                report.push_str("   Evidence:\n");
                for evidence in &verification.evidence {
                    report.push_str(&format!("     • {} ({})\n", evidence.evidence_id, evidence.evidence_type));
                    if !evidence.description.is_empty() {
                        report.push_str(&format!("       Description: {}\n", evidence.description));
                    }
                }
            }
        }
        
        report.push('\n');
        report.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        
        Ok(report)
    }
    
    /// Parse verification JSON manually
    fn parse_verification_json(&self, content: &str) -> QmsResult<HashMap<String, RequirementVerification>> {
        let mut verifications = HashMap::new();
        
        // Simple JSON parsing for verification data
        let lines: Vec<&str> = content.lines().collect();
        let mut current_verification: Option<RequirementVerification> = None;
        let mut _current_key = String::new();
        let _in_evidence_array = false;
        let _current_evidence: Option<VerificationEvidence> = None;
        
        for line in lines {
            let line = line.trim();
            
            if line.starts_with("\"") && line.contains("\":") {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim_matches('"');
                    let value = parts[1].trim().trim_matches(',');
                    
                    _current_key = key.to_string();
                    
                    if key == "requirement_id" {
                        current_verification = Some(RequirementVerification {
                            requirement_id: value.trim_matches('"').to_string(),
                            ..Default::default()
                        });
                    } else if let Some(ref mut verification) = current_verification {
                        match key {
                            "verification_method" => {
                                if let Some(method) = VerificationMethod::from_str(value.trim_matches('"')) {
                                    verification.verification_method = method;
                                }
                            }
                            "verification_status" => {
                                if let Some(status) = VerificationStatus::from_str(value.trim_matches('"')) {
                                    verification.verification_status = status;
                                }
                            }
                            "verified_at" => {
                                if value != "null" {
                                    verification.verified_at = Some(value.trim_matches('"').to_string());
                                }
                            }
                            "verified_by" => {
                                if value != "null" {
                                    verification.verified_by = Some(value.trim_matches('"').to_string());
                                }
                            }
                            "verification_notes" => {
                                verification.verification_notes = value.trim_matches('"').to_string();
                            }
                            _ => {}
                        }
                    }
                }
            } else if line == "}" && current_verification.is_some() {
                let verification = current_verification.take().unwrap();
                verifications.insert(verification.requirement_id.clone(), verification);
            }
        }
        
        Ok(verifications)
    }
    
    /// Serialize verifications to JSON manually
    fn serialize_verifications_to_json(&self) -> QmsResult<String> {
        let mut json = String::new();
        json.push_str("{\n");
        
        let mut first = true;
        for (req_id, verification) in &self.verifications {
            if !first {
                json.push_str(",\n");
            }
            first = false;
            
            json.push_str(&format!("  \"{req_id}\": {{\n"));
            json.push_str(&format!("    \"requirement_id\": \"{}\",\n", verification.requirement_id));
            json.push_str(&format!("    \"verification_method\": \"{}\",\n", verification.verification_method.to_string()));
            json.push_str(&format!("    \"verification_status\": \"{}\",\n", verification.verification_status.to_string()));
            
            if let Some(verified_at) = &verification.verified_at {
                json.push_str(&format!("    \"verified_at\": \"{verified_at}\",\n"));
            } else {
                json.push_str("    \"verified_at\": null,\n");
            }
            
            if let Some(verified_by) = &verification.verified_by {
                json.push_str(&format!("    \"verified_by\": \"{verified_by}\",\n"));
            } else {
                json.push_str("    \"verified_by\": null,\n");
            }
            
            json.push_str(&format!("    \"verification_notes\": \"{}\",\n", verification.verification_notes));
            json.push_str("    \"evidence\": [\n");
            
            for (i, evidence) in verification.evidence.iter().enumerate() {
                if i > 0 {
                    json.push_str(",\n");
                }
                json.push_str("      {\n");
                json.push_str(&format!("        \"evidence_id\": \"{}\",\n", evidence.evidence_id));
                json.push_str(&format!("        \"evidence_type\": \"{}\",\n", evidence.evidence_type));
                json.push_str(&format!("        \"description\": \"{}\",\n", evidence.description));
                json.push_str(&format!("        \"created_at\": \"{}\",\n", evidence.created_at));
                json.push_str(&format!("        \"created_by\": \"{}\",\n", evidence.created_by));
                json.push_str(&format!("        \"method\": \"{}\",\n", evidence.method.to_string()));
                json.push_str(&format!("        \"status\": \"{}\",\n", evidence.status.to_string()));
                json.push_str(&format!("        \"notes\": \"{}\"\n", evidence.notes));
                json.push_str("      }");
            }
            
            json.push_str("\n    ],\n");
            json.push_str("    \"acceptance_criteria\": [\n");
            
            for (i, criteria) in verification.acceptance_criteria.iter().enumerate() {
                if i > 0 {
                    json.push_str(",\n");
                }
                json.push_str(&format!("      \"{criteria}\""));
            }
            
            json.push_str("\n    ],\n");
            json.push_str("    \"test_results\": [\n");
            
            for (i, result) in verification.test_results.iter().enumerate() {
                if i > 0 {
                    json.push_str(",\n");
                }
                json.push_str(&format!("      \"{result}\""));
            }
            
            json.push_str("\n    ]\n");
            json.push_str("  }");
        }
        
        json.push_str("\n}\n");
        Ok(json)
    }
}
