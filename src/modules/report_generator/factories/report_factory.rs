//! Abstract Factory Pattern for Report Generator Creation
//! 
//! This module implements the Abstract Factory pattern for creating different
//! types of report generators while following SOLID principles.
//! 
//! SOLID Principles Applied:
//! - Single Responsibility: Each factory handles creation of one report type family
//! - Open/Closed: New report types can be added without modifying existing factories
//! - Liskov Substitution: All factories can be used interchangeably
//! - Interface Segregation: Focused factory interfaces for different report families
//! - Dependency Inversion: High-level code depends on factory abstractions

use crate::prelude::*;
use super::super::interfaces::report_interfaces::*;
use super::super::template_method::base_report_generator::BaseReportGenerator;
use super::super::strategies::format_strategies::{FormatStrategy, FormatStrategyFactory};
use std::sync::Arc;

/// Abstract Factory interface for report generator creation
/// Interface Segregation Principle: Focused interface for report generator creation
pub trait ReportGeneratorFactory {
    /// Create a report generator for the specified type
    fn create_generator(&self, report_type: &ReportType, context: &ReportContext) -> QmsResult<Box<dyn ReportGenerator>>;
    
    /// Get supported report types for this factory
    fn supported_types(&self) -> Vec<ReportType>;
    
    /// Get factory name for identification
    fn factory_name(&self) -> &'static str;
    
    /// Validate if factory can create the requested report type
    fn can_create(&self, report_type: &ReportType) -> bool {
        self.supported_types().contains(report_type)
    }
}

/// Report type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReportType {
    Audit,
    Risk,
    DHF,
    Traceability,
    Compliance,
    Performance,
    Custom(String),
}

impl ReportType {
    /// Parse report type from string
    pub fn from_str(s: &str) -> QmsResult<Self> {
        match s.to_lowercase().as_str() {
            "audit" => Ok(ReportType::Audit),
            "risk" => Ok(ReportType::Risk),
            "dhf" => Ok(ReportType::DHF),
            "traceability" => Ok(ReportType::Traceability),
            "compliance" => Ok(ReportType::Compliance),
            "performance" => Ok(ReportType::Performance),
            _ => Ok(ReportType::Custom(s.to_string())),
        }
    }
    
    /// Get string representation
    pub fn to_string(&self) -> String {
        match self {
            ReportType::Audit => "audit".to_string(),
            ReportType::Risk => "risk".to_string(),
            ReportType::DHF => "dhf".to_string(),
            ReportType::Traceability => "traceability".to_string(),
            ReportType::Compliance => "compliance".to_string(),
            ReportType::Performance => "performance".to_string(),
            ReportType::Custom(name) => name.clone(),
        }
    }
}

/// Generic report generator interface
/// Single Responsibility Principle: Focused on report generation operations
pub trait ReportGenerator {
    /// Generate report with specified format and destination
    fn generate(&self, format: &OutputFormat, destination: &OutputDestination) -> QmsResult<String>;
    
    /// Get report type
    fn report_type(&self) -> ReportType;
    
    /// Get supported formats
    fn supported_formats(&self) -> Vec<OutputFormat>;
    
    /// Validate generation parameters
    fn validate_parameters(&self, format: &OutputFormat, destination: &OutputDestination) -> QmsResult<()>;
}

/// Medical Device Report Factory
/// Single Responsibility Principle: Creates medical device compliance reports
pub struct MedicalDeviceReportFactory {
    project_path: std::path::PathBuf,
}

impl MedicalDeviceReportFactory {
    pub fn new(project_path: std::path::PathBuf) -> Self {
        Self { project_path }
    }
}

impl ReportGeneratorFactory for MedicalDeviceReportFactory {
    fn create_generator(&self, report_type: &ReportType, context: &ReportContext) -> QmsResult<Box<dyn ReportGenerator>> {
        match report_type {
            ReportType::Audit => Ok(Box::new(AuditReportGeneratorImpl::new(context.clone()))),
            ReportType::Risk => Ok(Box::new(RiskReportGeneratorImpl::new(context.clone()))),
            ReportType::DHF => Ok(Box::new(DHFReportGeneratorImpl::new(context.clone()))),
            ReportType::Compliance => Ok(Box::new(ComplianceReportGeneratorImpl::new(context.clone()))),
            _ => Err(QmsError::validation_error(&format!(
                "Medical device factory does not support report type: {:?}",
                report_type
            ))),
        }
    }
    
