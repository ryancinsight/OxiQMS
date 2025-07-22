// Template Method Pattern - Common workflows for user operations
// Provides consistent structure: Validation → Action → Audit → Response

pub mod base_user_operation;

pub use base_user_operation::BaseUserOperation;
