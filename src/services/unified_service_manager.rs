//! Unified Service Manager
//! 
//! Central coordinator for all business logic services across CLI, TUI, and web interfaces.
//! Implements dependency injection and service lifecycle management following SOLID principles.

use crate::prelude::*;
use crate::services::{
    UnifiedDocumentService, UnifiedRiskService, UnifiedUserService, UnifiedProjectService,
    UnifiedNavigationService, UnifiedDataAccessService, UnifiedValidationService,
};
use crate::services::document_service::DocumentServiceInterface;
use crate::services::risk_service::RiskServiceInterface;
use crate::services::user_service::UserServiceInterface;
use crate::services::project_service::ProjectServiceInterface;
use crate::services::navigation_service::NavigationServiceInterface;
use crate::services::data_access_service::DataAccessServiceInterface;
use crate::services::validation_service::ValidationServiceInterface;
use crate::modules::user_manager::SessionType;
use crate::interfaces::state::{StateManager, StateSnapshot, FileStateManager, SessionStateManager, MemoryStateManager};
use crate::interfaces::{InterfaceContext, InterfaceType};
use crate::modules::storage::{StorageConfig, StorageType};
use std::path::PathBuf;
use std::sync::Arc;

/// Service Manager Interface
///
/// Provides dependency injection and service coordination for all QMS operations.
/// Follows the Facade pattern to simplify service access for interface layers.
pub trait ServiceManagerInterface: Send + Sync {
    /// Get document service
    fn document_service(&self) -> Arc<dyn DocumentServiceInterface>;

    /// Get risk service
    fn risk_service(&self) -> Arc<dyn RiskServiceInterface>;

    /// Get user service
    fn user_service(&self) -> Arc<dyn UserServiceInterface>;

    /// Get project service
    fn project_service(&self) -> Arc<dyn ProjectServiceInterface>;

    /// Get state manager
    fn state_manager(&self) -> Arc<dyn StateManager>;

    /// Get navigation service
    fn navigation_service(&self) -> Arc<dyn NavigationServiceInterface>;

    /// Get data access service
    fn data_access_service(&self) -> Arc<UnifiedDataAccessService>;

    /// Get validation service
    fn validation_service(&self) -> Arc<dyn ValidationServiceInterface>;

    /// Initialize services for a specific project
    fn initialize_for_project(&mut self, project_path: PathBuf) -> QmsResult<()>;

    /// Get current project path
    fn current_project_path(&self) -> Option<&PathBuf>;

    /// Validate service dependencies
    fn validate_dependencies(&self) -> QmsResult<()>;

    /// Create interface context for the given interface type
    fn create_interface_context(&self, interface_type: InterfaceType) -> InterfaceContext;
}

/// Service Configuration
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub project_path: Option<PathBuf>,
    pub enable_audit_logging: bool,
    pub enable_caching: bool,
    pub max_cache_size: usize,
    pub session_timeout: u64,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            project_path: None,
            enable_audit_logging: true,
            enable_caching: false,
            max_cache_size: 1000,
            session_timeout: 3600, // 1 hour
        }
    }
}

/// Unified Service Manager Implementation
///
/// Central coordinator that manages all business logic services and provides
/// a unified interface for CLI, TUI, and web interfaces to access services.
pub struct UnifiedServiceManager {
    document_service: Arc<dyn DocumentServiceInterface>,
    risk_service: Arc<dyn RiskServiceInterface>,
    user_service: Arc<dyn UserServiceInterface>,
    project_service: Arc<dyn ProjectServiceInterface>,
    state_manager: Arc<dyn StateManager>,
    navigation_service: Arc<dyn NavigationServiceInterface>,
    data_access_service: Arc<UnifiedDataAccessService>,
    validation_service: Arc<dyn ValidationServiceInterface>,
    config: ServiceConfig,
    current_project_path: Option<PathBuf>,
}

