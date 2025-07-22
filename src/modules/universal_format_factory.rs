//! Universal Format Factory for Open/Closed Principle Compliance
//! 
//! REFACTORED: Consolidates all format handling across QMS modules
//! Eliminates the need to modify multiple format enums when adding new formats

use crate::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Universal format descriptor
#[derive(Debug, Clone)]
pub struct FormatDescriptor {
    pub name: String,
    pub extensions: Vec<String>,
    pub mime_type: String,
    pub description: String,
    pub category: FormatCategory,
}

/// Format categories for different use cases
#[derive(Debug, Clone, PartialEq)]
pub enum FormatCategory {
    Report,
    Export,
    Import,
    Document,
    Audit,
    Risk,
}

/// Format handler trait for different operations
pub trait FormatHandler: Send + Sync {
    /// Handle export operation
    fn handle_export(&self, data: &[u8], options: &FormatOptions) -> QmsResult<Vec<u8>>;
    
    /// Handle import operation  
    fn handle_import(&self, data: &[u8], options: &FormatOptions) -> QmsResult<Vec<u8>>;
    
    /// Validate format-specific data
    fn validate(&self, data: &[u8]) -> QmsResult<()>;
    
    /// Get format capabilities
    fn capabilities(&self) -> FormatCapabilities;
}

/// Format operation options
#[derive(Debug, Clone, Default)]
pub struct FormatOptions {
    pub encoding: Option<String>,
    pub compression: bool,
    pub metadata: HashMap<String, String>,
}

/// Format capabilities
#[derive(Debug, Clone)]
pub struct FormatCapabilities {
    pub supports_export: bool,
    pub supports_import: bool,
    pub supports_streaming: bool,
    pub supports_compression: bool,
    pub max_size_bytes: Option<usize>,
}

/// Universal format factory (OCP-compliant)
pub struct UniversalFormatFactory {
    descriptors: Arc<Mutex<HashMap<String, FormatDescriptor>>>,
    handlers: Arc<Mutex<HashMap<String, Box<dyn FormatHandler>>>>,
}

