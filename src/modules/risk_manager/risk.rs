//! Risk Item Schema & Storage Implementation
//! Task 3.1.1: Complete risk management system for ISO 14971 compliance
//! 
//! This module implements the comprehensive risk management system required for
//! medical device quality management per ISO 14971 and FDA 21 CFR Part 820.

#![allow(dead_code)] // Allow dead code during development - will be used in future tasks

use crate::prelude::*;
use crate::utils::RiskCalculator; // REFACTORED: Use centralized risk calculator
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// ISO 14971 Risk Severity levels with medical device context
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RiskSeverity {
    Catastrophic = 5,  // Death or permanent disability
    Critical = 4,      // Serious injury or illness requiring medical intervention
    Major = 3,         // Moderate injury requiring treatment
    Minor = 2,         // Minor injury, first aid required
    Negligible = 1,    // No injury expected
}

/// ISO 14971 Risk Occurrence probability levels
#[derive(Debug, Clone, PartialEq)]
pub enum RiskOccurrence {
    Frequent = 5,      // Very likely to occur repeatedly (>1 in 10)
    Probable = 4,      // Will occur several times (1 in 100 to 1 in 10)
    Occasional = 3,    // Likely to occur sometime (1 in 1,000 to 1 in 100)
    Remote = 2,        // Unlikely but possible (1 in 10,000 to 1 in 1,000)
    Improbable = 1,    // So unlikely, assumed not to occur (<1 in 10,000)
}

/// ISO 14971 Risk Detectability levels (inverse scale - lower is better)
#[derive(Debug, Clone, PartialEq)]
pub enum RiskDetectability {
    VeryLow = 5,       // Cannot detect or no controls in place
    Low = 4,           // Poor chance of detection
    Moderate = 3,      // Moderate chance of detection
    High = 2,          // Good chance of detection
    VeryHigh = 1,      // Almost certain detection
}

/// ISO 14971 Risk acceptability levels
#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Unacceptable,      // Requires immediate action, cannot be released
    ALARP,            // As Low As Reasonably Practicable - requires justification
    Acceptable,        // No further action required
}

/// Verification status for risk controls
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationStatus {
    Planned,          // Mitigation planned but not implemented
    InProgress,       // Implementation in progress
    Complete,         // Implementation complete and verified
    Failed,           // Verification failed, needs rework
}

/// Risk lifecycle status for tracking throughout project development
/// Task 3.1.8: Risk Monitoring & Tracking
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RiskStatus {
    Identified,       // Risk has been identified but not assessed
    Assessed,         // Initial risk assessment completed
    Mitigated,        // Mitigation measures implemented
    Verified,         // Mitigation effectiveness verified
    Closed,           // Risk completely addressed and closed
}

/// Risk alert types for monitoring and escalation
#[derive(Debug, Clone, PartialEq)]
pub enum RiskAlert {
    OverdueAssessment,    // Risk assessment is overdue
    PendingVerification,  // Verification is pending
    HighRPN,             // Risk has high RPN value
    OverdueMitigation,   // Mitigation implementation is overdue
    EscalationRequired,  // Risk requires management escalation
}

/// Risk mitigation measure per ISO 14971
#[derive(Debug, Clone)]
pub struct MitigationMeasure {
    pub id: String,                    // UUID v4
    pub description: String,           // Description of mitigation measure
    pub implementation: String,        // How the mitigation is implemented
    pub effectiveness: f32,            // Expected effectiveness (0.0-1.0)
    pub cost: Option<f64>,            // Implementation cost (optional)
    pub timeline: Option<String>,      // Implementation timeline
    pub verification_method: String,   // How mitigation effectiveness is verified
    pub verification_status: VerificationStatus,
    pub verification_evidence: Vec<String>, // Evidence of verification
    pub implementation_status: String, // Planned, InProgress, Implemented, Failed
    pub assigned_to: Option<String>,   // Person responsible for implementation
    pub due_date: Option<String>,      // Due date for implementation (ISO 8601)
    pub implemented_date: Option<String>, // Date implemented (ISO 8601)
    pub verified_date: Option<String>, // Date verified (ISO 8601)
}

/// Comprehensive Risk Item per ISO 14971 requirements
#[derive(Debug, Clone)]
pub struct RiskItem {
    // Core identification
    pub id: String,                    // UUID v4
    pub project_id: String,            // Foreign key to Project
    pub hazard_id: String,             // Unique hazard identifier (HAZ-001, HAZ-002...)
    
    // Risk analysis (ISO 14971 Section 5)
    pub hazard_description: String,    // Description of hazard
    pub hazardous_situation: String,   // Sequence of events leading to harm
    pub harm: String,                  // Type of harm that could result
    
    // Initial risk estimation
    pub severity: RiskSeverity,        // Severity of potential harm
    pub occurrence: RiskOccurrence,    // Probability of occurrence
    pub detectability: RiskDetectability, // Ability to detect before harm
    pub risk_priority_number: u32,     // Calculated RPN (1-125)
    pub initial_risk_level: RiskLevel, // Initial risk acceptability
    
    // Risk controls and mitigation
    pub mitigation_measures: Vec<MitigationMeasure>, // Risk controls
    
    // Residual risk (post-mitigation)
    pub residual_severity: RiskSeverity,       // Severity after mitigation
    pub residual_occurrence: RiskOccurrence,   // Occurrence after mitigation
    pub residual_detectability: RiskDetectability, // Detectability after mitigation
    pub residual_rpn: u32,             // Post-mitigation RPN
    pub residual_risk_level: RiskLevel, // Final risk assessment
    pub residual_risk_justification: Option<String>, // Justification for accepting residual risk
    pub residual_risk_approved: bool,  // Whether residual risk has been approved
    pub residual_risk_approved_by: Option<String>, // Who approved the residual risk
    pub residual_risk_approval_date: Option<String>, // When residual risk was approved
    
    // Verification and validation
    pub verification_method: String,   // How mitigation is verified
    pub verification_status: VerificationStatus, // Current verification status
    pub verification_evidence: Vec<String>, // Links to verification evidence
    
    // Metadata and tracking
    pub category: String,              // Risk category (safety, security, performance)
    pub source: String,               // Source of risk identification
    pub assigned_to: Option<String>,   // User responsible for mitigation
    pub due_date: Option<String>,      // Mitigation due date (ISO 8601)
    pub priority: String,             // Risk priority (Critical, High, Medium, Low)
    pub risk_status: RiskStatus,       // Overall risk lifecycle status (Task 3.1.8)
    pub tags: Vec<String>,            // Searchable tags
    
    // Regulatory mapping
    pub regulatory_references: Vec<String>, // References to regulations
    pub standard_references: Vec<String>,   // References to standards
    
    // Timestamps and audit
    pub created_at: String,            // ISO 8601 timestamp
    pub updated_at: String,            // ISO 8601 timestamp
    pub created_by: String,            // User who created risk
    pub approved_by: Option<String>,   // User who approved risk assessment
    pub approval_date: Option<String>, // Risk assessment approval date
    
    // Post-market surveillance
    pub post_market_data: Vec<String>, // Post-market surveillance information
    pub review_required: bool,         // Whether risk needs periodic review
    pub next_review_date: Option<String>, // Next scheduled review date
}

/// Risk registry index entry for efficient storage and retrieval
#[derive(Debug, Clone)]
pub struct RiskIndexEntry {
    pub id: String,
    pub hazard_id: String,
    pub description: String,
    pub severity: RiskSeverity,
    pub rpn: u32,
    pub risk_level: RiskLevel,
    pub status: VerificationStatus,
    pub created_at: String,
    pub updated_at: String,
}

/// Risk manager for CRUD operations and business logic
pub struct RiskManager {
    project_path: PathBuf,
    risks_dir: PathBuf,
    index_file: PathBuf,
}

impl RiskManager {
    /// Assess risk level based on RPN (static method)
    pub const fn assess_risk_level(rpn: u32) -> RiskLevel {
        match rpn {
            100..=125 => RiskLevel::Unacceptable,
            25..=99 => RiskLevel::ALARP,
            1..=24 => RiskLevel::Acceptable,
            _ => RiskLevel::Acceptable,
        }
    }

    /// Create new risk manager for a project
    pub fn new(project_path: &Path) -> QmsResult<Self> {
        let risks_dir = project_path.join("risks");
        let index_file = risks_dir.join("index.json");
        
        Ok(Self {
            project_path: project_path.to_path_buf(),
            risks_dir,
            index_file,
        })
    }
    
    /// Initialize risk management directory structure
    pub fn initialize(&self) -> QmsResult<()> {
        // Create main risks directory
        fs::create_dir_all(&self.risks_dir)?;
        
        // Create subdirectories per ISO 14971 organization
        fs::create_dir_all(self.risks_dir.join("assessments"))?;
        fs::create_dir_all(self.risks_dir.join("mitigations"))?;
        fs::create_dir_all(self.risks_dir.join("reports"))?;
        fs::create_dir_all(self.risks_dir.join("evidence"))?;
        fs::create_dir_all(self.risks_dir.join("reviews"))?;
        
        // Create empty index if it doesn't exist
        if !self.index_file.exists() {
            let empty_index = RiskIndex {
                version: "1.0".to_string(),
                risks: Vec::new(),
                metadata: HashMap::new(),
            };
            self.save_index(&empty_index)?;
        }
        
        Ok(())
    }
    
    /// Create a new risk item
    pub fn create_risk(&mut self, hazard_desc: &str, situation: &str, harm: &str) -> QmsResult<RiskItem> {
        let id = generate_uuid();
        let hazard_id = self.generate_hazard_id()?;
        let timestamp = crate::utils::current_timestamp_string();
        let current_user = get_current_user()?;
        
        let risk = RiskItem {
            id: id.clone(),
            project_id: get_current_project_id()?,
            hazard_id: hazard_id.clone(),
            hazard_description: hazard_desc.to_string(),
            hazardous_situation: situation.to_string(),
            harm: harm.to_string(),
            severity: RiskSeverity::Minor,
            occurrence: RiskOccurrence::Remote,
            detectability: RiskDetectability::High,
            risk_priority_number: 4, // 2*2*1 = 4
            initial_risk_level: RiskLevel::Acceptable,
            mitigation_measures: Vec::new(),
            residual_severity: RiskSeverity::Minor,
            residual_occurrence: RiskOccurrence::Remote,
            residual_detectability: RiskDetectability::High,
            residual_rpn: 4,
            residual_risk_level: RiskLevel::Acceptable,
            residual_risk_justification: None,
            residual_risk_approved: false,
            residual_risk_approved_by: None,
            residual_risk_approval_date: None,
            verification_method: String::new(),
            verification_status: VerificationStatus::Planned,
            verification_evidence: Vec::new(),
            category: "Safety".to_string(),
            source: "Design Review".to_string(),
            assigned_to: None,
            due_date: None,
            priority: "Medium".to_string(),
            risk_status: RiskStatus::Identified, // New risks start as Identified
            tags: Vec::new(),
            regulatory_references: vec!["ISO 14971".to_string()],
            standard_references: vec!["IEC 62304".to_string()],
            created_at: timestamp.clone(),
            updated_at: timestamp,
            created_by: current_user,
            approved_by: None,
            approval_date: None,
            post_market_data: Vec::new(),
            review_required: true,
            next_review_date: None,
        };
        
        // Save risk to individual file
        self.save_risk(&risk)?;
        
        // Update index
        self.add_to_index(&risk)?;
        
        // Log audit entry
        crate::audit::log_audit(&format!(
            "RISK_CREATED: {} - {} by {}",
            hazard_id, hazard_desc, risk.created_by
        ));
        
        Ok(risk)
    }
    
    /// Load a risk by ID
    pub fn load_risk(&self, risk_id: &str) -> QmsResult<RiskItem> {
        let risk_file = self.risks_dir.join(format!("{risk_id}.json"));
        if !risk_file.exists() {
            return Err(QmsError::NotFound(format!("Risk {risk_id} not found")));
        }
        
        let content = fs::read_to_string(&risk_file)?;
        self.parse_risk_json(&content)
    }
    
    /// Parse risk from JSON content (basic implementation)
    fn parse_risk_json(&self, content: &str) -> QmsResult<RiskItem> {
        // Basic JSON parsing for risk data
        // Look for key fields between quotes
        let id = extract_json_field(content, "id")?;
        let project_id = extract_json_field(content, "project_id")?;
        let hazard_id = extract_json_field(content, "hazard_id")?;
        let hazard_description = extract_json_field(content, "hazard_description")?;
        let hazardous_situation = extract_json_field(content, "hazardous_situation")?;
        let harm = extract_json_field(content, "harm")?;
        let created_at = extract_json_field(content, "created_at")?;
        let updated_at = extract_json_field(content, "updated_at")?;
        let created_by = extract_json_field(content, "created_by")?;
        
        // Parse numeric fields
        let rpn = extract_json_number(content, "risk_priority_number")? as u32;
        
        // Parse enum fields with defaults
        let severity = parse_severity_from_json(content).unwrap_or(RiskSeverity::Minor);
        let occurrence = parse_occurrence_from_json(content).unwrap_or(RiskOccurrence::Remote);
        let detectability = parse_detectability_from_json(content).unwrap_or(RiskDetectability::High);
        let risk_level = parse_risk_level_from_json(content).unwrap_or(RiskLevel::Acceptable);
        let risk_status = parse_risk_status_from_json(content).unwrap_or(RiskStatus::Identified);

        // Parse mitigation measures if present
        let mitigation_measures = parse_mitigation_measures_from_json(content).unwrap_or_else(|_| Vec::new());

        // Parse residual risk fields (with fallback to initial values)
        let residual_severity = parse_severity_from_json_field(content, "residual_severity").unwrap_or(severity.clone());
        let residual_occurrence = parse_occurrence_from_json_field(content, "residual_occurrence").unwrap_or(occurrence.clone());
        let residual_detectability = parse_detectability_from_json_field(content, "residual_detectability").unwrap_or(detectability.clone());
        let residual_rpn = extract_json_number(content, "residual_rpn").unwrap_or(rpn as f64) as u32;

        // Parse verification fields
        let verification_method = extract_json_field(content, "verification_method").unwrap_or_else(|_| String::new());
        let verification_status = parse_verification_status_from_json(content).unwrap_or(VerificationStatus::Planned);
        let verification_evidence = parse_verification_evidence_from_json(content).unwrap_or_else(|_| Vec::new());

        Ok(RiskItem {
            id,
            project_id,
            hazard_id,
            hazard_description,
            hazardous_situation,
            harm,
            severity: severity.clone(),
            occurrence: occurrence.clone(),
            detectability: detectability.clone(),
            risk_priority_number: rpn,
            initial_risk_level: risk_level.clone(),
            mitigation_measures,
            residual_severity,
            residual_occurrence,
            residual_detectability,
            residual_rpn,
            residual_risk_level: RiskManager::assess_risk_level(residual_rpn),
            residual_risk_justification: None,
            residual_risk_approved: false,
            residual_risk_approved_by: None,
            residual_risk_approval_date: None,
            verification_method,
            verification_status,
            verification_evidence,
            category: "Safety".to_string(),
            source: "Design Review".to_string(),
            assigned_to: None,
            due_date: None,
            priority: "Medium".to_string(),
            risk_status, // Parse from JSON for Task 3.1.8
            tags: Vec::new(),
            regulatory_references: vec!["ISO 14971".to_string()],
            standard_references: vec!["IEC 62304".to_string()],
            created_at,
            updated_at,
            created_by,
            approved_by: None,
            approval_date: None,
            post_market_data: Vec::new(),
            review_required: true,
            next_review_date: None,
        })
    }
    