impl UnifiedServiceManager {
    /// Create new service manager with default configuration
    pub fn new() -> QmsResult<Self> {
        Self::with_config(ServiceConfig::default())
    }

    /// Create service manager with custom configuration
    pub fn with_config(config: ServiceConfig) -> QmsResult<Self> {
        let project_path = config.project_path.clone().unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        });

        // Initialize services with dependency injection
        let document_service = Arc::new(UnifiedDocumentService::new(project_path.clone()));
        let risk_service = Arc::new(UnifiedRiskService::new(project_path.clone())?);
        let user_service = Arc::new(UnifiedUserService::new(project_path.clone())?);
        let project_service = Arc::new(UnifiedProjectService::new()?);

        // Initialize state manager based on configuration
        let state_manager: Arc<dyn StateManager> = Arc::new(FileStateManager::new(Some(project_path.clone())));

        // Initialize data access service
        let storage_config = StorageConfig {
            storage_type: StorageType::FileSystem,
            connection_string: None,
            max_connections: Some(10),
            timeout_seconds: Some(30),
            enable_compression: false,
            enable_encryption: false,
        };
        let data_access_service = Arc::new(UnifiedDataAccessService::new(project_path.clone(), storage_config)?);

        // Initialize validation service
        let validation_service = Arc::new(UnifiedValidationService::new());

        // Create a temporary service manager reference for navigation service
        let temp_manager = Arc::new(TempServiceManager {
            document_service: document_service.clone(),
            risk_service: risk_service.clone(),
            user_service: user_service.clone(),
            project_service: project_service.clone(),
            state_manager: state_manager.clone(),
            data_access_service: data_access_service.clone(),
            validation_service: validation_service.clone(),
            current_project_path: Some(project_path.clone()),
        });

        let navigation_service = Arc::new(UnifiedNavigationService::new(temp_manager)?);

        Ok(Self {
            document_service,
            risk_service,
            user_service,
            project_service,
            state_manager,
            navigation_service,
            data_access_service,
            validation_service,
            config,
            current_project_path: Some(project_path),
        })
    }

    /// Create service manager for specific project
    pub fn for_project(project_path: PathBuf) -> QmsResult<Self> {
        let mut config = ServiceConfig::default();
        config.project_path = Some(project_path);
        Self::with_config(config)
    }

    /// Create service manager with dependency injection (for testing)
    pub fn with_services(
        document_service: Arc<dyn DocumentServiceInterface>,
        risk_service: Arc<dyn RiskServiceInterface>,
        user_service: Arc<dyn UserServiceInterface>,
        project_service: Arc<dyn ProjectServiceInterface>,
        state_manager: Arc<dyn StateManager>,
        navigation_service: Arc<dyn NavigationServiceInterface>,
        data_access_service: Arc<UnifiedDataAccessService>,
        validation_service: Arc<dyn ValidationServiceInterface>,
        config: ServiceConfig,
    ) -> Self {
        Self {
            document_service,
            risk_service,
            user_service,
            project_service,
            state_manager,
            navigation_service,
            data_access_service,
            validation_service,
            current_project_path: config.project_path.clone(),
            config,
        }
    }

    /// Authenticate user and return service manager with user context
    pub fn authenticate(&self, username: &str, password: &str, session_type: SessionType) -> QmsResult<AuthenticatedServiceManager> {
        let session = self.user_service.login(username, password, session_type)?;
        
        Ok(AuthenticatedServiceManager {
            service_manager: self,
            user_session: session,
        })
    }

    /// Get service health status
    pub fn health_check(&self) -> ServiceHealthStatus {
        let mut status = ServiceHealthStatus {
            overall_status: HealthStatus::Healthy,
            services: std::collections::HashMap::new(),
            last_check: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Check each service (simplified implementation)
        status.services.insert("document".to_string(), HealthStatus::Healthy);
        status.services.insert("risk".to_string(), HealthStatus::Healthy);
        status.services.insert("user".to_string(), HealthStatus::Healthy);
        status.services.insert("project".to_string(), HealthStatus::Healthy);

        status
    }

    /// Shutdown services gracefully
    pub fn shutdown(&self) -> QmsResult<()> {
        // Perform cleanup operations
        // - Close database connections
        // - Save pending changes
        // - Clear caches
        // - Log shutdown event
        
        crate::modules::audit_logger::audit_log_action("SERVICE_MANAGER_SHUTDOWN", "System", "graceful")?;
        
        Ok(())
    }
}

