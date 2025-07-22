//! Observer Pattern Implementation for Audit Events
//! 
//! This module implements the Observer pattern for audit event notifications,
//! following SOLID principles for medical device quality management.
//! 
//! SOLID Principles Applied:
//! - Single Responsibility: Each observer handles one specific type of audit event processing
//! - Open/Closed: New observers can be added without modifying existing code
//! - Liskov Substitution: All observers can be used interchangeably
//! - Interface Segregation: Focused interfaces for different types of audit notifications
//! - Dependency Inversion: High-level audit system depends on observer abstractions

use crate::prelude::*;
use crate::models::AuditEntry;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Observer pattern interface for audit event notifications
/// Interface Segregation Principle: Focused interface for audit event handling
pub trait AuditEventObserver: Send + Sync {
    /// Handle audit event notification
    fn on_audit_event(&self, event: &AuditEvent) -> QmsResult<()>;
    
    /// Get observer name for identification and debugging
    fn observer_name(&self) -> &'static str;
    
    /// Check if observer is interested in this event type
    fn is_interested_in(&self, event_type: &AuditEventType) -> bool;
    
    /// Get observer priority (higher numbers = higher priority)
    fn priority(&self) -> u8 {
        50 // Default priority
    }
}

/// Types of audit events that can be observed
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AuditEventType {
    EntryCreated,
    EntryUpdated,
    SecurityAlert,
    ComplianceViolation,
    SystemEvent,
    UserActivity,
    DataIntegrityIssue,
    PerformanceAlert,
}

/// Audit event data structure
#[derive(Debug, Clone)]
pub struct AuditEvent {
    pub event_type: AuditEventType,
    pub entry: AuditEntry,
    pub metadata: HashMap<String, String>,
    pub timestamp: u64,
    pub severity: EventSeverity,
}

/// Event severity levels for prioritization
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl AuditEvent {
    /// Create new audit event
    pub fn new(event_type: AuditEventType, entry: AuditEntry) -> Self {
        Self {
            event_type,
            entry,
            metadata: HashMap::new(),
            timestamp: crate::utils::current_timestamp(),
            severity: EventSeverity::Medium,
        }
    }
    
    /// Add metadata to the event
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Set event severity
    pub const fn with_severity(mut self, severity: EventSeverity) -> Self {
        self.severity = severity;
        self
    }
}

/// Observable audit event subject
/// Single Responsibility Principle: Manages observer registration and notification
pub struct AuditEventSubject {
    observers: Arc<Mutex<Vec<Arc<dyn AuditEventObserver>>>>,
    event_filters: Arc<Mutex<HashMap<String, Vec<AuditEventType>>>>,
}

