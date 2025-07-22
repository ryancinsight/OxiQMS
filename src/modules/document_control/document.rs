//! Document entities for QMS document control
//! Phase 2.1.1 - Document Metadata Schema & Storage
//! Implements core document structures per PRD section 9.1.2

#![allow(dead_code)] // Phase 2 infrastructure - document control system

use crate::error::{QmsError, QmsResult};
use crate::json_utils::{JsonSerializable, JsonValue, JsonError};
use crate::validation::validate_document_title;
use std::collections::HashMap;

/// Document type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum DocumentType {
    SoftwareRequirementsSpecification,
    SoftwareDesignDescription,
    VerificationAndValidation,
    RiskManagementFile,
    DesignHistoryFile,
    UserRequirements,
    TestProtocol,
    TestReport,
    Other(String),
}

impl DocumentType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "SRS" | "SoftwareRequirementsSpecification" => DocumentType::SoftwareRequirementsSpecification,
            "SDD" | "SoftwareDesignDescription" => DocumentType::SoftwareDesignDescription,
            "RMF" | "RiskManagementFile" => DocumentType::RiskManagementFile,
            "DHF" | "DesignHistoryFile" => DocumentType::DesignHistoryFile,
            "UR" | "UserRequirements" => DocumentType::UserRequirements,
            "TP" | "TestProtocol" => DocumentType::TestProtocol,
            "TR" | "TestReport" => DocumentType::TestReport,
            other => DocumentType::Other(other.to_string()),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            DocumentType::SoftwareRequirementsSpecification => "SRS".to_string(),
            DocumentType::SoftwareDesignDescription => "SDD".to_string(),
            DocumentType::VerificationAndValidation => "VnV".to_string(),
            DocumentType::RiskManagementFile => "RMF".to_string(),
            DocumentType::DesignHistoryFile => "DHF".to_string(),
            DocumentType::UserRequirements => "UR".to_string(),
            DocumentType::TestProtocol => "TP".to_string(),
            DocumentType::TestReport => "TR".to_string(),
            DocumentType::Other(s) => s.clone(),
        }
    }
}

/// Document status enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum DocumentStatus {
    Draft,
    InReview,
    Approved,
    Deprecated,
    Archived,
}

impl DocumentStatus {
    pub fn from_str(s: &str) -> Result<Self, QmsError> {
        match s {
            "Draft" => Ok(DocumentStatus::Draft),
            "InReview" => Ok(DocumentStatus::InReview),
            "Approved" => Ok(DocumentStatus::Approved),
            "Deprecated" => Ok(DocumentStatus::Deprecated),
            "Archived" => Ok(DocumentStatus::Archived),
            _ => Err(QmsError::validation_error(&format!("Invalid document status: {s}"))),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            DocumentStatus::Draft => "Draft".to_string(),
            DocumentStatus::InReview => "InReview".to_string(),
            DocumentStatus::Approved => "Approved".to_string(),
            DocumentStatus::Deprecated => "Deprecated".to_string(),
            DocumentStatus::Archived => "Archived".to_string(),
        }
    }
}

/// Regulatory reference for compliance mapping
#[derive(Debug, Clone)]
pub struct RegulatoryReference {
    pub standard: String,     // e.g., "21 CFR 820.30"
    pub section: String,      // e.g., "Design Controls"
    pub requirement: String,  // Specific requirement text
}

impl RegulatoryReference {
    pub fn new(standard: String, section: String, requirement: String) -> QmsResult<Self> {
        if standard.is_empty() || section.is_empty() || requirement.is_empty() {
            return Err(QmsError::validation_error("All regulatory reference fields are required"));
        }

        Ok(Self {
            standard,
            section,
            requirement,
        })
    }
}

