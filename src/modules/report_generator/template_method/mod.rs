//! Template Method Pattern Module
//! 
//! This module contains the Template Method pattern implementation for report generation.
//! The base report generator defines the common algorithm structure while allowing
//! subclasses to customize specific steps.

pub mod base_report_generator;

// Re-export for convenience
pub use base_report_generator::*;