    /// Save a risk to storage
    pub fn save_risk(&self, risk: &RiskItem) -> QmsResult<()> {
        let risk_file = self.risks_dir.join(format!("{}.json", risk.id));

        // Serialize mitigation measures
        let mut mitigations_json = String::new();
        mitigations_json.push_str("[\n");
        for (i, measure) in risk.mitigation_measures.iter().enumerate() {
            if i > 0 {
                mitigations_json.push_str(",\n");
            }
            // Serialize verification evidence array
            let mut evidence_json = String::new();
            evidence_json.push('[');
            for (j, evidence) in measure.verification_evidence.iter().enumerate() {
                if j > 0 {
                    evidence_json.push_str(", ");
                }
                evidence_json.push_str(&format!("\"{}\"", escape_json_string(evidence)));
            }
            evidence_json.push(']');

            mitigations_json.push_str(&format!(r#"        {{
            "id": "{}",
            "description": "{}",
            "implementation": "{}",
            "effectiveness": {},
            "timeline": {},
            "verification_method": "{}",
            "verification_status": {:?},
            "implementation_status": "{}",
            "verification_evidence": {},
            "verified_date": {}
        }}"#,
                measure.id,
                escape_json_string(&measure.description),
                escape_json_string(&measure.implementation),
                measure.effectiveness,
                if let Some(ref timeline) = measure.timeline {
                    format!("\"{}\"", escape_json_string(timeline))
                } else {
                    "null".to_string()
                },
                escape_json_string(&measure.verification_method),
                measure.verification_status,
                escape_json_string(&measure.implementation_status),
                evidence_json,
                if let Some(ref verified_date) = measure.verified_date {
                    format!("\"{}\"", escape_json_string(verified_date))
                } else {
                    "null".to_string()
                }
            ));
        }
        mitigations_json.push_str("\n    ]");

        // Serialize verification evidence array
        let mut verification_evidence_json = String::new();
        verification_evidence_json.push('[');
        for (i, evidence) in risk.verification_evidence.iter().enumerate() {
            if i > 0 {
                verification_evidence_json.push_str(", ");
            }
            verification_evidence_json.push_str(&format!("\"{}\"", escape_json_string(evidence)));
        }
        verification_evidence_json.push(']');

        // Enhanced JSON serialization including mitigation measures and residual risk
        let json_content = format!(r#"{{
    "version": "1.0",
    "data": {{
        "id": "{}",
        "project_id": "{}",
        "hazard_id": "{}",
        "hazard_description": "{}",
        "hazardous_situation": "{}",
        "harm": "{}",
        "severity": {:?},
        "occurrence": {:?},
        "detectability": {:?},
        "risk_priority_number": {},
        "initial_risk_level": {:?},
        "risk_status": {:?},
        "residual_severity": {:?},
        "residual_occurrence": {:?},
        "residual_detectability": {:?},
        "residual_rpn": {},
        "verification_method": "{}",
        "verification_status": {:?},
        "verification_evidence": {},
        "mitigation_measures": {},
        "created_at": "{}",
        "updated_at": "{}",
        "created_by": "{}"
    }}
}}"#,
            risk.id, risk.project_id, risk.hazard_id,
            escape_json_string(&risk.hazard_description),
            escape_json_string(&risk.hazardous_situation),
            escape_json_string(&risk.harm),
            risk.severity, risk.occurrence, risk.detectability,
            risk.risk_priority_number, risk.initial_risk_level,
            risk.risk_status,
            risk.residual_severity, risk.residual_occurrence, risk.residual_detectability,
            risk.residual_rpn,
            escape_json_string(&risk.verification_method),
            risk.verification_status,
            verification_evidence_json,
            mitigations_json,
            risk.created_at, risk.updated_at, risk.created_by
        );

        fs::write(&risk_file, json_content)?;
        Ok(())
    }
    
    /// Generate next hazard ID (HAZ-001, HAZ-002, etc.)
    fn generate_hazard_id(&self) -> QmsResult<String> {
        let index = self.load_index()?;
        let next_num = index.risks.len() + 1;
        Ok(format!("HAZ-{next_num:03}"))
    }
    
    /// Add risk to index for efficient querying
    fn add_to_index(&self, risk: &RiskItem) -> QmsResult<()> {
        let mut index = self.load_index()?;
        
        let entry = RiskIndexEntry {
            id: risk.id.clone(),
            hazard_id: risk.hazard_id.clone(),
            description: risk.hazard_description.clone(),
            severity: risk.severity.clone(),
            rpn: risk.risk_priority_number,
            risk_level: risk.initial_risk_level.clone(),
            status: risk.verification_status.clone(),
            created_at: risk.created_at.clone(),
            updated_at: risk.updated_at.clone(),
        };
        
        index.risks.push(entry);
        self.save_index(&index)?;
        Ok(())
    }
    
    /// Load risk index
    fn load_index(&self) -> QmsResult<RiskIndex> {
        if !self.index_file.exists() {
            return Ok(RiskIndex {
                version: "1.0".to_string(),
                risks: Vec::new(),
                metadata: HashMap::new(),
            });
        }

        let content = fs::read_to_string(&self.index_file)?;

        // Parse JSON content using stdlib-only approach
        use crate::json_utils::JsonValue;
        let json = JsonValue::parse(&content)?;

        let mut risks = Vec::new();
        if let JsonValue::Object(obj) = json {
            if let Some(JsonValue::Array(risks_array)) = obj.get("risks") {
                for risk_value in risks_array {
                    if let JsonValue::Object(risk_obj) = risk_value {
                        // Extract risk index entry fields
                        let id = extract_string_field(risk_obj, "id")?;
                        let hazard_id = extract_string_field(risk_obj, "hazard_id")?;
                        let description = extract_string_field(risk_obj, "description")?;
                        let severity_str = extract_string_field(risk_obj, "severity")?;
                        let rpn = extract_number_field(risk_obj, "rpn")? as u32;
                        let risk_level_str = extract_string_field(risk_obj, "risk_level")?;
                        let status_str = extract_string_field(risk_obj, "status")?;
                        let created_at = extract_string_field(risk_obj, "created_at")?;
                        let updated_at = extract_string_field(risk_obj, "updated_at")?;

                        // Parse enum values
                        let severity = parse_risk_severity(&severity_str);
                        let risk_level = parse_risk_level(&risk_level_str);
                        let status = parse_verification_status(&status_str);

                        let entry = RiskIndexEntry {
                            id,
                            hazard_id,
                            description,
                            severity,
                            rpn,
                            risk_level,
                            status,
                            created_at,
                            updated_at,
                        };

                        risks.push(entry);
                    }
                }
            }
        }

        Ok(RiskIndex {
            version: "1.0".to_string(),
            risks,
            metadata: HashMap::new(),
        })
    }
    
    /// Save risk index
    fn save_index(&self, index: &RiskIndex) -> QmsResult<()> {
        // Serialize the risks array properly
        let mut risks_json = String::new();
        for (i, risk) in index.risks.iter().enumerate() {
            if i > 0 {
                risks_json.push_str(",\n        ");
            }
            risks_json.push_str(&format!(r#"{{
            "id": "{}",
            "hazard_id": "{}",
            "description": "{}",
            "severity": "{}",
            "rpn": {},
            "risk_level": "{}",
            "status": "{}",
            "created_at": "{}",
            "updated_at": "{}"
        }}"#,
                escape_json_string(&risk.id),
                escape_json_string(&risk.hazard_id),
                escape_json_string(&risk.description),
                format!("{:?}", risk.severity),
                risk.rpn,
                format!("{:?}", risk.risk_level),
                format!("{:?}", risk.status),
                escape_json_string(&risk.created_at),
                escape_json_string(&risk.updated_at)
            ));
        }

        let json_content = format!(r#"{{
    "version": "{}",
    "risks": [
        {}
    ],
    "metadata": {{}}
}}"#, index.version, risks_json);

        fs::write(&self.index_file, json_content)?;
        Ok(())
    }
    
    /// List all risks with optional filtering
    pub fn list_risks(&self, filter: Option<&RiskFilter>) -> QmsResult<Vec<RiskIndexEntry>> {
        let index = self.load_index()?;
        let mut risks = index.risks;
        
        if let Some(filter) = filter {
            risks = self.apply_filter(risks, filter);
        }
        
        Ok(risks)
    }
    
    /// List all risks as complete RiskItem objects
    pub fn list_all_risks(&self) -> QmsResult<Vec<RiskItem>> {
        let index_entries = self.list_risks(None)?;
        let mut risks = Vec::new();
        
        for entry in index_entries {
            match self.load_risk(&entry.id) {
                Ok(risk) => risks.push(risk),
                Err(e) => {
                    eprintln!("Warning: Failed to load risk {}: {}", entry.hazard_id, e);
                    continue;
                }
            }
        }
        
        Ok(risks)
    }
    
    /// Update an existing risk
    pub fn update_risk(&mut self, risk: &RiskItem) -> QmsResult<()> {
        // Validate the risk
        risk.validate()?;
        
        // Save the updated risk
        self.save_risk(risk)?;
        
        // Update index entry
        self.update_index_entry(risk)?;
        
        // Log audit entry
        crate::audit::log_audit(&format!(
            "RISK_UPDATED: {} - {} by {}",
            risk.hazard_id, risk.hazard_description, get_current_user()?
        ));
        
        Ok(())
    }
    
    /// Assess a risk by updating severity, occurrence, detectability
    pub fn assess_risk(&mut self, risk_id: &str, severity: Option<RiskSeverity>, 
                       occurrence: Option<RiskOccurrence>, detectability: Option<RiskDetectability>) -> QmsResult<RiskItem> {
        // Load existing risk
        let mut risk = self.load_risk(risk_id)?;
        
        // Update parameters if provided
        if let Some(sev) = severity {
            risk.severity = sev;
        }
        if let Some(occ) = occurrence {
            risk.occurrence = occ;
        }
        if let Some(det) = detectability {
            risk.detectability = det;
        }
        
        // Recalculate RPN and risk level
        risk.calculate_rpn();
        risk.assess_risk_level();
        
        // Update residual risk (initially same as initial risk)
        risk.residual_severity = risk.severity.clone();
        risk.residual_occurrence = risk.occurrence.clone();
        risk.residual_detectability = risk.detectability.clone();
        risk.calculate_residual_rpn();
        
        // Update timestamp
        risk.updated_at = crate::utils::current_timestamp_string();
        
        // Save the updated risk
        self.update_risk(&risk)?;
        
        // Log specific assessment action
        crate::audit::log_audit(&format!(
            "RISK_ASSESSED: {} - RPN: {} -> {}, Level: {:?} by {}",
            risk.hazard_id, risk.risk_priority_number, risk.risk_priority_number, risk.initial_risk_level, get_current_user()?
        ));
        
        Ok(risk)
    }
    
    /// Update index entry for a risk
    fn update_index_entry(&self, risk: &RiskItem) -> QmsResult<()> {
        let mut index = self.load_index()?;
        
        // Find and update existing entry, or add new one
        if let Some(entry) = index.risks.iter_mut().find(|e| e.id == risk.id) {
            entry.description = risk.hazard_description.clone();
            entry.severity = risk.severity.clone();
            entry.rpn = risk.risk_priority_number;
            entry.risk_level = risk.initial_risk_level.clone();
            entry.status = risk.verification_status.clone();
            entry.updated_at = risk.updated_at.clone();
        } else {
            // Add new entry if not found
            let entry = RiskIndexEntry {
                id: risk.id.clone(),
                hazard_id: risk.hazard_id.clone(),
                description: risk.hazard_description.clone(),
                severity: risk.severity.clone(),
                rpn: risk.risk_priority_number,
                risk_level: risk.initial_risk_level.clone(),
                status: risk.verification_status.clone(),
                created_at: risk.created_at.clone(),
                updated_at: risk.updated_at.clone(),
            };
            index.risks.push(entry);
        }
        
        self.save_index(&index)?;
        Ok(())
    }
    
    /// Apply filtering to risk list
    fn apply_filter(&self, mut risks: Vec<RiskIndexEntry>, filter: &RiskFilter) -> Vec<RiskIndexEntry> {
        if let Some(severity) = &filter.severity {
            risks.retain(|r| &r.severity == severity);
        }
        
        if let Some(min_rpn) = filter.min_rpn {
            risks.retain(|r| r.rpn >= min_rpn);
        }
        
        if let Some(risk_level) = &filter.risk_level {
            risks.retain(|r| &r.risk_level == risk_level);
        }
        
        risks
    }
    
    /// Add mitigation measure to a risk
    /// Task 3.1.5: Mitigation Management Implementation
    pub fn add_mitigation_measure(
        &mut self, 
        risk_id: &str, 
        description: &str, 
        effectiveness: f32,
        implementation: Option<&str>,
        timeline: Option<&str>,
        verification_method: Option<&str>
    ) -> QmsResult<MitigationMeasure> {
        // Validate effectiveness range
        let effectiveness_clamped = effectiveness.clamp(0.0, 1.0);
        
        // Load the risk
        let mut risk = self.load_risk(risk_id)?;
        
        // Create new mitigation measure
        let measure = MitigationMeasure {
            id: generate_uuid(),
            description: description.to_string(),
            implementation: implementation.unwrap_or("").to_string(),
            effectiveness: effectiveness_clamped,
            cost: None,
            timeline: timeline.map(|t| t.to_string()),
            verification_method: verification_method.unwrap_or("").to_string(),
            verification_status: VerificationStatus::Planned,
            verification_evidence: Vec::new(),
            implementation_status: "Planned".to_string(),
            assigned_to: None,
            due_date: None,
            implemented_date: None,
            verified_date: None,
        };
        
        // Add to risk's mitigation measures
        risk.mitigation_measures.push(measure.clone());
        
        // Recalculate residual risk with mitigation effectiveness
        self.calculate_residual_risk(&mut risk, effectiveness_clamped)?;
        
        // Update timestamp
        risk.updated_at = crate::utils::current_timestamp_string();
        
        // Save the updated risk
        self.save_risk(&risk)?;
        self.update_index_entry(&risk)?;
        
        // Log audit entry
        crate::audit::log_audit(&format!(
            "MITIGATION_ADDED: {} - {} (effectiveness: {:.1}%) by {}",
            risk.hazard_id, description, effectiveness_clamped * 100.0, get_current_user().unwrap_or_else(|_| "SYSTEM".to_string())
        ));
        
        Ok(measure)
    }
    
    /// Calculate residual risk after applying mitigation effectiveness
    fn calculate_residual_risk(&self, risk: &mut RiskItem, effectiveness: f32) -> QmsResult<()> {
        // Apply mitigation effectiveness to reduce occurrence
        // Effectiveness reduces the occurrence probability
        let reduced_occurrence = self.reduce_occurrence_with_effectiveness(risk.occurrence.clone(), effectiveness);
        
        risk.residual_severity = risk.severity.clone(); // Severity typically unchanged by mitigation
        risk.residual_occurrence = reduced_occurrence;
        risk.residual_detectability = risk.detectability.clone(); // Detection unchanged unless specific detection improvements
        
        // Recalculate residual RPN
        risk.calculate_residual_rpn();
        risk.assess_risk_level();
        
        Ok(())
    }
    
    /// Reduce occurrence based on mitigation effectiveness
    fn reduce_occurrence_with_effectiveness(&self, original: RiskOccurrence, effectiveness: f32) -> RiskOccurrence {
        let original_value = original as u32 as f32;
        let reduced_value = original_value * (1.0 - effectiveness);
        let reduced_int = reduced_value.round() as u32;
        
        // Ensure it's at least 1 (can't eliminate risk completely)
        let final_value = std::cmp::max(1, reduced_int);
        
        match final_value {
            5 => RiskOccurrence::Frequent,
            4 => RiskOccurrence::Probable,
            3 => RiskOccurrence::Occasional,
            2 => RiskOccurrence::Remote,
            _ => RiskOccurrence::Improbable,
        }
    }
    
    /// List mitigation measures for a risk
    pub fn list_mitigations(&self, risk_id: &str) -> QmsResult<Vec<MitigationMeasure>> {
        let risk = self.load_risk(risk_id)?;
        Ok(risk.mitigation_measures)
    }
    
    /// Update mitigation implementation status
    pub fn update_mitigation_status(
        &mut self, 
        risk_id: &str, 
        mitigation_id: &str, 
        status: &str
    ) -> QmsResult<()> {
        let mut risk = self.load_risk(risk_id)?;
        
        // Find and update the mitigation measure
        if let Some(measure) = risk.mitigation_measures.iter_mut().find(|m| m.id == mitigation_id) {
            measure.implementation_status = status.to_string();
            
            if status == "Implemented" {
                measure.implemented_date = Some(crate::utils::current_timestamp_string());
            }
            
            // Update risk timestamp
            risk.updated_at = crate::utils::current_timestamp_string();
            
            // Save the updated risk
            self.save_risk(&risk)?;
            self.update_index_entry(&risk)?;
            
            // Log audit entry
            crate::audit::log_audit(&format!(
                "MITIGATION_STATUS_UPDATED: {} - Mitigation {} status changed to {} by {}",
                risk.hazard_id, mitigation_id, status, get_current_user().unwrap_or_else(|_| "SYSTEM".to_string())
            ));
            
            Ok(())
        } else {
            Err(QmsError::not_found(&format!("Mitigation measure {mitigation_id} not found")))
        }
    }
    
    /// Verify mitigation effectiveness
    pub fn verify_mitigation(
        &mut self, 
        risk_id: &str, 
        mitigation_id: &str, 
        verification_method: &str,
        evidence: &str
    ) -> QmsResult<()> {
        let mut risk = self.load_risk(risk_id)?;
        
        // Find and update the mitigation measure
        if let Some(measure) = risk.mitigation_measures.iter_mut().find(|m| m.id == mitigation_id) {
            measure.verification_status = VerificationStatus::Complete;
            measure.verification_method = verification_method.to_string();
            measure.verification_evidence.push(evidence.to_string());
            measure.verified_date = Some(crate::utils::current_timestamp_string());
            
            // Update risk timestamp
            risk.updated_at = crate::utils::current_timestamp_string();
            
            // Save the updated risk
            self.save_risk(&risk)?;
            self.update_index_entry(&risk)?;
            
            // Log audit entry
            crate::audit::log_audit(&format!(
                "MITIGATION_VERIFIED: {} - Mitigation {} verified via {} by {}",
                risk.hazard_id, mitigation_id, verification_method, get_current_user().unwrap_or_else(|_| "SYSTEM".to_string())
            ));
            
            Ok(())
        } else {
            Err(QmsError::not_found(&format!("Mitigation measure {mitigation_id} not found")))
        }
    }

    /// Task 3.1.9: Verify effectiveness of risk controls
    /// Comprehensive verification of risk control measures with evidence tracking
    pub fn verify_risk_control(
        &mut self,
        risk_id: &str,
        verification_method: &str,
        evidence: &str
    ) -> QmsResult<()> {
        let mut risk = self.load_risk(risk_id)?;
        
        // Update risk verification
        risk.verification_method = verification_method.to_string();
        risk.verification_status = VerificationStatus::Complete;
        risk.verification_evidence.push(evidence.to_string());
        risk.updated_at = crate::utils::current_timestamp_string();
        
        // Update risk status in lifecycle
        if risk.risk_status == RiskStatus::Mitigated {
            risk.risk_status = RiskStatus::Verified;
        }
        
        // Save the updated risk
        self.save_risk(&risk)?;
        self.update_index_entry(&risk)?;
        
        // Log audit entry
        crate::audit::log_audit(&format!(
            "RISK_VERIFIED: {} - Risk control verified via {} with evidence: {}",
            risk.hazard_id, verification_method, evidence
        ));
        
        Ok(())
    }

    /// Task 3.1.9: Validate mitigation effectiveness
    /// Assess whether mitigation measures are working as intended
    pub fn validate_mitigation_effectiveness(
        &mut self,
        risk_id: &str,
        effectiveness_assessment: f32,
        validation_notes: &str
    ) -> QmsResult<()> {
        let mut risk = self.load_risk(risk_id)?;
        
        // Calculate effectiveness across all mitigations
        let mut total_effectiveness = 0.0;
        let mut validated_mitigations = 0;
        
        for measure in &mut risk.mitigation_measures {
            if measure.verification_status == VerificationStatus::Complete {
                // Update effectiveness based on validation
                measure.effectiveness = effectiveness_assessment;
                validated_mitigations += 1;
                total_effectiveness += measure.effectiveness;
            }
        }
        
        // Calculate average effectiveness
        let average_effectiveness = if validated_mitigations > 0 {
            total_effectiveness / validated_mitigations as f32
        } else {
            0.0
        };
        
        // Update risk verification evidence with effectiveness assessment
        risk.verification_evidence.push(format!(
            "Effectiveness validation: {:.1}% effective. Notes: {}",
            average_effectiveness * 100.0,
            validation_notes
        ));
        
        risk.updated_at = crate::utils::current_timestamp_string();
        
        // Save the updated risk
        self.save_risk(&risk)?;
        self.update_index_entry(&risk)?;
        
        // Log audit entry
        crate::audit::log_audit(&format!(
            "MITIGATION_EFFECTIVENESS_VALIDATED: {} - Effectiveness assessed at {:.1}% with notes: {}",
            risk.hazard_id, average_effectiveness * 100.0, validation_notes
        ));
        
        Ok(())
    }

    /// Task 3.1.9: Track verification evidence
    /// Centralized tracking of all verification evidence for a risk
    pub fn track_verification_evidence(
        &mut self,
        risk_id: &str,
        evidence_type: &str,
        evidence_reference: &str,
        description: &str
    ) -> QmsResult<()> {
        let mut risk = self.load_risk(risk_id)?;
        
        // Format evidence entry with metadata
        let evidence_entry = format!(
            "[{}] {} - {} (Added: {})",
            evidence_type,
            evidence_reference,
            description,
            crate::utils::current_timestamp_string()
        );
        
        // Add to verification evidence
        risk.verification_evidence.push(evidence_entry.clone());
        risk.updated_at = crate::utils::current_timestamp_string();



        // Save the updated risk
        self.save_risk(&risk)?;
        self.update_index_entry(&risk)?;
        
        // Log audit entry
        crate::audit::log_audit(&format!(
            "VERIFICATION_EVIDENCE_TRACKED: {} - {} evidence added: {}",
            risk.hazard_id, evidence_type, evidence_reference
        ));
        
        Ok(())
    }
}

