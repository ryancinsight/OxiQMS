//! Document checkout/checkin functionality for QMS document control
//! Phase 2.1.5 - Document Checkout/Checkin System
//! Implements collaborative editing workflow with document locking
//! Enhanced with robust file-level locking from lock.rs

#![allow(dead_code)] // Phase 2 infrastructure - checkout/checkin system

use crate::error::{QmsError, QmsResult};
use crate::json_utils::JsonSerializable;
use crate::utils::current_timestamp;
use crate::lock::LockGuard; // Import robust file locking system
use crate::modules::audit_logger::audit_log_action; // New integrated audit logging
use super::document::{Document, DocumentLock};
use super::service::DocumentService;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;

/// Detailed lock information for administrative purposes
#[derive(Debug, Clone)]
pub struct DetailedLockInfo {
    pub document_lock: DocumentLock,
    pub has_file_lock: bool,
    pub file_lock_holder: Option<String>,
    pub is_active_in_manager: bool,
}

/// Report for lock cleanup operations
#[derive(Debug, Clone)]
pub struct CleanupReport {
    pub cleaned_document_locks: usize,
    pub cleaned_file_locks: usize,
    pub errors: Vec<String>,
}

/// Lock statistics for administrative monitoring
#[derive(Debug, Clone)]
pub struct LockStatistics {
    pub total_document_locks: usize,
    pub total_file_locks: usize,
    pub active_in_manager: usize,
    pub longest_held_lock_duration: Option<std::time::Duration>,
}

/// Helper function to read file content
fn read_file_content(path: &str) -> QmsResult<String> {
    fs::read_to_string(path)
        .map_err(|e| QmsError::io_error(&format!("Failed to read file {path}: {e}")))
}

/// Helper function to write file content
fn write_file_content(path: &str, content: &str) -> QmsResult<()> {
    fs::write(path, content)
        .map_err(|e| QmsError::io_error(&format!("Failed to write file {path}: {e}")))
}

/// Helper function to create directory if it doesn't exist
fn create_directory_if_not_exists(path: &str) -> QmsResult<()> {
    if !Path::new(path).exists() {
        fs::create_dir_all(path)
            .map_err(|e| QmsError::io_error(&format!("Failed to create directory {path}: {e}")))?;
    }
    Ok(())
}

/// Document checkout/checkin manager with robust file locking
pub struct CheckoutManager {
    pub project_path: String,
    active_locks: HashMap<String, LockGuard>, // Track active file locks
}

impl CheckoutManager {
    /// Create a new checkout manager for a project
    pub fn new(project_path: String) -> Self {
        Self { 
            project_path,
            active_locks: HashMap::new(),
        }
    }

    /// Checkout a document for editing (locks the document with robust file locking)
    pub fn checkout_document(&mut self, doc_id: &str, user_id: &str, reason: Option<String>) -> QmsResult<DocumentLock> {
        // Load the document to check if it exists and isn't already locked
        let doc = self.load_document(doc_id)?;
        
        if doc.locked {
            return Err(QmsError::validation_error(&format!(
                "Document {} is already checked out by user: {}", 
                doc_id, 
                doc.locked_by.unwrap_or_else(|| "unknown".to_string())
            )));
        }

        // Get the document content file path for file-level locking
        let doc_content_path = PathBuf::from(&self.project_path)
            .join("documents")
            .join(doc_id)
            .join("content.md");

        // Acquire robust file lock
        let file_lock = LockGuard::acquire(&doc_content_path, user_id)
            .map_err(|e| QmsError::validation_error(&format!(
                "Failed to acquire file lock for document {doc_id}: {e}"
            )))?;

        // Create document lock
        let lock = DocumentLock {
            document_id: doc_id.to_string(),
            user_id: user_id.to_string(),
            locked_at: current_timestamp().to_string(),
            lock_reason: reason,
        };

        // Save the lock file
        self.save_lock(&lock)?;

        // Update document to mark as locked
        self.mark_document_locked(doc_id, user_id)?;

        // Store the file lock guard (keeping the lock alive)
        self.active_locks.insert(doc_id.to_string(), file_lock);

        // Audit log the checkout
        audit_log_action("CHECKOUT", "Document", doc_id)?;

        Ok(lock)
    }

