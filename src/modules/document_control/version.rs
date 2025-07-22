//! Document version control functionality
//! Phase 2.1.4 - Document Version Control Implementation
//! Implements semantic versioning with complete version history

use crate::error::{QmsError, QmsResult};
use crate::json_utils::{JsonValue, JsonSerializable};
use crate::modules::document_control::document::Document;
use std::collections::HashMap;
use std::path::Path;
use std::fs;

/// Version change type for determining increment rules
#[derive(Debug, Clone, PartialEq)]
pub enum VersionChangeType {
    /// Patch increment: Content changes only
    Patch,
    /// Minor increment: Status changes, metadata updates
    Minor,
    /// Major increment: Document type changes, major restructuring
    Major,
}

/// Document version history entry
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DocumentVersion {
    pub version: String,
    pub document_id: String,
    pub created_at: u64,
    pub created_by: String,
    pub change_type: VersionChangeType,
    pub change_description: String,
    pub checksum: String,
}

/// Version control service for documents
pub struct DocumentVersionControl;

impl DocumentVersionControl {
    /// Increment document version based on change type
    /// 
    /// Rules:
    /// - Patch: Content changes only (1.0.0 -> 1.0.1)
    /// - Minor: Status changes, metadata updates (1.0.0 -> 1.1.0)
    /// - Major: Document type changes, major restructuring (1.0.0 -> 2.0.0)
    pub fn increment_version(current_version: &str, change_type: VersionChangeType) -> QmsResult<String> {
        let parts: Vec<&str> = current_version.split('.').collect();
        if parts.len() != 3 {
            return Err(QmsError::validation_error("Invalid semantic version format"));
        }

        let major: u32 = parts[0].parse()
            .map_err(|_| QmsError::validation_error("Invalid major version number"))?;
        let minor: u32 = parts[1].parse()
            .map_err(|_| QmsError::validation_error("Invalid minor version number"))?;
        let patch: u32 = parts[2].parse()
            .map_err(|_| QmsError::validation_error("Invalid patch version number"))?;

        let new_version = match change_type {
            VersionChangeType::Patch => format!("{}.{}.{}", major, minor, patch + 1),
            VersionChangeType::Minor => format!("{}.{}.0", major, minor + 1),
            VersionChangeType::Major => format!("{}.0.0", major + 1),
        };

        Ok(new_version)
    }

