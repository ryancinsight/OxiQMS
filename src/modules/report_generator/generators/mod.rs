//! Report Generators Module - Canonical Implementations
//! 
//! This module contains the canonical implementations of all report generators,
//! consolidating functionality and following SOLID principles.
//! 
//! All generators in this module:
//! - Follow the Template Method pattern for consistent report generation workflow
//! - Use Strategy pattern for different output formats
//! - Implement dependency injection for testability and flexibility
//! - Maintain backward compatibility with legacy APIs
//! - Follow medical device compliance standards

pub mod audit_generator;
pub mod dhf_generator;
pub mod risk_generator;

// Re-export canonical implementations
pub use audit_generator::{AuditReportGenerator, FileAuditDataCollector, AuditDataFormatter};
pub use dhf_generator::{DHFReportGenerator, DocumentDataCollector, DHFDataFormatter};
pub use risk_generator::{RiskReportGenerator, RiskDataCollector, RiskDataFormatter};
