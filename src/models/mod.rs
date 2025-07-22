//! Core data models for QMS entities
//! Phase 2 infrastructure - comprehensive data structures for medical device QMS

#![allow(dead_code)] // Phase 2 infrastructure - core data models for document control, risk management, etc.

use std::path::PathBuf;

/// Project represents a QMS project
#[derive(Debug, Clone)]
pub struct Project {
    pub id: String,          // UUID v4 format
    pub name: String,        // max 100 chars
    pub description: String, // max 500 chars
    pub version: String,     // project version
    pub path: PathBuf,       // project directory path
    pub created_at: u64,     // UNIX timestamp
}

/// Document status enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum DocumentStatus {
    Draft,
    ReviewPending,
    InReview,
    Approved,
    Archived,
}

/// Document represents a controlled document
#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,             // DOC-YYYYMMDD-NNN format
    pub title: String,          // max 200 chars
    pub content: String,        // document content
    pub version: String,        // version number
    pub status: DocumentStatus, // current status
    pub author: String,         // username
    pub created_at: u64,        // UNIX timestamp
    pub updated_at: u64,        // UNIX timestamp
    pub locked: bool,           // document checkout status
    pub locked_by: Option<String>, // user who has document checked out
    pub locked_at: Option<u64>, // timestamp when document was locked
}

/// Document lock for checkout/checkin workflow
#[derive(Debug, Clone)]
pub struct DocumentLock {
    pub document_id: String,    // Document ID that is locked
    pub user_id: String,        // User who checked out the document
    pub locked_at: u64,         // Timestamp when locked
    pub lock_reason: Option<String>, // Optional reason for lock
}

/// Risk item for FMEA analysis
#[derive(Debug, Clone)]
pub struct RiskItem {
    pub id: String,                 // RISK-YYYYMMDD-NNN format
    pub description: String,        // max 500 chars
    pub severity: u8,               // 1-10 scale
    pub occurrence: u8,             // 1-10 scale
    pub detectability: u8,          // 1-10 scale
    pub rpn: u16,                   // calculated Risk Priority Number
    pub mitigation: Option<String>, // mitigation measures
    pub created_at: u64,            // UNIX timestamp
    pub updated_at: u64,            // UNIX timestamp
}

/// Requirement status enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum RequirementStatus {
    Draft,
    Approved,
    Implemented,
    Verified,
}

/// Requirement represents a system requirement
#[derive(Debug, Clone)]
pub struct Requirement {
    pub id: String,                // REQ-YYYYMMDD-NNN format
    pub text: String,              // max 1000 chars
    pub status: RequirementStatus, // current status
    pub priority: u8,              // 1-10 scale
    pub created_at: u64,           // UNIX timestamp
    pub updated_at: u64,           // UNIX timestamp
}

/// Test result enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum TestResult {
    NotRun,
    Pass,
    Fail,
    Blocked,
}

/// Test case for verification and validation
#[derive(Debug, Clone)]
pub struct TestCase {
    pub id: String,                    // TC-YYYYMMDD-NNN format
    pub description: String,           // max 500 chars
    pub steps: Vec<String>,            // test steps
    pub expected_result: String,       // expected outcome
    pub actual_result: Option<String>, // actual outcome
    pub result: TestResult,            // test result
    pub created_at: u64,               // UNIX timestamp
    pub executed_at: Option<u64>,      // execution timestamp
}

/// Audit action enumeration for regulatory compliance
#[derive(Debug, Clone, PartialEq)]
pub enum AuditAction {
    Create,         // Entity creation
    Read,           // Entity access/viewing
    Update,         // Entity modification
    Delete,         // Entity deletion
    Archive,        // Entity archiving
    Restore,        // System restoration
    Approve,        // Document/risk approval
    Reject,         // Document/risk rejection
    Submit,         // Workflow submission
    Checkout,       // Document checkout
    Checkin,        // Document checkin
    Login,          // User authentication
    Logout,         // User session end
    Export,         // Data export operation
    Import,         // Data import operation
    Configure,      // System configuration change
    Other(String),  // Custom action type
}