/// Risk index for efficient storage and querying
#[derive(Debug, Clone)]
pub struct RiskIndex {
    pub version: String,
    pub risks: Vec<RiskIndexEntry>,
    pub metadata: HashMap<String, String>,
}

/// Filter for risk queries
#[derive(Debug, Default)]
pub struct RiskFilter {
    pub severity: Option<RiskSeverity>,
    pub min_rpn: Option<u32>,
    pub risk_level: Option<RiskLevel>,
}

/// Risk Register Filter for comprehensive filtering and sorting
/// Task 3.1.7: Risk Register
#[derive(Debug, Default)]
pub struct RiskRegisterFilter {
    pub status: Option<String>,           // "open", "closed", "high-priority", "overdue"
    pub severity: Option<RiskSeverity>,   // Filter by severity level
    pub assignee: Option<String>,         // Filter by assigned user
    pub due_date: Option<String>,         // Filter by due date (ISO format)
    pub mitigation_status: Option<String>, // "pending", "in-progress", "complete"
    pub sort_by: String,                  // "rpn:desc", "severity:desc", "created:desc", etc.
}

/// Risk Register Statistics
/// Task 3.1.7: Risk Register
#[derive(Debug)]
pub struct RiskRegisterStats {
    pub total_risks: usize,
    pub open_risks: usize,
    pub high_priority_risks: usize,
    pub overdue_mitigations: usize,
    pub avg_rpn: f64,
    pub severity_distribution: HashMap<String, u32>,
    pub status_distribution: HashMap<String, u32>,
}

// Risk calculation functions per ISO 14971
impl RiskItem {
    /// Calculate Risk Priority Number (RPN)
    /// REFACTORED: Uses centralized RiskCalculator to eliminate DRY violation
    pub fn calculate_rpn(&mut self) {
        self.risk_priority_number = RiskCalculator::calculate_rpn(
            &self.severity,
            &self.occurrence,
            &self.detectability
        );
    }
    
    /// Calculate residual RPN after mitigations
    /// REFACTORED: Uses centralized RiskCalculator to eliminate DRY violation
    pub fn calculate_residual_rpn(&mut self) {
        self.residual_rpn = RiskCalculator::calculate_rpn(
            &self.residual_severity,
            &self.residual_occurrence,
            &self.residual_detectability
        );
    }
    
    /// Assess risk level based on RPN
    pub fn assess_risk_level(&mut self) {
        self.initial_risk_level = Self::rpn_to_risk_level(self.risk_priority_number);
        self.residual_risk_level = Self::rpn_to_risk_level(self.residual_rpn);
    }
    
    /// Convert RPN to risk level per ISO 14971 guidelines
    const fn rpn_to_risk_level(rpn: u32) -> RiskLevel {
        match rpn {
            100..=125 => RiskLevel::Unacceptable,
            25..=99 => RiskLevel::ALARP,
            1..=24 => RiskLevel::Acceptable,
            _ => RiskLevel::Acceptable,
        }
    }
    
    /// Validate risk parameters
    pub fn validate(&self) -> QmsResult<()> {
        if self.hazard_description.is_empty() {
            return Err(QmsError::validation_error("Hazard description cannot be empty"));
        }
        
        if self.hazardous_situation.is_empty() {
            return Err(QmsError::validation_error("Hazardous situation cannot be empty"));
        }
        
        if self.harm.is_empty() {
            return Err(QmsError::validation_error("Harm description cannot be empty"));
        }
        
        if self.risk_priority_number == 0 || self.risk_priority_number > 125 {
            return Err(QmsError::validation_error("RPN must be between 1 and 125"));
        }
        
        Ok(())
    }
    
    /// Check if risk has overdue mitigations
    /// Task 3.1.7: Risk Register
    pub fn is_overdue(&self) -> bool {
        if let Some(due_date_str) = &self.due_date {
            let current_time = crate::utils::current_timestamp_string();
            due_date_str < &current_time
        } else {
            false
        }
    }
}

// Helper functions for risk management
fn generate_uuid() -> String {
    format!("{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        rand_u32(), rand_u16(), rand_u16(), rand_u16(), rand_u64() & 0xffffffffffff)
}

fn rand_u32() -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let mut hasher = DefaultHasher::new();
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
    hasher.finish() as u32
}

fn rand_u16() -> u16 { rand_u32() as u16 }
fn rand_u64() -> u64 { ((rand_u32() as u64) << 32) | (rand_u32() as u64) }

fn get_current_project_id() -> QmsResult<String> {
    Ok("project-001".to_string()) // Placeholder
}

fn get_current_user() -> QmsResult<String> {
    Ok("admin".to_string()) // Placeholder
}

fn escape_json_string(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"', "\\\"")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
     .replace('\t', "\\t")
}

/// Helper function to extract string field from JSON object
fn extract_string_field(obj: &std::collections::HashMap<String, crate::json_utils::JsonValue>, field: &str) -> QmsResult<String> {
    use crate::json_utils::JsonValue;
    match obj.get(field) {
        Some(JsonValue::String(s)) => Ok(s.clone()),
        _ => Err(QmsError::validation_error(&format!("Missing or invalid field: {}", field))),
    }
}

