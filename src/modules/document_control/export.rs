//! Document export functionality for QMS
//! Phase 2.1.10 - Document Export
//! 
//! Supports export to multiple formats:
//! - JSON: Full document metadata with content and history
//! - Markdown: Clean markdown with metadata header
//! - HTML: HTML format with CSS styling
//! - PDF: Text-based report format (stdlib-only limitation)

use crate::error::{QmsError, QmsResult};
use crate::modules::document_control::document::Document;
use crate::modules::document_control::service::DocumentService;
use crate::modules::document_control::version::DocumentVersion;
use std::fs;
use std::path::Path;

/// Export format options
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Markdown,
    Html,
    Pdf,
}

impl ExportFormat {
    /// Parse export format from string
    pub fn from_str(s: &str) -> QmsResult<Self> {
        match s.to_lowercase().as_str() {
            "json" => Ok(ExportFormat::Json),
            "md" | "markdown" => Ok(ExportFormat::Markdown),
            "html" | "htm" => Ok(ExportFormat::Html),
            "pdf" => Ok(ExportFormat::Pdf),
            _ => Err(QmsError::validation_error(&format!("Unsupported export format: {s}"))),
        }
    }

    /// Get file extension for format
    #[allow(dead_code)] // May be used in future features
    pub const fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Json => "json",
            ExportFormat::Markdown => "md",
            ExportFormat::Html => "html",
            ExportFormat::Pdf => "pdf",
        }
    }
}

/// Export options for controlling output content
#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub include_history: bool,
    pub include_audit: bool,
    pub include_metadata: bool,
    pub include_regulatory_mapping: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_history: false,
            include_audit: false,
            include_metadata: true,
            include_regulatory_mapping: true,
        }
    }
}

/// Document exporter service
pub struct DocumentExporter;

impl DocumentExporter {
    /// Export document to specified format and file
    pub fn export_document(
        project_path: &Path,
        doc_id: &str,
        format: ExportFormat,
        output_path: &Path,
        options: ExportOptions,
    ) -> QmsResult<()> {
        let doc_service = DocumentService::new(project_path.to_path_buf());
        let document = doc_service.read_document(doc_id)?;

        let content = match format {
            ExportFormat::Json => Self::export_to_json(&document, project_path, &options)?,
            ExportFormat::Markdown => Self::export_to_markdown(&document, project_path, &options)?,
            ExportFormat::Html => Self::export_to_html(&document, project_path, &options)?,
            ExportFormat::Pdf => Self::export_to_pdf(&document, project_path, &options)?,
        };

        // Write to output file
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(output_path, content)?;

        // Log the export action
        crate::audit::log_audit(&format!(
            "DOCUMENT_EXPORT: {} exported to {} format at {}",
            doc_id,
            match format {
                ExportFormat::Json => "JSON",
                ExportFormat::Markdown => "Markdown", 
                ExportFormat::Html => "HTML",
                ExportFormat::Pdf => "PDF",
            },
            output_path.display()
        ));

        Ok(())
    }

    /// Export document to JSON format
    fn export_to_json(
        document: &Document,
        _project_path: &Path,
        options: &ExportOptions,
    ) -> QmsResult<String> {
        let mut export_data = std::collections::HashMap::new();

        // Document metadata
        if options.include_metadata {
            export_data.insert("document".to_string(), Self::document_to_json(document)?);
        }

        // Version history
        if options.include_history {
            // For now, we'll include a placeholder for version history
            // Full version history will be implemented when VersionService is available
            export_data.insert("version_history".to_string(), crate::json_utils::JsonValue::Array(vec![]));
        }

        // Audit trail
        if options.include_audit {
            export_data.insert("audit_trail".to_string(), Self::get_audit_trail_json(&document.id)?);
        }

        // Export metadata
        export_data.insert("export_metadata".to_string(), Self::export_metadata_to_json()?);

        let export_json = crate::json_utils::JsonValue::Object(export_data);
        Ok(export_json.json_to_string())
    }

