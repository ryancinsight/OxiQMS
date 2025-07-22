use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use crate::prelude::*;

/// Configuration for audit performance optimizations
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    pub buffer_size: usize,           // Entries to buffer before writing
    pub flush_interval_ms: u64,       // Auto-flush interval in milliseconds
    pub index_enabled: bool,          // Enable search indexing
    pub cache_size: usize,           // Max cached entries
    pub batch_write_enabled: bool,    // Enable batch write operations
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            buffer_size: 100,
            flush_interval_ms: 5000,  // 5 seconds
            index_enabled: true,
            cache_size: 1000,
            batch_write_enabled: true,
        }
    }
}

/// Performance metrics for audit operations
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_operations: u64,
    pub buffered_writes: u64,
    pub flushed_batches: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub index_searches: u64,
    pub avg_write_time_ms: f64,
    pub avg_search_time_ms: f64,
    pub memory_usage_mb: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMetrics {
    pub const fn new() -> Self {
        Self {
            total_operations: 0,
            buffered_writes: 0,
            flushed_batches: 0,
            cache_hits: 0,
            cache_misses: 0,
            index_searches: 0,
            avg_write_time_ms: 0.0,
            avg_search_time_ms: 0.0,
            memory_usage_mb: 0.0,
        }
    }

    pub fn cache_hit_ratio(&self) -> f64 {
        if self.cache_hits + self.cache_misses == 0 {
            0.0
        } else {
            self.cache_hits as f64 / (self.cache_hits + self.cache_misses) as f64
        }
    }
}

/// Buffered audit entry for performance optimization
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct BufferedEntry {
    content: String,
    timestamp: u64,
}

/// Search index for fast audit log searching
#[derive(Debug, Clone)]
struct SearchIndex {
    user_index: HashMap<String, Vec<u64>>,      // user -> entry timestamps
    action_index: HashMap<String, Vec<u64>>,    // action -> entry timestamps
    entity_index: HashMap<String, Vec<u64>>,    // entity_id -> entry timestamps
    date_index: HashMap<String, Vec<u64>>,      // date -> entry timestamps
}

impl SearchIndex {
    fn new() -> Self {
        Self {
            user_index: HashMap::new(),
            action_index: HashMap::new(),
            entity_index: HashMap::new(),
            date_index: HashMap::new(),
        }
    }

    fn add_entry(&mut self, user: &str, action: &str, entity_id: &str, timestamp: u64) {
        // Index by user
        self.user_index.entry(user.to_string())
            .or_default()
            .push(timestamp);

        // Index by action
        self.action_index.entry(action.to_string())
            .or_default()
            .push(timestamp);

        // Index by entity
        self.entity_index.entry(entity_id.to_string())
            .or_default()
            .push(timestamp);

        // Index by date (YYYY-MM-DD format)
        let date_str = timestamp_to_date_string(timestamp);
        self.date_index.entry(date_str)
            .or_default()
            .push(timestamp);
    }

    fn search_by_user(&self, user: &str) -> Option<&Vec<u64>> {
        self.user_index.get(user)
    }

    fn search_by_action(&self, action: &str) -> Option<&Vec<u64>> {
        self.action_index.get(action)
    }

    fn search_by_entity(&self, entity_id: &str) -> Option<&Vec<u64>> {
        self.entity_index.get(entity_id)
    }

    fn search_by_date(&self, date: &str) -> Option<&Vec<u64>> {
        self.date_index.get(date)
    }
}

/// Performance-optimized audit logger
#[allow(dead_code)]
pub struct PerformanceAuditLogger {
    config: PerformanceConfig,
    project_path: PathBuf,
    buffer: Vec<BufferedEntry>,
    last_flush: u64,
    search_index: SearchIndex,
    metrics: PerformanceMetrics,
    cache: HashMap<String, String>,  // Simple LRU-like cache
}

impl PerformanceAuditLogger {
    pub fn new(project_path: PathBuf) -> Self {
        Self {
            config: PerformanceConfig::default(),
            project_path,
            buffer: Vec::new(),
            last_flush: current_timestamp(),
            search_index: SearchIndex::new(),
            metrics: PerformanceMetrics::new(),
            cache: HashMap::new(),
        }
    }

