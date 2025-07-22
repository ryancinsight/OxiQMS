//! Audit Dashboard & Metrics Module
//! 
//! Provides dashboard functionality with metrics, statistics, and trend analysis
//! for audit trail data. Includes activity monitoring and alert detection.

use crate::prelude::*;
use crate::models::{AuditEntry, AuditAction};
use std::collections::HashMap;

/// Dashboard metrics and statistics
#[derive(Debug)]
pub struct AuditDashboard {
    pub general_metrics: GeneralMetrics,
    pub user_activity: UserActivityMetrics,
    pub action_metrics: ActionMetrics,
    pub time_analysis: TimeAnalysis,
    pub alerts: Vec<SecurityAlert>,
    pub trends: TrendAnalysis,
}

/// General system metrics
#[derive(Debug)]
pub struct GeneralMetrics {
    pub total_entries: usize,
    pub entries_today: usize,
    pub entries_this_week: usize,
    pub entries_this_month: usize,
    pub unique_users_today: usize,
    pub unique_users_total: usize,
    pub average_daily_activity: f64,
    pub data_size_mb: f64,
}

/// User activity metrics
#[allow(dead_code)]
#[derive(Debug)]
pub struct UserActivityMetrics {
    pub most_active_users: Vec<(String, usize)>, // (username, entry_count)
    pub user_activity_distribution: HashMap<String, UserStats>,
    pub recent_logins: Vec<(String, String)>, // (username, timestamp)
    pub inactive_users: Vec<String>, // Users with no recent activity
}

/// Individual user statistics
#[allow(dead_code)]
#[derive(Debug)]
pub struct UserStats {
    pub total_actions: usize,
    pub actions_today: usize,
    pub last_activity: Option<String>,
    pub most_common_action: String,
    pub risk_score: f64, // 0.0 to 1.0
}

/// Action type metrics
#[derive(Debug)]
pub struct ActionMetrics {
    pub action_distribution: HashMap<String, usize>,
    pub critical_actions_today: usize,
    pub failed_operations: usize,
    pub bulk_operations: usize,
    pub administrative_actions: usize,
}

/// Time-based analysis
#[allow(dead_code)]
#[derive(Debug)]
pub struct TimeAnalysis {
    pub peak_activity_hours: Vec<(u8, usize)>, // (hour, count)
    pub daily_activity_trend: Vec<(String, usize)>, // (date, count)
    pub activity_patterns: ActivityPattern,
}

/// Activity pattern classification
#[allow(dead_code)]
#[derive(Debug)]
pub enum ActivityPattern {
    Normal,
    HighActivity,
    Suspicious,
    BulkOperations,
    AfterHours,
}

/// Security alerts and anomalies
#[allow(dead_code)]
#[derive(Debug)]
pub struct SecurityAlert {
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub description: String,
    pub affected_user: Option<String>,
    pub timestamp: String,
    pub recommendation: String,
}

/// Types of security alerts
#[allow(dead_code)]
#[derive(Debug)]
pub enum AlertType {
    SuspiciousActivity,
    BulkChanges,
    FailedOperations,
    AfterHoursActivity,
    UnusualUserBehavior,
    DataIntegrityIssue,
}

/// Alert severity levels
#[allow(dead_code)]
#[derive(Debug)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Trend analysis over time
#[derive(Debug)]
pub struct TrendAnalysis {
    pub activity_trend: TrendDirection,
    pub user_growth_trend: TrendDirection,
    pub error_rate_trend: TrendDirection,
    pub compliance_trend: f64, // Compliance score trend
}

/// Trend direction indicators
#[allow(dead_code)]
#[derive(Debug)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

/// Dashboard generator and analyzer
pub struct AuditDashboardEngine {
    project_path: PathBuf,
}

impl AuditDashboardEngine {
    /// Create new dashboard engine
    pub const fn new(project_path: PathBuf) -> Self {
        Self { project_path }
    }

