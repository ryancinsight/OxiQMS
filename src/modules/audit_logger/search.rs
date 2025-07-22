//! Audit Search & Filtering Module
//! 
//! Provides comprehensive search and filtering capabilities for audit trail data.
//! Supports searching by user, action, entity type, date ranges, and more.

use crate::prelude::*;
use crate::models::AuditEntry;
use crate::json_utils::JsonSerializable;
use std::collections::HashMap;

/// Search criteria for audit entries
#[derive(Debug, Clone)]
pub struct AuditSearchCriteria {
    pub user_filter: Option<String>,
    pub action_filter: Option<String>,
    pub entity_type_filter: Option<String>,
    pub entity_id_filter: Option<String>,
    pub date_start: Option<u64>,
    pub date_end: Option<u64>,
    pub details_keyword: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Search results with metadata
#[derive(Debug)]
pub struct AuditSearchResults {
    pub entries: Vec<AuditEntry>,
    pub total_matches: usize,
    pub search_duration_ms: u128,
    pub sources_searched: Vec<String>,
}

/// Output format for search results
#[derive(Debug, Clone)]
pub enum AuditOutputFormat {
    Table,
    Json,
    Csv,
}

impl Default for AuditSearchCriteria {
    fn default() -> Self {
        Self {
            user_filter: None,
            action_filter: None,
            entity_type_filter: None,
            entity_id_filter: None,
            date_start: None,
            date_end: None,
            details_keyword: None,
            limit: Some(100), // Default limit
            offset: None,
        }
    }
}

impl AuditSearchCriteria {
    /// Create new search criteria
    pub fn new() -> Self {
        Self::default()
    }

    /// Set user filter
    pub fn with_user(mut self, user: &str) -> Self {
        self.user_filter = Some(user.to_string());
        self
    }

    /// Set action filter
    pub fn with_action(mut self, action: &str) -> Self {
        self.action_filter = Some(action.to_string());
        self
    }

    /// Set entity type filter
    pub fn with_entity_type(mut self, entity_type: &str) -> Self {
        self.entity_type_filter = Some(entity_type.to_string());
        self
    }

    /// Set entity ID filter
    pub fn with_entity_id(mut self, entity_id: &str) -> Self {
        self.entity_id_filter = Some(entity_id.to_string());
        self
    }

    /// Set date range filter
    pub const fn with_date_range(mut self, start: u64, end: u64) -> Self {
        self.date_start = Some(start);
        self.date_end = Some(end);
        self
    }

    /// Set details keyword filter
    pub fn with_details_keyword(mut self, keyword: &str) -> Self {
        self.details_keyword = Some(keyword.to_string());
        self
    }

    /// Set result limit
    pub const fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set result offset for pagination
    pub const fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}

/// Audit search engine
pub struct AuditSearchEngine {
    project_path: PathBuf,
}

impl AuditSearchEngine {
    /// Create new search engine
    pub const fn new(project_path: PathBuf) -> Self {
        Self { project_path }
    }

    /// Search audit entries with criteria
    pub fn search(&self, criteria: &AuditSearchCriteria) -> QmsResult<AuditSearchResults> {
        let start_time = std::time::Instant::now();
        let mut all_entries = Vec::new();
        let mut sources_searched = Vec::new();

        // Search main audit log
        let main_log_path = self.project_path.join("audit").join("audit.log");
        if main_log_path.exists() {
            let entries = self.search_file(&main_log_path, criteria)?;
            all_entries.extend(entries);
            sources_searched.push("audit/audit.log".to_string());
        }

        // Fallback: search root level audit.log for legacy compatibility
        let root_log_path = self.project_path.join("audit.log");
        if root_log_path.exists() && !main_log_path.exists() {
            // Only search root if main doesn't exist to avoid duplicates
            let entries = self.search_file(&root_log_path, criteria)?;
            all_entries.extend(entries);
            sources_searched.push("audit.log".to_string());
        }

        // Search daily logs if they exist
        let daily_dir = self.project_path.join("audit").join("daily");
        if daily_dir.exists() {
            for entry in fs::read_dir(&daily_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "log") {
                    let entries = self.search_file(&path, criteria)?;
                    all_entries.extend(entries);
                    if let Some(filename) = path.file_name() {
                        sources_searched.push(format!("daily/{}", filename.to_string_lossy()));
                    }
                }
            }
        }

