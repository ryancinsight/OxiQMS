use crate::prelude::*;
use std::collections::HashMap;

/// Medical device regulatory standards mapping
#[derive(Debug, Clone, PartialEq)]
pub enum RegulatoryStandard {
    /// FDA 21 CFR Part 820 - Quality System Regulation
    FDA21CFR820,
    /// ISO 13485 - Medical devices - Quality management systems
    ISO13485,
    /// ISO 14971 - Medical devices - Risk management
    ISO14971,
    /// ISO 62304 - Medical device software lifecycle processes
    ISO62304,
    /// IEC 62366 - Medical devices - Usability engineering
    IEC62366,
}

impl RegulatoryStandard {
    pub const fn as_str(&self) -> &'static str {
        match self {
            RegulatoryStandard::FDA21CFR820 => "FDA_21_CFR_820",
            RegulatoryStandard::ISO13485 => "ISO_13485",
            RegulatoryStandard::ISO14971 => "ISO_14971",
            RegulatoryStandard::ISO62304 => "ISO_62304",
            RegulatoryStandard::IEC62366 => "IEC_62366",
        }
    }

    pub const fn description(&self) -> &'static str {
        match self {
            RegulatoryStandard::FDA21CFR820 => "FDA Quality System Regulation for Medical Devices",
            RegulatoryStandard::ISO13485 => "Medical devices - Quality management systems",
            RegulatoryStandard::ISO14971 => "Medical devices - Risk management",
            RegulatoryStandard::ISO62304 => "Medical device software lifecycle processes",
            RegulatoryStandard::IEC62366 => "Medical devices - Usability engineering",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "FDA_21_CFR_820" | "FDA21CFR820" | "21CFR820" => Some(RegulatoryStandard::FDA21CFR820),
            "ISO_13485" | "ISO13485" | "13485" => Some(RegulatoryStandard::ISO13485),
            "ISO_14971" | "ISO14971" | "14971" => Some(RegulatoryStandard::ISO14971),
            "ISO_62304" | "ISO62304" | "62304" => Some(RegulatoryStandard::ISO62304),
            "IEC_62366" | "IEC62366" | "62366" => Some(RegulatoryStandard::IEC62366),
            _ => None,
        }
    }
}

/// Regulatory requirement mapping
#[derive(Debug, Clone)]
pub struct RegulatoryRequirement {
    pub id: String,
    pub standard: RegulatoryStandard,
    pub section: String,
    pub title: String,
    pub description: String,
    pub mandatory: bool,
    pub document_types: Vec<String>,
}

/// Document to regulatory requirement mapping
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RegulatoryMapping {
    pub document_id: String,
    pub requirement_id: String,
    pub standard: RegulatoryStandard,
    pub section: String,
    pub compliance_level: ComplianceLevel,
    pub evidence_description: String,
    pub created_at: u64,
    pub created_by: String,
    pub verified_at: Option<u64>,
    pub verified_by: Option<String>,
}

/// Compliance assessment levels
#[derive(Debug, Clone, PartialEq)]
pub enum ComplianceLevel {
    /// Fully compliant with requirement
    FullyCompliant,
    /// Partially compliant, needs improvement
    PartiallyCompliant,
    /// Not compliant with requirement
    NonCompliant,
    /// Not applicable to this document
    NotApplicable,
    /// Under review/assessment
    UnderReview,
}

impl ComplianceLevel {
    pub const fn as_str(&self) -> &'static str {
        match self {
            ComplianceLevel::FullyCompliant => "fully_compliant",
            ComplianceLevel::PartiallyCompliant => "partially_compliant",
            ComplianceLevel::NonCompliant => "non_compliant",
            ComplianceLevel::NotApplicable => "not_applicable",
            ComplianceLevel::UnderReview => "under_review",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "fully_compliant" | "compliant" | "full" => Some(ComplianceLevel::FullyCompliant),
            "partially_compliant" | "partial" => Some(ComplianceLevel::PartiallyCompliant),
            "non_compliant" | "noncompliant" | "not_compliant" => Some(ComplianceLevel::NonCompliant),
            "not_applicable" | "na" | "n/a" => Some(ComplianceLevel::NotApplicable),
            "under_review" | "review" | "pending" => Some(ComplianceLevel::UnderReview),
            _ => None,
        }
    }
}

/// Compliance status summary
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ComplianceStatus {
    pub standard: RegulatoryStandard,
    pub total_requirements: usize,
    pub compliant_count: usize,
    pub partially_compliant_count: usize,
    pub non_compliant_count: usize,
    pub not_applicable_count: usize,
    pub under_review_count: usize,
    pub compliance_percentage: f64,
    pub coverage_percentage: f64,
}

