//! Document service for QMS document control CRUD operations
//! Phase 2.1.2 - Document CRUD Operations
//! Implements file-based document management with versioning

#![allow(dead_code)] // Phase 2 infrastructure - doc        audit_log_action("DOCUMENT_UPDATED", "Document", &document.id)?;    audit_log_action("DOCUMENT_CREATED", "Document", &document.id)?;t CRUD service

use crate::modules::audit_logger::audit_log_action;
use crate::error::{QmsError, QmsResult};
use crate::json_utils::{JsonSerializable, JsonValue, save_json_with_schema, load_json_with_schema};
use crate::modules::document_control::document::{Document, DocumentType};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Document service for managing document CRUD operations
pub struct DocumentService {
    project_path: PathBuf,
}

/// Document index entry for quick lookups
#[derive(Debug, Clone)]
pub struct DocumentIndexEntry {
    pub id: String,
    pub title: String,
    pub doc_type: String,
    pub version: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub file_path: String,
}

impl DocumentService {
    /// Create a new document service for the given project path
    pub fn new(project_path: PathBuf) -> Self {
        Self { project_path }
    }

    /// Add a document from an external file
    /// Phase 2.1.3 - Document Add from File
    pub fn add_document_from_file(
        &self,
        file_path: &str,
        title: &str,
        doc_type: DocumentType,
        author: String,
    ) -> QmsResult<Document> {
        // Validate file exists and is readable
        let source_path = Path::new(file_path);
        if !source_path.exists() {
            return Err(QmsError::validation_error(&format!("File not found: {}", file_path)));
        }

        // Check file size (10MB limit for medical device docs)
        const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB
        match fs::metadata(source_path) {
            Ok(metadata) => {
                if metadata.len() > MAX_FILE_SIZE {
                    return Err(QmsError::validation_error(&format!(
                        "File too large: {} bytes (max: {} bytes)", 
                        metadata.len(), 
                        MAX_FILE_SIZE
                    )));
                }
            }
            Err(e) => return Err(QmsError::from(e)),
        }

        // Detect document type from file extension if not specified explicitly
        let detected_type = self.detect_document_type_from_extension(source_path, &doc_type)?;

        // Read file content
        let content = match fs::read_to_string(source_path) {
            Ok(content) => content,
            Err(e) => {
                return Err(QmsError::validation_error(&format!(
                    "Failed to read file content: {}. Ensure file is text-based and readable.", 
                    e
                )));
            }
        };

        // Validate content (basic checks)
        self.validate_file_content(&content)?;

        // Create document using existing service
        let mut document = self.create_document(
            title.to_string(),
            content,
            detected_type,
            author,
        )?;

        // Copy source file to document directory for archival
        let doc_dir = self.project_path.join("documents").join(&document.id);
        let archive_file = doc_dir.join(format!("original{}", 
            source_path.extension()
                .and_then(|ext| ext.to_str())
                .map(|s| format!(".{}", s))
                .unwrap_or_default()
        ));

        if let Err(e) = fs::copy(source_path, &archive_file) {
            // Log warning but don't fail - document is already created
            eprintln!("Warning: Failed to archive original file: {}", e);
        }

        // Update file path to reference original file location
        document.file_path = format!("documents/{}/original{}", 
            document.id,
            source_path.extension()
                .and_then(|ext| ext.to_str())
                .map(|s| format!(".{}", s))
                .unwrap_or_default()
        );

        // Save updated document with new file path
        self.save_document_metadata(&document)?;

        log_audit(&format!(
            "DOCUMENT_ADDED_FROM_FILE: {} from {} by {}",
            document.id, file_path, author
        ));

        Ok(document)
    }

