//! QMS system configuration management
//! Phase 2 infrastructure - configuration system for medical device compliance

#![allow(dead_code)] // Phase 2 infrastructure - configuration management system

use crate::error::{QmsError, QmsResult};
use crate::json_utils::{save_json, JsonValue};
use crate::utils::validate_string_length;
use crate::constants::{fda_qsr, system, cfr_part_11}; // SSOT: Use centralized constants
use std::fs;
use std::path::{Path, PathBuf};

/// Logging configuration for FDA-compliant audit trails
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Path to the log file directory
    pub log_file_path: PathBuf,
    /// Maximum size of each log file before rotation (in bytes)
    pub max_file_size: u64,
    /// Maximum number of rotated log files to keep
    pub max_files: usize,
    /// Log level (TRACE, DEBUG, INFO, WARN, ERROR)
    pub level: String,
    /// Whether to enable console logging in addition to file logging
    pub console_logging: bool,
    /// Whether to enable JSON formatted logs for structured logging
    pub json_format: bool,
    /// Whether to enable audit-specific logging
    pub audit_logging: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_file_path: PathBuf::from("logs/qms.log"),
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_files: 10,
            level: "INFO".to_string(),
            console_logging: true,
            json_format: true, // For FDA compliance and audit trails
            audit_logging: true,
        }
    }
}

impl LoggingConfig {
    /// Create FDA-compliant logging configuration
    pub fn new_fda_compliant() -> Self {
        Self {
            log_file_path: PathBuf::from("logs/audit.log"),
            max_file_size: 100 * 1024 * 1024, // 100MB for comprehensive audit trails
            max_files: 50, // Keep more files for compliance
            level: "INFO".to_string(),
            console_logging: false, // File-only for compliance
            json_format: true, // Required for audit trails
            audit_logging: true,
        }
    }

    /// Validate logging configuration
    pub fn validate(&self) -> QmsResult<()> {
        let valid_levels = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR"];
        if !valid_levels.contains(&self.level.as_str()) {
            return Err(QmsError::validation_error(&format!(
                "Invalid log level '{}'. Must be one of: {:?}",
                self.level, valid_levels
            )));
        }

        if self.max_file_size < 1024 {
            return Err(QmsError::validation_error(
                "Maximum file size must be at least 1KB"
            ));
        }

        if self.max_files == 0 {
            return Err(QmsError::validation_error(
                "Must keep at least 1 log file"
            ));
        }

        Ok(())
    }

    /// Get the log directory path
    pub fn log_dir(&self) -> PathBuf {
        self.log_file_path.parent().unwrap_or(Path::new("logs")).to_path_buf()
    }

    /// Convert logging configuration to JSON value
    pub fn to_json_value(&self) -> JsonValue {
        let mut fields = std::collections::HashMap::new();

        fields.insert(
            "log_file_path".to_string(),
            JsonValue::String(self.log_file_path.to_string_lossy().to_string()),
        );
        fields.insert(
            "max_file_size".to_string(),
            JsonValue::Number(self.max_file_size as f64),
        );
        fields.insert(
            "max_files".to_string(),
            JsonValue::Number(self.max_files as f64),
        );
        fields.insert(
            "level".to_string(),
            JsonValue::String(self.level.clone()),
        );
        fields.insert(
            "console_logging".to_string(),
            JsonValue::Bool(self.console_logging),
        );
        fields.insert(
            "json_format".to_string(),
            JsonValue::Bool(self.json_format),
        );
        fields.insert(
            "audit_logging".to_string(),
            JsonValue::Bool(self.audit_logging),
        );

        JsonValue::Object(fields)
    }

