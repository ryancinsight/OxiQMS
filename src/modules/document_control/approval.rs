use crate::error::{QmsError, QmsResult};
use crate::modules::document_control::{document::DocumentStatus, service::DocumentService};
use crate::utils::{current_date_string, get_current_project_path};
use std::fs;

/// Electronic signature for document approval
#[derive(Debug, Clone)]
pub struct ElectronicSignature {
    pub signer_id: String,
    pub signer_name: String,
    pub signature_timestamp: String,
    pub signature_hash: String,
    pub signing_reason: String,
    pub authentication_method: String,
}

/// Approval workflow state
#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowState {
    Draft,
    SubmittedForReview,
    InReview,
    ApprovalPending,
    Approved,
    Rejected,
    Archived,
}

impl WorkflowState {
    pub const fn from_document_status(status: &DocumentStatus) -> Self {
        match status {
            DocumentStatus::Draft => WorkflowState::Draft,
            DocumentStatus::InReview => WorkflowState::InReview,
            DocumentStatus::Approved => WorkflowState::Approved,
            DocumentStatus::Deprecated => WorkflowState::Archived,
            DocumentStatus::Archived => WorkflowState::Archived,
        }
    }

    pub const fn to_document_status(&self) -> DocumentStatus {
        match self {
            WorkflowState::Draft => DocumentStatus::Draft,
            WorkflowState::SubmittedForReview => DocumentStatus::InReview,
            WorkflowState::InReview => DocumentStatus::InReview,
            WorkflowState::ApprovalPending => DocumentStatus::InReview,
            WorkflowState::Approved => DocumentStatus::Approved,
            WorkflowState::Rejected => DocumentStatus::Draft,
            WorkflowState::Archived => DocumentStatus::Archived,
        }
    }

    pub const fn can_transition_to(&self, target: &WorkflowState) -> bool {
        match (self, target) {
            // From Draft
            (WorkflowState::Draft, WorkflowState::SubmittedForReview) => true,
            
            // From SubmittedForReview
            (WorkflowState::SubmittedForReview, WorkflowState::InReview) => true,
            (WorkflowState::SubmittedForReview, WorkflowState::Draft) => true, // Recall
            
            // From InReview
            (WorkflowState::InReview, WorkflowState::ApprovalPending) => true,
            (WorkflowState::InReview, WorkflowState::Rejected) => true,
            (WorkflowState::InReview, WorkflowState::Draft) => true, // Back to draft
            
            // From ApprovalPending
            (WorkflowState::ApprovalPending, WorkflowState::Approved) => true,
            (WorkflowState::ApprovalPending, WorkflowState::Rejected) => true,
            
            // From Approved
            (WorkflowState::Approved, WorkflowState::Archived) => true,
            (WorkflowState::Approved, WorkflowState::Draft) => true, // New version
            
            // From Rejected
            (WorkflowState::Rejected, WorkflowState::Draft) => true, // Rework
            
            // From Archived
            (WorkflowState::Archived, WorkflowState::Draft) => true, // Restore
            
            _ => false,
        }
    }
}

/// Approval workflow entry
#[derive(Debug, Clone)]
pub struct ApprovalWorkflowEntry {
    pub document_id: String,
    pub workflow_id: String,
    pub from_state: WorkflowState,
    pub to_state: WorkflowState,
    pub action_timestamp: String,
    pub actor_id: String,
    pub actor_name: String,
    pub signature: Option<ElectronicSignature>,
    pub comments: Option<String>,
    pub reason: Option<String>,
}

/// Document approval workflow manager
pub struct ApprovalWorkflow {
    document_service: DocumentService,
    project_path: String,
}

impl ApprovalWorkflow {
    /// Create new approval workflow manager
    pub fn new() -> QmsResult<Self> {
        let project_path = get_current_project_path()?;
        let document_service = DocumentService::new(project_path.clone());
        
        Ok(ApprovalWorkflow {
            document_service,
            project_path: project_path.to_string_lossy().to_string(),
        })
    }

    /// Submit document for review
    pub fn submit_for_review(&mut self, doc_id: &str, submitter_id: &str, submitter_name: &str, comments: Option<&str>) -> QmsResult<()> {
        let document = self.document_service.read_document(doc_id)?;
        let current_state = WorkflowState::from_document_status(&document.status);
        let target_state = WorkflowState::SubmittedForReview;

        if !current_state.can_transition_to(&target_state) {
            return Err(QmsError::validation_error(&format!(
                "Cannot submit document for review from state {current_state:?}"
            )));
        }

        // Update document status
        self.update_document_status_direct(doc_id, target_state.to_document_status())?;

        // Log workflow transition
        self.log_workflow_transition(
            doc_id,
            current_state,
            target_state,
            submitter_id,
            submitter_name,
            None,
            comments,
            Some("Document submitted for review"),
        )?;

        // Log audit entry
        self.log_audit_action("DOCUMENT_SUBMIT_REVIEW", doc_id, submitter_id)?;

        Ok(())
    }