/// Electronic signature for critical operations (21 CFR Part 11)
#[derive(Debug, Clone)]
pub struct ElectronicSignature {
    pub user_id: String,               // Signing user
    pub timestamp: String,             // Signature timestamp (ISO 8601)
    pub meaning: String,               // Signature meaning/intent
    pub signed_data_hash: String,      // SHA-256 hash of signed data
    pub certificate_info: Option<String>, // Digital certificate info if used
}

/// Comprehensive audit entry for regulatory compliance (21 CFR Part 11, ISO 13485)
#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub id: String,                    // UUID v4
    pub timestamp: String,             // ISO 8601 timestamp
    pub user_id: String,               // User performing action
    pub session_id: Option<String>,    // User session ID
    pub action: AuditAction,           // Action performed
    pub entity_type: String,           // Type of entity (Document, Risk, etc.)
    pub entity_id: String,             // ID of affected entity
    pub old_value: Option<String>,     // Previous value (for updates)
    pub new_value: Option<String>,     // New value (for updates)
    pub details: Option<String>,       // Additional details
    pub ip_address: Option<String>,    // Client IP address (for web access)
    pub signature: Option<ElectronicSignature>, // Electronic signature if required
    pub checksum: String,              // Entry integrity checksum
    pub previous_hash: Option<String>, // Hash of previous entry for chain integrity
}

/// User permission enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
    ReadDocuments,
    WriteDocuments,
    DeleteDocuments,
    ReadRisks,
    WriteRisks,
    DeleteRisks,
    ReadTrace,
    WriteTrace,
    DeleteTrace,
    ReadAudit,
    ExportAudit,
    ManageUsers,
    GenerateReports,
}

/// User role with permissions
#[derive(Debug, Clone)]
pub struct Role {
    pub name: String,
    pub permissions: Vec<Permission>,
}

/// User account
#[derive(Debug, Clone)]
pub struct User {
    pub username: String,        // 3-50 chars, alphanumeric+underscore
    pub password_hash: String,   // SHA256 hash
    pub roles: Vec<Role>,        // assigned roles
    pub created_at: u64,         // UNIX timestamp
    pub last_login: Option<u64>, // last login timestamp
}

impl Project {
    /// Validate project name (max 100 chars)
    pub const fn validate_name(name: &str) -> bool {
        !name.is_empty() && name.len() <= 100
    }
}

impl Document {
    /// Validate document title (max 200 chars)
    pub const fn validate_title(title: &str) -> bool {
        !title.is_empty() && title.len() <= 200
    }
}

impl RiskItem {
    /// Calculate Risk Priority Number (RPN) - DRY: uses common calculation
    pub fn calculate_rpn(&mut self) {
        self.rpn = crate::utils::SimpleCalculations::calculate_rpn(
            self.severity,
            self.occurrence,
            self.detectability
        );
    }

    /// Validate severity range - DRY: uses common validation
    pub fn validate_severity(severity: u8) -> bool {
        crate::utils::CommonValidation::validate_severity(severity)
    }

    /// Validate occurrence range - DRY: uses common validation
    pub fn validate_occurrence(occurrence: u8) -> bool {
        crate::utils::CommonValidation::validate_occurrence(occurrence)
    }

    /// Validate detectability range - DRY: uses common validation
    pub fn validate_detectability(detectability: u8) -> bool {
        crate::utils::CommonValidation::validate_detectability(detectability)
    }
}

impl Requirement {
    /// Validate requirement text - DRY: uses common validation
    pub const fn validate_text(text: &str) -> bool {
        crate::utils::CommonValidation::validate_text_content(text)
    }
}

impl TestCase {
    /// Validate test description (max 500 chars)
    pub const fn validate_description(description: &str) -> bool {
        !description.is_empty() && description.len() <= 500
    }
}

impl User {
    /// Validate username (3-50 chars, alphanumeric+underscore)
    pub fn validate_username(username: &str) -> bool {
        if username.len() < 3 || username.len() > 50 {
            return false;
        }
        username.chars().all(|c| c.is_alphanumeric() || c == '_')
    }
}