/// Helper function to extract number field from JSON object
fn extract_number_field(obj: &std::collections::HashMap<String, crate::json_utils::JsonValue>, field: &str) -> QmsResult<f64> {
    use crate::json_utils::JsonValue;
    match obj.get(field) {
        Some(JsonValue::Number(n)) => Ok(*n),
        _ => Err(QmsError::validation_error(&format!("Missing or invalid number field: {}", field))),
    }
}

/// Parse RiskSeverity from string
fn parse_risk_severity(s: &str) -> RiskSeverity {
    match s {
        "Negligible" => RiskSeverity::Negligible,
        "Minor" => RiskSeverity::Minor,
        "Major" => RiskSeverity::Major,
        "Critical" => RiskSeverity::Critical,
        "Catastrophic" => RiskSeverity::Catastrophic,
        _ => RiskSeverity::Minor, // Default fallback
    }
}

/// Parse RiskLevel from string
fn parse_risk_level(s: &str) -> RiskLevel {
    match s {
        "Acceptable" => RiskLevel::Acceptable,
        "ALARP" => RiskLevel::ALARP,
        "Unacceptable" => RiskLevel::Unacceptable,
        _ => RiskLevel::Acceptable, // Default fallback
    }
}

/// Parse VerificationStatus from string
fn parse_verification_status(s: &str) -> VerificationStatus {
    match s {
        "Planned" => VerificationStatus::Planned,
        "InProgress" => VerificationStatus::InProgress,
        "Complete" => VerificationStatus::Complete,
        "Failed" => VerificationStatus::Failed,
        _ => VerificationStatus::Planned, // Default fallback
    }
}

/// Risk statistics for comprehensive reporting
#[derive(Debug)]
struct RiskStatistics {
    total_risks: usize,
    high_risk_count: usize,
    medium_risk_count: usize,
    low_risk_count: usize,
    high_risk_percentage: f64,
    medium_risk_percentage: f64,
    low_risk_percentage: f64,
    average_rpn: f64,
    risks_requiring_mitigation: usize,
    mitigated_risks: usize,
    verified_risks: usize,
    verification_percentage: f64,
    severity_distribution: std::collections::HashMap<RiskSeverity, usize>,
}

/// Escape CSV field content to handle commas, quotes, and newlines
/// Task 3.1.7: Risk Register
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        let escaped = s.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        s.to_string()
    }
}

/// Simple JSON serialization for risk items (Risk Register export)
/// Task 3.1.7: Risk Register
fn risk_to_simple_json(risk: &RiskItem) -> String {
    format!(
        "{{\"hazard_id\":\"{}\",\"hazard_description\":\"{}\",\"rpn\":{},\"severity\":\"{:?}\",\"initial_risk_level\":\"{:?}\"}}",
        escape_json_string(&risk.hazard_id),
        escape_json_string(&risk.hazard_description),
        risk.risk_priority_number,
        risk.severity,
        risk.initial_risk_level
    )
}

// JSON parsing helper functions for basic field extraction
fn extract_json_field(content: &str, field: &str) -> QmsResult<String> {
    let pattern = format!("\"{field}\":");
    if let Some(start) = content.find(&pattern) {
        let after_colon = start + pattern.len();
        if let Some(value_start) = content[after_colon..].find('"') {
            let value_start_abs = after_colon + value_start + 1;
            if let Some(value_end) = content[value_start_abs..].find('"') {
                let value = &content[value_start_abs..value_start_abs + value_end];
                return Ok(value.to_string());
            }
        }
    }
    Err(QmsError::validation_error(&format!("Field '{field}' not found in JSON")))
}

fn extract_json_number(content: &str, field: &str) -> QmsResult<f64> {
    let pattern = format!("\"{field}\":");
    if let Some(start) = content.find(&pattern) {
        let after_colon = start + pattern.len();
        let remaining = &content[after_colon..];
        
        // Skip whitespace
        let trimmed = remaining.trim_start();
        
        // Find the number (until comma, newline, or brace)
        let mut end = 0;
        for (i, c) in trimmed.chars().enumerate() {
            if c == ',' || c == '\n' || c == '}' || c == '\r' {
                end = i;
                break;
            }
        }
        
        if end > 0 {
            let number_str = trimmed[..end].trim();
            if let Ok(num) = number_str.parse::<f64>() {
                return Ok(num);
            }
        }
    }
    Err(QmsError::validation_error(&format!("Numeric field '{field}' not found in JSON")))
}