    fn supported_types(&self) -> Vec<ReportType> {
        vec![
            ReportType::Audit,
            ReportType::Risk,
            ReportType::DHF,
            ReportType::Compliance,
        ]
    }
    
    fn factory_name(&self) -> &'static str {
        "MedicalDeviceReportFactory"
    }
}

/// Quality Management Report Factory
/// Single Responsibility Principle: Creates quality management reports
pub struct QualityManagementReportFactory {
    project_path: std::path::PathBuf,
}

impl QualityManagementReportFactory {
    pub fn new(project_path: std::path::PathBuf) -> Self {
        Self { project_path }
    }
}

impl ReportGeneratorFactory for QualityManagementReportFactory {
    fn create_generator(&self, report_type: &ReportType, context: &ReportContext) -> QmsResult<Box<dyn ReportGenerator>> {
        match report_type {
            ReportType::Traceability => Ok(Box::new(TraceabilityReportGeneratorImpl::new(context.clone()))),
            ReportType::Performance => Ok(Box::new(PerformanceReportGeneratorImpl::new(context.clone()))),
            ReportType::Compliance => Ok(Box::new(ComplianceReportGeneratorImpl::new(context.clone()))),
            _ => Err(QmsError::validation_error(&format!(
                "Quality management factory does not support report type: {:?}",
                report_type
            ))),
        }
    }
    
    fn supported_types(&self) -> Vec<ReportType> {
        vec![
            ReportType::Traceability,
            ReportType::Performance,
            ReportType::Compliance,
        ]
    }
    
    fn factory_name(&self) -> &'static str {
        "QualityManagementReportFactory"
    }
}

/// Master Report Factory - coordinates multiple specialized factories
/// Single Responsibility Principle: Coordinates factory selection and delegation
pub struct MasterReportFactory {
    factories: Vec<Box<dyn ReportGeneratorFactory>>,
}

impl MasterReportFactory {
    pub fn new() -> Self {
        Self {
            factories: Vec::new(),
        }
    }
    
    /// Register a specialized factory
    /// Open/Closed Principle: Can add new factories without modifying existing code
    pub fn register_factory(&mut self, factory: Box<dyn ReportGeneratorFactory>) {
        self.factories.push(factory);
    }
    
    /// Create report generator using appropriate factory
    pub fn create_generator(&self, report_type: &ReportType, context: &ReportContext) -> QmsResult<Box<dyn ReportGenerator>> {
        for factory in &self.factories {
            if factory.can_create(report_type) {
                return factory.create_generator(report_type, context);
            }
        }
        
        Err(QmsError::validation_error(&format!(
            "No factory available for report type: {:?}",
            report_type
        )))
    }
    
    /// Get all supported report types across all factories
    pub fn get_all_supported_types(&self) -> Vec<ReportType> {
        let mut types = Vec::new();
        for factory in &self.factories {
            types.extend(factory.supported_types());
        }
        types.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
        types.dedup();
        types
    }
    
    /// Get factory information
    pub fn get_factory_info(&self) -> Vec<(String, Vec<ReportType>)> {
        self.factories
            .iter()
            .map(|f| (f.factory_name().to_string(), f.supported_types()))
            .collect()
    }
}

impl Default for MasterReportFactory {
    fn default() -> Self {
        Self::new()
    }
}

// Concrete report generator implementations
// These would be implemented using the Template Method pattern

/// Audit Report Generator Implementation
struct AuditReportGeneratorImpl {
    context: ReportContext,
}

impl AuditReportGeneratorImpl {
    fn new(context: ReportContext) -> Self {
        Self { context }
    }
}

impl ReportGenerator for AuditReportGeneratorImpl {
    fn generate(&self, format: &OutputFormat, destination: &OutputDestination) -> QmsResult<String> {
        // Implementation would use Template Method pattern
        // For now, return placeholder
        Ok(format!("Audit report in {:?} format", format))
    }
    
    fn report_type(&self) -> ReportType {
        ReportType::Audit
    }
    