        // Sort by timestamp (most recent first)
        all_entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        let total_matches = all_entries.len();

        // Apply pagination
        let offset = criteria.offset.unwrap_or(0);
        let limit = criteria.limit.unwrap_or(total_matches);
        
        let end_index = std::cmp::min(offset + limit, total_matches);
        let paginated_entries = if offset < total_matches {
            all_entries[offset..end_index].to_vec()
        } else {
            Vec::new()
        };

        let search_duration = start_time.elapsed();

        Ok(AuditSearchResults {
            entries: paginated_entries,
            total_matches,
            search_duration_ms: search_duration.as_millis(),
            sources_searched,
        })
    }

    /// Search a specific audit log file
    fn search_file(&self, file_path: &Path, criteria: &AuditSearchCriteria) -> QmsResult<Vec<AuditEntry>> {
        let content = fs::read_to_string(file_path)?;
        let mut matching_entries = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Try to parse as JSON audit entry
            match AuditEntry::from_json(line) {
                Ok(entry) => {
                    if self.matches_criteria(&entry, criteria) {
                        matching_entries.push(entry);
                    }
                }
                Err(_) => {
                    // Skip malformed entries or old text format entries
                    continue;
                }
            }
        }

        Ok(matching_entries)
    }

    /// Check if an entry matches the search criteria
    fn matches_criteria(&self, entry: &AuditEntry, criteria: &AuditSearchCriteria) -> bool {
        // User filter
        if let Some(ref user_filter) = criteria.user_filter {
            if !entry.user_id.to_lowercase().contains(&user_filter.to_lowercase()) {
                return false;
            }
        }

        // Action filter
        if let Some(ref action_filter) = criteria.action_filter {
            let action_str = format!("{:?}", entry.action);
            if !action_str.to_lowercase().contains(&action_filter.to_lowercase()) {
                return false;
            }
        }

        // Entity type filter
        if let Some(ref entity_type_filter) = criteria.entity_type_filter {
            if !entry.entity_type.to_lowercase().contains(&entity_type_filter.to_lowercase()) {
                return false;
            }
        }

        // Entity ID filter
        if let Some(ref entity_id_filter) = criteria.entity_id_filter {
            if !entry.entity_id.to_lowercase().contains(&entity_id_filter.to_lowercase()) {
                return false;
            }
        }

        // Date range filter
        let entry_timestamp = self.parse_timestamp(&entry.timestamp).unwrap_or(0);
        if let Some(start) = criteria.date_start {
            if entry_timestamp < start {
                return false;
            }
        }
        if let Some(end) = criteria.date_end {
            if entry_timestamp > end {
                return false;
            }
        }

        // Details keyword filter
        if let Some(ref keyword) = criteria.details_keyword {
            if let Some(ref details) = entry.details {
                if !details.to_lowercase().contains(&keyword.to_lowercase()) {
                    return false;
                }
            } else {
                return false; // No details to search
            }
        }

        true
    }

    /// Parse timestamp string to u64
    fn parse_timestamp(&self, timestamp_str: &str) -> QmsResult<u64> {
        // Parse ISO 8601 format: "2025-07-25T01:11:10Z"
        // For simplicity, we'll extract the date parts and convert to Unix timestamp
        // In a real implementation, you'd want proper ISO 8601 parsing
        
        if timestamp_str.len() >= 19 {
            // Extract year, month, day, hour, minute, second
            let year: u64 = timestamp_str[0..4].parse().unwrap_or(2025);
            let month: u64 = timestamp_str[5..7].parse().unwrap_or(1);
            let day: u64 = timestamp_str[8..10].parse().unwrap_or(1);
            let hour: u64 = timestamp_str[11..13].parse().unwrap_or(0);
            let minute: u64 = timestamp_str[14..16].parse().unwrap_or(0);
            let second: u64 = timestamp_str[17..19].parse().unwrap_or(0);
            
            // Rough conversion to Unix timestamp (not exact, but good for filtering)
            let days_since_epoch = (year - 1970) * 365 + (month - 1) * 30 + (day - 1);
            let timestamp = days_since_epoch * 24 * 3600 + hour * 3600 + minute * 60 + second;
            
            Ok(timestamp)
        } else {
            Err(QmsError::parse_error("Invalid timestamp format"))
        }
    }

    /// Get audit statistics
    #[allow(dead_code)]
    pub fn get_statistics(&self) -> QmsResult<AuditStatistics> {
        let criteria = AuditSearchCriteria::new().with_limit(10000); // Large limit for stats
        let results = self.search(&criteria)?;
        
        let mut stats = AuditStatistics {
            total_entries: results.total_matches,
            users: HashMap::new(),
            actions: HashMap::new(),
            entity_types: HashMap::new(),
            date_range: None,
            sources: results.sources_searched,
        };

        // Analyze entries for statistics
        for entry in &results.entries {
            // Count by user
            *stats.users.entry(entry.user_id.clone()).or_insert(0) += 1;
            
            // Count by action
            let action_str = format!("{:?}", entry.action);
            *stats.actions.entry(action_str).or_insert(0) += 1;
            
            // Count by entity type
            *stats.entity_types.entry(entry.entity_type.clone()).or_insert(0) += 1;
        }

        Ok(stats)
    }
}

