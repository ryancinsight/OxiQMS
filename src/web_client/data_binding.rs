// Data Binding Module - Dynamic UI Updates for WASM Client
// Medical Device QMS - FDA 21 CFR Part 820, ISO 13485, ISO 14971 Compliant
// Handles binding API data to DOM elements and updating UI state

use crate::prelude::*;
use crate::web_client::api_client::*;
use std::collections::HashMap;

/// Data binding configuration for automatic UI updates
#[derive(Debug, Clone)]
pub struct DataBinding {
    pub element_id: String,
    pub data_source: String,
    pub bind_type: BindingType,
    pub format_template: Option<String>,
    pub update_interval: Option<u32>, // seconds
    pub last_updated: u64,
}

/// Types of data binding supported
#[derive(Debug, Clone)]
pub enum BindingType {
    Text,           // Bind to element text content
    Html,           // Bind to element innerHTML
    Attribute(String), // Bind to specific attribute
    Class,          // Bind to CSS classes
    Style(String),  // Bind to specific CSS style
    Table,          // Bind to table data
    List,           // Bind to list elements
}

/// Data Binder for managing UI updates from API data
pub struct DataBinder {
    bindings: HashMap<String, DataBinding>,
    cached_data: HashMap<String, String>,
    update_queue: Vec<String>,
    auto_refresh_enabled: bool,
}