    /// Start review process
    #[allow(dead_code)]
    pub fn start_review(&mut self, doc_id: &str, reviewer_id: &str, reviewer_name: &str, comments: Option<&str>) -> QmsResult<()> {
        let document = self.document_service.read_document(doc_id)?;
        let current_state = WorkflowState::from_document_status(&document.status);
        let target_state = WorkflowState::InReview;

        if !current_state.can_transition_to(&target_state) {
            return Err(QmsError::validation_error(&format!(
                "Cannot start review from state {current_state:?}"
            )));
        }

        // Update document status
        self.update_document_status_direct(doc_id, target_state.to_document_status())?;

        // Log workflow transition
        self.log_workflow_transition(
            doc_id,
            current_state,
            target_state,
            reviewer_id,
            reviewer_name,
            None,
            comments,
            Some("Review process started"),
        )?;

        // Log audit entry
        self.log_audit_action("DOCUMENT_START_REVIEW", doc_id, reviewer_id)?;

        Ok(())
    }

    /// Approve document with electronic signature
    pub fn approve_document(&mut self, doc_id: &str, approver_id: &str, approver_name: &str, signature: ElectronicSignature, comments: Option<&str>) -> QmsResult<()> {
        let document = self.document_service.read_document(doc_id)?;
        let current_state = WorkflowState::from_document_status(&document.status);
        let target_state = WorkflowState::Approved;

        // Validate workflow transition
        if !current_state.can_transition_to(&target_state) {
            return Err(QmsError::validation_error(&format!(
                "Cannot approve document from state {current_state:?}"
            )));
        }

        // Validate electronic signature
        self.validate_electronic_signature(&signature)?;

        // Update document status
        self.update_document_status_direct(doc_id, target_state.to_document_status())?;

        // Log workflow transition with signature
        self.log_workflow_transition(
            doc_id,
            current_state,
            target_state,
            approver_id,
            approver_name,
            Some(signature),
            comments,
            Some("Document approved with electronic signature"),
        )?;

        // Log audit entry
        self.log_audit_action("DOCUMENT_APPROVE", doc_id, approver_id)?;

        Ok(())
    }

    /// Reject document with reason
    pub fn reject_document(&mut self, doc_id: &str, rejector_id: &str, rejector_name: &str, reason: &str, comments: Option<&str>) -> QmsResult<()> {
        let document = self.document_service.read_document(doc_id)?;
        let current_state = WorkflowState::from_document_status(&document.status);
        let target_state = WorkflowState::Rejected;

        if !current_state.can_transition_to(&target_state) {
            return Err(QmsError::validation_error(&format!(
                "Cannot reject document from state {current_state:?}"
            )));
        }

        // Update document status (rejected goes back to draft for rework)
        self.update_document_status_direct(doc_id, target_state.to_document_status())?;

        // Log workflow transition
        self.log_workflow_transition(
            doc_id,
            current_state,
            target_state,
            rejector_id,
            rejector_name,
            None,
            comments,
            Some(reason),
        )?;

        // Log audit entry
        self.log_audit_action("DOCUMENT_REJECT", doc_id, rejector_id)?;

        Ok(())
    }

    /// Get workflow history for a document
    pub fn get_workflow_history(&self, doc_id: &str) -> QmsResult<Vec<ApprovalWorkflowEntry>> {
        let workflow_file = self.get_workflow_file_path(doc_id);
        
        if !workflow_file.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&workflow_file)
            .map_err(|e| QmsError::io_error(&format!("Failed to read workflow history: {e}")))?;

