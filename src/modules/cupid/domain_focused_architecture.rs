/// CUPID Principles Enhancement: Domain-Focused Architecture
/// 
/// This module demonstrates domain-focused design by organizing code around
/// business domain concepts rather than technical concerns. Each domain
/// encapsulates its own business rules, terminology, and operations.

use crate::prelude::*;
use std::collections::HashMap;

/// Medical Device Risk Management Domain
/// Encapsulates all business concepts related to medical device risk management
pub mod medical_device_risk_domain {
    use super::*;
    
    /// Domain-specific risk severity aligned with medical device standards
    #[derive(Debug, Clone, PartialEq)]
    pub enum MedicalDeviceRiskSeverity {
        Negligible,      // No injury expected
        Minor,           // Minor injury, first aid required
        Moderate,        // Moderate injury requiring treatment
        Serious,         // Serious injury requiring medical intervention
        Catastrophic,    // Death or permanent disability
    }
    
    impl MedicalDeviceRiskSeverity {
        /// Domain-specific method: Get clinical impact description
        pub fn clinical_impact(&self) -> &'static str {
            match self {
                Self::Negligible => "No clinical impact expected",
                Self::Minor => "Temporary discomfort, no lasting effects",
                Self::Moderate => "Requires medical attention, temporary impairment",
                Self::Serious => "Significant injury, potential permanent effects",
                Self::Catastrophic => "Life-threatening or fatal outcome",
            }
        }
        
        /// Domain-specific method: Get regulatory reporting requirement
        pub fn requires_regulatory_reporting(&self) -> bool {
            matches!(self, Self::Serious | Self::Catastrophic)
        }
        