impl RiskManager {
    /// Get comprehensive risk register with filtering and sorting
    /// Task 3.1.7: Risk Register
    pub fn get_risk_register(&self, filter: &RiskRegisterFilter) -> QmsResult<Vec<RiskItem>> {
        let mut risks = self.load_all_risks()?;
        
        // Apply filters
        if let Some(status) = &filter.status {
            risks.retain(|r| {
                match status.as_str() {
                    "open" => r.verification_status != VerificationStatus::Complete,
                    "closed" => r.verification_status == VerificationStatus::Complete,
                    "high-priority" => matches!(r.initial_risk_level, RiskLevel::Unacceptable | RiskLevel::ALARP),
                    "overdue" => r.is_overdue(),
                    _ => true,
                }
            });
        }
        
        if let Some(severity) = &filter.severity {
            risks.retain(|r| r.severity == *severity);
        }
        
        if let Some(assignee) = &filter.assignee {
            risks.retain(|r| {
                r.assigned_to.as_ref() == Some(assignee)
            });
        }
        
        if let Some(due_date) = &filter.due_date {
            risks.retain(|r| {
                r.due_date.as_ref().is_some_and(|d| d <= due_date)
            });
        }
        
        if let Some(mitigation_status) = &filter.mitigation_status {
            risks.retain(|r| {
                r.mitigation_measures.iter().any(|m| {
                    match mitigation_status.as_str() {
                        "pending" => m.implementation_status == "Planned",
                        "in-progress" => m.implementation_status == "InProgress",
                        "complete" => m.implementation_status == "Implemented",
                        _ => true,
                    }
                })
            });
        }
        
        // Apply sorting
        match filter.sort_by.as_str() {
            "rpn:desc" => risks.sort_by(|a, b| b.risk_priority_number.cmp(&a.risk_priority_number)),
            "rpn:asc" => risks.sort_by(|a, b| a.risk_priority_number.cmp(&b.risk_priority_number)),
            "severity:desc" => risks.sort_by(|a, b| (b.severity.clone() as u8).cmp(&(a.severity.clone() as u8))),
            "severity:asc" => risks.sort_by(|a, b| (a.severity.clone() as u8).cmp(&(b.severity.clone() as u8))),
            "created:desc" => risks.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
            "created:asc" => risks.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
            "updated:desc" => risks.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)),
            "updated:asc" => risks.sort_by(|a, b| a.updated_at.cmp(&b.updated_at)),
            _ => {}, // Default: no sorting
        }
        
        Ok(risks)
    }
    
    /// Load all risks from the project directory
    /// Task 3.1.7: Risk Register
    pub fn load_all_risks(&self) -> QmsResult<Vec<RiskItem>> {
        let risks_dir = self.project_path.join("risks");
        let mut risks = Vec::new();
        
        if !risks_dir.exists() {
            return Ok(risks);
        }
        
        // Read all .json files in the risks directory
        for entry in fs::read_dir(risks_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                let filename = path.file_name().unwrap().to_string_lossy();
                
                // Skip index.json and other non-risk files
                if filename == "index.json" {
                    continue;
                }
                
                // Try to extract risk ID from filename
                if let Some(risk_id) = filename.strip_suffix(".json") {
                    if let Ok(risk) = self.load_risk(risk_id) {
                        risks.push(risk);
                    }
                }
            }
        }
        
        Ok(risks)
    }
    
    /// Get risk register statistics
    /// Task 3.1.7: Risk Register
    pub fn get_risk_register_stats(&self) -> QmsResult<RiskRegisterStats> {
        let risks = self.load_all_risks()?;
        
        let total_risks = risks.len();
        let open_risks = risks.iter().filter(|r| r.verification_status != VerificationStatus::Complete).count();
        let high_priority_risks = risks.iter().filter(|r| matches!(r.initial_risk_level, RiskLevel::Unacceptable | RiskLevel::ALARP)).count();
        let overdue_mitigations = risks.iter().filter(|r| r.is_overdue()).count();
        
        let avg_rpn = if !risks.is_empty() {
            risks.iter().map(|r| r.risk_priority_number as f64).sum::<f64>() / risks.len() as f64
        } else {
            0.0
        };
        
        let mut severity_distribution = HashMap::new();
        for risk in &risks {
            let severity_str = format!("{:?}", risk.severity);
            *severity_distribution.entry(severity_str).or_insert(0) += 1;
        }
        
        let mut status_distribution = HashMap::new();
        for risk in &risks {
            let status_str = format!("{:?}", risk.risk_status);
            *status_distribution.entry(status_str).or_insert(0) += 1;
        }
        
        Ok(RiskRegisterStats {
            total_risks,
            open_risks,
            high_priority_risks,
            overdue_mitigations,
            avg_rpn,
            severity_distribution,
            status_distribution,
        })
    }
    
    /// Export risk register to various formats
    /// Task 3.1.7: Risk Register
    pub fn export_risk_register(&self, filter: &RiskRegisterFilter, format: &str, output_path: &str) -> QmsResult<()> {
        let risks = self.get_risk_register(filter)?;
        
        match format.to_lowercase().as_str() {
            "csv" => self.export_risk_register_csv(&risks, output_path),
            "pdf" => self.export_risk_register_pdf(&risks, output_path),
            "json" => self.export_risk_register_json(&risks, output_path),
            _ => Err(QmsError::validation_error(&format!("Unsupported export format: {format}"))),
        }
    }
    
    /// Export risk register to CSV format
    fn export_risk_register_csv(&self, risks: &[RiskItem], output_path: &str) -> QmsResult<()> {
        let mut csv_content = String::new();
        
        // CSV Header
        csv_content.push_str("Risk ID,Hazard Description,Severity,Occurrence,Detectability,RPN,Risk Level,Mitigations,Status,Assigned To,Due Date,Created,Updated\n");
        
        // CSV Data
        for risk in risks {
            let mitigation_count = risk.mitigation_measures.len();
            let assigned_to = risk.assigned_to.as_deref().unwrap_or("Unassigned");
            let due_date = risk.due_date.as_deref().unwrap_or("No due date");
            
            csv_content.push_str(&format!(
                "{},{},{:?},{:?},{:?},{},{:?},{},{:?},{},{},{},{}\n",
                escape_csv(&risk.hazard_id),
                escape_csv(&risk.hazard_description),
                risk.severity,
                risk.occurrence,
                risk.detectability,
                risk.risk_priority_number,
                risk.initial_risk_level,
                mitigation_count,
                risk.verification_status,
                escape_csv(assigned_to),
                escape_csv(due_date),
                escape_csv(&risk.created_at),
                escape_csv(&risk.updated_at)
            ));
        }
        
        crate::fs_utils::atomic_write(Path::new(output_path), &csv_content)?;
        
        println!("📊 Risk register exported to: {output_path}");
        println!("📋 Total risks exported: {}", risks.len());
        
        Ok(())
    }
    
    /// Export comprehensive risk register to PDF format (medical device compliant)
    fn export_risk_register_pdf(&self, risks: &[RiskItem], output_path: &str) -> QmsResult<()> {
        let mut pdf_content = String::new();
        let line_width = 100;
        let timestamp = crate::utils::current_timestamp_string();

        // PDF Header with medical device compliance information
        pdf_content.push_str(&"=".repeat(line_width));
        pdf_content.push('\n');
        pdf_content.push_str(&Self::center_text("MEDICAL DEVICE RISK MANAGEMENT REGISTER", line_width));
        pdf_content.push('\n');
        pdf_content.push_str(&Self::center_text("ISO 14971:2019 Compliant Risk Analysis Report", line_width));
        pdf_content.push('\n');
        pdf_content.push_str(&"=".repeat(line_width));
        pdf_content.push('\n');
        pdf_content.push('\n');

        // Document metadata and compliance information
        pdf_content.push_str("DOCUMENT INFORMATION\n");
        pdf_content.push_str(&"-".repeat(50));
        pdf_content.push('\n');
        pdf_content.push_str(&format!("Generated: {}\n", timestamp));
        pdf_content.push_str(&format!("Total Risks Analyzed: {}\n", risks.len()));
        pdf_content.push_str(&format!("Project ID: {}\n", self.get_project_id()));
        pdf_content.push_str("Regulatory Standard: ISO 14971:2019 - Medical devices — Application of risk management to medical devices\n");
        pdf_content.push_str("Report Type: Risk Register Export\n");
        pdf_content.push_str("Audit Trail: Maintained per 21 CFR Part 820 requirements\n");
        pdf_content.push('\n');

        // Risk statistics and analysis summary
        let stats = Self::calculate_risk_statistics(risks);
        pdf_content.push_str("RISK ANALYSIS SUMMARY\n");
        pdf_content.push_str(&"-".repeat(50));
        pdf_content.push('\n');
        pdf_content.push_str(&format!("Total Risks: {}\n", stats.total_risks));
        pdf_content.push_str(&format!("High Risk (RPN ≥ 50): {} ({:.1}%)\n", stats.high_risk_count, stats.high_risk_percentage));
        pdf_content.push_str(&format!("Medium Risk (RPN 20-49): {} ({:.1}%)\n", stats.medium_risk_count, stats.medium_risk_percentage));
        pdf_content.push_str(&format!("Low Risk (RPN < 20): {} ({:.1}%)\n", stats.low_risk_count, stats.low_risk_percentage));
        pdf_content.push_str(&format!("Average RPN: {:.1}\n", stats.average_rpn));
        pdf_content.push_str(&format!("Risks Requiring Mitigation: {}\n", stats.risks_requiring_mitigation));
        pdf_content.push_str(&format!("Mitigated Risks: {}\n", stats.mitigated_risks));
        pdf_content.push_str(&format!("Verification Complete: {} ({:.1}%)\n", stats.verified_risks, stats.verification_percentage));
        pdf_content.push('\n');

        // Risk severity distribution
        pdf_content.push_str("RISK SEVERITY DISTRIBUTION\n");
        pdf_content.push_str(&"-".repeat(50));
        pdf_content.push('\n');
        for (severity, count) in &stats.severity_distribution {
            let percentage = if stats.total_risks > 0 { (*count as f64 / stats.total_risks as f64) * 100.0 } else { 0.0 };
            pdf_content.push_str(&format!("{:?}: {} ({:.1}%)\n", severity, count, percentage));
        }
        pdf_content.push('\n');

        // Detailed risk entries
        pdf_content.push_str(&"=".repeat(line_width));
        pdf_content.push('\n');
        pdf_content.push_str(&Self::center_text("DETAILED RISK ANALYSIS", line_width));
        pdf_content.push('\n');
        pdf_content.push_str(&"=".repeat(line_width));
        pdf_content.push('\n');
        pdf_content.push('\n');

        for (index, risk) in risks.iter().enumerate() {
            pdf_content.push_str(&format!("RISK #{}: {}\n", index + 1, risk.hazard_id));
            pdf_content.push_str(&"-".repeat(80));
            pdf_content.push('\n');

            // Basic risk information
            pdf_content.push_str(&format!("Hazard Description: {}\n", risk.hazard_description));
            pdf_content.push_str(&format!("Hazardous Situation: {}\n", risk.hazardous_situation));
            pdf_content.push_str(&format!("Potential Harm: {}\n", risk.harm));
            pdf_content.push_str(&format!("Category: {}\n", risk.category));
            pdf_content.push_str(&format!("Source: {}\n", risk.source));
            pdf_content.push('\n');

            // Risk assessment
            pdf_content.push_str("RISK ASSESSMENT:\n");
            pdf_content.push_str(&format!("  Severity: {:?} ({})\n", risk.severity, risk.severity.clone() as u8));
            pdf_content.push_str(&format!("  Occurrence: {:?} ({})\n", risk.occurrence, risk.occurrence.clone() as u8));
            pdf_content.push_str(&format!("  Detectability: {:?} ({})\n", risk.detectability, risk.detectability.clone() as u8));
            pdf_content.push_str(&format!("  Initial RPN: {}\n", risk.risk_priority_number));
            pdf_content.push_str(&format!("  Risk Level: {:?}\n", risk.initial_risk_level));
            pdf_content.push('\n');

            // Mitigation measures
            if !risk.mitigation_measures.is_empty() {
                pdf_content.push_str("MITIGATION MEASURES:\n");
                for (i, mitigation) in risk.mitigation_measures.iter().enumerate() {
                    pdf_content.push_str(&format!("  {}. {:?}\n", i + 1, mitigation));
                }
                pdf_content.push('\n');
            }

            // Residual risk assessment
            pdf_content.push_str("RESIDUAL RISK ASSESSMENT:\n");
            pdf_content.push_str(&format!("  Residual Severity: {:?} ({})\n", risk.residual_severity, risk.residual_severity.clone() as u8));
            pdf_content.push_str(&format!("  Residual Occurrence: {:?} ({})\n", risk.residual_occurrence, risk.residual_occurrence.clone() as u8));
            pdf_content.push_str(&format!("  Residual Detectability: {:?} ({})\n", risk.residual_detectability, risk.residual_detectability.clone() as u8));
            pdf_content.push_str(&format!("  Residual RPN: {}\n", risk.residual_rpn));
            pdf_content.push_str(&format!("  Residual Risk Level: {:?}\n", risk.residual_risk_level));

            if let Some(justification) = &risk.residual_risk_justification {
                pdf_content.push_str(&format!("  Risk Justification: {}\n", justification));
            }

            if risk.residual_risk_approved {
                pdf_content.push_str("  Status: APPROVED");
                if let Some(approved_by) = &risk.residual_risk_approved_by {
                    pdf_content.push_str(&format!(" by {}", approved_by));
                }
                if let Some(approval_date) = &risk.residual_risk_approval_date {
                    pdf_content.push_str(&format!(" on {}", approval_date));
                }
                pdf_content.push('\n');
            } else {
                pdf_content.push_str("  Status: PENDING APPROVAL\n");
            }
            pdf_content.push('\n');

            // Verification and validation
            pdf_content.push_str("VERIFICATION & VALIDATION:\n");
            pdf_content.push_str(&format!("  Method: {}\n", risk.verification_method));
            pdf_content.push_str(&format!("  Status: {:?}\n", risk.verification_status));

            if !risk.verification_evidence.is_empty() {
                pdf_content.push_str("  Evidence:\n");
                for evidence in &risk.verification_evidence {
                    pdf_content.push_str(&format!("    - {}\n", evidence));
                }
            }
            pdf_content.push('\n');

            // Regulatory references
            if !risk.regulatory_references.is_empty() {
                pdf_content.push_str("REGULATORY REFERENCES:\n");
                for reference in &risk.regulatory_references {
                    pdf_content.push_str(&format!("  - {}\n", reference));
                }
                pdf_content.push('\n');
            }

            // Audit trail
            pdf_content.push_str("AUDIT TRAIL:\n");
            pdf_content.push_str(&format!("  Created: {} by {}\n", risk.created_at, risk.created_by));
            pdf_content.push_str(&format!("  Last Updated: {}\n", risk.updated_at));
            if let Some(approved_by) = &risk.approved_by {
                pdf_content.push_str(&format!("  Approved by: {}", approved_by));
                if let Some(approval_date) = &risk.approval_date {
                    pdf_content.push_str(&format!(" on {}", approval_date));
                }
                pdf_content.push('\n');
            }

            pdf_content.push('\n');
            pdf_content.push_str(&"~".repeat(80));
            pdf_content.push('\n');
            pdf_content.push('\n');
        }

        // Footer with compliance statement
        pdf_content.push_str(&"=".repeat(line_width));
        pdf_content.push('\n');
        pdf_content.push_str(&Self::center_text("COMPLIANCE STATEMENT", line_width));
        pdf_content.push('\n');
        pdf_content.push_str(&"=".repeat(line_width));
        pdf_content.push('\n');
        pdf_content.push('\n');
        pdf_content.push_str("This risk management report has been generated in accordance with:\n");
        pdf_content.push_str("• ISO 14971:2019 - Medical devices — Application of risk management to medical devices\n");
        pdf_content.push_str("• 21 CFR Part 820 - Quality System Regulation\n");
        pdf_content.push_str("• IEC 62304 - Medical device software — Software life cycle processes\n");
        pdf_content.push('\n');
        pdf_content.push_str("All risk assessments have been conducted using systematic risk analysis methods.\n");
        pdf_content.push_str("Audit trails are maintained for regulatory compliance and traceability.\n");
        pdf_content.push('\n');
        pdf_content.push_str(&format!("Report generated: {}\n", timestamp));
        pdf_content.push_str("Document format: Text-based PDF (convert using pandoc/wkhtmltopdf for graphical PDF)\n");

        // Write to file with atomic operation for data integrity
        crate::fs_utils::atomic_write(Path::new(output_path), &pdf_content)?;

        println!("📄 Comprehensive risk register PDF report generated: {}", output_path);
        println!("📊 Report includes {} risks with full compliance documentation", risks.len());
        println!("🏥 Medical device regulatory compliance: ISO 14971:2019, 21 CFR Part 820");
        println!("💡 Convert to graphical PDF using: pandoc {} -o {}.pdf", output_path, output_path.trim_end_matches(".txt"));

        Ok(())
    }

    /// Calculate comprehensive risk statistics for reporting
    fn calculate_risk_statistics(risks: &[RiskItem]) -> RiskStatistics {
        let total_risks = risks.len();
        let mut high_risk_count = 0;
        let mut medium_risk_count = 0;
        let mut low_risk_count = 0;
        let mut total_rpn = 0;
        let mut risks_requiring_mitigation = 0;
        let mut mitigated_risks = 0;
        let mut verified_risks = 0;
        let mut severity_distribution = std::collections::HashMap::new();

        for risk in risks {
            // RPN categorization
            if risk.risk_priority_number >= 50 {
                high_risk_count += 1;
            } else if risk.risk_priority_number >= 20 {
                medium_risk_count += 1;
            } else {
                low_risk_count += 1;
            }

            total_rpn += risk.risk_priority_number;

            // Mitigation analysis
            if risk.initial_risk_level == RiskLevel::Unacceptable || risk.risk_priority_number >= 50 {
                risks_requiring_mitigation += 1;
            }

            if !risk.mitigation_measures.is_empty() {
                mitigated_risks += 1;
            }

            // Verification status
            if risk.verification_status == VerificationStatus::Complete {
                verified_risks += 1;
            }

            // Severity distribution
            *severity_distribution.entry(risk.severity.clone()).or_insert(0) += 1;
        }

        let high_risk_percentage = if total_risks > 0 { (high_risk_count as f64 / total_risks as f64) * 100.0 } else { 0.0 };
        let medium_risk_percentage = if total_risks > 0 { (medium_risk_count as f64 / total_risks as f64) * 100.0 } else { 0.0 };
        let low_risk_percentage = if total_risks > 0 { (low_risk_count as f64 / total_risks as f64) * 100.0 } else { 0.0 };
        let average_rpn = if total_risks > 0 { total_rpn as f64 / total_risks as f64 } else { 0.0 };
        let verification_percentage = if total_risks > 0 { (verified_risks as f64 / total_risks as f64) * 100.0 } else { 0.0 };

        RiskStatistics {
            total_risks,
            high_risk_count,
            medium_risk_count,
            low_risk_count,
            high_risk_percentage,
            medium_risk_percentage,
            low_risk_percentage,
            average_rpn,
            risks_requiring_mitigation,
            mitigated_risks,
            verified_risks,
            verification_percentage,
            severity_distribution,
        }
    }

    /// Center text within a given width for PDF formatting
    fn center_text(text: &str, width: usize) -> String {
        if text.len() >= width {
            return text.to_string();
        }

        let padding = (width - text.len()) / 2;
        format!("{}{}", " ".repeat(padding), text)
    }

    /// Export risk register to JSON format
    fn export_risk_register_json(&self, risks: &[RiskItem], output_path: &str) -> QmsResult<()> {
        let mut json_content = String::new();
        json_content.push_str("{\n");
        json_content.push_str("  \"version\": \"1.0\",\n");
        json_content.push_str(&format!("  \"exported_at\": \"{}\",\n", crate::utils::current_timestamp_string()));
        json_content.push_str(&format!("  \"total_risks\": {},\n", risks.len()));
        json_content.push_str("  \"risks\": [\n");
        
        for (i, risk) in risks.iter().enumerate() {
            let comma = if i < risks.len() - 1 { "," } else { "" };
            json_content.push_str(&format!("    {}{}\n", risk_to_simple_json(risk), comma));
        }
        
        json_content.push_str("  ]\n");
        json_content.push_str("}\n");
        
        crate::fs_utils::atomic_write(Path::new(output_path), &json_content)?;
        
        println!("📄 Risk register JSON exported: {output_path}");
        
        Ok(())
    }

    // ===============================================================
    // Task 3.1.8: Risk Monitoring & Tracking Functions
    // ===============================================================

    /// Update risk status through lifecycle: Identified → Assessed → Mitigated → Verified → Closed
    pub fn update_risk_status(&mut self, risk_id: &str, new_status: RiskStatus) -> QmsResult<()> {
        let mut risk = self.load_risk(risk_id)?;
        let old_status = risk.risk_status.clone();
        
        // Validate status transition
        match (&old_status, &new_status) {
            (RiskStatus::Identified, RiskStatus::Assessed) |
            (RiskStatus::Assessed, RiskStatus::Mitigated) |
            (RiskStatus::Mitigated, RiskStatus::Verified) |
            (RiskStatus::Verified, RiskStatus::Closed) |
            // Allow reverse transitions for corrections
            (RiskStatus::Assessed, RiskStatus::Identified) |
            (RiskStatus::Mitigated, RiskStatus::Assessed) |
            (RiskStatus::Verified, RiskStatus::Mitigated) |
            (RiskStatus::Closed, RiskStatus::Verified) => {
                // Valid transition
            }
            _ => {
                return Err(QmsError::validation_error(&format!(
                    "Invalid status transition from {old_status:?} to {new_status:?}"
                )));
            }
        }
        
        risk.risk_status = new_status.clone();
        risk.updated_at = crate::utils::current_timestamp_string();
        
        self.save_risk(&risk)?;
        
        // Log status change
        let _ = crate::modules::audit_logger::log_action(
            "system_user", // TODO: Get actual current user
            crate::models::AuditAction::Update,
            "Risk",
            risk_id,
        );
        
        println!("✅ Risk {} status updated to {:?}", risk.hazard_id, new_status);
        Ok(())
    }

    /// Get overdue risks based on due dates and status
    pub fn get_overdue_risks(&self) -> QmsResult<Vec<RiskItem>> {
        let risks = self.load_all_risks()?;
        let now = crate::utils::current_timestamp_string();
        
        let overdue_risks: Vec<RiskItem> = risks.into_iter()
            .filter(|risk| {
                // Check if risk has a due date and is past due
                if let Some(due_date) = &risk.due_date {
                    // Basic date comparison (ISO 8601 format allows string comparison)
                    due_date < &now && 
                    // Only include risks that are not closed
                    risk.risk_status != RiskStatus::Closed
                } else {
                    false
                }
            })
            .collect();
        
        Ok(overdue_risks)
    }

    /// Get risk status dashboard with counts and metrics
    pub fn risk_status_dashboard(&self) -> QmsResult<HashMap<String, u32>> {
        let risks = self.load_all_risks()?;
        let mut dashboard = HashMap::new();
        
        // Initialize counters
        dashboard.insert("total".to_string(), 0);
        dashboard.insert("identified".to_string(), 0);
        dashboard.insert("assessed".to_string(), 0);
        dashboard.insert("mitigated".to_string(), 0);
        dashboard.insert("verified".to_string(), 0);
        dashboard.insert("closed".to_string(), 0);
        dashboard.insert("overdue".to_string(), 0);
        dashboard.insert("high_rpn".to_string(), 0);
        
        let now = crate::utils::current_timestamp_string();
        
        for risk in risks {
            // Count by status
            let status_key = match risk.risk_status {
                RiskStatus::Identified => "identified",
                RiskStatus::Assessed => "assessed", 
                RiskStatus::Mitigated => "mitigated",
                RiskStatus::Verified => "verified",
                RiskStatus::Closed => "closed",
            };
            *dashboard.get_mut(status_key).unwrap() += 1;
            *dashboard.get_mut("total").unwrap() += 1;
            
            // Count overdue
            if let Some(due_date) = &risk.due_date {
                if due_date < &now && risk.risk_status != RiskStatus::Closed {
                    *dashboard.get_mut("overdue").unwrap() += 1;
                }
            }
            
            // Count high RPN (>= 50)
            if risk.risk_priority_number >= 50 {
                *dashboard.get_mut("high_rpn").unwrap() += 1;
            }
        }
        
        Ok(dashboard)
    }

    /// Escalate high-priority risks that need management attention
    pub fn escalate_high_risks(&self, rpn_threshold: u32) -> QmsResult<Vec<RiskAlert>> {
        let risks = self.load_all_risks()?;
        let mut alerts = Vec::new();
        let now = crate::utils::current_timestamp_string();
        
        for risk in risks {
            // High RPN alert
            if risk.risk_priority_number >= rpn_threshold {
                alerts.push(RiskAlert::HighRPN);
            }
            
            // Overdue assessment (identified but not assessed after 30 days)
            if risk.risk_status == RiskStatus::Identified {
                // In real implementation, would check creation date vs current time
                alerts.push(RiskAlert::OverdueAssessment);
            }
            
            // Pending verification (mitigated but not verified)
            if risk.risk_status == RiskStatus::Mitigated {
                alerts.push(RiskAlert::PendingVerification);
            }
            
            // Overdue mitigation
            if let Some(due_date) = &risk.due_date {
                if due_date < &now && risk.risk_status != RiskStatus::Closed {
                    alerts.push(RiskAlert::OverdueMitigation);
                }
            }
            
            // Escalation required for critical risks not closed
            if matches!(risk.severity, RiskSeverity::Critical | RiskSeverity::Catastrophic) && 
               risk.risk_status != RiskStatus::Closed {
                alerts.push(RiskAlert::EscalationRequired);
            }
        }
        
        Ok(alerts)
    }

    /// Display formatted risk status dashboard
    pub fn display_risk_status_dashboard(&self) -> QmsResult<()> {
        let dashboard = self.risk_status_dashboard()?;
        
        println!("📊 Risk Status Dashboard");
        println!("════════════════════════");
        println!("Total Risks: {}", dashboard.get("total").unwrap_or(&0));
        println!("🔍 Identified: {}", dashboard.get("identified").unwrap_or(&0));
        println!("📋 Assessed: {}", dashboard.get("assessed").unwrap_or(&0));
        println!("🛡️ Mitigated: {}", dashboard.get("mitigated").unwrap_or(&0));
        println!("✅ Verified: {}", dashboard.get("verified").unwrap_or(&0));
        println!("🔒 Closed: {}", dashboard.get("closed").unwrap_or(&0));
        println!("⚠️ Overdue: {}", dashboard.get("overdue").unwrap_or(&0));
        println!("🚨 High RPN (≥50): {}", dashboard.get("high_rpn").unwrap_or(&0));
        
        Ok(())
    }

    /// Display overdue risks with details
    pub fn display_overdue_risks(&self) -> QmsResult<()> {
        let overdue_risks = self.get_overdue_risks()?;
        
        if overdue_risks.is_empty() {
            println!("✅ No overdue risks found");
            return Ok(());
        }
        
        println!("⚠️ Overdue Risks ({} total)", overdue_risks.len());
        println!("═══════════════════════════════════════════════");
        
        for risk in overdue_risks {
            println!("🚨 {} - {}", risk.hazard_id, risk.hazard_description);
            println!("   Status: {:?} | Due: {} | RPN: {}", 
                     risk.risk_status, 
                     risk.due_date.as_deref().unwrap_or("No due date"),
                     risk.risk_priority_number);
            if let Some(assignee) = &risk.assigned_to {
                println!("   Assigned to: {assignee}");
            }
            println!();
        }
        
        Ok(())
    }

    /// Display escalation alerts for management
    pub fn display_escalation_alerts(&self, rpn_threshold: u32) -> QmsResult<()> {
        let alerts = self.escalate_high_risks(rpn_threshold)?;
        
        if alerts.is_empty() {
            println!("✅ No escalation alerts");
            return Ok(());
        }
        
        println!("🚨 Risk Escalation Alerts ({} total)", alerts.len());
        println!("═════════════════════════════════════");
        
        let mut alert_counts = HashMap::new();
        for alert in alerts {
            let count = alert_counts.entry(format!("{alert:?}")).or_insert(0);
            *count += 1;
        }
        
        for (alert_type, count) in alert_counts {
            println!("📢 {alert_type}: {count} instances");
        }
        
        Ok(())
    }

    /// Validate that residual risk is less than or equal to initial risk
    /// Task 3.1.6: Residual Risk Analysis
    pub fn validate_risk_reduction(&self, risk_id: &str) -> QmsResult<bool> {
        let risk = self.load_risk(risk_id)?;
        
        // Residual RPN should be <= initial RPN
        let risk_reduced = risk.residual_rpn <= risk.risk_priority_number;
        
        // Log the validation result
        let message = if risk_reduced {
            format!("Risk {} validation passed: residual RPN {} <= initial RPN {}", 
                   risk_id, risk.residual_rpn, risk.risk_priority_number)
        } else {
            format!("Risk {} validation failed: residual RPN {} > initial RPN {}", 
                   risk_id, risk.residual_rpn, risk.risk_priority_number)
        };
        
        log_audit(&message);
        
        Ok(risk_reduced)
    }
    
    /// Justify acceptance of residual risk and mark for approval
    /// Task 3.1.6: Residual Risk Analysis 
    pub fn justify_residual_risk(&mut self, risk_id: &str, justification: &str) -> QmsResult<RiskItem> {
        let mut risk = self.load_risk(risk_id)?;
        
        // Set justification
        risk.residual_risk_justification = Some(justification.to_string());
        risk.updated_at = crate::utils::current_timestamp_string();
        
        // Check if residual risk level requires approval
        let requires_approval = matches!(risk.residual_risk_level, RiskLevel::ALARP | RiskLevel::Unacceptable);
        
        if requires_approval && !risk.residual_risk_approved {
            println!("⚠️  Warning: Residual risk level {:?} requires Quality Engineer approval", risk.residual_risk_level);
            println!("   Use 'qms risk approve-residual {risk_id}' command to approve this risk");
        }
        
        // Save updated risk
        self.save_risk(&risk)?;
        
        // Log the justification
        let message = format!("Residual risk justified for {risk_id} with: {justification}");
        log_audit(&message);
        
        Ok(risk)
    }
    
    /// Approve residual risk after justification (Quality Engineer function)
    /// Task 3.1.6: Residual Risk Analysis
    pub fn approve_residual_risk(&mut self, risk_id: &str, approver: &str) -> QmsResult<RiskItem> {
        let mut risk = self.load_risk(risk_id)?;
        
        // Check if justification exists
        if risk.residual_risk_justification.is_none() {
            return Err(QmsError::validation_error(
                "Residual risk must be justified before approval"
            ));
        }
        
        // Set approval
        risk.residual_risk_approved = true;
        risk.residual_risk_approved_by = Some(approver.to_string());
        risk.residual_risk_approval_date = Some(crate::utils::current_timestamp_string());
        risk.updated_at = crate::utils::current_timestamp_string();
        
        // Save updated risk
        self.save_risk(&risk)?;
        
        // Log the approval
        let message = format!("Residual risk approved for {risk_id} by {approver}");
        log_audit(&message);
        
        println!("✅ Residual risk approved for {risk_id}");
        
        Ok(risk)
    }
    
    /// Get residual risk analysis summary for a risk
    /// Task 3.1.6: Residual Risk Analysis
    pub fn get_residual_risk_analysis(&self, risk_id: &str) -> QmsResult<String> {
        let risk = self.load_risk(risk_id)?;
        
        let mut analysis = format!("📊 Residual Risk Analysis for {risk_id}\n");
        analysis.push_str("══════════════════════════════════════\n");
        analysis.push_str(&format!("Hazard: {}\n", risk.hazard_description));
        analysis.push_str(&format!("Initial RPN: {} ({:?})\n", risk.risk_priority_number, risk.initial_risk_level));
        analysis.push_str(&format!("Residual RPN: {} ({:?})\n", risk.residual_rpn, risk.residual_risk_level));
        
        let reduction_percent = if risk.risk_priority_number > 0 {
            100.0 * (1.0 - (risk.residual_rpn as f32 / risk.risk_priority_number as f32))
        } else {
            0.0
        };
        analysis.push_str(&format!("Risk Reduction: {reduction_percent:.1}%\n"));
        
        let validation_status = if risk.residual_rpn <= risk.risk_priority_number {
            "✅ PASSED"
        } else {
            "❌ FAILED"
        };
        analysis.push_str(&format!("Validation: {validation_status}\n"));
        
        if let Some(ref justification) = risk.residual_risk_justification {
            analysis.push_str(&format!("Justification: {justification}\n"));
        } else {
            analysis.push_str("Justification: Not provided\n");
        };
        
        let approval_status = if risk.residual_risk_approved {
            format!("✅ APPROVED by {} on {}", 
                   risk.residual_risk_approved_by.as_ref().unwrap_or(&"Unknown".to_string()),
                   risk.residual_risk_approval_date.as_ref().unwrap_or(&"Unknown".to_string()))
        } else {
            "⏳ PENDING APPROVAL".to_string()
        };
        analysis.push_str(&format!("Approval Status: {approval_status}\n"));
        
        analysis.push_str(&format!("Mitigation Count: {}\n", risk.mitigation_measures.len()));
        
        Ok(analysis)
    }

    /// Check if a risk exists by hazard_id
    pub fn risk_exists(&self, hazard_id: &str) -> bool {
        self.load_risk(hazard_id).is_ok()
    }

    /// Get the project ID
    pub fn get_project_id(&self) -> &str {
        // Extract project ID from path
        self.project_path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown_project")
    }

    /// List all risks as RiskItem objects (used by import/export)
    pub fn list_all_risk_items(&self) -> QmsResult<Vec<RiskItem>> {
        self.list_all_risks()
    }

    /// Get high-risk items (RPN >= 50)
    pub fn get_high_risk_items(&self) -> QmsResult<Vec<RiskItem>> {
        let risks = self.list_all_risks()?;
        Ok(risks.into_iter().filter(|r| r.risk_priority_number >= 50).collect())
    }

    /// Get total risk count
    pub fn get_total_risk_count(&self) -> QmsResult<usize> {
        let risks = self.list_all_risks()?;
        Ok(risks.len())
    }

    /// Get high risk count (RPN >= 50)
    pub fn get_high_risk_count(&self) -> QmsResult<usize> {
        let risks = self.list_all_risks()?;
        Ok(risks.iter().filter(|r| r.risk_priority_number >= 50).count())
    }

    /// Get critical risk count (RPN >= 100)
    pub fn get_critical_risk_count(&self) -> QmsResult<usize> {
        let risks = self.list_all_risks()?;
        Ok(risks.iter().filter(|r| r.risk_priority_number >= 100).count())
    }

    /// Get risks by status
    pub fn get_risks_by_status(&self, status: RiskStatus) -> QmsResult<Vec<RiskItem>> {
        let risks = self.list_all_risks()?;
        Ok(risks.into_iter().filter(|r| r.risk_status == status).collect())
    }
}

