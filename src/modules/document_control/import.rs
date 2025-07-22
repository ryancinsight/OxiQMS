use std::fs;
use std::path::Path;
use crate::prelude::*;
use crate::modules::document_control::document::{Document, DocumentType, DocumentStatus, RegulatoryReference};
use crate::modules::document_control::service::DocumentService;
use crate::modules::audit_logger::audit_log_action;
use crate::utils::{get_current_project_path, current_date_string};

/// Supported import formats
#[derive(Debug, Clone, PartialEq)]
pub enum ImportFormat {
    Markdown,
    Csv,
    Json,
}

impl ImportFormat {
    pub fn from_str(s: &str) -> Result<ImportFormat, QmsError> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Ok(ImportFormat::Markdown),
            "csv" => Ok(ImportFormat::Csv),
            "json" => Ok(ImportFormat::Json),
            _ => Err(QmsError::validation_error(&format!("Unsupported import format: {s}"))),
        }
    }

    #[allow(dead_code)]
    pub const fn extension(&self) -> &'static str {
        match self {
            ImportFormat::Markdown => "md",
            ImportFormat::Csv => "csv",
            ImportFormat::Json => "json",
        }
    }
}

/// Import options and configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ImportOptions {
    pub format: ImportFormat,
    pub overwrite_existing: bool,
    pub validate_content: bool,
    pub default_author: Option<String>,
    pub default_status: DocumentStatus,
    pub default_type: DocumentType,
    pub preview_only: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        ImportOptions {
            format: ImportFormat::Markdown,
            overwrite_existing: false,
            validate_content: true,
            default_author: None,
            default_status: DocumentStatus::Draft,
            default_type: DocumentType::Other("General".to_string()),
            preview_only: false,
        }
    }
}

/// CSV import record structure
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CsvImportRecord {
    pub title: String,
    pub doc_type: String,
    pub content_file: String,
    pub author: String,
    pub status: String,
    pub version: Option<String>,
    pub tags: Option<String>,
}

