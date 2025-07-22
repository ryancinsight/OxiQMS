/// CUPID Principles Enhancement Module
/// 
/// This module demonstrates CUPID principles:
/// - Composable: Services can be combined and reused
/// - Unix philosophy: Each service does one thing well
/// - Predictable: Consistent interfaces and behavior
/// - Idiomatic: Follows Rust best practices
/// - Domain-focused: Aligned with business domain concepts

pub mod composable_services;
pub mod predictable_interfaces;
pub mod domain_focused_architecture;

// Re-export for convenience
pub use composable_services::{
    ComposableService, RiskAssessmentPipeline, ServiceRegistry,
    RiskValidationInput, RiskValidationOutput, RpnCalculationOutput, RiskLevelOutput
};

pub use predictable_interfaces::{
    PredictableOperation, OperationInfo, OperationConfig, OperationConfigBuilder,
    ValidationLevel, PredictableRiskAssessment, RiskAssessmentInput, RiskAssessmentOutput
};

pub use domain_focused_architecture::{
    medical_device_risk_domain, quality_management_domain, regulatory_affairs_domain
};