fn parse_severity_from_json(content: &str) -> Option<RiskSeverity> {
    if content.contains("Catastrophic") { Some(RiskSeverity::Catastrophic) }
    else if content.contains("Critical") { Some(RiskSeverity::Critical) }
    else if content.contains("Major") { Some(RiskSeverity::Major) }
    else if content.contains("Minor") { Some(RiskSeverity::Minor) }
    else if content.contains("Negligible") { Some(RiskSeverity::Negligible) }
    else { None }
}

fn parse_occurrence_from_json(content: &str) -> Option<RiskOccurrence> {
    if content.contains("Frequent") { Some(RiskOccurrence::Frequent) }
    else if content.contains("Probable") { Some(RiskOccurrence::Probable) }
    else if content.contains("Occasional") { Some(RiskOccurrence::Occasional) }
    else if content.contains("Remote") { Some(RiskOccurrence::Remote) }
    else if content.contains("Improbable") { Some(RiskOccurrence::Improbable) }
    else { None }
}

fn parse_detectability_from_json(content: &str) -> Option<RiskDetectability> {
    if content.contains("VeryLow") { Some(RiskDetectability::VeryLow) }
    else if content.contains("Low") { Some(RiskDetectability::Low) }
    else if content.contains("Moderate") { Some(RiskDetectability::Moderate) }
    else if content.contains("High") { Some(RiskDetectability::High) }
    else if content.contains("VeryHigh") { Some(RiskDetectability::VeryHigh) }
    else { None }
}

fn parse_risk_level_from_json(content: &str) -> Option<RiskLevel> {
    if content.contains("Unacceptable") { Some(RiskLevel::Unacceptable) }
    else if content.contains("ALARP") { Some(RiskLevel::ALARP) }
    else if content.contains("Acceptable") { Some(RiskLevel::Acceptable) }
    else { None }
}

fn parse_risk_status_from_json(content: &str) -> Option<RiskStatus> {
    if content.contains("Identified") { Some(RiskStatus::Identified) }
    else if content.contains("Assessed") { Some(RiskStatus::Assessed) }
    else if content.contains("Mitigated") { Some(RiskStatus::Mitigated) }
    else if content.contains("Verified") { Some(RiskStatus::Verified) }
    else if content.contains("Closed") { Some(RiskStatus::Closed) }
    else { None }
}

fn parse_severity_from_json_field(content: &str, field: &str) -> Option<RiskSeverity> {
    let pattern = format!("\"{field}\":");
    if let Some(start) = content.find(&pattern) {
        let after_field = &content[start + pattern.len()..];
        if after_field.contains("Catastrophic") { Some(RiskSeverity::Catastrophic) }
        else if after_field.contains("Critical") { Some(RiskSeverity::Critical) }
        else if after_field.contains("Major") { Some(RiskSeverity::Major) }
        else if after_field.contains("Minor") { Some(RiskSeverity::Minor) }
        else if after_field.contains("Negligible") { Some(RiskSeverity::Negligible) }
        else { None }
    } else {
        None
    }
}

fn parse_occurrence_from_json_field(content: &str, field: &str) -> Option<RiskOccurrence> {
    let pattern = format!("\"{field}\":");
    if let Some(start) = content.find(&pattern) {
        let after_field = &content[start + pattern.len()..];
        if after_field.contains("Frequent") { Some(RiskOccurrence::Frequent) }
        else if after_field.contains("Probable") { Some(RiskOccurrence::Probable) }
        else if after_field.contains("Occasional") { Some(RiskOccurrence::Occasional) }
        else if after_field.contains("Remote") { Some(RiskOccurrence::Remote) }
        else if after_field.contains("Improbable") { Some(RiskOccurrence::Improbable) }
        else { None }
    } else {
        None
    }
}