    fn supported_formats(&self) -> Vec<OutputFormat> {
        vec![OutputFormat::Markdown, OutputFormat::CSV, OutputFormat::JSON, OutputFormat::HTML]
    }
    
    fn validate_parameters(&self, format: &OutputFormat, _destination: &OutputDestination) -> QmsResult<()> {
        if !self.supported_formats().contains(format) {
            return Err(QmsError::validation_error(&format!(
                "Audit reports do not support {:?} format",
                format
            )));
        }
        Ok(())
    }
}

/// Risk Report Generator Implementation
struct RiskReportGeneratorImpl {
    context: ReportContext,
}

impl RiskReportGeneratorImpl {
    fn new(context: ReportContext) -> Self {
        Self { context }
    }
}

impl ReportGenerator for RiskReportGeneratorImpl {
    fn generate(&self, format: &OutputFormat, destination: &OutputDestination) -> QmsResult<String> {
        // Implementation would use Template Method pattern
        Ok(format!("Risk report in {:?} format", format))
    }
    
    fn report_type(&self) -> ReportType {
        ReportType::Risk
    }
    
    fn supported_formats(&self) -> Vec<OutputFormat> {
        vec![OutputFormat::Markdown, OutputFormat::CSV, OutputFormat::JSON, OutputFormat::HTML, OutputFormat::PDF]
    }
    
    fn validate_parameters(&self, format: &OutputFormat, _destination: &OutputDestination) -> QmsResult<()> {
        if !self.supported_formats().contains(format) {
            return Err(QmsError::validation_error(&format!(
                "Risk reports do not support {:?} format",
                format
            )));
        }
        Ok(())
    }
}

/// DHF Report Generator Implementation
struct DHFReportGeneratorImpl {
    context: ReportContext,
}

impl DHFReportGeneratorImpl {
    fn new(context: ReportContext) -> Self {
        Self { context }
    }
}

impl ReportGenerator for DHFReportGeneratorImpl {
    fn generate(&self, format: &OutputFormat, destination: &OutputDestination) -> QmsResult<String> {
        // Implementation would use Template Method pattern
        Ok(format!("DHF report in {:?} format", format))
    }
    
    fn report_type(&self) -> ReportType {
        ReportType::DHF
    }
    
    fn supported_formats(&self) -> Vec<OutputFormat> {
        vec![OutputFormat::Markdown, OutputFormat::HTML, OutputFormat::PDF]
    }
    
    fn validate_parameters(&self, format: &OutputFormat, _destination: &OutputDestination) -> QmsResult<()> {
        if !self.supported_formats().contains(format) {
            return Err(QmsError::validation_error(&format!(
                "DHF reports do not support {:?} format",
                format
            )));
        }
        Ok(())
    }
}

/// Compliance Report Generator Implementation
struct ComplianceReportGeneratorImpl {
    context: ReportContext,
}

impl ComplianceReportGeneratorImpl {
    fn new(context: ReportContext) -> Self {
        Self { context }
    }
}

impl ReportGenerator for ComplianceReportGeneratorImpl {
    fn generate(&self, format: &OutputFormat, destination: &OutputDestination) -> QmsResult<String> {
        Ok(format!("Compliance report in {:?} format", format))
    }
    
    fn report_type(&self) -> ReportType {
        ReportType::Compliance
    }
    
    fn supported_formats(&self) -> Vec<OutputFormat> {
        vec![OutputFormat::Markdown, OutputFormat::JSON, OutputFormat::HTML, OutputFormat::PDF]
    }
    
    fn validate_parameters(&self, format: &OutputFormat, _destination: &OutputDestination) -> QmsResult<()> {
        if !self.supported_formats().contains(format) {
            return Err(QmsError::validation_error(&format!(
                "Compliance reports do not support {:?} format",
                format
            )));
        }
        Ok(())
    }
}

/// Traceability Report Generator Implementation
struct TraceabilityReportGeneratorImpl {
    context: ReportContext,
}

impl TraceabilityReportGeneratorImpl {
    fn new(context: ReportContext) -> Self {
        Self { context }
    }
}

impl ReportGenerator for TraceabilityReportGeneratorImpl {
    fn generate(&self, format: &OutputFormat, destination: &OutputDestination) -> QmsResult<String> {
        Ok(format!("Traceability report in {:?} format", format))
    }
    
