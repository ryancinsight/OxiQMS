// Command handlers for CLI interface
// Each command module handles parsing and execution for its respective command

pub mod audit;
pub mod doc;
pub mod init;
pub mod report;
pub mod req;
pub mod risk;
pub mod test;
pub mod trace;
pub mod user;

// SOLID Principles Enhancement
pub mod command_handler_trait;
pub mod handlers;
pub mod solid_command_demo;
pub mod solid_validation_test;