/// Audit statistics summary
#[allow(dead_code)]
#[derive(Debug)]
pub struct AuditStatistics {
    pub total_entries: usize,
    pub users: HashMap<String, usize>,
    pub actions: HashMap<String, usize>,
    pub entity_types: HashMap<String, usize>,
    pub date_range: Option<(String, String)>,
    pub sources: Vec<String>,
}

/// Format search results for output
pub fn format_search_results(results: &AuditSearchResults, format: &AuditOutputFormat) -> String {
    match format {
        AuditOutputFormat::Table => format_table(results),
        AuditOutputFormat::Json => format_json(results),
        AuditOutputFormat::Csv => format_csv(results),
    }
}

/// Format results as a table
fn format_table(results: &AuditSearchResults) -> String {
    let mut output = String::new();
    
    output.push_str("Audit Search Results\n");
    output.push_str("==================\n");
    output.push_str(&format!("Total matches: {} (showing {} entries)\n", 
                            results.total_matches, results.entries.len()));
    output.push_str(&format!("Search duration: {}ms\n", results.search_duration_ms));
    output.push_str(&format!("Sources searched: {}\n\n", results.sources_searched.join(", ")));
    
    if results.entries.is_empty() {
        output.push_str("No matching audit entries found.\n");
        return output;
    }

    // Table header
    output.push_str(&format!("{:<20} {:<15} {:<10} {:<15} {:<20} {:<30}\n",
                            "Timestamp", "User", "Action", "Entity Type", "Entity ID", "Details"));
    output.push_str(&format!("{}\n", "-".repeat(110)));

    // Table rows
    for entry in &results.entries {
        let details = entry.details.as_ref()
            .map(|d| if d.len() > 25 { format!("{}...", &d[..25]) } else { d.clone() })
            .unwrap_or_else(|| "".to_string());
        
        output.push_str(&format!("{:<20} {:<15} {:<10} {:<15} {:<20} {:<30}\n",
                                &entry.timestamp[..19], // Trim to just date/time
                                &entry.user_id,
                                format!("{:?}", entry.action),
                                &entry.entity_type,
                                &entry.entity_id,
                                details));
    }

    output
}

/// Format results as JSON
fn format_json(results: &AuditSearchResults) -> String {
    let mut json = String::from("{\n");
    json.push_str(&format!("  \"total_matches\": {},\n", results.total_matches));
    json.push_str(&format!("  \"entries_shown\": {},\n", results.entries.len()));
    json.push_str(&format!("  \"search_duration_ms\": {},\n", results.search_duration_ms));
    json.push_str("  \"entries\": [\n");
    
    for (i, entry) in results.entries.iter().enumerate() {
        if i > 0 {
            json.push_str(",\n");
        }
        json.push_str("    ");
        json.push_str(&entry.to_json());
    }
    
    json.push_str("\n  ]\n");
    json.push_str("}\n");
    json
}

