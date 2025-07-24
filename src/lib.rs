#![forbid(unsafe_code)]
#![allow(unused, clippy::used_underscore_binding)]
#![warn(clippy::missing_const_for_fn, clippy::approx_constant, clippy::all)]

// Re-export all modules for testing
pub mod audit;
pub mod commands;
pub mod config;
pub mod constants; // SSOT: Centralized regulatory compliance constants
pub mod error;
pub mod fs_utils;
pub mod interfaces;
pub mod json_utils;
pub mod tui;
pub mod lock;
pub mod models;
pub mod modules;
pub mod prelude;
pub mod services;
pub mod utils;
pub mod validation;
pub mod web;
pub mod wasm;

// Integration tests module - re-enabled with proper implementations
#[cfg(test)]
pub mod integration;

// Minimal test module
#[cfg(test)]
mod test_minimal;
