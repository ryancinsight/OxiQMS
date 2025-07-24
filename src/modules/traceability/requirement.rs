/*
 * QMS (Quality Management System)
 * Requirement Management Module - Task 3.2.1
 * 
 * Implements comprehensive requirement management system with traceability
 * for medical device quality management per ISO 13485:2016 and FDA 21 CFR Part 820
 * 
 * Features:
 * - Complete requirement schema with validation
 * - Storage structure for requirements, test cases, and traceability links
 * - JSON serialization for persistence
 * - Regulatory mapping support
 * - Hierarchical requirement relationships
 * 
 * Author: QMS Development Team
 * Date: January 2025
 * Version: 1.0.0
 */

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use crate::prelude::*;
use crate::modules::document_control::document::RegulatoryReference;
use crate::modules::audit_logger::functions::audit_log_create;
use crate::utils::{generate_uuid, current_timestamp};

/// Requirement category classification
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RequirementCategory {
    Functional,      // Functional requirements
    Performance,     // Performance requirements
    Usability,       // User experience requirements
    Reliability,     // Reliability requirements
    Safety,          // Safety requirements
    Security,        // Security requirements
    Regulatory,      // Regulatory compliance requirements
    Interface,       // Interface requirements
    Data,            // Data requirements
    System,          // System requirements
    Other(String),   // Custom categories
}

impl RequirementCategory {
    pub fn as_str(&self) -> &str {
        match self {
            RequirementCategory::Functional => "functional",
            RequirementCategory::Performance => "performance",
            RequirementCategory::Usability => "usability",
            RequirementCategory::Reliability => "reliability",
            RequirementCategory::Safety => "safety",
            RequirementCategory::Security => "security",
            RequirementCategory::Regulatory => "regulatory",
            RequirementCategory::Interface => "interface",
            RequirementCategory::Data => "data",
            RequirementCategory::System => "system",
            RequirementCategory::Other(name) => name,
        }
    }
    
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "functional" => RequirementCategory::Functional,
            "performance" => RequirementCategory::Performance,
            "usability" => RequirementCategory::Usability,
            "reliability" => RequirementCategory::Reliability,
            "safety" => RequirementCategory::Safety,
            "security" => RequirementCategory::Security,
            "regulatory" => RequirementCategory::Regulatory,
            "interface" => RequirementCategory::Interface,
            "data" => RequirementCategory::Data,
            "system" => RequirementCategory::System,
            _ => RequirementCategory::Other(s.to_string()),
        }
    }
}

/// Requirement priority classification
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RequirementPriority {
    Critical,    // Must have - system fails without it
    High,        // Should have - important for success
    Medium,      // Could have - nice to have
    Low,         // Won't have this time - future consideration
}

impl RequirementPriority {
    pub const fn as_str(&self) -> &str {
        match self {
            RequirementPriority::Critical => "critical",
            RequirementPriority::High => "high",
            RequirementPriority::Medium => "medium",
            RequirementPriority::Low => "low",
        }
    }
    
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "critical" => RequirementPriority::Critical,
            "high" => RequirementPriority::High,
            "medium" => RequirementPriority::Medium,
            "low" => RequirementPriority::Low,
            _ => RequirementPriority::Medium,
        }
    }
}

/// Verification method for requirements
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VerificationMethod {
    Test,           // Execute test cases
    Analysis,       // Mathematical or simulation analysis
    Inspection,     // Visual or measurement inspection
    Demonstration,  // Demonstrate functionality
    Review,         // Design review
}

impl VerificationMethod {
    pub const fn as_str(&self) -> &str {
        match self {
            VerificationMethod::Test => "test",
            VerificationMethod::Analysis => "analysis",
            VerificationMethod::Inspection => "inspection",
            VerificationMethod::Demonstration => "demonstration",
            VerificationMethod::Review => "review",
        }
    }
    
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "test" => VerificationMethod::Test,
            "analysis" => VerificationMethod::Analysis,
            "inspection" => VerificationMethod::Inspection,
            "demonstration" => VerificationMethod::Demonstration,
            "review" => VerificationMethod::Review,
            _ => VerificationMethod::Test,
        }
    }
}

