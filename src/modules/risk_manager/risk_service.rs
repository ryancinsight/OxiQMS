//! Risk Service - DIP Abstraction Layer
//! 
//! DIP (Dependency Inversion Principle): High-level modules should not depend on low-level modules
//! Both should depend on abstractions. This module provides the abstraction layer for risk services.
//! 
//! Medical Device Compliance: Maintains ISO 14971 compliance through abstraction

use crate::prelude::*;
use super::risk::{RiskItem, RiskSeverity, RiskOccurrence, RiskDetectability, RiskIndexEntry, RiskFilter};

/// Risk Service Trait - DIP: Abstraction for risk management operations
/// 
/// This trait abstracts risk management operations, enabling:
/// - Dependency injection in controllers
/// - Multiple implementations (file-based, database, cloud, etc.)
/// - Easy testing with mock implementations
/// - Clear separation between high-level controllers and low-level implementations
pub trait RiskService {
    /// Create a new risk item
    fn create_risk(&mut self, description: &str, situation: &str, harm: &str) -> QmsResult<RiskItem>;
    
    /// Assess a risk by updating its parameters
    fn assess_risk(
        &mut self,
        risk_id: &str,
        severity: Option<RiskSeverity>,
        occurrence: Option<RiskOccurrence>,
        detectability: Option<RiskDetectability>,
    ) -> QmsResult<RiskItem>;
    
    /// Get a risk by ID
    fn get_risk(&self, risk_id: &str) -> QmsResult<RiskItem>;
    
    /// List all risks with optional filtering
    fn list_risks(&self, filter: Option<RiskFilter>) -> QmsResult<Vec<RiskIndexEntry>>;
    
    /// Get risk statistics
    fn get_risk_statistics(&self) -> QmsResult<RiskStatistics>;
    
    /// Check if risk service is healthy
    fn health_check(&self) -> QmsResult<ServiceHealth>;
}

/// Risk Statistics Structure
#[derive(Debug, Clone)]
pub struct RiskStatistics {
    pub total_risks: usize,
    pub high_priority_risks: usize,
    pub unacceptable_risks: usize,
    pub alarp_risks: usize,
    pub acceptable_risks: usize,
}

impl Default for RiskStatistics {
    fn default() -> Self {
        Self {
            total_risks: 0,
            high_priority_risks: 0,
            unacceptable_risks: 0,
            alarp_risks: 0,
            acceptable_risks: 0,
        }
    }
}

/// Service Health Status
#[derive(Debug, Clone)]
pub struct ServiceHealth {
    pub is_healthy: bool,
    pub message: String,
    pub last_check: String,
}

impl Default for ServiceHealth {
    fn default() -> Self {
        Self {
            is_healthy: true,
            message: "Service operational".to_string(),
            last_check: crate::utils::current_timestamp_string(),
        }
    }
}

/// Concrete Risk Service Implementation - DIP: Concrete implementation
/// 
/// This implementation uses the existing RiskManager but provides the abstraction
/// layer needed for dependency injection
pub struct FileRiskService {
    risk_manager: super::risk::RiskManager,
}

impl FileRiskService {
    /// Create new file-based risk service
    pub fn new(project_path: &std::path::Path) -> QmsResult<Self> {
        let risk_manager = super::risk::RiskManager::new(project_path)?;
        Ok(Self { risk_manager })
    }
}

impl RiskService for FileRiskService {
    fn create_risk(&mut self, description: &str, situation: &str, harm: &str) -> QmsResult<RiskItem> {
        self.risk_manager.create_risk(description, situation, harm)
    }
    
    fn assess_risk(
        &mut self,
        risk_id: &str,
        severity: Option<RiskSeverity>,
        occurrence: Option<RiskOccurrence>,
        detectability: Option<RiskDetectability>,
    ) -> QmsResult<RiskItem> {
        self.risk_manager.assess_risk(risk_id, severity, occurrence, detectability)
    }
    