    /// Detect document type from file extension
    fn detect_document_type_from_extension(
        &self,
        file_path: &Path,
        specified_type: &DocumentType,
    ) -> QmsResult<DocumentType> {
        // If a specific type other than "Other" is provided, use it
        match specified_type {
            DocumentType::Other(_) => {
                // Auto-detect from extension
                if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
                    match extension.to_lowercase().as_str() {
                        "md" | "markdown" => Ok(DocumentType::Other("Markdown".to_string())),
                        "txt" => Ok(DocumentType::Other("Text".to_string())),
                        "json" => Ok(DocumentType::Other("JSON".to_string())),
                        "csv" => Ok(DocumentType::Other("CSV".to_string())),
                        "xml" => Ok(DocumentType::Other("XML".to_string())),
                        "html" | "htm" => Ok(DocumentType::Other("HTML".to_string())),
                        _ => Ok(DocumentType::Other("Unknown".to_string())),
                    }
                } else {
                    Ok(DocumentType::Other("Unknown".to_string()))
                }
            }
            _ => Ok(specified_type.clone()),
        }
    }

    /// Validate file content for basic quality checks
    fn validate_file_content(&self, content: &str) -> QmsResult<()> {
        // Check content is not empty
        if content.trim().is_empty() {
            return Err(QmsError::validation_error("File content is empty"));
        }

        // Check for reasonable content length (minimum 10 characters)
        if content.len() < 10 {
            return Err(QmsError::validation_error("File content too short (minimum 10 characters)"));
        }

        // Check for basic text validity (no null bytes)
        if content.contains('\0') {
            return Err(QmsError::validation_error("File contains binary data - only text files are supported"));
        }

        // Check for extremely long lines (potential binary file)
        const MAX_LINE_LENGTH: usize = 10000;
        for line in content.lines() {
            if line.len() > MAX_LINE_LENGTH {
                return Err(QmsError::validation_error("File contains extremely long lines - may be binary data"));
            }
        }

        Ok(())
    }

    /// Create a new document in the project
    pub fn create_document(
        &self,
        title: String,
        content: String,
        doc_type: DocumentType,
        created_by: String,
    ) -> QmsResult<Document> {
        // Generate UUID
        let id = crate::utils::generate_uuid();
        let project_id = crate::utils::user_context::get_current_project_id(); // Get actual project ID
        let version = "1.0.0".to_string();
        let file_path = format!("documents/{}/content.md", id);

        // Create document instance
        let document = Document::new(
            id.clone(),
            project_id,
            title,
            content,
            doc_type,
            version,
            created_by.clone(),
            file_path,
        )?;

        // Ensure documents directory structure exists
        self.ensure_documents_directory()?;
        let doc_dir = self.project_path.join("documents").join(&id);
        fs::create_dir_all(&doc_dir)?;

        // Create subdirectories
        fs::create_dir_all(doc_dir.join("versions"))?;

        // Save document files
        self.save_document_files(&document)?;

        // Update document index
        self.update_document_index(&document)?;

        // Log audit entry
        log_audit(&format!(
            "DOCUMENT_CREATED: {} '{}' by {}",
            document.id, document.title, created_by
        ));

        Ok(document)
    }

    /// Read a document by ID
    pub fn read_document(&self, document_id: &str) -> QmsResult<Document> {
        let doc_dir = self.project_path.join("documents").join(document_id);
        
        if !doc_dir.exists() {
            return Err(QmsError::not_found(&format!("Document not found: {}", document_id)));
        }

        let metadata_file = doc_dir.join("metadata.json");
        if !metadata_file.exists() {
            return Err(QmsError::not_found(&format!("Document metadata not found: {}", document_id)));
        }

        // Load document metadata
        let metadata_content = fs::read_to_string(metadata_file)?;
        let document = Document::from_json(&metadata_content)?;

        // Log audit entry
        audit_log_action("DOCUMENT_READ", "Document", document_id)?;

        Ok(document)
    }

    /// Update a document
    pub fn update_document(
        &self,
        document_id: &str,
        title: Option<String>,
        content: Option<String>,
        updated_by: String,
    ) -> QmsResult<Document> {
        // Load existing document
        let mut document = self.read_document(document_id)?;

        // Apply updates
        if let Some(new_title) = title {
            document.title = new_title;
        }

        if let Some(new_content) = content {
            document.update_content(new_content)?;
        }

        // Update timestamp
        document.updated_at = crate::utils::current_timestamp_string();

        // Save updated document
        self.save_document_files(&document)?;
        self.update_document_index(&document)?;

        // Log audit entry
        log_audit(&format!(
            "DOCUMENT_UPDATED: {} by {}",
            document_id, updated_by
        ));

        Ok(document)
    }

    /// Delete a document (move to archive)
    pub fn delete_document(&self, document_id: &str, deleted_by: String) -> QmsResult<()> {
        let doc_dir = self.project_path.join("documents").join(document_id);
        
        if !doc_dir.exists() {
            return Err(QmsError::not_found(&format!("Document not found: {}", document_id)));
        }

        // Ensure archive directory exists
        let archive_dir = self.project_path.join("documents").join("archive");
        fs::create_dir_all(&archive_dir)?;

        // Move document to archive
        let archive_doc_dir = archive_dir.join(document_id);
        fs::rename(&doc_dir, &archive_doc_dir)?;

        // Remove from index
        self.remove_from_document_index(document_id)?;

        // Log audit entry
        audit_log_action("DOCUMENT_DELETED", "Document", document_id)?;

        Ok(())
    }

    /// List all documents
    pub fn list_documents(&self) -> QmsResult<Vec<DocumentIndexEntry>> {
        let index_file = self.project_path.join("documents").join("index.json");
        
        if !index_file.exists() {
            return Ok(Vec::new());
        }

        let index_content = fs::read_to_string(index_file)?;
        let index_data = load_json_with_schema(&index_content, "1.0")?;

        let mut documents = Vec::new();
        if let JsonValue::Object(obj) = index_data {
            if let Some(JsonValue::Array(docs)) = obj.get("data") {
                for doc_value in docs {
                    if let JsonValue::Object(doc_obj) = doc_value {
                        let entry = DocumentIndexEntry {
                            id: doc_obj.get("id")
                                .and_then(|v| if let JsonValue::String(s) = v { Some(s.clone()) } else { None })
                                .unwrap_or_default(),
                            title: doc_obj.get("title")
                                .and_then(|v| if let JsonValue::String(s) = v { Some(s.clone()) } else { None })
                                .unwrap_or_default(),
                            doc_type: doc_obj.get("doc_type")
                                .and_then(|v| if let JsonValue::String(s) = v { Some(s.clone()) } else { None })
                                .unwrap_or_default(),
                            version: doc_obj.get("version")
                                .and_then(|v| if let JsonValue::String(s) = v { Some(s.clone()) } else { None })
                                .unwrap_or_default(),
                            status: doc_obj.get("status")
                                .and_then(|v| if let JsonValue::String(s) = v { Some(s.clone()) } else { None })
                                .unwrap_or_default(),
                            created_at: doc_obj.get("created_at")
                                .and_then(|v| if let JsonValue::String(s) = v { Some(s.clone()) } else { None })
                                .unwrap_or_default(),
                            updated_at: doc_obj.get("updated_at")
                                .and_then(|v| if let JsonValue::String(s) = v { Some(s.clone()) } else { None })
                                .unwrap_or_default(),
                            file_path: doc_obj.get("file_path")
                                .and_then(|v| if let JsonValue::String(s) = v { Some(s.clone()) } else { None })
                                .unwrap_or_default(),
                        };
                        documents.push(entry);
                    }
                }
            }
        }

        Ok(documents)
    }

    /// Search documents by title or content
    pub fn search_documents(&self, query: &str) -> QmsResult<Vec<DocumentIndexEntry>> {
        let all_documents = self.list_documents()?;
        let query_lower = query.to_lowercase();

        let filtered: Vec<DocumentIndexEntry> = all_documents
            .into_iter()
            .filter(|doc| {
                doc.title.to_lowercase().contains(&query_lower)
            })
            .collect();

        Ok(filtered)
    }

    /// Save document files to disk
    fn save_document_files(&self, document: &Document) -> QmsResult<()> {
        let doc_dir = self.project_path.join("documents").join(&document.id);

        // Save metadata
        self.save_document_metadata(document)?;

        // Save content
        let content_file = doc_dir.join("content.md");
        fs::write(content_file, &document.content)?;

        // Save version snapshot
        let version_file = doc_dir.join("versions").join(format!("{}.json", document.version));
        let version_data = document.to_json();
        save_json_with_schema(&version_data, &version_file, "1.0")?;

        Ok(())
    }

    /// Save document metadata
    fn save_document_metadata(&self, document: &Document) -> QmsResult<()> {
        let doc_dir = self.project_path.join("documents").join(&document.id);
        let metadata_file = doc_dir.join("metadata.json");
        let metadata = document.to_json();
        save_json_with_schema(&metadata, &metadata_file, "1.0")?;
        Ok(())
    }

    /// Update the document index
    fn update_document_index(&self, document: &Document) -> QmsResult<()> {
        let index_file = self.project_path.join("documents").join("index.json");
        
        // Load existing index or create new
        let mut documents = Vec::new();
        if index_file.exists() {
            let existing_docs = self.list_documents()?;
            documents = existing_docs;
        }

        // Remove existing entry for this document (if updating)
        documents.retain(|d| d.id != document.id);

        // Add current document
        let entry = DocumentIndexEntry {
            id: document.id.clone(),
            title: document.title.clone(),
            doc_type: format!("{:?}", document.doc_type),
            version: document.version.clone(),
            status: format!("{:?}", document.status),
            created_at: document.created_at.clone(),
            updated_at: document.updated_at.clone(),
            file_path: document.file_path.clone(),
        };
        documents.push(entry);

        // Build index JSON
        let mut index_data = HashMap::new();
        let mut doc_objects = Vec::new();

        for doc in documents {
            let mut doc_obj = HashMap::new();
            doc_obj.insert("id".to_string(), JsonValue::String(doc.id));
            doc_obj.insert("title".to_string(), JsonValue::String(doc.title));
            doc_obj.insert("doc_type".to_string(), JsonValue::String(doc.doc_type));
            doc_obj.insert("version".to_string(), JsonValue::String(doc.version));
            doc_obj.insert("status".to_string(), JsonValue::String(doc.status));
            doc_obj.insert("created_at".to_string(), JsonValue::String(doc.created_at));
            doc_obj.insert("updated_at".to_string(), JsonValue::String(doc.updated_at));
            doc_obj.insert("file_path".to_string(), JsonValue::String(doc.file_path));
            doc_objects.push(JsonValue::Object(doc_obj));
        }

        index_data.insert("data".to_string(), JsonValue::Array(doc_objects));
        let index_json = JsonValue::Object(index_data);

        save_json_with_schema(&index_json.to_json(), &index_file, "1.0")?;
        Ok(())
    }

    /// Remove document from index
    fn remove_from_document_index(&self, document_id: &str) -> QmsResult<()> {
        let index_file = self.project_path.join("documents").join("index.json");
        
        if !index_file.exists() {
            return Ok(());
        }

        let mut documents = self.list_documents()?;
        documents.retain(|d| d.id != document_id);

        // Rebuild index
        let mut index_data = HashMap::new();
        let mut doc_objects = Vec::new();

        for doc in documents {
            let mut doc_obj = HashMap::new();
            doc_obj.insert("id".to_string(), JsonValue::String(doc.id));
            doc_obj.insert("title".to_string(), JsonValue::String(doc.title));
            doc_obj.insert("doc_type".to_string(), JsonValue::String(doc.doc_type));
            doc_obj.insert("version".to_string(), JsonValue::String(doc.version));
            doc_obj.insert("status".to_string(), JsonValue::String(doc.status));
            doc_obj.insert("created_at".to_string(), JsonValue::String(doc.created_at));
            doc_obj.insert("updated_at".to_string(), JsonValue::String(doc.updated_at));
            doc_obj.insert("file_path".to_string(), JsonValue::String(doc.file_path));
            doc_objects.push(JsonValue::Object(doc_obj));
        }

        index_data.insert("data".to_string(), JsonValue::Array(doc_objects));
        let index_json = JsonValue::Object(index_data);

        save_json_with_schema(&index_json.to_json(), &index_file, "1.0")?;
        Ok(())
    }

    /// Ensure documents directory exists
    fn ensure_documents_directory(&self) -> QmsResult<()> {
        let docs_dir = self.project_path.join("documents");
        if !docs_dir.exists() {
            fs::create_dir_all(&docs_dir)?;
        }

        // Create index file if it doesn't exist
        let index_file = docs_dir.join("index.json");
        if !index_file.exists() {
            let empty_index = r#"{"version": "1.0", "data": []}"#;
            fs::write(index_file, empty_index)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::document_control::document::DocumentStatus;
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    fn create_test_project() -> PathBuf {
        let mut temp_dir = env::temp_dir();
        temp_dir.push(format!("qms_test_{}", crate::utils::generate_uuid()));
        fs::create_dir_all(&temp_dir).unwrap();
        temp_dir
    }

    fn cleanup_test_project(path: &PathBuf) {
        if path.exists() {
            fs::remove_dir_all(path).unwrap_or_else(|e| {
                eprintln!("Warning: Failed to cleanup test directory: {}", e);
            });
        }
    }

    #[test]
    fn test_create_document() {
        let project_path = create_test_project();
        let service = DocumentService::new(project_path.clone());

        let result = service.create_document(
            "Test Document".to_string(),
            "This is test content".to_string(),
            DocumentType::SoftwareRequirementsSpecification,
            "test_user".to_string(),
        );

        assert!(result.is_ok(), "Failed to create document: {:?}", result.err());
        let document = result.unwrap();
        assert_eq!(document.title, "Test Document");
        assert_eq!(document.content, "This is test content");
        assert_eq!(document.version, "1.0.0");
        assert_eq!(document.status, DocumentStatus::Draft);

        // Verify files were created
        let doc_dir = project_path.join("documents").join(&document.id);
        assert!(doc_dir.exists());
        assert!(doc_dir.join("metadata.json").exists());
        assert!(doc_dir.join("content.md").exists());
        assert!(doc_dir.join("versions").join("1.0.0.json").exists());

        cleanup_test_project(&project_path);
    }

    #[test]
    fn test_read_document() {
        let project_path = create_test_project();
        let service = DocumentService::new(project_path.clone());

        // Create a document first
        let created_doc = service.create_document(
            "Read Test Document".to_string(),
            "Content for reading".to_string(),
            DocumentType::TestProtocol,
            "test_user".to_string(),
        ).unwrap();

        // Read the document back
        let result = service.read_document(&created_doc.id);
        assert!(result.is_ok());
        
        let read_doc = result.unwrap();
        assert_eq!(read_doc.id, created_doc.id);
        assert_eq!(read_doc.title, "Read Test Document");
        assert_eq!(read_doc.content, "Content for reading");

        cleanup_test_project(&project_path);
    }

    #[test]
    fn test_update_document() {
        let project_path = create_test_project();
        let service = DocumentService::new(project_path.clone());

        // Create a document first
        let created_doc = service.create_document(
            "Update Test Document".to_string(),
            "Original content".to_string(),
            DocumentType::UserRequirements,
            "test_user".to_string(),
        ).unwrap();

        // Add a small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_secs(1));

        // Update the document
        let result = service.update_document(
            &created_doc.id,
            Some("Updated Title".to_string()),
            Some("Updated content".to_string()),
            "test_user".to_string(),
        );

        assert!(result.is_ok());
        let updated_doc = result.unwrap();
        assert_eq!(updated_doc.title, "Updated Title");
        assert_eq!(updated_doc.content, "Updated content");
        assert_ne!(updated_doc.updated_at, created_doc.updated_at, "Updated timestamp should be different");

        cleanup_test_project(&project_path);
    }

    #[test]
    fn test_delete_document() {
        let project_path = create_test_project();
        let service = DocumentService::new(project_path.clone());

        // Create a document first
        let created_doc = service.create_document(
            "Delete Test Document".to_string(),
            "Content to be deleted".to_string(),
            DocumentType::DesignHistoryFile,
            "test_user".to_string(),
        ).unwrap();

        // Verify document exists
        let doc_dir = project_path.join("documents").join(&created_doc.id);
        assert!(doc_dir.exists());

        // Delete the document
        let result = service.delete_document(&created_doc.id, "test_user".to_string());
        assert!(result.is_ok());

        // Verify document was moved to archive
        assert!(!doc_dir.exists());
        let archive_dir = project_path.join("documents").join("archive").join(&created_doc.id);
        assert!(archive_dir.exists());

        // Verify reading the document now fails
        let read_result = service.read_document(&created_doc.id);
        assert!(read_result.is_err());

        cleanup_test_project(&project_path);
    }

    #[test]
    fn test_list_documents() {
        let project_path = create_test_project();
        let service = DocumentService::new(project_path.clone());

        // Initially no documents
        let result = service.list_documents();
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());

        // Create some documents
        let _doc1 = service.create_document(
            "Document 1".to_string(),
            "Content 1".to_string(),
            DocumentType::SoftwareRequirementsSpecification,
            "user1".to_string(),
        ).unwrap();

        let _doc2 = service.create_document(
            "Document 2".to_string(),
            "Content 2".to_string(),
            DocumentType::TestReport,
            "user2".to_string(),
        ).unwrap();

        // List documents
        let result = service.list_documents();
        assert!(result.is_ok(), "Failed to list documents: {:?}", result.err());
        let documents = result.unwrap();
        assert_eq!(documents.len(), 2);

        cleanup_test_project(&project_path);
    }

    #[test]
    fn test_search_documents() {
        let project_path = create_test_project();
        let service = DocumentService::new(project_path.clone());

        // Create test documents
        let _doc1 = service.create_document(
            "Requirements Document".to_string(),
            "System requirements".to_string(),
            DocumentType::SoftwareRequirementsSpecification,
            "user1".to_string(),
        ).unwrap();

        let _doc2 = service.create_document(
            "Test Protocol".to_string(),
            "Testing procedures".to_string(),
            DocumentType::TestProtocol,
            "user2".to_string(),
        ).unwrap();

        // Search for "Requirements"
        let result = service.search_documents("Requirements");
        assert!(result.is_ok());
        let found_docs = result.unwrap();
        assert_eq!(found_docs.len(), 1);
        assert_eq!(found_docs[0].title, "Requirements Document");

        // Search for "Test"
        let result = service.search_documents("Test");
        assert!(result.is_ok());
        let found_docs = result.unwrap();
        assert_eq!(found_docs.len(), 1);
        assert_eq!(found_docs[0].title, "Test Protocol");

        cleanup_test_project(&project_path);
    }

    #[test]
    fn test_add_document_from_file() {
        let project_path = create_test_project();
        let service = DocumentService::new(project_path.clone());

        // Create a test file
        let test_file = project_path.join("test_doc.md");
        let test_content = "# Test Document\n\nThis is a test document with markdown content.\n\n## Section 1\n\nSome content here.";
        fs::write(&test_file, test_content).unwrap();

        // Test adding document from file
        let doc = service.add_document_from_file(
            test_file.to_str().unwrap(),
            "Test Markdown Document",
            DocumentType::Other("auto".to_string()),
            "test_user".to_string(),
        ).unwrap();

        assert_eq!(doc.title, "Test Markdown Document");
        assert_eq!(doc.content, test_content);
        assert!(matches!(doc.doc_type, DocumentType::Other(ref s) if s == "Markdown"));
        assert_eq!(doc.created_by, "test_user");

        // Verify document files were created
        let doc_dir = project_path.join("documents").join(&doc.id);
        assert!(doc_dir.exists());
        assert!(doc_dir.join("metadata.json").exists());
        assert!(doc_dir.join("content.md").exists());
        assert!(doc_dir.join("original.md").exists()); // Archive copy

        // Verify content
        let saved_content = fs::read_to_string(doc_dir.join("content.md")).unwrap();
        assert_eq!(saved_content, test_content);

        cleanup_test_project(&project_path);
    }

    #[test]
    fn test_add_document_from_file_not_found() {
        let project_path = create_test_project();
        let service = DocumentService::new(project_path.clone());

        let result = service.add_document_from_file(
            "nonexistent.txt",
            "Test",
            DocumentType::Other("text".to_string()),
            "user".to_string(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("File not found"));

        cleanup_test_project(&project_path);
    }

    #[test]
    fn test_add_document_from_file_empty() {
        let project_path = create_test_project();
        let service = DocumentService::new(project_path.clone());

        // Create empty file
        let test_file = project_path.join("empty.txt");
        fs::write(&test_file, "").unwrap();

        let result = service.add_document_from_file(
            test_file.to_str().unwrap(),
            "Empty File",
            DocumentType::Other("text".to_string()),
            "user".to_string(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));

        cleanup_test_project(&project_path);
    }

    #[test]
    fn test_detect_document_type_from_extension() {
        let project_path = create_test_project();
        let service = DocumentService::new(project_path.clone());

        // Test auto-detection
        let md_path = Path::new("test.md");
        let detected = service.detect_document_type_from_extension(
            md_path, 
            &DocumentType::Other("auto".to_string())
        ).unwrap();
        assert!(matches!(detected, DocumentType::Other(ref s) if s == "Markdown"));

        let txt_path = Path::new("test.txt");
        let detected = service.detect_document_type_from_extension(
            txt_path, 
            &DocumentType::Other("auto".to_string())
        ).unwrap();
        assert!(matches!(detected, DocumentType::Other(ref s) if s == "Text"));

        // Test explicit type takes precedence
        let detected = service.detect_document_type_from_extension(
            md_path, 
            &DocumentType::SoftwareRequirementsSpecification
        ).unwrap();
        assert!(matches!(detected, DocumentType::SoftwareRequirementsSpecification));

        cleanup_test_project(&project_path);
    }

    #[test]
    fn test_validate_file_content() {
        let project_path = create_test_project();
        let service = DocumentService::new(project_path.clone());

        // Valid content
        assert!(service.validate_file_content("This is valid content with enough characters.").is_ok());

        // Empty content
        assert!(service.validate_file_content("").is_err());
        assert!(service.validate_file_content("   \n\t  ").is_err());

        // Too short
        assert!(service.validate_file_content("short").is_err());

        // Contains null bytes
        let null_content = format!("valid content{}\0more content", " ".repeat(50));
        assert!(service.validate_file_content(&null_content).is_err());

        // Extremely long line
        let long_line = "x".repeat(15000);
        assert!(service.validate_file_content(&long_line).is_err());

        cleanup_test_project(&project_path);
    }

    #[test]
    fn test_add_document_from_various_file_types() {
        let project_path = create_test_project();
        let service = DocumentService::new(project_path.clone());

        // Test JSON file
        let json_file = project_path.join("test.json");
        let json_content = r#"{"name": "test", "version": "1.0", "description": "Test JSON document"}"#;
        fs::write(&json_file, json_content).unwrap();

        let doc = service.add_document_from_file(
            json_file.to_str().unwrap(),
            "Test JSON",
            DocumentType::Other("auto".to_string()),
            "user".to_string(),
        ).unwrap();
        assert!(matches!(doc.doc_type, DocumentType::Other(ref s) if s == "JSON"));

        // Test CSV file
        let csv_file = project_path.join("test.csv");
        let csv_content = "name,version,type\nDocument1,1.0,SRS\nDocument2,2.0,SDD\n";
        fs::write(&csv_file, csv_content).unwrap();

        let doc = service.add_document_from_file(
            csv_file.to_str().unwrap(),
            "Test CSV",
            DocumentType::Other("auto".to_string()),
            "user".to_string(),
        ).unwrap();
        assert!(matches!(doc.doc_type, DocumentType::Other(ref s) if s == "CSV"));

        cleanup_test_project(&project_path);
    }
}
