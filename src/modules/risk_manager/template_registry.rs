//! Template Registry - OCP Compliance Implementation
//! 
//! OCP (Open/Closed Principle): Open for extension, closed for modification
//! Replaces switch statements with registry pattern for template generation
//! 
//! Medical Device Compliance: Maintains ISO 14971 template requirements

use crate::prelude::*;
use super::documentation::{TemplateType, TemplateConfig, OutputFormat};
use std::collections::HashMap;

/// Template Generator Trait - OCP: Abstraction for template generation
/// 
/// Each template type implements this trait, enabling new templates
/// to be added without modifying existing code
pub trait TemplateGenerator {
    /// Generate template content
    fn generate(&self, config: &TemplateConfig) -> QmsResult<String>;
    
    /// Get template type identifier
    fn template_type(&self) -> TemplateType;
    
    /// Get supported output formats
    fn supported_formats(&self) -> Vec<OutputFormat>;
    
    /// Validate template configuration
    fn validate_config(&self, config: &TemplateConfig) -> QmsResult<()>;
}

/// Template Registry - OCP: Registry pattern for dynamic template management
/// 
/// Enables adding new template generators without modifying existing code
/// Follows the Registry pattern for extensibility
pub struct TemplateRegistry {
    generators: HashMap<TemplateType, Box<dyn TemplateGenerator>>,
}

impl TemplateRegistry {
    /// Create new template registry
    pub fn new() -> Self {
        Self {
            generators: HashMap::new(),
        }
    }
    
    /// Register a template generator - OCP: Extension point
    pub fn register<T: TemplateGenerator + 'static>(&mut self, generator: T) {
        let template_type = generator.template_type();
        self.generators.insert(template_type, Box::new(generator));
    }
    
    /// Generate template using registered generator
    pub fn generate_template(
        &self,
        template_type: &TemplateType,
        config: &TemplateConfig,
    ) -> QmsResult<String> {
        match self.generators.get(&template_type) {
            Some(generator) => {
                // Validate configuration first
                generator.validate_config(config)?;
                
                // Generate template content
                let content = generator.generate(config)?;
                
                // Log template generation for audit trail
                crate::audit::log_audit(&format!(
                    "TEMPLATE_GENERATED: {:?} - {} characters",
                    template_type,
                    content.len()
                ));
                
                Ok(content)
            }
            None => Err(QmsError::validation_error(&format!(
                "No generator registered for template type: {:?}",
                template_type
            ))),
        }
    }
    
    /// Check if template type is supported
    pub fn is_supported(&self, template_type: &TemplateType) -> bool {
        self.generators.contains_key(template_type)
    }
    
    /// Get all supported template types
    pub fn supported_types(&self) -> Vec<TemplateType> {
        self.generators.keys().cloned().collect()
    }
    
    /// Get supported formats for a template type
    pub fn supported_formats(&self, template_type: &TemplateType) -> QmsResult<Vec<OutputFormat>> {
        match self.generators.get(template_type) {
            Some(generator) => Ok(generator.supported_formats()),
            None => Err(QmsError::validation_error(&format!(
                "Template type not supported: {:?}",
                template_type
            ))),
        }
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        
        // Register default template generators - OCP: Default extensions
        registry.register(RiskAssessmentTemplateGenerator::new());
        registry.register(FmeaTemplateGenerator::new());
        registry.register(RiskManagementPlanTemplateGenerator::new());
        registry.register(ControlEffectivenessTemplateGenerator::new());
        registry.register(PostMarketSurveillanceTemplateGenerator::new());
        
        registry
    }
}

/// Risk Assessment Template Generator - OCP: Concrete implementation
pub struct RiskAssessmentTemplateGenerator;

impl RiskAssessmentTemplateGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl TemplateGenerator for RiskAssessmentTemplateGenerator {
    fn generate(&self, config: &TemplateConfig) -> QmsResult<String> {
        // Generate ISO 14971 compliant risk assessment template
        let template = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Risk Assessment Report - {}</title>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        .header {{ text-align: center; margin-bottom: 30px; }}
        .section {{ margin-bottom: 20px; }}
        .risk-table {{ width: 100%; border-collapse: collapse; }}
        .risk-table th, .risk-table td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        .risk-table th {{ background-color: #f2f2f2; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Risk Assessment Report</h1>
        <p>Project: {}</p>
        <p>Author: {}</p>
        <p>Organization: {}</p>
        <p>Generated: {}</p>
        <p>Standard: ISO 14971:2019</p>
    </div>
    
    <div class="section">
        <h2>1. Risk Management Process</h2>
        <p>This risk assessment follows ISO 14971:2019 requirements for medical device risk management.</p>
    </div>
    
    <div class="section">
        <h2>2. Risk Analysis</h2>
        <table class="risk-table">
            <thead>
                <tr>
                    <th>Hazard ID</th>
                    <th>Hazard Description</th>
                    <th>Hazardous Situation</th>
                    <th>Harm</th>
                    <th>Severity</th>
                    <th>Occurrence</th>
                    <th>Detectability</th>
                    <th>RPN</th>
                    <th>Risk Level</th>
                </tr>
            </thead>
            <tbody>
                <!-- Risk data will be populated here -->
            </tbody>
        </table>
    </div>
    
    <div class="section">
        <h2>3. Risk Control Measures</h2>
        <p>Risk control measures implemented per ISO 14971 Section 6.</p>
    </div>
    
    <div class="section">
        <h2>4. Residual Risk Assessment</h2>
        <p>Post-mitigation risk levels and acceptability assessment.</p>
    </div>
</body>
</html>"#,
            &config.title,
            &config.device_name,
            &config.author,
            &config.organization,
            crate::utils::current_timestamp_string()
        );
        
        Ok(template)
    }
    
    fn template_type(&self) -> TemplateType {
        TemplateType::RiskAssessment
    }
    
    fn supported_formats(&self) -> Vec<OutputFormat> {
        vec![OutputFormat::HTML, OutputFormat::Markdown]
    }
    
    fn validate_config(&self, _config: &TemplateConfig) -> QmsResult<()> {
        // Basic validation - can be extended
        Ok(())
    }
}

/// FMEA Template Generator - OCP: Another concrete implementation
pub struct FmeaTemplateGenerator;

impl FmeaTemplateGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl TemplateGenerator for FmeaTemplateGenerator {
    fn generate(&self, config: &TemplateConfig) -> QmsResult<String> {
        let template = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>FMEA Report - {}</title>
</head>
<body>
    <h1>Failure Mode and Effects Analysis (FMEA)</h1>
    <p>Project: {}</p>
    <p>Author: {}</p>
    <p>Standard: ISO 14971:2019 Section 5</p>

    <h2>Risk Priority Number (RPN) Calculation</h2>
    <p>RPN = Severity × Occurrence × Detectability</p>

    <table border="1">
        <thead>
            <tr>
                <th>Failure Mode</th>
                <th>Effects</th>
                <th>Severity</th>
                <th>Occurrence</th>
                <th>Detectability</th>
                <th>RPN</th>
                <th>Actions</th>
            </tr>
        </thead>
        <tbody>
            <!-- FMEA data will be populated here -->
        </tbody>
    </table>
</body>
</html>"#,
            &config.title,
            &config.device_name,
            &config.author
        );

        Ok(template)
    }
    
    fn template_type(&self) -> TemplateType {
        TemplateType::FMEA
    }
    
    fn supported_formats(&self) -> Vec<OutputFormat> {
        vec![OutputFormat::HTML, OutputFormat::Markdown]
    }
    
    fn validate_config(&self, _config: &TemplateConfig) -> QmsResult<()> {
        Ok(())
    }
}

// Placeholder implementations for other template generators
pub struct RiskManagementPlanTemplateGenerator;
impl RiskManagementPlanTemplateGenerator {
    pub fn new() -> Self { Self }
}
impl TemplateGenerator for RiskManagementPlanTemplateGenerator {
    fn generate(&self, config: &TemplateConfig) -> QmsResult<String> {
        let template = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Risk Management Plan - {}</title>
</head>
<body>
    <h1>Risk Management Plan</h1>
    <p>Device: {}</p>
    <p>Author: {}</p>
    <p>Organization: {}</p>
    <p>Standard: ISO 14971:2019</p>

    <h2>Risk Management Process</h2>
    <p>This document outlines the risk management process for the medical device development lifecycle.</p>

    <h3>1. Risk Analysis Process</h3>
    <p>Systematic process for identifying hazards and estimating risks.</p>

    <h3>2. Risk Evaluation Process</h3>
    <p>Process for determining risk acceptability criteria.</p>

    <h3>3. Risk Control Process</h3>
    <p>Process for implementing and verifying risk control measures.</p>

    <h3>4. Risk Acceptability Criteria</h3>
    <p>Criteria for determining when risks are acceptable per ISO 14971.</p>

    <h3>5. Residual Risk Evaluation</h3>
    <p>Process for evaluating residual risks after risk control implementation.</p>
</body>
</html>"#,
            &config.title,
            &config.device_name,
            &config.author,
            &config.organization
        );
        Ok(template)
    }
    fn template_type(&self) -> TemplateType { TemplateType::RiskManagementPlan }
    fn supported_formats(&self) -> Vec<OutputFormat> { vec![OutputFormat::HTML] }
    fn validate_config(&self, _config: &TemplateConfig) -> QmsResult<()> { Ok(()) }
}

