/// SOLID Principles Enhancement: Storage Interfaces
/// 
/// This module provides storage interfaces that follow SOLID principles:
/// - Interface Segregation: Small, focused interfaces for different storage operations
/// - Dependency Inversion: Concrete implementations depend on these abstractions
/// - Single Responsibility: Each interface has one clear purpose

use crate::prelude::*;
use std::path::Path;

/// Generic storage reader interface - Interface Segregation Principle
/// Focused solely on reading operations
pub trait StorageReader<T> {
    /// Read a single item by ID
    fn read(&self, id: &str) -> QmsResult<T>;
    
    /// Read all items
    fn read_all(&self) -> QmsResult<Vec<T>>;
    
    /// Check if an item exists
    fn exists(&self, id: &str) -> QmsResult<bool>;
    
    /// Count total items
    fn count(&self) -> QmsResult<usize>;
}

/// Generic storage writer interface - Interface Segregation Principle
/// Focused solely on writing operations
pub trait StorageWriter<T> {
    /// Save a single item
    fn save(&self, item: &T) -> QmsResult<()>;
    
    /// Save multiple items in batch
    fn save_batch(&self, items: &[T]) -> QmsResult<()>;
    
    /// Delete an item by ID
    fn delete(&self, id: &str) -> QmsResult<()>;
    
    /// Delete multiple items by IDs
    fn delete_batch(&self, ids: &[String]) -> QmsResult<()>;
}

/// Generic storage searcher interface - Interface Segregation Principle
/// Focused solely on search operations
pub trait StorageSearcher<T, C> {
    /// Search items based on criteria
    fn search(&self, criteria: &C) -> QmsResult<Vec<T>>;
    
    /// Search with pagination
    fn search_paginated(&self, criteria: &C, offset: usize, limit: usize) -> QmsResult<Vec<T>>;
    
    /// Count search results
    fn count_search_results(&self, criteria: &C) -> QmsResult<usize>;
}

/// Generic storage indexer interface - Interface Segregation Principle
/// Focused solely on indexing operations
pub trait StorageIndexer {
    /// Rebuild index
    fn rebuild_index(&self) -> QmsResult<()>;
    
    /// Verify index integrity
    fn verify_index(&self) -> QmsResult<bool>;
    
    /// Get index statistics
    fn get_index_stats(&self) -> QmsResult<IndexStats>;
}

/// Generic backup manager interface - Interface Segregation Principle
/// Focused solely on backup operations
pub trait BackupManager {
    /// Create a backup
    fn create_backup(&self) -> QmsResult<String>;
    
    /// List available backups
    fn list_backups(&self) -> QmsResult<Vec<BackupInfo>>;
    
    /// Restore from backup
    fn restore_backup(&self, backup_id: &str) -> QmsResult<()>;
    
    /// Verify backup integrity
    fn verify_backup(&self, backup_id: &str) -> QmsResult<bool>;
    
    /// Delete a backup
    fn delete_backup(&self, backup_id: &str) -> QmsResult<()>;
}

/// Storage transaction interface - Interface Segregation Principle
/// Focused solely on transaction management
pub trait TransactionManager {
    type Transaction;
    
    /// Begin a new transaction
    fn begin_transaction(&self) -> QmsResult<Self::Transaction>;
    
    /// Commit a transaction
    fn commit_transaction(&self, transaction: Self::Transaction) -> QmsResult<()>;
    
    /// Rollback a transaction
    fn rollback_transaction(&self, transaction: Self::Transaction) -> QmsResult<()>;
}

/// Index statistics structure
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub total_entries: usize,
    pub index_size_bytes: u64,
    pub last_updated: String,
    pub is_valid: bool,
}

/// Backup information structure
#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub id: String,
    pub created_at: String,
    pub size_bytes: u64,
    pub checksum: String,
    pub description: Option<String>,
}

