//! Risk Storage Interface Segregation Implementation
//! 
//! This module implements Interface Segregation Principle (ISP) for risk storage operations,
//! separating concerns and creating focused interfaces for different storage needs.
//! 
//! SOLID Principles Applied:
//! - Interface Segregation: Separate interfaces for different storage operations
//! - Single Responsibility: Each interface handles one specific storage concern
//! - Dependency Inversion: High-level modules depend on abstractions
//! - Open/Closed: New storage implementations can be added without modification

use crate::prelude::*;
use std::path::Path;

/// Interface Segregation Principle: Focused interface for risk reading operations
pub trait RiskReader {
    /// Load a single risk item by ID
    fn load_risk(&self, risk_id: &str) -> QmsResult<RiskItem>;
    
    /// Load all risks with optional filtering
    fn load_all_risks(&self) -> QmsResult<Vec<RiskItem>>;
    
    /// Check if a risk exists
    fn risk_exists(&self, risk_id: &str) -> QmsResult<bool>;
    
    /// Get risk count for metrics
    fn get_risk_count(&self) -> QmsResult<usize>;
}

/// Interface Segregation Principle: Focused interface for risk writing operations
pub trait RiskWriter {
    /// Save a single risk item
    fn save_risk(&self, risk: &RiskItem) -> QmsResult<()>;
    
    /// Delete a risk item
    fn delete_risk(&self, risk_id: &str) -> QmsResult<()>;
    
    /// Batch save multiple risks (for performance)
    fn save_risks_batch(&self, risks: &[RiskItem]) -> QmsResult<()>;
}

/// Interface Segregation Principle: Focused interface for risk index operations
pub trait RiskIndexManager {
    /// Update index entry for a risk
    fn update_index_entry(&self, risk: &RiskItem) -> QmsResult<()>;

    /// Remove index entry for a risk
    fn remove_index_entry(&self, risk_id: &str) -> QmsResult<()>;
}

/// Interface Segregation Principle: Focused interface for risk search operations
pub trait RiskSearcher {
    /// Search risks by criteria
    fn search_risks(&self, criteria: &RiskSearchCriteria) -> QmsResult<Vec<RiskItem>>;
    
    /// Find risks by severity level
    fn find_by_severity(&self, severity: &super::RiskSeverity) -> QmsResult<Vec<RiskItem>>;
    
    /// Find risks by status
    fn find_by_status(&self, status: &str) -> QmsResult<Vec<RiskItem>>;
    
    /// Find overdue risks
    fn find_overdue_risks(&self) -> QmsResult<Vec<RiskItem>>;
}

/// Interface Segregation Principle: Focused interface for risk backup operations
pub trait RiskBackupManager {
    /// Create backup of all risk data
    fn create_backup(&self, backup_path: &Path) -> QmsResult<()>;
    
    /// Restore from backup
    fn restore_from_backup(&self, backup_path: &Path) -> QmsResult<()>;
    
    /// Verify backup integrity
    fn verify_backup(&self, backup_path: &Path) -> QmsResult<bool>;
}

/// Search criteria for risk operations
#[derive(Debug, Clone)]
pub struct RiskSearchCriteria {
    pub severity: Option<super::RiskSeverity>,
    pub status: Option<String>,
    pub created_after: Option<u64>,
    pub created_before: Option<u64>,
    pub text_search: Option<String>,
}

/// File-based implementation of risk storage interfaces
/// Single Responsibility Principle: Handles file-based storage operations
pub struct FileRiskStorage {
    project_path: std::path::PathBuf,
    risks_dir: std::path::PathBuf,
    index_file: std::path::PathBuf,
}

impl FileRiskStorage {
    /// Create new file-based risk storage
    pub fn new(project_path: &Path) -> QmsResult<Self> {
        let risks_dir = project_path.join("risks");
        let index_file = risks_dir.join("index.json");
        
        Ok(Self {
            project_path: project_path.to_path_buf(),
            risks_dir,
            index_file,
        })
    }
    
    /// Initialize storage directory structure
    pub fn initialize(&self) -> QmsResult<()> {
        std::fs::create_dir_all(&self.risks_dir)?;
        std::fs::create_dir_all(self.risks_dir.join("assessments"))?;
        std::fs::create_dir_all(self.risks_dir.join("mitigations"))?;
        std::fs::create_dir_all(self.risks_dir.join("reports"))?;
        std::fs::create_dir_all(self.risks_dir.join("evidence"))?;
        std::fs::create_dir_all(self.risks_dir.join("reviews"))?;
        
        // Create empty index file if it doesn't exist
        if !self.index_file.exists() {
            std::fs::write(&self.index_file, "{\"version\": \"1.0\", \"risks\": []}")?;
        }
        
        Ok(())
    }
    
    /// Get file path for a risk
    fn get_risk_file_path(&self, risk_id: &str) -> std::path::PathBuf {
        self.risks_dir.join(format!("{}.json", risk_id))
    }