    /// Create logging configuration from JSON value
    pub fn from_json_value(json: Option<&JsonValue>) -> QmsResult<Self> {
        match json {
            Some(JsonValue::Object(obj)) => {
                let log_file_path = match obj.get("log_file_path") {
                    Some(JsonValue::String(s)) => PathBuf::from(s),
                    _ => PathBuf::from("logs/qms.log"),
                };

                let max_file_size = match obj.get("max_file_size") {
                    Some(JsonValue::Number(n)) => *n as u64,
                    _ => 10 * 1024 * 1024,
                };

                let max_files = match obj.get("max_files") {
                    Some(JsonValue::Number(n)) => *n as usize,
                    _ => 10,
                };

                let level = match obj.get("level") {
                    Some(JsonValue::String(s)) => s.clone(),
                    _ => "INFO".to_string(),
                };

                let console_logging = match obj.get("console_logging") {
                    Some(JsonValue::Bool(b)) => *b,
                    _ => true,
                };

                let json_format = match obj.get("json_format") {
                    Some(JsonValue::Bool(b)) => *b,
                    _ => true,
                };

                let audit_logging = match obj.get("audit_logging") {
                    Some(JsonValue::Bool(b)) => *b,
                    _ => true,
                };

                let config = LoggingConfig {
                    log_file_path,
                    max_file_size,
                    max_files,
                    level,
                    console_logging,
                    json_format,
                    audit_logging,
                };

                config.validate()?;
                Ok(config)
            }
            None => {
                // If no logging config is provided, use default
                Ok(LoggingConfig::default())
            }
            _ => Err(QmsError::parse_error("Logging configuration must be a JSON object")),
        }
    }
}

/// QMS system configuration structure
#[derive(Debug, Clone)]
pub struct Config {
    pub project_root: PathBuf,
    pub log_level: String,
    pub version: String,
    pub audit_retention_days: u32,
    pub max_document_versions: u32,
    pub require_electronic_signature: bool,
    pub backup_enabled: bool,
    pub encryption_enabled: bool,
    pub logging: LoggingConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self::create_regulatory_compliant_config()
    }
}

impl Config {
    /// Create regulatory-compliant configuration (DRY: Single source of truth)
    /// This method eliminates duplication between Default::default() and new_default()
    /// SSOT: Uses centralized constants for regulatory compliance values
    fn create_regulatory_compliant_config() -> Self {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());