/// Import result summary
#[derive(Debug, Clone)]
pub struct ImportResult {
    pub total_processed: usize,
    pub successful_imports: usize,
    pub failed_imports: usize,
    pub skipped_duplicates: usize,
    pub imported_documents: Vec<String>, // Document IDs
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl Default for ImportResult {
    fn default() -> Self {
        Self::new()
    }
}

impl ImportResult {
    pub const fn new() -> Self {
        ImportResult {
            total_processed: 0,
            successful_imports: 0,
            failed_imports: 0,
            skipped_duplicates: 0,
            imported_documents: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_success(&mut self, doc_id: String) {
        self.successful_imports += 1;
        self.imported_documents.push(doc_id);
    }

    pub fn add_failure(&mut self, error: String) {
        self.failed_imports += 1;
        self.errors.push(error);
    }

    pub fn add_skip(&mut self, reason: String) {
        self.skipped_duplicates += 1;
        self.warnings.push(reason);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// Document importer for various formats
#[allow(dead_code)]
pub struct DocumentImporter {
    document_service: DocumentService,
    project_path: String,
}

impl DocumentImporter {
    pub fn new() -> Result<Self, QmsError> {
        let project_path = get_current_project_path()?;
        let document_service = DocumentService::new(project_path.clone());

        Ok(DocumentImporter {
            document_service,
            project_path: project_path.to_string_lossy().to_string(),
        })
    }

    /// Create a new DocumentImporter for testing purposes (Dependency Inversion Principle)
    /// This allows testing without requiring a full QMS project setup
    #[cfg(test)]
    pub fn new_for_test() -> Self {
        use std::path::PathBuf;
        let test_path = PathBuf::from("test_project");
        let document_service = DocumentService::new(test_path.clone());

        DocumentImporter {
            document_service,
            project_path: test_path.to_string_lossy().to_string(),
        }
    }

    /// Import documents from a file
    pub fn import_from_file(&mut self, file_path: &str, options: ImportOptions) -> Result<ImportResult, QmsError> {
        let mut result = ImportResult::new();
        
        if !Path::new(file_path).exists() {
            return Err(QmsError::not_found(&format!("Import file not found: {file_path}")));
        }

        match options.format {
            ImportFormat::Markdown => self.import_markdown_file(file_path, &options, &mut result)?,
            ImportFormat::Csv => self.import_csv_file(file_path, &options, &mut result)?,
            ImportFormat::Json => self.import_json_file(file_path, &options, &mut result)?,
        }

        // Audit the import operation
        audit_log_action("IMPORT_DOCUMENT", "System", "DocumentImport")?;

        Ok(result)
    }

    /// Import a single Markdown document
    fn import_markdown_file(&mut self, file_path: &str, options: &ImportOptions, result: &mut ImportResult) -> Result<(), QmsError> {
        result.total_processed += 1;

        let content = fs::read_to_string(file_path)
            .map_err(|e| QmsError::io_error(&format!("Failed to read {file_path}: {e}")))?;

        // Extract metadata from front matter or filename
        let (title, metadata) = self.extract_markdown_metadata(&content, file_path)?;
        
        // Create document with metadata
        let doc_id = self.generate_document_id(&title);
        
        // Check for duplicates unless overwriting
        if !options.overwrite_existing && self.document_exists(&doc_id)? {
            result.add_skip(format!("Document {doc_id} already exists"));
            return Ok(());
        }

        if options.preview_only {
            result.add_warning(format!("Preview: Would import document {doc_id} with title '{title}'"));
            return Ok(());
        }

        let document = self.document_service.create_document(
            title,
            content,
            metadata.doc_type.unwrap_or_else(|| options.default_type.clone()),
            metadata.author.or_else(|| options.default_author.clone()).unwrap_or_else(|| "Unknown".to_string()),
        )?;
        result.add_success(document.id);
        
        Ok(())
    }

    /// Import documents from a CSV file
    fn import_csv_file(&mut self, file_path: &str, options: &ImportOptions, result: &mut ImportResult) -> Result<(), QmsError> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| QmsError::io_error(&format!("Failed to read CSV {file_path}: {e}")))?;

        let records = self.parse_csv_content(&content)?;
        
        for (line_num, record) in records.iter().enumerate() {
            result.total_processed += 1;
            
            // Validate record
            if let Err(e) = self.validate_csv_record(record) {
                result.add_failure(format!("Line {}: {}", line_num + 2, e)); // +2 for header and 1-based indexing
                continue;
            }

            // Check if content file exists
            if !Path::new(&record.content_file).exists() {
                result.add_failure(format!("Line {}: Content file not found: {}", line_num + 2, record.content_file));
                continue;
            }

            // Read content file
            let content = match fs::read_to_string(&record.content_file) {
                Ok(content) => content,
                Err(e) => {
                    result.add_failure(format!("Line {}: Failed to read content file {}: {}", line_num + 2, record.content_file, e));
                    continue;
                }
            };

            let doc_id = self.generate_document_id(&record.title);
            
            // Check for duplicates
            if !options.overwrite_existing && self.document_exists(&doc_id)? {
                result.add_skip(format!("Line {}: Document {} already exists", line_num + 2, doc_id));
                continue;
            }

            if options.preview_only {
                result.add_warning(format!("Preview: Would import document {} with title '{}'", doc_id, record.title));
                continue;
            }

            // Parse document type and status
            let doc_type = self.parse_document_type(&record.doc_type).unwrap_or_else(|_| options.default_type.clone());
            let _status = self.parse_document_status(&record.status).unwrap_or_else(|_| options.default_status.clone());
            let _tags: Vec<String> = record.tags.as_ref().map(|t| t.split(',').map(|s| s.trim().to_string()).collect()).unwrap_or_default();

            match self.document_service.create_document(
                record.title.clone(),
                content,
                doc_type,
                record.author.clone(),
            ) {
                Ok(document) => result.add_success(document.id),
                Err(e) => result.add_failure(format!("Line {}: Failed to create document: {}", line_num + 2, e)),
            }
        }

        Ok(())
    }

    /// Import documents from a JSON file
    fn import_json_file(&mut self, file_path: &str, options: &ImportOptions, result: &mut ImportResult) -> Result<(), QmsError> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| QmsError::io_error(&format!("Failed to read JSON {file_path}: {e}")))?;

        // Parse JSON manually (no external dependencies)
        let documents = self.parse_json_documents(&content)?;
        
        for (index, document) in documents.into_iter().enumerate() {
            result.total_processed += 1;
            
            // Validate document
            if let Err(e) = self.validate_document(&document) {
                result.add_failure(format!("Document {}: {}", index + 1, e));
                continue;
            }

            // Check for duplicates
            if !options.overwrite_existing && self.document_exists(&document.id)? {
                result.add_skip(format!("Document {}: {} already exists", index + 1, document.id));
                continue;
            }

            if options.preview_only {
                result.add_warning(format!("Preview: Would import document {} with title '{}'", document.id, document.title));
                continue;
            }

            match self.document_service.create_document(
                document.title.clone(),
                document.content.clone(),
                document.doc_type.clone(),
                document.created_by.clone(),
            ) {
                Ok(created_doc) => result.add_success(created_doc.id),
                Err(e) => result.add_failure(format!("Document {}: Failed to create: {}", index + 1, e)),
            }
        }

        Ok(())
    }