    /// Export document to Markdown format
    fn export_to_markdown(
        document: &Document,
        _project_path: &Path,
        options: &ExportOptions,
    ) -> QmsResult<String> {
        let mut content = String::new();

        // Metadata header
        if options.include_metadata {
            content.push_str("---\n");
            content.push_str(&format!("title: {}\n", document.title));
            content.push_str(&format!("id: {}\n", document.id));
            content.push_str(&format!("version: {}\n", document.version));
            content.push_str(&format!("status: {:?}\n", document.status));
            content.push_str(&format!("created: {}\n", document.created_at));
            content.push_str(&format!("updated: {}\n", document.updated_at));
            content.push_str(&format!("author: {}\n", document.created_by));
            content.push_str(&format!("type: {:?}\n", document.doc_type));
            if let Some(approved_by) = &document.approved_by {
                content.push_str(&format!("approved_by: {approved_by}\n"));
            }
            content.push_str("---\n\n");
        }

        // Document content
        content.push_str(&document.content);

        // Version history
        if options.include_history {
            content.push_str("\n\n---\n\n# Version History\n\n");
            // For now, show current version only
            content.push_str(&format!("## Version {}\n", document.version));
            content.push_str(&format!("- **Date**: {}\n", document.updated_at));
            content.push_str("- **Changes**: Version history will be available in future releases\n");
            content.push('\n');
        }

        // Regulatory mapping
        if options.include_regulatory_mapping && !document.regulatory_mapping.is_empty() {
            content.push_str("\n\n---\n\n# Regulatory Mapping\n\n");
            for mapping in &document.regulatory_mapping {
                content.push_str(&format!("- **{}**: {} - {}\n", mapping.standard, mapping.section, mapping.requirement));
            }
        }

        Ok(content)
    }

    /// Export document to HTML format
    fn export_to_html(
        document: &Document,
        _project_path: &Path,
        options: &ExportOptions,
    ) -> QmsResult<String> {
        let mut html = String::new();

        // HTML header
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html lang=\"en\">\n");
        html.push_str("<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str(&format!("    <title>{}</title>\n", Self::html_escape(&document.title)));
        html.push_str("    <style>\n");
        html.push_str(Self::get_html_styles());
        html.push_str("    </style>\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");

        // Document header
        html.push_str("    <div class=\"document-header\">\n");
        html.push_str(&format!("        <h1>{}</h1>\n", Self::html_escape(&document.title)));
        
        // Metadata table
        if options.include_metadata {
            html.push_str("        <table class=\"metadata-table\">\n");
            html.push_str(&format!("            <tr><td><strong>Document ID:</strong></td><td>{}</td></tr>\n", Self::html_escape(&document.id)));
            html.push_str(&format!("            <tr><td><strong>Version:</strong></td><td>{}</td></tr>\n", Self::html_escape(&document.version)));
            html.push_str(&format!("            <tr><td><strong>Status:</strong></td><td>{:?}</td></tr>\n", document.status));
            html.push_str(&format!("            <tr><td><strong>Type:</strong></td><td>{:?}</td></tr>\n", document.doc_type));
            html.push_str(&format!("            <tr><td><strong>Created:</strong></td><td>{}</td></tr>\n", document.created_at));
            html.push_str(&format!("            <tr><td><strong>Updated:</strong></td><td>{}</td></tr>\n", document.updated_at));
            html.push_str(&format!("            <tr><td><strong>Author:</strong></td><td>{}</td></tr>\n", Self::html_escape(&document.created_by)));
            if let Some(approved_by) = &document.approved_by {
                html.push_str(&format!("            <tr><td><strong>Approved By:</strong></td><td>{}</td></tr>\n", Self::html_escape(approved_by)));
            }
            html.push_str("        </table>\n");
        }
        html.push_str("    </div>\n\n");

        // Document content
        html.push_str("    <div class=\"document-content\">\n");
        html.push_str(&Self::markdown_to_html(&document.content));
        html.push_str("    </div>\n");

        // Version history
        if options.include_history {
            html.push_str("\n    <div class=\"version-history\">\n");
            html.push_str("        <h2>Version History</h2>\n");
            html.push_str("        <table class=\"version-table\">\n");
            html.push_str("            <thead>\n");
            html.push_str("                <tr><th>Version</th><th>Date</th><th>Changes</th></tr>\n");
            html.push_str("            </thead>\n");
            html.push_str("            <tbody>\n");
            html.push_str("                <tr>\n");
            html.push_str(&format!("                    <td>{}</td>\n", Self::html_escape(&document.version)));
            html.push_str(&format!("                    <td>{}</td>\n", Self::html_escape(&document.updated_at)));
            html.push_str("                    <td>Current version</td>\n");
            html.push_str("                </tr>\n");
            html.push_str("            </tbody>\n");
            html.push_str("        </table>\n");
            html.push_str("    </div>\n");
        }

        // Regulatory mapping
        if options.include_regulatory_mapping && !document.regulatory_mapping.is_empty() {
            html.push_str("\n    <div class=\"regulatory-mapping\">\n");
            html.push_str("        <h2>Regulatory Mapping</h2>\n");
            html.push_str("        <table class=\"regulatory-table\">\n");
            html.push_str("            <thead>\n");
            html.push_str("                <tr><th>Standard</th><th>Section</th><th>Requirement</th></tr>\n");
            html.push_str("            </thead>\n");
            html.push_str("            <tbody>\n");
            for mapping in &document.regulatory_mapping {
                html.push_str("                <tr>\n");
                html.push_str(&format!("                    <td>{}</td>\n", Self::html_escape(&mapping.standard)));
                html.push_str(&format!("                    <td>{}</td>\n", Self::html_escape(&mapping.section)));
                html.push_str(&format!("                    <td>{}</td>\n", Self::html_escape(&mapping.requirement)));
                html.push_str("                </tr>\n");
            }
            html.push_str("            </tbody>\n");
            html.push_str("        </table>\n");
            html.push_str("    </div>\n");
        }

        // HTML footer
        html.push_str("\n    <div class=\"document-footer\">\n");
        html.push_str(&format!("        <p><em>Exported on {}</em></p>\n", crate::utils::current_date_string()));
        html.push_str("    </div>\n");
        html.push_str("</body>\n");
        html.push_str("</html>\n");

        Ok(html)
    }