    pub fn with_config(project_path: PathBuf, config: PerformanceConfig) -> Self {
        Self {
            config,
            project_path,
            buffer: Vec::new(),
            last_flush: current_timestamp(),
            search_index: SearchIndex::new(),
            metrics: PerformanceMetrics::new(),
            cache: HashMap::new(),
        }
    }

    /// Buffer an audit entry for efficient writing
    pub fn buffer_entry(&mut self, entry_content: &str) -> QmsResult<()> {
        let start_time = SystemTime::now();

        let buffered_entry = BufferedEntry {
            content: entry_content.to_string(),
            timestamp: current_timestamp(),
        };

        self.buffer.push(buffered_entry);
        self.metrics.buffered_writes += 1;

        // Parse entry for indexing
        if self.config.index_enabled {
            self.update_search_index(entry_content)?;
        }

        // Auto-flush if buffer is full or time interval exceeded
        let should_flush = self.buffer.len() >= self.config.buffer_size ||
            (current_timestamp() - self.last_flush) * 1000 > self.config.flush_interval_ms;

        if should_flush {
            self.flush_buffer()?;
        }

        // Update metrics
        if let Ok(duration) = start_time.elapsed() {
            let duration_ms = duration.as_millis() as f64;
            self.update_avg_write_time(duration_ms);
        }

        self.metrics.total_operations += 1;
        Ok(())
    }

    /// Flush buffered entries to disk
    pub fn flush_buffer(&mut self) -> QmsResult<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let audit_log_path = self.project_path.join("audit").join("audit.log");
        