impl AuditEventSubject {
    /// Create new audit event subject
    pub fn new() -> Self {
        Self {
            observers: Arc::new(Mutex::new(Vec::new())),
            event_filters: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Register an observer
    /// Open/Closed Principle: Can add new observers without modifying existing code
    pub fn register_observer(&self, observer: Arc<dyn AuditEventObserver>) -> QmsResult<()> {
        let mut observers = self.observers.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire observers lock"))?;
        
        // Check if observer already registered
        let observer_name = observer.observer_name();
        if observers.iter().any(|o| o.observer_name() == observer_name) {
            return Err(QmsError::validation_error(&format!(
                "Observer '{observer_name}' is already registered"
            )));
        }
        
        observers.push(observer);
        
        // Sort by priority (highest first)
        observers.sort_by_key(|b| std::cmp::Reverse(b.priority()));
        
        Ok(())
    }
    
    /// Unregister an observer
    pub fn unregister_observer(&self, observer_name: &str) -> QmsResult<bool> {
        let mut observers = self.observers.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire observers lock"))?;
        
        let initial_len = observers.len();
        observers.retain(|o| o.observer_name() != observer_name);
        
        Ok(observers.len() < initial_len)
    }
    
    /// Notify all interested observers about an audit event
    /// Dependency Inversion Principle: Depends on observer abstractions
    pub fn notify_observers(&self, event: &AuditEvent) -> QmsResult<()> {
        let observers = self.observers.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire observers lock"))?;
        
        let mut errors = Vec::new();
        
        for observer in observers.iter() {
            if observer.is_interested_in(&event.event_type) {
                if let Err(e) = observer.on_audit_event(event) {
                    errors.push(format!("Observer '{}' failed: {}", observer.observer_name(), e));
                }
            }
        }
        
        if !errors.is_empty() {
            return Err(QmsError::domain_error(&format!(
                "Observer notification errors: {}", errors.join("; ")
            )));
        }
        
        Ok(())
    }
    
    /// Get list of registered observers
    pub fn list_observers(&self) -> QmsResult<Vec<String>> {
        let observers = self.observers.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire observers lock"))?;
        
        Ok(observers.iter().map(|o| o.observer_name().to_string()).collect())
    }
    
    /// Get observer count
    pub fn observer_count(&self) -> QmsResult<usize> {
        let observers = self.observers.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire observers lock"))?;
        
        Ok(observers.len())
    }
}

impl Default for AuditEventSubject {
    fn default() -> Self {
        Self::new()
    }
}

/// Security Alert Observer - monitors for security-related audit events
/// Single Responsibility Principle: Handles only security alert processing
pub struct SecurityAlertObserver {
    alert_threshold: u32,
    failed_attempts: Arc<Mutex<HashMap<String, u32>>>,
}

impl SecurityAlertObserver {
    pub fn new(alert_threshold: u32) -> Self {
        Self {
            alert_threshold,
            failed_attempts: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl AuditEventObserver for SecurityAlertObserver {
    fn on_audit_event(&self, event: &AuditEvent) -> QmsResult<()> {
        if event.event_type == AuditEventType::SecurityAlert {
            // Log security alert
            eprintln!("SECURITY ALERT: {:?} - {}",
                event.entry.action,
                event.entry.details.as_deref().unwrap_or("No details"));

            // Track failed attempts
            if event.entry.details.as_deref().unwrap_or("").contains("failed") {
                let mut attempts = self.failed_attempts.lock()
                    .map_err(|_| QmsError::domain_error("Failed to acquire failed attempts lock"))?;

                let count = attempts.entry(event.entry.user_id.clone()).or_insert(0);
                *count += 1;

                if *count >= self.alert_threshold {
                    eprintln!("CRITICAL: User {} has {} failed attempts",
                        event.entry.user_id, count);
                }
            }
        }
        
        Ok(())
    }
    
    fn observer_name(&self) -> &'static str {
        "SecurityAlertObserver"
    }
    
    fn is_interested_in(&self, event_type: &AuditEventType) -> bool {
        matches!(event_type, AuditEventType::SecurityAlert | AuditEventType::UserActivity)
    }
    
    fn priority(&self) -> u8 {
        90 // High priority for security events
    }
}

/// Compliance Monitoring Observer - monitors for regulatory compliance issues
/// Single Responsibility Principle: Handles only compliance monitoring
pub struct ComplianceMonitorObserver {
    compliance_rules: Vec<ComplianceRule>,
}

#[derive(Debug, Clone)]
pub struct ComplianceRule {
    pub name: String,
    pub description: String,
    pub applies_to: Vec<String>, // Entity types this rule applies to
    pub required_fields: Vec<String>,
}

impl ComplianceMonitorObserver {
    pub fn new() -> Self {
        let rules = vec![
            // FDA 21 CFR Part 11 compliance rules
            ComplianceRule {
                name: "21CFR11_ElectronicSignature".to_string(),
                description: "Electronic signatures required for critical operations".to_string(),
                applies_to: vec!["Document".to_string(), "Risk".to_string()],
                required_fields: vec!["signature".to_string()],
            },
            // ISO 13485 compliance rules
            ComplianceRule {
                name: "ISO13485_Traceability".to_string(),
                description: "Full traceability required for medical device records".to_string(),
                applies_to: vec!["Document".to_string(), "Requirement".to_string()],
                required_fields: vec!["user_id".to_string(), "timestamp".to_string()],
            }
        ];
        
        Self { compliance_rules: rules }
    }
    
    fn check_compliance(&self, event: &AuditEvent) -> QmsResult<Vec<String>> {
        let mut violations = Vec::new();
        
        for rule in &self.compliance_rules {
            if rule.applies_to.contains(&event.entry.entity_type) {
                // Check required fields
                for field in &rule.required_fields {
                    match field.as_str() {
                        "signature" => {
                            if event.entry.signature.is_none() {
                                violations.push(format!("Rule '{}': Missing electronic signature", rule.name));
                            }
                        }
                        "user_id" => {
                            if event.entry.user_id.is_empty() {
                                violations.push(format!("Rule '{}': Missing user ID", rule.name));
                            }
                        }
                        "timestamp" => {
                            if event.entry.timestamp.is_empty() {
                                violations.push(format!("Rule '{}': Missing timestamp", rule.name));
                            }
                        }
                        _ => {} // Unknown field
                    }
                }
            }
        }
        
        Ok(violations)
    }
}

impl Default for ComplianceMonitorObserver {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditEventObserver for ComplianceMonitorObserver {
    fn on_audit_event(&self, event: &AuditEvent) -> QmsResult<()> {
        let violations = self.check_compliance(event)?;
        
        if !violations.is_empty() {
            eprintln!("COMPLIANCE VIOLATIONS detected:");
            for violation in violations {
                eprintln!("  - {violation}");
            }
            
            // In a production system, this would trigger alerts, reports, etc.
        }
        
        Ok(())
    }
    
    fn observer_name(&self) -> &'static str {
        "ComplianceMonitorObserver"
    }
    
    fn is_interested_in(&self, event_type: &AuditEventType) -> bool {
        matches!(event_type, 
            AuditEventType::EntryCreated | 
            AuditEventType::EntryUpdated | 
            AuditEventType::ComplianceViolation
        )
    }
    
    fn priority(&self) -> u8 {
        80 // High priority for compliance
    }
}

/// Performance Monitoring Observer - tracks audit system performance
/// Single Responsibility Principle: Handles only performance monitoring
pub struct PerformanceMonitorObserver {
    event_counts: Arc<Mutex<HashMap<AuditEventType, u64>>>,
    start_time: u64,
}

impl PerformanceMonitorObserver {
    pub fn new() -> Self {
        Self {
            event_counts: Arc::new(Mutex::new(HashMap::new())),
            start_time: crate::utils::current_timestamp(),
        }
    }
    
    pub fn get_statistics(&self) -> QmsResult<HashMap<AuditEventType, u64>> {
        let counts = self.event_counts.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire event counts lock"))?;
        
        Ok(counts.clone())
    }
}

impl Default for PerformanceMonitorObserver {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditEventObserver for PerformanceMonitorObserver {
    fn on_audit_event(&self, event: &AuditEvent) -> QmsResult<()> {
        let mut counts = self.event_counts.lock()
            .map_err(|_| QmsError::domain_error("Failed to acquire event counts lock"))?;
        
        *counts.entry(event.event_type.clone()).or_insert(0) += 1;
        
        // Check for performance alerts
        let total_events: u64 = counts.values().sum();
        let elapsed_time = crate::utils::current_timestamp() - self.start_time;
        
        if elapsed_time > 0 {
            let events_per_second = total_events as f64 / elapsed_time as f64;
            
            // Alert if processing rate is too high (potential DoS or system issue)
            if events_per_second > 100.0 {
                eprintln!("PERFORMANCE ALERT: High audit event rate: {events_per_second:.2} events/second");
            }
        }
        
        Ok(())
    }
    
    fn observer_name(&self) -> &'static str {
        "PerformanceMonitorObserver"
    }
    
    fn is_interested_in(&self, _event_type: &AuditEventType) -> bool {
        true // Interested in all events for performance monitoring
    }
    
    fn priority(&self) -> u8 {
        10 // Low priority - performance monitoring doesn't need to be first
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AuditAction;

    #[test]
    fn test_audit_event_creation() {
        let entry = AuditEntry {
            id: "test-id".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            user_id: "test-user".to_string(),
            session_id: None,
            action: AuditAction::Create,
            entity_type: "Document".to_string(),
            entity_id: "DOC-001".to_string(),
            old_value: None,
            new_value: None,
            details: None,
            ip_address: None,
            signature: None,
            checksum: "test-checksum".to_string(),
            previous_hash: None,
        };
        
        let event = AuditEvent::new(AuditEventType::EntryCreated, entry)
            .with_metadata("source".to_string(), "test".to_string())
            .with_severity(EventSeverity::High);
        
        assert_eq!(event.event_type, AuditEventType::EntryCreated);
        assert_eq!(event.severity, EventSeverity::High);
        assert_eq!(event.metadata.get("source"), Some(&"test".to_string()));
    }

    #[test]
    fn test_observer_registration() {
        let subject = AuditEventSubject::new();
        let observer = Arc::new(SecurityAlertObserver::new(3));
        
        assert!(subject.register_observer(observer.clone()).is_ok());
        assert_eq!(subject.observer_count().unwrap(), 1);
        
        // Test duplicate registration
        assert!(subject.register_observer(observer).is_err());
        assert_eq!(subject.observer_count().unwrap(), 1);
    }

    #[test]
    fn test_observer_notification() {
        let subject = AuditEventSubject::new();
        let observer = Arc::new(PerformanceMonitorObserver::new());
        
        subject.register_observer(observer.clone()).unwrap();
        
        let entry = AuditEntry {
            id: "test-id".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            user_id: "test-user".to_string(),
            session_id: None,
            action: AuditAction::Create,
            entity_type: "Document".to_string(),
            entity_id: "DOC-001".to_string(),
            old_value: None,
            new_value: None,
            details: None,
            ip_address: None,
            signature: None,
            checksum: "test-checksum".to_string(),
            previous_hash: None,
        };
        
        let event = AuditEvent::new(AuditEventType::EntryCreated, entry);
        
        assert!(subject.notify_observers(&event).is_ok());
        
        let stats = observer.get_statistics().unwrap();
        assert_eq!(stats.get(&AuditEventType::EntryCreated), Some(&1));
    }
}