    /// Extract metadata from Markdown front matter
    fn extract_markdown_metadata(&self, content: &str, file_path: &str) -> Result<(String, DocumentMetadata), QmsError> {
        let mut title = None;
        let mut doc_type = None;
        let mut version = None;
        let mut status = None;
        let mut author = None;
        let mut tags = None;
        let regulatory_mapping = None;

        // Check for YAML front matter
        if content.starts_with("---\n") {
            if let Some(end_index) = content[4..].find("\n---\n") {
                let front_matter = &content[4..end_index + 4];
                
                // Parse YAML-like front matter manually
                for line in front_matter.lines() {
                    if let Some((key, value)) = line.split_once(':') {
                        let key = key.trim();
                        let value = value.trim().trim_matches('"');
                        
                        match key {
                            "title" => title = Some(value.to_string()),
                            "type" => doc_type = self.parse_document_type(value).ok(),
                            "version" => version = Some(value.to_string()),
                            "status" => status = self.parse_document_status(value).ok(),
                            "author" => author = Some(value.to_string()),
                            "tags" => {
                                tags = Some(value.split(',').map(|s| s.trim().to_string()).collect());
                            },
                            _ => {}
                        }
                    }
                }
            }
        }

        // Fallback to first heading or filename
        if title.is_none() {
            title = content.lines()
                .find(|line| line.starts_with("# "))
                .map(|line| line[2..].trim().to_string())
                .or_else(|| {
                    Path::new(file_path)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                });
        }

        let title = title.unwrap_or_else(|| "Untitled Document".to_string());
        
        Ok((title, DocumentMetadata {
            doc_type,
            version,
            status,
            author,
            tags,
            regulatory_mapping,
        }))
    }

    /// Parse CSV content into records
    fn parse_csv_content(&self, content: &str) -> Result<Vec<CsvImportRecord>, QmsError> {
        let lines: Vec<&str> = content.lines().collect();
        
        if lines.is_empty() {
            return Err(QmsError::validation_error("CSV file is empty"));
        }

        // Parse header
        let header = lines[0];
        let expected_columns = ["title", "type", "content_file", "author", "status"];
        let _optional_columns = ["version", "tags"];
        
        let columns: Vec<&str> = header.split(',').map(|s| s.trim()).collect();
        
        // Validate required columns
        for required in &expected_columns {
            if !columns.contains(required) {
                return Err(QmsError::validation_error(&format!("CSV missing required column: {required}")));
            }
        }

        let mut records = Vec::new();
        
        for (line_num, line) in lines.iter().skip(1).enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            
            let values: Vec<&str> = line.split(',').map(|s| s.trim().trim_matches('"')).collect();
            
            if values.len() < expected_columns.len() {
                return Err(QmsError::validation_error(&format!("Line {}: Insufficient columns", line_num + 2)));
            }

            let record = CsvImportRecord {
                title: values[0].to_string(),
                doc_type: values[1].to_string(),
                content_file: values[2].to_string(),
                author: values[3].to_string(),
                status: values[4].to_string(),
                version: values.get(5).map(|s| s.to_string()),
                tags: values.get(6).map(|s| s.to_string()),
            };
            
            records.push(record);
        }