impl UniversalFormatFactory {
    /// Create new universal format factory
    pub fn new() -> Self {
        let factory = Self {
            descriptors: Arc::new(Mutex::new(HashMap::new())),
            handlers: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // Register built-in formats
        factory.register_builtin_formats();
        factory
    }
    
    /// Register a new format (OCP: extension without modification)
    pub fn register_format<H>(&self, descriptor: FormatDescriptor, handler: H) -> QmsResult<()>
    where
        H: FormatHandler + 'static,
    {
        let name = descriptor.name.clone();
        
        // Register descriptor
        {
            let mut descriptors = self.descriptors.lock()
                .map_err(|_| QmsError::domain_error("Failed to acquire descriptors lock"))?;
            descriptors.insert(name.clone(), descriptor);
        }
        
        // Register handler
        {
            let mut handlers = self.handlers.lock()
                .map_err(|_| QmsError::domain_error("Failed to acquire handlers lock"))?;
            handlers.insert(name, Box::new(handler));
        }
        
        Ok(())
    }
    
    /// Get format handler by name
    pub fn get_handler(&self, format_name: &str) -> QmsResult<&dyn FormatHandler> {
        // Note: This is a simplified implementation
        // In a real system, we'd need to handle the lifetime issues differently
        Err(QmsError::validation_error(&format!("Format not found: {}", format_name)))
    }
    
    /// Export data using specified format
    pub fn export_data(&self, format_name: &str, data: &[u8], options: &FormatOptions) -> QmsResult<Vec<u8>> {
        let handlers = self.handlers.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire handlers lock"))?;
        
        if let Some(handler) = handlers.get(format_name) {
            handler.handle_export(data, options)
        } else {
            Err(QmsError::validation_error(&format!("Unknown format: {}", format_name)))
        }
    }
    
    /// Import data using specified format
    pub fn import_data(&self, format_name: &str, data: &[u8], options: &FormatOptions) -> QmsResult<Vec<u8>> {
        let handlers = self.handlers.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire handlers lock"))?;
        
        if let Some(handler) = handlers.get(format_name) {
            handler.handle_import(data, options)
        } else {
            Err(QmsError::validation_error(&format!("Unknown format: {}", format_name)))
        }
    }
    
    /// Get format by file extension
    pub fn get_format_by_extension(&self, extension: &str) -> QmsResult<Option<FormatDescriptor>> {
        let descriptors = self.descriptors.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire descriptors lock"))?;
        
        let ext = extension.trim_start_matches('.');
        
        for descriptor in descriptors.values() {
            if descriptor.extensions.iter().any(|e| e == ext) {
                return Ok(Some(descriptor.clone()));
            }
        }
        
        Ok(None)
    }
    
    /// Get all formats by category
    pub fn get_formats_by_category(&self, category: &FormatCategory) -> QmsResult<Vec<FormatDescriptor>> {
        let descriptors = self.descriptors.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire descriptors lock"))?;
        
        Ok(descriptors.values()
            .filter(|d| &d.category == category)
            .cloned()
            .collect())
    }
    
    /// Check if format is supported
    pub fn is_format_supported(&self, format_name: &str) -> bool {
        if let Ok(descriptors) = self.descriptors.lock() {
            descriptors.contains_key(format_name)
        } else {
            false
        }
    }
    
    /// Register built-in formats
    fn register_builtin_formats(&self) {
        // CSV format
        let csv_descriptor = FormatDescriptor {
            name: "csv".to_string(),
            extensions: vec!["csv".to_string()],
            mime_type: "text/csv".to_string(),
            description: "Comma-separated values format".to_string(),
            category: FormatCategory::Export,
        };
        let _ = self.register_format(csv_descriptor, CsvFormatHandler);
        
        // JSON format
        let json_descriptor = FormatDescriptor {
            name: "json".to_string(),
            extensions: vec!["json".to_string()],
            mime_type: "application/json".to_string(),
            description: "JavaScript Object Notation format".to_string(),
            category: FormatCategory::Export,
        };
        let _ = self.register_format(json_descriptor, JsonFormatHandler);
        
        // HTML format
        let html_descriptor = FormatDescriptor {
            name: "html".to_string(),
            extensions: vec!["html".to_string(), "htm".to_string()],
            mime_type: "text/html".to_string(),
            description: "HyperText Markup Language format".to_string(),
            category: FormatCategory::Report,
        };
        let _ = self.register_format(html_descriptor, HtmlFormatHandler);
        
        // Markdown format
        let md_descriptor = FormatDescriptor {
            name: "markdown".to_string(),
            extensions: vec!["md".to_string(), "markdown".to_string()],
            mime_type: "text/markdown".to_string(),
            description: "Markdown format".to_string(),
            category: FormatCategory::Document,
        };
        let _ = self.register_format(md_descriptor, MarkdownFormatHandler);
    }
}

impl Default for UniversalFormatFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// CSV format handler implementation
pub struct CsvFormatHandler;

impl FormatHandler for CsvFormatHandler {
    fn handle_export(&self, data: &[u8], _options: &FormatOptions) -> QmsResult<Vec<u8>> {
        // Simple CSV export implementation
        let input = String::from_utf8_lossy(data);
        let csv_data = format!("data\n{}", input);
        Ok(csv_data.into_bytes())
    }
    
    fn handle_import(&self, data: &[u8], _options: &FormatOptions) -> QmsResult<Vec<u8>> {
        // Simple CSV import implementation
        let input = String::from_utf8_lossy(data);
        let lines: Vec<&str> = input.lines().skip(1).collect(); // Skip header
        let result = lines.join("\n");
        Ok(result.into_bytes())
    }
    
    fn validate(&self, data: &[u8]) -> QmsResult<()> {
        let input = String::from_utf8_lossy(data);
        if input.is_empty() {
            return Err(QmsError::validation_error("CSV data cannot be empty"));
        }
        Ok(())
    }
    
    fn capabilities(&self) -> FormatCapabilities {
        FormatCapabilities {
            supports_export: true,
            supports_import: true,
            supports_streaming: true,
            supports_compression: false,
            max_size_bytes: Some(100 * 1024 * 1024), // 100MB
        }
    }
}

/// JSON format handler implementation
pub struct JsonFormatHandler;

impl FormatHandler for JsonFormatHandler {
    fn handle_export(&self, data: &[u8], _options: &FormatOptions) -> QmsResult<Vec<u8>> {
        let input = String::from_utf8_lossy(data);
        let json_data = format!("{{\"data\": \"{}\"}}", input.replace('"', "\\\""));
        Ok(json_data.into_bytes())
    }
    
    fn handle_import(&self, data: &[u8], _options: &FormatOptions) -> QmsResult<Vec<u8>> {
        let input = String::from_utf8_lossy(data);
        // Simple JSON parsing (in real implementation, use proper JSON parser)
        if input.starts_with('{') && input.ends_with('}') {
            Ok(input.to_string().into_bytes())
        } else {
            Err(QmsError::validation_error("Invalid JSON format"))
        }
    }
    
    fn validate(&self, data: &[u8]) -> QmsResult<()> {
        let input = String::from_utf8_lossy(data);
        if !input.starts_with('{') || !input.ends_with('}') {
            return Err(QmsError::validation_error("Invalid JSON format"));
        }
        Ok(())
    }
    