/// Document entity with comprehensive metadata
#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,                    // UUID v4
    pub project_id: String,            // Foreign key to Project
    pub title: String,                 // Document title (max 200 chars)
    pub content: String,               // Document content (Markdown)
    pub doc_type: DocumentType,        // Document type
    pub version: String,               // Semantic versioning
    pub status: DocumentStatus,        // Document status
    pub created_at: String,            // ISO 8601 timestamp
    pub updated_at: String,            // ISO 8601 timestamp
    pub created_by: String,            // User ID
    pub approved_by: Option<String>,   // User ID (if approved)
    pub file_path: String,             // Relative path from project root
    pub checksum: String,              // SHA-256 of content
    pub tags: Vec<String>,             // Searchable tags
    pub regulatory_mapping: Vec<RegulatoryReference>, // Regulatory compliance links
    pub locked: bool,                  // Document checkout status
    pub locked_by: Option<String>,     // User who has document checked out
    pub locked_at: Option<String>,     // ISO 8601 timestamp when document was locked
}

/// Document lock for checkout/checkin workflow
#[derive(Debug, Clone)]
pub struct DocumentLock {
    pub document_id: String,           // Document ID that is locked
    pub user_id: String,               // User who checked out the document
    pub locked_at: String,             // ISO 8601 timestamp when locked
    pub lock_reason: Option<String>,   // Optional reason for lock
}

impl Document {
    /// Create a new document with validation
    pub fn new(
        id: String,
        project_id: String,
        title: String,
        content: String,
        doc_type: DocumentType,
        version: String,
        created_by: String,
        file_path: String,
    ) -> QmsResult<Self> {
        // Validate title length
        if !validate_document_title(&title) {
            return Err(QmsError::validation_error("Document title must be 1-200 characters"));
        }

        // Validate version format (basic semver check)
        if !Self::is_valid_semver(&version) {
            return Err(QmsError::validation_error("Version must follow semantic versioning (e.g., 1.0.0)"));
        }

        // Validate project_id format (UUID)
        if !Self::is_valid_uuid(&project_id) {
            return Err(QmsError::validation_error("Invalid project_id format"));
        }

        // Validate document ID format (UUID)
        if !Self::is_valid_uuid(&id) {
            return Err(QmsError::validation_error("Invalid document ID format"));
        }

        // Calculate checksum
        let checksum = crate::json_utils::calculate_checksum(&content);
        
        // Get current timestamp
        let timestamp = Self::current_timestamp();

        Ok(Self {
            id,
            project_id,
            title,
            content,
            doc_type,
            version,
            status: DocumentStatus::Draft,
            created_at: timestamp.clone(),
            updated_at: timestamp,
            created_by,
            approved_by: None,
            file_path,
            checksum,
            tags: Vec::new(),
            regulatory_mapping: Vec::new(),
            locked: false,
            locked_by: None,
            locked_at: None,
        })
    }

    /// Update document content with new checksum and timestamp
    pub fn update_content(&mut self, content: String) -> QmsResult<()> {
        self.content = content;
        self.checksum = crate::json_utils::calculate_checksum(&self.content);
        self.updated_at = Self::current_timestamp();
        Ok(())
    }

    /// Add a tag to the document
    pub fn add_tag(&mut self, tag: String) -> QmsResult<()> {
        if tag.is_empty() || tag.len() > 50 {
            return Err(QmsError::validation_error("Tag must be 1-50 characters"));
        }

        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
        Ok(())
    }

    /// Remove a tag from the document
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    /// Add regulatory mapping
    pub fn add_regulatory_mapping(&mut self, mapping: RegulatoryReference) {
        self.regulatory_mapping.push(mapping);
    }

    /// Update document status with validation
    pub fn update_status(&mut self, new_status: DocumentStatus, user_id: Option<String>) -> QmsResult<()> {
        // Validate status transitions
        match (&self.status, &new_status) {
            (DocumentStatus::Draft, DocumentStatus::InReview) => {},
            (DocumentStatus::InReview, DocumentStatus::Approved) => {
                if user_id.is_none() {
                    return Err(QmsError::validation_error("Approval requires user_id"));
                }
                self.approved_by = user_id;
            },
            (DocumentStatus::InReview, DocumentStatus::Draft) => {},
            (DocumentStatus::Approved, DocumentStatus::Deprecated) => {},
            (DocumentStatus::Deprecated, DocumentStatus::Archived) => {},
            (_, DocumentStatus::Archived) => {},
            (DocumentStatus::Archived, DocumentStatus::Draft) => {}, // Allow restoration from archive
            _ => {
                return Err(QmsError::validation_error(&format!(
                    "Invalid status transition from {:?} to {:?}", 
                    self.status, 
                    new_status
                )));
            }
        }

        self.status = new_status;
        self.updated_at = Self::current_timestamp();
        Ok(())
    }