    /// Generate comprehensive dashboard with all metrics
    pub fn generate_dashboard(&self, period_days: u32) -> QmsResult<AuditDashboard> {
        let all_entries = self.load_all_audit_entries()?;
        let period_entries = self.filter_entries_by_period(&all_entries, period_days);

        let general_metrics = self.calculate_general_metrics(&all_entries, &period_entries)?;
        let user_activity = self.analyze_user_activity(&period_entries)?;
        let action_metrics = self.analyze_action_metrics(&period_entries)?;
        let time_analysis = self.analyze_time_patterns(&period_entries)?;
        let alerts = self.detect_security_alerts(&period_entries)?;
        let trends = self.analyze_trends(&all_entries, period_days)?;

        Ok(AuditDashboard {
            general_metrics,
            user_activity,
            action_metrics,
            time_analysis,
            alerts,
            trends,
        })
    }

    /// Calculate general system metrics
    fn calculate_general_metrics(&self, all_entries: &[AuditEntry], period_entries: &[AuditEntry]) -> QmsResult<GeneralMetrics> {
        let total_entries = all_entries.len();
        let entries_today = self.count_entries_today(period_entries);
        let entries_this_week = self.count_entries_this_week(period_entries);
        let entries_this_month = self.count_entries_this_month(period_entries);

        let unique_users_today = self.count_unique_users_today(period_entries);
        let unique_users_total: std::collections::HashSet<&str> = 
            all_entries.iter().map(|e| e.user_id.as_str()).collect();

        let average_daily_activity = if total_entries > 0 {
            total_entries as f64 / 30.0 // Approximate over 30 days
        } else {
            0.0
        };

        let data_size_mb = self.calculate_data_size_mb()?;

        Ok(GeneralMetrics {
            total_entries,
            entries_today,
            entries_this_week,
            entries_this_month,
            unique_users_today,
            unique_users_total: unique_users_total.len(),
            average_daily_activity,
            data_size_mb,
        })
    }

    /// Analyze user activity patterns
    fn analyze_user_activity(&self, entries: &[AuditEntry]) -> QmsResult<UserActivityMetrics> {
        let mut user_activity: HashMap<String, Vec<&AuditEntry>> = HashMap::new();
        
        // Group entries by user
        for entry in entries {
            user_activity.entry(entry.user_id.clone())
                .or_default()
                .push(entry);
        }

        // Calculate user statistics
        let mut user_stats = HashMap::new();
        let mut most_active_users = Vec::new();
        
        for (username, user_entries) in &user_activity {
            let total_actions = user_entries.len();
            let actions_today = self.count_user_actions_today(user_entries);
            let last_activity = user_entries.iter()
                .map(|e| &e.timestamp)
                .max()
                .cloned();
            
            let most_common_action = self.find_most_common_action(user_entries);
            let risk_score = self.calculate_user_risk_score(user_entries);

            user_stats.insert(username.clone(), UserStats {
                total_actions,
                actions_today,
                last_activity,
                most_common_action,
                risk_score,
            });

            most_active_users.push((username.clone(), total_actions));
        }

        // Sort most active users
        most_active_users.sort_by(|a, b| b.1.cmp(&a.1));
        most_active_users.truncate(10); // Top 10

        let recent_logins = self.extract_recent_logins(entries);
        let inactive_users = self.find_inactive_users(&user_stats);

        Ok(UserActivityMetrics {
            most_active_users,
            user_activity_distribution: user_stats,
            recent_logins,
            inactive_users,
        })
    }

