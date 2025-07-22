// QMS WebAssembly Client Module
// Medical Device Quality Management System - Client-side functionality
// Replaces JavaScript with Rust-compiled WASM for regulatory compliance
// Regulatory Compliance: FDA 21 CFR Part 820, ISO 13485, ISO 14971

pub mod api_client;
pub mod dom_utils;
pub mod event_handlers;
pub mod app;

// Re-exports for easier access
pub use api_client::QmsApiClient;
pub use dom_utils::DomUtils;
pub use event_handlers::EventHandlers;
pub use app::QmsApp;