pub struct ControlEffectivenessTemplateGenerator;
impl ControlEffectivenessTemplateGenerator {
    pub fn new() -> Self { Self }
}
impl TemplateGenerator for ControlEffectivenessTemplateGenerator {
    fn generate(&self, config: &TemplateConfig) -> QmsResult<String> {
        let template = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Control Effectiveness Report - {}</title>
</head>
<body>
    <h1>Control Effectiveness Report</h1>
    <p>Device: {}</p>
    <p>Author: {}</p>
    <p>Organization: {}</p>
    <p>Standard: ISO 14971:2019</p>

    <h2>Risk Control Effectiveness Evaluation</h2>
    <p>This report evaluates the effectiveness of implemented risk controls per ISO 14971 requirements.</p>
</body>
</html>"#,
            &config.title,
            &config.device_name,
            &config.author,
            &config.organization
        );
        Ok(template)
    }
    fn template_type(&self) -> TemplateType { TemplateType::ControlEffectiveness }
    fn supported_formats(&self) -> Vec<OutputFormat> { vec![OutputFormat::HTML] }
    fn validate_config(&self, _config: &TemplateConfig) -> QmsResult<()> { Ok(()) }
}

pub struct PostMarketSurveillanceTemplateGenerator;
impl PostMarketSurveillanceTemplateGenerator {
    pub fn new() -> Self { Self }
}
impl TemplateGenerator for PostMarketSurveillanceTemplateGenerator {
    fn generate(&self, config: &TemplateConfig) -> QmsResult<String> {
        let template = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Post-Market Surveillance Plan - {}</title>
</head>
<body>
    <h1>Post-Market Surveillance Plan</h1>
    <p>Device: {}</p>
    <p>Author: {}</p>
    <p>Organization: {}</p>
    <p>Standard: ISO 14971:2019</p>

    <h2>Post-Market Surveillance Activities</h2>
    <p>This plan outlines post-market surveillance activities per ISO 14971 Section 9.</p>

    <h3>Surveillance Data Collection</h3>
    <p>Methods for collecting post-market data on device performance and safety.</p>
</body>
</html>"#,
            &config.title,
            &config.device_name,
            &config.author,
            &config.organization
        );
        Ok(template)
    }
    fn template_type(&self) -> TemplateType { TemplateType::PostMarketSurveillance }
    fn supported_formats(&self) -> Vec<OutputFormat> { vec![OutputFormat::HTML] }
    fn validate_config(&self, _config: &TemplateConfig) -> QmsResult<()> { Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_template_registry_creation() {
        let registry = TemplateRegistry::new();
        assert_eq!(registry.generators.len(), 0);
    }
    
    #[test]
    fn test_template_registration() {
        let mut registry = TemplateRegistry::new();
        registry.register(RiskAssessmentTemplateGenerator::new());
        
        assert!(registry.is_supported(&TemplateType::RiskAssessment));
        assert_eq!(registry.supported_types().len(), 1);
    }
    
    #[test]
    fn test_default_registry() {
        let registry = TemplateRegistry::default();
        
        // Should have all default template generators registered
        assert!(registry.is_supported(&TemplateType::RiskAssessment));
        assert!(registry.is_supported(&TemplateType::FMEA));
        assert_eq!(registry.supported_types().len(), 5);
    }
    
    #[test]
    fn test_template_generation() {
        let registry = TemplateRegistry::default();
        let config = TemplateConfig {
            title: "Test Risk Assessment".to_string(),
            version: "1.0".to_string(),
            author: "Test Author".to_string(),
            organization: "Test Org".to_string(),
            device_name: "Test Device".to_string(),
            device_version: "1.0".to_string(),
            date_generated: "2024-01-01".to_string(),
            regulatory_basis: vec!["ISO_14971".to_string()],
        };

        let result = registry.generate_template(&TemplateType::RiskAssessment, &config);
        assert!(result.is_ok());

        let content = result.unwrap();
        assert!(content.contains("Test Risk Assessment"));
        assert!(content.contains("Test Device"));
    }
}