/// Regulatory mapping manager
pub struct RegulatoryManager {
    requirements: HashMap<String, RegulatoryRequirement>,
    mappings: HashMap<String, Vec<RegulatoryMapping>>,
}

impl Default for RegulatoryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RegulatoryManager {
    pub fn new() -> Self {
        let mut manager = Self {
            requirements: HashMap::new(),
            mappings: HashMap::new(),
        };
        manager.initialize_default_requirements();
        manager
    }

    /// Initialize predefined regulatory requirements
    fn initialize_default_requirements(&mut self) {
        // FDA 21 CFR 820 requirements
        self.add_requirement(RegulatoryRequirement {
            id: "21CFR820.30".to_string(),
            standard: RegulatoryStandard::FDA21CFR820,
            section: "820.30".to_string(),
            title: "Design Controls".to_string(),
            description: "Design controls for medical devices".to_string(),
            mandatory: true,
            document_types: vec!["SRS".to_string(), "SDD".to_string(), "TP".to_string()],
        });

        self.add_requirement(RegulatoryRequirement {
            id: "21CFR820.40".to_string(),
            standard: RegulatoryStandard::FDA21CFR820,
            section: "820.40".to_string(),
            title: "Document Controls".to_string(),
            description: "Document control procedures".to_string(),
            mandatory: true,
            document_types: vec!["SOP".to_string(), "QP".to_string()],
        });

        self.add_requirement(RegulatoryRequirement {
            id: "21CFR820.70".to_string(),
            standard: RegulatoryStandard::FDA21CFR820,
            section: "820.70".to_string(),
            title: "Production and Process Controls".to_string(),
            description: "Production and process controls procedures".to_string(),
            mandatory: true,
            document_types: vec!["SOP".to_string(), "TP".to_string()],
        });

        // ISO 13485 requirements
        self.add_requirement(RegulatoryRequirement {
            id: "ISO13485.4.2.3".to_string(),
            standard: RegulatoryStandard::ISO13485,
            section: "4.2.3".to_string(),
            title: "Control of Documents".to_string(),
            description: "Document control procedures".to_string(),
            mandatory: true,
            document_types: vec!["QM".to_string(), "SOP".to_string()],
        });

        self.add_requirement(RegulatoryRequirement {
            id: "ISO13485.7.3".to_string(),
            standard: RegulatoryStandard::ISO13485,
            section: "7.3".to_string(),
            title: "Design and Development".to_string(),
            description: "Design and development procedures".to_string(),
            mandatory: true,
            document_types: vec!["SRS".to_string(), "SDD".to_string(), "TP".to_string()],
        });

        // ISO 14971 requirements
        self.add_requirement(RegulatoryRequirement {
            id: "ISO14971.4.2".to_string(),
            standard: RegulatoryStandard::ISO14971,
            section: "4.2".to_string(),
            title: "Risk Management Plan".to_string(),
            description: "Risk management plan documentation".to_string(),
            mandatory: true,
            document_types: vec!["RMP".to_string(), "RMF".to_string()],
        });

        self.add_requirement(RegulatoryRequirement {
            id: "ISO14971.4.3".to_string(),
            standard: RegulatoryStandard::ISO14971,
            section: "4.3".to_string(),
            title: "Risk Management File".to_string(),
            description: "Risk management file maintenance".to_string(),
            mandatory: true,
            document_types: vec!["RMF".to_string(), "RA".to_string()],
        });
    }

    /// Add a regulatory requirement
    pub fn add_requirement(&mut self, requirement: RegulatoryRequirement) {
        self.requirements.insert(requirement.id.clone(), requirement);
    }

    /// Add regulatory mapping for a document
    pub fn add_regulatory_mapping(
        &mut self,
        document_id: &str,
        requirement_id: &str,
        compliance_level: ComplianceLevel,
        evidence_description: &str,
        user: &str,
    ) -> QmsResult<()> {
        let requirement = self.requirements.get(requirement_id)
            .ok_or_else(|| QmsError::not_found(&format!("Regulatory requirement: {requirement_id}")))?;

        let mapping = RegulatoryMapping {
            document_id: document_id.to_string(),
            requirement_id: requirement_id.to_string(),
            standard: requirement.standard.clone(),
            section: requirement.section.clone(),
            compliance_level,
            evidence_description: evidence_description.to_string(),
            created_at: current_timestamp(),
            created_by: user.to_string(),
            verified_at: None,
            verified_by: None,
        };

        self.mappings.entry(document_id.to_string())
            .or_default()
            .push(mapping);

        Ok(())
    }

    /// Get regulatory mappings for a document
    pub fn get_document_mappings(&self, document_id: &str) -> Vec<&RegulatoryMapping> {
        self.mappings.get(document_id)
            .map(|mappings| mappings.iter().collect())
            .unwrap_or_default()
    }

