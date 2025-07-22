//! Enhanced Risk Manager with SOLID Principles
//! 
//! This module implements an enhanced risk manager that follows all SOLID principles
//! and uses dependency injection for better testability and maintainability.
//! 
//! SOLID Principles Applied:
//! - Single Responsibility: Each component has one clear responsibility
//! - Open/Closed: Extensible through strategy and interface implementations
//! - Liskov Substitution: All implementations can be substituted
//! - Interface Segregation: Uses focused interfaces for different operations
//! - Dependency Inversion: Depends on abstractions, not concrete implementations

use crate::prelude::*;
use super::{
    RiskSeverity, RiskOccurrence, RiskDetectability,
    RiskAssessmentStrategy, RiskAssessmentContext, RiskAssessmentResult,
    RiskReader, RiskWriter, RiskIndexManager, RiskSearcher, RiskBackupManager,
    RiskSearchCriteria, FileRiskStorage
};
use crate::modules::audit_logger::audit_log_action;
use std::path::Path;

/// Enhanced Risk Manager using Dependency Inversion Principle
/// Depends on abstractions (traits) rather than concrete implementations
pub struct EnhancedRiskManager {
    // Dependency Inversion: Inject storage dependencies
    reader: Box<dyn RiskReader>,
    writer: Box<dyn RiskWriter>,
    index_manager: Box<dyn RiskIndexManager>,
    searcher: Box<dyn RiskSearcher>,
    backup_manager: Box<dyn RiskBackupManager>,
    
    // Strategy pattern for risk assessment
    assessment_context: RiskAssessmentContext,
    
    // Configuration
    project_path: std::path::PathBuf,
}

impl EnhancedRiskManager {
    /// Create new enhanced risk manager with dependency injection
    /// Dependency Inversion Principle: Accept abstractions as parameters
    pub fn new_with_dependencies(
        project_path: &Path,
        reader: Box<dyn RiskReader>,
        writer: Box<dyn RiskWriter>,
        index_manager: Box<dyn RiskIndexManager>,
        searcher: Box<dyn RiskSearcher>,
        backup_manager: Box<dyn RiskBackupManager>,
        assessment_strategy: Box<dyn RiskAssessmentStrategy>,
    ) -> Self {
        Self {
            reader,
            writer,
            index_manager,
            searcher,
            backup_manager,
            assessment_context: RiskAssessmentContext::new(assessment_strategy),
            project_path: project_path.to_path_buf(),
        }
    }
    
    /// Create new enhanced risk manager with default file-based storage
    /// Factory method for common use case
    pub fn new_with_file_storage(project_path: &Path, strategy_type: &str) -> QmsResult<Self> {
        let storage = FileRiskStorage::new(project_path)?;
        storage.initialize()?;
        
        let assessment_context = RiskAssessmentContext::with_strategy_type(strategy_type)?;
        
        Ok(Self {
            reader: Box::new(storage),
            writer: Box::new(FileRiskStorage::new(project_path)?),
            index_manager: Box::new(FileRiskStorage::new(project_path)?),
            searcher: Box::new(FileRiskStorage::new(project_path)?),
            backup_manager: Box::new(FileRiskStorage::new(project_path)?),
            assessment_context,
            project_path: project_path.to_path_buf(),
        })
    }
    
    /// Create risk with enhanced assessment
    /// Single Responsibility Principle: Focused on risk creation with assessment
    pub fn create_risk_with_assessment(
        &self,
        description: String,
        severity: RiskSeverity,
        occurrence: RiskOccurrence,
        detectability: RiskDetectability,
    ) -> QmsResult<(RiskItem, RiskAssessmentResult)> {
        // Perform risk assessment using strategy pattern
        let assessment_result = self.assessment_context.assess_risk(&severity, &occurrence, &detectability)?;

        // Create risk item with assessment results
        let risk = RiskItem {
            id: crate::utils::generate_uuid(),
            description,
            severity: severity as u8,
            occurrence: occurrence as u8,
            detectability: detectability as u8,
            rpn: assessment_result.rpn as u16, // Convert u32 to u16
            mitigation: None,
            created_at: crate::utils::current_timestamp(),
            updated_at: crate::utils::current_timestamp(),
        };

        // Save risk using injected writer
        self.writer.save_risk(&risk)?;

        // Audit log the creation
        audit_log_action("RISK_CREATED_WITH_ASSESSMENT", "Risk", &risk.id)?;

        Ok((risk, assessment_result))
    }
    
    /// Load risk using injected reader
    /// Interface Segregation Principle: Uses focused reader interface
    pub fn load_risk(&self, risk_id: &str) -> QmsResult<RiskItem> {
        self.reader.load_risk(risk_id)
    }
    