        Ok(records)
    }

    /// Parse JSON documents (simplified JSON parsing)
    #[allow(dead_code)]
    fn parse_json_documents(&self, _content: &str) -> Result<Vec<Document>, QmsError> {
        // This is a simplified JSON parser for basic document structure
        // In a real implementation, this would be more robust
        
        // For now, return an error indicating this feature needs implementation
        Err(QmsError::validation_error("JSON import not yet implemented - use CSV or Markdown"))
    }

    /// Validate CSV record
    fn validate_csv_record(&self, record: &CsvImportRecord) -> Result<(), QmsError> {
        if record.title.trim().is_empty() {
            return Err(QmsError::validation_error("Title cannot be empty"));
        }
        
        if record.title.len() > 200 {
            return Err(QmsError::validation_error("Title too long (max 200 characters)"));
        }
        
        if record.author.trim().is_empty() {
            return Err(QmsError::validation_error("Author cannot be empty"));
        }
        
        if record.content_file.trim().is_empty() {
            return Err(QmsError::validation_error("Content file path cannot be empty"));
        }
        
        Ok(())
    }

    /// Validate document structure
    fn validate_document(&self, document: &Document) -> Result<(), QmsError> {
        if document.title.trim().is_empty() {
            return Err(QmsError::validation_error("Document title cannot be empty"));
        }
        
        if document.title.len() > 200 {
            return Err(QmsError::validation_error("Document title too long (max 200 characters)"));
        }
        
        if document.created_by.trim().is_empty() {
            return Err(QmsError::validation_error("Document author cannot be empty"));
        }
        
        Ok(())
    }

    /// Parse document type from string
    fn parse_document_type(&self, type_str: &str) -> Result<DocumentType, QmsError> {
        match type_str.to_lowercase().as_str() {
            "srs" | "software_requirements_specification" => Ok(DocumentType::SoftwareRequirementsSpecification),
            "sdd" | "software_design_description" => Ok(DocumentType::SoftwareDesignDescription),
            "rmf" | "risk_management_file" => Ok(DocumentType::RiskManagementFile),
            "test_protocol" => Ok(DocumentType::TestProtocol),
            "user_requirements" => Ok(DocumentType::UserRequirements),
            other => Ok(DocumentType::Other(other.to_string())),
        }
    }

    /// Parse document status from string
    fn parse_document_status(&self, status_str: &str) -> Result<DocumentStatus, QmsError> {
        match status_str.to_lowercase().as_str() {
            "draft" => Ok(DocumentStatus::Draft),
            "review_pending" | "review" => Ok(DocumentStatus::InReview),
            "approved" => Ok(DocumentStatus::Approved),
            "archived" => Ok(DocumentStatus::Archived),
            _ => Err(QmsError::validation_error(&format!("Unknown document status: {status_str}"))),
        }
    }

    /// Generate document ID based on title
    fn generate_document_id(&self, title: &str) -> String {
        let date = current_date_string().split('T').next().unwrap_or("20240101").replace("-", "");
        let title_part = title.chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ')
            .collect::<String>()
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join("")
            .to_uppercase();
        
        format!("DOC-{date}-{title_part}")
    }