/// Requirement status lifecycle
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RequirementStatus {
    Draft,          // Being written
    UnderReview,    // Under review
    Approved,       // Approved for implementation
    Implemented,    // Implementation complete
    Verified,       // Verification complete
    Validated,      // Validation complete
    Obsolete,       // No longer applicable
}

impl RequirementStatus {
    pub const fn as_str(&self) -> &str {
        match self {
            RequirementStatus::Draft => "draft",
            RequirementStatus::UnderReview => "under_review",
            RequirementStatus::Approved => "approved",
            RequirementStatus::Implemented => "implemented",
            RequirementStatus::Verified => "verified",
            RequirementStatus::Validated => "validated",
            RequirementStatus::Obsolete => "obsolete",
        }
    }
    
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "draft" => RequirementStatus::Draft,
            "under_review" => RequirementStatus::UnderReview,
            "approved" => RequirementStatus::Approved,
            "implemented" => RequirementStatus::Implemented,
            "verified" => RequirementStatus::Verified,
            "validated" => RequirementStatus::Validated,
            "obsolete" => RequirementStatus::Obsolete,
            _ => RequirementStatus::Draft,
        }
    }
}

/// Complete requirement entity per PRD specification
#[derive(Debug, Clone)]
pub struct Requirement {
    pub id: String,                    // UUID v4
    pub project_id: String,            // Foreign key to Project
    pub req_id: String,                // User-facing ID (REQ-001, REQ-002, etc.)
    pub title: String,                 // Requirement title
    pub description: String,           // Detailed requirement description
    pub category: RequirementCategory, // Functional, Performance, etc.
    pub priority: RequirementPriority, // Critical, High, Medium, Low
    pub source: String,                // Source of requirement
    pub rationale: String,             // Reason for requirement
    pub acceptance_criteria: String,   // How to verify requirement
    pub verification_method: VerificationMethod, // Test, Analysis, etc.
    pub status: RequirementStatus,     // Draft, Approved, Implemented, etc.
    pub created_at: String,            // ISO 8601 timestamp
    pub updated_at: String,            // ISO 8601 timestamp
    pub created_by: String,            // User ID
    pub assigned_to: Option<String>,   // User responsible for implementation
    pub tags: Vec<String>,             // Searchable tags
    pub linked_requirements: Vec<String>, // Parent/child relationships
    pub linked_tests: Vec<String>,     // Test case IDs
    pub linked_risks: Vec<String>,     // Risk IDs
    pub regulatory_mapping: Vec<RegulatoryReference>, // Regulatory compliance
}

impl Default for Requirement {
    fn default() -> Self {
        Self {
            id: String::new(),
            project_id: String::new(),
            req_id: String::new(),
            title: String::new(),
            description: String::new(),
            category: RequirementCategory::Functional,
            priority: RequirementPriority::Medium,
            source: String::new(),
            rationale: String::new(),
            acceptance_criteria: String::new(),
            verification_method: VerificationMethod::Test,
            status: RequirementStatus::Draft,
            created_at: String::new(),
            updated_at: String::new(),
            created_by: String::new(),
            assigned_to: None,
            tags: Vec::new(),
            linked_requirements: Vec::new(),
            linked_tests: Vec::new(),
            linked_risks: Vec::new(),
            regulatory_mapping: Vec::new(),
        }
    }
}

/// Structure for updating requirements
#[derive(Debug)]
pub struct RequirementUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub category: Option<RequirementCategory>,
    pub priority: Option<RequirementPriority>,
    pub status: Option<RequirementStatus>,
    pub source: Option<String>,
    pub rationale: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub verification_method: Option<VerificationMethod>,
}

