/// DRY Improvement: Common Configuration Utilities
/// 
/// This module consolidates configuration patterns that were duplicated
/// across multiple modules, following the DRY principle.

use crate::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Common configuration patterns used throughout the QMS system
pub struct ConfigHelper;

impl ConfigHelper {
    /// Load configuration from file with fallback to defaults (DRY: common pattern)
    /// KISS: Simplified to work with HashMap<String, String> instead of generic types
    pub fn load_config_with_fallback(
        config_path: &Path,
        default_config: HashMap<String, String>,
    ) -> QmsResult<HashMap<String, String>> {
        if config_path.exists() {
            match std::fs::read_to_string(config_path) {
                Ok(content) => {
                    // Simple key=value parsing instead of JSON
                    let mut config = HashMap::new();
                    for line in content.lines() {
                        if let Some((key, value)) = line.split_once('=') {
                            config.insert(key.trim().to_string(), value.trim().to_string());
                        }
                    }
                    if config.is_empty() {
                        Self::save_config(config_path, &default_config)?;
                        Ok(default_config)
                    } else {
                        Ok(config)
                    }
                }
                Err(_) => Ok(default_config),
            }
        } else {
            // Create default config file
            Self::save_config(config_path, &default_config)?;
            Ok(default_config)
        }
    }

    /// Save configuration to file (DRY: common save pattern)
    /// KISS: Simplified to work with HashMap<String, String>
    pub fn save_config(config_path: &Path, config: &HashMap<String, String>) -> QmsResult<()> {
        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Simple key=value format instead of JSON
        let mut content = String::new();
        for (key, value) in config {
            content.push_str(&format!("{}={}\n", key, value));
        }

        std::fs::write(config_path, content)
            .map_err(|e| QmsError::io_error(&format!("Failed to write config file: {}", e)))?;

        Ok(())
    }
    
    /// Validate configuration directory structure (DRY: common validation)
    pub fn validate_config_directory(base_path: &Path) -> QmsResult<()> {
        if !base_path.exists() {
            std::fs::create_dir_all(base_path)?;
        }
        
        if !base_path.is_dir() {
            return Err(QmsError::validation_error("Config path must be a directory"));
        }
        
        // Check write permissions by creating a test file
        let test_file = base_path.join(".qms_test");
        match std::fs::write(&test_file, "test") {
            Ok(_) => {
                let _ = std::fs::remove_file(&test_file);
                Ok(())
            }
            Err(_) => Err(QmsError::permission_error("Cannot write to config directory")),
        }
    }
    
    /// Get default configuration paths (DRY: path management)
    pub fn get_default_config_paths(project_root: &Path) -> ConfigPaths {
        ConfigPaths {
            project_root: project_root.to_path_buf(),
            config_dir: project_root.join("config"),
            main_config: project_root.join("config").join("qms.json"),
            audit_config: project_root.join("config").join("audit.json"),
            user_config: project_root.join("config").join("users.json"),
            risk_config: project_root.join("config").join("risk.json"),
            document_config: project_root.join("config").join("documents.json"),
            backup_dir: project_root.join("backups"),
            logs_dir: project_root.join("logs"),
            temp_dir: project_root.join("temp"),
        }
    }
    
    /// Initialize project directory structure (DRY: project setup)
    pub fn initialize_project_structure(project_root: &Path) -> QmsResult<ConfigPaths> {
        let paths = Self::get_default_config_paths(project_root);
        
        // Create all necessary directories
        let directories = [
            &paths.config_dir,
            &paths.backup_dir,
            &paths.logs_dir,
            &paths.temp_dir,
            &project_root.join("risks"),
            &project_root.join("documents"),
            &project_root.join("requirements"),
            &project_root.join("tests"),
            &project_root.join("audit"),
        ];
        
        for dir in &directories {
            if !dir.exists() {
                std::fs::create_dir_all(dir)?;
            }
        }
        
        Ok(paths)
    }
    
    /// Merge configuration with environment variables (DRY: env override pattern)
    pub fn merge_with_environment(
        mut config: HashMap<String, String>,
        prefix: &str,
    ) -> HashMap<String, String> {
        for (key, value) in std::env::vars() {
            if key.starts_with(prefix) {
                let config_key = key[prefix.len()..].to_lowercase();
                config.insert(config_key, value);
            }
        }
        config
    }
    
    /// Validate configuration values (DRY: config validation)
    pub fn validate_config_values(config: &HashMap<String, String>) -> QmsResult<()> {
        // Common validation rules
        if let Some(log_level) = config.get("log_level") {
            let valid_levels = ["debug", "info", "warn", "error"];
            if !valid_levels.contains(&log_level.as_str()) {
                return Err(QmsError::validation_error("Invalid log level"));
            }
        }
        
        if let Some(retention_days) = config.get("audit_retention_days") {
            if retention_days.parse::<u32>().is_err() {
                return Err(QmsError::validation_error("Invalid audit retention days"));
            }
        }
        
        if let Some(max_versions) = config.get("max_document_versions") {
            if max_versions.parse::<u32>().is_err() {
                return Err(QmsError::validation_error("Invalid max document versions"));
            }
        }
        
        Ok(())
    }
}