impl ServiceManagerInterface for UnifiedServiceManager {
    fn document_service(&self) -> Arc<dyn DocumentServiceInterface> {
        self.document_service.clone()
    }

    fn risk_service(&self) -> Arc<dyn RiskServiceInterface> {
        self.risk_service.clone()
    }

    fn user_service(&self) -> Arc<dyn UserServiceInterface> {
        self.user_service.clone()
    }

    fn project_service(&self) -> Arc<dyn ProjectServiceInterface> {
        self.project_service.clone()
    }

    fn state_manager(&self) -> Arc<dyn StateManager> {
        self.state_manager.clone()
    }

    fn navigation_service(&self) -> Arc<dyn NavigationServiceInterface> {
        self.navigation_service.clone()
    }

    fn data_access_service(&self) -> Arc<UnifiedDataAccessService> {
        self.data_access_service.clone()
    }

    fn validation_service(&self) -> Arc<dyn ValidationServiceInterface> {
        self.validation_service.clone()
    }

    fn initialize_for_project(&mut self, project_path: PathBuf) -> QmsResult<()> {
        // Reinitialize services for new project
        self.document_service = Arc::new(UnifiedDocumentService::new(project_path.clone()));
        self.risk_service = Arc::new(UnifiedRiskService::new(project_path.clone())?);
        self.user_service = Arc::new(UnifiedUserService::new(project_path.clone())?);
        self.state_manager = Arc::new(FileStateManager::new(Some(project_path.clone())));

        // Reinitialize data access service
        let storage_config = StorageConfig {
            storage_type: StorageType::FileSystem,
            connection_string: None,
            max_connections: Some(10),
            timeout_seconds: Some(30),
            enable_compression: false,
            enable_encryption: false,
        };
        self.data_access_service = Arc::new(UnifiedDataAccessService::new(project_path.clone(), storage_config)?);

        // Reinitialize validation service
        self.validation_service = Arc::new(UnifiedValidationService::new());

        // Reinitialize navigation service with new services
        let temp_manager = Arc::new(TempServiceManager {
            document_service: self.document_service.clone(),
            risk_service: self.risk_service.clone(),
            user_service: self.user_service.clone(),
            project_service: self.project_service.clone(),
            state_manager: self.state_manager.clone(),
            data_access_service: self.data_access_service.clone(),
            validation_service: self.validation_service.clone(),
            current_project_path: Some(project_path.clone()),
        });
        self.navigation_service = Arc::new(UnifiedNavigationService::new(temp_manager)?);

        self.current_project_path = Some(project_path);

        // Validate the new project setup
        self.validate_dependencies()?;

        Ok(())
    }

    fn current_project_path(&self) -> Option<&PathBuf> {
        self.current_project_path.as_ref()
    }

    fn validate_dependencies(&self) -> QmsResult<()> {
        // Validate that all required services are available and properly configured
        if self.current_project_path.is_none() {
            return Err(QmsError::validation_error("No project path configured"));
        }

        let project_path = self.current_project_path.as_ref().unwrap();

        // Check if project directory exists
        if !project_path.exists() {
            return Err(QmsError::not_found(&format!(
                "Project directory does not exist: {}",
                project_path.display()
            )));
        }

        // Validate project structure
        let validation = self.project_service.validate_project_directory(project_path)?;
        if !validation.is_valid {
            return Err(QmsError::validation_error(&format!(
                "Invalid project structure: {:?}",
                validation.errors
            )));
        }

        Ok(())
    }