        /// Domain-specific method: Get patient notification requirement
        pub fn requires_patient_notification(&self) -> bool {
            matches!(self, Self::Moderate | Self::Serious | Self::Catastrophic)
        }
    }
    
    /// Domain-specific hazard classification for medical devices
    #[derive(Debug, Clone, PartialEq)]
    pub enum MedicalDeviceHazardType {
        Biological,      // Infection, contamination
        Chemical,        // Toxic materials, leachables
        Electrical,      // Shock, burn, electromagnetic interference
        Mechanical,      // Sharp edges, moving parts, pressure
        Thermal,         // Burns, hypothermia, hyperthermia
        Radiation,       // Ionizing, non-ionizing radiation
        Software,        // Algorithm errors, cybersecurity
        Usability,       // Use errors, misuse
    }
    
    impl MedicalDeviceHazardType {
        /// Domain-specific method: Get applicable standards
        pub fn applicable_standards(&self) -> Vec<&'static str> {
            match self {
                Self::Biological => vec!["ISO 10993", "ISO 11737"],
                Self::Chemical => vec!["ISO 10993", "USP Class VI"],
                Self::Electrical => vec!["IEC 60601-1", "IEC 62304"],
                Self::Mechanical => vec!["ISO 14971", "IEC 60601-1"],
                Self::Thermal => vec!["IEC 60601-1", "ISO 14971"],
                Self::Radiation => vec!["IEC 60601-2", "21 CFR 1020"],
                Self::Software => vec!["IEC 62304", "ISO 14971"],
                Self::Usability => vec!["IEC 62366", "ISO 14971"],
            }
        }
        
        /// Domain-specific method: Get typical mitigation strategies
        pub fn typical_mitigations(&self) -> Vec<&'static str> {
            match self {
                Self::Biological => vec!["Sterilization", "Biocompatible materials", "Barrier protection"],
                Self::Chemical => vec!["Material selection", "Extraction testing", "Coating"],
                Self::Electrical => vec!["Insulation", "Grounding", "Current limiting"],
                Self::Mechanical => vec!["Guards", "Interlocks", "Rounded edges"],
                Self::Thermal => vec!["Temperature monitoring", "Thermal barriers", "Alarms"],
                Self::Radiation => vec!["Shielding", "Distance", "Time limits"],
                Self::Software => vec!["Validation", "Verification", "Cybersecurity controls"],
                Self::Usability => vec!["User training", "Intuitive design", "Error prevention"],
            }
        }
    }
    
    /// Domain-focused risk assessment for medical devices
    pub struct MedicalDeviceRiskAssessment {
        pub hazard_type: MedicalDeviceHazardType,
        pub severity: MedicalDeviceRiskSeverity,
        pub clinical_context: ClinicalContext,
        pub patient_population: PatientPopulation,
        pub use_environment: UseEnvironment,
    }
    
    impl MedicalDeviceRiskAssessment {
        /// Domain-specific method: Calculate clinical risk score
        pub fn calculate_clinical_risk_score(&self) -> u32 {
            let base_score = match self.severity {
                MedicalDeviceRiskSeverity::Negligible => 1,
                MedicalDeviceRiskSeverity::Minor => 2,
                MedicalDeviceRiskSeverity::Moderate => 3,
                MedicalDeviceRiskSeverity::Serious => 4,
                MedicalDeviceRiskSeverity::Catastrophic => 5,
            };
            
            // Adjust based on patient population vulnerability
            let population_multiplier = match self.patient_population {
                PatientPopulation::Healthy => 1.0,
                PatientPopulation::Chronic => 1.2,
                PatientPopulation::Critical => 1.5,
                PatientPopulation::Pediatric => 1.3,
                PatientPopulation::Elderly => 1.2,
            };
            
            // Adjust based on clinical context
            let context_multiplier = match self.clinical_context {
                ClinicalContext::HomeUse => 1.1,
                ClinicalContext::Hospital => 1.0,
                ClinicalContext::Emergency => 1.4,
                ClinicalContext::Surgery => 1.3,
                ClinicalContext::ICU => 1.5,
            };
            
            (base_score as f64 * population_multiplier * context_multiplier).round() as u32
        }
        
        /// Domain-specific method: Get regulatory pathway impact
        pub fn regulatory_pathway_impact(&self) -> RegulatoryPathwayImpact {
            let clinical_score = self.calculate_clinical_risk_score();
            
            match clinical_score {
                1..=2 => RegulatoryPathwayImpact::ClassI,
                3..=4 => RegulatoryPathwayImpact::ClassII,
                5..=u32::MAX => RegulatoryPathwayImpact::ClassIII,
                _ => RegulatoryPathwayImpact::ClassI,
            }
        }
        
        /// Domain-specific method: Generate clinical evaluation requirements
        pub fn clinical_evaluation_requirements(&self) -> Vec<String> {
            let mut requirements = Vec::new();
            
            match self.severity {
                MedicalDeviceRiskSeverity::Catastrophic | MedicalDeviceRiskSeverity::Serious => {
                    requirements.push("Clinical trial required".to_string());
                    requirements.push("Independent safety monitoring board".to_string());
                }
                MedicalDeviceRiskSeverity::Moderate => {
                    requirements.push("Clinical evaluation report required".to_string());
                    requirements.push("Literature review and clinical data analysis".to_string());
                }
                _ => {
                    requirements.push("Predicate device comparison acceptable".to_string());
                }
            }
            
            if matches!(self.patient_population, PatientPopulation::Pediatric) {
                requirements.push("Pediatric-specific clinical data required".to_string());
            }
            
            requirements
        }
    }
    
    /// Domain-specific clinical context
    #[derive(Debug, Clone, PartialEq)]
    pub enum ClinicalContext {
        HomeUse,
        Hospital,
        Emergency,
        Surgery,
        ICU,
    }
    
    /// Domain-specific patient population
    #[derive(Debug, Clone, PartialEq)]
    pub enum PatientPopulation {
        Healthy,
        Chronic,
        Critical,
        Pediatric,
        Elderly,
    }
    
    /// Domain-specific use environment
    #[derive(Debug, Clone, PartialEq)]
    pub enum UseEnvironment {
        Sterile,
        Clean,
        Uncontrolled,
        Harsh,
    }
    
    /// Domain-specific regulatory pathway impact
    #[derive(Debug, Clone, PartialEq)]
    pub enum RegulatoryPathwayImpact {
        ClassI,    // Low risk - 510(k) exempt or simple 510(k)
        ClassII,   // Moderate risk - 510(k) required
        ClassIII,  // High risk - PMA required
    }
}

/// Quality Management System Domain
/// Encapsulates business concepts related to quality management
pub mod quality_management_domain {
    use super::*;
    