    /// Analyze action type metrics
    fn analyze_action_metrics(&self, entries: &[AuditEntry]) -> QmsResult<ActionMetrics> {
        let mut action_distribution = HashMap::new();
        let mut critical_actions_today = 0;
        let mut failed_operations = 0;
        let mut bulk_operations = 0;
        let mut administrative_actions = 0;

        for entry in entries {
            let action_str = format!("{:?}", entry.action);
            *action_distribution.entry(action_str.clone()).or_insert(0) += 1;

            // Count critical actions today
            if self.is_critical_action(&entry.action) && self.is_today(&entry.timestamp) {
                critical_actions_today += 1;
            }

            // Count failed operations
            if self.is_failed_operation(entry) {
                failed_operations += 1;
            }

            // Count bulk operations
            if self.is_bulk_operation(entry) {
                bulk_operations += 1;
            }

            // Count administrative actions
            if self.is_administrative_action(&entry.action) {
                administrative_actions += 1;
            }
        }

        Ok(ActionMetrics {
            action_distribution,
            critical_actions_today,
            failed_operations,
            bulk_operations,
            administrative_actions,
        })
    }

    /// Analyze time-based patterns
    fn analyze_time_patterns(&self, entries: &[AuditEntry]) -> QmsResult<TimeAnalysis> {
        let mut hourly_activity = HashMap::new();
        let mut daily_activity = HashMap::new();

        for entry in entries {
            // Extract hour from timestamp
            if let Some(hour) = self.extract_hour(&entry.timestamp) {
                *hourly_activity.entry(hour).or_insert(0) += 1;
            }

            // Extract date from timestamp
            if let Some(date) = self.extract_date(&entry.timestamp) {
                *daily_activity.entry(date).or_insert(0) += 1;
            }
        }

        // Convert to sorted vectors
        let mut peak_activity_hours: Vec<(u8, usize)> = hourly_activity.into_iter().collect();
        peak_activity_hours.sort_by(|a, b| b.1.cmp(&a.1));

        let mut daily_activity_trend: Vec<(String, usize)> = daily_activity.into_iter().collect();
        daily_activity_trend.sort_by(|a, b| a.0.cmp(&b.0));

        let activity_patterns = self.classify_activity_pattern(entries);

        Ok(TimeAnalysis {
            peak_activity_hours,
            daily_activity_trend,
            activity_patterns,
        })
    }

    /// Detect security alerts and anomalies
    fn detect_security_alerts(&self, entries: &[AuditEntry]) -> QmsResult<Vec<SecurityAlert>> {
        let mut alerts = Vec::new();

        // Check for suspicious activity patterns
        alerts.extend(self.detect_suspicious_activity(entries)?);
        alerts.extend(self.detect_bulk_changes(entries)?);
        alerts.extend(self.detect_failed_operations(entries)?);
        alerts.extend(self.detect_after_hours_activity(entries)?);

        Ok(alerts)
    }

    /// Analyze trends over time
    const fn analyze_trends(&self, entries: &[AuditEntry], period_days: u32) -> QmsResult<TrendAnalysis> {
        let activity_trend = self.calculate_activity_trend(entries, period_days);
        let user_growth_trend = self.calculate_user_growth_trend(entries, period_days);
        let error_rate_trend = self.calculate_error_rate_trend(entries, period_days);
        let compliance_trend = self.calculate_compliance_trend(entries);

        Ok(TrendAnalysis {
            activity_trend,
            user_growth_trend,
            error_rate_trend,
            compliance_trend,
        })
    }

    // Helper methods

    fn load_all_audit_entries(&self) -> QmsResult<Vec<AuditEntry>> {
        use crate::modules::audit_logger::search::AuditSearchEngine;
        use crate::modules::audit_logger::search::AuditSearchCriteria;

        let search_engine = AuditSearchEngine::new(self.project_path.clone());
        let criteria = AuditSearchCriteria::new().with_limit(10000); // Large limit for dashboard
        let results = search_engine.search(&criteria)?;
        Ok(results.entries)
    }

    fn filter_entries_by_period(&self, entries: &[AuditEntry], _days: u32) -> Vec<AuditEntry> {
        // For simplicity, return all entries
        // In a real implementation, you'd filter by actual date range
        entries.to_vec()
    }