    /// Checkin a document with optional new content (unlocks the document and releases file lock)
    pub fn checkin_document(
        &mut self, 
        doc_id: &str, 
        user_id: &str, 
        new_content_path: Option<&str>,
        _message: Option<&str>
    ) -> QmsResult<Document> {
        // Validate lock ownership
        self.validate_lock_ownership(doc_id, user_id)?;

        // Load the current document
        let mut doc = self.load_document(doc_id)?;

        // Update content if provided
        if let Some(path) = new_content_path {
            let new_content = read_file_content(path)?;
            doc.content = new_content;
            doc.checksum = crate::json_utils::calculate_checksum(&doc.content);
        }

        // Update timestamp
        doc.updated_at = current_timestamp().to_string();

        // Create version snapshot before updating
        let _doc_service = DocumentService::new(PathBuf::from(self.project_path.clone()));
        
        // For now, just save the version - we'll integrate proper version control later
        // doc_service.create_version_snapshot(&doc)?;

        // Increment version if content changed (simplified for now)
        if new_content_path.is_some() {
            // For now, just increment patch version manually
            // TODO: Integrate with proper version control system
            let parts: Vec<&str> = doc.version.split('.').collect();
            if parts.len() == 3 {
                if let Ok(patch) = parts[2].parse::<u32>() {
                    doc.version = format!("{}.{}.{}", parts[0], parts[1], patch + 1);
                }
            }
        }

        // Release the document lock file first
        self.release_document_lock(doc_id)?;

        // Release the robust file lock by removing from active locks
        if let Some(_file_lock) = self.active_locks.remove(doc_id) {
            // LockGuard will automatically release on drop
            audit_log_action("FILE_LOCK_RELEASED", "Document", doc_id)?;
        }

        // Mark document as unlocked
        doc.locked = false;
        doc.locked_by = None;
        doc.locked_at = None;

        // Save the updated document
        self.save_document(&doc)?;

        // Audit log the checkin
        audit_log_action("CHECKIN", "Document", doc_id)?;

        Ok(doc)
    }

    /// Get checkout status for a document
    pub fn get_checkout_status(&self, doc_id: &str) -> QmsResult<Option<DocumentLock>> {
        let lock_path = self.get_lock_file_path(doc_id);
        
        if Path::new(&lock_path).exists() {
            let lock_content = read_file_content(&lock_path)?;
            let lock = DocumentLock::from_json(&lock_content)
                .map_err(|e| QmsError::parse_error(&format!("Failed to parse lock file: {e}")))?;
            Ok(Some(lock))
        } else {
            Ok(None)
        }
    }

    /// Force release a document lock (admin operation) with robust file lock cleanup
    pub fn force_release_lock(&mut self, doc_id: &str, _admin_user: &str, _reason: &str) -> QmsResult<()> {
        // Get the document content file path for force releasing file lock
        let doc_content_path = PathBuf::from(&self.project_path)
            .join("documents")
            .join(doc_id)
            .join("content.md");

        // Force release the robust file lock
        if let Err(_e) = LockGuard::force_release_lock(&doc_content_path) {
            // Log warning but continue with document lock release
            audit_log_action("FILE_LOCK_FORCE_RELEASE_WARNING", "Document", doc_id)?;
        }

        // Remove from active locks if present
        if let Some(_file_lock) = self.active_locks.remove(doc_id) {
            audit_log_action("ACTIVE_LOCK_REMOVED", "Document", doc_id)?;
        }

        // Release the document lock file
        self.release_document_lock(doc_id)?;

        // Mark document as unlocked
        self.mark_document_unlocked(doc_id)?;

        // Audit log the force release
        audit_log_action("FORCE_UNLOCK", "Document", doc_id)?;

        Ok(())
    }

    /// List all currently locked documents with enhanced lock information
    pub fn list_locked_documents(&self) -> QmsResult<Vec<DocumentLock>> {
        let locks_dir = format!("{}/locks", self.project_path);
        let mut locked_docs = Vec::new();

        if !Path::new(&locks_dir).exists() {
            return Ok(locked_docs);
        }

        let entries = std::fs::read_dir(&locks_dir)
            .map_err(|e| QmsError::io_error(&format!("Failed to read locks directory: {e}")))?;

        for entry in entries {
            let entry = entry.map_err(|e| QmsError::io_error(&format!("Failed to read directory entry: {e}")))?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                let path_str = path.to_string_lossy().to_string();
                if let Ok(content) = read_file_content(&path_str) {
                    if let Ok(lock) = DocumentLock::from_json(&content) {
                        locked_docs.push(lock);
                    }
                }
            }
        }