    /// Domain-specific quality process types
    #[derive(Debug, Clone, PartialEq)]
    pub enum QualityProcessType {
        DesignControl,
        ProcessValidation,
        SupplierManagement,
        CAPA,              // Corrective and Preventive Action
        ManagementReview,
        InternalAudit,
        PostMarketSurveillance,
    }
    
    impl QualityProcessType {
        /// Domain-specific method: Get ISO 13485 clause reference
        pub fn iso13485_clause(&self) -> &'static str {
            match self {
                Self::DesignControl => "7.3",
                Self::ProcessValidation => "7.5.6",
                Self::SupplierManagement => "7.4",
                Self::CAPA => "8.5.2, 8.5.3",
                Self::ManagementReview => "5.6",
                Self::InternalAudit => "8.2.2",
                Self::PostMarketSurveillance => "8.2.1",
            }
        }
        
        /// Domain-specific method: Get required documentation
        pub fn required_documentation(&self) -> Vec<&'static str> {
            match self {
                Self::DesignControl => vec!["Design Plan", "Design Input", "Design Output", "Design Review", "Design Verification", "Design Validation", "Design Transfer", "Design Changes"],
                Self::ProcessValidation => vec!["Validation Protocol", "Validation Report", "Process Parameters", "Acceptance Criteria"],
                Self::SupplierManagement => vec!["Supplier Evaluation", "Purchase Orders", "Incoming Inspection", "Supplier Agreements"],
                Self::CAPA => vec!["CAPA Request", "Investigation Report", "Action Plan", "Effectiveness Check"],
                Self::ManagementReview => vec!["Review Agenda", "Review Minutes", "Action Items", "Resource Allocation"],
                Self::InternalAudit => vec!["Audit Plan", "Audit Checklist", "Audit Report", "Nonconformance Reports"],
                Self::PostMarketSurveillance => vec!["Surveillance Plan", "Complaint Records", "Adverse Event Reports", "Trend Analysis"],
            }
        }
    }
    
    /// Domain-focused quality metrics
    pub struct QualityMetrics {
        pub process_type: QualityProcessType,
        pub metrics: HashMap<String, f64>,
        pub targets: HashMap<String, f64>,
        pub measurement_period: String,
    }
    
    impl QualityMetrics {
        /// Domain-specific method: Calculate process capability
        pub fn calculate_process_capability(&self) -> Option<f64> {
            // Cp = (USL - LSL) / (6 * σ)
            let usl = self.targets.get("upper_spec_limit")?;
            let lsl = self.targets.get("lower_spec_limit")?;
            let sigma = self.metrics.get("standard_deviation")?;
            
            Some((usl - lsl) / (6.0 * sigma))
        }
        
        /// Domain-specific method: Assess compliance status
        pub fn assess_compliance_status(&self) -> ComplianceStatus {
            let mut compliant_metrics = 0;
            let mut total_metrics = 0;
            
            for (metric_name, &actual_value) in &self.metrics {
                if let Some(&target_value) = self.targets.get(metric_name) {
                    total_metrics += 1;
                    if actual_value >= target_value {
                        compliant_metrics += 1;
                    }
                }
            }
            
            let compliance_rate = if total_metrics > 0 {
                compliant_metrics as f64 / total_metrics as f64
            } else {
                0.0
            };
            
            match compliance_rate {
                rate if rate >= 0.95 => ComplianceStatus::FullyCompliant,
                rate if rate >= 0.80 => ComplianceStatus::MostlyCompliant,
                rate if rate >= 0.60 => ComplianceStatus::PartiallyCompliant,
                _ => ComplianceStatus::NonCompliant,
            }
        }
    }
    
    /// Domain-specific compliance status
    #[derive(Debug, Clone, PartialEq)]
    pub enum ComplianceStatus {
        FullyCompliant,
        MostlyCompliant,
        PartiallyCompliant,
        NonCompliant,
    }
}

/// Regulatory Affairs Domain
/// Encapsulates business concepts related to regulatory compliance
pub mod regulatory_affairs_domain {
    use super::*;
    