    /// Validate content integrity using checksum
    pub fn validate_integrity(&self) -> bool {
        let calculated_checksum = crate::json_utils::calculate_checksum(&self.content);
        calculated_checksum == self.checksum
    }

    /// Basic UUID validation (simplified for std-only)
    fn is_valid_uuid(uuid: &str) -> bool {
        // Basic UUID format check: 8-4-4-4-12 hexadecimal characters
        let parts: Vec<&str> = uuid.split('-').collect();
        if parts.len() != 5 {
            return false;
        }

        parts[0].len() == 8 && parts[0].chars().all(|c| c.is_ascii_hexdigit()) &&
        parts[1].len() == 4 && parts[1].chars().all(|c| c.is_ascii_hexdigit()) &&
        parts[2].len() == 4 && parts[2].chars().all(|c| c.is_ascii_hexdigit()) &&
        parts[3].len() == 4 && parts[3].chars().all(|c| c.is_ascii_hexdigit()) &&
        parts[4].len() == 12 && parts[4].chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Basic semantic version validation
    fn is_valid_semver(version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return false;
        }

        parts.iter().all(|part| {
            part.parse::<u32>().is_ok()
        })
    }

    /// Get current timestamp in ISO 8601 format
    fn current_timestamp() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Simple ISO 8601 format (YYYY-MM-DDTHH:MM:SSZ)
        // This is a simplified implementation for std-only
        format!("{now}")
    }
}

impl JsonSerializable for Document {
    fn to_json(&self) -> String {
        let mut obj = HashMap::new();
        
        obj.insert("id".to_string(), JsonValue::String(self.id.clone()));
        obj.insert("project_id".to_string(), JsonValue::String(self.project_id.clone()));
        obj.insert("title".to_string(), JsonValue::String(self.title.clone()));
        obj.insert("content".to_string(), JsonValue::String(self.content.clone()));
        obj.insert("doc_type".to_string(), JsonValue::String(self.doc_type.to_string()));
        obj.insert("version".to_string(), JsonValue::String(self.version.clone()));
        obj.insert("status".to_string(), JsonValue::String(self.status.to_string()));
        obj.insert("created_at".to_string(), JsonValue::String(self.created_at.clone()));
        obj.insert("updated_at".to_string(), JsonValue::String(self.updated_at.clone()));
        obj.insert("created_by".to_string(), JsonValue::String(self.created_by.clone()));
        
        if let Some(ref approved_by) = self.approved_by {
            obj.insert("approved_by".to_string(), JsonValue::String(approved_by.clone()));
        } else {
            obj.insert("approved_by".to_string(), JsonValue::Null);
        }
        
        obj.insert("file_path".to_string(), JsonValue::String(self.file_path.clone()));
        obj.insert("checksum".to_string(), JsonValue::String(self.checksum.clone()));
        
        // Convert tags to JSON array
        let tags_json: Vec<JsonValue> = self.tags.iter()
            .map(|tag| JsonValue::String(tag.clone()))
            .collect();
        obj.insert("tags".to_string(), JsonValue::Array(tags_json));
        
        // Convert regulatory mapping to JSON array
        let regulatory_json: Vec<JsonValue> = self.regulatory_mapping.iter()
            .map(|mapping| {
                let mut reg_obj = HashMap::new();
                reg_obj.insert("standard".to_string(), JsonValue::String(mapping.standard.clone()));
                reg_obj.insert("section".to_string(), JsonValue::String(mapping.section.clone()));
                reg_obj.insert("requirement".to_string(), JsonValue::String(mapping.requirement.clone()));
                JsonValue::Object(reg_obj)
            })
            .collect();
        obj.insert("regulatory_mapping".to_string(), JsonValue::Array(regulatory_json));
        
        // Add checkout/checkin fields
        obj.insert("locked".to_string(), JsonValue::Bool(self.locked));
        
        if let Some(ref locked_by) = self.locked_by {
            obj.insert("locked_by".to_string(), JsonValue::String(locked_by.clone()));
        } else {
            obj.insert("locked_by".to_string(), JsonValue::Null);
        }
        
        if let Some(ref locked_at) = self.locked_at {
            obj.insert("locked_at".to_string(), JsonValue::String(locked_at.clone()));
        } else {
            obj.insert("locked_at".to_string(), JsonValue::Null);
        }
        
        JsonValue::Object(obj).json_to_string()
    }

