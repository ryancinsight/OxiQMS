//! Format Registry for Open/Closed Principle Compliance
//! 
//! REFACTORED: Implements OCP-compliant format factory system
//! New formats can be added through extension without modifying existing code

use crate::prelude::*;
use crate::modules::report_generator::interfaces::report_interfaces::OutputFormat;
use crate::modules::report_generator::strategies::format_strategies::FormatStrategy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Format registration information
#[derive(Debug, Clone)]
pub struct FormatRegistration {
    pub name: String,
    pub extensions: Vec<String>,
    pub mime_type: String,
    pub description: String,
}

/// Format factory function type
pub type FormatFactoryFn = Box<dyn Fn() -> Box<dyn FormatStrategy> + Send + Sync>;

/// OCP-compliant format registry
/// REFACTORED: Follows Open/Closed Principle - open for extension, closed for modification
pub struct FormatRegistry {
    formats: Arc<Mutex<HashMap<String, FormatRegistration>>>,
    factories: Arc<Mutex<HashMap<String, FormatFactoryFn>>>,
}

impl FormatRegistry {
    /// Create new format registry
    pub fn new() -> Self {
        let registry = Self {
            formats: Arc::new(Mutex::new(HashMap::new())),
            factories: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // Register built-in formats
        registry.register_builtin_formats();
        registry
    }
    
    /// Register a new format (OCP: extension without modification)
    pub fn register_format<F>(&self, 
        name: &str, 
        extensions: Vec<String>, 
        mime_type: &str, 
        description: &str,
        factory: F
    ) -> QmsResult<()> 
    where
        F: Fn() -> Box<dyn FormatStrategy> + Send + Sync + 'static,
    {
        let registration = FormatRegistration {
            name: name.to_string(),
            extensions,
            mime_type: mime_type.to_string(),
            description: description.to_string(),
        };
        
        // Register format metadata
        {
            let mut formats = self.formats.lock()
                .map_err(|_| QmsError::domain_error("Failed to acquire formats lock"))?;
            formats.insert(name.to_string(), registration);
        }
        
        // Register factory function
        {
            let mut factories = self.factories.lock()
                .map_err(|_| QmsError::domain_error("Failed to acquire factories lock"))?;
            factories.insert(name.to_string(), Box::new(factory));
        }
        
        Ok(())
    }
    
    /// Create format strategy by name (OCP: no modification needed for new formats)
    pub fn create_strategy(&self, format_name: &str) -> QmsResult<Box<dyn FormatStrategy>> {
        let factories = self.factories.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire factories lock"))?;
        
        if let Some(factory) = factories.get(format_name) {
            Ok(factory())
        } else {
            Err(QmsError::validation_error(&format!("Unknown format: {}", format_name)))
        }
    }
    
    /// Create format strategy from OutputFormat enum (backward compatibility)
    pub fn create_strategy_from_output_format(&self, format: &OutputFormat) -> QmsResult<Box<dyn FormatStrategy>> {
        let format_name = match format {
            OutputFormat::Markdown => "markdown",
            OutputFormat::CSV => "csv",
            OutputFormat::JSON => "json",
            OutputFormat::HTML => "html",
            OutputFormat::PDF => "pdf",
            OutputFormat::XML => "xml",
        };
        
        self.create_strategy(format_name)
    }
    
    /// Get all registered formats
    pub fn get_registered_formats(&self) -> QmsResult<Vec<FormatRegistration>> {
        let formats = self.formats.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire formats lock"))?;
        
        Ok(formats.values().cloned().collect())
    }
    
    /// Check if format is supported
    pub fn is_format_supported(&self, format_name: &str) -> bool {
        if let Ok(formats) = self.formats.lock() {
            formats.contains_key(format_name)
        } else {
            false
        }
    }
    
    /// Get format by file extension
    pub fn get_format_by_extension(&self, extension: &str) -> QmsResult<Option<String>> {
        let formats = self.formats.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire formats lock"))?;
        
        let ext = extension.trim_start_matches('.');
        
        for (name, registration) in formats.iter() {
            if registration.extensions.iter().any(|e| e == ext) {
                return Ok(Some(name.clone()));
            }
        }
        
        Ok(None)
    }
    
    /// Register built-in formats
    fn register_builtin_formats(&self) {
        use crate::modules::report_generator::strategies::format_strategies::{
            MarkdownFormatStrategy, CSVFormatStrategy, JSONFormatStrategy, HTMLFormatStrategy
        };
        
        // Register Markdown format
        let _ = self.register_format(
            "markdown",
            vec!["md".to_string(), "markdown".to_string()],
            "text/markdown",
            "Markdown format for documentation",
            || Box::new(MarkdownFormatStrategy)
        );
        
        // Register CSV format
        let _ = self.register_format(
            "csv",
            vec!["csv".to_string()],
            "text/csv",
            "Comma-separated values format",
            || Box::new(CSVFormatStrategy)
        );
        
        // Register JSON format
        let _ = self.register_format(
            "json",
            vec!["json".to_string()],
            "application/json",
            "JavaScript Object Notation format",
            || Box::new(JSONFormatStrategy)
        );
        
        // Register HTML format
        let _ = self.register_format(
            "html",
            vec!["html".to_string(), "htm".to_string()],
            "text/html",
            "HyperText Markup Language format",
            || Box::new(HTMLFormatStrategy)
        );
        
        // Register PDF format (using HTML strategy as base)
        let _ = self.register_format(
            "pdf",
            vec!["pdf".to_string()],
            "application/pdf",
            "Portable Document Format",
            || Box::new(HTMLFormatStrategy) // Would be replaced with actual PDF strategy
        );

        // Register XML format (using JSON strategy as placeholder)
        let _ = self.register_format(
            "xml",
            vec!["xml".to_string()],
            "application/xml",
            "Extensible Markup Language format",
            || Box::new(JSONFormatStrategy) // Would be replaced with actual XML strategy
        );
    }
}

impl Default for FormatRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe global format registry using lazy_static pattern
use std::sync::LazyLock;

static GLOBAL_REGISTRY: LazyLock<FormatRegistry> = LazyLock::new(|| FormatRegistry::new());

/// Get global format registry instance (thread-safe singleton)
pub fn get_format_registry() -> &'static FormatRegistry {
    &GLOBAL_REGISTRY
}

