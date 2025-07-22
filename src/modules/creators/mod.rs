/// GRASP Principles Enhancement: Creator Pattern Module
/// 
/// This module contains factory classes that follow the Creator principle
/// by being responsible for creating domain objects.

pub mod domain_object_factory;

// Re-export for convenience
pub use domain_object_factory::DomainObjectFactory;
