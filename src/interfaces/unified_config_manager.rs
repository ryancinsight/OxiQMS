//! Unified Configuration Management
//! 
//! Consolidates QMS settings, preferences, and configuration across all interfaces
//! following SOLID, DRY, and Single Source of Truth principles.

use crate::prelude::*;
use crate::interfaces::{InterfaceType, InterfaceContext};
use crate::interfaces::unified_context::{ConfigurationManager, UserPreferences, InterfaceSettings};
use crate::config::Config as QmsConfig;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Unified Configuration Manager
/// 
/// Central coordinator for all configuration management across interfaces.
/// Implements Single Responsibility Principle by focusing solely on configuration.
pub struct UnifiedConfigManager {
    /// Configuration storage
    config_storage: Arc<Mutex<ConfigurationStorage>>,
    
    /// Configuration providers for different sources
    config_providers: HashMap<ConfigSource, Box<dyn ConfigProvider>>,
    
    /// Configuration validators
    validators: HashMap<String, Box<dyn ConfigValidator>>,
    
    /// Configuration change listeners
    change_listeners: Vec<Arc<dyn ConfigChangeListener>>,
}

/// Configuration storage backend
#[derive(Debug, Clone)]
struct ConfigurationStorage {
    /// Global configuration
    global_config: QmsConfig,
    
    /// User-specific configurations
    user_configs: HashMap<String, UserConfiguration>,
    
    /// Interface-specific configurations
    interface_configs: HashMap<InterfaceType, InterfaceConfiguration>,
    
    /// Project-specific configurations
    project_configs: HashMap<String, ProjectConfiguration>,
    
    /// Configuration file paths
    config_paths: ConfigurationPaths,
}

/// Configuration source enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConfigSource {
    /// Global system configuration
    Global,
    
    /// User-specific configuration
    User,
    
    /// Project-specific configuration
    Project,
    
    /// Interface-specific configuration
    Interface,
    
    /// Environment variables
    Environment,
    
    /// Command line arguments
    CommandLine,
    
    /// Default values
    Default,
}

/// Configuration provider trait
pub trait ConfigProvider: Send + Sync {
    /// Load configuration from source
    fn load_config(&self, source_path: &Path) -> QmsResult<ConfigurationData>;
    
    /// Save configuration to source
    fn save_config(&self, config: &ConfigurationData, target_path: &Path) -> QmsResult<()>;
    
    /// Check if configuration source exists
    fn exists(&self, source_path: &Path) -> bool;
    
    /// Get configuration source priority (higher = more important)
    fn priority(&self) -> u32;
}

/// Configuration validator trait
pub trait ConfigValidator: Send + Sync {
    /// Validate configuration value
    fn validate(&self, key: &str, value: &str) -> QmsResult<ValidationResult>;
    
    /// Get validation rules for key
    fn get_rules(&self, key: &str) -> Vec<ValidationRule>;
}

/// Configuration change listener trait
pub trait ConfigChangeListener: Send + Sync {
    /// Handle configuration change
    fn on_config_changed(&self, change: &ConfigChange) -> QmsResult<()>;
}

/// Configuration data structure
#[derive(Debug, Clone)]
pub struct ConfigurationData {
    /// Configuration values
    pub values: HashMap<String, String>,
    
    /// Configuration metadata
    pub metadata: ConfigMetadata,
    
    /// Configuration source
    pub source: ConfigSource,
}

/// Configuration metadata
#[derive(Debug, Clone)]
pub struct ConfigMetadata {
    /// Configuration version
    pub version: String,
    
    /// Last modified timestamp
    pub last_modified: u64,
    
    /// Modified by user
    pub modified_by: Option<String>,
    
    /// Configuration schema version
    pub schema_version: String,
}

/// User configuration
#[derive(Debug, Clone)]
pub struct UserConfiguration {
    /// User preferences
    pub preferences: UserPreferences,
    
    /// User-specific settings
    pub settings: HashMap<String, String>,
    
    /// User configuration metadata
    pub metadata: ConfigMetadata,
}

/// Interface configuration
#[derive(Debug, Clone)]
pub struct InterfaceConfiguration {
    /// Interface settings
    pub settings: InterfaceSettings,
    
    /// Interface-specific overrides
    pub overrides: HashMap<String, String>,
    
    /// Interface configuration metadata
    pub metadata: ConfigMetadata,
}