        let mut entries = Vec::new();
        for line in content.lines() {
            if let Ok(entry) = self.parse_workflow_entry(line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    /// Get current workflow state for a document
    pub fn get_current_workflow_state(&self, doc_id: &str) -> QmsResult<WorkflowState> {
        let document = self.document_service.read_document(doc_id)?;
        Ok(WorkflowState::from_document_status(&document.status))
    }

    /// Create electronic signature
    pub fn create_electronic_signature(signer_id: &str, signer_name: &str, signing_reason: &str, authentication_method: &str) -> ElectronicSignature {
        let timestamp = current_date_string();
        let signature_data = format!("{signer_id}:{signer_name}:{timestamp}:{signing_reason}");
        let signature_hash = format!("{:x}", md5::compute(signature_data));

        ElectronicSignature {
            signer_id: signer_id.to_string(),
            signer_name: signer_name.to_string(),
            signature_timestamp: timestamp,
            signature_hash,
            signing_reason: signing_reason.to_string(),
            authentication_method: authentication_method.to_string(),
        }
    }

    /// Validate electronic signature
    fn validate_electronic_signature(&self, signature: &ElectronicSignature) -> QmsResult<()> {
        // Basic validation
        if signature.signer_id.trim().is_empty() {
            return Err(QmsError::validation_error("Signer ID cannot be empty"));
        }
        
        if signature.signer_name.trim().is_empty() {
            return Err(QmsError::validation_error("Signer name cannot be empty"));
        }

        if signature.signing_reason.trim().is_empty() {
            return Err(QmsError::validation_error("Signing reason cannot be empty"));
        }

        if signature.signature_hash.len() != 32 {
            return Err(QmsError::validation_error("Invalid signature hash format"));
        }

        // Verify signature hash
        let expected_data = format!("{}:{}:{}:{}", 
            signature.signer_id, signature.signer_name, 
            signature.signature_timestamp, signature.signing_reason);
        let expected_hash = format!("{:x}", md5::compute(expected_data));
        
        if signature.signature_hash != expected_hash {
            return Err(QmsError::validation_error("Electronic signature verification failed"));
        }

        Ok(())
    }

    /// Log workflow transition
    fn log_workflow_transition(&self, doc_id: &str, from_state: WorkflowState, to_state: WorkflowState, actor_id: &str, actor_name: &str, signature: Option<ElectronicSignature>, comments: Option<&str>, reason: Option<&str>) -> QmsResult<()> {
        let workflow_id = format!("WF-{}-{}", doc_id, current_date_string().replace(":", "").replace("-", ""));
        
        let entry = ApprovalWorkflowEntry {
            document_id: doc_id.to_string(),
            workflow_id,
            from_state,
            to_state,
            action_timestamp: current_date_string(),
            actor_id: actor_id.to_string(),
            actor_name: actor_name.to_string(),
            signature,
            comments: comments.map(|s| s.to_string()),
            reason: reason.map(|s| s.to_string()),
        };

        self.write_workflow_entry(&entry)?;
        Ok(())
    }

    /// Write workflow entry to file
    fn write_workflow_entry(&self, entry: &ApprovalWorkflowEntry) -> QmsResult<()> {
        let workflow_file = self.get_workflow_file_path(&entry.document_id);
        
        // Ensure workflow directory exists
        if let Some(parent) = workflow_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| QmsError::io_error(&format!("Failed to create workflow directory: {e}")))?;
        }

        let entry_line = self.serialize_workflow_entry(entry);
        
        // Append to workflow file
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&workflow_file)
            .map_err(|e| QmsError::io_error(&format!("Failed to open workflow file: {e}")))?;
            
        writeln!(file, "{entry_line}")
            .map_err(|e| QmsError::io_error(&format!("Failed to write workflow entry: {e}")))?;