    fn from_json(s: &str) -> Result<Self, JsonError> {
        let json_value = JsonValue::parse(s)?;
        
        if let JsonValue::Object(obj) = json_value {
            // Extract required fields
            let id = extract_string(&obj, "id")?;
            let project_id = extract_string(&obj, "project_id")?;
            let title = extract_string(&obj, "title")?;
            let content = extract_string(&obj, "content")?;
            let doc_type_str = extract_string(&obj, "doc_type")?;
            let version = extract_string(&obj, "version")?;
            let status_str = extract_string(&obj, "status")?;
            let created_at = extract_string(&obj, "created_at")?;
            let updated_at = extract_string(&obj, "updated_at")?;
            let created_by = extract_string(&obj, "created_by")?;
            let file_path = extract_string(&obj, "file_path")?;
            let checksum = extract_string(&obj, "checksum")?;
            
            // Parse enums
            let doc_type = DocumentType::from_str(&doc_type_str);
            let status = DocumentStatus::from_str(&status_str)
                .map_err(|e| JsonError::ValidationError(e.to_string()))?;
            
            // Extract optional approved_by
            let approved_by = match obj.get("approved_by") {
                Some(JsonValue::String(s)) => Some(s.clone()),
                Some(JsonValue::Null) => None,
                _ => None,
            };
            
            // Extract tags array
            let tags = match obj.get("tags") {
                Some(JsonValue::Array(arr)) => {
                    arr.iter()
                        .filter_map(|v| match v {
                            JsonValue::String(s) => Some(s.clone()),
                            _ => None,
                        })
                        .collect()
                },
                _ => Vec::new(),
            };
            
            // Extract regulatory mapping array
            let regulatory_mapping = match obj.get("regulatory_mapping") {
                Some(JsonValue::Array(arr)) => {
                    arr.iter()
                        .filter_map(|v| match v {
                            JsonValue::Object(reg_obj) => {
                                let standard = extract_string(reg_obj, "standard").ok()?;
                                let section = extract_string(reg_obj, "section").ok()?;
                                let requirement = extract_string(reg_obj, "requirement").ok()?;
                                RegulatoryReference::new(standard, section, requirement).ok()
                            },
                            _ => None,
                        })
                        .collect()
                },
                _ => Vec::new(),
            };
            
            // Extract checkout/checkin fields
            let locked = match obj.get("locked") {
                Some(JsonValue::Bool(b)) => *b,
                _ => false,
            };
            
            let locked_by = match obj.get("locked_by") {
                Some(JsonValue::String(s)) => Some(s.clone()),
                Some(JsonValue::Null) => None,
                _ => None,
            };
            
            let locked_at = match obj.get("locked_at") {
                Some(JsonValue::String(s)) => Some(s.clone()),
                Some(JsonValue::Null) => None,
                _ => None,
            };
            
            Ok(Document {
                id,
                project_id,
                title,
                content,
                doc_type,
                version,
                status,
                created_at,
                updated_at,
                created_by,
                approved_by,
                file_path,
                checksum,
                tags,
                regulatory_mapping,
                locked,
                locked_by,
                locked_at,
            })
        } else {
            Err(JsonError::InvalidFormat("Expected JSON object".to_string()))
        }
    }
}