    fn get_risk(&self, risk_id: &str) -> QmsResult<RiskItem> {
        self.risk_manager.load_risk(risk_id)
    }
    
    fn list_risks(&self, filter: Option<RiskFilter>) -> QmsResult<Vec<RiskIndexEntry>> {
        self.risk_manager.list_risks(filter.as_ref())
    }
    
    fn get_risk_statistics(&self) -> QmsResult<RiskStatistics> {
        let risks = self.risk_manager.list_risks(None)?;
        
        let mut stats = RiskStatistics::default();
        stats.total_risks = risks.len();
        
        for risk in &risks {
            if risk.rpn >= 50 {
                stats.high_priority_risks += 1;
            }
            
            // Categorize by risk level using ISO 14971 thresholds
            match risk.rpn {
                rpn if rpn >= crate::constants::iso_14971::RPN_UNACCEPTABLE_THRESHOLD => {
                    stats.unacceptable_risks += 1;
                }
                rpn if rpn >= crate::constants::iso_14971::RPN_ALARP_THRESHOLD => {
                    stats.alarp_risks += 1;
                }
                _ => {
                    stats.acceptable_risks += 1;
                }
            }
        }
        
        Ok(stats)
    }
    
    fn health_check(&self) -> QmsResult<ServiceHealth> {
        // Perform basic health checks
        match self.risk_manager.list_risks(None) {
            Ok(_) => Ok(ServiceHealth {
                is_healthy: true,
                message: "Risk service operational - file system accessible".to_string(),
                last_check: crate::utils::current_timestamp_string(),
            }),
            Err(e) => Ok(ServiceHealth {
                is_healthy: false,
                message: format!("Risk service error: {}", e),
                last_check: crate::utils::current_timestamp_string(),
            }),
        }
    }
}

/// Mock Risk Service for Testing - DIP: Alternative implementation
/// 
/// This mock implementation enables testing without file system dependencies
#[cfg(test)]
pub struct MockRiskService {
    risks: std::collections::HashMap<String, RiskItem>,
    next_id: usize,
}

#[cfg(test)]
impl MockRiskService {
    pub fn new() -> Self {
        Self {
            risks: std::collections::HashMap::new(),
            next_id: 1,
        }
    }
}

#[cfg(test)]
impl RiskService for MockRiskService {
    fn create_risk(&mut self, description: &str, _situation: &str, _harm: &str) -> QmsResult<RiskItem> {
        let id = format!("MOCK-{:03}", self.next_id);
        self.next_id += 1;
        
        let risk = RiskItem {
            id: id.clone(),
            project_id: "mock-project".to_string(),
            hazard_id: format!("HAZ-{:03}", self.next_id - 1),
            hazard_description: description.to_string(),
            hazardous_situation: "Mock situation".to_string(),
            harm: "Mock harm".to_string(),
            severity: RiskSeverity::Minor,
            occurrence: RiskOccurrence::Remote,
            detectability: super::risk::RiskDetectability::High,
            risk_priority_number: 6, // 2 * 2 * 1.5 (rounded)
            initial_risk_level: super::risk::RiskLevel::Acceptable,
            mitigation_measures: Vec::new(),
            residual_severity: RiskSeverity::Minor,
            residual_occurrence: RiskOccurrence::Remote,
            residual_detectability: super::risk::RiskDetectability::High,
            residual_rpn: 6,
            residual_risk_level: super::risk::RiskLevel::Acceptable,
            residual_risk_justification: None,
            residual_risk_approved: false,
            residual_risk_approved_by: None,
            residual_risk_approval_date: None,
            verification_method: "Mock verification".to_string(),
            verification_status: super::risk::VerificationStatus::Planned,
            verification_evidence: Vec::new(),
            category: "Safety".to_string(),
            source: "Mock".to_string(),
            assigned_to: None,
            due_date: None,
            priority: "Low".to_string(),
            risk_status: super::risk::RiskStatus::Identified,
            tags: Vec::new(),
            regulatory_references: vec!["ISO_14971".to_string()],
            standard_references: vec!["ISO_14971:2019".to_string()],
            created_at: crate::utils::current_timestamp_string(),
            updated_at: crate::utils::current_timestamp_string(),
            created_by: "mock-user".to_string(),
            approved_by: None,
            approval_date: None,
            post_market_data: Vec::new(),
            review_required: false,
            next_review_date: None,
        };
        
        self.risks.insert(id.clone(), risk.clone());
        Ok(risk)
    }
    
