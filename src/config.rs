//! QMS system configuration management
//! Phase 2 infrastructure - configuration system for medical device compliance

#![allow(dead_code)] // Phase 2 infrastructure - configuration management system

use crate::error::{QmsError, QmsResult};
use crate::json_utils::{save_json, JsonValue};
use crate::utils::validate_string_length;
use crate::constants::{fda_qsr, system, cfr_part_11}; // SSOT: Use centralized constants
use std::fs;
use std::path::{Path, PathBuf};

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

                let config = Config {
                    project_root,
                    log_level,
                    version,
                    audit_retention_days,
                    max_document_versions,
                    require_electronic_signature,
                    backup_enabled,
                    encryption_enabled,
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