/// Format results as CSV
fn format_csv(results: &AuditSearchResults) -> String {
    let mut csv = String::from("timestamp,user_id,action,entity_type,entity_id,details\n");
    
    for entry in &results.entries {
        let details = entry.details.as_ref()
            .map(|d| d.replace("\"", "\"\"")) // Escape quotes
            .unwrap_or_else(|| "".to_string());
        
        csv.push_str(&format!("\"{}\",\"{}\",\"{:?}\",\"{}\",\"{}\",\"{}\"\n",
                             entry.timestamp,
                             entry.user_id,
                             entry.action,
                             entry.entity_type,
                             entry.entity_id,
                             details));
    }
    
    csv
}

/// Parse date string to Unix timestamp
pub fn parse_date_to_timestamp(date_str: &str) -> QmsResult<u64> {
    // Accept formats: YYYY-MM-DD, YYYY-MM-DD HH:MM:SS
    let parts: Vec<&str> = date_str.split_whitespace().collect();
    let date_part = parts[0];
    let time_part = parts.get(1).unwrap_or(&"00:00:00");
    
    let date_components: Vec<&str> = date_part.split('-').collect();
    if date_components.len() != 3 {
        return Err(QmsError::parse_error("Invalid date format. Use YYYY-MM-DD"));
    }
    
    let time_components: Vec<&str> = time_part.split(':').collect();
    if time_components.len() != 3 {
        return Err(QmsError::parse_error("Invalid time format. Use HH:MM:SS"));
    }
    
    let year: u64 = date_components[0].parse()
        .map_err(|_| QmsError::parse_error("Invalid year"))?;
    let month: u64 = date_components[1].parse()
        .map_err(|_| QmsError::parse_error("Invalid month"))?;
    let day: u64 = date_components[2].parse()            .map_err(|_| QmsError::parse_error("Invalid day"))?;
    
    let hour: u64 = time_components[0].parse()
        .map_err(|_| QmsError::parse_error("Invalid hour"))?;
    let minute: u64 = time_components[1].parse()
        .map_err(|_| QmsError::parse_error("Invalid minute"))?;
    let second: u64 = time_components[2].parse()
        .map_err(|_| QmsError::parse_error("Invalid second"))?;
    
    // Rough conversion to Unix timestamp (good enough for filtering)
    let days_since_epoch = (year - 1970) * 365 + (month - 1) * 30 + (day - 1);
    let timestamp = days_since_epoch * 24 * 3600 + hour * 3600 + minute * 60 + second;
    
    Ok(timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_criteria_builder() {
        let criteria = AuditSearchCriteria::new()
            .with_user("testuser")
            .with_action("Create")
            .with_limit(50);
        
        assert_eq!(criteria.user_filter, Some("testuser".to_string()));
        assert_eq!(criteria.action_filter, Some("Create".to_string()));
        assert_eq!(criteria.limit, Some(50));
    }

    #[test]
    fn test_date_parsing() {
        let timestamp = parse_date_to_timestamp("2025-07-25").expect("Should parse date");
        assert!(timestamp > 0);
        
        let timestamp_with_time = parse_date_to_timestamp("2025-07-25 15:30:45").expect("Should parse datetime");
        assert!(timestamp_with_time > timestamp);
    }

    #[test]
    fn test_format_output() {
        let results = AuditSearchResults {
            entries: Vec::new(),
            total_matches: 0,
            search_duration_ms: 10,
            sources_searched: vec!["audit.log".to_string()],
        };
        
        let table_output = format_search_results(&results, &AuditOutputFormat::Table);
        assert!(table_output.contains("No matching audit entries found"));
        
        let json_output = format_search_results(&results, &AuditOutputFormat::Json);
        assert!(json_output.contains("\"total_matches\": 0"));
    }
}