impl DataBinder {
    /// Create new data binder instance
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            cached_data: HashMap::new(),
            update_queue: Vec::new(),
            auto_refresh_enabled: true,
        }
    }

    /// Update system statistics on dashboard
    pub fn update_stats(&mut self, stats: &SystemStats) -> QmsResult<()> {
        // Update document count
        self.queue_text_update("doc-count", &stats.document_count.to_string())?;
        
        // Update risk count
        self.queue_text_update("risk-count", &stats.risk_count.to_string())?;
        
        // Update requirement count
        self.queue_text_update("req-count", &stats.requirement_count.to_string())?;
        
        // Update audit count
        self.queue_text_update("audit-count", &stats.audit_count.to_string())?;
        
        // Update system status
        self.queue_text_update("system-status", &format!("🟢 {}", stats.system_status))?;
        
        // Update compliance score
        self.queue_text_update("compliance-score", &format!("{:.1}%", stats.compliance_score))?;
        
        // Update last audit timestamp
        self.queue_text_update("last-audit", &self.format_timestamp(&stats.last_audit))?;
        
        Ok(())
    }

    /// Update activity feed on dashboard
    pub fn update_activity_feed(&mut self, activities: &[ActivityEntry]) -> QmsResult<()> {
        let mut activity_html = String::new();
        
        if activities.is_empty() {
            activity_html = r#"<div class="activity-empty">
                <p>No recent activity</p>
            </div>"#.to_string();
        } else {
            activity_html.push_str(r#"<div class="activity-list">"#);
            
            for activity in activities.iter().take(10) {
                let activity_item = format!(
                    r#"<div class="activity-item">
                        <div class="activity-time">{}</div>
                        <div class="activity-user">{}</div>
                        <div class="activity-action">{}</div>
                        <div class="activity-target">{} {}</div>
                        <div class="activity-description">{}</div>
                    </div>"#,
                    self.format_timestamp(&activity.timestamp),
                    activity.user,
                    self.format_action(&activity.action),
                    activity.entity_type,
                    activity.entity_id,
                    activity.description
                );
                activity_html.push_str(&activity_item);
            }
            
            activity_html.push_str("</div>");
        }
        
        self.queue_html_update("activity-feed", &activity_html)?;
        Ok(())
    }

    /// Update compliance badges display
    pub fn update_compliance_badges(&mut self, badges: &[ComplianceBadge]) -> QmsResult<()> {
        let mut badges_html = String::new();
        
        for badge in badges {
            let status_class = match badge.score {
                score if score >= 95.0 => "badge-excellent",
                score if score >= 90.0 => "badge-good", 
                score if score >= 80.0 => "badge-acceptable",
                _ => "badge-needs-improvement",
            };
            
            let badge_html = format!(
                r#"<div class="compliance-badge {status_class}">
                    <div class="badge-title">{}</div>
                    <div class="badge-score">{:.1}%</div>
                    <div class="badge-status">{}</div>
                    <div class="badge-updated">Updated: {}</div>
                </div>"#,
                badge.standard,
                badge.score,
                badge.status,
                badge.last_updated
            );
            badges_html.push_str(&badge_html);
        }
        
        self.queue_html_update("compliance-badges", &badges_html)?;
        Ok(())
    }

    /// Update documents view
    pub fn update_documents_view(&mut self, documents: &[DocumentSummary]) -> QmsResult<()> {
        let mut table_html = String::new();
        
        // Table header
        table_html.push_str(r#"
            <thead>
                <tr>
                    <th>ID</th>
                    <th>Title</th>
                    <th>Type</th>
                    <th>Status</th>
                    <th>Version</th>
                    <th>Updated</th>
                    <th>Actions</th>
                </tr>
            </thead>
        "#);
        
        // Table body
        table_html.push_str("<tbody>");
        for doc in documents {
            let status_class = match doc.status.as_str() {
                "Approved" => "status-approved",
                "Draft" => "status-draft",
                "InReview" => "status-review",
                _ => "status-unknown",
            };
            
            let row_html = format!(
                r#"<tr>
                    <td><code>{}</code></td>
                    <td>{}</td>
                    <td><span class="doc-type">{}</span></td>
                    <td><span class="status {status_class}">{}</span></td>
                    <td><span class="version">{}</span></td>
                    <td>{}</td>
                    <td>
                        <button class="btn btn-sm" onclick="viewDocument('{}')">View</button>
                        <button class="btn btn-sm" onclick="editDocument('{}')">Edit</button>
                    </td>
                </tr>"#,
                doc.id, doc.title, doc.doc_type, doc.status, doc.version,
                self.format_date(&doc.updated_at), doc.id, doc.id
            );
            table_html.push_str(&row_html);
        }
        table_html.push_str("</tbody>");
        
        self.queue_html_update("documents-table", &table_html)?;
        Ok(())
    }

    /// Update risks view
    pub fn update_risks_view(&mut self, risks: &[RiskSummary]) -> QmsResult<()> {
        let mut table_html = String::new();
        
        // Table header
        table_html.push_str(r#"
            <thead>
                <tr>
                    <th>ID</th>
                    <th>Hazard Description</th>
                    <th>Severity</th>
                    <th>Occurrence</th>
                    <th>Detectability</th>
                    <th>RPN</th>
                    <th>Risk Level</th>
                    <th>Status</th>
                    <th>Actions</th>
                </tr>
            </thead>
        "#);
        
        // Table body
        table_html.push_str("<tbody>");
        for risk in risks {
            let risk_level_class = match risk.risk_level.as_str() {
                "Unacceptable" => "risk-unacceptable",
                "ALARP" => "risk-alarp",
                "Acceptable" => "risk-acceptable",
                _ => "risk-unknown",
            };
            
            let rpn_class = match risk.rpn {
                rpn if rpn >= 100 => "rpn-high",
                rpn if rpn >= 50 => "rpn-medium",
                _ => "rpn-low",
            };
            
            let row_html = format!(
                r#"<tr>
                    <td><code>{}</code></td>
                    <td>{}</td>
                    <td><span class="severity">{}</span></td>
                    <td><span class="occurrence">{}</span></td>
                    <td><span class="detectability">{}</span></td>
                    <td><span class="rpn {rpn_class}">{}</span></td>
                    <td><span class="risk-level {risk_level_class}">{}</span></td>
                    <td><span class="status">{}</span></td>
                    <td>
                        <button class="btn btn-sm" onclick="viewRisk('{}')">View</button>
                        <button class="btn btn-sm" onclick="assessRisk('{}')">Assess</button>
                    </td>
                </tr>"#,
                risk.id, risk.hazard_description, risk.severity, risk.occurrence,
                risk.detectability, risk.rpn, risk.risk_level, risk.status, risk.id, risk.id
            );
            table_html.push_str(&row_html);
        }
        table_html.push_str("</tbody>");
        
        self.queue_html_update("risks-table", &table_html)?;
        Ok(())
    }

    /// Update requirements view
    pub fn update_requirements_view(&mut self, requirements: &[RequirementSummary]) -> QmsResult<()> {
        let mut table_html = String::new();
        
        // Table header
        table_html.push_str(r#"
            <thead>
                <tr>
                    <th>ID</th>
                    <th>Title</th>
                    <th>Category</th>
                    <th>Priority</th>
                    <th>Status</th>
                    <th>Verification</th>
                    <th>Actions</th>
                </tr>
            </thead>
        "#);
        
        // Table body
        table_html.push_str("<tbody>");
        for req in requirements {
            let priority_class = match req.priority.as_str() {
                "Critical" => "priority-critical",
                "High" => "priority-high",
                "Medium" => "priority-medium",
                "Low" => "priority-low",
                _ => "priority-unknown",
            };
            
            let status_class = match req.status.as_str() {
                "Approved" => "status-approved",
                "Draft" => "status-draft",
                "Implemented" => "status-implemented",
                "Verified" => "status-verified",
                _ => "status-unknown",
            };
            
            let row_html = format!(
                r#"<tr>
                    <td><code>{}</code></td>
                    <td>{}</td>
                    <td><span class="category">{}</span></td>
                    <td><span class="priority {priority_class}">{}</span></td>
                    <td><span class="status {status_class}">{}</span></td>
                    <td><span class="verification">{}</span></td>
                    <td>
                        <button class="btn btn-sm" onclick="viewRequirement('{}')">View</button>
                        <button class="btn btn-sm" onclick="editRequirement('{}')">Edit</button>
                    </td>
                </tr>"#,
                req.id, req.title, req.category, req.priority, req.status,
                req.verification_method, req.id, req.id
            );
            table_html.push_str(&row_html);
        }
        table_html.push_str("</tbody>");
        
        self.queue_html_update("requirements-table", &table_html)?;
        Ok(())
    }

    /// Update audit trail view
    pub fn update_audit_view(&mut self, audit_entries: &[AuditEntrySummary]) -> QmsResult<()> {
        let mut table_html = String::new();
        
        // Table header
        table_html.push_str(r#"
            <thead>
                <tr>
                    <th>Timestamp</th>
                    <th>User</th>
                    <th>Action</th>
                    <th>Entity Type</th>
                    <th>Entity ID</th>
                    <th>Details</th>
                </tr>
            </thead>
        "#);
        
        // Table body
        table_html.push_str("<tbody>");
        for entry in audit_entries {
            let action_class = match entry.action.as_str() {
                "CREATE" => "action-create",
                "UPDATE" => "action-update",
                "DELETE" => "action-delete",
                "APPROVE" => "action-approve",
                _ => "action-other",
            };
            
            let row_html = format!(
                r#"<tr>
                    <td>{}</td>
                    <td><code>{}</code></td>
                    <td><span class="action {action_class}">{}</span></td>
                    <td>{}</td>
                    <td><code>{}</code></td>
                    <td>{}</td>
                </tr>"#,
                self.format_timestamp(&entry.timestamp),
                entry.user_id,
                entry.action,
                entry.entity_type,
                entry.entity_id,
                entry.details.as_deref().unwrap_or("—")
            );
            table_html.push_str(&row_html);
        }
        table_html.push_str("</tbody>");
        
        self.queue_html_update("audit-table", &table_html)?;
        Ok(())
    }

    /// Show success message to user
    pub fn show_success_message(&mut self, message: &str) -> QmsResult<()> {
        let alert_html = format!(
            r#"<div class="alert alert-success" role="alert">
                <span class="alert-icon">✅</span>
                <span class="alert-message">{message}</span>
                <button class="alert-close" onclick="this.parentElement.remove()">×</button>
            </div>"#
        );
        
        self.queue_html_update("notifications", &alert_html)?;
        Ok(())
    }

    /// Show error message to user
    pub fn show_error_message(&mut self, message: &str) -> QmsResult<()> {
        let alert_html = format!(
            r#"<div class="alert alert-error" role="alert">
                <span class="alert-icon">❌</span>
                <span class="alert-message">{message}</span>
                <button class="alert-close" onclick="this.parentElement.remove()">×</button>
            </div>"#
        );
        
        self.queue_html_update("notifications", &alert_html)?;
        Ok(())
    }

    /// Queue text content update
    fn queue_text_update(&mut self, element_id: &str, text: &str) -> QmsResult<()> {
        self.cached_data.insert(format!("{element_id}_text"), text.to_string());
        self.update_queue.push(format!("text:{element_id}:{text}"));
        Ok(())
    }

    /// Queue HTML content update
    fn queue_html_update(&mut self, element_id: &str, html: &str) -> QmsResult<()> {
        self.cached_data.insert(format!("{element_id}_html"), html.to_string());
        self.update_queue.push(format!("html:{element_id}:{html}"));
        Ok(())
    }

    /// Process all queued updates
    pub fn flush_updates(&mut self) -> QmsResult<()> {
        for update in &self.update_queue {
            self.process_update(update)?;
        }
        
        self.update_queue.clear();
        Ok(())
    }

    /// Process a single update command
    fn process_update(&self, update: &str) -> QmsResult<()> {
        let parts: Vec<&str> = update.splitn(3, ':').collect();
        if parts.len() != 3 {
            return Err(QmsError::parse_error("Invalid update format"));
        }

        let update_type = parts[0];
        let element_id = parts[1];
        let content = parts[2];

        match update_type {
            "text" => {
                // In WASM: document.getElementById(element_id).textContent = content
                eprintln!("TEXT UPDATE: {} = '{}'", element_id, content);
            },
            "html" => {
                // In WASM: document.getElementById(element_id).innerHTML = content
                eprintln!("HTML UPDATE: {} = '{}'", element_id, 
                    if content.len() > 100 { &format!("{}...", &content[..100]) } else { content });
            },
            _ => {
                return Err(QmsError::parse_error(&format!("Unknown update type: {update_type}")));
            }
        }

        Ok(())
    }

    /// Format timestamp for display
    fn format_timestamp(&self, timestamp: &str) -> String {
        // Simple timestamp formatting - in production would use proper date formatting
        if timestamp.contains('T') {
            // ISO format: 2025-01-19T10:30:00Z
            timestamp.replace('T', " ").replace('Z', " UTC")
        } else {
            timestamp.to_string()
        }
    }

    /// Format date for display  
    fn format_date(&self, date: &str) -> String {
        // Simple date formatting
        date.to_string()
    }

    /// Format action name for display
    fn format_action(&self, action: &str) -> String {
        match action {
            "DOCUMENT_CREATED" => "📄 Created Document".to_string(),
            "DOCUMENT_APPROVED" => "✅ Approved Document".to_string(),
            "RISK_CREATED" => "⚠️ Created Risk".to_string(),
            "RISK_ASSESSED" => "📊 Assessed Risk".to_string(),
            "REQUIREMENT_CREATED" => "📋 Created Requirement".to_string(),
            "AUDIT_ENTRY" => "🔍 Audit Entry".to_string(),
            _ => action.replace('_', " ").to_string(),
        }
    }

    /// Get cached data for element
    pub fn get_cached_data(&self, key: &str) -> Option<&String> {
        self.cached_data.get(key)
    }

    /// Clear cached data
    pub fn clear_cache(&mut self) {
        self.cached_data.clear();
    }

    /// Get update queue size
    pub fn update_queue_size(&self) -> usize {
        self.update_queue.len()
    }

    /// Enable/disable auto refresh
    pub fn set_auto_refresh(&mut self, enabled: bool) {
        self.auto_refresh_enabled = enabled;
    }
}

impl Default for DataBinder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_binder_creation() {
        let binder = DataBinder::new();
        assert!(binder.auto_refresh_enabled);
        assert_eq!(binder.update_queue_size(), 0);
    }

    #[test]
    fn test_system_stats_update() {
        let mut binder = DataBinder::new();
        let stats = SystemStats {
            document_count: 15,
            risk_count: 8,
            requirement_count: 23,
            audit_count: 156,
            system_status: "Operational".to_string(),
            compliance_score: 92.5,
            last_audit: "2025-01-19T09:15:00Z".to_string(),
        };

        assert!(binder.update_stats(&stats).is_ok());
        assert!(binder.update_queue_size() > 0);
    }

    #[test]
    fn test_activity_feed_update() {
        let mut binder = DataBinder::new();
        let activities = vec![
            ActivityEntry {
                timestamp: "2025-01-19T10:30:00Z".to_string(),
                user: "system".to_string(),
                action: "DOCUMENT_CREATED".to_string(),
                entity_type: "Document".to_string(),
                entity_id: "DOC-001".to_string(),
                description: "New document created".to_string(),
            }
        ];

        assert!(binder.update_activity_feed(&activities).is_ok());
        assert!(binder.update_queue_size() > 0);
    }

    #[test]
    fn test_compliance_badges_update() {
        let mut binder = DataBinder::new();
        let badges = vec![
            ComplianceBadge {
                standard: "FDA 21 CFR Part 820".to_string(),
                status: "Compliant".to_string(),
                score: 95.0,
                last_updated: "2025-01-19".to_string(),
            }
        ];

        assert!(binder.update_compliance_badges(&badges).is_ok());
        assert!(binder.update_queue_size() > 0);
    }

    #[test]
    fn test_documents_view_update() {
        let mut binder = DataBinder::new();
        let documents = vec![
            DocumentSummary {
                id: "DOC-001".to_string(),
                title: "Requirements".to_string(),
                doc_type: "SRS".to_string(),
                status: "Approved".to_string(),
                version: "1.0.0".to_string(),
                created_at: "2025-01-15".to_string(),
                updated_at: "2025-01-19".to_string(),
            }
        ];

        assert!(binder.update_documents_view(&documents).is_ok());
        assert!(binder.update_queue_size() > 0);
    }

    #[test]
    fn test_message_display() {
        let mut binder = DataBinder::new();
        
        assert!(binder.show_success_message("Operation successful").is_ok());
        assert!(binder.show_error_message("Operation failed").is_ok());
        assert!(binder.update_queue_size() > 0);
    }

    #[test]
    fn test_timestamp_formatting() {
        let binder = DataBinder::new();
        let formatted = binder.format_timestamp("2025-01-19T10:30:00Z");
        assert!(formatted.contains("2025-01-19 10:30:00 UTC"));
    }

    #[test]
    fn test_action_formatting() {
        let binder = DataBinder::new();
        
        assert_eq!(binder.format_action("DOCUMENT_CREATED"), "📄 Created Document");
        assert_eq!(binder.format_action("RISK_ASSESSED"), "📊 Assessed Risk");
        assert_eq!(binder.format_action("CUSTOM_ACTION"), "CUSTOM ACTION");
    }

    #[test]
    fn test_cache_operations() {
        let mut binder = DataBinder::new();
        
        binder.queue_text_update("test-element", "Test Content").unwrap();
        assert!(binder.get_cached_data("test-element_text").is_some());
        
        binder.clear_cache();
        assert!(binder.get_cached_data("test-element_text").is_none());
    }

    #[test]
    fn test_update_flushing() {
        let mut binder = DataBinder::new();
        
        binder.queue_text_update("element1", "Text 1").unwrap();
        binder.queue_html_update("element2", "<p>HTML</p>").unwrap();
        
        assert_eq!(binder.update_queue_size(), 2);
        
        assert!(binder.flush_updates().is_ok());
        assert_eq!(binder.update_queue_size(), 0);
    }
}
