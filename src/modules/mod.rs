pub mod audit_logger;
pub mod document_control;
pub mod report_generator;
pub mod repository;
pub mod risk_manager;
pub mod traceability;
pub mod user_manager;

// SOLID Principles Enhancement
pub mod storage;

// GRASP Principles Enhancement
pub mod domain_experts;
pub mod creators;
pub mod controllers;
pub mod cohesion;

// CUPID Principles Enhancement
pub mod cupid;

// Open/Closed Principle Enhancement
pub mod universal_format_factory; // REFACTORED: OCP-compliant universal format factory