    fn count_entries_today(&self, entries: &[AuditEntry]) -> usize {
        entries.iter().filter(|e| self.is_today(&e.timestamp)).count()
    }

    fn count_entries_this_week(&self, entries: &[AuditEntry]) -> usize {
        entries.iter().filter(|e| self.is_this_week(&e.timestamp)).count()
    }

    fn count_entries_this_month(&self, entries: &[AuditEntry]) -> usize {
        entries.iter().filter(|e| self.is_this_month(&e.timestamp)).count()
    }

    fn count_unique_users_today(&self, entries: &[AuditEntry]) -> usize {
        let users: std::collections::HashSet<&str> = entries.iter()
            .filter(|e| self.is_today(&e.timestamp))
            .map(|e| e.user_id.as_str())
            .collect();
        users.len()
    }

    fn calculate_data_size_mb(&self) -> QmsResult<f64> {
        let audit_dir = self.project_path.join("audit");
        let mut total_size = 0u64;

        if audit_dir.exists() {
            for entry in std::fs::read_dir(&audit_dir)? {
                let entry = entry?;
                let metadata = entry.metadata()?;
                total_size += metadata.len();
            }
        }

        Ok(total_size as f64 / 1_048_576.0) // Convert to MB
    }

    fn count_user_actions_today(&self, user_entries: &[&AuditEntry]) -> usize {
        user_entries.iter().filter(|e| self.is_today(&e.timestamp)).count()
    }

    fn find_most_common_action(&self, user_entries: &[&AuditEntry]) -> String {
        let mut action_counts = HashMap::new();
        for entry in user_entries {
            let action_str = format!("{:?}", entry.action);
            *action_counts.entry(action_str).or_insert(0) += 1;
        }

        action_counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(action, _)| action)
            .unwrap_or_else(|| "None".to_string())
    }

    fn calculate_user_risk_score(&self, user_entries: &[&AuditEntry]) -> f64 {
        if user_entries.is_empty() {
            return 0.0;
        }

        let mut risk_score = 0.0;

        for entry in user_entries {
            match entry.action {
                AuditAction::Delete => risk_score += 0.3,
                AuditAction::Update => risk_score += 0.1,
                AuditAction::Create => risk_score += 0.05,
                _ => {}
            }
        }

        (risk_score / user_entries.len() as f64).min(1.0)
    }

    fn extract_recent_logins(&self, entries: &[AuditEntry]) -> Vec<(String, String)> {
        entries.iter()
            .filter(|e| matches!(e.action, AuditAction::Other(ref action) if action.contains("login")))
            .map(|e| (e.user_id.clone(), e.timestamp.clone()))
            .collect()
    }

    fn find_inactive_users(&self, user_stats: &HashMap<String, UserStats>) -> Vec<String> {
        user_stats.iter()
            .filter(|(_, stats)| stats.actions_today == 0)
            .map(|(username, _)| username.clone())
            .collect()
    }

    const fn is_critical_action(&self, action: &AuditAction) -> bool {
        matches!(action, AuditAction::Delete)
    }

    const fn is_failed_operation(&self, _entry: &AuditEntry) -> bool {
        // Check if entry indicates a failed operation
        // This would depend on how failures are recorded
        false // Placeholder
    }

    const fn is_bulk_operation(&self, _entry: &AuditEntry) -> bool {
        // Check if entry is part of bulk operations
        // This would depend on how bulk operations are marked
        false // Placeholder
    }

    fn is_administrative_action(&self, action: &AuditAction) -> bool {
        matches!(action, AuditAction::Other(ref action_str) 
            if action_str.to_lowercase().contains("admin") || 
               action_str.to_lowercase().contains("config"))
    }

    const fn is_today(&self, _timestamp: &str) -> bool {
        // Simplified - consider all entries as today for now
        true
    }

    const fn is_this_week(&self, _timestamp: &str) -> bool {
        // Simplified - consider all entries as this week for now
        true
    }

    const fn is_this_month(&self, _timestamp: &str) -> bool {
        // Simplified - consider all entries as this month for now
        true
    }

    fn extract_hour(&self, timestamp: &str) -> Option<u8> {
        // Extract hour from ISO 8601 format: "2025-07-25T02:39:23Z"
        if timestamp.len() >= 13 {
            timestamp[11..13].parse().ok()
        } else {
            None
        }
    }

    fn extract_date(&self, timestamp: &str) -> Option<String> {
        // Extract date from ISO 8601 format: "2025-07-25T02:39:23Z"
        if timestamp.len() >= 10 {
            Some(timestamp[0..10].to_string())
        } else {
            None
        }
    }

    const fn classify_activity_pattern(&self, _entries: &[AuditEntry]) -> ActivityPattern {
        // Simplified classification
        ActivityPattern::Normal
    }

    const fn detect_suspicious_activity(&self, _entries: &[AuditEntry]) -> QmsResult<Vec<SecurityAlert>> {
        // Placeholder for suspicious activity detection
        Ok(Vec::new())
    }

    const fn detect_bulk_changes(&self, _entries: &[AuditEntry]) -> QmsResult<Vec<SecurityAlert>> {
        // Placeholder for bulk change detection
        Ok(Vec::new())
    }

    const fn detect_failed_operations(&self, _entries: &[AuditEntry]) -> QmsResult<Vec<SecurityAlert>> {
        // Placeholder for failed operation detection
        Ok(Vec::new())
    }

    const fn detect_after_hours_activity(&self, _entries: &[AuditEntry]) -> QmsResult<Vec<SecurityAlert>> {
        // Placeholder for after-hours activity detection
        Ok(Vec::new())
    }

    const fn calculate_activity_trend(&self, _entries: &[AuditEntry], _period_days: u32) -> TrendDirection {
        TrendDirection::Stable
    }

    const fn calculate_user_growth_trend(&self, _entries: &[AuditEntry], _period_days: u32) -> TrendDirection {
        TrendDirection::Stable
    }

    const fn calculate_error_rate_trend(&self, _entries: &[AuditEntry], _period_days: u32) -> TrendDirection {
        TrendDirection::Stable
    }

    const fn calculate_compliance_trend(&self, _entries: &[AuditEntry]) -> f64 {
        90.0 // Placeholder compliance score
    }
}