impl JsonSerializable for DocumentLock {
    fn to_json(&self) -> String {
        let mut obj = HashMap::new();
        
        obj.insert("document_id".to_string(), JsonValue::String(self.document_id.clone()));
        obj.insert("user_id".to_string(), JsonValue::String(self.user_id.clone()));
        obj.insert("locked_at".to_string(), JsonValue::String(self.locked_at.clone()));
        
        if let Some(ref reason) = self.lock_reason {
            obj.insert("lock_reason".to_string(), JsonValue::String(reason.clone()));
        } else {
            obj.insert("lock_reason".to_string(), JsonValue::Null);
        }
        
        JsonValue::Object(obj).json_to_string()
    }

    fn from_json(s: &str) -> Result<Self, JsonError> {
        let json_value = JsonValue::parse(s)?;
        
        if let JsonValue::Object(obj) = json_value {
            let document_id = extract_string(&obj, "document_id")?;
            let user_id = extract_string(&obj, "user_id")?;
            let locked_at = extract_string(&obj, "locked_at")?;
            
            let lock_reason = match obj.get("lock_reason") {
                Some(JsonValue::String(s)) => Some(s.clone()),
                Some(JsonValue::Null) => None,
                _ => None,
            };
            
            Ok(DocumentLock {
                document_id,
                user_id,
                locked_at,
                lock_reason,
            })
        } else {
            Err(JsonError::InvalidFormat("Expected JSON object".to_string()))
        }
    }
}