    fn report_type(&self) -> ReportType {
        ReportType::Traceability
    }
    
    fn supported_formats(&self) -> Vec<OutputFormat> {
        vec![OutputFormat::Markdown, OutputFormat::CSV, OutputFormat::JSON, OutputFormat::HTML]
    }
    
    fn validate_parameters(&self, format: &OutputFormat, _destination: &OutputDestination) -> QmsResult<()> {
        if !self.supported_formats().contains(format) {
            return Err(QmsError::validation_error(&format!(
                "Traceability reports do not support {:?} format",
                format
            )));
        }
        Ok(())
    }
}

/// Performance Report Generator Implementation
struct PerformanceReportGeneratorImpl {
    context: ReportContext,
}

impl PerformanceReportGeneratorImpl {
    fn new(context: ReportContext) -> Self {
        Self { context }
    }
}

impl ReportGenerator for PerformanceReportGeneratorImpl {
    fn generate(&self, format: &OutputFormat, destination: &OutputDestination) -> QmsResult<String> {
        Ok(format!("Performance report in {:?} format", format))
    }
    
    fn report_type(&self) -> ReportType {
        ReportType::Performance
    }
    
    fn supported_formats(&self) -> Vec<OutputFormat> {
        vec![OutputFormat::Markdown, OutputFormat::CSV, OutputFormat::JSON, OutputFormat::HTML]
    }
    
    fn validate_parameters(&self, format: &OutputFormat, _destination: &OutputDestination) -> QmsResult<()> {
        if !self.supported_formats().contains(format) {
            return Err(QmsError::validation_error(&format!(
                "Performance reports do not support {:?} format",
                format
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_type_parsing() {
        assert_eq!(ReportType::from_str("audit").unwrap(), ReportType::Audit);
        assert_eq!(ReportType::from_str("RISK").unwrap(), ReportType::Risk);
        assert_eq!(ReportType::from_str("dhf").unwrap(), ReportType::DHF);
        
        match ReportType::from_str("custom_report").unwrap() {
            ReportType::Custom(name) => assert_eq!(name, "custom_report"),
            _ => panic!("Expected Custom variant"),
        }
    }

    #[test]
    fn test_medical_device_factory() {
        let factory = MedicalDeviceReportFactory::new(std::path::PathBuf::from("/test"));
        
        assert!(factory.can_create(&ReportType::Audit));
        assert!(factory.can_create(&ReportType::Risk));
        assert!(factory.can_create(&ReportType::DHF));
        assert!(!factory.can_create(&ReportType::Traceability));
        
        assert_eq!(factory.factory_name(), "MedicalDeviceReportFactory");
    }

    #[test]
    fn test_quality_management_factory() {
        let factory = QualityManagementReportFactory::new(std::path::PathBuf::from("/test"));
        
        assert!(factory.can_create(&ReportType::Traceability));
        assert!(factory.can_create(&ReportType::Performance));
        assert!(!factory.can_create(&ReportType::Audit));
        
        assert_eq!(factory.factory_name(), "QualityManagementReportFactory");
    }

    #[test]
    fn test_master_factory() {
        let mut master = MasterReportFactory::new();
        
        master.register_factory(Box::new(MedicalDeviceReportFactory::new(std::path::PathBuf::from("/test"))));
        master.register_factory(Box::new(QualityManagementReportFactory::new(std::path::PathBuf::from("/test"))));
        
        let supported_types = master.get_all_supported_types();
        assert!(supported_types.contains(&ReportType::Audit));
        assert!(supported_types.contains(&ReportType::Risk));
        assert!(supported_types.contains(&ReportType::Traceability));
        assert!(supported_types.contains(&ReportType::Performance));
        
        let factory_info = master.get_factory_info();
        assert_eq!(factory_info.len(), 2);
    }

    #[test]
    fn test_report_generator_creation() {
        let context = ReportContext::new(
            std::path::PathBuf::from("/test"),
            "test-user".to_string(),
        );
        
        let factory = MedicalDeviceReportFactory::new(std::path::PathBuf::from("/test"));
        let generator = factory.create_generator(&ReportType::Audit, &context).unwrap();
        
        assert_eq!(generator.report_type(), ReportType::Audit);
        assert!(generator.supported_formats().contains(&OutputFormat::Markdown));
    }
}
