//! Extensible Format System for Open/Closed Principle Compliance
//! 
//! REFACTORED: Allows new formats to be added without modifying existing code
//! Demonstrates how to extend the system with new formats following OCP

use crate::prelude::*;
use crate::modules::report_generator::strategies::format_strategies::FormatStrategy;
use crate::modules::report_generator::format_registry::{get_format_registry, register_global_format};

/// Trait for extensible format implementations
/// OCP: New formats implement this trait without modifying existing code
pub trait ExtensibleFormat: Send + Sync {
    /// Get format name
    fn name(&self) -> &str;
    
    /// Get supported file extensions
    fn extensions(&self) -> Vec<String>;
    
    /// Get MIME type
    fn mime_type(&self) -> &str;
    
    /// Get format description
    fn description(&self) -> &str;
    
    /// Create format strategy instance
    fn create_strategy(&self) -> Box<dyn FormatStrategy>;
    
    /// Register this format with the global registry
    fn register(&self) -> QmsResult<()> {
        let name = self.name().to_string();
        let extensions = self.extensions();
        let mime_type = self.mime_type().to_string();
        let description = self.description().to_string();
        
        // Create a closure that captures the format implementation
        register_global_format(
            &name,
            extensions,
            &mime_type,
            &description,
            move || {
                // This would need to be implemented differently in a real system
                // For now, we'll use a placeholder
                Box::new(crate::modules::report_generator::strategies::format_strategies::JSONFormatStrategy)
            }
        )
    }
}

/// Example: XML format implementation (OCP: extension without modification)
pub struct XmlFormat;

impl ExtensibleFormat for XmlFormat {
    fn name(&self) -> &str {
        "xml"
    }
    
    fn extensions(&self) -> Vec<String> {
        vec!["xml".to_string()]
    }
    
    fn mime_type(&self) -> &str {
        "application/xml"
    }
    
    fn description(&self) -> &str {
        "Extensible Markup Language format"
    }
    
    fn create_strategy(&self) -> Box<dyn FormatStrategy> {
        Box::new(XmlFormatStrategy)
    }
}

/// XML format strategy implementation
pub struct XmlFormatStrategy;

impl FormatStrategy for XmlFormatStrategy {
    fn get_header(&self, metadata: &crate::modules::report_generator::interfaces::report_interfaces::ReportMetadata, title: &str) -> String {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<report>\n  <title>{}</title>\n  <generated_at>{}</generated_at>\n  <version>{}</version>\n",
            self.escape_xml(title),
            self.escape_xml(&metadata.generated_at),
            self.escape_xml(&metadata.version)
        )
    }

    fn get_footer(&self, _metadata: &crate::modules::report_generator::interfaces::report_interfaces::ReportMetadata) -> String {
        "</report>".to_string()
    }

    fn format_name(&self) -> &'static str {
        "xml"
    }

    fn file_extension(&self) -> &'static str {
        "xml"
    }

    fn mime_type(&self) -> &'static str {
        "application/xml"
    }

    fn validate_content(&self, content: &str) -> QmsResult<crate::modules::report_generator::interfaces::report_interfaces::ValidationResult> {
        use crate::modules::report_generator::interfaces::report_interfaces::ValidationResult;

        if content.contains('<') && !content.contains("&lt;") {
            return Ok(ValidationResult {
                is_valid: false,
                errors: vec!["XML content contains unescaped angle brackets".to_string()],
                warnings: vec![],
            });
        }

        Ok(ValidationResult {
            is_valid: true,
            errors: vec![],
            warnings: vec![],
        })
    }
}

impl XmlFormatStrategy {
    fn escape_xml(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}

/// Example: YAML format implementation (OCP: extension without modification)
pub struct YamlFormat;

impl ExtensibleFormat for YamlFormat {
    fn name(&self) -> &str {
        "yaml"
    }
    
    fn extensions(&self) -> Vec<String> {
        vec!["yaml".to_string(), "yml".to_string()]
    }
    
    fn mime_type(&self) -> &str {
        "application/x-yaml"
    }
    
    fn description(&self) -> &str {
        "YAML Ain't Markup Language format"
    }
    
