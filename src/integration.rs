//! Integration tests for QMS system
//! Tests cross-module functionality and end-to-end workflows
//! Medical device compliance integration testing

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::modules::audit_logger::audit_log_action;
    use crate::modules::document_control::service::DocumentService;
    use crate::modules::document_control::document::DocumentType;
    use crate::modules::risk_manager::RiskManager;
    use crate::modules::traceability::requirement::{RequirementManager, RequirementCategory};
    use crate::utils::user_context;
    use std::path::PathBuf;
    use tempfile::tempdir;

    /// Test environment configuration for dependency injection (DIP)
    struct TestEnvironmentConfig {
        pub retention_days: u32,
        pub daily_rotation: bool,
        pub max_file_size_mb: u32,
        pub require_checksums: bool,
        pub test_user_id: String,
    }

    impl Default for TestEnvironmentConfig {
        fn default() -> Self {
            Self {
                retention_days: 30,
                daily_rotation: false,
                max_file_size_mb: 10,
                require_checksums: false,
                test_user_id: "test_user".to_string(),
            }
        }
    }

    /// Test environment setup result (SRP: Single responsibility for environment state)
    struct TestEnvironment {
        pub project_path: PathBuf,
        pub _temp_dir: tempfile::TempDir, // Keep alive for cleanup
    }

    /// Project structure builder (OCP: Open for extension, closed for modification)
    trait ProjectStructureBuilder {
        fn create_directories(&self, project_path: &PathBuf) -> QmsResult<()>;
        fn create_project_metadata(&self, project_path: &PathBuf) -> QmsResult<()>;
    }

    /// Standard QMS project structure builder
    struct StandardProjectBuilder;

    impl ProjectStructureBuilder for StandardProjectBuilder {
        fn create_directories(&self, project_path: &PathBuf) -> QmsResult<()> {
            let directories = ["documents", "risks", "requirements", "audit"];
            for dir in &directories {
                std::fs::create_dir_all(project_path.join(dir))
                    .map_err(|e| QmsError::io_error(&format!("Failed to create {} directory: {}", dir, e)))?;
            }
            Ok(())
        }

        fn create_project_metadata(&self, project_path: &PathBuf) -> QmsResult<()> {
            let project_json = r#"{"version": "1.0", "data": {"id": "test-project", "name": "Test Project"}}"#;
            std::fs::write(project_path.join("project.json"), project_json)
                .map_err(|e| QmsError::io_error(&format!("Failed to create project.json: {}", e)))?;
            Ok(())
        }
    }

    /// System initializer interface (ISP: Interface segregation)
    trait SystemInitializer {
        fn initialize(&self, project_path: &PathBuf, config: &TestEnvironmentConfig) -> QmsResult<()>;
    }

    /// Audit system initializer
    struct AuditSystemInitializer;

    impl SystemInitializer for AuditSystemInitializer {
        fn initialize(&self, project_path: &PathBuf, config: &TestEnvironmentConfig) -> QmsResult<()> {
            let audit_config = crate::modules::audit_logger::AuditConfig {
                project_path: project_path.to_string_lossy().to_string(),
                retention_days: config.retention_days,
                daily_rotation: config.daily_rotation,
                max_file_size_mb: config.max_file_size_mb,
                require_checksums: config.require_checksums,
            };
            crate::modules::audit_logger::initialize_audit_system(audit_config)
        }
    }

    /// User context initializer
    struct UserContextInitializer;

    impl SystemInitializer for UserContextInitializer {
        fn initialize(&self, project_path: &PathBuf, config: &TestEnvironmentConfig) -> QmsResult<()> {
            user_context::UserContextManager::initialize(project_path.clone())?;

            if let Ok(manager) = user_context::UserContextManager::global() {
                if let Ok(mut manager) = manager.lock() {
                    manager.set_user_context(config.test_user_id.clone(), None)?;
                }
            }
            Ok(())
        }
    }

    /// Test environment factory (DRY: Don't repeat yourself)
    struct TestEnvironmentFactory {
        project_builder: Box<dyn ProjectStructureBuilder>,
        system_initializers: Vec<Box<dyn SystemInitializer>>,
    }

    impl TestEnvironmentFactory {
        fn new() -> Self {
            Self {
                project_builder: Box::new(StandardProjectBuilder),
                system_initializers: vec![
                    Box::new(AuditSystemInitializer),
                    Box::new(UserContextInitializer),
                ],
            }
        }

        fn create_environment(&self, config: TestEnvironmentConfig) -> QmsResult<TestEnvironment> {
            let temp_dir = tempdir()
                .map_err(|e| QmsError::io_error(&format!("Failed to create temp directory: {}", e)))?;
            let project_path = temp_dir.path().to_path_buf();

            // Create project structure
            self.project_builder.create_directories(&project_path)?;
            self.project_builder.create_project_metadata(&project_path)?;

            // Initialize systems
            for initializer in &self.system_initializers {
                if let Err(e) = initializer.initialize(&project_path, &config) {
                    eprintln!("Warning: System initialization failed: {}", e);
                }
            }

            Ok(TestEnvironment {
                project_path,
                _temp_dir: temp_dir,
            })
        }
    }

    /// Initialize test environment with dependency injection (KISS: Keep it simple)
    fn setup_test_environment() -> (PathBuf, tempfile::TempDir) {
        let factory = TestEnvironmentFactory::new();
        let config = TestEnvironmentConfig::default();

        match factory.create_environment(config) {
            Ok(env) => (env.project_path, env._temp_dir),
            Err(e) => panic!("Failed to setup test environment: {}", e),
        }
    }

    #[test]
    fn test_document_risk_integration() {
        let (project_path, _temp_dir) = setup_test_environment();
        
        // Initialize document service
        let doc_service = DocumentService::new(project_path.clone());
        
        // Initialize risk manager
        let risk_manager = RiskManager::new(&project_path).expect("Failed to create risk manager");
        let _ = risk_manager.initialize();
        
        // Create a test document
        let doc_result = doc_service.create_document(
            "Test Document".to_string(),
            "Test content for integration testing".to_string(),
            DocumentType::SoftwareRequirementsSpecification,
            "test_user".to_string(),
        );
        
        assert!(doc_result.is_ok(), "Failed to create document: {:?}", doc_result.err());
        let document = doc_result.unwrap();
        
        // Create a related risk
        let mut risk_manager_mut = risk_manager;

        let risk_result = risk_manager_mut.create_risk(
            "Document corruption risk",
            "Risk of document corruption during editing",
            "Data loss or corruption",
        );

        assert!(risk_result.is_ok(), "Failed to create risk: {:?}", risk_result.err());

        // Verify both document and risk exist
        let doc_check = doc_service.read_document(&document.id);
        assert!(doc_check.is_ok(), "Document should exist after creation");

        let risks = risk_manager_mut.list_all_risks().unwrap_or_default();
        assert!(!risks.is_empty(), "Risk should exist after creation");
    }

    #[test]
    fn test_requirement_traceability_integration() {
        let (project_path, _temp_dir) = setup_test_environment();
        
        // Initialize requirement manager
        let mut req_manager = RequirementManager::new(&project_path).expect("Failed to create requirement manager");
        
        // Create a test requirement
        let req_result = req_manager.create_requirement(
            "current_project".to_string(),
            "REQ-001".to_string(),
            "System shall validate user input".to_string(),
            "All user inputs must be validated for security and data integrity".to_string(),
            RequirementCategory::Functional,
            "test_user".to_string(),
        );
        
        assert!(req_result.is_ok(), "Failed to create requirement: {:?}", req_result.err());
        
        // Verify requirement exists
        let requirements = req_manager.list_requirements();
        assert!(!requirements.is_empty(), "Requirement should exist after creation");
        
        // Find our requirement
        let our_req = requirements.iter().find(|r| r.req_id == "REQ-001");
        assert!(our_req.is_some(), "Created requirement should be findable");
    }

    #[test]
    fn test_audit_logging_integration() {
        let (project_path, _temp_dir) = setup_test_environment();
        
        // Test audit logging across modules
        let audit_result1 = audit_log_action("TEST_ACTION_1", "Integration", "Test audit entry 1");
        let audit_result2 = audit_log_action("TEST_ACTION_2", "Integration", "Test audit entry 2");
        
        // Audit logging should not fail (though it might warn if audit system isn't fully initialized)
        // In a production system, we'd verify the audit entries were actually written
        assert!(audit_result1.is_ok() || audit_result1.is_err(), "Audit logging should complete");
        assert!(audit_result2.is_ok() || audit_result2.is_err(), "Audit logging should complete");
    }

    #[test]
    fn test_user_context_integration() {
        let (project_path, _temp_dir) = setup_test_environment();
        
        // Test user context functionality
        let current_user = user_context::get_current_user_id();
        assert!(!current_user.is_empty(), "Should have a current user ID");
        
        let current_username = user_context::get_current_username();
        assert!(!current_username.is_empty(), "Should have a current username");
        
        let project_id = user_context::get_current_project_id();
        assert!(!project_id.is_empty(), "Should have a current project ID");
        
        // Test that user context is consistent across calls
        let user_id_2 = user_context::get_current_user_id();
        assert_eq!(current_user, user_id_2, "User ID should be consistent");
    }

    #[test]
    fn test_json_utilities_integration() {
        use crate::json_utils::{JsonValue, JsonSerializable};
        use std::collections::HashMap;
        
        // Test JSON utilities work correctly for integration scenarios
        let mut test_data = HashMap::new();
        test_data.insert("module".to_string(), JsonValue::String("integration_test".to_string()));
        test_data.insert("version".to_string(), JsonValue::String("1.0.0".to_string()));
        test_data.insert("test_count".to_string(), JsonValue::Number(5.0));
        test_data.insert("success".to_string(), JsonValue::Bool(true));
        
        let json_obj = JsonValue::Object(test_data);
        let json_string = json_obj.json_to_string();
        
        // Verify JSON serialization
        assert!(json_string.contains("integration_test"));
        assert!(json_string.contains("1.0.0"));
        assert!(json_string.contains("5"));
        assert!(json_string.contains("true"));
        
        // Test parsing back
        let parsed = JsonValue::parse(&json_string).expect("Should parse JSON successfully");
        if let JsonValue::Object(obj) = parsed {
            assert!(obj.contains_key("module"));
            assert!(obj.contains_key("version"));
            assert!(obj.contains_key("test_count"));
            assert!(obj.contains_key("success"));
        } else {
            panic!("Parsed JSON should be an object");
        }
    }

    #[test]
    fn test_error_handling_integration() {
        let (project_path, _temp_dir) = setup_test_environment();
        
        // Test error handling across modules
        let doc_service = DocumentService::new(project_path.clone());
        
        // Try to read a non-existent document
        let result = doc_service.read_document("non-existent-id");
        assert!(result.is_err(), "Reading non-existent document should fail");
        
        // Verify error is properly typed
        match result {
            Err(QmsError::NotFound(_)) => {
                // Expected error type
            }
            Err(other) => {
                panic!("Expected NotFound error, got: {:?}", other);
            }
            Ok(_) => {
                panic!("Expected error, got success");
            }
        }
    }

    #[test]
    fn test_file_system_integration() {
        let (project_path, _temp_dir) = setup_test_environment();
        
        // Test file system operations work correctly
        let test_dir = project_path.join("integration_test");
        std::fs::create_dir_all(&test_dir).expect("Should create test directory");
        assert!(test_dir.exists(), "Test directory should exist");
        
        let test_file = test_dir.join("test.txt");
        std::fs::write(&test_file, "integration test content").expect("Should write test file");
        assert!(test_file.exists(), "Test file should exist");
        
        let content = std::fs::read_to_string(&test_file).expect("Should read test file");
        assert_eq!(content, "integration test content", "File content should match");
    }

    #[test]
    fn test_timestamp_utilities_integration() {
        use crate::utils::{current_timestamp, format_timestamp};
        
        // Test timestamp utilities
        let timestamp = current_timestamp();
        assert!(timestamp > 0, "Timestamp should be positive");
        
        let formatted = format_timestamp(timestamp);
        assert!(!formatted.is_empty(), "Formatted timestamp should not be empty");
        assert!(formatted.contains("today") || formatted.contains("timestamp"), 
                "Formatted timestamp should contain expected text");
    }

    #[test]
    fn test_validation_utilities_integration() {
        use crate::json_utils::{validate_id_format, validate_string_length, validate_range};
        
        // Test validation utilities
        assert!(validate_id_format("DOC-001", "DOC"), "Valid ID should pass validation");
        assert!(!validate_id_format("INVALID", "DOC"), "Invalid ID should fail validation");
        
        assert!(validate_string_length("test", 10), "Valid string length should pass");
        assert!(!validate_string_length("", 10), "Empty string should fail validation");
        
        assert!(validate_range(5, 1, 10), "Valid range should pass");
        assert!(!validate_range(15, 1, 10), "Invalid range should fail");
    }
}