/// Project configuration
#[derive(Debug, Clone)]
pub struct ProjectConfiguration {
    /// Project-specific settings
    pub settings: HashMap<String, String>,
    
    /// Project metadata
    pub metadata: ConfigMetadata,
    
    /// Project configuration schema
    pub schema: Option<String>,
}

/// Configuration paths
#[derive(Debug, Clone)]
pub struct ConfigurationPaths {
    /// Global configuration directory
    pub global_config_dir: PathBuf,
    
    /// User configuration directory
    pub user_config_dir: PathBuf,
    
    /// Project configuration file
    pub project_config_file: PathBuf,
    
    /// Interface configuration directory
    pub interface_config_dir: PathBuf,
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    
    /// Validation errors
    pub errors: Vec<String>,
    
    /// Validation warnings
    pub warnings: Vec<String>,
    
    /// Suggested corrections
    pub suggestions: Vec<String>,
}

/// Validation rule
#[derive(Debug, Clone)]
pub struct ValidationRule {
    /// Rule name
    pub name: String,
    
    /// Rule type
    pub rule_type: ValidationRuleType,
    
    /// Rule parameters
    pub parameters: HashMap<String, String>,
    
    /// Error message template
    pub error_message: String,
}

/// Validation rule type
#[derive(Debug, Clone)]
pub enum ValidationRuleType {
    /// Required field
    Required,
    
    /// String length validation
    StringLength { min: Option<usize>, max: Option<usize> },
    
    /// Numeric range validation
    NumericRange { min: Option<f64>, max: Option<f64> },
    
    /// Regular expression validation
    Regex(String),
    
    /// Enum value validation
    Enum(Vec<String>),
    
    /// Custom validation function
    Custom(String),
}

/// Configuration change event
#[derive(Debug, Clone)]
pub struct ConfigChange {
    /// Configuration key that changed
    pub key: String,
    
    /// Old value
    pub old_value: Option<String>,
    
    /// New value
    pub new_value: String,
    
    /// Change source
    pub source: ConfigSource,
    
    /// Change timestamp
    pub timestamp: u64,
    
    /// User who made the change
    pub changed_by: Option<String>,
}

impl UnifiedConfigManager {
    /// Create new unified configuration manager
    pub fn new(project_path: Option<PathBuf>) -> QmsResult<Self> {
        let config_paths = ConfigurationPaths::new(project_path)?;
        
        let config_storage = Arc::new(Mutex::new(ConfigurationStorage {
            global_config: QmsConfig::default(),
            user_configs: HashMap::new(),
            interface_configs: HashMap::new(),
            project_configs: HashMap::new(),
            config_paths,
        }));
        
        let mut config_providers = HashMap::new();
        config_providers.insert(ConfigSource::Global, Box::new(JsonConfigProvider::new()) as Box<dyn ConfigProvider>);
        config_providers.insert(ConfigSource::User, Box::new(JsonConfigProvider::new()) as Box<dyn ConfigProvider>);
        config_providers.insert(ConfigSource::Project, Box::new(JsonConfigProvider::new()) as Box<dyn ConfigProvider>);
        config_providers.insert(ConfigSource::Interface, Box::new(JsonConfigProvider::new()) as Box<dyn ConfigProvider>);
        config_providers.insert(ConfigSource::Environment, Box::new(EnvironmentConfigProvider::new()) as Box<dyn ConfigProvider>);
        
        let mut validators = HashMap::new();
        validators.insert("theme".to_string(), Box::new(ThemeValidator::new()) as Box<dyn ConfigValidator>);
        validators.insert("language".to_string(), Box::new(LanguageValidator::new()) as Box<dyn ConfigValidator>);
        validators.insert("project_path".to_string(), Box::new(PathValidator::new()) as Box<dyn ConfigValidator>);
        
        Ok(Self {
            config_storage,
            config_providers,
            validators,
            change_listeners: Vec::new(),
        })
    }
    