    /// Get compliance status for a standard
    pub fn get_compliance_status(&self, standard: &RegulatoryStandard) -> ComplianceStatus {
        let relevant_requirements: Vec<_> = self.requirements.values()
            .filter(|req| &req.standard == standard)
            .collect();

        let total_requirements = relevant_requirements.len();
        let mut compliant_count = 0;
        let mut partially_compliant_count = 0;
        let mut non_compliant_count = 0;
        let mut not_applicable_count = 0;
        let mut under_review_count = 0;
        let mut covered_requirements = 0;

        for requirement in &relevant_requirements {
            let mut found_mapping = false;
            
            for mappings in self.mappings.values() {
                for mapping in mappings {
                    if mapping.requirement_id == requirement.id {
                        found_mapping = true;
                        covered_requirements += 1;
                        match mapping.compliance_level {
                            ComplianceLevel::FullyCompliant => compliant_count += 1,
                            ComplianceLevel::PartiallyCompliant => partially_compliant_count += 1,
                            ComplianceLevel::NonCompliant => non_compliant_count += 1,
                            ComplianceLevel::NotApplicable => not_applicable_count += 1,
                            ComplianceLevel::UnderReview => under_review_count += 1,
                        }
                        break;
                    }
                }
                if found_mapping {
                    break;
                }
            }
        }

        let compliance_percentage = if total_requirements > 0 {
            (compliant_count as f64 / total_requirements as f64) * 100.0
        } else {
            0.0
        };

        let coverage_percentage = if total_requirements > 0 {
            (covered_requirements as f64 / total_requirements as f64) * 100.0
        } else {
            0.0
        };

        ComplianceStatus {
            standard: standard.clone(),
            total_requirements,
            compliant_count,
            partially_compliant_count,
            non_compliant_count,
            not_applicable_count,
            under_review_count,
            compliance_percentage,
            coverage_percentage,
        }
    }

    /// Generate compliance gap analysis
    pub fn generate_gap_analysis(&self, standard: &RegulatoryStandard) -> Vec<String> {
        let mut gaps = Vec::new();
        
        let relevant_requirements: Vec<_> = self.requirements.values()
            .filter(|req| &req.standard == standard)
            .collect();

        for requirement in relevant_requirements {
            let mut has_mapping = false;
            let mut has_compliant_mapping = false;

            for mappings in self.mappings.values() {
                for mapping in mappings {
                    if mapping.requirement_id == requirement.id {
                        has_mapping = true;
                        if mapping.compliance_level == ComplianceLevel::FullyCompliant {
                            has_compliant_mapping = true;
                        }
                        break;
                    }
                }
                if has_compliant_mapping {
                    break;
                }
            }

            if !has_mapping {
                gaps.push(format!("Missing mapping for {} {}: {}", 
                    standard.as_str(), requirement.section, requirement.title));
            } else if !has_compliant_mapping {
                gaps.push(format!("Non-compliant mapping for {} {}: {}",
                    standard.as_str(), requirement.section, requirement.title));
            }
        }

        gaps
    }

    /// List all requirements for a standard
    pub fn list_requirements(&self, standard: &RegulatoryStandard) -> Vec<&RegulatoryRequirement> {
        self.requirements.values()
            .filter(|req| &req.standard == standard)
            .collect()
    }

    /// Verify a regulatory mapping
    pub fn verify_mapping(
        &mut self,
        document_id: &str,
        requirement_id: &str,
        verifier: &str,
    ) -> QmsResult<()> {
        let mappings = self.mappings.get_mut(document_id)
            .ok_or_else(|| QmsError::not_found(&format!("Document mappings: {document_id}")))?;

        for mapping in mappings {
            if mapping.requirement_id == requirement_id {
                mapping.verified_at = Some(current_timestamp());
                mapping.verified_by = Some(verifier.to_string());
                return Ok(());
            }
        }

        Err(QmsError::not_found(&format!("Mapping for requirement: {requirement_id}")))
    }

