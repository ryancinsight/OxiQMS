// SOLID Principle Architecture - Canonical Implementation
pub mod interfaces;
pub mod template_method;
pub mod strategies;
pub mod factories;
pub mod generators;
pub mod timeout;
pub mod format_registry; // REFACTORED: OCP-compliant format registry
pub mod extensible_formats; // REFACTORED: OCP-compliant extensible format system

// Utility modules
pub mod csv;
pub mod markdown;



// SOLID Principle Architecture Exports
pub use interfaces::*;
pub use template_method::*;
pub use strategies::*;
pub use factories::*;
pub use timeout::*;

// Canonical Report Generator Implementations
pub use generators::{
    AuditReportGenerator, DHFReportGenerator, RiskReportGenerator,
    FileAuditDataCollector, AuditDataFormatter,
    DocumentDataCollector, DHFDataFormatter,
    RiskDataCollector, RiskDataFormatter,
};
