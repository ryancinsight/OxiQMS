//! Interface Adapters Module
//! 
//! This module contains adapters that bridge existing QMS systems to the new
//! unified interface abstractions, enabling progressive migration while
//! maintaining backward compatibility.

pub mod cli_adapter;
pub mod web_adapter;
pub mod project_adapter;
pub mod interface_adapters;

pub use cli_adapter::{
    CliCommandAdapter, LegacyCliRouterAdapter, CliInterfaceManager, CliCommandFactory
};
pub use web_adapter::{
    WebRouterAdapter, WebInterfaceManager, WebCommandBridge
};
pub use project_adapter::{
    ProjectServiceAdapter, SharedProjectManager
};
pub use interface_adapters::{
    CliInterfaceAdapter, WebInterfaceAdapter, TuiInterfaceAdapter
};