        Config {
            project_root: PathBuf::from(home_dir).join(system::DEFAULT_PROJECT_DIR),
            log_level: system::DEFAULT_LOG_LEVEL.to_string(),
            version: system::DEFAULT_VERSION.to_string(),
            audit_retention_days: fda_qsr::AUDIT_RETENTION_DAYS,
            max_document_versions: fda_qsr::MAX_DOCUMENT_VERSIONS,
            require_electronic_signature: cfr_part_11::REQUIRE_AUDIT_TRAIL,
            backup_enabled: system::BACKUP_ENABLED_DEFAULT,
            encryption_enabled: system::ENCRYPTION_ENABLED_DEFAULT,
            logging: LoggingConfig::default(),
        }
    }

    /// Load configuration from a JSON file
    pub fn load(path: &Path) -> QmsResult<Self> {
        if !path.exists() {
            return Err(QmsError::not_found(&format!(
                "Configuration file not found: {}",
                path.display()
            )));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| QmsError::io_error(&format!("Failed to read config file: {e}")))?;

        let json_value = JsonValue::parse(&content)
            .map_err(|e| QmsError::parse_error(&format!("Invalid JSON in config file: {e:?}")))?;

        Self::from_json_value(&json_value)
    }

    /// Create default configuration with regulatory-compliant settings
    /// REFACTORED: Now delegates to single source of truth to eliminate DRY violation
    pub fn new_default() -> Self {
        Self::create_regulatory_compliant_config()
    }

    /// Save configuration to a JSON file
    pub fn save(&self, path: &Path) -> QmsResult<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                QmsError::io_error(&format!("Failed to create config directory: {e}"))
            })?;
        }

        let json_value = self.to_json_value();
        save_json(path, &json_value)
            .map_err(|e| QmsError::io_error(&format!("Failed to save config: {e}")))
    }

    /// Validate all configuration values
    pub fn validate(&self) -> QmsResult<()> {
        // Validate log level
        let valid_levels = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR"];
        if !valid_levels.contains(&self.log_level.as_str()) {
            return Err(QmsError::validation_error(&format!(
                "Invalid log level '{}'. Must be one of: {:?}",
                self.log_level, valid_levels
            )));
        }

        // Validate version format (basic semver check)
        if !self.version.contains('.') || self.version.len() < 5 {
            return Err(QmsError::validation_error(&format!(
                "Invalid version format '{}'. Expected semantic version (e.g., '1.0.0')",
                self.version
            )));
        }

        // Validate audit retention (minimum 1 year for medical devices)
        if self.audit_retention_days < 365 {
            return Err(QmsError::validation_error(
                "Audit retention must be at least 365 days for regulatory compliance",
            ));
        }

        // Validate max document versions
        if self.max_document_versions < 1 || self.max_document_versions > 1000 {
            return Err(QmsError::validation_error(
                "Max document versions must be between 1 and 1000",
            ));
        }

        // Validate project root path
        if !validate_string_length(&self.project_root.to_string_lossy(), 1000) {
            return Err(QmsError::validation_error(
                "Project root path is too long (max 1000 characters)",
            ));
        }

        // For medical devices, electronic signatures should be required
        if !self.require_electronic_signature {
            return Err(QmsError::validation_error(
                "Electronic signatures are required for FDA 21 CFR Part 11 compliance",
            ));
        }

        // Validate logging configuration
        self.logging.validate()?;

        Ok(())
    }

    /// Convert configuration to JSON value for serialization
    fn to_json_value(&self) -> JsonValue {
        let mut fields = std::collections::HashMap::new();

        fields.insert(
            "project_root".to_string(),
            JsonValue::String(self.project_root.to_string_lossy().to_string()),
        );
        fields.insert(
            "log_level".to_string(),
            JsonValue::String(self.log_level.clone()),
        );
        fields.insert(
            "version".to_string(),
            JsonValue::String(self.version.clone()),
        );
        fields.insert(
            "audit_retention_days".to_string(),
            JsonValue::Number(self.audit_retention_days as f64),
        );
        fields.insert(
            "max_document_versions".to_string(),
            JsonValue::Number(self.max_document_versions as f64),
        );
        fields.insert(
            "require_electronic_signature".to_string(),
            JsonValue::Bool(self.require_electronic_signature),
        );
        fields.insert(
            "backup_enabled".to_string(),
            JsonValue::Bool(self.backup_enabled),
        );
        fields.insert(
            "encryption_enabled".to_string(),
            JsonValue::Bool(self.encryption_enabled),
        );
        fields.insert(
            "logging".to_string(),
            self.logging.to_json_value(),
        );

        JsonValue::Object(fields)
    }

    /// Create configuration from JSON value
    fn from_json_value(json: &JsonValue) -> QmsResult<Self> {
        match json {
            JsonValue::Object(obj) => {
                let project_root = match obj.get("project_root") {
                    Some(JsonValue::String(s)) => PathBuf::from(s),
                    _ => {
                        return Err(QmsError::parse_error(
                            "Missing or invalid 'project_root' field",
                        ))
                    }
                };

                let log_level = match obj.get("log_level") {
                    Some(JsonValue::String(s)) => s.clone(),
                    _ => {
                        return Err(QmsError::parse_error(
                            "Missing or invalid 'log_level' field",
                        ))
                    }
                };

                let version = match obj.get("version") {
                    Some(JsonValue::String(s)) => s.clone(),
                    _ => return Err(QmsError::parse_error("Missing or invalid 'version' field")),
                };

                let audit_retention_days = match obj.get("audit_retention_days") {
                    Some(JsonValue::Number(n)) => *n as u32,
                    _ => {
                        return Err(QmsError::parse_error(
                            "Missing or invalid 'audit_retention_days' field",
                        ))
                    }
                };

                let max_document_versions = match obj.get("max_document_versions") {
                    Some(JsonValue::Number(n)) => *n as u32,
                    _ => {
                        return Err(QmsError::parse_error(
                            "Missing or invalid 'max_document_versions' field",
                        ))
                    }
                };

                let require_electronic_signature = match obj.get("require_electronic_signature") {
                    Some(JsonValue::Bool(b)) => *b,
                    _ => {
                        return Err(QmsError::parse_error(
                            "Missing or invalid 'require_electronic_signature' field",
                        ))
                    }
                };

                let backup_enabled = match obj.get("backup_enabled") {
                    Some(JsonValue::Bool(b)) => *b,
                    _ => {
                        return Err(QmsError::parse_error(
                            "Missing or invalid 'backup_enabled' field",
                        ))
                    }
                };

                let encryption_enabled = match obj.get("encryption_enabled") {
                    Some(JsonValue::Bool(b)) => *b,
                    _ => {
                        return Err(QmsError::parse_error(
                            "Missing or invalid 'encryption_enabled' field",
                        ))
                    }
                };

                let logging = LoggingConfig::from_json_value(obj.get("logging"))?;

                let config = Config {
                    project_root,
                    log_level,
                    version,
                    audit_retention_days,
                    max_document_versions,
                    require_electronic_signature,
                    backup_enabled,
                    encryption_enabled,
                    logging,
                };

                // Validate the loaded configuration
                config.validate()?;
                Ok(config)
            }
            _ => Err(QmsError::parse_error("Configuration must be a JSON object")),
        }
    }

    /// Get the full path to the audit log file
    pub fn audit_log_path(&self) -> PathBuf {
        self.project_root.join("audit").join("audit.log")
    }

    /// Get the full path to the projects directory
    pub fn projects_dir(&self) -> PathBuf {
        self.project_root.join("projects")
    }

    /// Get the full path to the config file
    pub fn config_file_path(&self) -> PathBuf {
        self.project_root.join("config.json")
    }

    /// Check if the configuration is valid for medical device use
    pub const fn is_medical_device_compliant(&self) -> bool {
        self.require_electronic_signature
            && self.backup_enabled
            && self.encryption_enabled
            && self.audit_retention_days >= 365
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.log_level, "INFO");
        assert_eq!(config.version, "1.0.0");
        assert_eq!(config.audit_retention_days, 2555); // 7 years
        assert!(config.require_electronic_signature);
        assert!(config.backup_enabled);
        assert!(config.encryption_enabled);
        assert_eq!(config.logging.level, "INFO");
        assert!(config.logging.console_logging);
        assert!(config.logging.json_format);
        assert!(config.logging.audit_logging);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();

        // Test valid configuration
        assert!(config.validate().is_ok());

        // Test invalid log level
        config.log_level = "INVALID".to_string();
        assert!(config.validate().is_err());
        config.log_level = "INFO".to_string();

        // Test invalid version
        config.version = "invalid".to_string();
        assert!(config.validate().is_err());
        config.version = "1.0.0".to_string();

        // Test invalid audit retention
        config.audit_retention_days = 100;
        assert!(config.validate().is_err());
        config.audit_retention_days = 365;

        // Test invalid max versions
        config.max_document_versions = 0;
        assert!(config.validate().is_err());
        config.max_document_versions = 50;

        // Test electronic signature requirement
        config.require_electronic_signature = false;
        assert!(config.validate().is_err());

        // Test invalid logging configuration
        let mut invalid_logging = config.logging.clone();
        invalid_logging.level = "INVALID".to_string();
        config.logging = invalid_logging;
        assert!(config.validate().is_err());
        config.logging.level = "INFO".to_string();

        let mut invalid_logging = config.logging.clone();
        invalid_logging.max_file_size = 0;
        config.logging = invalid_logging;
        assert!(config.validate().is_err());
        config.logging.max_file_size = 10 * 1024 * 1024;

        let mut invalid_logging = config.logging.clone();
        invalid_logging.max_files = 0;
        config.logging = invalid_logging;
        assert!(config.validate().is_err());
        config.logging.max_files = 10;
    }

    #[test]
    fn test_config_medical_compliance() {
        let config = Config::default();
        assert!(config.is_medical_device_compliant());

        let mut non_compliant = config.clone();
        non_compliant.require_electronic_signature = false;
        assert!(!non_compliant.is_medical_device_compliant());
    }

    #[test]
    fn test_config_paths() {
        let config = Config::default();

        let audit_path = config.audit_log_path();
        assert!(audit_path.to_string_lossy().contains("audit.log"));

        let projects_dir = config.projects_dir();
        assert!(projects_dir.to_string_lossy().contains("projects"));

        let config_path = config.config_file_path();
        assert!(config_path.to_string_lossy().contains("config.json"));
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json_value = config.to_json_value();

        // Test round-trip serialization
        let restored_config = Config::from_json_value(&json_value).unwrap();
        assert_eq!(config.log_level, restored_config.log_level);
        assert_eq!(config.version, restored_config.version);
        assert_eq!(
            config.audit_retention_days,
            restored_config.audit_retention_days
        );
        assert_eq!(config.logging.level, restored_config.logging.level);
    }

    #[test]
    fn test_config_save_load() {
        let temp_dir = env::temp_dir();
        let config_path = temp_dir.join("test_config.json");

        // Create and save config
        let original_config = Config::default();
        assert!(original_config.save(&config_path).is_ok());

        // Load and verify
        let loaded_config = Config::load(&config_path).unwrap();
        assert_eq!(original_config.log_level, loaded_config.log_level);
        assert_eq!(original_config.version, loaded_config.version);

        // Clean up
        let _ = fs::remove_file(&config_path);
    }
}