    fn capabilities(&self) -> FormatCapabilities {
        FormatCapabilities {
            supports_export: true,
            supports_import: true,
            supports_streaming: false,
            supports_compression: true,
            max_size_bytes: Some(50 * 1024 * 1024), // 50MB
        }
    }
}

/// HTML format handler implementation
pub struct HtmlFormatHandler;

impl FormatHandler for HtmlFormatHandler {
    fn handle_export(&self, data: &[u8], _options: &FormatOptions) -> QmsResult<Vec<u8>> {
        let input = String::from_utf8_lossy(data);
        let html_data = format!("<html><body><pre>{}</pre></body></html>", 
                               input.replace('<', "&lt;").replace('>', "&gt;"));
        Ok(html_data.into_bytes())
    }
    
    fn handle_import(&self, _data: &[u8], _options: &FormatOptions) -> QmsResult<Vec<u8>> {
        Err(QmsError::validation_error("HTML import not supported"))
    }
    
    fn validate(&self, data: &[u8]) -> QmsResult<()> {
        let input = String::from_utf8_lossy(data);
        if input.is_empty() {
            return Err(QmsError::validation_error("HTML data cannot be empty"));
        }
        Ok(())
    }
    
    fn capabilities(&self) -> FormatCapabilities {
        FormatCapabilities {
            supports_export: true,
            supports_import: false,
            supports_streaming: false,
            supports_compression: true,
            max_size_bytes: None,
        }
    }
}

/// Markdown format handler implementation
pub struct MarkdownFormatHandler;

impl FormatHandler for MarkdownFormatHandler {
    fn handle_export(&self, data: &[u8], _options: &FormatOptions) -> QmsResult<Vec<u8>> {
        let input = String::from_utf8_lossy(data);
        let md_data = format!("# Report\n\n{}", input);
        Ok(md_data.into_bytes())
    }
    
    fn handle_import(&self, data: &[u8], _options: &FormatOptions) -> QmsResult<Vec<u8>> {
        // Simple markdown import (strip headers)
        let input = String::from_utf8_lossy(data);
        let content = input.lines()
            .filter(|line| !line.starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");
        Ok(content.into_bytes())
    }
    
    fn validate(&self, data: &[u8]) -> QmsResult<()> {
        let input = String::from_utf8_lossy(data);
        if input.is_empty() {
            return Err(QmsError::validation_error("Markdown data cannot be empty"));
        }
        Ok(())
    }
    
    fn capabilities(&self) -> FormatCapabilities {
        FormatCapabilities {
            supports_export: true,
            supports_import: true,
            supports_streaming: true,
            supports_compression: false,
            max_size_bytes: None,
        }
    }
}

/// Thread-safe global universal format factory using lazy_static pattern
use std::sync::LazyLock;

static GLOBAL_FACTORY: LazyLock<UniversalFormatFactory> = LazyLock::new(|| UniversalFormatFactory::new());

/// Get global universal format factory instance (thread-safe singleton)
pub fn get_universal_format_factory() -> &'static UniversalFormatFactory {
    &GLOBAL_FACTORY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universal_format_factory_creation() {
        let factory = UniversalFormatFactory::new();
        assert!(factory.is_format_supported("csv"));
        assert!(factory.is_format_supported("json"));
        assert!(factory.is_format_supported("html"));
        assert!(factory.is_format_supported("markdown"));
        assert!(!factory.is_format_supported("unknown"));
    }

    #[test]
    fn test_format_by_extension() {
        let factory = UniversalFormatFactory::new();
        
        let format = factory.get_format_by_extension("csv").unwrap();
        assert!(format.is_some());
        assert_eq!(format.unwrap().name, "csv");
        
        let format = factory.get_format_by_extension(".json").unwrap();
        assert!(format.is_some());
        assert_eq!(format.unwrap().name, "json");
    }

    #[test]
    fn test_formats_by_category() {
        let factory = UniversalFormatFactory::new();
        
        let export_formats = factory.get_formats_by_category(&FormatCategory::Export).unwrap();
        assert!(!export_formats.is_empty());
        
        let report_formats = factory.get_formats_by_category(&FormatCategory::Report).unwrap();
        assert!(!report_formats.is_empty());
    }

    #[test]
    fn test_csv_handler() {
        let handler = CsvFormatHandler;
        let data = b"test data";
        let options = FormatOptions::default();
        
        let result = handler.handle_export(data, &options);
        assert!(result.is_ok());
        
        let exported = result.unwrap();
        let exported_str = String::from_utf8_lossy(&exported);
        assert!(exported_str.contains("data\ntest data"));
    }
}