/// Composite storage interface combining all operations
/// This violates Interface Segregation Principle - use specific interfaces instead
#[deprecated(note = "Use specific storage interfaces instead for better SOLID compliance")]
pub trait CompositeStorage<T, C>: 
    StorageReader<T> + 
    StorageWriter<T> + 
    StorageSearcher<T, C> + 
    StorageIndexer + 
    BackupManager 
{
    // This trait intentionally left empty to demonstrate ISP violation
}

/// Storage factory interface - Abstract Factory Pattern
/// Follows Dependency Inversion Principle
pub trait StorageFactory<T, C> {
    /// Create a reader instance
    fn create_reader(&self, path: &Path) -> QmsResult<Box<dyn StorageReader<T>>>;
    
    /// Create a writer instance
    fn create_writer(&self, path: &Path) -> QmsResult<Box<dyn StorageWriter<T>>>;
    
    /// Create a searcher instance
    fn create_searcher(&self, path: &Path) -> QmsResult<Box<dyn StorageSearcher<T, C>>>;
    
    /// Create an indexer instance
    fn create_indexer(&self, path: &Path) -> QmsResult<Box<dyn StorageIndexer>>;
    
    /// Create a backup manager instance
    fn create_backup_manager(&self, path: &Path) -> QmsResult<Box<dyn BackupManager>>;
}

/// Storage configuration for dependency injection
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub storage_type: StorageType,
    pub connection_string: Option<String>,
    pub max_connections: Option<usize>,
    pub timeout_seconds: Option<u64>,
    pub enable_compression: bool,
    pub enable_encryption: bool,
}

/// Storage type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum StorageType {
    FileSystem,
    Database,
    Memory,
    Cloud,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            storage_type: StorageType::FileSystem,
            connection_string: None,
            max_connections: Some(10),
            timeout_seconds: Some(30),
            enable_compression: false,
            enable_encryption: false,
        }
    }
}

/// Storage metrics for monitoring and observability
#[derive(Debug, Clone)]
pub struct StorageMetrics {
    pub read_operations: u64,
    pub write_operations: u64,
    pub search_operations: u64,
    pub average_read_time_ms: f64,
    pub average_write_time_ms: f64,
    pub average_search_time_ms: f64,
    pub error_count: u64,
    pub last_error: Option<String>,
}

/// Storage metrics collector interface
pub trait MetricsCollector {
    /// Record a read operation
    fn record_read(&self, duration_ms: f64, success: bool);
    
    /// Record a write operation
    fn record_write(&self, duration_ms: f64, success: bool);
    
    /// Record a search operation
    fn record_search(&self, duration_ms: f64, success: bool);
    
    /// Get current metrics
    fn get_metrics(&self) -> StorageMetrics;
    
    /// Reset metrics
    fn reset_metrics(&self);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_storage_config_default() {
        let config = StorageConfig::default();
        assert_eq!(config.storage_type, StorageType::FileSystem);
        assert_eq!(config.max_connections, Some(10));
        assert_eq!(config.timeout_seconds, Some(30));
        assert!(!config.enable_compression);
        assert!(!config.enable_encryption);
    }
    
    #[test]
    fn test_index_stats() {
        let stats = IndexStats {
            total_entries: 100,
            index_size_bytes: 1024,
            last_updated: "2024-01-01".to_string(),
            is_valid: true,
        };
        
        assert_eq!(stats.total_entries, 100);
        assert_eq!(stats.index_size_bytes, 1024);
        assert!(stats.is_valid);
    }
    
    #[test]
    fn test_backup_info() {
        let backup = BackupInfo {
            id: "backup-001".to_string(),
            created_at: "2024-01-01".to_string(),
            size_bytes: 2048,
            checksum: "abc123".to_string(),
            description: Some("Test backup".to_string()),
        };
        
        assert_eq!(backup.id, "backup-001");
        assert_eq!(backup.size_bytes, 2048);
        assert_eq!(backup.checksum, "abc123");
    }
}
