/// GRASP Principles Enhancement: High Cohesion and Low Coupling Module
/// 
/// This module demonstrates high cohesion by grouping related functionality
/// together and low coupling by minimizing dependencies between modules.

pub mod risk_cohesion_module;

// Re-export for convenience
pub use risk_cohesion_module::{RiskCohesionModule, RiskAnalysisResult, RiskValidationRules};
