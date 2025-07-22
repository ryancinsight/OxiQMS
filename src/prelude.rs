//! Prelude module for convenient imports across the QMS project
//! This will be heavily used in Phase 2 development

#![allow(unused_imports)]

// Re-export common standard library types
pub use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

// Re-export our common types
pub use crate::audit::{log_audit, log_command_execution, log_error, log_project_event};
pub use crate::error::{QmsError, QmsResult};
pub use crate::models::*;
pub use crate::utils::*;