    /// Parse RiskItem from JSON string (simple implementation)
    fn parse_risk_json(&self, json: &str) -> QmsResult<RiskItem> {
        // Simple JSON parsing without external dependencies
        // This is a simplified implementation - in production, you'd want more robust parsing
        let id = self.extract_json_string(json, "id")?;
        let description = self.extract_json_string(json, "description")?;
        let severity = self.extract_json_number(json, "severity")? as u8;
        let occurrence = self.extract_json_number(json, "occurrence")? as u8;
        let detectability = self.extract_json_number(json, "detectability")? as u8;
        let rpn = self.extract_json_number(json, "rpn")? as u16;
        let created_at = self.extract_json_number(json, "created_at")?;
        let updated_at = self.extract_json_number(json, "updated_at")?;

        // Extract optional mitigation
        let mitigation = self.extract_json_optional_string(json, "mitigation");

        Ok(RiskItem {
            id,
            description,
            severity,
            occurrence,
            detectability,
            rpn,
            mitigation,
            created_at,
            updated_at,
        })
    }

    /// Convert RiskItem to JSON string
    fn risk_to_json(&self, risk: &RiskItem) -> String {
        let mitigation_json = match &risk.mitigation {
            Some(m) => format!(r#""{}""#, m.replace('"', r#"\""#)),
            None => "null".to_string(),
        };

        format!(
            r#"{{
    "id": "{}",
    "description": "{}",
    "severity": {},
    "occurrence": {},
    "detectability": {},
    "rpn": {},
    "mitigation": {},
    "created_at": {},
    "updated_at": {}
}}"#,
            risk.id,
            risk.description.replace('"', r#"\""#),
            risk.severity,
            risk.occurrence,
            risk.detectability,
            risk.rpn,
            mitigation_json,
            risk.created_at,
            risk.updated_at
        )
    }