    /// Load all risks using injected reader
    pub fn load_all_risks(&self) -> QmsResult<Vec<RiskItem>> {
        self.reader.load_all_risks()
    }
    
    /// Search risks using injected searcher
    /// Interface Segregation Principle: Uses focused searcher interface
    pub fn search_risks(&self, criteria: &RiskSearchCriteria) -> QmsResult<Vec<RiskItem>> {
        self.searcher.search_risks(criteria)
    }
    
    /// Find high-priority risks using business logic
    /// Single Responsibility Principle: Focused on business logic for high-priority risks
    pub fn find_high_priority_risks(&self) -> QmsResult<Vec<RiskItem>> {
        let all_risks = self.reader.load_all_risks()?;
        let high_priority_risks = all_risks.into_iter()
            .filter(|risk| risk.rpn >= 50) // High priority based on RPN threshold
            .collect();

        Ok(high_priority_risks)
    }
    
    /// Update risk with re-assessment
    /// Single Responsibility Principle: Focused on risk updates with assessment
    pub fn update_risk_with_reassessment(&self, mut risk: RiskItem) -> QmsResult<RiskAssessmentResult> {
        // Convert u8 values back to enums for assessment
        let severity = match risk.severity {
            5 => RiskSeverity::Catastrophic,
            4 => RiskSeverity::Critical,
            3 => RiskSeverity::Major,
            2 => RiskSeverity::Minor,
            _ => RiskSeverity::Negligible,
        };

        let occurrence = match risk.occurrence {
            5 => RiskOccurrence::Frequent,
            4 => RiskOccurrence::Probable,
            3 => RiskOccurrence::Occasional,
            2 => RiskOccurrence::Remote,
            _ => RiskOccurrence::Improbable,
        };

        let detectability = match risk.detectability {
            5 => RiskDetectability::VeryLow,
            4 => RiskDetectability::Low,
            3 => RiskDetectability::Moderate,
            2 => RiskDetectability::High,
            _ => RiskDetectability::VeryHigh,
        };

        // Re-assess risk with current strategy
        let assessment_result = self.assessment_context.assess_risk(&severity, &occurrence, &detectability)?;

        // Update risk with new assessment
        risk.rpn = assessment_result.rpn as u16; // Convert u32 to u16
        risk.updated_at = crate::utils::current_timestamp();

        // Save updated risk
        self.writer.save_risk(&risk)?;

        // Audit log the update
        audit_log_action("RISK_REASSESSED", "Risk", &risk.id)?;

        Ok(assessment_result)
    }
    
    /// Delete risk using injected writer
    pub fn delete_risk(&self, risk_id: &str) -> QmsResult<()> {
        self.writer.delete_risk(risk_id)?;
        audit_log_action("RISK_DELETED", "Risk", risk_id)?;
        Ok(())
    }
    
    /// Change assessment strategy at runtime
    /// Open/Closed Principle: Can extend with new strategies without modification
    pub fn change_assessment_strategy(&mut self, strategy: Box<dyn RiskAssessmentStrategy>) {
        self.assessment_context.set_strategy(strategy);
        
        // Audit log the strategy change
        let strategy_name = self.assessment_context.current_strategy_name();
        let _ = audit_log_action("RISK_ASSESSMENT_STRATEGY_CHANGED", "System", strategy_name);
    }
    
    /// Get current assessment strategy name
    pub fn current_assessment_strategy(&self) -> &str {
        self.assessment_context.current_strategy_name()
    }
    
    /// Create backup using injected backup manager
    /// Interface Segregation Principle: Uses focused backup interface
    pub fn create_backup(&self, backup_path: &Path) -> QmsResult<()> {
        self.backup_manager.create_backup(backup_path)?;
        audit_log_action("RISK_BACKUP_CREATED", "System", &backup_path.display().to_string())?;
        Ok(())
    }
    
    /// Restore from backup using injected backup manager
    pub fn restore_from_backup(&self, backup_path: &Path) -> QmsResult<()> {
        // Verify backup before restoring
        if !self.backup_manager.verify_backup(backup_path)? {
            return Err(QmsError::validation_error("Backup verification failed"));
        }
        
        self.backup_manager.restore_from_backup(backup_path)?;
        audit_log_action("RISK_BACKUP_RESTORED", "System", &backup_path.display().to_string())?;
        Ok(())
    }
    