        Ok(locked_docs)
    }

    /// Get detailed lock information including file lock status
    pub fn get_detailed_lock_info(&self, doc_id: &str) -> QmsResult<Option<DetailedLockInfo>> {
        // Check document lock
        let doc_lock = self.get_checkout_status(doc_id)?;
        
        // Check file lock status
        let doc_content_path = PathBuf::from(&self.project_path)
            .join("documents")
            .join(doc_id)
            .join("content.md");
        
        let file_lock_info = LockGuard::get_lock_status(&doc_content_path);
        
        if let Some(doc_lock) = doc_lock {
            Ok(Some(DetailedLockInfo {
                document_lock: doc_lock,
                has_file_lock: file_lock_info.is_some(),
                file_lock_holder: file_lock_info.map(|(holder, _)| holder),
                is_active_in_manager: self.active_locks.contains_key(doc_id),
            }))
        } else {
            Ok(None)
        }
    }

    /// Clean up stale locks (admin utility)
    pub fn cleanup_stale_locks(&mut self, admin_user: &str) -> QmsResult<CleanupReport> {
        use std::time::{Duration, SystemTime};

        let mut report = CleanupReport {
            cleaned_document_locks: 0,
            cleaned_file_locks: 0,
            errors: Vec::new(),
        };

        // Clean up document locks
        let locks_dir = format!("{}/locks", self.project_path);
        if Path::new(&locks_dir).exists() {
            let entries = std::fs::read_dir(&locks_dir)
                .map_err(|e| QmsError::io_error(&format!("Failed to read locks directory: {e}")))?;

            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                        // Check if lock is stale (e.g., older than 24 hours without activity)
                        if let Ok(metadata) = path.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                                    if elapsed > Duration::from_secs(24 * 60 * 60) { // 24 hours
                                        if let Some(filename) = path.file_stem() {
                                            let doc_id = filename.to_string_lossy();
                                            match self.force_release_lock(&doc_id, admin_user, "Stale lock cleanup") {
                                                Ok(_) => {
                                                    report.cleaned_document_locks += 1;
                                                    audit_log_action("STALE_LOCK_CLEANED", "Document", &doc_id)?;
                                                }
                                                Err(e) => {
                                                    report.errors.push(format!("Failed to clean lock for {doc_id}: {e}"));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Clean up file locks using lock utilities
        let documents_dir = PathBuf::from(&self.project_path).join("documents");
        if let Ok(cleaned_file_locks) = crate::lock::lock_utils::cleanup_stale_locks(&documents_dir, Duration::from_secs(24 * 60 * 60)) {
            report.cleaned_file_locks = cleaned_file_locks;
        }

        Ok(report)
    }

    // Administrative utilities

    /// Get system-wide lock statistics (admin utility)
    pub fn get_lock_statistics(&self) -> QmsResult<LockStatistics> {
        let locks = self.list_locked_documents()?;
        let documents_dir = PathBuf::from(&self.project_path).join("documents");
        
        let file_locks = crate::lock::lock_utils::list_active_locks(&documents_dir)
            .unwrap_or_else(|_| Vec::new());

        Ok(LockStatistics {
            total_document_locks: locks.len(),
            total_file_locks: file_locks.len(),
            active_in_manager: self.active_locks.len(),
            longest_held_lock_duration: self.calculate_longest_lock_duration(&locks),
        })
    }

    /// Calculate the duration of the longest held lock
    fn calculate_longest_lock_duration(&self, locks: &[DocumentLock]) -> Option<std::time::Duration> {
        use std::time::SystemTime;
        
        locks.iter()
            .filter_map(|lock| {
                // Parse the timestamp and calculate duration
                if let Ok(timestamp) = lock.locked_at.parse::<u64>() {
                    let locked_time = std::time::UNIX_EPOCH + std::time::Duration::from_secs(timestamp);
                    SystemTime::now().duration_since(locked_time).ok()
                } else {
                    None
                }
            })
            .max()
    }

    // Private helper methods

    /// Load a document from file
    fn load_document(&self, doc_id: &str) -> QmsResult<Document> {
        let doc_path = format!("{}/documents/{}/metadata.json", self.project_path, doc_id);
        let content = read_file_content(&doc_path)?;
        Document::from_json(&content)
            .map_err(|e| QmsError::parse_error(&format!("Failed to parse document: {e}")))
    }

    /// Save a document to file
    fn save_document(&self, doc: &Document) -> QmsResult<()> {
        let doc_dir = format!("{}/documents/{}", self.project_path, doc.id);
        create_directory_if_not_exists(&doc_dir)?;
        
        let doc_path = format!("{doc_dir}/metadata.json");
        write_file_content(&doc_path, &doc.to_json())?;
        Ok(())
    }

    /// Save a document lock to file
    fn save_lock(&self, lock: &DocumentLock) -> QmsResult<()> {
        let locks_dir = format!("{}/locks", self.project_path);
        create_directory_if_not_exists(&locks_dir)?;
        
        let lock_path = self.get_lock_file_path(&lock.document_id);
        write_file_content(&lock_path, &lock.to_json())?;
        Ok(())
    }

    /// Release a document lock (delete lock file)
    fn release_document_lock(&self, doc_id: &str) -> QmsResult<()> {
        let lock_path = self.get_lock_file_path(doc_id);
        
        if Path::new(&lock_path).exists() {
            std::fs::remove_file(&lock_path)
                .map_err(|e| QmsError::io_error(&format!("Failed to remove lock file: {e}")))?;
        }
        
        Ok(())
    }

    /// Mark document as locked in its metadata
    fn mark_document_locked(&self, doc_id: &str, user_id: &str) -> QmsResult<()> {
        let mut doc = self.load_document(doc_id)?;
        doc.locked = true;
        doc.locked_by = Some(user_id.to_string());
        doc.locked_at = Some(current_timestamp().to_string());
        self.save_document(&doc)
    }

    /// Mark document as unlocked in its metadata
    fn mark_document_unlocked(&self, doc_id: &str) -> QmsResult<()> {
        let mut doc = self.load_document(doc_id)?;
        doc.locked = false;
        doc.locked_by = None;
        doc.locked_at = None;
        self.save_document(&doc)
    }

    /// Validate that the current user owns the lock
    fn validate_lock_ownership(&self, doc_id: &str, user_id: &str) -> QmsResult<()> {
        let lock_path = self.get_lock_file_path(doc_id);
        
        if !Path::new(&lock_path).exists() {
            return Err(QmsError::validation_error(&format!(
                "Document {doc_id} is not checked out"
            )));
        }

        let lock_content = read_file_content(&lock_path)?;
        let lock = DocumentLock::from_json(&lock_content)
            .map_err(|e| QmsError::parse_error(&format!("Failed to parse lock file: {e}")))?;

        if lock.user_id != user_id {
            return Err(QmsError::validation_error(&format!(
                "Document {} is checked out by another user: {}", doc_id, lock.user_id
            )));
        }

        Ok(())
    }

    /// Get the file path for a document lock
    fn get_lock_file_path(&self, doc_id: &str) -> String {
        format!("{}/locks/{}.json", self.project_path, doc_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use crate::modules::audit_logger::{AuditConfig, initialize_audit_system};

    fn create_temp_dir() -> std::io::Result<std::path::PathBuf> {
        // Add a small delay to prevent timestamp collisions in parallel tests
        std::thread::sleep(std::time::Duration::from_millis(50));
        let temp_dir = std::env::temp_dir().join(format!("qms_test_{}_{:?}", current_timestamp(), std::thread::current().id()));
        std::fs::create_dir_all(&temp_dir)?;
        Ok(temp_dir)
    }

    /// Initialize audit system for tests - lightweight version to avoid hanging
    fn init_audit_for_test(temp_dir: &std::path::Path) {
        // Create audit directory structure but don't initialize the full audit system
        let audit_dir = temp_dir.join("audit");
        let _ = std::fs::create_dir_all(&audit_dir);

        // Only initialize if not already done (avoid re-initialization errors)
        let config = AuditConfig {
            project_path: temp_dir.to_string_lossy().to_string(),
            retention_days: 30,
            daily_rotation: false,
            max_file_size_mb: 10,
            require_checksums: false,
        };

        // Use a timeout-safe initialization approach
        match initialize_audit_system(config) {
            Ok(_) => {},
            Err(_) => {
                // If initialization fails, continue with test - audit is not critical for checkout logic
                eprintln!("Warning: Audit system initialization failed in test, continuing without audit");
            }
        }
    }

    #[test]
    fn test_checkout_document() {
        let temp_dir = create_temp_dir().unwrap();

        // Initialize audit system for test
        init_audit_for_test(&temp_dir);
        let project_path = temp_dir.to_string_lossy().to_string();
        
        // Create a test document first with unique ID
        let doc_id = format!("550e8400-e29b-41d4-a716-44665544{:04x}", current_timestamp() % 10000);
        let doc_dir = format!("{}/documents/{}", project_path, doc_id);
        fs::create_dir_all(&doc_dir).unwrap();
        
        let doc = Document::new(
            doc_id.to_string(),
            "550e8400-e29b-41d4-a716-446655440001".to_string(),
            "Test Document".to_string(),
            "Test content".to_string(),
            crate::modules::document_control::document::DocumentType::Other("Test".to_string()),
            "1.0.0".to_string(),
            "test_user".to_string(),
            "test_doc.md".to_string(),
        ).unwrap();
        
        let doc_path = format!("{}/metadata.json", doc_dir);
        fs::write(&doc_path, doc.to_json()).unwrap();
        
        // Create content.md file that the locking system expects
        let content_path = format!("{}/content.md", doc_dir);
        fs::write(&content_path, "Test content").unwrap();
        
        // Test checkout
        let mut checkout_manager = CheckoutManager::new(project_path);
        
        // Ensure clean state by releasing any existing locks for this doc
        let _ = checkout_manager.force_release_lock(&doc_id, "test_admin", "Test cleanup");
        
        let result = checkout_manager.checkout_document(&doc_id, "test_user", Some("Testing checkout".to_string()));
        
        if let Err(ref e) = result {
            println!("Checkout failed: {:?}", e);
        }
        assert!(result.is_ok());
        let lock = result.unwrap();
        assert_eq!(lock.document_id, doc_id);
        assert_eq!(lock.user_id, "test_user");
        
        // Cleanup: checkin document and cleanup directory
        let _ = checkout_manager.checkin_document(&doc_id, "test_user", None, Some("Test cleanup"));
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_checkout_already_locked_document() {
        let temp_dir = create_temp_dir().unwrap();

        // Initialize audit system for test
        init_audit_for_test(&temp_dir);

        let project_path = temp_dir.to_string_lossy().to_string();
        
        // Create a test document that's already locked with unique ID
        let doc_id = format!("550e8400-e29b-41d4-a716-44665545{:04x}", current_timestamp() % 10000);
        let doc_dir = format!("{}/documents/{}", project_path, doc_id);
        fs::create_dir_all(&doc_dir).unwrap();
        
        let mut doc = Document::new(
            doc_id.to_string(),
            "550e8400-e29b-41d4-a716-446655440001".to_string(),
            "Test Document".to_string(),
            "Test content".to_string(),
            crate::modules::document_control::document::DocumentType::Other("Test".to_string()),
            "1.0.0".to_string(),
            "test_user".to_string(),
            "test_doc.md".to_string(),
        ).unwrap();
        
        // Mark as locked
        doc.locked = true;
        doc.locked_by = Some("other_user".to_string());
        doc.locked_at = Some(current_timestamp().to_string());
        
        let doc_path = format!("{}/metadata.json", doc_dir);
        fs::write(&doc_path, doc.to_json()).unwrap();
        
        // Create content.md file that the locking system expects
        let content_path = format!("{}/content.md", doc_dir);
        fs::write(&content_path, "Test content").unwrap();
        
        // Test checkout should fail
        let mut checkout_manager = CheckoutManager::new(project_path);
        let result = checkout_manager.checkout_document(&doc_id, "test_user", None);
        
        assert!(result.is_err());
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