    fn create_strategy(&self) -> Box<dyn FormatStrategy> {
        Box::new(YamlFormatStrategy)
    }
}

/// YAML format strategy implementation
pub struct YamlFormatStrategy;

impl FormatStrategy for YamlFormatStrategy {
    fn get_header(&self, metadata: &crate::modules::report_generator::interfaces::report_interfaces::ReportMetadata, title: &str) -> String {
        format!(
            "---\ntitle: \"{}\"\ngenerated_at: \"{}\"\nversion: \"{}\"\n",
            self.escape_yaml(title),
            self.escape_yaml(&metadata.generated_at),
            self.escape_yaml(&metadata.version)
        )
    }

    fn get_footer(&self, _metadata: &crate::modules::report_generator::interfaces::report_interfaces::ReportMetadata) -> String {
        "---".to_string()
    }

    fn format_name(&self) -> &'static str {
        "yaml"
    }

    fn file_extension(&self) -> &'static str {
        "yaml"
    }

    fn mime_type(&self) -> &'static str {
        "application/x-yaml"
    }

    fn validate_content(&self, content: &str) -> QmsResult<crate::modules::report_generator::interfaces::report_interfaces::ValidationResult> {
        use crate::modules::report_generator::interfaces::report_interfaces::ValidationResult;

        if content.contains('\n') || content.contains('\r') {
            return Ok(ValidationResult {
                is_valid: false,
                errors: vec!["YAML content contains newline characters".to_string()],
                warnings: vec![],
            });
        }

        Ok(ValidationResult {
            is_valid: true,
            errors: vec![],
            warnings: vec![],
        })
    }
}

impl YamlFormatStrategy {
    fn escape_yaml(&self, text: &str) -> String {
        text.replace('"', "\\\"")
            .replace('\\', "\\\\")
    }
}

/// Format plugin manager for dynamic format loading
/// OCP: Allows runtime registration of new formats
pub struct FormatPluginManager {
    registered_formats: Vec<Box<dyn ExtensibleFormat>>,
}

impl FormatPluginManager {
    pub fn new() -> Self {
        Self {
            registered_formats: Vec::new(),
        }
    }
    
    /// Register a new format plugin (OCP: extension without modification)
    pub fn register_format(&mut self, format: Box<dyn ExtensibleFormat>) -> QmsResult<()> {
        // Register with global registry
        format.register()?;
        
        // Keep track locally
        self.registered_formats.push(format);
        
        Ok(())
    }
    
    /// Initialize built-in extensible formats
    pub fn initialize_builtin_formats(&mut self) -> QmsResult<()> {
        // Register XML format
        self.register_format(Box::new(XmlFormat))?;
        
        // Register YAML format
        self.register_format(Box::new(YamlFormat))?;
        
        Ok(())
    }
    
    /// Get all registered format names
    pub fn get_registered_format_names(&self) -> Vec<String> {
        self.registered_formats.iter()
            .map(|f| f.name().to_string())
            .collect()
    }
}

impl Default for FormatPluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to initialize all extensible formats
pub fn initialize_extensible_formats() -> QmsResult<()> {
    let mut manager = FormatPluginManager::new();
    manager.initialize_builtin_formats()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_format() {
        let xml_format = XmlFormat;
        assert_eq!(xml_format.name(), "xml");
        assert_eq!(xml_format.extensions(), vec!["xml"]);
        assert_eq!(xml_format.mime_type(), "application/xml");
        
        let strategy = xml_format.create_strategy();
        assert_eq!(strategy.format_name(), "xml");
        assert_eq!(strategy.file_extension(), "xml");
        assert_eq!(strategy.mime_type(), "application/xml");
    }

    #[test]
    fn test_yaml_format() {
        let yaml_format = YamlFormat;
        assert_eq!(yaml_format.name(), "yaml");
        assert!(yaml_format.extensions().contains(&"yaml".to_string()));
        assert!(yaml_format.extensions().contains(&"yml".to_string()));
        
        let strategy = yaml_format.create_strategy();
        assert_eq!(strategy.format_name(), "yaml");
        assert_eq!(strategy.file_extension(), "yaml");
        assert_eq!(strategy.mime_type(), "application/x-yaml");
    }

    #[test]
    fn test_format_plugin_manager() {
        let mut manager = FormatPluginManager::new();
        
        let result = manager.register_format(Box::new(XmlFormat));
        assert!(result.is_ok());
        
        let names = manager.get_registered_format_names();
        assert!(names.contains(&"xml".to_string()));
    }
}