    /// Get risk statistics using injected reader
    pub fn get_risk_statistics(&self) -> QmsResult<RiskStatistics> {
        let all_risks = self.reader.load_all_risks()?;
        
        let mut stats = RiskStatistics {
            total_risks: all_risks.len(),
            unacceptable_risks: 0,
            alarp_risks: 0,
            acceptable_risks: 0,
            overdue_risks: 0,
            by_severity: std::collections::HashMap::new(),
        };
        
        let current_time = crate::utils::current_timestamp();
        
        for risk in &all_risks {
            // Count by risk level based on RPN
            match risk.rpn {
                100..=u16::MAX => stats.unacceptable_risks += 1,
                25..=99 => stats.alarp_risks += 1,
                _ => stats.acceptable_risks += 1,
            }

            // For simplicity, we'll skip overdue risk counting since the model doesn't have due_date
            // In a full implementation, this would be tracked separately

            // Count by severity
            let severity_key = format!("Severity_{}", risk.severity);
            *stats.by_severity.entry(severity_key).or_insert(0) += 1;
        }
        
        Ok(stats)
    }
    
    /// Batch process risks with assessment
    /// Single Responsibility Principle: Focused on batch processing
    pub fn batch_assess_risks(&self, risk_ids: &[String]) -> QmsResult<Vec<(RiskItem, RiskAssessmentResult)>> {
        let mut results = Vec::new();

        for risk_id in risk_ids {
            let risk = self.reader.load_risk(risk_id)?;

            // Convert u8 values back to enums for assessment
            let severity = match risk.severity {
                5 => RiskSeverity::Catastrophic,
                4 => RiskSeverity::Critical,
                3 => RiskSeverity::Major,
                2 => RiskSeverity::Minor,
                _ => RiskSeverity::Negligible,
            };

            let occurrence = match risk.occurrence {
                5 => RiskOccurrence::Frequent,
                4 => RiskOccurrence::Probable,
                3 => RiskOccurrence::Occasional,
                2 => RiskOccurrence::Remote,
                _ => RiskOccurrence::Improbable,
            };

            let detectability = match risk.detectability {
                5 => RiskDetectability::VeryLow,
                4 => RiskDetectability::Low,
                3 => RiskDetectability::Moderate,
                2 => RiskDetectability::High,
                _ => RiskDetectability::VeryHigh,
            };

            let assessment = self.assessment_context.assess_risk(&severity, &occurrence, &detectability)?;

            results.push((risk, assessment));
        }

        audit_log_action("RISK_BATCH_ASSESSMENT", "System", &format!("Processed {} risks", risk_ids.len()))?;

        Ok(results)
    }
}

/// Risk statistics for reporting and dashboard
#[derive(Debug, Clone)]
pub struct RiskStatistics {
    pub total_risks: usize,
    pub unacceptable_risks: usize,
    pub alarp_risks: usize,
    pub acceptable_risks: usize,
    pub overdue_risks: usize,
    pub by_severity: std::collections::HashMap<String, usize>,
}

impl RiskStatistics {
    /// Convert to JSON for reporting
    pub fn to_json(&self) -> String {
        // Simple JSON serialization without external dependencies
        let severity_json = self.by_severity.iter()
            .map(|(k, v)| format!(r#""{}": {}"#, k, v))
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            r#"{{
    "total_risks": {},
    "unacceptable_risks": {},
    "alarp_risks": {},
    "acceptable_risks": {},
    "overdue_risks": {},
    "by_severity": {{{}}}
}}"#,
            self.total_risks,
            self.unacceptable_risks,
            self.alarp_risks,
            self.acceptable_risks,
            self.overdue_risks,
            severity_json
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_enhanced_risk_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let manager = EnhancedRiskManager::new_with_file_storage(temp_dir.path(), "iso14971").unwrap();
        
        assert_eq!(manager.current_assessment_strategy(), "ISO14971_Standard");
    }

    #[test]
    fn test_risk_creation_with_assessment() {
        let temp_dir = tempdir().unwrap();
        let manager = EnhancedRiskManager::new_with_file_storage(temp_dir.path(), "iso14971").unwrap();

        let (risk, assessment) = manager.create_risk_with_assessment(
            "Test hazard".to_string(),
            RiskSeverity::Major,
            RiskOccurrence::Occasional,
            RiskDetectability::High,
        ).unwrap();

        assert_eq!(u32::from(risk.rpn), assessment.rpn);
        assert_eq!(assessment.strategy_used, "ISO14971_Standard");
    }

    #[test]
    fn test_strategy_change() {
        let temp_dir = tempdir().unwrap();
        let mut manager = EnhancedRiskManager::new_with_file_storage(temp_dir.path(), "iso14971").unwrap();
        
        assert_eq!(manager.current_assessment_strategy(), "ISO14971_Standard");
        
        let fda_strategy = super::super::assessment_strategy::FDA820AssessmentStrategy;
        manager.change_assessment_strategy(Box::new(fda_strategy));
        
        assert_eq!(manager.current_assessment_strategy(), "FDA_21CFR820_Enhanced");
    }
}
