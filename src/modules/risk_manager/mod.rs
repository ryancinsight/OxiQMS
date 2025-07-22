pub mod fmea;
pub mod iso14971;
pub mod scoring;
pub mod risk;
pub mod reporting;
pub mod import_export;
pub mod surveillance;
pub mod documentation;
pub mod categorization;
pub mod approval;
pub mod communication;
pub mod metrics;

// SOLID Principle Enhancement Modules
pub mod assessment_strategy;
pub mod storage_interfaces;
pub mod enhanced_manager;

// SRP Refactoring Modules - Separated Responsibilities
pub mod repository;      // Data persistence layer
pub mod validator;       // Validation logic layer
pub mod business_logic;  // Core business operations

// OCP Enhancement Modules - Open/Closed Principle
pub mod template_registry; // Template generation registry pattern

// DIP Enhancement Modules - Dependency Inversion Principle
pub mod risk_service; // Risk service abstraction for dependency injection

#[cfg(test)]
pub mod integration_tests;

// Re-export key types for convenience
pub use risk::{
    RiskSeverity, RiskOccurrence, RiskDetectability, RiskLevel,
    RiskManager, RiskFilter, RiskRegisterFilter, RiskStatus
};

pub use fmea::{
    FMEAManager
};

pub use iso14971::{
    ISO14971Validator, ComplianceStatus,
    RMFOptions, RMFFormat
};

pub use scoring::{
    RiskMatrix, MatrixColor, RiskScoring
};

pub use reporting::{
    RiskReporter, ReportType, ReportFormat, TimePeriod, RiskTrend
};

pub use import_export::{
    ImportFormat, ExportFormat, ImportOptions, ExportOptions,
    RiskImporter, RiskExporter
};

pub use surveillance::{
    SurveillanceManager //, SurveillanceData, SurveillanceType, SurveillanceStatus,
    // FrequencyData, DeviceInfo, CorrectiveAction, ActionType, ActionStatus
};

pub use documentation::{
    DocumentationManager, TemplateType, OutputFormat, TemplateConfig
};

pub use categorization::{
    RiskCategory, ClassificationDimension, RiskCategorizationManager
};

pub use approval::{
    RiskApprovalManager
};

pub use communication::{
    RiskCommunicationManager, StakeholderType
};

pub use metrics::{
    RiskMetricsManager, RiskKPIs, MetricsPeriod, RiskDashboard
};

// SOLID Principle Enhancement Exports
pub use assessment_strategy::{
    RiskAssessmentStrategy, RiskAssessmentContext, RiskAssessmentResult,
    RiskAssessmentStrategyFactory, ISO14971AssessmentStrategy,
    FDA820AssessmentStrategy, ConservativeAssessmentStrategy
};

pub use storage_interfaces::{
    RiskReader, RiskWriter, RiskIndexManager, RiskSearcher, RiskBackupManager,
    RiskSearchCriteria, FileRiskStorage
};

pub use enhanced_manager::{
    EnhancedRiskManager, RiskStatistics
};