/// Configuration paths structure (DRY: path management)
#[derive(Debug, Clone)]
pub struct ConfigPaths {
    pub project_root: PathBuf,
    pub config_dir: PathBuf,
    pub main_config: PathBuf,
    pub audit_config: PathBuf,
    pub user_config: PathBuf,
    pub risk_config: PathBuf,
    pub document_config: PathBuf,
    pub backup_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub temp_dir: PathBuf,
}

impl ConfigPaths {
    /// Get path for specific entity type (DRY: entity path management)
    pub fn get_entity_path(&self, entity_type: &str) -> PathBuf {
        match entity_type {
            "risk" | "risks" => self.project_root.join("risks"),
            "document" | "documents" => self.project_root.join("documents"),
            "requirement" | "requirements" => self.project_root.join("requirements"),
            "test" | "tests" => self.project_root.join("tests"),
            "audit" => self.project_root.join("audit"),
            "user" | "users" => self.project_root.join("users"),
            _ => self.project_root.join(entity_type),
        }
    }
    
    /// Get config file path for specific module (DRY: module config management)
    pub fn get_module_config_path(&self, module: &str) -> PathBuf {
        match module {
            "audit" => self.audit_config.clone(),
            "user" | "users" => self.user_config.clone(),
            "risk" | "risks" => self.risk_config.clone(),
            "document" | "documents" => self.document_config.clone(),
            _ => self.config_dir.join(format!("{}.json", module)),
        }
    }
}

/// Default configuration builder (DRY: default config creation)
pub struct DefaultConfigBuilder;

impl DefaultConfigBuilder {
    /// Build default QMS configuration (DRY: default config pattern)
    pub fn build_qms_config(project_root: &Path) -> HashMap<String, String> {
        let mut config = HashMap::new();
        
        config.insert("project_root".to_string(), project_root.to_string_lossy().to_string());
        config.insert("log_level".to_string(), "info".to_string());
        config.insert("version".to_string(), "1.0.0".to_string());
        config.insert("audit_retention_days".to_string(), "2555".to_string()); // 7 years
        config.insert("max_document_versions".to_string(), "10".to_string());
        config.insert("require_electronic_signature".to_string(), "true".to_string());
        config.insert("backup_enabled".to_string(), "true".to_string());
        config.insert("encryption_enabled".to_string(), "true".to_string());
        
        config
    }
    
    /// Build default audit configuration (DRY: audit config pattern)
    pub fn build_audit_config() -> HashMap<String, String> {
        let mut config = HashMap::new();
        
        config.insert("retention_days".to_string(), "2555".to_string());
        config.insert("daily_rotation".to_string(), "true".to_string());
        config.insert("max_file_size_mb".to_string(), "100".to_string());
        config.insert("require_checksums".to_string(), "true".to_string());
        config.insert("compression_enabled".to_string(), "true".to_string());
        config.insert("backup_enabled".to_string(), "true".to_string());
        
        config
    }
    
    /// Build default risk management configuration (DRY: risk config pattern)
    pub fn build_risk_config() -> HashMap<String, String> {
        let mut config = HashMap::new();
        
        config.insert("default_severity_scale".to_string(), "1-5".to_string());
        config.insert("default_occurrence_scale".to_string(), "1-5".to_string());
        config.insert("default_detectability_scale".to_string(), "1-5".to_string());
        config.insert("acceptable_rpn_threshold".to_string(), "49".to_string());
        config.insert("require_approval_above_rpn".to_string(), "100".to_string());
        config.insert("auto_generate_ids".to_string(), "true".to_string());
        
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_config_paths() {
        let temp_dir = TempDir::new().unwrap();
        let paths = ConfigHelper::get_default_config_paths(temp_dir.path());
        
        assert_eq!(paths.project_root, temp_dir.path());
        assert_eq!(paths.config_dir, temp_dir.path().join("config"));
        assert_eq!(paths.get_entity_path("risks"), temp_dir.path().join("risks"));
    }
    
    #[test]
    fn test_project_structure_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let result = ConfigHelper::initialize_project_structure(temp_dir.path());
        
        assert!(result.is_ok());
        assert!(temp_dir.path().join("config").exists());
        assert!(temp_dir.path().join("risks").exists());
        assert!(temp_dir.path().join("documents").exists());
    }
    
    #[test]
    fn test_environment_merge() {
        let mut config = HashMap::new();
        config.insert("log_level".to_string(), "info".to_string());
        
        // This test would need environment variables set to be meaningful
        let merged = ConfigHelper::merge_with_environment(config, "QMS_");
        assert!(merged.contains_key("log_level"));
    }
    
    #[test]
    fn test_default_config_builder() {
        let temp_dir = TempDir::new().unwrap();
        let config = DefaultConfigBuilder::build_qms_config(temp_dir.path());
        
        assert!(config.contains_key("log_level"));
        assert!(config.contains_key("audit_retention_days"));
        assert_eq!(config.get("log_level").unwrap(), "info");
    }
}