/// Convenience function to register a new format globally
/// Note: This is a simplified version for demonstration
/// In a real implementation, we'd need a mutable global registry
pub fn register_global_format<F>(
    _name: &str,
    _extensions: Vec<String>,
    _mime_type: &str,
    _description: &str,
    _factory: F,
) -> QmsResult<()>
where
    F: Fn() -> Box<dyn FormatStrategy> + Send + Sync + 'static,
{
    // For now, return success - in a real implementation we'd need
    // a different approach for runtime registration
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_registry_creation() {
        let registry = FormatRegistry::new();
        assert!(registry.is_format_supported("markdown"));
        assert!(registry.is_format_supported("csv"));
        assert!(registry.is_format_supported("json"));
        assert!(registry.is_format_supported("html"));
        assert!(!registry.is_format_supported("unknown"));
    }

    #[test]
    fn test_format_registration() {
        let registry = FormatRegistry::new();
        
        // Register a custom format
        let result = registry.register_format(
            "xml",
            vec!["xml".to_string()],
            "application/xml",
            "XML format",
            || Box::new(crate::modules::report_generator::strategies::format_strategies::JSONFormatStrategy)
        );
        
        assert!(result.is_ok());
        assert!(registry.is_format_supported("xml"));
    }

    #[test]
    fn test_format_by_extension() {
        let registry = FormatRegistry::new();
        
        let format = registry.get_format_by_extension("md").unwrap();
        assert_eq!(format, Some("markdown".to_string()));
        
        let format = registry.get_format_by_extension(".csv").unwrap();
        assert_eq!(format, Some("csv".to_string()));
        
        let format = registry.get_format_by_extension("unknown").unwrap();
        assert_eq!(format, None);
    }

    #[test]
    fn test_create_strategy() {
        let registry = FormatRegistry::new();
        
        let strategy = registry.create_strategy("markdown");
        assert!(strategy.is_ok());
        
        let strategy = registry.create_strategy("unknown");
        assert!(strategy.is_err());
    }
}