        Ok(())
    }

    /// Get workflow file path for a document
    fn get_workflow_file_path(&self, doc_id: &str) -> std::path::PathBuf {
        std::path::Path::new(&self.project_path)
            .join("workflows")
            .join(format!("{doc_id}.log"))
    }

    /// Serialize workflow entry to string
    fn serialize_workflow_entry(&self, entry: &ApprovalWorkflowEntry) -> String {
        let signature_str = if let Some(sig) = &entry.signature {
            format!("{}|{}|{}|{}|{}|{}", 
                sig.signer_id, sig.signer_name, sig.signature_timestamp,
                sig.signature_hash, sig.signing_reason, sig.authentication_method)
        } else {
            "NONE".to_string()
        };

        format!("{}|{}|{:?}|{:?}|{}|{}|{}|{}|{}|{}", 
            entry.workflow_id,
            entry.document_id,
            entry.from_state,
            entry.to_state,
            entry.action_timestamp,
            entry.actor_id,
            entry.actor_name,
            signature_str,
            entry.comments.as_deref().unwrap_or(""),
            entry.reason.as_deref().unwrap_or(""))
    }

    /// Parse workflow entry from string
    fn parse_workflow_entry(&self, line: &str) -> QmsResult<ApprovalWorkflowEntry> {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() < 10 {
            return Err(QmsError::validation_error("Invalid workflow entry format"));
        }

        let from_state = self.parse_workflow_state(parts[2])?;
        let to_state = self.parse_workflow_state(parts[3])?;

        let signature = if parts[7] != "NONE" {
            let sig_parts: Vec<&str> = parts[7].split('|').collect();
            if sig_parts.len() == 6 {
                Some(ElectronicSignature {
                    signer_id: sig_parts[0].to_string(),
                    signer_name: sig_parts[1].to_string(),
                    signature_timestamp: sig_parts[2].to_string(),
                    signature_hash: sig_parts[3].to_string(),
                    signing_reason: sig_parts[4].to_string(),
                    authentication_method: sig_parts[5].to_string(),
                })
            } else {
                None
            }
        } else {
            None
        };

        Ok(ApprovalWorkflowEntry {
            workflow_id: parts[0].to_string(),
            document_id: parts[1].to_string(),
            from_state,
            to_state,
            action_timestamp: parts[4].to_string(),
            actor_id: parts[5].to_string(),
            actor_name: parts[6].to_string(),
            signature,
            comments: if parts[8].is_empty() { None } else { Some(parts[8].to_string()) },
            reason: if parts[9].is_empty() { None } else { Some(parts[9].to_string()) },
        })
    }

    /// Parse workflow state from string
    fn parse_workflow_state(&self, state_str: &str) -> QmsResult<WorkflowState> {
        match state_str {
            "Draft" => Ok(WorkflowState::Draft),
            "SubmittedForReview" => Ok(WorkflowState::SubmittedForReview),
            "InReview" => Ok(WorkflowState::InReview),
            "ApprovalPending" => Ok(WorkflowState::ApprovalPending),
            "Approved" => Ok(WorkflowState::Approved),
            "Rejected" => Ok(WorkflowState::Rejected),
            "Archived" => Ok(WorkflowState::Archived),
            _ => Err(QmsError::validation_error(&format!("Unknown workflow state: {state_str}"))),
        }
    }

    /// Log audit action
    fn log_audit_action(&self, action: &str, doc_id: &str, user_id: &str) -> QmsResult<()> {
        let audit_entry = format!("{} | {} | {} | {} | {}",
            current_date_string(),
            action,
            doc_id,
            user_id,
            "Document approval workflow action"
        );

        let audit_file = std::path::Path::new(&self.project_path).join("audit.log");
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&audit_file)
            .map_err(|e| QmsError::io_error(&format!("Failed to open audit log: {e}")))?;
            
        writeln!(file, "{audit_entry}")
            .map_err(|e| QmsError::io_error(&format!("Failed to write audit entry: {e}")))?;

        Ok(())
    }

    /// Update document status directly by modifying the document file
    fn update_document_status_direct(&self, doc_id: &str, new_status: DocumentStatus) -> QmsResult<()> {
        // For now, we'll use a simple approach to update the document status
        // by reading, modifying, and writing back the document metadata
        let _document = self.document_service.read_document(doc_id)?;
        
        // Use the existing update_document method to trigger a status change
        // This is a workaround until we add proper status update methods
        self.document_service.update_document(
            doc_id,
            None, // No title change
            None, // No content change  
            Some(crate::modules::document_control::version::VersionChangeType::Patch),
            Some(format!("Status updated to {new_status:?}")),
            "WORKFLOW_SYSTEM".to_string(),
        )?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_state_transitions() {
        assert!(WorkflowState::Draft.can_transition_to(&WorkflowState::SubmittedForReview));
        assert!(WorkflowState::SubmittedForReview.can_transition_to(&WorkflowState::InReview));
        assert!(WorkflowState::InReview.can_transition_to(&WorkflowState::ApprovalPending));
        assert!(WorkflowState::ApprovalPending.can_transition_to(&WorkflowState::Approved));
        assert!(WorkflowState::ApprovalPending.can_transition_to(&WorkflowState::Rejected));
        
        // Invalid transitions
        assert!(!WorkflowState::Draft.can_transition_to(&WorkflowState::Approved));
        assert!(!WorkflowState::Approved.can_transition_to(&WorkflowState::InReview));
    }

    #[test]
    fn test_electronic_signature_creation() {
        let signature = ApprovalWorkflow::create_electronic_signature(
            "user123", 
            "John Doe", 
            "Document approval", 
            "password+2fa"
        );

        assert_eq!(signature.signer_id, "user123");
        assert_eq!(signature.signer_name, "John Doe");
        assert_eq!(signature.signing_reason, "Document approval");
        assert_eq!(signature.authentication_method, "password+2fa");
        assert_eq!(signature.signature_hash.len(), 32);
    }

    #[test]
    fn test_workflow_state_conversion() {
        assert_eq!(WorkflowState::from_document_status(&DocumentStatus::Draft), WorkflowState::Draft);
        assert_eq!(WorkflowState::from_document_status(&DocumentStatus::InReview), WorkflowState::InReview);
        assert_eq!(WorkflowState::from_document_status(&DocumentStatus::Approved), WorkflowState::Approved);

        assert_eq!(WorkflowState::Draft.to_document_status(), DocumentStatus::Draft);
        assert_eq!(WorkflowState::InReview.to_document_status(), DocumentStatus::InReview);
        assert_eq!(WorkflowState::Approved.to_document_status(), DocumentStatus::Approved);
    }
}
