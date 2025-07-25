//! Document service for QMS document control CRUD operations
//! Phase 2.1.2 - Document CRUD Operations
//! Implements file-based document management with versioning

#![allow(dead_code)] // Phase 2 infrastructure - document CRUD service

use crate::modules::audit_logger::{audit_log_create, audit_log_read, audit_log_update, audit_log_delete, audit_log_action};
use crate::error::{QmsError, QmsResult};
use crate::json_utils::{JsonSerializable, JsonValue};
use crate::modules::document_control::document::{Document, DocumentType};
use crate::modules::document_control::version::{DocumentVersionControl, VersionChangeType, DocumentVersion};
use crate::modules::document_control::template::{TemplateManager, TemplateContext};
use crate::modules::document_control::backup::DocumentBackupManager;
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
    pub author: String,
}

impl DocumentService {
    /// Create a new document service for the given project path
    pub const fn new(project_path: PathBuf) -> Self {
        Self { project_path }
    }

    /// Create automatic backup for a document (Phase 2.1.14)
    fn create_document_backup(
        &self,
        document: &Document,
        created_by: &str,
        reason: &str,
    ) -> QmsResult<()> {
        let backup_manager = DocumentBackupManager::new(self.project_path.clone());
        let document_path = self.project_path.join("documents").join(&document.id).join("content.md");
        
        match backup_manager.create_backup(
            &document.id,
            &document.version,
            &document_path,
            &document.content,
            created_by,
            reason,
        ) {
            Ok(metadata) => {
                audit_log_action("AUTOMATIC_BACKUP_CREATED", "DocumentBackup", &metadata.backup_id)?;
                Ok(())
            }
            Err(_e) => {
                // Log warning but don't fail the main operation
                audit_log_action("BACKUP_WARNING", "DocumentBackup", &document.id)?;
                // Return Ok to not interrupt document operations
                Ok(())
            }
        }
    }

    /// Initialize templates for the project (Phase 2.1.9)
    pub fn initialize_templates(&self) -> QmsResult<()> {
        let template_manager = TemplateManager::new(self.project_path.clone());
        template_manager.initialize_templates()
    }

    /// List available document templates (Phase 2.1.9)
    pub fn list_templates(&self) -> QmsResult<Vec<super::template::DocumentTemplate>> {
        let template_manager = TemplateManager::new(self.project_path.clone());
        template_manager.list_templates()
    }

    /// Create document from template (Phase 2.1.9)
    pub fn create_document_from_template(
        &self,
        template_name: &str,
        title: String,
        project_name: String,
        created_by: String,
        custom_variables: Option<HashMap<String, String>>,
    ) -> QmsResult<Document> {
        let template_manager = TemplateManager::new(self.project_path.clone());
        
        // Create template context
        let mut context = TemplateContext::new(project_name, created_by.clone());
        
        // Add custom variables if provided
        if let Some(vars) = custom_variables {
            for (key, value) in vars {
                context.add_variable(key, value);
            }
        }

        // Get template content and type
        let (content, doc_type) = template_manager.create_document_from_template(
            template_name,
            title.clone(),
            context,
        )?;

        // Create document using existing service
        let document = self.create_document(title, content, doc_type, created_by.clone())?;

        // Log template usage
        audit_log_action("DOCUMENT_CREATED_FROM_TEMPLATE", "Document", &document.id)?;

        Ok(document)
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
            return Err(QmsError::validation_error(&format!("File not found: {file_path}")));
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
                    "Failed to read file content: {e}. Ensure file is text-based and readable."
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
            author.clone(),
        )?;

        // Copy source file to document directory for archival
        let doc_dir = self.project_path.join("documents").join(&document.id);
        let archive_file = doc_dir.join(format!("original{}", 
            source_path.extension()
                .and_then(|ext| ext.to_str())
                .map(|s| format!(".{s}"))
                .unwrap_or_default()
        ));

        if let Err(e) = fs::copy(source_path, &archive_file) {
            // Log warning but don't fail - document is already created
            eprintln!("Warning: Failed to archive original file: {e}");
        }

        // Update file path to reference original file location
        document.file_path = format!("documents/{}/original{}", 
            document.id,
            source_path.extension()
                .and_then(|ext| ext.to_str())
                .map(|s| format!(".{s}"))
                .unwrap_or_default()
        );