    /// Export document to PDF-like text format
    fn export_to_pdf(
        document: &Document,
        _project_path: &Path,
        options: &ExportOptions,
    ) -> QmsResult<String> {
        let mut content = String::new();
        let line_width = 80;

        // Header
        content.push_str(&"=".repeat(line_width));
        content.push('\n');
        content.push_str(&Self::center_text(&document.title, line_width));
        content.push('\n');
        content.push_str(&"=".repeat(line_width));
        content.push('\n');
        content.push('\n');

        // Metadata section
        if options.include_metadata {
            content.push_str("DOCUMENT INFORMATION\n");
            content.push_str(&"-".repeat(line_width));
            content.push('\n');
            content.push_str(&format!("Document ID:       {}\n", document.id));
            content.push_str(&format!("Version:           {}\n", document.version));
            content.push_str(&format!("Status:            {:?}\n", document.status));
            content.push_str(&format!("Type:              {:?}\n", document.doc_type));
            content.push_str(&format!("Created:           {}\n", document.created_at));
            content.push_str(&format!("Updated:           {}\n", document.updated_at));
            content.push_str(&format!("Author:            {}\n", document.created_by));
            if let Some(approved_by) = &document.approved_by {
                content.push_str(&format!("Approved By:       {approved_by}\n"));
            }
            content.push_str(&format!("Checksum:          {}\n", document.checksum));
            content.push('\n');
            content.push('\n');
        }

        // Document content
        content.push_str("DOCUMENT CONTENT\n");
        content.push_str(&"-".repeat(line_width));
        content.push('\n');
        content.push('\n');
        
        // Format content for text display
        let formatted_content = Self::format_text_content(&document.content, line_width);
        content.push_str(&formatted_content);
        content.push('\n');
        content.push('\n');

        // Version history
        if options.include_history {
            content.push_str("VERSION HISTORY\n");
            content.push_str(&"-".repeat(line_width));
            content.push('\n');
            content.push_str(&format!("Version {}:\n", document.version));
            content.push_str(&format!("  Date: {}\n", document.updated_at));
            content.push_str("  Changes: Current version\n");
            content.push('\n');
        }

        // Regulatory mapping
        if options.include_regulatory_mapping && !document.regulatory_mapping.is_empty() {
            content.push_str("REGULATORY MAPPING\n");
            content.push_str(&"-".repeat(line_width));
            content.push('\n');
            for mapping in &document.regulatory_mapping {
                content.push_str(&format!("Standard: {}\n", mapping.standard));
                content.push_str(&format!("Section:  {}\n", mapping.section));
                content.push_str(&format!("Requirement: {}\n", mapping.requirement));
                content.push('\n');
            }
        }

        // Footer
        content.push_str(&"=".repeat(line_width));
        content.push('\n');
        content.push_str(&Self::center_text(&format!("Generated on {}", crate::utils::current_date_string()), line_width));
        content.push('\n');
        content.push_str(&"=".repeat(line_width));
        content.push('\n');

        Ok(content)
    }

