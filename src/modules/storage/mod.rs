/// SOLID Principles Enhancement: Storage Module
/// 
/// This module provides storage abstractions and implementations that follow SOLID principles.

pub mod storage_interfaces;
pub mod file_storage;

// Re-export key interfaces
pub use storage_interfaces::{
    StorageReader, StorageWriter, StorageSearcher, StorageIndexer, BackupManager,
    TransactionManager, StorageFactory, StorageConfig, StorageType, StorageMetrics,
    MetricsCollector, IndexStats, BackupInfo
};

pub use file_storage::{
    FileStorageReader, FileStorageWriter, FileStorageSearcher, FileStorageIndexer,
    FileBackupManager, FileStorageFactory
};