impl Default for RequirementUpdate {
    fn default() -> Self {
        Self::new()
    }
}

impl RequirementUpdate {
    pub const fn new() -> Self {
        RequirementUpdate {
            title: None,
            description: None,
            category: None,
            priority: None,
            status: None,
            source: None,
            rationale: None,
            acceptance_criteria: None,
            verification_method: None,
        }
    }
    
    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }
    
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }
    
    pub fn category(mut self, category: RequirementCategory) -> Self {
        self.category = Some(category);
        self
    }
    
    pub const fn priority(mut self, priority: RequirementPriority) -> Self {
        self.priority = Some(priority);
        self
    }
    
    pub const fn status(mut self, status: RequirementStatus) -> Self {
        self.status = Some(status);
        self
    }
    
    pub fn source<S: Into<String>>(mut self, source: S) -> Self {
        self.source = Some(source.into());
        self
    }
    
    pub fn rationale<S: Into<String>>(mut self, rationale: S) -> Self {
        self.rationale = Some(rationale.into());
        self
    }
    
    pub fn acceptance_criteria<S: Into<String>>(mut self, acceptance_criteria: S) -> Self {
        self.acceptance_criteria = Some(acceptance_criteria.into());
        self
    }
    
    pub const fn verification_method(mut self, verification_method: VerificationMethod) -> Self {
        self.verification_method = Some(verification_method);
        self
    }
}

impl Requirement {
    /// Create a new requirement
    pub fn new(
        project_id: String,
        req_id: String,
        title: String,
        description: String,
        category: RequirementCategory,
        created_by: String,
    ) -> Self {
        let now = current_timestamp().to_string();
        Self {
            id: generate_uuid(),
            project_id,
            req_id,
            title,
            description,
            category,
            priority: RequirementPriority::Medium,
            source: String::new(),
            rationale: String::new(),
            acceptance_criteria: String::new(),
            verification_method: VerificationMethod::Test,
            status: RequirementStatus::Draft,
            created_at: now.clone(),
            updated_at: now,
            created_by,
            assigned_to: None,
            tags: Vec::new(),
            linked_requirements: Vec::new(),
            linked_tests: Vec::new(),
            linked_risks: Vec::new(),
            regulatory_mapping: Vec::new(),
        }
    }
    
    /// Validate requirement data
    pub fn validate(&self) -> QmsResult<()> {
        if self.req_id.is_empty() {
            return Err(QmsError::validation_error("Requirement ID cannot be empty"));
        }
        
        if self.title.is_empty() {
            return Err(QmsError::validation_error("Requirement title cannot be empty"));
        }
        
        if self.description.is_empty() {
            return Err(QmsError::validation_error("Requirement description cannot be empty"));
        }
        
        if self.title.len() > 200 {
            return Err(QmsError::validation_error("Requirement title must be 200 characters or less"));
        }
        
        if self.description.len() > 1000 {
            return Err(QmsError::validation_error("Requirement description must be 1000 characters or less"));
        }
        
        // Validate req_id format (REQ-YYYYMMDD-NNN)
        if !self.req_id.starts_with("REQ-") {
            return Err(QmsError::validation_error("Requirement ID must start with 'REQ-'"));
        }
        
        Ok(())
    }
    