    /// Domain-specific regulatory submission types
    #[derive(Debug, Clone, PartialEq)]
    pub enum RegulatorySubmissionType {
        FDA510k,
        FDAPMA,
        CEMarking,
        HealthCanada,
        PMDA,           // Japan
        NMPA,           // China
        ANVISA,         // Brazil
    }
    
    impl RegulatorySubmissionType {
        /// Domain-specific method: Get required sections
        pub fn required_sections(&self) -> Vec<&'static str> {
            match self {
                Self::FDA510k => vec!["Device Description", "Intended Use", "Substantial Equivalence", "Performance Data", "Labeling", "Risk Analysis"],
                Self::FDAPMA => vec!["Device Description", "Intended Use", "Risk-Benefit Analysis", "Clinical Data", "Manufacturing Information", "Labeling"],
                Self::CEMarking => vec!["Technical Documentation", "Risk Management", "Clinical Evaluation", "Post-Market Surveillance", "Declaration of Conformity"],
                Self::HealthCanada => vec!["Device License Application", "Quality System Certificate", "Clinical Evidence", "Risk Management", "Labeling"],
                Self::PMDA => vec!["Application Form", "Device Overview", "Clinical Data", "Quality Management", "Risk Analysis"],
                Self::NMPA => vec!["Registration Application", "Product Technical Requirements", "Clinical Trial Data", "Quality Management System"],
                Self::ANVISA => vec!["Registration Request", "Technical Documentation", "Clinical Evidence", "Good Manufacturing Practices"],
            }
        }
        
        /// Domain-specific method: Get typical review timeline
        pub fn typical_review_timeline_days(&self) -> u32 {
            match self {
                Self::FDA510k => 90,
                Self::FDAPMA => 180,
                Self::CEMarking => 60,
                Self::HealthCanada => 75,
                Self::PMDA => 120,
                Self::NMPA => 300,
                Self::ANVISA => 180,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use medical_device_risk_domain::*;
    use quality_management_domain::*;
    use regulatory_affairs_domain::*;
    
    #[test]
    fn test_medical_device_risk_domain_focus() {
        let assessment = MedicalDeviceRiskAssessment {
            hazard_type: MedicalDeviceHazardType::Electrical,
            severity: MedicalDeviceRiskSeverity::Serious,
            clinical_context: ClinicalContext::Surgery,
            patient_population: PatientPopulation::Critical,
            use_environment: UseEnvironment::Sterile,
        };
        
        // Domain-specific calculations
        let clinical_score = assessment.calculate_clinical_risk_score();
        assert!(clinical_score > 4); // Should be elevated due to critical patients and surgery context
        
        // Domain-specific regulatory impact
        let pathway = assessment.regulatory_pathway_impact();
        assert_eq!(pathway, RegulatoryPathwayImpact::ClassIII);
        
        // Domain-specific requirements
        let requirements = assessment.clinical_evaluation_requirements();
        assert!(requirements.contains(&"Clinical trial required".to_string()));
    }
    
    #[test]
    fn test_quality_management_domain_focus() {
        let mut metrics = QualityMetrics {
            process_type: QualityProcessType::DesignControl,
            metrics: HashMap::new(),
            targets: HashMap::new(),
            measurement_period: "Q1 2024".to_string(),
        };
        
        // Add domain-specific metrics
        metrics.metrics.insert("design_reviews_completed".to_string(), 8.0);
        metrics.targets.insert("design_reviews_completed".to_string(), 10.0);
        
        // Domain-specific assessment
        let compliance = metrics.assess_compliance_status();
        assert_eq!(compliance, ComplianceStatus::NonCompliant); // 8/10 = 80%, but only one metric
        
        // Domain-specific documentation requirements
        let docs = QualityProcessType::DesignControl.required_documentation();
        assert!(docs.contains(&"Design Plan"));
        assert!(docs.contains(&"Design Verification"));
    }
    
    #[test]
    fn test_regulatory_affairs_domain_focus() {
        let submission = RegulatorySubmissionType::FDA510k;
        
        // Domain-specific requirements
        let sections = submission.required_sections();
        assert!(sections.contains(&"Substantial Equivalence"));
        assert!(sections.contains(&"Performance Data"));
        
        // Domain-specific timeline
        let timeline = submission.typical_review_timeline_days();
        assert_eq!(timeline, 90);
    }
}
