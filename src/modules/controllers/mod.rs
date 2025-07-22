/// GRASP Principles Enhancement: Controller Pattern Module
/// 
/// This module contains controller classes that follow the Controller principle
/// by handling system events and coordinating activities.

pub mod system_controller;

// Re-export for convenience
pub use system_controller::{SystemController, SystemStatus};