    /// Convert to JSON string
    pub fn to_json(&self) -> String {
        let mut json = String::new();
        json.push_str("{\n");
        json.push_str("  \"version\": \"1.0\",\n");
        json.push_str(&format!("  \"id\": \"{}\",\n", self.id));
        json.push_str(&format!("  \"project_id\": \"{}\",\n", self.project_id));
        json.push_str(&format!("  \"req_id\": \"{}\",\n", self.req_id));
        json.push_str(&format!("  \"title\": \"{}\",\n", escape_json_string(&self.title)));
        json.push_str(&format!("  \"description\": \"{}\",\n", escape_json_string(&self.description)));
        json.push_str(&format!("  \"category\": \"{}\",\n", self.category.as_str()));
        json.push_str(&format!("  \"priority\": \"{}\",\n", self.priority.as_str()));
        json.push_str(&format!("  \"source\": \"{}\",\n", escape_json_string(&self.source)));
        json.push_str(&format!("  \"rationale\": \"{}\",\n", escape_json_string(&self.rationale)));
        json.push_str(&format!("  \"acceptance_criteria\": \"{}\",\n", escape_json_string(&self.acceptance_criteria)));
        json.push_str(&format!("  \"verification_method\": \"{}\",\n", self.verification_method.as_str()));
        json.push_str(&format!("  \"status\": \"{}\",\n", self.status.as_str()));
        json.push_str(&format!("  \"created_at\": \"{}\",\n", self.created_at));
        json.push_str(&format!("  \"updated_at\": \"{}\",\n", self.updated_at));
        json.push_str(&format!("  \"created_by\": \"{}\",\n", self.created_by));
        
        if let Some(assigned_to) = &self.assigned_to {
            json.push_str(&format!("  \"assigned_to\": \"{assigned_to}\",\n"));
        } else {
            json.push_str("  \"assigned_to\": null,\n");
        }
        
        // Tags array
        json.push_str("  \"tags\": [");
        for (i, tag) in self.tags.iter().enumerate() {
            if i > 0 { json.push_str(", "); }
            json.push_str(&format!("\"{}\"", escape_json_string(tag)));
        }
        json.push_str("],\n");
        
        // Linked requirements array
        json.push_str("  \"linked_requirements\": [");
        for (i, req_id) in self.linked_requirements.iter().enumerate() {
            if i > 0 { json.push_str(", "); }
            json.push_str(&format!("\"{req_id}\""));
        }
        json.push_str("],\n");
        
        // Linked tests array
        json.push_str("  \"linked_tests\": [");
        for (i, test_id) in self.linked_tests.iter().enumerate() {
            if i > 0 { json.push_str(", "); }
            json.push_str(&format!("\"{test_id}\""));
        }
        json.push_str("],\n");
        
        // Linked risks array
        json.push_str("  \"linked_risks\": [");
        for (i, risk_id) in self.linked_risks.iter().enumerate() {
            if i > 0 { json.push_str(", "); }
            json.push_str(&format!("\"{risk_id}\""));
        }
        json.push_str("]\n");
        
        json.push('}');
        json
    }
}

/// Requirement manager for CRUD operations
pub struct RequirementManager {
    project_path: PathBuf,
    trace_dir: PathBuf,
    requirements_file: PathBuf,
    requirements: HashMap<String, Requirement>,
}

impl RequirementManager {
    /// Create new requirement manager
    pub fn new(project_path: &Path) -> QmsResult<Self> {
        let trace_dir = project_path.join("trace");
        let requirements_file = trace_dir.join("requirements.json");
        
        let mut manager = Self {
            project_path: project_path.to_path_buf(),
            trace_dir,
            requirements_file,
            requirements: HashMap::new(),
        };
        
        manager.initialize()?;
        Ok(manager)
    }
    
    /// Initialize requirement management system
    pub fn initialize(&mut self) -> QmsResult<()> {
        // Create trace directory structure
        fs::create_dir_all(&self.trace_dir)?;
        
        // Create subdirectories
        fs::create_dir_all(self.trace_dir.join("reports"))?;
        
        // Create empty index files if they don't exist
        let requirements_file = self.trace_dir.join("requirements.json");
        if !requirements_file.exists() {
            let empty_index = r#"{
  "version": "1.0",
  "data": []
}"#;
            fs::write(&requirements_file, empty_index)?;
        }
        