        // Ensure audit directory exists
        if let Some(parent) = audit_log_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| QmsError::io_error(&format!("Failed to create audit directory: {e}")))?;
        }

        // Write all buffered entries in a single operation for efficiency
        let mut content = String::new();
        for entry in &self.buffer {
            content.push_str(&entry.content);
            content.push('\n');
        }

        // Append to log file
        fs::write(&audit_log_path, &content)
            .map_err(|e| QmsError::io_error(&format!("Failed to write audit log: {e}")))?;

        self.metrics.flushed_batches += 1;
        self.buffer.clear();
        self.last_flush = current_timestamp();

        Ok(())
    }

    /// Update search index with new entry
    fn update_search_index(&mut self, entry_content: &str) -> QmsResult<()> {
        // Parse JSON entry to extract indexable fields
        if let Some((user, action, entity_id, timestamp)) = self.parse_audit_entry(entry_content) {
            self.search_index.add_entry(&user, &action, &entity_id, timestamp);
        }
        Ok(())
    }

    /// Parse audit entry to extract key fields for indexing
    fn parse_audit_entry(&self, entry_content: &str) -> Option<(String, String, String, u64)> {
        let mut user = String::new();
        let mut action = String::new();
        let mut entity_id = String::new();
        let mut timestamp = 0u64;

        // Simple JSON parsing for key fields
        for line in entry_content.lines() {
            let line = line.trim();
            if line.contains("\"user_id\"") {
                if let Some(value) = extract_json_string_value(line) {
                    user = value;
                }
            } else if line.contains("\"action\"") {
                if let Some(value) = extract_json_string_value(line) {
                    action = value;
                }
            } else if line.contains("\"entity_id\"") {
                if let Some(value) = extract_json_string_value(line) {
                    entity_id = value;
                }
            } else if line.contains("\"timestamp\"") {
                if let Some(value) = extract_json_number_value(line) {
                    timestamp = value.parse().unwrap_or(0);
                }
            }
        }

        if !user.is_empty() && !action.is_empty() {
            Some((user, action, entity_id, timestamp))
        } else {
            None
        }
    }

    /// Fast search using built indexes
    pub fn indexed_search(&mut self, user: Option<&str>, action: Option<&str>, entity_id: Option<&str>, date: Option<&str>) -> QmsResult<Vec<u64>> {
        let start_time = SystemTime::now();
        let mut results = Vec::new();

        if let Some(user) = user {
            if let Some(timestamps) = self.search_index.search_by_user(user) {
                results.extend_from_slice(timestamps);
                self.metrics.cache_hits += 1;
            } else {
                self.metrics.cache_misses += 1;
            }
        }

        if let Some(action) = action {
            if let Some(timestamps) = self.search_index.search_by_action(action) {
                if results.is_empty() {
                    results.extend_from_slice(timestamps);
                } else {
                    // Intersect with previous results
                    results.retain(|&ts| timestamps.contains(&ts));
                }
                self.metrics.cache_hits += 1;
            } else {
                self.metrics.cache_misses += 1;
            }
        }

        if let Some(entity_id) = entity_id {
            if let Some(timestamps) = self.search_index.search_by_entity(entity_id) {
                if results.is_empty() {
                    results.extend_from_slice(timestamps);
                } else {
                    results.retain(|&ts| timestamps.contains(&ts));
                }
                self.metrics.cache_hits += 1;
            } else {
                self.metrics.cache_misses += 1;
            }
        }

        if let Some(date) = date {
            if let Some(timestamps) = self.search_index.search_by_date(date) {
                if results.is_empty() {
                    results.extend_from_slice(timestamps);
                } else {
                    results.retain(|&ts| timestamps.contains(&ts));
                }
                self.metrics.cache_hits += 1;
            } else {
                self.metrics.cache_misses += 1;
            }
        }

        // Update search metrics
        if let Ok(duration) = start_time.elapsed() {
            let duration_ms = duration.as_millis() as f64;
            self.update_avg_search_time(duration_ms);
        }

        self.metrics.index_searches += 1;
        results.sort_by(|a, b| b.cmp(a)); // Sort descending (newest first)
        Ok(results)
    }

    /// Stream processing for large files with pagination
    #[allow(dead_code)]
    pub fn stream_search(&self, file_path: &std::path::Path, pattern: &str, offset: usize, limit: usize) -> QmsResult<Vec<String>> {
        use std::io::{BufRead, BufReader};
        
        let file = fs::File::open(file_path)
            .map_err(|e| QmsError::io_error(&format!("Failed to open file: {e}")))?;
        
        let reader = BufReader::new(file);
        let mut results = Vec::new();
        let mut _line_count = 0;
        let mut matched_count = 0;

        for line in reader.lines() {
            let line = line
                .map_err(|e| QmsError::io_error(&format!("Failed to read line: {e}")))?;

            if line.contains(pattern) {
                if matched_count >= offset {
                    results.push(line);
                    if results.len() >= limit {
                        break;
                    }
                }
                matched_count += 1;
            }
        }

        Ok(results)
    }

    /// Get current performance metrics
    pub const fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }

    /// Reset performance metrics
    #[allow(dead_code)]
    pub fn reset_metrics(&mut self) {
        self.metrics = PerformanceMetrics::new();
    }

    /// Cache management
    #[allow(dead_code)]
    pub fn cache_put(&mut self, key: String, value: String) {
        if self.cache.len() >= self.config.cache_size {
            // Simple cache eviction - remove oldest entry
            if let Some(oldest_key) = self.cache.keys().next().cloned() {
                self.cache.remove(&oldest_key);
            }
        }
        self.cache.insert(key, value);
    }

    #[allow(dead_code)]
    pub fn cache_get(&mut self, key: &str) -> Option<String> {
        if let Some(value) = self.cache.get(key) {
            self.metrics.cache_hits += 1;
            Some(value.clone())
        } else {
            self.metrics.cache_misses += 1;
            None
        }
    }

    /// Update average write time metric
    fn update_avg_write_time(&mut self, duration_ms: f64) {
        let count = self.metrics.buffered_writes as f64;
        self.metrics.avg_write_time_ms = 
            (self.metrics.avg_write_time_ms * (count - 1.0) + duration_ms) / count;
    }

    /// Update average search time metric
    fn update_avg_search_time(&mut self, duration_ms: f64) {
        let count = self.metrics.index_searches as f64;
        self.metrics.avg_search_time_ms = 
            (self.metrics.avg_search_time_ms * (count - 1.0) + duration_ms) / count;
    }

    /// Build search index from existing log files
    pub fn build_index_from_logs(&mut self) -> QmsResult<()> {
        let audit_dir = self.project_path.join("audit");
        if !audit_dir.exists() {
            return Ok(());
        }

        // Index main audit log
        let main_log = audit_dir.join("audit.log");
        if main_log.exists() {
            self.index_log_file(&main_log)?;
        }

        // Index daily logs
        let daily_dir = audit_dir.join("daily");
        if daily_dir.exists() {
            let entries = fs::read_dir(&daily_dir)
                .map_err(|e| QmsError::io_error(&format!("Failed to read daily logs directory: {e}")))?;

            for entry in entries {
                let entry = entry
                    .map_err(|e| QmsError::io_error(&format!("Failed to read daily log entry: {e}")))?;
                let path = entry.path();

                if path.extension().is_some_and(|ext| ext == "log") {
                    self.index_log_file(&path)?;
                }
            }
        }

        Ok(())
    }

    /// Index a single log file
    fn index_log_file(&mut self, log_path: &std::path::Path) -> QmsResult<()> {
        use std::io::{BufRead, BufReader};
        
        let file = fs::File::open(log_path)
            .map_err(|e| QmsError::io_error(&format!("Failed to open log file: {e}")))?;
        
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line
                .map_err(|e| QmsError::io_error(&format!("Failed to read log line: {e}")))?;
            
            self.update_search_index(&line)?;
        }

        Ok(())
    }
}