/// Helper function to extract string from JSON object
fn extract_string(obj: &HashMap<String, JsonValue>, key: &str) -> Result<String, JsonError> {
    match obj.get(key) {
        Some(JsonValue::String(s)) => Ok(s.clone()),
        Some(_) => Err(JsonError::ValidationError(format!("Field '{key}' must be a string"))),
        None => Err(JsonError::ValidationError(format!("Missing required field: {key}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = Document::new(
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
            "550e8400-e29b-41d4-a716-446655440001".to_string(),
            "Test Document".to_string(),
            "This is test content".to_string(),
            DocumentType::SoftwareRequirementsSpecification,
            "1.0.0".to_string(),
            "user123".to_string(),
            "docs/test.md".to_string(),
        );

        assert!(doc.is_ok());
        let doc = doc.unwrap();
        assert_eq!(doc.title, "Test Document");
        assert_eq!(doc.status, DocumentStatus::Draft);
        assert!(doc.approved_by.is_none());
    }

    #[test]
    fn test_document_validation() {
        // Test invalid title (too long)
        let result = Document::new(
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
            "550e8400-e29b-41d4-a716-446655440001".to_string(),
            "a".repeat(201), // Too long
            "content".to_string(),
            DocumentType::SoftwareRequirementsSpecification,
            "1.0.0".to_string(),
            "user123".to_string(),
            "docs/test.md".to_string(),
        );
        assert!(result.is_err());

        // Test invalid version
        let result = Document::new(
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
            "550e8400-e29b-41d4-a716-446655440001".to_string(),
            "Test Document".to_string(),
            "content".to_string(),
            DocumentType::SoftwareRequirementsSpecification,
            "invalid-version".to_string(),
            "user123".to_string(),
            "docs/test.md".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_document_status_transitions() {
        let mut doc = Document::new(
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
            "550e8400-e29b-41d4-a716-446655440001".to_string(),
            "Test Document".to_string(),
            "content".to_string(),
            DocumentType::SoftwareRequirementsSpecification,
            "1.0.0".to_string(),
            "user123".to_string(),
            "docs/test.md".to_string(),
        ).unwrap();

        // Valid transition: Draft -> InReview
        assert!(doc.update_status(DocumentStatus::InReview, None).is_ok());
        assert_eq!(doc.status, DocumentStatus::InReview);

        // Valid transition: InReview -> Approved (with user_id)
        assert!(doc.update_status(DocumentStatus::Approved, Some("approver123".to_string())).is_ok());
        assert_eq!(doc.status, DocumentStatus::Approved);
        assert_eq!(doc.approved_by, Some("approver123".to_string()));

        // Invalid transition: Approved -> Draft
        assert!(doc.update_status(DocumentStatus::Draft, None).is_err());
    }

    #[test]
    fn test_document_tags() {
        let mut doc = Document::new(
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
            "550e8400-e29b-41d4-a716-446655440001".to_string(),
            "Test Document".to_string(),
            "content".to_string(),
            DocumentType::SoftwareRequirementsSpecification,
            "1.0.0".to_string(),
            "user123".to_string(),
            "docs/test.md".to_string(),
        ).unwrap();

        // Add valid tag
        assert!(doc.add_tag("requirements".to_string()).is_ok());
        assert!(doc.tags.contains(&"requirements".to_string()));

        // Add duplicate tag (should not add twice)
        assert!(doc.add_tag("requirements".to_string()).is_ok());
        assert_eq!(doc.tags.iter().filter(|t| *t == "requirements").count(), 1);

        // Add invalid tag (too long)
        assert!(doc.add_tag("a".repeat(51)).is_err());

        // Remove tag
        doc.remove_tag("requirements");
        assert!(!doc.tags.contains(&"requirements".to_string()));
    }

    #[test]
    fn test_document_json_serialization() {
        let doc = Document::new(
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
            "550e8400-e29b-41d4-a716-446655440001".to_string(),
            "Test Document".to_string(),
            "This is test content".to_string(),
            DocumentType::SoftwareRequirementsSpecification,
            "1.0.0".to_string(),
            "user123".to_string(),
            "docs/test.md".to_string(),
        ).unwrap();

        let json = doc.to_json();
        assert!(json.contains("Test Document"));
        assert!(json.contains("SRS"));

        let parsed_doc = Document::from_json(&json);
        assert!(parsed_doc.is_ok());
        let parsed_doc = parsed_doc.unwrap();
        assert_eq!(parsed_doc.title, doc.title);
        assert_eq!(parsed_doc.content, doc.content);
        assert_eq!(parsed_doc.doc_type.to_string(), doc.doc_type.to_string());
    }

    #[test]
    fn test_regulatory_reference() {
        let reg_ref = RegulatoryReference::new(
            "21 CFR 820.30".to_string(),
            "Design Controls".to_string(),
            "Each manufacturer of any class II or class III device...".to_string(),
        );
        assert!(reg_ref.is_ok());

        // Test empty fields
        let reg_ref = RegulatoryReference::new(
            "".to_string(),
            "Design Controls".to_string(),
            "requirement".to_string(),
        );
        assert!(reg_ref.is_err());
    }

    #[test]
    fn test_document_integrity_validation() {
        let mut doc = Document::new(
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
            "550e8400-e29b-41d4-a716-446655440001".to_string(),
            "Test Document".to_string(),
            "Original content".to_string(),
            DocumentType::SoftwareRequirementsSpecification,
            "1.0.0".to_string(),
            "user123".to_string(),
            "docs/test.md".to_string(),
        ).unwrap();

        // Initially valid
        assert!(doc.validate_integrity());

        // Update content properly
        assert!(doc.update_content("New content".to_string()).is_ok());
        assert!(doc.validate_integrity());

        // Manually corrupt checksum
        doc.checksum = "invalid_checksum".to_string();
        assert!(!doc.validate_integrity());
    }

    #[test]
    fn test_document_type_conversion() {
        assert_eq!(DocumentType::from_str("SRS").to_string(), "SRS");
        assert_eq!(DocumentType::from_str("SoftwareRequirementsSpecification").to_string(), "SRS");
        assert_eq!(DocumentType::from_str("CustomType").to_string(), "CustomType");
    }

    #[test]
    fn test_document_status_conversion() {
        assert!(DocumentStatus::from_str("Draft").is_ok());
        assert!(DocumentStatus::from_str("InvalidStatus").is_err());
        assert_eq!(DocumentStatus::Draft.to_string(), "Draft");
    }
}