    /// Convert document to JSON value
    fn document_to_json(document: &Document) -> QmsResult<crate::json_utils::JsonValue> {
        let mut doc_obj = std::collections::HashMap::new();
        
        doc_obj.insert("id".to_string(), crate::json_utils::JsonValue::String(document.id.clone()));
        doc_obj.insert("project_id".to_string(), crate::json_utils::JsonValue::String(document.project_id.clone()));
        doc_obj.insert("title".to_string(), crate::json_utils::JsonValue::String(document.title.clone()));
        doc_obj.insert("content".to_string(), crate::json_utils::JsonValue::String(document.content.clone()));
        doc_obj.insert("doc_type".to_string(), crate::json_utils::JsonValue::String(format!("{:?}", document.doc_type)));
        doc_obj.insert("version".to_string(), crate::json_utils::JsonValue::String(document.version.clone()));
        doc_obj.insert("status".to_string(), crate::json_utils::JsonValue::String(format!("{:?}", document.status)));
        doc_obj.insert("created_at".to_string(), crate::json_utils::JsonValue::String(document.created_at.clone()));
        doc_obj.insert("updated_at".to_string(), crate::json_utils::JsonValue::String(document.updated_at.clone()));
        doc_obj.insert("created_by".to_string(), crate::json_utils::JsonValue::String(document.created_by.clone()));
        
        if let Some(approved_by) = &document.approved_by {
            doc_obj.insert("approved_by".to_string(), crate::json_utils::JsonValue::String(approved_by.clone()));
        } else {
            doc_obj.insert("approved_by".to_string(), crate::json_utils::JsonValue::Null);
        }
        
        doc_obj.insert("file_path".to_string(), crate::json_utils::JsonValue::String(document.file_path.clone()));
        doc_obj.insert("checksum".to_string(), crate::json_utils::JsonValue::String(document.checksum.clone()));

        // Tags array
        let tags_array = document.tags.iter()
            .map(|tag| crate::json_utils::JsonValue::String(tag.clone()))
            .collect();
        doc_obj.insert("tags".to_string(), crate::json_utils::JsonValue::Array(tags_array));

        // Regulatory mapping array
        let regulatory_array = document.regulatory_mapping.iter()
            .map(|mapping| {
                let mut mapping_obj = std::collections::HashMap::new();
                mapping_obj.insert("standard".to_string(), crate::json_utils::JsonValue::String(mapping.standard.clone()));
                mapping_obj.insert("section".to_string(), crate::json_utils::JsonValue::String(mapping.section.clone()));
                mapping_obj.insert("requirement".to_string(), crate::json_utils::JsonValue::String(mapping.requirement.clone()));
                crate::json_utils::JsonValue::Object(mapping_obj)
            })
            .collect();
        doc_obj.insert("regulatory_mapping".to_string(), crate::json_utils::JsonValue::Array(regulatory_array));

        Ok(crate::json_utils::JsonValue::Object(doc_obj))
    }

    /// Convert version history to JSON (placeholder for future implementation)
    #[allow(dead_code)] // Will be used when version service is fully implemented
    const fn version_history_to_json(_history: &[DocumentVersion]) -> QmsResult<crate::json_utils::JsonValue> {
        // Placeholder implementation - will be fully implemented when version service is available
        Ok(crate::json_utils::JsonValue::Array(vec![]))
    }

    /// Get audit trail for document as JSON
    fn get_audit_trail_json(doc_id: &str) -> QmsResult<crate::json_utils::JsonValue> {
        // For now, return empty array - full audit trail integration will be in Phase 2.2
        let mut filter_criteria = std::collections::HashMap::new();
        filter_criteria.insert("entity_type".to_string(), "Document".to_string());
        filter_criteria.insert("entity_id".to_string(), doc_id.to_string());
        
        // Return placeholder audit entries
        Ok(crate::json_utils::JsonValue::Array(vec![]))
    }