    /// Check if document exists
    fn document_exists(&self, doc_id: &str) -> Result<bool, QmsError> {
        match self.document_service.read_document(doc_id) {
            Ok(_) => Ok(true),
            Err(QmsError::NotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Get current project ID
    #[allow(dead_code)]
    fn get_project_id(&self) -> Result<String, QmsError> {
        // For now, use a default project ID
        // In a real implementation, this would come from the project configuration
        Ok("default-project".to_string())
    }
}

/// Metadata extracted from document
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct DocumentMetadata {
    pub doc_type: Option<DocumentType>,
    pub version: Option<String>,
    pub status: Option<DocumentStatus>,
    pub author: Option<String>,
    pub tags: Option<Vec<String>>,
    pub regulatory_mapping: Option<Vec<RegulatoryReference>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_format_parsing() {
        assert_eq!(ImportFormat::from_str("markdown").unwrap(), ImportFormat::Markdown);
        assert_eq!(ImportFormat::from_str("md").unwrap(), ImportFormat::Markdown);
        assert_eq!(ImportFormat::from_str("csv").unwrap(), ImportFormat::Csv);
        assert_eq!(ImportFormat::from_str("json").unwrap(), ImportFormat::Json);
        assert!(ImportFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_import_format_extension() {
        assert_eq!(ImportFormat::Markdown.extension(), "md");
        assert_eq!(ImportFormat::Csv.extension(), "csv");
        assert_eq!(ImportFormat::Json.extension(), "json");
    }

    #[test]
    fn test_import_result() {
        let mut result = ImportResult::new();
        
        result.add_success("DOC-001".to_string());
        result.add_failure("Error message".to_string());
        result.add_skip("Skip reason".to_string());
        result.add_warning("Warning message".to_string());
        
        assert_eq!(result.successful_imports, 1);
        assert_eq!(result.failed_imports, 1);
        assert_eq!(result.skipped_duplicates, 1);
        assert_eq!(result.imported_documents.len(), 1);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.warnings.len(), 2); // 1 skip + 1 warning
    }

    #[test]
    fn test_csv_parsing() {
        let csv_content = r#"title,type,content_file,author,status
"Test Document","SRS","test.md","John Doe","draft"
"Another Doc","Test Protocol","another.md","Jane Smith","approved""#;

        let importer = DocumentImporter::new_for_test();
        let records = importer.parse_csv_content(csv_content).unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].title, "Test Document");
        assert_eq!(records[0].doc_type, "SRS");
        assert_eq!(records[1].title, "Another Doc");
        assert_eq!(records[1].status, "approved");
    }

    #[test]
    fn test_document_id_generation() {
        let importer = DocumentImporter::new_for_test();
        let id = importer.generate_document_id("Software Requirements Specification");

        assert!(id.starts_with("DOC-"));
        assert!(id.contains("SOFTWARE"));
    }

    #[test]
    fn test_document_validation() {
        let importer = DocumentImporter::new_for_test();
        let mut document = create_test_document();

        // Valid document should pass
        assert!(importer.validate_document(&document).is_ok());

        // Empty title should fail
        document.title = "".to_string();
        assert!(importer.validate_document(&document).is_err());

        // Too long title should fail
        document.title = "x".repeat(250);
        assert!(importer.validate_document(&document).is_err());
    }

    fn create_test_document() -> Document {
        Document {
            id: "DOC-20240115-001".to_string(),
            project_id: "test-project".to_string(),
            title: "Test Document".to_string(),
            content: "# Test Document\n\nThis is a test document.".to_string(),
            doc_type: DocumentType::SoftwareRequirementsSpecification,
            version: "1.0.0".to_string(),
            status: DocumentStatus::Draft,
            created_at: "2022-01-01T00:00:00Z".to_string(),
            updated_at: "2022-01-01T00:05:00Z".to_string(),
            created_by: "test_user".to_string(),
            approved_by: None,
            file_path: "docs/DOC-20240115-001/content.md".to_string(),
            checksum: "abc123def456".to_string(),
            tags: vec!["test".to_string()],
            regulatory_mapping: Vec::new(),
            locked: false,
            locked_by: None,
            locked_at: None,
        }
    }
}