    fn create_interface_context(&self, interface_type: InterfaceType) -> InterfaceContext {
        let mut context = InterfaceContext::new(interface_type);
        context.project_path = self.current_project_path.clone();
        context
    }
}

/// Authenticated Service Manager
/// 
/// Wrapper around ServiceManager that includes user authentication context.
/// Provides user-aware service operations with automatic permission checking.
pub struct AuthenticatedServiceManager<'a> {
    service_manager: &'a UnifiedServiceManager,
    user_session: crate::modules::user_manager::UserSession,
}

impl<'a> AuthenticatedServiceManager<'a> {
    /// Get current user session
    pub fn user_session(&self) -> &crate::modules::user_manager::UserSession {
        &self.user_session
    }

    /// Get current username
    pub fn username(&self) -> &str {
        &self.user_session.username
    }

    /// Check if user has specific permission
    pub fn has_permission(&self, permission: &str) -> QmsResult<bool> {
        self.service_manager.user_service.has_permission(&self.user_session.username, permission)
    }

    /// Logout and invalidate session
    pub fn logout(self) -> QmsResult<()> {
        self.service_manager.user_service.logout(&self.user_session.session_id)
    }
}

impl<'a> ServiceManagerInterface for AuthenticatedServiceManager<'a> {
    fn document_service(&self) -> Arc<dyn DocumentServiceInterface> {
        self.service_manager.document_service()
    }

    fn risk_service(&self) -> Arc<dyn RiskServiceInterface> {
        self.service_manager.risk_service()
    }

    fn user_service(&self) -> Arc<dyn UserServiceInterface> {
        self.service_manager.user_service()
    }

    fn project_service(&self) -> Arc<dyn ProjectServiceInterface> {
        self.service_manager.project_service()
    }

    fn state_manager(&self) -> Arc<dyn StateManager> {
        self.service_manager.state_manager()
    }

    fn navigation_service(&self) -> Arc<dyn NavigationServiceInterface> {
        self.service_manager.navigation_service()
    }

    fn data_access_service(&self) -> Arc<UnifiedDataAccessService> {
        self.service_manager.data_access_service()
    }

    fn validation_service(&self) -> Arc<dyn ValidationServiceInterface> {
        self.service_manager.validation_service()
    }

    fn initialize_for_project(&mut self, _project_path: PathBuf) -> QmsResult<()> {
        Err(QmsError::InvalidOperation(
            "Cannot reinitialize project on authenticated service manager".to_string()
        ))
    }

    fn current_project_path(&self) -> Option<&PathBuf> {
        self.service_manager.current_project_path()
    }

    fn validate_dependencies(&self) -> QmsResult<()> {
        self.service_manager.validate_dependencies()
    }

    fn create_interface_context(&self, interface_type: InterfaceType) -> InterfaceContext {
        let mut context = self.service_manager.create_interface_context(interface_type);
        context.user_session = Some(self.user_session.clone());
        context
    }
}

/// Service health status
#[derive(Debug, Clone)]
pub struct ServiceHealthStatus {
    pub overall_status: HealthStatus,
    pub services: std::collections::HashMap<String, HealthStatus>,
    pub last_check: u64,
}

/// Health status enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Temporary Service Manager for circular dependency resolution
/// Used during initialization to create navigation service
struct TempServiceManager {
    document_service: Arc<dyn DocumentServiceInterface>,
    risk_service: Arc<dyn RiskServiceInterface>,
    user_service: Arc<dyn UserServiceInterface>,
    project_service: Arc<dyn ProjectServiceInterface>,
    state_manager: Arc<dyn StateManager>,
    data_access_service: Arc<UnifiedDataAccessService>,
    validation_service: Arc<dyn ValidationServiceInterface>,
    current_project_path: Option<PathBuf>,
}

impl ServiceManagerInterface for TempServiceManager {
    fn document_service(&self) -> Arc<dyn DocumentServiceInterface> {
        self.document_service.clone()
    }

