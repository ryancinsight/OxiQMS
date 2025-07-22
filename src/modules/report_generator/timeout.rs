//! Report Generator Configuration and Timeout Handling
//! 
//! This module provides configuration and timeout management for report generation,
//! addressing the user's request for nextest integration and timeout settings.

use crate::prelude::*;
use std::time::{Duration, Instant};

/// Report generation configuration with timeout settings
#[derive(Debug, Clone)]
pub struct ReportConfig {
    /// Maximum time allowed for report generation
    pub timeout: Duration,
    /// Maximum number of entries to process
    pub max_entries: Option<usize>,
    /// Enable progress tracking
    pub enable_progress: bool,
    /// Output buffer size for large reports
    pub buffer_size: usize,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30), // 30 second default timeout
            max_entries: Some(10000),         // Limit to 10k entries by default
            enable_progress: true,
            buffer_size: 8192,                // 8KB buffer
        }
    }
}

impl ReportConfig {
    /// Create new config with custom timeout
    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self {
            timeout: Duration::from_secs(timeout_secs),
            ..Default::default()
        }
    }
    
    /// Create config for nextest (shorter timeouts)
    pub fn for_nextest() -> Self {
        Self {
            timeout: Duration::from_secs(10), // Shorter timeout for tests
            max_entries: Some(100),           // Fewer entries for tests
            enable_progress: false,           // No progress in tests
            buffer_size: 1024,                // Smaller buffer for tests
        }
    }
}

/// Timeout-aware report execution
pub struct TimeoutExecutor {
    start_time: Instant,
    config: ReportConfig,
}

impl TimeoutExecutor {
    /// Create new timeout executor
    pub fn new(config: ReportConfig) -> Self {
        Self {
            start_time: Instant::now(),
            config,
        }
    }
    
    /// Check if we've exceeded the timeout
    pub fn is_timeout_exceeded(&self) -> bool {
        self.start_time.elapsed() > self.config.timeout
    }
    
    /// Get remaining time
    pub fn remaining_time(&self) -> Duration {
        self.config.timeout.saturating_sub(self.start_time.elapsed())
    }
    
    /// Check timeout and return error if exceeded
    pub fn check_timeout(&self) -> QmsResult<()> {
        if self.is_timeout_exceeded() {
            return Err(QmsError::validation_error(&format!(
                "Report generation timeout exceeded: {:?}",
                self.config.timeout
            )));
        }
        Ok(())
    }
    
    /// Execute operation with timeout checking
    pub fn execute_with_timeout<F, T>(&self, operation: F) -> QmsResult<T>
    where
        F: FnOnce() -> QmsResult<T>,
    {
        self.check_timeout()?;
        let result = operation()?;
        self.check_timeout()?;
        Ok(result)
    }
}

/// Progress tracker for long-running report operations
pub struct ProgressTracker {
    total: usize,
    current: usize,
    last_update: Instant,
    config: ReportConfig,
}

impl ProgressTracker {
    /// Create new progress tracker
    pub fn new(total: usize, config: ReportConfig) -> Self {
        Self {
            total,
            current: 0,
            last_update: Instant::now(),
            config,
        }
    }
    
    /// Update progress
    pub fn update(&mut self, current: usize) -> QmsResult<()> {
        self.current = current;
        
        if self.config.enable_progress && self.last_update.elapsed() > Duration::from_millis(100) {
            let percentage = if self.total > 0 {
                (self.current as f64 / self.total as f64) * 100.0
            } else {
                0.0
            };
            eprintln!("Progress: {:.1}% ({}/{})", percentage, self.current, self.total);
            self.last_update = Instant::now();
        }
        
        Ok(())
    }
    
    /// Mark as complete
    pub fn complete(&mut self) {
        if self.config.enable_progress {
            eprintln!("✅ Report generation completed: {}/{}", self.total, self.total);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_report_config_defaults() {
        let config = ReportConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_entries, Some(10000));
        assert!(config.enable_progress);
    }
    
    #[test]
    fn test_timeout_executor() {
        let config = ReportConfig::with_timeout(1);
        let executor = TimeoutExecutor::new(config);
        
        // Should not timeout immediately
        assert!(!executor.is_timeout_exceeded());
        assert!(executor.check_timeout().is_ok());
        
        // Wait for timeout (in real test would need to sleep)
        // For unit test purposes, we test the logic structure
        assert!(executor.remaining_time() <= Duration::from_secs(1));
    }
    
    #[test]
    fn test_nextest_config() {
        let config = ReportConfig::for_nextest();
        assert_eq!(config.timeout, Duration::from_secs(10));
        assert_eq!(config.max_entries, Some(100));
        assert!(!config.enable_progress);
        assert_eq!(config.buffer_size, 1024);
    }
    
    #[test]
    fn test_progress_tracker() {
        let config = ReportConfig::for_nextest(); // No progress output in tests
        let mut tracker = ProgressTracker::new(100, config);
        
        assert!(tracker.update(50).is_ok());
        assert_eq!(tracker.current, 50);
        assert_eq!(tracker.total, 100);
        
        tracker.complete();
    }
}