    /// Load configuration from all sources
    pub fn load_all_configurations(&self) -> QmsResult<()> {
        let storage = self.config_storage.lock().unwrap();
        let config_paths = &storage.config_paths;
        
        // Load configurations in priority order (lowest to highest)
        let sources = vec![
            (ConfigSource::Default, PathBuf::new()),
            (ConfigSource::Global, config_paths.global_config_dir.join("config.json")),
            (ConfigSource::User, config_paths.user_config_dir.join("config.json")),
            (ConfigSource::Project, config_paths.project_config_file.clone()),
            (ConfigSource::Interface, config_paths.interface_config_dir.join("config.json")),
            (ConfigSource::Environment, PathBuf::new()),
        ];
        
        drop(storage); // Release lock before loading
        
        for (source, path) in sources {
            if let Some(provider) = self.config_providers.get(&source) {
                if source == ConfigSource::Environment || provider.exists(&path) {
                    match provider.load_config(&path) {
                        Ok(config_data) => {
                            self.merge_configuration(config_data)?;
                        }
                        Err(e) => {
                            // Log error but continue loading other configurations
                            eprintln!("Warning: Failed to load configuration from {:?}: {}", source, e);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get configuration value with fallback chain
    pub fn get_config_value(&self, key: &str) -> Option<String> {
        let storage = self.config_storage.lock().unwrap();
        
        // Check sources in priority order (highest to lowest)
        let sources = vec![
            ConfigSource::CommandLine,
            ConfigSource::Environment,
            ConfigSource::Interface,
            ConfigSource::Project,
            ConfigSource::User,
            ConfigSource::Global,
            ConfigSource::Default,
        ];
        
        for source in sources {
            if let Some(value) = self.get_value_from_source(&storage, &source, key) {
                return Some(value);
            }
        }
        
        None
    }
    
    /// Set configuration value
    pub fn set_config_value(&self, key: &str, value: &str, source: ConfigSource) -> QmsResult<()> {
        // Validate configuration value
        if let Some(validator) = self.validators.get(key) {
            let validation_result = validator.validate(key, value)?;
            if !validation_result.is_valid {
                return Err(QmsError::validation_error(&format!(
                    "Invalid configuration value for '{}': {}",
                    key,
                    validation_result.errors.join(", ")
                )));
            }
        }
        
        // Get old value for change event
        let old_value = self.get_config_value(key);
        
        // Update configuration
        {
            let mut storage = self.config_storage.lock().unwrap();
            self.set_value_in_source(&mut storage, &source, key, value)?;
        }
        
        // Notify change listeners
        let change = ConfigChange {
            key: key.to_string(),
            old_value,
            new_value: value.to_string(),
            source,
            timestamp: crate::utils::current_timestamp(),
            changed_by: None, // Would be set from context in real implementation
        };
        
        for listener in &self.change_listeners {
            if let Err(e) = listener.on_config_changed(&change) {
                eprintln!("Warning: Configuration change listener failed: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Get user preferences
    pub fn get_user_preferences(&self, username: &str) -> Option<UserPreferences> {
        let storage = self.config_storage.lock().unwrap();
        storage.user_configs.get(username).map(|config| config.preferences.clone())
    }
    
    /// Set user preferences
    pub fn set_user_preferences(&self, username: &str, preferences: UserPreferences) -> QmsResult<()> {
        let mut storage = self.config_storage.lock().unwrap();
        
        let user_config = storage.user_configs.entry(username.to_string()).or_insert_with(|| {
            UserConfiguration {
                preferences: UserPreferences::default(),
                settings: HashMap::new(),
                metadata: ConfigMetadata {
                    version: "1.0.0".to_string(),
                    last_modified: crate::utils::current_timestamp(),
                    modified_by: Some(username.to_string()),
                    schema_version: "1.0".to_string(),
                },
            }
        });
        
        user_config.preferences = preferences;
        user_config.metadata.last_modified = crate::utils::current_timestamp();
        
        Ok(())
    }
    
    /// Get interface settings
    pub fn get_interface_settings(&self, interface_type: InterfaceType) -> Option<InterfaceSettings> {
        let storage = self.config_storage.lock().unwrap();
        storage.interface_configs.get(&interface_type).map(|config| config.settings.clone())
    }
    
    /// Set interface settings
    pub fn set_interface_settings(&self, interface_type: InterfaceType, settings: InterfaceSettings) -> QmsResult<()> {
        let mut storage = self.config_storage.lock().unwrap();
        
        let interface_config = storage.interface_configs.entry(interface_type).or_insert_with(|| {
            InterfaceConfiguration {
                settings: InterfaceSettings::default(),
                overrides: HashMap::new(),
                metadata: ConfigMetadata {
                    version: "1.0.0".to_string(),
                    last_modified: crate::utils::current_timestamp(),
                    modified_by: None,
                    schema_version: "1.0".to_string(),
                },
            }
        });
        
        interface_config.settings = settings;
        interface_config.metadata.last_modified = crate::utils::current_timestamp();
        
        Ok(())
    }
    
    /// Save all configurations to persistent storage
    pub fn save_all_configurations(&self) -> QmsResult<()> {
        let storage = self.config_storage.lock().unwrap();
        
        // Save user configurations
        for (username, user_config) in &storage.user_configs {
            let user_config_path = storage.config_paths.user_config_dir.join(format!("{}.json", username));
            let config_data = ConfigurationData {
                values: user_config.settings.clone(),
                metadata: user_config.metadata.clone(),
                source: ConfigSource::User,
            };
            
            if let Some(provider) = self.config_providers.get(&ConfigSource::User) {
                provider.save_config(&config_data, &user_config_path)?;
            }
        }
        
        // Save interface configurations
        for (interface_type, interface_config) in &storage.interface_configs {
            let interface_config_path = storage.config_paths.interface_config_dir
                .join(format!("{:?}.json", interface_type).to_lowercase());
            let config_data = ConfigurationData {
                values: interface_config.overrides.clone(),
                metadata: interface_config.metadata.clone(),
                source: ConfigSource::Interface,
            };
            
            if let Some(provider) = self.config_providers.get(&ConfigSource::Interface) {
                provider.save_config(&config_data, &interface_config_path)?;
            }
        }
        
        Ok(())
    }
    
    /// Add configuration change listener
    pub fn add_change_listener(&mut self, listener: Arc<dyn ConfigChangeListener>) {
        self.change_listeners.push(listener);
    }
    
    // Private helper methods
    
    /// Merge configuration data into storage
    fn merge_configuration(&self, config_data: ConfigurationData) -> QmsResult<()> {
        let mut storage = self.config_storage.lock().unwrap();
        
        match config_data.source {
            ConfigSource::Global => {
                // Merge into global config
                for (key, value) in config_data.values {
                    // In a real implementation, this would properly merge into QmsConfig
                    // For now, we'll just store in a generic way
                }
            }
            ConfigSource::User => {
                // Would merge user-specific configuration
            }
            ConfigSource::Project => {
                // Would merge project-specific configuration
            }
            ConfigSource::Interface => {
                // Would merge interface-specific configuration
            }
            ConfigSource::Environment => {
                // Environment variables are handled differently
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Get value from specific source
    fn get_value_from_source(&self, storage: &ConfigurationStorage, source: &ConfigSource, key: &str) -> Option<String> {
        match source {
            ConfigSource::Environment => {
                std::env::var(format!("QMS_{}", key.to_uppercase())).ok()
            }
            ConfigSource::Global => {
                // Would extract from global config
                None
            }
            ConfigSource::User => {
                // Would extract from user configs
                None
            }
            ConfigSource::Project => {
                // Would extract from project configs
                None
            }
            ConfigSource::Interface => {
                // Would extract from interface configs
                None
            }
            _ => None,
        }
    }
    
    /// Set value in specific source
    fn set_value_in_source(&self, storage: &mut ConfigurationStorage, source: &ConfigSource, key: &str, value: &str) -> QmsResult<()> {
        match source {
            ConfigSource::Global => {
                // Would update global config
            }
            ConfigSource::User => {
                // Would update user configs
            }
            ConfigSource::Project => {
                // Would update project configs
            }
            ConfigSource::Interface => {
                // Would update interface configs
            }
            _ => {
                return Err(QmsError::domain_error(&format!("Cannot set value in source: {:?}", source)));
            }
        }
        
        Ok(())
    }
}

impl ConfigurationPaths {
    /// Create new configuration paths
    pub fn new(project_path: Option<PathBuf>) -> QmsResult<Self> {
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());

        let qms_config_dir = PathBuf::from(home_dir).join(".qms");

        Ok(Self {
            global_config_dir: qms_config_dir.join("global"),
            user_config_dir: qms_config_dir.join("users"),
            project_config_file: project_path.unwrap_or_else(|| PathBuf::from(".")).join("qms.config.json"),
            interface_config_dir: qms_config_dir.join("interfaces"),
        })
    }
}

/// JSON Configuration Provider
///
/// Handles loading and saving JSON configuration files.
pub struct JsonConfigProvider;

impl JsonConfigProvider {
    pub fn new() -> Self {
        Self
    }
}

impl ConfigProvider for JsonConfigProvider {
    fn load_config(&self, source_path: &Path) -> QmsResult<ConfigurationData> {
        if !source_path.exists() {
            return Err(QmsError::not_found(&format!("Configuration file not found: {}", source_path.display())));
        }

        let content = std::fs::read_to_string(source_path)
            .map_err(|e| QmsError::io_error(&format!("Failed to read config file: {}", e)))?;

        // Simplified JSON parsing - in real implementation would use proper JSON parser
        let mut values = HashMap::new();

        // For demonstration, just parse simple key=value pairs
        for line in content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                values.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        Ok(ConfigurationData {
            values,
            metadata: ConfigMetadata {
                version: "1.0.0".to_string(),
                last_modified: crate::utils::current_timestamp(),
                modified_by: None,
                schema_version: "1.0".to_string(),
            },
            source: ConfigSource::Global, // Would be determined from context
        })
    }

    fn save_config(&self, config: &ConfigurationData, target_path: &Path) -> QmsResult<()> {
        // Create directory if it doesn't exist
        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                QmsError::io_error(&format!("Failed to create config directory: {}", e))
            })?;
        }

        // Simplified JSON writing - in real implementation would use proper JSON serialization
        let mut content = String::new();
        for (key, value) in &config.values {
            content.push_str(&format!("{}={}\n", key, value));
        }

        std::fs::write(target_path, content).map_err(|e| {
            QmsError::io_error(&format!("Failed to write config file: {}", e))
        })?;

        Ok(())
    }

    fn exists(&self, source_path: &Path) -> bool {
        source_path.exists()
    }

    fn priority(&self) -> u32 {
        100 // Medium priority
    }
}

/// Environment Configuration Provider
///
/// Handles loading configuration from environment variables.
pub struct EnvironmentConfigProvider;

impl EnvironmentConfigProvider {
    pub fn new() -> Self {
        Self
    }
}

impl ConfigProvider for EnvironmentConfigProvider {
    fn load_config(&self, _source_path: &Path) -> QmsResult<ConfigurationData> {
        let mut values = HashMap::new();

        // Load QMS-specific environment variables
        for (key, value) in std::env::vars() {
            if key.starts_with("QMS_") {
                let config_key = key.strip_prefix("QMS_").unwrap().to_lowercase();
                values.insert(config_key, value);
            }
        }

        Ok(ConfigurationData {
            values,
            metadata: ConfigMetadata {
                version: "1.0.0".to_string(),
                last_modified: crate::utils::current_timestamp(),
                modified_by: None,
                schema_version: "1.0".to_string(),
            },
            source: ConfigSource::Environment,
        })
    }

    fn save_config(&self, _config: &ConfigurationData, _target_path: &Path) -> QmsResult<()> {
        // Environment variables cannot be saved persistently
        Err(QmsError::domain_error("Cannot save environment configuration"))
    }

    fn exists(&self, _source_path: &Path) -> bool {
        true // Environment is always available
    }

    fn priority(&self) -> u32 {
        200 // High priority
    }
}

/// Theme Configuration Validator
///
/// Validates theme configuration values.
pub struct ThemeValidator;

impl ThemeValidator {
    pub fn new() -> Self {
        Self
    }
}

impl ConfigValidator for ThemeValidator {
    fn validate(&self, key: &str, value: &str) -> QmsResult<ValidationResult> {
        if key != "theme" {
            return Ok(ValidationResult {
                is_valid: true,
                errors: Vec::new(),
                warnings: Vec::new(),
                suggestions: Vec::new(),
            });
        }

        let valid_themes = vec!["default", "dark", "light", "high_contrast", "colorblind"];

        if valid_themes.contains(&value) {
            Ok(ValidationResult {
                is_valid: true,
                errors: Vec::new(),
                warnings: Vec::new(),
                suggestions: Vec::new(),
            })
        } else {
            Ok(ValidationResult {
                is_valid: false,
                errors: vec![format!("Invalid theme: {}", value)],
                warnings: Vec::new(),
                suggestions: valid_themes.into_iter().map(|t| format!("Use '{}'", t)).collect(),
            })
        }
    }

    fn get_rules(&self, key: &str) -> Vec<ValidationRule> {
        if key == "theme" {
            vec![ValidationRule {
                name: "theme_enum".to_string(),
                rule_type: ValidationRuleType::Enum(vec![
                    "default".to_string(),
                    "dark".to_string(),
                    "light".to_string(),
                    "high_contrast".to_string(),
                    "colorblind".to_string(),
                ]),
                parameters: HashMap::new(),
                error_message: "Theme must be one of: default, dark, light, high_contrast, colorblind".to_string(),
            }]
        } else {
            Vec::new()
        }
    }
}

/// Language Configuration Validator
///
/// Validates language/locale configuration values.
pub struct LanguageValidator;

impl LanguageValidator {
    pub fn new() -> Self {
        Self
    }
}

impl ConfigValidator for LanguageValidator {
    fn validate(&self, key: &str, value: &str) -> QmsResult<ValidationResult> {
        if key != "language" && key != "locale" {
            return Ok(ValidationResult {
                is_valid: true,
                errors: Vec::new(),
                warnings: Vec::new(),
                suggestions: Vec::new(),
            });
        }

        // Simple language code validation (ISO 639-1)
        let valid_languages = vec!["en", "es", "fr", "de", "it", "pt", "ja", "zh", "ko"];

        if valid_languages.contains(&value) {
            Ok(ValidationResult {
                is_valid: true,
                errors: Vec::new(),
                warnings: Vec::new(),
                suggestions: Vec::new(),
            })
        } else {
            Ok(ValidationResult {
                is_valid: false,
                errors: vec![format!("Invalid language code: {}", value)],
                warnings: Vec::new(),
                suggestions: valid_languages.into_iter().map(|l| format!("Use '{}'", l)).collect(),
            })
        }
    }

    fn get_rules(&self, key: &str) -> Vec<ValidationRule> {
        if key == "language" || key == "locale" {
            vec![ValidationRule {
                name: "language_code".to_string(),
                rule_type: ValidationRuleType::Regex("^[a-z]{2}$".to_string()),
                parameters: HashMap::new(),
                error_message: "Language must be a valid ISO 639-1 code (e.g., 'en', 'es')".to_string(),
            }]
        } else {
            Vec::new()
        }
    }
}

/// Path Configuration Validator
///
/// Validates file and directory path configuration values.
pub struct PathValidator;

impl PathValidator {
    pub fn new() -> Self {
        Self
    }
}

impl ConfigValidator for PathValidator {
    fn validate(&self, key: &str, value: &str) -> QmsResult<ValidationResult> {
        if !key.contains("path") && !key.contains("dir") && !key.contains("file") {
            return Ok(ValidationResult {
                is_valid: true,
                errors: Vec::new(),
                warnings: Vec::new(),
                suggestions: Vec::new(),
            });
        }

        let path = PathBuf::from(value);
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();

        // Check if path is absolute or relative
        if !path.is_absolute() {
            warnings.push("Path is relative, consider using absolute path".to_string());
        }

        // Check if path exists (for existing paths)
        if key.contains("existing") && !path.exists() {
            errors.push(format!("Path does not exist: {}", value));
            suggestions.push("Create the directory or check the path".to_string());
        }

        // Check if parent directory exists (for new files)
        if key.contains("file") {
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    warnings.push("Parent directory does not exist".to_string());
                    suggestions.push(format!("Create parent directory: {}", parent.display()));
                }
            }
        }

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            suggestions,
        })
    }

    fn get_rules(&self, key: &str) -> Vec<ValidationRule> {
        if key.contains("path") || key.contains("dir") || key.contains("file") {
            vec![ValidationRule {
                name: "path_format".to_string(),
                rule_type: ValidationRuleType::Custom("path_validation".to_string()),
                parameters: HashMap::new(),
                error_message: "Path must be a valid file system path".to_string(),
            }]
        } else {
            Vec::new()
        }
    }
}

// Default implementations for configuration structures

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            locale: "en".to_string(),
            default_project: None,
            interface_prefs: HashMap::new(),
            accessibility: crate::interfaces::unified_context::AccessibilitySettings {
                high_contrast: false,
                screen_reader_enabled: false,
                font_scale: 1.0,
                color_blind_friendly: false,
            },
        }
    }
}

impl Default for InterfaceSettings {
    fn default() -> Self {
        Self {
            display_config: crate::interfaces::unified_context::DisplayConfiguration {
                color_scheme: "default".to_string(),
                font_family: "monospace".to_string(),
                font_size: 12,
                layout_prefs: HashMap::new(),
            },
            command_aliases: HashMap::new(),
            shortcuts: HashMap::new(),
            behavior_settings: crate::interfaces::unified_context::BehaviorSettings {
                auto_save: true,
                confirm_actions: true,
                auto_complete: true,
                history_size: 100,
            },
        }
    }
}