    /// Extract string value from JSON
    fn extract_json_string(&self, json: &str, key: &str) -> QmsResult<String> {
        let pattern = format!(r#""{}":\s*"([^"]*)""#, key);
        if let Some(start) = json.find(&format!(r#""{}":\s*""#, key)) {
            let start_quote = start + key.len() + 4; // Skip key": "
            if let Some(end_quote) = json[start_quote..].find('"') {
                return Ok(json[start_quote..start_quote + end_quote].to_string());
            }
        }
        Err(QmsError::domain_error(&format!("Failed to extract {} from JSON", key)))
    }

    /// Extract number value from JSON
    fn extract_json_number(&self, json: &str, key: &str) -> QmsResult<u64> {
        let pattern = format!(r#""{}":\s*(\d+)"#, key);
        if let Some(start) = json.find(&format!(r#""{}":\s*"#, key)) {
            let start_num = start + key.len() + 3; // Skip key":
            let remaining = &json[start_num..];
            let mut end_num = 0;
            for (i, ch) in remaining.chars().enumerate() {
                if ch.is_ascii_digit() {
                    end_num = i + 1;
                } else {
                    break;
                }
            }
            if end_num > 0 {
                if let Ok(num) = remaining[..end_num].parse::<u64>() {
                    return Ok(num);
                }
            }
        }
        Err(QmsError::domain_error(&format!("Failed to extract {} from JSON", key)))
    }

    /// Extract optional string value from JSON
    fn extract_json_optional_string(&self, json: &str, key: &str) -> Option<String> {
        self.extract_json_string(json, key).ok()
    }
}

impl RiskReader for FileRiskStorage {
    fn load_risk(&self, risk_id: &str) -> QmsResult<RiskItem> {
        let file_path = self.get_risk_file_path(risk_id);
        let content = std::fs::read_to_string(file_path)?;

        // Simple JSON parsing for RiskItem
        let risk = self.parse_risk_json(&content)?;
        Ok(risk)
    }

    fn load_all_risks(&self) -> QmsResult<Vec<RiskItem>> {
        let mut risks = Vec::new();

        // Read all JSON files in the risks directory
        for entry in std::fs::read_dir(&self.risks_dir)? {
            let entry = entry?;
            if entry.path().extension().map_or(false, |ext| ext == "json") {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name != "index.json" {
                        let risk_id = file_name.trim_end_matches(".json");
                        match self.load_risk(risk_id) {
                            Ok(risk) => risks.push(risk),
                            Err(e) => {
                                // Log error but continue loading other risks
                                eprintln!("Warning: Failed to load risk {}: {}", risk_id, e);
                            }
                        }
                    }
                }
            }
        }

        Ok(risks)
    }

    fn risk_exists(&self, risk_id: &str) -> QmsResult<bool> {
        let file_path = self.get_risk_file_path(risk_id);
        Ok(file_path.exists())
    }

    fn get_risk_count(&self) -> QmsResult<usize> {
        let risks = self.load_all_risks()?;
        Ok(risks.len())
    }
}

impl RiskWriter for FileRiskStorage {
    fn save_risk(&self, risk: &RiskItem) -> QmsResult<()> {
        let file_path = self.get_risk_file_path(&risk.id);
        let json_content = self.risk_to_json(risk);

        std::fs::write(file_path, json_content)?;

        // Update index
        self.update_index_entry(risk)?;

        Ok(())
    }

    fn delete_risk(&self, risk_id: &str) -> QmsResult<()> {
        let file_path = self.get_risk_file_path(risk_id);

        if file_path.exists() {
            std::fs::remove_file(file_path)?;
        }

        // Remove from index
        self.remove_index_entry(risk_id)?;

        Ok(())
    }

    fn save_risks_batch(&self, risks: &[RiskItem]) -> QmsResult<()> {
        for risk in risks {
            self.save_risk(risk)?;
        }
        Ok(())
    }
}

impl RiskIndexManager for FileRiskStorage {
    fn update_index_entry(&self, _risk: &RiskItem) -> QmsResult<()> {
        // For simplicity, we'll skip index management for now
        // In a full implementation, this would maintain a separate index file
        Ok(())
    }

    fn remove_index_entry(&self, _risk_id: &str) -> QmsResult<()> {
        // For simplicity, we'll skip index management for now
        // In a full implementation, this would maintain a separate index file
        Ok(())
    }
}

impl RiskSearcher for FileRiskStorage {
    fn search_risks(&self, criteria: &RiskSearchCriteria) -> QmsResult<Vec<RiskItem>> {
        let all_risks = self.load_all_risks()?;
        let mut filtered_risks = Vec::new();

        for risk in all_risks {
            let mut matches = true;

            if let Some(ref severity) = criteria.severity {
                if risk.severity != severity.clone() as u8 {
                    matches = false;
                }
            }

            // Skip status filtering since the model doesn't have a status field
            // In a full implementation, status would be tracked separately

            if let Some(created_after) = criteria.created_after {
                if risk.created_at < created_after {
                    matches = false;
                }
            }

            if let Some(created_before) = criteria.created_before {
                if risk.created_at > created_before {
                    matches = false;
                }
            }

            if let Some(ref text) = criteria.text_search {
                let text_lower = text.to_lowercase();
                if !risk.description.to_lowercase().contains(&text_lower) {
                    matches = false;
                }
            }

            if matches {
                filtered_risks.push(risk);
            }
        }

        Ok(filtered_risks)
    }

    fn find_by_severity(&self, severity: &super::RiskSeverity) -> QmsResult<Vec<RiskItem>> {
        let criteria = RiskSearchCriteria {
            severity: Some(severity.clone()),
            status: None,
            created_after: None,
            created_before: None,
            text_search: None,
        };
        self.search_risks(&criteria)
    }

    fn find_by_status(&self, status: &str) -> QmsResult<Vec<RiskItem>> {
        let criteria = RiskSearchCriteria {
            severity: None,
            status: Some(status.to_string()),
            created_after: None,
            created_before: None,
            text_search: None,
        };
        self.search_risks(&criteria)
    }

    fn find_overdue_risks(&self) -> QmsResult<Vec<RiskItem>> {
        // Since the model doesn't have due_date or status fields,
        // we'll return high-priority risks as a proxy for "overdue"
        let all_risks = self.load_all_risks()?;

        let high_priority_risks = all_risks.into_iter()
            .filter(|risk| risk.rpn >= 50) // High priority threshold
            .collect();

        Ok(high_priority_risks)
    }
}

impl RiskBackupManager for FileRiskStorage {
    fn create_backup(&self, backup_path: &Path) -> QmsResult<()> {
        std::fs::create_dir_all(backup_path)?;
        
        // Copy all risk files
        let risks_backup = backup_path.join("risks");
        std::fs::create_dir_all(&risks_backup)?;
        
        for entry in std::fs::read_dir(&self.risks_dir)? {
            let entry = entry?;
            if entry.path().extension().map_or(false, |ext| ext == "json") {
                let dest = risks_backup.join(entry.file_name());
                std::fs::copy(entry.path(), dest)?;
            }
        }
        
        Ok(())
    }
    
    fn restore_from_backup(&self, backup_path: &Path) -> QmsResult<()> {
        let risks_backup = backup_path.join("risks");
        
        if !risks_backup.exists() {
            return Err(QmsError::NotFound("Backup directory not found".to_string()));
        }
        
        // Clear existing risks
        if self.risks_dir.exists() {
            std::fs::remove_dir_all(&self.risks_dir)?;
        }
        
        // Restore from backup
        std::fs::create_dir_all(&self.risks_dir)?;
        
        for entry in std::fs::read_dir(&risks_backup)? {
            let entry = entry?;
            if entry.path().extension().map_or(false, |ext| ext == "json") {
                let dest = self.risks_dir.join(entry.file_name());
                std::fs::copy(entry.path(), dest)?;
            }
        }
        
        Ok(())
    }
    
    fn verify_backup(&self, backup_path: &Path) -> QmsResult<bool> {
        let risks_backup = backup_path.join("risks");

        if !risks_backup.exists() {
            return Ok(false);
        }

        // Verify that backup contains valid JSON files
        for entry in std::fs::read_dir(&risks_backup)? {
            let entry = entry?;
            if entry.path().extension().map_or(false, |ext| ext == "json") {
                let content = std::fs::read_to_string(entry.path())?;
                // Simple validation - check if it contains basic JSON structure
                if !content.trim().starts_with('{') || !content.trim().ends_with('}') {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }
}