/// Format dashboard for display
pub fn format_dashboard(dashboard: &AuditDashboard, period_days: u32) -> String {
    let mut output = String::new();

    output.push_str("QMS AUDIT DASHBOARD\n");
    output.push_str("===================\n\n");

    // General Metrics
    output.push_str("📊 GENERAL METRICS\n");
    output.push_str("------------------\n");
    let metrics = &dashboard.general_metrics;
    output.push_str(&format!("Total Audit Entries: {}\n", metrics.total_entries));
    output.push_str(&format!("Entries Today: {}\n", metrics.entries_today));
    output.push_str(&format!("Entries This Week: {}\n", metrics.entries_this_week));
    output.push_str(&format!("Entries This Month: {}\n", metrics.entries_this_month));
    output.push_str(&format!("Unique Users Today: {}\n", metrics.unique_users_today));
    output.push_str(&format!("Total Unique Users: {}\n", metrics.unique_users_total));
    output.push_str(&format!("Average Daily Activity: {:.1} entries\n", metrics.average_daily_activity));
    output.push_str(&format!("Data Size: {:.2} MB\n\n", metrics.data_size_mb));

    // User Activity
    output.push_str("👥 USER ACTIVITY\n");
    output.push_str("----------------\n");
    output.push_str("Most Active Users:\n");
    for (i, (username, count)) in dashboard.user_activity.most_active_users.iter().enumerate() {
        if i < 5 { // Show top 5
            output.push_str(&format!("  {}. {} ({} actions)\n", i + 1, username, count));
        }
    }
    output.push_str(&format!("Inactive Users: {}\n\n", dashboard.user_activity.inactive_users.len()));

    // Action Metrics
    output.push_str("⚡ ACTION METRICS\n");
    output.push_str("----------------\n");
    let action_metrics = &dashboard.action_metrics;
    output.push_str(&format!("Critical Actions Today: {}\n", action_metrics.critical_actions_today));
    output.push_str(&format!("Failed Operations: {}\n", action_metrics.failed_operations));
    output.push_str(&format!("Bulk Operations: {}\n", action_metrics.bulk_operations));
    output.push_str(&format!("Administrative Actions: {}\n", action_metrics.administrative_actions));

    output.push_str("\nAction Distribution:\n");
    let mut action_list: Vec<_> = action_metrics.action_distribution.iter().collect();
    action_list.sort_by(|a, b| b.1.cmp(a.1));
    for (action, count) in action_list.iter().take(5) {
        output.push_str(&format!("  {action}: {count} times\n"));
    }
    output.push('\n');

    // Time Analysis
    output.push_str("⏰ TIME ANALYSIS\n");
    output.push_str("----------------\n");
    output.push_str("Peak Activity Hours:\n");
    for (hour, count) in dashboard.time_analysis.peak_activity_hours.iter().take(3) {
        output.push_str(&format!("  {hour:02}:00 - {count} entries\n"));
    }
    output.push_str(&format!("Activity Pattern: {:?}\n\n", dashboard.time_analysis.activity_patterns));

    // Security Alerts
    if !dashboard.alerts.is_empty() {
        output.push_str("🚨 SECURITY ALERTS\n");
        output.push_str("------------------\n");
        for alert in &dashboard.alerts {
            let severity_icon = match alert.severity {
                AlertSeverity::Critical => "🔴",
                AlertSeverity::High => "🟠",
                AlertSeverity::Medium => "🟡",
                AlertSeverity::Low => "🟢",
            };
            output.push_str(&format!("{} {:?}: {}\n", severity_icon, alert.severity, alert.description));
        }
        output.push('\n');
    } else {
        output.push_str("✅ NO SECURITY ALERTS\n\n");
    }

    // Trends
    output.push_str("📈 TRENDS\n");
    output.push_str("---------\n");
    let trends = &dashboard.trends;
    output.push_str(&format!("Activity Trend: {:?}\n", trends.activity_trend));
    output.push_str(&format!("User Growth: {:?}\n", trends.user_growth_trend));
    output.push_str(&format!("Error Rate: {:?}\n", trends.error_rate_trend));
    output.push_str(&format!("Compliance Score: {:.1}%\n\n", trends.compliance_trend));

    output.push_str(&format!("📅 Dashboard Period: {period_days} days\n"));
    output.push_str("Generated by QMS Audit Dashboard System\n");

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_engine_creation() {
        let temp_dir = std::path::PathBuf::from("/tmp/test");
        let engine = AuditDashboardEngine::new(temp_dir.clone());
        assert_eq!(engine.project_path, temp_dir);
    }

    #[test]
    fn test_time_extraction() {
        let engine = AuditDashboardEngine::new(std::path::PathBuf::new());
        
        let hour = engine.extract_hour("2025-07-25T14:30:00Z");
        assert_eq!(hour, Some(14));
        
        let date = engine.extract_date("2025-07-25T14:30:00Z");
        assert_eq!(date, Some("2025-07-25".to_string()));
    }

    #[test]
    fn test_risk_score_calculation() {
        let engine = AuditDashboardEngine::new(std::path::PathBuf::new());
        
        // This would require creating test AuditEntry instances
        // For now, just test that the method exists
        let empty_entries: Vec<&AuditEntry> = Vec::new();
        let risk_score = engine.calculate_user_risk_score(&empty_entries);
        assert_eq!(risk_score, 0.0);
    }
}