    fn assess_risk(
        &mut self,
        risk_id: &str,
        severity: Option<RiskSeverity>,
        occurrence: Option<RiskOccurrence>,
        detectability: Option<RiskDetectability>,
    ) -> QmsResult<RiskItem> {
        match self.risks.get_mut(risk_id) {
            Some(risk) => {
                if let Some(sev) = severity {
                    risk.severity = sev;
                }
                if let Some(occ) = occurrence {
                    risk.occurrence = occ;
                }
                if let Some(det) = detectability {
                    risk.detectability = det;
                }
                
                // Recalculate RPN (simplified)
                risk.risk_priority_number = (risk.severity.clone() as u32) * 
                                           (risk.occurrence.clone() as u32) * 
                                           (risk.detectability.clone() as u32);
                
                Ok(risk.clone())
            }
            None => Err(QmsError::not_found(&format!("Risk not found: {}", risk_id))),
        }
    }
    
    fn get_risk(&self, risk_id: &str) -> QmsResult<RiskItem> {
        self.risks.get(risk_id)
            .cloned()
            .ok_or_else(|| QmsError::not_found(&format!("Risk not found: {}", risk_id)))
    }
    
    fn list_risks(&self, _filter: Option<RiskFilter>) -> QmsResult<Vec<RiskIndexEntry>> {
        let entries = self.risks.values().map(|risk| RiskIndexEntry {
            id: risk.id.clone(),
            hazard_id: risk.hazard_id.clone(),
            description: risk.hazard_description.clone(),
            severity: risk.severity.clone(),
            rpn: risk.risk_priority_number,
            risk_level: risk.initial_risk_level.clone(),
            status: risk.verification_status.clone(),
            created_at: risk.created_at.clone(),
            updated_at: risk.updated_at.clone(),
        }).collect();
        
        Ok(entries)
    }
    
    fn get_risk_statistics(&self) -> QmsResult<RiskStatistics> {
        let risks = self.list_risks(None)?;
        let mut stats = RiskStatistics::default();
        stats.total_risks = risks.len();
        stats.high_priority_risks = risks.iter().filter(|r| r.rpn >= 50).count();
        Ok(stats)
    }
    
    fn health_check(&self) -> QmsResult<ServiceHealth> {
        Ok(ServiceHealth::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_helpers::TestHelper;
    
    #[test]
    fn test_file_risk_service_creation() {
        let temp_dir = TestHelper::create_temp_dir();
        let service = FileRiskService::new(&temp_dir);
        assert!(service.is_ok());
    }
    
    #[test]
    fn test_mock_risk_service() {
        let mut service = MockRiskService::new();
        
        // Test risk creation
        let risk = service.create_risk("Test risk", "Test situation", "Test harm");
        assert!(risk.is_ok());
        
        let created_risk = risk.unwrap();
        assert_eq!(created_risk.hazard_description, "Test risk");
        
        // Test risk retrieval
        let retrieved = service.get_risk(&created_risk.id);
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().id, created_risk.id);
        
        // Test statistics
        let stats = service.get_risk_statistics();
        assert!(stats.is_ok());
        assert_eq!(stats.unwrap().total_risks, 1);
    }
    
    #[test]
    fn test_risk_statistics() {
        let stats = RiskStatistics::default();
        assert_eq!(stats.total_risks, 0);
        assert_eq!(stats.high_priority_risks, 0);
    }
    
    #[test]
    fn test_service_health() {
        let health = ServiceHealth::default();
        assert!(health.is_healthy);
        assert_eq!(health.message, "Service operational");
    }
}
