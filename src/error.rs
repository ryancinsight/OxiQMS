//! Error handling for QMS operations
//! Phase 2 infrastructure - comprehensive error types for medical device compliance

#![allow(dead_code)] // Phase 2 infrastructure - error handling system

use std::fmt;
use std::io;

/// Centralized error type for the QMS system
#[derive(Debug)]
pub enum QmsError {
    /// I/O operation failed
    Io(io::Error),
    /// Parsing failed
    Parse(String),
    /// File system lock failed
    Lock(String),
    /// Domain-specific business logic error
    Domain(String),
    /// Validation failed
    Validation(String),
    /// Permission denied
    Permission(String),
    /// Resource not found
    NotFound(String),
    /// Resource already exists
    AlreadyExists(String),
    /// Authentication failed
    Authentication(String),
    /// Invalid operation
    InvalidOperation(String),
}

impl fmt::Display for QmsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QmsError::Io(e) => write!(f, "I/O error: {e}"),
            QmsError::Parse(msg) => write!(f, "Parse error: {msg}"),
            QmsError::Lock(msg) => write!(f, "Lock error: {msg}"),
            QmsError::Domain(msg) => write!(f, "Domain error: {msg}"),
            QmsError::Validation(msg) => write!(f, "Validation error: {msg}"),
            QmsError::Permission(msg) => write!(f, "Permission error: {msg}"),
            QmsError::NotFound(msg) => write!(f, "Not found: {msg}"),
            QmsError::AlreadyExists(msg) => write!(f, "Already exists: {msg}"),
            QmsError::Authentication(msg) => write!(f, "Authentication error: {msg}"),
            QmsError::InvalidOperation(msg) => write!(f, "Invalid operation: {msg}"),
        }
    }
}

impl std::error::Error for QmsError {}

impl From<io::Error> for QmsError {
    fn from(error: io::Error) -> Self {
        QmsError::Io(error)
    }
}

impl From<crate::json_utils::JsonError> for QmsError {
    fn from(error: crate::json_utils::JsonError) -> Self {
        QmsError::Parse(error.to_string())
    }
}

impl From<std::time::SystemTimeError> for QmsError {
    fn from(error: std::time::SystemTimeError) -> Self {
        QmsError::Domain(format!("System time error: {error}"))
    }
}

impl QmsError {
    /// Create a validation error
    pub fn validation_error(msg: &str) -> Self {
        QmsError::Validation(msg.to_string())
    }

    /// Create a not found error
    pub fn not_found(msg: &str) -> Self {
        QmsError::NotFound(msg.to_string())
    }

    /// Create a domain error
    pub fn domain_error(msg: &str) -> Self {
        QmsError::Domain(msg.to_string())
    }

    /// Create a permission error
    pub fn permission_error(msg: &str) -> Self {
        QmsError::Permission(msg.to_string())
    }

    /// Create an already exists error
    pub fn already_exists(msg: &str) -> Self {
        QmsError::AlreadyExists(msg.to_string())
    }

    /// Create an IO error
    pub fn io_error(msg: &str) -> Self {
        QmsError::Io(std::io::Error::new(std::io::ErrorKind::Other, msg))
    }

    /// Create a parse error
    pub fn parse_error(msg: &str) -> Self {
        QmsError::Parse(msg.to_string())
    }

    /// Create an invalid operation error
    pub fn invalid_operation(msg: &str) -> Self {
        QmsError::InvalidOperation(msg.to_string())
    }
}

/// Result type alias for QMS operations
pub type QmsResult<T> = Result<T, QmsError>;