    /// Create export metadata JSON
    fn export_metadata_to_json() -> QmsResult<crate::json_utils::JsonValue> {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("export_timestamp".to_string(), crate::json_utils::JsonValue::Number(crate::utils::current_timestamp() as f64));
        metadata.insert("export_tool".to_string(), crate::json_utils::JsonValue::String("QMS Document Exporter".to_string()));
        metadata.insert("export_version".to_string(), crate::json_utils::JsonValue::String("1.0".to_string()));
        metadata.insert("format_version".to_string(), crate::json_utils::JsonValue::String("1.0".to_string()));
        
        Ok(crate::json_utils::JsonValue::Object(metadata))
    }

    /// HTML escape text
    fn html_escape(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }

    /// Simple markdown to HTML conversion
    fn markdown_to_html(markdown: &str) -> String {
        let mut html = String::new();
        let lines: Vec<&str> = markdown.split('\n').collect();
        let mut in_code_block = false;
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();
            
            // Code blocks
            if line.starts_with("```") {
                if in_code_block {
                    html.push_str("        </code></pre>\n");
                    in_code_block = false;
                } else {
                    html.push_str("        <pre><code>\n");
                    in_code_block = true;
                }
                i += 1;
                continue;
            }

            if in_code_block {
                html.push_str("            ");
                html.push_str(&Self::html_escape(line));
                html.push('\n');
                i += 1;
                continue;
            }

            // Headers
            if line.starts_with("# ") {
                html.push_str(&format!("        <h1>{}</h1>\n", Self::html_escape(&line[2..])));
            } else if line.starts_with("## ") {
                html.push_str(&format!("        <h2>{}</h2>\n", Self::html_escape(&line[3..])));
            } else if line.starts_with("### ") {
                html.push_str(&format!("        <h3>{}</h3>\n", Self::html_escape(&line[4..])));
            } else if line.starts_with("#### ") {
                html.push_str(&format!("        <h4>{}</h4>\n", Self::html_escape(&line[5..])));
            } else if line.starts_with("- ") {
                // List items - simple implementation
                html.push_str(&format!("        <p>• {}</p>\n", Self::html_escape(&line[2..])));
            } else if line.is_empty() {
                html.push_str("        <br>\n");
            } else {
                // Regular paragraph
                html.push_str(&format!("        <p>{}</p>\n", Self::html_escape(line)));
            }

            i += 1;
        }