/// Extract string value from JSON line
fn extract_json_string_value(line: &str) -> Option<String> {
    if let Some(start) = line.find(": \"") {
        if let Some(end) = line[start + 3..].find('"') {
            return Some(line[start + 3..start + 3 + end].to_string());
        }
    }
    None
}

/// Extract number value from JSON line
fn extract_json_number_value(line: &str) -> Option<String> {
    if let Some(colon_pos) = line.find(':') {
        let after_colon = &line[colon_pos + 1..];
        let cleaned = after_colon.trim().trim_end_matches(',');
        return Some(cleaned.to_string());
    }
    None
}

/// Convert timestamp to date string (YYYY-MM-DD)
fn timestamp_to_date_string(timestamp: u64) -> String {
    // Simple conversion - in real implementation would use proper date formatting
    let days_since_epoch = timestamp / (24 * 60 * 60);
    let base_year = 1970;
    let year = base_year + (days_since_epoch / 365);
    let month = ((days_since_epoch % 365) / 30) + 1;
    let day = ((days_since_epoch % 365) % 30) + 1;
    
    format!("{:04}-{:02}-{:02}", year, month.min(12), day.min(31))
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Format performance metrics for display
pub fn format_performance_metrics(metrics: &PerformanceMetrics) -> String {
    let mut output = String::new();
    
    output.push_str("📊 Audit Performance Metrics\n");
    output.push_str("============================\n");
    output.push_str(&format!("Total operations: {}\n", metrics.total_operations));
    output.push_str(&format!("Buffered writes: {}\n", metrics.buffered_writes));
    output.push_str(&format!("Flushed batches: {}\n", metrics.flushed_batches));
    output.push_str(&format!("Index searches: {}\n", metrics.index_searches));
    output.push('\n');
    
    output.push_str("🚀 Performance:\n");
    output.push_str(&format!("Avg write time: {:.2} ms\n", metrics.avg_write_time_ms));
    output.push_str(&format!("Avg search time: {:.2} ms\n", metrics.avg_search_time_ms));
    output.push('\n');
    
    output.push_str("💾 Cache:\n");
    output.push_str(&format!("Cache hits: {}\n", metrics.cache_hits));
    output.push_str(&format!("Cache misses: {}\n", metrics.cache_misses));
    output.push_str(&format!("Cache hit ratio: {:.1}%\n", metrics.cache_hit_ratio() * 100.0));
    output.push_str(&format!("Memory usage: {:.2} MB\n", metrics.memory_usage_mb));
    
    output
}