    /// Generate compliance coverage report
    pub fn generate_coverage_report(&self) -> String {
        let mut report = String::new();
        report.push_str("# Regulatory Compliance Coverage Report\n\n");

        let standards = vec![
            RegulatoryStandard::FDA21CFR820,
            RegulatoryStandard::ISO13485,
            RegulatoryStandard::ISO14971,
        ];

        for standard in standards {
            let status = self.get_compliance_status(&standard);
            report.push_str(&format!("## {} - {}\n\n", standard.as_str(), standard.description()));
            report.push_str(&format!("- **Total Requirements**: {}\n", status.total_requirements));
            report.push_str(&format!("- **Coverage**: {:.1}%\n", status.coverage_percentage));
            report.push_str(&format!("- **Compliance**: {:.1}%\n", status.compliance_percentage));
            report.push_str(&format!("- **Fully Compliant**: {}\n", status.compliant_count));
            report.push_str(&format!("- **Partially Compliant**: {}\n", status.partially_compliant_count));
            report.push_str(&format!("- **Non-Compliant**: {}\n", status.non_compliant_count));
            report.push_str(&format!("- **Not Applicable**: {}\n", status.not_applicable_count));
            report.push_str(&format!("- **Under Review**: {}\n\n", status.under_review_count));
        }

        report
    }
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regulatory_standard_conversions() {
        assert_eq!(RegulatoryStandard::FDA21CFR820.as_str(), "FDA_21_CFR_820");
        assert_eq!(RegulatoryStandard::from_str("FDA21CFR820"), Some(RegulatoryStandard::FDA21CFR820));
        assert_eq!(RegulatoryStandard::from_str("21CFR820"), Some(RegulatoryStandard::FDA21CFR820));
        assert_eq!(RegulatoryStandard::from_str("INVALID"), None);
    }

    #[test]
    fn test_compliance_level_conversions() {
        assert_eq!(ComplianceLevel::FullyCompliant.as_str(), "fully_compliant");
        assert_eq!(ComplianceLevel::from_str("compliant"), Some(ComplianceLevel::FullyCompliant));
        assert_eq!(ComplianceLevel::from_str("partial"), Some(ComplianceLevel::PartiallyCompliant));
        assert_eq!(ComplianceLevel::from_str("invalid"), None);
    }

    #[test]
    fn test_regulatory_manager_initialization() {
        let manager = RegulatoryManager::new();
        assert!(!manager.requirements.is_empty());
        assert!(manager.requirements.contains_key("21CFR820.30"));
        assert!(manager.requirements.contains_key("ISO13485.4.2.3"));
        assert!(manager.requirements.contains_key("ISO14971.4.2"));
    }

    #[test]
    fn test_add_regulatory_mapping() {
        let mut manager = RegulatoryManager::new();
        
        let result = manager.add_regulatory_mapping(
            "doc-001",
            "21CFR820.30",
            ComplianceLevel::FullyCompliant,
            "Document meets all design control requirements",
            "john.doe"
        );
        
        assert!(result.is_ok());
        let mappings = manager.get_document_mappings("doc-001");
        assert_eq!(mappings.len(), 1);
        assert_eq!(mappings[0].requirement_id, "21CFR820.30");
        assert_eq!(mappings[0].compliance_level, ComplianceLevel::FullyCompliant);
    }

    #[test]
    fn test_compliance_status_calculation() {
        let mut manager = RegulatoryManager::new();
        
        // Add some mappings
        manager.add_regulatory_mapping(
            "doc-001", "21CFR820.30", ComplianceLevel::FullyCompliant,
            "Design controls implemented", "user1"
        ).unwrap();
        
        manager.add_regulatory_mapping(
            "doc-002", "21CFR820.40", ComplianceLevel::PartiallyCompliant,
            "Document controls partially implemented", "user1"
        ).unwrap();

        let status = manager.get_compliance_status(&RegulatoryStandard::FDA21CFR820);
        assert_eq!(status.compliant_count, 1);
        assert_eq!(status.partially_compliant_count, 1);
        assert!(status.compliance_percentage > 0.0);
        assert!(status.coverage_percentage > 0.0);
    }

    #[test]
    fn test_gap_analysis() {
        let manager = RegulatoryManager::new();
        let gaps = manager.generate_gap_analysis(&RegulatoryStandard::FDA21CFR820);
        
        // Should have gaps since no mappings are added
        assert!(!gaps.is_empty());
        assert!(gaps.iter().any(|gap| gap.contains("820.30")));
    }

    #[test]
    fn test_verify_mapping() {
        let mut manager = RegulatoryManager::new();
        
        manager.add_regulatory_mapping(
            "doc-001", "21CFR820.30", ComplianceLevel::FullyCompliant,
            "Design controls implemented", "user1"
        ).unwrap();

        let result = manager.verify_mapping("doc-001", "21CFR820.30", "verifier");
        assert!(result.is_ok());

        let mappings = manager.get_document_mappings("doc-001");
        assert!(mappings[0].verified_at.is_some());
        assert_eq!(mappings[0].verified_by, Some("verifier".to_string()));
    }

    #[test]
    fn test_coverage_report_generation() {
        let manager = RegulatoryManager::new();
        let report = manager.generate_coverage_report();
        
        assert!(report.contains("# Regulatory Compliance Coverage Report"));
        assert!(report.contains("FDA_21_CFR_820"));
        assert!(report.contains("ISO_13485"));
        assert!(report.contains("ISO_14971"));
        assert!(report.contains("Total Requirements"));
        assert!(report.contains("Coverage"));
        assert!(report.contains("Compliance"));
    }
}