    fn risk_service(&self) -> Arc<dyn RiskServiceInterface> {
        self.risk_service.clone()
    }

    fn user_service(&self) -> Arc<dyn UserServiceInterface> {
        self.user_service.clone()
    }

    fn project_service(&self) -> Arc<dyn ProjectServiceInterface> {
        self.project_service.clone()
    }

    fn state_manager(&self) -> Arc<dyn StateManager> {
        self.state_manager.clone()
    }

    fn navigation_service(&self) -> Arc<dyn NavigationServiceInterface> {
        // This should not be called during initialization
        panic!("Navigation service not available during initialization")
    }

    fn data_access_service(&self) -> Arc<UnifiedDataAccessService> {
        self.data_access_service.clone()
    }

    fn validation_service(&self) -> Arc<dyn ValidationServiceInterface> {
        self.validation_service.clone()
    }

    fn initialize_for_project(&mut self, _project_path: PathBuf) -> QmsResult<()> {
        Err(QmsError::InvalidOperation("Cannot reinitialize temp service manager".to_string()))
    }

    fn current_project_path(&self) -> Option<&PathBuf> {
        self.current_project_path.as_ref()
    }

    fn validate_dependencies(&self) -> QmsResult<()> {
        Ok(())
    }

    fn create_interface_context(&self, interface_type: InterfaceType) -> InterfaceContext {
        let mut context = InterfaceContext::new(interface_type);
        context.project_path = self.current_project_path.clone();
        context
    }
}

/// Service Manager Factory
/// 
/// Factory for creating service managers with different configurations.
/// Implements the Factory pattern for service manager creation.
pub struct ServiceManagerFactory;

impl ServiceManagerFactory {
    /// Create service manager for CLI interface
    pub fn create_for_cli(project_path: Option<PathBuf>) -> QmsResult<UnifiedServiceManager> {
        let mut config = ServiceConfig::default();
        config.project_path = project_path;
        config.enable_caching = false; // CLI doesn't need caching

        let mut manager = UnifiedServiceManager::with_config(config)?;

        // Use file-based state manager for CLI
        if let Some(ref path) = manager.current_project_path {
            manager.state_manager = Arc::new(FileStateManager::new(Some(path.clone())));
        }

        Ok(manager)
    }

    /// Create service manager for web interface
    pub fn create_for_web(project_path: Option<PathBuf>) -> QmsResult<UnifiedServiceManager> {
        let mut config = ServiceConfig::default();
        config.project_path = project_path;
        config.enable_caching = true; // Web benefits from caching
        config.session_timeout = 1800; // 30 minutes for web sessions

        let mut manager = UnifiedServiceManager::with_config(config)?;

        // Use session-based state manager for web
        manager.state_manager = Arc::new(SessionStateManager::new());

        Ok(manager)
    }

    /// Create service manager for TUI interface
    pub fn create_for_tui(project_path: Option<PathBuf>) -> QmsResult<UnifiedServiceManager> {
        let mut config = ServiceConfig::default();
        config.project_path = project_path;
        config.enable_caching = false; // TUI doesn't need caching
        config.session_timeout = 7200; // 2 hours for TUI sessions

        let mut manager = UnifiedServiceManager::with_config(config)?;

        // Use file-based state manager for TUI (similar to CLI)
        if let Some(ref path) = manager.current_project_path {
            manager.state_manager = Arc::new(FileStateManager::new(Some(path.clone())));
        }

        Ok(manager)
    }

    /// Create service manager for testing
    pub fn create_for_testing() -> QmsResult<UnifiedServiceManager> {
        let mut config = ServiceConfig::default();
        config.enable_audit_logging = false; // Disable audit logging in tests
        config.project_path = Some(std::env::temp_dir().join("qms_test"));

        let mut manager = UnifiedServiceManager::with_config(config)?;

        // Use memory-based state manager for testing
        manager.state_manager = Arc::new(MemoryStateManager::new());

        Ok(manager)
    }
}