    /// Get version history for a document
    pub fn get_version_history(project_path: &Path, document_id: &str) -> QmsResult<Vec<DocumentVersion>> {
        let versions_dir = project_path.join("documents").join(document_id).join("versions");
        
        if !versions_dir.exists() {
            return Ok(Vec::new());
        }

        let mut versions = Vec::new();
        
        // Read all version files
        for entry in fs::read_dir(&versions_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                    match Self::load_version_from_file(&path, filename, document_id) {
                        Ok(version) => versions.push(version),
                        Err(e) => {
                            // Log error but continue processing other versions
                            eprintln!("Warning: Failed to load version {filename}: {e}");
                        }
                    }
                }
            }
        }

        // Sort versions by semantic version order
        versions.sort_by(|a, b| Self::compare_versions(&a.version, &b.version));
        
        Ok(versions)
    }

    /// Compare two semantic versions
    /// Returns std::cmp::Ordering: Less, Equal, or Greater
    pub fn compare_versions(version_a: &str, version_b: &str) -> std::cmp::Ordering {
        let parts_a: Vec<u32> = version_a.split('.').filter_map(|s| s.parse().ok()).collect();
        let parts_b: Vec<u32> = version_b.split('.').filter_map(|s| s.parse().ok()).collect();

        // Compare major.minor.patch in order
        for i in 0..3 {
            let a = parts_a.get(i).unwrap_or(&0);
            let b = parts_b.get(i).unwrap_or(&0);
            
            match a.cmp(b) {
                std::cmp::Ordering::Equal => continue,
                other => return other,
            }
        }
        
        std::cmp::Ordering::Equal
    }

    /// Create a new version snapshot of a document
    pub fn create_version_snapshot(
        project_path: &Path,
        document: &Document,
        change_type: VersionChangeType,
        change_description: &str,
        created_by: &str,
    ) -> QmsResult<DocumentVersion> {
        let doc_dir = project_path.join("documents").join(&document.id);
        let versions_dir = doc_dir.join("versions");
        
        // Ensure versions directory exists
        fs::create_dir_all(&versions_dir)?;

        // Create version entry
        let version_entry = DocumentVersion {
            version: document.version.clone(),
            document_id: document.id.clone(),
            created_at: crate::utils::current_timestamp(), // Use current timestamp as u64
            created_by: created_by.to_string(),
            change_type,
            change_description: change_description.to_string(),
            checksum: document.checksum.clone(),
        };

        // Save version file with document data
        let version_file = versions_dir.join(format!("{}.json", document.version));
        let version_data = Self::create_version_file_content(document, &version_entry)?;
        fs::write(version_file, version_data)?;

        Ok(version_entry)
    }

    /// Get specific version of a document
    pub fn get_document_version(project_path: &Path, document_id: &str, version: &str) -> QmsResult<Document> {
        let version_file = project_path
            .join("documents")
            .join(document_id)
            .join("versions")
            .join(format!("{version}.json"));

        if !version_file.exists() {
            return Err(QmsError::not_found(&format!("Document version {version} not found")));
        }

        let content = fs::read_to_string(version_file)?;
        let json_value = JsonValue::parse(&content)?;
        
        if let JsonValue::Object(obj) = json_value {
            if let Some(doc_data) = obj.get("document") {
                let doc_json = doc_data.json_to_string();
                return Document::from_json(&doc_json).map_err(|e| QmsError::validation_error(&e.to_string()));
            }
        }

        Err(QmsError::validation_error("Invalid version file format"))
    }

    /// List all available versions for a document
    pub fn list_document_versions(project_path: &Path, document_id: &str) -> QmsResult<Vec<String>> {
        let versions_dir = project_path.join("documents").join(document_id).join("versions");
        
        if !versions_dir.exists() {
            return Ok(Vec::new());
        }

        let mut versions = Vec::new();
        
        for entry in fs::read_dir(&versions_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                    versions.push(filename.to_string());
                }
            }
        }

        // Sort versions semantically
        versions.sort_by(|a, b| Self::compare_versions(a, b));
        
        Ok(versions)
    }

    /// Validate version format
    #[allow(dead_code)]
    pub fn is_valid_semantic_version(version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        
        if parts.len() != 3 {
            return false;
        }

        // Check that all parts are valid numbers
        parts.iter().all(|part| part.parse::<u32>().is_ok())
    }

    /// Get latest version number for a document
    pub fn get_latest_version(project_path: &Path, document_id: &str) -> QmsResult<Option<String>> {
        let versions = Self::list_document_versions(project_path, document_id)?;
        
        if versions.is_empty() {
            Ok(None)
        } else {
            // Versions are already sorted, get the last one
            Ok(Some(versions.last().unwrap().clone()))
        }
    }

    /// Helper function to load version from file
    fn load_version_from_file(
        file_path: &Path,
        version: &str,
        document_id: &str,
    ) -> QmsResult<DocumentVersion> {
        let content = fs::read_to_string(file_path)?;
        let json_value = JsonValue::parse(&content)?;
        
        if let JsonValue::Object(obj) = json_value {
            let metadata = obj.get("metadata")
                .and_then(|v| if let JsonValue::Object(m) = v { Some(m) } else { None })
                .ok_or_else(|| QmsError::validation_error("Missing metadata in version file"))?;

            let created_at = metadata.get("created_at")
                .and_then(|v| if let JsonValue::Number(n) = v { Some(*n as u64) } else { None })
                .unwrap_or(0);

            let created_by = metadata.get("created_by")
                .and_then(|v| if let JsonValue::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_else(|| "unknown".to_string());

            let change_description = metadata.get("change_description")
                .and_then(|v| if let JsonValue::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_else(|| "No description".to_string());

            let change_type_str = metadata.get("change_type")
                .and_then(|v| if let JsonValue::String(s) = v { Some(s.as_str()) } else { None })
                .unwrap_or("Patch");

            let change_type = match change_type_str {
                "Major" => VersionChangeType::Major,
                "Minor" => VersionChangeType::Minor,
                _ => VersionChangeType::Patch,
            };

            let checksum = metadata.get("checksum")
                .and_then(|v| if let JsonValue::String(s) = v { Some(s.clone()) } else { None })
                .unwrap_or_else(|| "unknown".to_string());

            return Ok(DocumentVersion {
                version: version.to_string(),
                document_id: document_id.to_string(),
                created_at,
                created_by,
                change_type,
                change_description,
                checksum,
            });
        }

        Err(QmsError::validation_error("Invalid version file format"))
    }

    /// Helper function to create version file content
    fn create_version_file_content(
        document: &Document,
        version_entry: &DocumentVersion,
    ) -> QmsResult<String> {
        let mut root = HashMap::new();
        
        // Add document data
        let doc_json = document.to_json();
        let doc_value = JsonValue::parse(&doc_json)?;
        root.insert("document".to_string(), doc_value);

        // Add version metadata
        let mut metadata = HashMap::new();
        metadata.insert("created_at".to_string(), JsonValue::Number(version_entry.created_at as f64));
        metadata.insert("created_by".to_string(), JsonValue::String(version_entry.created_by.clone()));
        metadata.insert("change_description".to_string(), JsonValue::String(version_entry.change_description.clone()));
        metadata.insert("checksum".to_string(), JsonValue::String(version_entry.checksum.clone()));
        
        let change_type_str = match version_entry.change_type {
            VersionChangeType::Major => "Major",
            VersionChangeType::Minor => "Minor",
            VersionChangeType::Patch => "Patch",
        };
        metadata.insert("change_type".to_string(), JsonValue::String(change_type_str.to_string()));

        root.insert("metadata".to_string(), JsonValue::Object(metadata));

        // Convert to JSON string
        let root_value = JsonValue::Object(root);
        Ok(root_value.json_to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment_version_patch() {
        let result = DocumentVersionControl::increment_version("1.0.0", VersionChangeType::Patch);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1.0.1");
    }

    #[test]
    fn test_increment_version_minor() {
        let result = DocumentVersionControl::increment_version("1.0.5", VersionChangeType::Minor);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1.1.0");
    }

    #[test]
    fn test_increment_version_major() {
        let result = DocumentVersionControl::increment_version("1.5.3", VersionChangeType::Major);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "2.0.0");
    }

    #[test]
    fn test_increment_version_invalid() {
        let result = DocumentVersionControl::increment_version("1.0", VersionChangeType::Patch);
        assert!(result.is_err());
    }

    #[test]
    fn test_compare_versions() {
        use std::cmp::Ordering;
        
        assert_eq!(DocumentVersionControl::compare_versions("1.0.0", "1.0.1"), Ordering::Less);
        assert_eq!(DocumentVersionControl::compare_versions("1.1.0", "1.0.9"), Ordering::Greater);
        assert_eq!(DocumentVersionControl::compare_versions("2.0.0", "1.9.9"), Ordering::Greater);
        assert_eq!(DocumentVersionControl::compare_versions("1.0.0", "1.0.0"), Ordering::Equal);
    }

    #[test]
    fn test_is_valid_semantic_version() {
        assert!(DocumentVersionControl::is_valid_semantic_version("1.0.0"));
        assert!(DocumentVersionControl::is_valid_semantic_version("10.5.3"));
        assert!(!DocumentVersionControl::is_valid_semantic_version("1.0"));
        assert!(!DocumentVersionControl::is_valid_semantic_version("1.0.0.1"));
        assert!(!DocumentVersionControl::is_valid_semantic_version("1.a.0"));
    }

    #[test]
    fn test_version_change_type() {
        // Test that version change types are handled correctly
        let patch_result = DocumentVersionControl::increment_version("1.0.0", VersionChangeType::Patch);
        let minor_result = DocumentVersionControl::increment_version("1.0.0", VersionChangeType::Minor);
        let major_result = DocumentVersionControl::increment_version("1.0.0", VersionChangeType::Major);
        
        assert_eq!(patch_result.unwrap(), "1.0.1");
        assert_eq!(minor_result.unwrap(), "1.1.0");
        assert_eq!(major_result.unwrap(), "2.0.0");
    }
}