        let testcases_file = self.trace_dir.join("testcases.json");
        if !testcases_file.exists() {
            let empty_index = r#"{
  "version": "1.0",
  "data": []
}"#;
            fs::write(&testcases_file, empty_index)?;
        }
        
        // Note: links.json is managed by TraceabilityManager, not RequirementManager
        
        let coverage_file = self.trace_dir.join("coverage.json");
        if !coverage_file.exists() {
            let empty_index = r#"{
  "version": "1.0",
  "data": {
    "requirements_coverage": 0.0,
    "test_coverage": 0.0,
    "verification_coverage": 0.0
  }
}"#;
            fs::write(&coverage_file, empty_index)?;
        }
        
        // Load existing requirements
        self.load_requirements()?;
        
        Ok(())
    }
    
    /// Load requirements from storage
    fn load_requirements(&mut self) -> QmsResult<()> {
        let requirements_file = self.project_path.join("trace/requirements.json");
        
        if !requirements_file.exists() {
            return Ok(());
        }
        
        let content = fs::read_to_string(&requirements_file)?;
        if content.trim().is_empty() {
            return Ok(());
        }
        
        // Basic JSON parsing - extract requirements array
        if let Some(data_start) = content.find("\"data\": [") {
            let data_section = &content[data_start + 9..];
            if let Some(data_end) = data_section.find("]") {
                let data_content = &data_section[..data_end];
                if !data_content.trim().is_empty() {
                    // Parse individual requirement objects
                    let mut current_obj = String::new();
                    let mut brace_count = 0;
                    let mut in_string = false;
                    let mut escaped = false;
                    
                    for ch in data_content.chars() {
                        current_obj.push(ch);
                        
                        if !escaped && ch == '\\' {
                            escaped = true;
                            continue;
                        }
                        
                        if !escaped && ch == '"' {
                            in_string = !in_string;
                        }
                        
                        if !in_string {
                            if ch == '{' {
                                brace_count += 1;
                            } else if ch == '}' {
                                brace_count -= 1;
                                if brace_count == 0 {
                                    // Parse complete requirement object
                                    if let Ok(req) = self.parse_requirement_json(&current_obj) {
                                        self.requirements.insert(req.id.clone(), req);
                                    }
                                    current_obj.clear();
                                }
                            }
                        }
                        
                        escaped = false;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Parse requirement from JSON string
    fn parse_requirement_json(&self, json: &str) -> QmsResult<Requirement> {
        let mut id = String::new();
        let mut project_id = String::new();
        let mut req_id = String::new();
        let mut title = String::new();
        let mut description = String::new();
        let mut category = RequirementCategory::Functional;
        let mut priority = RequirementPriority::Medium;
        let mut source = String::new();
        let mut rationale = String::new();
        let mut acceptance_criteria = String::new();
        let mut verification_method = VerificationMethod::Test;
        let mut status = RequirementStatus::Draft;
        let mut created_at = String::new();
        let mut updated_at = String::new();
        let mut created_by = String::new();
        let mut assigned_to = None;
        let tags = Vec::new();
        let linked_requirements = Vec::new();
        let linked_tests = Vec::new();
        let linked_risks = Vec::new();
        let regulatory_mapping = Vec::new(); // TODO: Implement parsing
        
        // Extract fields from JSON
        if let Some(field_value) = extract_json_field(json, "id") {
            id = field_value;
        }
        if let Some(field_value) = extract_json_field(json, "project_id") {
            project_id = field_value;
        }
        if let Some(field_value) = extract_json_field(json, "req_id") {
            req_id = field_value;
        }
        if let Some(field_value) = extract_json_field(json, "title") {
            title = field_value;
        }
        if let Some(field_value) = extract_json_field(json, "description") {
            description = field_value;
        }
        if let Some(field_value) = extract_json_field(json, "category") {
            category = RequirementCategory::from_str(&field_value);
        }
        if let Some(field_value) = extract_json_field(json, "priority") {
            priority = RequirementPriority::from_str(&field_value);
        }
        if let Some(field_value) = extract_json_field(json, "source") {
            source = field_value;
        }
        if let Some(field_value) = extract_json_field(json, "rationale") {
            rationale = field_value;
        }
        if let Some(field_value) = extract_json_field(json, "acceptance_criteria") {
            acceptance_criteria = field_value;
        }
        if let Some(field_value) = extract_json_field(json, "verification_method") {
            verification_method = VerificationMethod::from_str(&field_value);
        }
        if let Some(field_value) = extract_json_field(json, "status") {
            status = RequirementStatus::from_str(&field_value);
        }
        if let Some(field_value) = extract_json_field(json, "created_at") {
            created_at = field_value;
        }
        if let Some(field_value) = extract_json_field(json, "updated_at") {
            updated_at = field_value;
        }
        if let Some(field_value) = extract_json_field(json, "created_by") {
            created_by = field_value;
        }
        if let Some(field_value) = extract_json_field(json, "assigned_to") {
            if field_value != "null" {
                assigned_to = Some(field_value);
            }
        }
        
        // TODO: Parse arrays (tags, linked_requirements, etc.)
        
        Ok(Requirement {
            id,
            project_id,
            req_id,
            title,
            description,
            category,
            priority,
            source,
            rationale,
            acceptance_criteria,
            verification_method,
            status,
            created_at,
            updated_at,
            created_by,
            assigned_to,
            tags,
            linked_requirements,
            linked_tests,
            linked_risks,
            regulatory_mapping,
        })
    }
    
    /// Create a new requirement
    pub fn create_requirement(
        &mut self,
        project_id: String,
        req_id: String,
        title: String,
        description: String,
        category: RequirementCategory,
        created_by: String,
    ) -> QmsResult<String> {
        let requirement = Requirement::new(
            project_id,
            req_id,
            title,
            description,
            category,
            created_by,
        );
        
        requirement.validate()?;
        
        let id = requirement.id.clone();
        self.requirements.insert(id.clone(), requirement.clone());
        
        // Save to storage
        self.save_requirements()?;
        
        // Log audit entry
        let _ = audit_log_create("Requirement", &id, &requirement.to_json());
        
        Ok(id)
    }
    
    /// Save requirements to storage
    fn save_requirements(&self) -> QmsResult<()> {
        let requirements_file = self.project_path.join("trace/requirements.json");
        
        let mut json = String::new();
        json.push_str("{\n");
        json.push_str("  \"version\": \"1.0\",\n");
        json.push_str("  \"data\": [\n");
        
        let mut first = true;
        for requirement in self.requirements.values() {
            if !first {
                json.push_str(",\n");
            }
            first = false;
            
            // Indent requirement JSON
            let req_json = requirement.to_json();
            for line in req_json.lines() {
                json.push_str("    ");
                json.push_str(line);
                json.push('\n');
            }
        }
        
        json.push_str("  ]\n");
        json.push_str("}\n");
        
        fs::write(&requirements_file, json)?;
        Ok(())
    }
    
    /// Get requirement by req_id
    pub fn get_requirement_by_req_id(&self, req_id: &str) -> Option<&Requirement> {
        self.requirements.values().find(|r| r.req_id == req_id)
    }
    
    /// List all requirements
    pub fn list_requirements(&self) -> Vec<&Requirement> {
        self.requirements.values().collect()
    }
    
    /// Generate next requirement ID
    pub fn generate_next_req_id(&self) -> String {
        let mut next_num = 1;
        
        // Find the highest existing number
        for req in self.requirements.values() {
            if req.req_id.starts_with("REQ-") {
                if let Some(num_str) = req.req_id.strip_prefix("REQ-") {
                    if let Ok(num) = num_str.parse::<u32>() {
                        if num >= next_num {
                            next_num = num + 1;
                        }
                    }
                }
            }
        }
        
        format!("REQ-{next_num:03}")
    }
    
    /// Save requirements to file
    pub fn save(&self) -> QmsResult<()> {
        let requirements_vec: Vec<&Requirement> = self.requirements.values().collect();
        
        // Manual JSON serialization
        let mut json_content = String::from("[\n");
        
        for (i, requirement) in requirements_vec.iter().enumerate() {
            if i > 0 {
                json_content.push_str(",\n");
            }
            json_content.push_str(&requirement.to_json());
        }
        
        json_content.push_str("\n]");
        
        fs::write(&self.requirements_file, json_content)
            .map_err(|e| QmsError::io_error(&format!("Failed to write requirements file: {e}")))?;
        
        Ok(())
    }
    
    /// Update an existing requirement
    pub fn update_requirement(&mut self, req_id: &str, updates: RequirementUpdate) -> Result<(), QmsError> {
        // Collect audit info before mutable borrow
        let (req_id_copy, requirement_id_copy) = {
            let requirement = self.requirements.values_mut()
                .find(|r| r.req_id == req_id)
                .ok_or_else(|| QmsError::validation_error(&format!("Requirement {req_id} not found")))?;

            // Update fields if provided
            if let Some(title) = updates.title {
                requirement.title = title;
            }
            if let Some(description) = updates.description {
                requirement.description = description;
            }
            if let Some(category) = updates.category {
                requirement.category = category;
            }
            if let Some(priority) = updates.priority {
                requirement.priority = priority;
            }
            if let Some(status) = updates.status {
                requirement.status = status;
            }
            if let Some(source) = updates.source {
                requirement.source = source;
            }
            if let Some(rationale) = updates.rationale {
                requirement.rationale = rationale;
            }
            if let Some(acceptance_criteria) = updates.acceptance_criteria {
                requirement.acceptance_criteria = acceptance_criteria;
            }
            if let Some(verification_method) = updates.verification_method {
                requirement.verification_method = verification_method;
            }

            // Update timestamp
            requirement.updated_at = current_timestamp().to_string();

            // Return copies for audit logging
            (requirement.req_id.clone(), requirement.id.clone())
        };

        // Save changes
        self.save()?;

        // Add audit logging for requirement update
        if let Err(e) = crate::modules::audit_logger::audit_log_action(
            "REQUIREMENT_UPDATED",
            "Requirement",
            &format!("Requirement {} ({}) updated", req_id_copy, requirement_id_copy)
        ) {
            eprintln!("Warning: Failed to log requirement update: {}", e);
        }

        Ok(())
    }
    
    /// Delete a requirement
    pub fn delete_requirement(&mut self, req_id: &str) -> Result<(), QmsError> {
        // Find the requirement to delete
        let requirement_id = self.requirements.values()
            .find(|r| r.req_id == req_id)
            .map(|r| r.id.clone())
            .ok_or_else(|| QmsError::validation_error(&format!("Requirement {req_id} not found")))?;
        
        // Remove from collection
        let removed = self.requirements.remove(&requirement_id);
        
        if removed.is_some() {
            // Save changes
            self.save()?;
            
            // Add audit logging for requirement deletion
            if let Err(e) = crate::modules::audit_logger::audit_log_action(
                "REQUIREMENT_DELETED",
                "Requirement",
                &format!("Requirement {} deleted", requirement_id)
            ) {
                eprintln!("Warning: Failed to log requirement deletion: {}", e);
            }
        }
        
        Ok(())
    }
}

/// Helper function to extract JSON field value
fn extract_json_field(json: &str, field_name: &str) -> Option<String> {
    let pattern = format!("\"{field_name}\": \"");
    if let Some(start) = json.find(&pattern) {
        let value_start = start + pattern.len();
        let remaining = &json[value_start..];
        
        if let Some(end) = remaining.find('"') {
            let value = &remaining[..end];
            return Some(value.to_string());
        }
    }
    None
}

/// Helper function to escape JSON strings
fn escape_json_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_requirement_creation() {
        let req = Requirement::new(
            "proj-001".to_string(),
            "REQ-001".to_string(),
            "User Authentication".to_string(),
            "System shall authenticate users".to_string(),
            RequirementCategory::Security,
            "test_user".to_string(),
        );
        
        assert_eq!(req.req_id, "REQ-001");
        assert_eq!(req.title, "User Authentication");
        assert_eq!(req.category, RequirementCategory::Security);
        assert_eq!(req.status, RequirementStatus::Draft);
    }
    
    #[test]
    fn test_requirement_validation() {
        let mut req = Requirement::new(
            "proj-001".to_string(),
            "REQ-001".to_string(),
            "Test".to_string(),
            "Test description".to_string(),
            RequirementCategory::Functional,
            "test_user".to_string(),
        );
        
        assert!(req.validate().is_ok());
        
        // Test empty req_id
        req.req_id = String::new();
        assert!(req.validate().is_err());
        
        // Test invalid req_id format
        req.req_id = "INVALID".to_string();
        assert!(req.validate().is_err());
        
        // Test valid req_id
        req.req_id = "REQ-001".to_string();
        assert!(req.validate().is_ok());
    }
    
    #[test]
    fn test_requirement_manager_initialization() {
        let test_dir = std::env::temp_dir().join("qms_test_req_init");
        let _ = fs::remove_dir_all(&test_dir);
        
        let manager = RequirementManager::new(&test_dir);
        assert!(manager.is_ok());
        
        // Check directory structure
        assert!(test_dir.join("trace").exists());
        assert!(test_dir.join("trace/requirements.json").exists());
        assert!(test_dir.join("trace/testcases.json").exists());
        // Note: links.json is managed by TraceabilityManager, not RequirementManager
        assert!(test_dir.join("trace/coverage.json").exists());
        assert!(test_dir.join("trace/reports").exists());
        
        let _ = fs::remove_dir_all(&test_dir);
    }
    
    #[test]
    fn test_requirement_json_serialization() {
        let req = Requirement::new(
            "proj-001".to_string(),
            "REQ-001".to_string(),
            "Test Requirement".to_string(),
            "Test description".to_string(),
            RequirementCategory::Functional,
            "test_user".to_string(),
        );
        
        let json = req.to_json();
        assert!(json.contains("\"req_id\": \"REQ-001\""));
        assert!(json.contains("\"title\": \"Test Requirement\""));
        assert!(json.contains("\"category\": \"functional\""));
        assert!(json.contains("\"status\": \"draft\""));
    }
    
    #[test]
    fn test_requirement_manager_create() {
        let test_dir = std::env::temp_dir().join("qms_test_req_create");
        let _ = fs::remove_dir_all(&test_dir);
        
        let mut manager = RequirementManager::new(&test_dir).unwrap();
        
        let req_id = manager.create_requirement(
            "proj-001".to_string(),
            "REQ-001".to_string(),
            "Test Requirement".to_string(),
            "Test description".to_string(),
            RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();
        
        assert!(!req_id.is_empty());
        
        let requirement = manager.get_requirement_by_req_id("REQ-001");
        assert!(requirement.is_some());
        assert_eq!(requirement.unwrap().title, "Test Requirement");
        
        let _ = fs::remove_dir_all(&test_dir);
    }
    
    #[test]
    fn test_next_req_id_generation() {
        let test_dir = std::env::temp_dir().join("qms_test_req_id_gen");
        let _ = fs::remove_dir_all(&test_dir);
        
        let mut manager = RequirementManager::new(&test_dir).unwrap();
        
        // First requirement should be REQ-001
        let next_id = manager.generate_next_req_id();
        assert_eq!(next_id, "REQ-001");
        
        // Create a requirement
        let _ = manager.create_requirement(
            "proj-001".to_string(),
            "REQ-001".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();
        
        // Next should be REQ-002
        let next_id = manager.generate_next_req_id();
        assert_eq!(next_id, "REQ-002");
        
        let _ = fs::remove_dir_all(&test_dir);
    }
}
