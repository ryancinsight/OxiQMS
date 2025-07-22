/// GRASP Principles Enhancement: Domain Experts Module
/// 
/// This module contains domain expert classes that follow the Information Expert
/// principle by encapsulating domain knowledge and responsibilities.

pub mod risk_expert;
pub mod audit_expert;

// Re-export for convenience
pub use risk_expert::{RiskExpert, MitigationPriority};
pub use audit_expert::{AuditExpert, AuditSummary, ComplianceStatus};