fn parse_detectability_from_json_field(content: &str, field: &str) -> Option<RiskDetectability> {
    let pattern = format!("\"{field}\":");
    if let Some(start) = content.find(&pattern) {
        let after_field = &content[start + pattern.len()..];
        if after_field.contains("VeryLow") { Some(RiskDetectability::VeryLow) }
        else if after_field.contains("Low") { Some(RiskDetectability::Low) }
        else if after_field.contains("Moderate") { Some(RiskDetectability::Moderate) }
        else if after_field.contains("High") { Some(RiskDetectability::High) }
        else if after_field.contains("VeryHigh") { Some(RiskDetectability::VeryHigh) }
        else { None }
    } else {
        None
    }
}

fn parse_mitigation_measures_from_json(content: &str) -> QmsResult<Vec<MitigationMeasure>> {
    let mut measures = Vec::new();

    // Look for mitigation_measures array in JSON
    if let Some(start) = content.find("\"mitigation_measures\":") {
        let after_colon = start + "\"mitigation_measures\":".len();
        if let Some(array_start) = content[after_colon..].find('[') {
            let array_start_abs = after_colon + array_start + 1;

            // Simple parsing - look for measure objects between { and }
            let mut pos = array_start_abs;
            let content_bytes = content.as_bytes();

            while pos < content.len() {
                // Find next object start
                if let Some(obj_start) = content[pos..].find('{') {
                    let obj_start_abs = pos + obj_start;

                    // Find matching closing brace
                    let mut brace_count = 0;
                    let mut obj_end_abs = obj_start_abs;

                    for i in obj_start_abs..content.len() {
                        match content_bytes[i] {
                            b'{' => brace_count += 1,
                            b'}' => {
                                brace_count -= 1;
                                if brace_count == 0 {
                                    obj_end_abs = i;
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }

                    if obj_end_abs > obj_start_abs {
                        let obj_content = &content[obj_start_abs..=obj_end_abs];

                        // Parse individual mitigation measure
                        if let Ok(measure) = parse_single_mitigation_from_json(obj_content) {
                            measures.push(measure);
                        }

                        pos = obj_end_abs + 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
    }

    Ok(measures)
}

fn parse_single_mitigation_from_json(content: &str) -> QmsResult<MitigationMeasure> {
    let id = extract_json_field(content, "id")?;
    let description = extract_json_field(content, "description")?;
    let implementation = extract_json_field(content, "implementation")?;
    let verification_method = extract_json_field(content, "verification_method")?;
    let implementation_status = extract_json_field(content, "implementation_status")?;

    let effectiveness = extract_json_number(content, "effectiveness")? as f32;

    // Parse timeline (optional field)
    let timeline = extract_json_field(content, "timeline").ok();

    // Parse verification status
    let verification_status = if content.contains("Complete") {
        VerificationStatus::Complete
    } else if content.contains("InProgress") {
        VerificationStatus::InProgress
    } else {
        VerificationStatus::Planned
    };

    // Parse verification evidence array
    let verification_evidence = parse_verification_evidence_from_json(content).unwrap_or_else(|_| Vec::new());

    // Parse verified_date (optional field)
    let verified_date = extract_json_field(content, "verified_date").ok();

    Ok(MitigationMeasure {
        id,
        description,
        implementation,
        effectiveness,
        cost: None,
        timeline,
        verification_method,
        verification_status,
        verification_evidence,
        implementation_status,
        assigned_to: None,
        due_date: None,
        implemented_date: None,
        verified_date,
    })
}

fn parse_verification_evidence_from_json(content: &str) -> QmsResult<Vec<String>> {
    let mut evidence = Vec::new();

    // Look for verification_evidence array in JSON
    if let Some(start) = content.find("\"verification_evidence\":") {
        let after_colon = start + "\"verification_evidence\":".len();
        if let Some(array_start) = content[after_colon..].find('[') {
            let array_start_abs = after_colon + array_start + 1;

            // Find the matching closing bracket for the verification evidence array
            let mut bracket_count = 1; // We've already seen the opening [
            let mut end_pos = array_start_abs;
            let mut in_string = false;
            let mut escape_next = false;

            for (i, ch) in content[array_start_abs..].char_indices() {
                if escape_next {
                    escape_next = false;
                    continue;
                }

                match ch {
                    '\\' if in_string => escape_next = true,
                    '"' => in_string = !in_string,
                    '[' if !in_string => bracket_count += 1,
                    ']' if !in_string => {
                        bracket_count -= 1;
                        if bracket_count == 0 {
                            end_pos = array_start_abs + i;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            if bracket_count == 0 {
                let array_content = &content[array_start_abs..end_pos];

                // Simple parsing - split by quotes and commas
                let mut in_string = false;
                let mut current_string = String::new();
                let mut chars = array_content.chars().peekable();

                while let Some(ch) = chars.next() {
                    match ch {
                        '"' => {
                            if in_string {
                                // End of string
                                if !current_string.is_empty() {
                                    evidence.push(current_string.clone());
                                    current_string.clear();
                                }
                                in_string = false;
                            } else {
                                // Start of string
                                in_string = true;
                            }
                        }
                        '\\' if in_string => {
                            // Handle escaped characters
                            if let Some(next_ch) = chars.next() {
                                match next_ch {
                                    '"' => current_string.push('"'),
                                    '\\' => current_string.push('\\'),
                                    'n' => current_string.push('\n'),
                                    't' => current_string.push('\t'),
                                    _ => {
                                        current_string.push('\\');
                                        current_string.push(next_ch);
                                    }
                                }
                            }
                        }
                        _ if in_string => {
                            current_string.push(ch);
                        }
                        _ => {
                            // Ignore whitespace and commas outside strings
                        }
                    }
                }
            }
        }
    }

    Ok(evidence)
}

fn parse_verification_status_from_json(content: &str) -> Option<VerificationStatus> {
    if content.contains("\"verification_status\": Complete") { Some(VerificationStatus::Complete) }
    else if content.contains("\"verification_status\": InProgress") { Some(VerificationStatus::InProgress) }
    else if content.contains("\"verification_status\": Planned") { Some(VerificationStatus::Planned) }
    else { None }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::env;
    
    fn create_test_project_dir() -> PathBuf {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let test_dir = env::temp_dir().join(format!("qms_risk_test_{}_{}", timestamp, std::process::id()));

        // Remove directory if it exists (cleanup from previous failed tests)
        if test_dir.exists() {
            let _ = fs::remove_dir_all(&test_dir);
        }

        fs::create_dir_all(&test_dir).expect("Failed to create test directory");
        test_dir
    }
    
    #[test]
    fn test_risk_manager_initialization() {
        let test_dir = create_test_project_dir();
        let risk_manager = RiskManager::new(&test_dir).unwrap();
        
        // Initialize should create directory structure
        risk_manager.initialize().unwrap();
        
        // Verify directories exist
        assert!(test_dir.join("risks").exists());
        assert!(test_dir.join("risks/assessments").exists());
        assert!(test_dir.join("risks/mitigations").exists());
        assert!(test_dir.join("risks/reports").exists());
        assert!(test_dir.join("risks/evidence").exists());
        assert!(test_dir.join("risks/reviews").exists());
        assert!(test_dir.join("risks/index.json").exists());
        
        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();
    }
    
    #[test]
    fn test_risk_item_rpn_calculation() {
        let mut risk = RiskItem {
            id: "test-id".to_string(),
            project_id: "test-project".to_string(),
            hazard_id: "HAZ-001".to_string(),
            hazard_description: "Test hazard".to_string(),
            hazardous_situation: "Test situation".to_string(),
            harm: "Test harm".to_string(),
            severity: RiskSeverity::Critical,
            occurrence: RiskOccurrence::Probable,
            detectability: RiskDetectability::Low,
            risk_priority_number: 0,
            initial_risk_level: RiskLevel::Acceptable,
            mitigation_measures: Vec::new(),
            residual_severity: RiskSeverity::Minor,
            residual_occurrence: RiskOccurrence::Remote,
            residual_detectability: RiskDetectability::High,
            residual_rpn: 0,
            residual_risk_level: RiskLevel::Acceptable,
            residual_risk_justification: None,
            residual_risk_approved: false,
            residual_risk_approved_by: None,
            residual_risk_approval_date: None,
            verification_method: String::new(),
            verification_status: VerificationStatus::Planned,
            verification_evidence: Vec::new(),
            category: "Safety".to_string(),
            source: "Test".to_string(),
            assigned_to: None,
            due_date: None,
            priority: "High".to_string(),
            risk_status: RiskStatus::Identified,
            tags: Vec::new(),
            regulatory_references: Vec::new(),
            standard_references: Vec::new(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
            created_by: "test-user".to_string(),
            approved_by: None,
            approval_date: None,
            post_market_data: Vec::new(),
            review_required: false,
            next_review_date: None,
        };
        
        // Test RPN calculation: Critical(4) * Probable(4) * Low(4) = 64
        risk.calculate_rpn();
        assert_eq!(risk.risk_priority_number, 64);
        
        // Test residual RPN: Minor(2) * Remote(2) * High(2) = 8
        risk.calculate_residual_rpn();
        assert_eq!(risk.residual_rpn, 8);
    }
    
    #[test]
    fn test_risk_level_assessment() {
        let mut risk = RiskItem {
            id: "test-id".to_string(),
            project_id: "test-project".to_string(),
            hazard_id: "HAZ-001".to_string(),
            hazard_description: "Test hazard".to_string(),
            hazardous_situation: "Test situation".to_string(),
            harm: "Test harm".to_string(),
            severity: RiskSeverity::Catastrophic,
            occurrence: RiskOccurrence::Frequent,
            detectability: RiskDetectability::VeryLow,
            risk_priority_number: 125, // Maximum RPN
            initial_risk_level: RiskLevel::Acceptable,
            mitigation_measures: Vec::new(),
            residual_severity: RiskSeverity::Minor,
            residual_occurrence: RiskOccurrence::Improbable,
            residual_detectability: RiskDetectability::VeryHigh,
            residual_rpn: 2,
            residual_risk_level: RiskLevel::Acceptable,
            residual_risk_justification: None,
            residual_risk_approved: false,
            residual_risk_approved_by: None,
            residual_risk_approval_date: None,
            verification_method: String::new(),
            verification_status: VerificationStatus::Planned,
            verification_evidence: Vec::new(),
            category: "Safety".to_string(),
            source: "Test".to_string(),
            assigned_to: None,
            due_date: None,
            priority: "Critical".to_string(),
            risk_status: RiskStatus::Identified,
            tags: Vec::new(),
            regulatory_references: Vec::new(),
            standard_references: Vec::new(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
            created_by: "test-user".to_string(),
            approved_by: None,
            approval_date: None,
            post_market_data: Vec::new(),
            review_required: true,
            next_review_date: None,
        };
        
        // Assess risk levels
        risk.assess_risk_level();
        
        // High RPN should be Unacceptable
        assert_eq!(risk.initial_risk_level, RiskLevel::Unacceptable);
        // Low residual RPN should be Acceptable
        assert_eq!(risk.residual_risk_level, RiskLevel::Acceptable);
    }
    
    #[test]
    fn test_risk_validation() {
        let risk = RiskItem {
            id: "test-id".to_string(),
            project_id: "test-project".to_string(),
            hazard_id: "HAZ-001".to_string(),
            hazard_description: String::new(), // Empty - should fail validation
            hazardous_situation: "Test situation".to_string(),
            harm: "Test harm".to_string(),
            severity: RiskSeverity::Minor,
            occurrence: RiskOccurrence::Remote,
            detectability: RiskDetectability::High,
            risk_priority_number: 4,
            initial_risk_level: RiskLevel::Acceptable,
            mitigation_measures: Vec::new(),
            residual_severity: RiskSeverity::Minor,
            residual_occurrence: RiskOccurrence::Remote,
            residual_detectability: RiskDetectability::High,
            residual_rpn: 4,
            residual_risk_level: RiskLevel::Acceptable,
            residual_risk_justification: None,
            residual_risk_approved: false,
            residual_risk_approved_by: None,
            residual_risk_approval_date: None,
            verification_method: String::new(),
            verification_status: VerificationStatus::Planned,
            verification_evidence: Vec::new(),
            category: "Safety".to_string(),
            source: "Test".to_string(),
            assigned_to: None,
            due_date: None,
            priority: "Medium".to_string(),
            risk_status: RiskStatus::Identified,
            tags: Vec::new(),
            regulatory_references: Vec::new(),
            standard_references: Vec::new(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
            created_by: "test-user".to_string(),
            approved_by: None,
            approval_date: None,
            post_market_data: Vec::new(),
            review_required: false,
            next_review_date: None,
        };
        
        // Should fail validation due to empty hazard description
        assert!(risk.validate().is_err());
    }
    
    #[test]
    fn test_hazard_id_generation() {
        let test_dir = create_test_project_dir();
        let risk_manager = RiskManager::new(&test_dir).unwrap();
        risk_manager.initialize().unwrap();
        
        // First hazard ID should be HAZ-001
        let hazard_id = risk_manager.generate_hazard_id().unwrap();
        assert_eq!(hazard_id, "HAZ-001");
        
        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();
    }
    
    #[test]
    fn test_risk_assessment_functionality() {
        let test_dir = create_test_project_dir();
        let mut risk_manager = RiskManager::new(&test_dir).unwrap();
        risk_manager.initialize().unwrap();
        
        // Create a test risk
        let risk = risk_manager.create_risk(
            "Test hazard", 
            "Test situation", 
            "Test harm"
        ).unwrap();
        
        // Initial values should be defaults
        assert_eq!(risk.severity, RiskSeverity::Minor);
        assert_eq!(risk.occurrence, RiskOccurrence::Remote);
        assert_eq!(risk.detectability, RiskDetectability::High);
        assert_eq!(risk.risk_priority_number, 4); // 2*2*1
        assert_eq!(risk.initial_risk_level, RiskLevel::Acceptable);
        
        // Assess the risk with new values
        let updated_risk = risk_manager.assess_risk(
            &risk.id,
            Some(RiskSeverity::Critical),
            Some(RiskOccurrence::Probable),
            Some(RiskDetectability::Low)
        ).unwrap();
        
        // Verify the updated values
        assert_eq!(updated_risk.severity, RiskSeverity::Critical);
        assert_eq!(updated_risk.occurrence, RiskOccurrence::Probable);
        assert_eq!(updated_risk.detectability, RiskDetectability::Low);
        assert_eq!(updated_risk.risk_priority_number, 64); // 4*4*4 = 64
        assert_eq!(updated_risk.initial_risk_level, RiskLevel::ALARP);
        
        // Verify residual risk matches initial
        assert_eq!(updated_risk.residual_severity, updated_risk.severity);
        assert_eq!(updated_risk.residual_occurrence, updated_risk.occurrence);
        assert_eq!(updated_risk.residual_detectability, updated_risk.detectability);
        assert_eq!(updated_risk.residual_rpn, updated_risk.risk_priority_number);
        
        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();
    }
    
    #[test]
    fn test_partial_risk_assessment() {
        let test_dir = create_test_project_dir();
        let mut risk_manager = RiskManager::new(&test_dir).unwrap();
        risk_manager.initialize().unwrap();
        
        // Create a test risk
        let risk = risk_manager.create_risk(
            "Test hazard", 
            "Test situation", 
            "Test harm"
        ).unwrap();
        
        // Update only severity
        let updated_risk = risk_manager.assess_risk(
            &risk.id,
            Some(RiskSeverity::Major),
            None,
            None
        ).unwrap();

        // Verify only severity changed
        assert_eq!(updated_risk.severity, RiskSeverity::Major);
        assert_eq!(updated_risk.occurrence, RiskOccurrence::Remote); // unchanged (2)
        assert_eq!(updated_risk.detectability, RiskDetectability::High); // unchanged (2)
        assert_eq!(updated_risk.risk_priority_number, 12); // 3*2*2 = 12
        
        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();
    }
    
    #[test]
    fn test_risk_loading_and_updating() {
        let test_dir = create_test_project_dir();
        let mut risk_manager = RiskManager::new(&test_dir).unwrap();
        risk_manager.initialize().unwrap();
        
        // Create and save a risk
        let original_risk = risk_manager.create_risk(
            "Load test hazard", 
            "Load test situation", 
            "Load test harm"
        ).unwrap();
        
        // Load the risk back
        let loaded_risk = risk_manager.load_risk(&original_risk.id).unwrap();
        
        // Verify core fields match
        assert_eq!(loaded_risk.id, original_risk.id);
        assert_eq!(loaded_risk.hazard_id, original_risk.hazard_id);
        assert_eq!(loaded_risk.hazard_description, original_risk.hazard_description);
        assert_eq!(loaded_risk.hazardous_situation, original_risk.hazardous_situation);
        assert_eq!(loaded_risk.harm, original_risk.harm);
        
        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();
    }
    
    #[test]
    fn test_mitigation_management() {
        let test_dir = create_test_project_dir();
        let mut risk_manager = RiskManager::new(&test_dir).unwrap();
        risk_manager.initialize().unwrap();
        
        // Create a test risk
        let risk = risk_manager.create_risk(
            "Test hazard for mitigation", 
            "Test situation", 
            "Test harm"
        ).unwrap();
        
        // Add mitigation measure
        let mitigation = risk_manager.add_mitigation_measure(
            &risk.id,
            "Implement input validation",
            0.8,
            Some("Add server-side validation checks"),
            Some("2 weeks"),
            Some("Code review and testing")
        ).unwrap();
        
        // Verify mitigation was created
        assert_eq!(mitigation.description, "Implement input validation");
        assert_eq!(mitigation.effectiveness, 0.8);
        assert_eq!(mitigation.implementation, "Add server-side validation checks");
        assert_eq!(mitigation.timeline, Some("2 weeks".to_string()));
        assert_eq!(mitigation.verification_method, "Code review and testing");
        assert_eq!(mitigation.implementation_status, "Planned");
        
        // List mitigations
        let mitigations = risk_manager.list_mitigations(&risk.id).unwrap();
        assert_eq!(mitigations.len(), 1);
        assert_eq!(mitigations[0].description, "Implement input validation");
        
        // Verify residual risk was calculated
        let updated_risk = risk_manager.load_risk(&risk.id).unwrap();
        assert!(updated_risk.residual_rpn <= updated_risk.risk_priority_number);
        
        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();
    }
    
    #[test]
    fn test_mitigation_verification() {
        let test_dir = create_test_project_dir();
        let mut risk_manager = RiskManager::new(&test_dir).unwrap();
        risk_manager.initialize().unwrap();
        
        // Create risk and add mitigation
        let risk = risk_manager.create_risk(
            "Test hazard for verification", 
            "Test situation", 
            "Test harm"
        ).unwrap();
        
        let mitigation = risk_manager.add_mitigation_measure(
            &risk.id,
            "Implement security checks",
            0.7,
            None,
            None,
            None
        ).unwrap();
        
        // Verify the mitigation
        risk_manager.verify_mitigation(
            &risk.id,
            &mitigation.id,
            "Penetration testing",
            "Security scan passed with 0 vulnerabilities"
        ).unwrap();
        
        // Check verification status
        let mitigations = risk_manager.list_mitigations(&risk.id).unwrap();
        assert_eq!(mitigations.len(), 1);
        assert_eq!(mitigations[0].verification_status, VerificationStatus::Complete);
        assert_eq!(mitigations[0].verification_method, "Penetration testing");
        assert_eq!(mitigations[0].verification_evidence.len(), 1);
        assert_eq!(mitigations[0].verification_evidence[0], "Security scan passed with 0 vulnerabilities");
        assert!(mitigations[0].verified_date.is_some());
        
        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();
    }
    
    #[test]
    fn test_residual_risk_calculation() {
        let test_dir = create_test_project_dir();
        let mut risk_manager = RiskManager::new(&test_dir).unwrap();
        risk_manager.initialize().unwrap();
        
        // Create risk with high RPN
        let risk = risk_manager.create_risk(
            "High risk test", 
            "Critical situation", 
            "Severe harm"
        ).unwrap();
        
        // Set high risk parameters
        let risk = risk_manager.assess_risk(
            &risk.id,
            Some(RiskSeverity::Critical),
            Some(RiskOccurrence::Probable),
            Some(RiskDetectability::Low)
        ).unwrap();
        
        let initial_rpn = risk.risk_priority_number; // 4*4*4 = 64
        assert_eq!(initial_rpn, 64);
        
        // Add highly effective mitigation
        risk_manager.add_mitigation_measure(
            &risk.id,
            "Implement comprehensive safety controls",
            0.9, // 90% effective
            None,
            None,
            None
        ).unwrap();
        
        // Verify residual risk is reduced
        let updated_risk = risk_manager.load_risk(&risk.id).unwrap();
        assert!(updated_risk.residual_rpn < initial_rpn);
        
        // With 90% effectiveness, occurrence should be reduced significantly
        assert!((updated_risk.residual_occurrence as u32) < (risk.occurrence as u32));
        
        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();
    }
    
    #[test]
    fn test_risk_register_filtering() {
        let test_dir = create_test_project_dir();
        let mut risk_manager = RiskManager::new(&test_dir).unwrap();
        risk_manager.initialize().unwrap();
        
        // Create multiple test risks with different properties
        let _risk1 = risk_manager.create_risk(
            "Low severity risk", 
            "Minor issue", 
            "Minimal harm"
        ).unwrap();
        
        let _risk2 = risk_manager.create_risk(
            "High severity risk", 
            "Critical issue", 
            "Severe harm"
        ).unwrap();
        
        // Assess risks with different severities
        risk_manager.assess_risk(
            &_risk1.id,
            Some(RiskSeverity::Minor),
            Some(RiskOccurrence::Remote),
            Some(RiskDetectability::High)
        ).unwrap();
        
        risk_manager.assess_risk(
            &_risk2.id,
            Some(RiskSeverity::Critical),
            Some(RiskOccurrence::Probable),
            Some(RiskDetectability::Low)
        ).unwrap();
        
        // Test filtering by severity
        let mut filter = RiskRegisterFilter::default();
        filter.severity = Some(RiskSeverity::Critical);
        
        let filtered_risks = risk_manager.get_risk_register(&filter).unwrap();
        assert_eq!(filtered_risks.len(), 1);
        assert_eq!(filtered_risks[0].severity, RiskSeverity::Critical);
        
        // Test sorting by RPN descending
        filter.severity = None;
        filter.sort_by = "rpn:desc".to_string();
        
        let sorted_risks = risk_manager.get_risk_register(&filter).unwrap();
        assert_eq!(sorted_risks.len(), 2);
        // First risk should have higher RPN (64 > 4)
        assert!(sorted_risks[0].risk_priority_number >= sorted_risks[1].risk_priority_number);
        
        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();
    }
    
    #[test]
    fn test_risk_register_stats() {
        let test_dir = create_test_project_dir();
        let mut risk_manager = RiskManager::new(&test_dir).unwrap();
        risk_manager.initialize().unwrap();
        
        // Create test risks
        let _risk1 = risk_manager.create_risk(
            "Test risk 1", 
            "Test situation 1", 
            "Test harm 1"
        ).unwrap();
        
        let _risk2 = risk_manager.create_risk(
            "Test risk 2", 
            "Test situation 2", 
            "Test harm 2"
        ).unwrap();
        
        // Get statistics
        let stats = risk_manager.get_risk_register_stats().unwrap();
        
        assert_eq!(stats.total_risks, 2);
        assert_eq!(stats.open_risks, 2); // Both should be planned (open)
        assert!(stats.avg_rpn > 0.0);
        assert!(stats.severity_distribution.len() > 0);
        assert!(stats.status_distribution.len() > 0);
        
        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();
    }
    
    #[test]
    fn test_risk_register_export_csv() {
        let test_dir = create_test_project_dir();
        let mut risk_manager = RiskManager::new(&test_dir).unwrap();
        risk_manager.initialize().unwrap();
        
        // Create test risk
        let _risk = risk_manager.create_risk(
            "Export test risk", 
            "Export test situation", 
            "Export test harm"
        ).unwrap();
        
        // Export to CSV
        let filter = RiskRegisterFilter::default();
        let output_path = test_dir.join("test_export.csv");
        
        risk_manager.export_risk_register(
            &filter, 
            "csv", 
            output_path.to_str().unwrap()
        ).unwrap();
        
        // Verify file was created and contains data
        assert!(output_path.exists());
        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("Risk ID"));
        assert!(content.contains("HAZ-001"));
        assert!(content.contains("Export test risk"));
        
        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_verify_risk_control() {
        let test_dir = create_test_project_dir();
        let mut risk_manager = RiskManager::new(&test_dir).unwrap();
        risk_manager.initialize().unwrap();
        
        // Create test risk
        let risk = risk_manager.create_risk(
            "Test risk for verification",
            "Test situation",
            "Test harm"
        ).unwrap();

        // Follow proper risk lifecycle: Identified → Assessed → Mitigated
        risk_manager.update_risk_status(&risk.id, RiskStatus::Assessed).unwrap();
        risk_manager.update_risk_status(&risk.id, RiskStatus::Mitigated).unwrap();

        // Verify risk control
        let result = risk_manager.verify_risk_control(
            &risk.id,
            "Testing",
            "TC-001 verification passed"
        );
        
        assert!(result.is_ok());
        
        // Verify the risk was updated
        let updated_risk = risk_manager.load_risk(&risk.id).unwrap();
        assert_eq!(updated_risk.verification_method, "Testing");
        assert_eq!(updated_risk.verification_status, VerificationStatus::Complete);
        assert!(updated_risk.verification_evidence.contains(&"TC-001 verification passed".to_string()));
        assert_eq!(updated_risk.risk_status, RiskStatus::Verified);
        
        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_validate_mitigation_effectiveness() {
        let test_dir = create_test_project_dir();
        let mut risk_manager = RiskManager::new(&test_dir).unwrap();
        risk_manager.initialize().unwrap();
        
        // Create test risk
        let risk = risk_manager.create_risk(
            "Test risk for effectiveness validation", 
            "Test situation", 
            "Test harm"
        ).unwrap();
        
        // Add a mitigation
        let _mitigation = risk_manager.add_mitigation_measure(
            &risk.id,
            "Test mitigation",
            0.8,
            Some("Testing implementation"),
            Some("Q2 2025"),
            Some("Testing")
        ).unwrap();
        
        // First verify the mitigation to set status to Complete
        let updated_risk = risk_manager.load_risk(&risk.id).unwrap();
        if let Some(mitigation) = updated_risk.mitigation_measures.first() {
            let _ = risk_manager.verify_mitigation(
                &risk.id,
                &mitigation.id,
                "Testing",
                "Verification completed"
            );
        }
        
        // Validate effectiveness
        let result = risk_manager.validate_mitigation_effectiveness(
            &risk.id,
            0.85,
            "85% effectiveness confirmed through testing"
        );
        
        assert!(result.is_ok());
        
        // Verify evidence was added
        let updated_risk = risk_manager.load_risk(&risk.id).unwrap();
        assert!(updated_risk.verification_evidence.iter()
            .any(|e| e.contains("85% effectiveness confirmed")));
        
        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_track_verification_evidence() {
        let test_dir = create_test_project_dir();
        let mut risk_manager = RiskManager::new(&test_dir).unwrap();
        risk_manager.initialize().unwrap();
        
        // Create test risk
        let risk = risk_manager.create_risk(
            "Test risk for evidence tracking", 
            "Test situation", 
            "Test harm"
        ).unwrap();
        
        // Track verification evidence
        let result = risk_manager.track_verification_evidence(
            &risk.id,
            "TestCase",
            "TC-001",
            "Unit test passed successfully"
        );
        
        assert!(result.is_ok());
        
        // Verify evidence was tracked
        let updated_risk = risk_manager.load_risk(&risk.id).unwrap();



        assert!(updated_risk.verification_evidence.iter()
            .any(|e| e.contains("[TestCase] TC-001 - Unit test passed successfully")));
        
        // Track another piece of evidence
        let result2 = risk_manager.track_verification_evidence(
            &risk.id,
            "Document",
            "DOC-002",
            "Design specification review completed"
        );
        
        assert!(result2.is_ok());
        
        // Verify both pieces of evidence exist
        let final_risk = risk_manager.load_risk(&risk.id).unwrap();
        assert_eq!(final_risk.verification_evidence.len(), 2);
        assert!(final_risk.verification_evidence.iter()
            .any(|e| e.contains("TestCase")));
        assert!(final_risk.verification_evidence.iter()
            .any(|e| e.contains("Document")));
        
        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();
    }
}