        // Save updated document with new file path
        self.save_document_metadata(&document)?;

        audit_log_action("DOCUMENT_ADDED_FROM_FILE", "Document", &document.id)?;

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
        let project_id = crate::utils::generate_uuid(); // Generate proper project UUID instead of "default"
        let version = "1.0.0".to_string();
        let file_path = format!("documents/{id}/content.md");

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

        // Create automatic backup (Phase 2.1.14)
        self.create_document_backup(&document, &created_by, "Document creation")?;

        // Log audit entry
        audit_log_create("Document", &document.id, &document.to_json())?;

        Ok(document)
    }

    /// Read a document by ID
    pub fn read_document(&self, document_id: &str) -> QmsResult<Document> {
        let doc_dir = self.project_path.join("documents").join(document_id);
        
        if !doc_dir.exists() {
            return Err(QmsError::not_found(&format!("Document not found: {document_id}")));
        }

        let metadata_file = doc_dir.join("metadata.json");
        if !metadata_file.exists() {
            return Err(QmsError::not_found(&format!("Document metadata not found: {document_id}")));
        }

        // Load document metadata
        let metadata_content = fs::read_to_string(metadata_file)?;
        let document = Document::from_json(&metadata_content)?;

        // Log audit entry
        audit_log_read("Document", document_id)?;

        Ok(document)
    }

    /// Update a document with version control
    pub fn update_document(
        &self,
        document_id: &str,
        title: Option<String>,
        content: Option<String>,
        change_type: Option<VersionChangeType>,
        change_description: Option<String>,
        updated_by: String,
    ) -> QmsResult<Document> {
        // Load existing document
        let mut document = self.read_document(document_id)?;
        let original_content = document.content.clone();
        let original_title = document.title.clone();
        let original_version = document.version.clone();

        // Determine change type if not provided
        let determined_change_type = if let Some(ct) = change_type {
            ct
        } else {
            // Auto-determine change type based on what changed
            if title.is_some() && title.as_ref() != Some(&original_title) {
                VersionChangeType::Minor // Title changes are minor
            } else if content.is_some() && content.as_ref() != Some(&original_content) {
                VersionChangeType::Patch // Content changes are patch
            } else {
                VersionChangeType::Patch // Default to patch for any other updates
            }
        };

        // Apply updates
        if let Some(new_title) = title {
            document.title = new_title;
        }

        if let Some(new_content) = content {
            document.update_content(new_content)?;
        }

        // Increment version based on change type
        document.version = DocumentVersionControl::increment_version(&document.version, determined_change_type.clone())?;

        // Update timestamp
        document.updated_at = crate::utils::current_timestamp_string();

        // Create version snapshot before saving
        let description = change_description.unwrap_or_else(|| {
            match determined_change_type {
                VersionChangeType::Major => "Major document update".to_string(),
                VersionChangeType::Minor => "Minor document update".to_string(),
                VersionChangeType::Patch => "Document content update".to_string(),
            }
        });

        DocumentVersionControl::create_version_snapshot(
            &self.project_path,
            &document,
            determined_change_type,
            &description,
            &updated_by,
        )?;

        // Save updated document
        self.save_document_files(&document)?;
        self.update_document_index(&document)?;

        // Create automatic backup (Phase 2.1.14)
        self.create_document_backup(&document, &updated_by, &format!("Document update: {description}"))?;

        // Log audit entry
        let old_value = format!("version:{} title:{} content_length:{}", 
                                &original_version, &original_title, original_content.len());
        let new_value = format!("version:{} title:{} content_length:{}", 
                                &document.version, &document.title, document.content.len());
        audit_log_update("Document", document_id, &old_value, &new_value)?;

        Ok(document)
    }

    /// Delete a document (move to archive)
    pub fn delete_document(&self, document_id: &str, _deleted_by: String) -> QmsResult<()> {
        let doc_dir = self.project_path.join("documents").join(document_id);
        
        if !doc_dir.exists() {
            return Err(QmsError::not_found(&format!("Document not found: {document_id}")));
        }

        // Read document data before deletion for audit trail
        let document = self.read_document(document_id)?;
        let document_data = document.to_json();

        // Ensure archive directory exists
        let archive_dir = self.project_path.join("documents").join("archive");
        fs::create_dir_all(&archive_dir)?;

        // Move document to archive
        let archive_doc_dir = archive_dir.join(document_id);
        fs::rename(&doc_dir, &archive_doc_dir)?;

        // Remove from index
        self.remove_from_document_index(document_id)?;

        // Log audit entry
        audit_log_delete("Document", document_id, &document_data)?;

        Ok(())
    }

    /// List all documents
    pub fn list_documents(&self) -> QmsResult<Vec<DocumentIndexEntry>> {
        let index_file = self.project_path.join("documents").join("index.json");
        
        if !index_file.exists() {
            return Ok(Vec::new());
        }

        let index_content = fs::read_to_string(index_file)?;
        let index_data = crate::json_utils::JsonValue::parse(&index_content)?;

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
                            author: doc_obj.get("created_by")
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

    /// Advanced search with filtering capabilities
    pub fn search_documents_advanced(&self, query: &str, filters: &crate::commands::doc::SearchFilters) -> QmsResult<Vec<DocumentIndexEntry>> {
        let all_documents = self.list_documents()?;
        let query_lower = query.to_lowercase();

        let filtered: Vec<DocumentIndexEntry> = all_documents
            .into_iter()
            .filter(|doc| {
                // Text search in title and content
                let title_match = doc.title.to_lowercase().contains(&query_lower);
                let content_match = self.search_document_content(&doc.id, &query_lower).unwrap_or(false);
                let text_match = title_match || content_match;
                
                if !text_match {
                    return false;
                }
                
                // Apply filters
                if let Some(ref doc_type) = filters.document_type {
                    let matches_type = match doc_type.to_lowercase().as_str() {
                        "srs" => doc.doc_type.to_lowercase().contains("softwarerequirementsspecification"),
                        "sdd" => doc.doc_type.to_lowercase().contains("softwaredesigndescription"),
                        "tp" => doc.doc_type.to_lowercase().contains("testprotocol"),
                        "rmf" => doc.doc_type.to_lowercase().contains("riskmanagementfile"),
                        "ur" => doc.doc_type.to_lowercase().contains("userrequirements"),
                        other => doc.doc_type.to_lowercase().contains(&other.to_lowercase()),
                    };
                    if !matches_type {
                        return false;
                    }
                }
                
                if let Some(ref status) = filters.status {
                    let matches_status = match status.to_lowercase().as_str() {
                        "draft" => doc.status.to_lowercase() == "draft",
                        "inreview" => doc.status.to_lowercase() == "inreview",
                        "approved" => doc.status.to_lowercase() == "approved",
                        "archived" => doc.status.to_lowercase() == "archived",
                        _ => false,
                    };
                    if !matches_status {
                        return false;
                    }
                }
                
                if let Some(ref author) = filters.author {
                    if !doc.author.to_lowercase().contains(&author.to_lowercase()) {
                        return false;
                    }
                }
                
                true
            })
            .collect();

        Ok(filtered)
    }

    /// Search within document content
    fn search_document_content(&self, doc_id: &str, query: &str) -> QmsResult<bool> {
        let doc_dir = self.project_path.join("documents").join(doc_id);
        let content_file = doc_dir.join("content.md");
        
        if !content_file.exists() {
            return Ok(false);
        }
        
        match fs::read_to_string(&content_file) {
            Ok(content) => Ok(content.to_lowercase().contains(query)),
            Err(_) => Ok(false), // Ignore read errors for search
        }
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
        fs::write(version_file, version_data)?;

        Ok(())
    }

    /// Save document metadata
    fn save_document_metadata(&self, document: &Document) -> QmsResult<()> {
        let doc_dir = self.project_path.join("documents").join(&document.id);
        let metadata_file = doc_dir.join("metadata.json");
        let metadata = document.to_json();
        fs::write(metadata_file, metadata)?;
        Ok(())
    }

    /// Update the document index
    fn update_document_index(&self, document: &Document) -> QmsResult<()> {
        let index_file = self.project_path.join("documents").join("index.json");
        
        // Load existing index or create new
        let mut documents = Vec::new();
        if index_file.exists() {
            let index_content = fs::read_to_string(&index_file)?;
            let index_data = crate::json_utils::JsonValue::parse(&index_content)?;

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
                                author: doc_obj.get("created_by")
                                    .and_then(|v| if let JsonValue::String(s) = v { Some(s.clone()) } else { None })
                                    .unwrap_or_default(),
                            };
                            documents.push(entry);
                        }
                    }
                }
            }
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
            author: document.created_by.clone(),
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

        // Save without schema wrapper for index file (it has its own structure)
        let json_string = index_json.json_to_string();
        fs::write(&index_file, json_string)?;
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

        // Save without schema wrapper for index file (it has its own structure)
        let json_string = index_json.json_to_string();
        fs::write(&index_file, json_string)?;
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

    // ========== VERSION CONTROL METHODS ==========

    /// Get version history for a document
    pub fn get_document_version_history(&self, document_id: &str) -> QmsResult<Vec<DocumentVersion>> {
        DocumentVersionControl::get_version_history(&self.project_path, document_id)
    }

    /// Get specific version of a document
    pub fn get_document_version(&self, document_id: &str, version: &str) -> QmsResult<Document> {
        DocumentVersionControl::get_document_version(&self.project_path, document_id, version)
    }

    /// List all available versions for a document
    pub fn list_document_versions(&self, document_id: &str) -> QmsResult<Vec<String>> {
        DocumentVersionControl::list_document_versions(&self.project_path, document_id)
    }

    /// Get latest version number for a document
    pub fn get_latest_document_version(&self, document_id: &str) -> QmsResult<Option<String>> {
        DocumentVersionControl::get_latest_version(&self.project_path, document_id)
    }

    /// Compare two versions of a document
    pub fn compare_document_versions(&self, document_id: &str, version_a: &str, version_b: &str) -> QmsResult<(Document, Document)> {
        let doc_a = self.get_document_version(document_id, version_a)?;
        let doc_b = self.get_document_version(document_id, version_b)?;
        Ok((doc_a, doc_b))
    }

    /// Create a new version of a document with explicit version increment
    pub fn create_document_version(
        &self,
        document_id: &str,
        change_type: VersionChangeType,
        change_description: &str,
        updated_by: &str,
    ) -> QmsResult<Document> {
        // Load current document
        let mut document = self.read_document(document_id)?;
        
        // Increment version
        document.version = DocumentVersionControl::increment_version(&document.version, change_type.clone())?;
        document.updated_at = crate::utils::current_timestamp_string();

        // Create version snapshot
        DocumentVersionControl::create_version_snapshot(
            &self.project_path,
            &document,
            change_type,
            change_description,
            updated_by,
        )?;

        // Save updated document
        self.save_document_files(&document)?;
        self.update_document_index(&document)?;

        // Log audit entry
        audit_log_action("DOCUMENT_VERSION_CREATED", "Document", &document.id)?;

        Ok(document)
    }

    /// Rollback document to a previous version
    pub fn rollback_document_to_version(
        &self,
        document_id: &str,
        target_version: &str,
        rollback_reason: &str,
        updated_by: &str,
    ) -> QmsResult<Document> {
        // Get the target version
        let target_document = self.get_document_version(document_id, target_version)?;
        
        // Load current document to get the current version for incrementing
        let current_document = self.read_document(document_id)?;
        
        // Create a new document with rollback content but new version number
        let mut rollback_document = target_document.clone();
        rollback_document.version = DocumentVersionControl::increment_version(
            &current_document.version, 
            VersionChangeType::Major // Rollbacks are major changes
        )?;
        rollback_document.updated_at = crate::utils::current_timestamp_string();

        // Create version snapshot for the rollback
        let change_description = format!("Rollback to version {target_version}: {rollback_reason}");
        DocumentVersionControl::create_version_snapshot(
            &self.project_path,
            &rollback_document,
            VersionChangeType::Major,
            &change_description,
            updated_by,
        )?;

        // Save the rolled-back document
        self.save_document_files(&rollback_document)?;
        self.update_document_index(&rollback_document)?;

        // Log audit entry
        audit_log_action("DOCUMENT_ROLLBACK", "Document", document_id)?;

        Ok(rollback_document)
    }

    /// Get version statistics for a document
    pub fn get_document_version_stats(&self, document_id: &str) -> QmsResult<HashMap<String, String>> {
        let versions = self.list_document_versions(document_id)?;
        let history = self.get_document_version_history(document_id)?;
        
        let mut stats = HashMap::new();
        
        stats.insert("total_versions".to_string(), versions.len().to_string());
        
        if let Some(latest) = versions.last() {
            stats.insert("latest_version".to_string(), latest.clone());
        }
        
        if let Some(oldest) = versions.first() {
            stats.insert("oldest_version".to_string(), oldest.clone());
        }

        // Count by change type
        let mut major_count = 0;
        let mut minor_count = 0;
        let mut patch_count = 0;
        
        for version in &history {
            match version.change_type {
                VersionChangeType::Major => major_count += 1,
                VersionChangeType::Minor => minor_count += 1,
                VersionChangeType::Patch => patch_count += 1,
            }
        }
        
        stats.insert("major_versions".to_string(), major_count.to_string());
        stats.insert("minor_versions".to_string(), minor_count.to_string());
        stats.insert("patch_versions".to_string(), patch_count.to_string());
        
        Ok(stats)
    }

    // Document Checkout/Checkin Methods

    /// Checkout a document for editing (Phase 2.1.5)
    pub fn checkout_document(&self, doc_id: &str, user_id: &str, reason: Option<String>) -> QmsResult<super::document::DocumentLock> {
        let mut checkout_manager = super::checkout::CheckoutManager::new(self.project_path.to_string_lossy().to_string());
        checkout_manager.checkout_document(doc_id, user_id, reason)
    }

    /// Checkin a document with optional new content (Phase 2.1.5)
    pub fn checkin_document(
        &self, 
        doc_id: &str, 
        user_id: &str, 
        new_content_path: Option<&str>,
        message: Option<&str>
    ) -> QmsResult<Document> {
        let mut checkout_manager = super::checkout::CheckoutManager::new(self.project_path.to_string_lossy().to_string());
        checkout_manager.checkin_document(doc_id, user_id, new_content_path, message)
    }

    /// Get checkout status for a document (Phase 2.1.5)
    pub fn get_checkout_status(&self, doc_id: &str) -> QmsResult<Option<super::document::DocumentLock>> {
        let checkout_manager = super::checkout::CheckoutManager::new(self.project_path.to_string_lossy().to_string());
        checkout_manager.get_checkout_status(doc_id)
    }

    /// Force release a document lock (admin operation) (Phase 2.1.5)
    pub fn force_release_lock(&self, doc_id: &str, admin_user: &str, reason: &str) -> QmsResult<()> {
        let mut checkout_manager = super::checkout::CheckoutManager::new(self.project_path.to_string_lossy().to_string());
        checkout_manager.force_release_lock(doc_id, admin_user, reason)
    }

    /// List all currently locked documents (Phase 2.1.5)
    pub fn list_locked_documents(&self) -> QmsResult<Vec<super::document::DocumentLock>> {
        let checkout_manager = super::checkout::CheckoutManager::new(self.project_path.to_string_lossy().to_string());
        checkout_manager.list_locked_documents()
    }

    /// Get detailed lock information (admin utility)
    pub fn get_detailed_lock_info(&self, doc_id: &str) -> QmsResult<Option<super::checkout::DetailedLockInfo>> {
        let checkout_manager = super::checkout::CheckoutManager::new(self.project_path.to_string_lossy().to_string());
        checkout_manager.get_detailed_lock_info(doc_id)
    }

    /// Clean up stale locks (admin utility)
    pub fn cleanup_stale_locks(&self, admin_user: &str) -> QmsResult<super::checkout::CleanupReport> {
        let mut checkout_manager = super::checkout::CheckoutManager::new(self.project_path.to_string_lossy().to_string());
        checkout_manager.cleanup_stale_locks(admin_user)
    }

    /// Get lock statistics (admin utility)
    pub fn get_lock_statistics(&self) -> QmsResult<super::checkout::LockStatistics> {
        let checkout_manager = super::checkout::CheckoutManager::new(self.project_path.to_string_lossy().to_string());
        checkout_manager.get_lock_statistics()
    }

    // === Phase 2.1.6 Document Status Management ===

    /// Submit a document for review (Draft → InReview)
    pub fn submit_for_review(&self, doc_id: &str, _user_id: &str) -> QmsResult<Document> {
        use super::document::DocumentStatus;
        
        let mut document = self.read_document(doc_id)?;
        
        // Validate current status allows submission
        if document.status != DocumentStatus::Draft {
            return Err(QmsError::validation_error(&format!(
                "Document can only be submitted from Draft status, current status: {:?}",
                document.status
            )));
        }

        // Update status to InReview
        document.update_status(DocumentStatus::InReview, None)?;
        
        // Save document
        self.save_document_files(&document)?;
        
        // Update index
        self.update_document_index(&document)?;
        
        // Audit log the submission
        audit_log_action("SUBMIT_FOR_REVIEW", "Document", doc_id)?;
        
        Ok(document)
    }
    
    /// Approve a document (InReview → Approved)
    pub fn approve_document(&self, doc_id: &str, approver_id: &str, _signature: Option<&str>) -> QmsResult<Document> {
        use super::document::DocumentStatus;
        
        // Validate approver has necessary permissions (placeholder for Phase 4 user management)
        if !self.validate_approval_permission(approver_id)? {
            return Err(QmsError::validation_error(&format!(
                "User {approver_id} does not have permission to approve documents"
            )));
        }
        
        let mut document = self.read_document(doc_id)?;
        
        // Validate current status allows approval
        if document.status != DocumentStatus::InReview {
            return Err(QmsError::validation_error(&format!(
                "Document can only be approved from InReview status, current status: {:?}",
                document.status
            )));
        }

        // Update status to Approved with approver
        document.update_status(DocumentStatus::Approved, Some(approver_id.to_string()))?;
        
        // Save document
        self.save_document_files(&document)?;
        
        // Update index
        self.update_document_index(&document)?;
        
        // Audit log the approval with signature
        audit_log_action("APPROVE", "Document", doc_id)?;
        
        Ok(document)
    }
    
    /// Reject a document (InReview → Draft)
    pub fn reject_document(&self, doc_id: &str, _reviewer_id: &str, _reason: Option<&str>) -> QmsResult<Document> {
        use super::document::DocumentStatus;
        
        let mut document = self.read_document(doc_id)?;
        
        // Validate current status allows rejection
        if document.status != DocumentStatus::InReview {
            return Err(QmsError::validation_error(&format!(
                "Document can only be rejected from InReview status, current status: {:?}",
                document.status
            )));
        }

        // Update status back to Draft
        document.update_status(DocumentStatus::Draft, None)?;
        
        // Save document
        self.save_document_files(&document)?;
        
        // Update index
        self.update_document_index(&document)?;
        
        // Audit log the rejection
        audit_log_action("REJECT", "Document", doc_id)?;
        
        Ok(document)
    }
    
    /// Archive a document (Any status → Archived)
    pub fn archive_document(&self, doc_id: &str, _user_id: &str, _reason: Option<&str>) -> QmsResult<Document> {
        use super::document::DocumentStatus;
        
        let mut document = self.read_document(doc_id)?;
        
        // Validate current status allows archiving (from any status except already archived)
        if document.status == DocumentStatus::Archived {
            return Err(QmsError::validation_error(
                "Document is already archived"
            ));
        }

        // Update status to Archived
        document.update_status(DocumentStatus::Archived, None)?;
        
        // Save document (keep in place, just update status)
        self.save_document_files(&document)?;
        
        // Update index
        self.update_document_index(&document)?;
        
        // Audit log the archival
        audit_log_action("ARCHIVE", "Document", doc_id)?;
        
        Ok(document)
    }
    
    /// Get documents by status for workflow management
    pub fn get_documents_by_status(&self, status: super::document::DocumentStatus) -> QmsResult<Vec<DocumentIndexEntry>> {
        let documents = self.list_documents()?;
        let filtered: Vec<DocumentIndexEntry> = documents
            .into_iter()
            .filter(|doc| doc.status == status.to_string())
            .collect();
        Ok(filtered)
    }
    
    /// Restore an archived document (Archived → Draft)
    pub fn restore_document(&self, doc_id: &str, _user_id: &str, _reason: Option<&str>) -> QmsResult<Document> {
        use super::document::DocumentStatus;
        
        let mut document = self.read_document(doc_id)?;
        
        // Validate current status allows restoration (only archived documents can be restored)
        if document.status != DocumentStatus::Archived {
            return Err(QmsError::validation_error(&format!(
                "Document can only be restored from Archived status, current status: {:?}",
                document.status
            )));
        }

        // Update status to Draft (starting point for restored documents)
        document.update_status(DocumentStatus::Draft, None)?;
        
        // Save document
        self.save_document_files(&document)?;
        
        // Update index
        self.update_document_index(&document)?;
        
        // Audit log the restoration
        audit_log_action("RESTORE", "Document", doc_id)?;
        
        Ok(document)
    }

    /// Validate that a user has approval permissions (placeholder for Phase 4)
    const fn validate_approval_permission(&self, _user_id: &str) -> QmsResult<bool> {
        // Placeholder implementation - in Phase 4 this will check actual user roles
        // For now, assume "QualityEngineer" users can approve (based on PRD)
        // This is a simplified check that will be replaced with proper user management
        Ok(true) // Allow all users for now until Phase 4 user management is implemented
    }

    // ========== BACKUP MANAGEMENT METHODS (Phase 2.1.14) ==========

    /// Create an automatic backup of a document
    pub fn create_automatic_backup(&self, document: &Document, created_by: &str, reason: &str) -> QmsResult<String> {
        let backup_manager = DocumentBackupManager::new(self.project_path.clone());

        // Read document content for backup
        let document_content = std::fs::read_to_string(&document.file_path)
            .unwrap_or_else(|_| "Document content unavailable".to_string());

        let backup_metadata = backup_manager.create_backup(
            &document.id,
            &document.version,
            std::path::Path::new(&document.file_path),
            &document_content,
            created_by,
            reason,
        )?;

        Ok(backup_metadata.backup_id)
    }

    /// List all backups for a specific document
    pub fn list_document_backups(&self, document_id: &str) -> QmsResult<Vec<super::backup::BackupMetadata>> {
        let backup_manager = DocumentBackupManager::new(self.project_path.clone());
        backup_manager.list_document_backups(document_id)
    }

    /// List all backups in the system
    pub fn list_all_backups(&self) -> QmsResult<Vec<super::backup::BackupMetadata>> {
        let backup_manager = DocumentBackupManager::new(self.project_path.clone());
        backup_manager.list_all_backups()
    }

    /// Verify backup integrity
    pub fn verify_backup(&self, backup_id: &str) -> QmsResult<bool> {
        let backup_manager = DocumentBackupManager::new(self.project_path.clone());
        let metadata = backup_manager.get_backup_metadata(backup_id)?;
        backup_manager.verify_backup(&metadata)
    }

    /// Recover document from backup
    pub fn recover_from_backup(
        &self,
        backup_id: &str,
        target_document_id: Option<&str>,
        recovered_by: &str,
    ) -> QmsResult<Document> {
        let backup_manager = DocumentBackupManager::new(self.project_path.clone());
        let metadata = backup_manager.get_backup_metadata(backup_id)?;

        // Determine recovery target
        let document_id = target_document_id.unwrap_or(&metadata.document_id);
        
        // Create recovery path
        let doc_dir = self.project_path.join("documents").join(document_id);
        let recovery_path = doc_dir.join("content.md");

        // Ensure target directory exists
        fs::create_dir_all(&doc_dir)?;

        // Recover content from backup
        let recovered_content = backup_manager.recover_from_backup(
            backup_id,
            &recovery_path,
            recovered_by,
        )?;

        // If recovering to original document, update the document
        if target_document_id.is_none() || target_document_id == Some(&metadata.document_id) {
            // Read existing document metadata if it exists
            if let Ok(mut document) = self.read_document(document_id) {
                // Update content and version
                document.content = recovered_content;
                document.version = DocumentVersionControl::increment_version(&document.version, VersionChangeType::Patch)?;
                document.updated_at = crate::utils::current_timestamp_string();

                // Save updated document
                self.save_document_files(&document)?;
                self.update_document_index(&document)?;

                audit_log_action("DOCUMENT_RECOVERED_IN_PLACE", "Document", document_id)?;

                Ok(document)
            } else {
                Err(QmsError::NotFound(format!("Target document not found: {document_id}")))
            }
        } else {
            // Create new document with recovered content
            let document = self.create_document(
                format!("Recovered from backup {backup_id}"),
                recovered_content,
                DocumentType::Other(format!("Recovered-{}", metadata.document_version)),
                recovered_by.to_string(),
            )?;

            audit_log_action("DOCUMENT_RECOVERED_AS_NEW", "Document", &document.id)?;

            Ok(document)
        }
    }

    /// Delete a backup
    pub fn delete_backup(&self, backup_id: &str, deleted_by: &str) -> QmsResult<()> {
        let backup_manager = DocumentBackupManager::new(self.project_path.clone());
        backup_manager.delete_backup(backup_id, deleted_by)
    }

    /// Clean up old backups based on retention policy
    pub fn cleanup_old_backups(&self, retention_days: u64, cleaned_by: &str) -> QmsResult<u32> {
        let backup_manager = DocumentBackupManager::new(self.project_path.clone());
        backup_manager.cleanup_old_backups(retention_days, cleaned_by)
    }

    /// Get backup metadata
    pub fn get_backup_metadata(&self, backup_id: &str) -> QmsResult<super::backup::BackupMetadata> {
        let backup_manager = DocumentBackupManager::new(self.project_path.clone());
        backup_manager.get_backup_metadata(backup_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::document_control::document::DocumentStatus;
    use crate::modules::audit_logger::{AuditConfig, initialize_audit_system};
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    fn create_test_project() -> PathBuf {
        let mut temp_dir = env::temp_dir();
        // Use current timestamp and random UUID for better uniqueness
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        temp_dir.push(format!("qms_test_{}_{}", timestamp, crate::utils::generate_uuid()));
        fs::create_dir_all(&temp_dir).unwrap();

        // Initialize audit system for test
        init_audit_for_test(&temp_dir);

        temp_dir
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
                // If initialization fails, continue with test - audit is not critical for document service logic
                eprintln!("Warning: Audit system initialization failed in test, continuing without audit");
            }
        }
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
            None, // change_type - auto-determine
            None, // change_description - use default
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

    #[test]
    fn test_archive_and_restore_document() {
        let project_path = create_test_project();
        let service = DocumentService::new(project_path.clone());

        // Create a test document
        let doc = service.create_document(
            "Archive Test Document".to_string(),
            "This document will be archived and restored".to_string(),
            DocumentType::UserRequirements,
            "test_user".to_string(),
        ).unwrap();

        // Initially in Draft status
        assert_eq!(doc.status, DocumentStatus::Draft);

        // Archive the document
        let archived_doc = service.archive_document(&doc.id, "test_user", Some("Testing archive functionality")).unwrap();
        assert_eq!(archived_doc.status, DocumentStatus::Archived);
        assert_eq!(archived_doc.id, doc.id);
        assert_eq!(archived_doc.title, "Archive Test Document");
        assert_eq!(archived_doc.content, doc.content);

        // Verify document can be read while archived
        let read_archived = service.read_document(&doc.id).unwrap();
        assert_eq!(read_archived.status, DocumentStatus::Archived);

        // Restore the document
        let restored_doc = service.restore_document(&doc.id, "test_user", Some("Testing restore functionality")).unwrap();
        assert_eq!(restored_doc.status, DocumentStatus::Draft);
        assert_eq!(restored_doc.id, doc.id);
        assert_eq!(restored_doc.title, "Archive Test Document");
        assert_eq!(restored_doc.content, doc.content);

        // Verify document can be read after restoration
        let read_restored = service.read_document(&doc.id).unwrap();
        assert_eq!(read_restored.status, DocumentStatus::Draft);

        cleanup_test_project(&project_path);
    }

    #[test]
    fn test_archive_restore_validation() {
        let project_path = create_test_project();
        let service = DocumentService::new(project_path.clone());

        // Create a test document
        let doc = service.create_document(
            "Validation Test Document".to_string(),
            "Testing archive/restore validation".to_string(),
            DocumentType::SoftwareRequirementsSpecification,
            "test_user".to_string(),
        ).unwrap();

        // Try to restore a non-archived document (should fail)
        let restore_result = service.restore_document(&doc.id, "test_user", None);
        assert!(restore_result.is_err());
        assert!(restore_result.unwrap_err().to_string().contains("can only be restored from Archived status"));

        // Archive the document
        let archived_doc = service.archive_document(&doc.id, "test_user", None).unwrap();
        assert_eq!(archived_doc.status, DocumentStatus::Archived);

        // Try to archive an already archived document (should fail since document is already archived)
        let archive_again_result = service.archive_document(&doc.id, "test_user", Some("Testing double archive"));
        assert!(archive_again_result.is_err(), "Should not be able to archive an already archived document");
        assert!(archive_again_result.unwrap_err().to_string().contains("already archived"));

        // Restore should work
        let restored_doc = service.restore_document(&doc.id, "test_user", None).unwrap();
        assert_eq!(restored_doc.status, DocumentStatus::Draft);

        cleanup_test_project(&project_path);
    }
}