        html
    }

    /// Get HTML styles for document export
    const fn get_html_styles() -> &'static str {
        r#"
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f9f9f9;
        }
        .document-header {
            background-color: #fff;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            margin-bottom: 30px;
        }
        .document-header h1 {
            color: #2c3e50;
            margin-bottom: 20px;
            border-bottom: 2px solid #3498db;
            padding-bottom: 10px;
        }
        .metadata-table {
            width: 100%;
            border-collapse: collapse;
            margin-top: 20px;
        }
        .metadata-table td {
            padding: 8px 12px;
            border-bottom: 1px solid #eee;
        }
        .metadata-table td:first-child {
            width: 30%;
            background-color: #f8f9fa;
        }
        .document-content {
            background-color: #fff;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            margin-bottom: 30px;
        }
        .document-content h1, .document-content h2, .document-content h3, .document-content h4 {
            color: #2c3e50;
            margin-top: 30px;
            margin-bottom: 15px;
        }
        .document-content h1 {
            border-bottom: 2px solid #3498db;
            padding-bottom: 10px;
        }
        .document-content h2 {
            border-bottom: 1px solid #bdc3c7;
            padding-bottom: 5px;
        }
        .document-content p {
            margin-bottom: 15px;
            text-align: justify;
        }
        .document-content pre {
            background-color: #f8f9fa;
            border: 1px solid #e9ecef;
            border-radius: 4px;
            padding: 15px;
            overflow-x: auto;
        }
        .version-history, .regulatory-mapping {
            background-color: #fff;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            margin-bottom: 30px;
        }
        .version-history h2, .regulatory-mapping h2 {
            color: #2c3e50;
            margin-bottom: 20px;
            border-bottom: 2px solid #3498db;
            padding-bottom: 10px;
        }
        .version-table, .regulatory-table {
            width: 100%;
            border-collapse: collapse;
            margin-top: 15px;
        }
        .version-table th, .version-table td,
        .regulatory-table th, .regulatory-table td {
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }
        .version-table th, .regulatory-table th {
            background-color: #f8f9fa;
            font-weight: bold;
            color: #2c3e50;
        }
        .document-footer {
            text-align: center;
            padding: 20px;
            color: #666;
            font-style: italic;
        }
        @media print {
            body { background-color: white; }
            .document-header, .document-content, .version-history, .regulatory-mapping {
                box-shadow: none;
                border: 1px solid #ddd;
            }
        }
        "#
    }

    /// Center text within given width
    fn center_text(text: &str, width: usize) -> String {
        if text.len() >= width {
            return text.to_string();
        }
        
        let padding = (width - text.len()) / 2;
        format!("{}{}", " ".repeat(padding), text)
    }

    /// Format text content for fixed-width display
    fn format_text_content(content: &str, width: usize) -> String {
        let mut formatted = String::new();
        
        for line in content.lines() {
            if line.len() <= width {
                formatted.push_str(line);
                formatted.push('\n');
            } else {
                // Simple word wrap
                let words: Vec<&str> = line.split_whitespace().collect();
                let mut current_line = String::new();
                
                for word in words {
                    if current_line.len() + word.len() < width {
                        if !current_line.is_empty() {
                            current_line.push(' ');
                        }
                        current_line.push_str(word);
                    } else {
                        if !current_line.is_empty() {
                            formatted.push_str(&current_line);
                            formatted.push('\n');
                        }
                        current_line = word.to_string();
                    }
                }
                
                if !current_line.is_empty() {
                    formatted.push_str(&current_line);
                    formatted.push('\n');
                }
            }
        }
        
        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::document_control::document::{DocumentType, DocumentStatus, RegulatoryReference};
    use std::env;

    fn create_test_dir() -> std::path::PathBuf {
        let mut temp_dir = env::temp_dir();
        temp_dir.push(format!("qms_export_test_{}", crate::utils::generate_uuid()));
        fs::create_dir_all(&temp_dir).unwrap();
        temp_dir
    }

    fn cleanup_test_dir(dir: &std::path::PathBuf) {
        if dir.exists() {
            fs::remove_dir_all(dir).unwrap_or_else(|e| {
                eprintln!("Warning: Failed to cleanup test directory: {}", e);
            });
        }
    }

    fn create_test_document() -> Document {
        Document {
            id: "DOC-20240115-001".to_string(),
            project_id: "test-project".to_string(),
            title: "Test Document".to_string(),
            content: "# Test Document\n\nThis is a test document with **bold** text and `code`.\n\n## Section 1\n\nSome content here.\n\n- Item 1\n- Item 2\n\n```\ncode block\n```".to_string(),
            doc_type: DocumentType::SoftwareRequirementsSpecification,
            version: "1.0.0".to_string(),
            status: DocumentStatus::Approved,
            created_at: "2022-01-01T00:00:00Z".to_string(),
            updated_at: "2022-01-01T00:05:00Z".to_string(),
            created_by: "test_user".to_string(),
            approved_by: Some("quality_engineer".to_string()),
            file_path: "docs/DOC-20240115-001/content.md".to_string(),
            checksum: "abc123def456".to_string(),
            tags: vec!["test".to_string(), "requirements".to_string()],
            regulatory_mapping: vec![
                RegulatoryReference {
                    standard: "21 CFR 820.30".to_string(),
                    section: "Design Controls".to_string(),
                    requirement: "Document control procedures".to_string(),
                }
            ],
            locked: false,
            locked_by: None,
            locked_at: None,
        }
    }

    #[test]
    fn test_export_format_parsing() {
        assert!(matches!(ExportFormat::from_str("json").unwrap(), ExportFormat::Json));
        assert!(matches!(ExportFormat::from_str("JSON").unwrap(), ExportFormat::Json));
        assert!(matches!(ExportFormat::from_str("markdown").unwrap(), ExportFormat::Markdown));
        assert!(matches!(ExportFormat::from_str("md").unwrap(), ExportFormat::Markdown));
        assert!(matches!(ExportFormat::from_str("html").unwrap(), ExportFormat::Html));
        assert!(matches!(ExportFormat::from_str("pdf").unwrap(), ExportFormat::Pdf));
        assert!(ExportFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_export_format_extensions() {
        assert_eq!(ExportFormat::Json.extension(), "json");
        assert_eq!(ExportFormat::Markdown.extension(), "md");
        assert_eq!(ExportFormat::Html.extension(), "html");
        assert_eq!(ExportFormat::Pdf.extension(), "pdf");
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(DocumentExporter::html_escape("Test & <tag>"), "Test &amp; &lt;tag&gt;");
        assert_eq!(DocumentExporter::html_escape("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(DocumentExporter::html_escape("'apostrophe'"), "&#39;apostrophe&#39;");
    }

    #[test]
    fn test_markdown_to_html() {
        let markdown = "# Header\n\nParagraph with **bold**.\n\n- List item\n\n```\ncode\n```";
        let html = DocumentExporter::markdown_to_html(markdown);
        
        assert!(html.contains("<h1>Header</h1>"));
        assert!(html.contains("<p>Paragraph with **bold**.</p>"));
        assert!(html.contains("<p>• List item</p>"));
        assert!(html.contains("<pre><code>"));
    }

    #[test]
    fn test_center_text() {
        assert_eq!(DocumentExporter::center_text("Test", 10), "   Test");
        assert_eq!(DocumentExporter::center_text("LongTextThatExceedsWidth", 10), "LongTextThatExceedsWidth");
        assert_eq!(DocumentExporter::center_text("Hello", 11), "   Hello");
    }

    #[test]
    fn test_format_text_content() {
        let content = "This is a very long line that should be wrapped at some point to fit within the specified width.";
        let formatted = DocumentExporter::format_text_content(content, 20);
        
        // Should be split into multiple lines
        assert!(formatted.lines().count() > 1);
        for line in formatted.lines() {
            assert!(line.len() <= 20);
        }
    }

    #[test]
    fn test_document_to_json() {
        let document = create_test_document();
        let json_value = DocumentExporter::document_to_json(&document).unwrap();
        
        if let crate::json_utils::JsonValue::Object(obj) = json_value {
            assert!(obj.contains_key("id"));
            assert!(obj.contains_key("title"));
            assert!(obj.contains_key("content"));
            assert!(obj.contains_key("version"));
            assert!(obj.contains_key("regulatory_mapping"));
        } else {
            panic!("Expected JSON object");
        }
    }

    #[test]
    fn test_export_to_json() {
        let document = create_test_document();
        let options = ExportOptions {
            include_metadata: true,
            include_history: false,
            include_audit: false,
            include_regulatory_mapping: true,
        };
        
        let test_dir = create_test_dir();
        let json_content = DocumentExporter::export_to_json(&document, &test_dir, &options).unwrap();
        
        assert!(json_content.contains("\"document\""));
        assert!(json_content.contains("\"export_metadata\""));
        assert!(json_content.contains("Test Document"));
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_export_to_markdown() {
        let document = create_test_document();
        let options = ExportOptions {
            include_metadata: true,
            include_history: false,
            include_audit: false,
            include_regulatory_mapping: true,
        };
        
        let test_dir = create_test_dir();
        let markdown_content = DocumentExporter::export_to_markdown(&document, &test_dir, &options).unwrap();
        
        assert!(markdown_content.contains("---"));
        assert!(markdown_content.contains("title: Test Document"));
        assert!(markdown_content.contains("version: 1.0.0"));
        assert!(markdown_content.contains("# Test Document"));
        assert!(markdown_content.contains("21 CFR 820.30"));
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_export_to_html() {
        let document = create_test_document();
        let options = ExportOptions {
            include_metadata: true,
            include_history: false,
            include_audit: false,
            include_regulatory_mapping: true,
        };
        
        let test_dir = create_test_dir();
        let html_content = DocumentExporter::export_to_html(&document, &test_dir, &options).unwrap();
        
        assert!(html_content.contains("<!DOCTYPE html>"));
        assert!(html_content.contains("<title>Test Document</title>"));
        assert!(html_content.contains("metadata-table"));
        assert!(html_content.contains("regulatory-table"));
        assert!(html_content.contains("</html>"));
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_export_to_pdf() {
        let document = create_test_document();
        let options = ExportOptions {
            include_metadata: true,
            include_history: false,
            include_audit: false,
            include_regulatory_mapping: true,
        };
        
        let test_dir = create_test_dir();
        let pdf_content = DocumentExporter::export_to_pdf(&document, &test_dir, &options).unwrap();
        
        assert!(pdf_content.contains("Test Document"));
        assert!(pdf_content.contains("DOCUMENT INFORMATION"));
        assert!(pdf_content.contains("DOCUMENT CONTENT"));
        assert!(pdf_content.contains("REGULATORY MAPPING"));
        assert!(pdf_content.contains("Document ID:       DOC-20240115-001"));
        
        cleanup_test_dir(&test_dir);
    }
}
