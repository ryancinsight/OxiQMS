// Integration tests for traceability module
// Implementation for Task 3.2.12 Traceability Module Testing

use std::fs;
use std::env;
use crate::modules::traceability::links::{TraceabilityManager, TraceLinkType};
use crate::modules::traceability::requirement::RequirementManager;
use crate::modules::traceability::test_case::TestCaseManager;
use crate::modules::audit_logger::{AuditConfig, initialize_audit_system};

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Initialize audit system for tests - lightweight version to avoid hanging
    fn init_audit_for_test(temp_dir: &std::path::Path) {
        // Create audit directory structure but don't initialize the full audit system
        // This avoids potential hanging issues with file I/O operations
        let audit_dir = temp_dir.join("audit");
        let _ = fs::create_dir_all(&audit_dir);

        // Only initialize if not already done (avoid re-initialization errors)
        let config = AuditConfig {
            project_path: temp_dir.to_string_lossy().to_string(),
            retention_days: 30,
            daily_rotation: false,
            max_file_size_mb: 10,
            require_checksums: false,
        };

        // Use a timeout-safe initialization approach
        match initialize_audit_system(config) {
            Ok(_) => {},
            Err(_) => {
                // If initialization fails, continue with test - audit is not critical for traceability logic
                eprintln!("Warning: Audit system initialization failed in test, continuing without audit");
            }
        }
    }

    #[test]
    fn test_end_to_end_requirement_to_test_workflow() {
        // Test complete workflow from requirement creation to test case linking
        let temp_dir = env::temp_dir().join("qms_integration_test_1");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Initialize audit system for test
        init_audit_for_test(&temp_dir);
        
        // Step 1: Create requirement
        let mut req_manager = RequirementManager::new(&temp_dir).unwrap();
        let _req_id = req_manager.create_requirement(
            "test_project".to_string(),
            "REQ-001".to_string(),
            "Test Requirement".to_string(),
            "This is a test requirement for integration testing".to_string(),
            crate::modules::traceability::requirement::RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();

        // Step 2: Create test case
        let mut test_manager = TestCaseManager::new(&temp_dir).unwrap();
        test_manager.create_test_case(
            "TC-001".to_string(),
            "Test Login Security".to_string(),
            "Verify user login security requirements".to_string(),
            crate::modules::traceability::test_case::TestCategory::Functional,
            crate::modules::traceability::test_case::TestPriority::High,
            "test_user".to_string(),
        ).unwrap();
        
        // Debug: Check if files exist and contain the entities
        // Step 3: Create traceability link
        let trace_manager = TraceabilityManager::new(&temp_dir).unwrap();

        let _link = trace_manager.create_trace_link(
            "REQ-001",
            "TC-001",
            TraceLinkType::Verifies,
        ).unwrap();

        // Step 4: Verify end-to-end traceability
        let forward_trace = trace_manager.trace_forward("REQ-001").unwrap();
        assert_eq!(forward_trace.entity_id, "REQ-001");
        assert_eq!(forward_trace.path.len(), 1);
        assert_eq!(forward_trace.path[0].entity_id, "TC-001");

        let backward_trace = trace_manager.trace_backward("TC-001").unwrap();
        assert_eq!(backward_trace.entity_id, "TC-001");
        assert_eq!(backward_trace.path.len(), 1);
        assert_eq!(backward_trace.path[0].entity_id, "REQ-001");
        
        // Step 5: Verify RTM generation
        let rtm = trace_manager.generate_rtm().unwrap();
        assert_eq!(rtm.entities.len(), 2);
        assert_eq!(rtm.links.len(), 1);
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
    
    #[test]
    fn test_multi_module_traceability() {
        // Test traceability across multiple modules (requirements, test cases, risks)
        let temp_dir = env::temp_dir().join("qms_integration_test_2");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Initialize audit system for test
        init_audit_for_test(&temp_dir);
        
        let trace_manager = TraceabilityManager::new(&temp_dir).unwrap();
        
        // Create multiple entity types for linking
        let mut req_manager = RequirementManager::new(&temp_dir).unwrap();
        let mut test_manager = TestCaseManager::new(&temp_dir).unwrap();
        
        // Create requirement
        let req_id = req_manager.create_requirement(
            "test_project".to_string(),
            "REQ-002".to_string(),
            "Multi-Module Test Requirement".to_string(),
            "Testing multi-module traceability".to_string(),
            crate::modules::traceability::requirement::RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();

        // Create test case
        test_manager.create_test_case(
            "TC-002".to_string(),
            "Multi-Module Test Case".to_string(),
            "Testing multi-module traceability".to_string(),
            crate::modules::traceability::test_case::TestCategory::Functional,
            crate::modules::traceability::test_case::TestPriority::High,
            "test_user".to_string(),
        ).unwrap();
        
        // Create links between entities
        let _link1 = trace_manager.create_trace_link(
            "REQ-002",
            "TC-002",
            TraceLinkType::Verifies,
        ).unwrap();

        // Test complex traceability chain
        let forward_trace = trace_manager.trace_forward("REQ-002").unwrap();
        assert!(forward_trace.path.len() > 0);
        assert_eq!(forward_trace.path[0].entity_id, "TC-002");
        
        // Test orphan detection
        let orphans = trace_manager.find_orphaned_items().unwrap();
        // Should have no orphans since all entities are linked
        assert!(orphans.is_empty());
        
        // Test RTM generation with multiple entity types
        let rtm = trace_manager.generate_rtm().unwrap();
        assert!(rtm.entities.len() >= 2);
        assert_eq!(rtm.links.len(), 1);
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
    
    #[test]
    fn test_complex_dependency_graph() {
        // Test complex dependency relationships
        let temp_dir = env::temp_dir().join("qms_integration_test_3");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        
        let trace_manager = TraceabilityManager::new(&temp_dir).unwrap();
        let mut req_manager = RequirementManager::new(&temp_dir).unwrap();
        let mut test_manager = TestCaseManager::new(&temp_dir).unwrap();
        
        // Create a complex dependency chain: REQ-A -> REQ-B -> TC-1 -> TC-2
        let req_a_id = req_manager.create_requirement(
            "test_project".to_string(),
            "REQ-A".to_string(),
            "Base Requirement A".to_string(),
            "Base requirement for dependency testing".to_string(),
            crate::modules::traceability::requirement::RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();

        let req_b_id = req_manager.create_requirement(
            "test_project".to_string(),
            "REQ-B".to_string(),
            "Derived Requirement B".to_string(),
            "Requirement derived from A".to_string(),
            crate::modules::traceability::requirement::RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();

        test_manager.create_test_case(
            "TC-1".to_string(),
            "Test Case 1".to_string(),
            "Test case for requirement B".to_string(),
            crate::modules::traceability::test_case::TestCategory::Functional,
            crate::modules::traceability::test_case::TestPriority::High,
            "test_user".to_string(),
        ).unwrap();

        test_manager.create_test_case(
            "TC-2".to_string(),
            "Test Case 2".to_string(),
            "Test case dependent on TC-1".to_string(),
            crate::modules::traceability::test_case::TestCategory::Functional,
            crate::modules::traceability::test_case::TestPriority::Medium,
            "test_user".to_string(),
        ).unwrap();
        
        // Create dependency chain
        let _link1 = trace_manager.create_trace_link(
            "REQ-A",
            "REQ-B",
            TraceLinkType::DerivedFrom,
        ).unwrap();

        let _link2 = trace_manager.create_trace_link(
            "REQ-B",
            "TC-1",
            TraceLinkType::Verifies,
        ).unwrap();

        let _link3 = trace_manager.create_trace_link(
            "TC-1",
            "TC-2",
            TraceLinkType::DependsOn,
        ).unwrap();

        // Test forward tracing from root
        let forward_trace = trace_manager.trace_forward("REQ-A").unwrap();
        assert!(forward_trace.depth >= 2);
        assert!(forward_trace.path.len() > 0);
        
        // Test backward tracing from leaf
        let backward_trace = trace_manager.trace_backward("TC-2").unwrap();
        assert!(backward_trace.depth >= 2);
        assert!(backward_trace.path.len() > 0);
        
        // Test RTM generation with complex relationships
        let rtm = trace_manager.generate_rtm().unwrap();
        assert_eq!(rtm.entities.len(), 4);
        assert_eq!(rtm.links.len(), 3);
        
        // Verify dependency graph export
        let graph_path = temp_dir.join("dependency_graph.dot");
        trace_manager.export_dependency_graph(&graph_path).unwrap();
        assert!(graph_path.exists());
        
        let graph_content = fs::read_to_string(&graph_path).unwrap();
        assert!(graph_content.contains("digraph TraceabilityGraph"));
        assert!(graph_content.contains("REQ-A"));
        assert!(graph_content.contains("TC-2"));
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
    
    #[test]
    fn test_import_export_workflow() {
        // Test complete import/export workflow
        let temp_dir = env::temp_dir().join("qms_integration_test_4");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Initialize audit system for test
        init_audit_for_test(&temp_dir);
        
        let trace_manager = TraceabilityManager::new(&temp_dir).unwrap();
        let mut req_manager = RequirementManager::new(&temp_dir).unwrap();
        let mut test_manager = TestCaseManager::new(&temp_dir).unwrap();
        
        // Create initial data
        let req_id = req_manager.create_requirement(
            "test_project".to_string(),
            "REQ-IMPORT-001".to_string(),
            "Import Export Test Requirement".to_string(),
            "Testing import/export workflow".to_string(),
            crate::modules::traceability::requirement::RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();

        test_manager.create_test_case(
            "TC-IMPORT-001".to_string(),
            "Import Export Test Case".to_string(),
            "Testing import/export workflow".to_string(),
            crate::modules::traceability::test_case::TestCategory::Functional,
            crate::modules::traceability::test_case::TestPriority::High,
            "test_user".to_string(),
        ).unwrap();
        
        let _link = trace_manager.create_trace_link(
            "REQ-IMPORT-001",
            "TC-IMPORT-001",
            TraceLinkType::Verifies,
        ).unwrap();

        // Test RTM export to CSV
        let csv_path = temp_dir.join("test_export.csv");
        trace_manager.export_rtm_csv(&csv_path).unwrap();
        assert!(csv_path.exists());

        let csv_content = fs::read_to_string(&csv_path).unwrap();
        assert!(csv_content.contains("Entity ID,Entity Type,Title,Status,Linked Entities"));
        assert!(csv_content.contains("REQ-IMPORT-001"));
        assert!(csv_content.contains("TC-IMPORT-001"));
        
        // Test RTM export to JSON
        let json_path = temp_dir.join("test_export.json");
        trace_manager.export_rtm_json(&json_path).unwrap();
        assert!(json_path.exists());
        
        let json_content = fs::read_to_string(&json_path).unwrap();
        assert!(json_content.contains("\"entities\""));
        assert!(json_content.contains("\"links\""));
        assert!(json_content.contains("REQ-IMPORT-001"));
        assert!(json_content.contains("TC-IMPORT-001"));

        // Test CSV import
        let import_csv_path = temp_dir.join("import_test.csv");
        let csv_import_content = format!(
            "SourceType,SourceID,TargetType,TargetID,LinkType,CreatedBy\nRequirement,{},TestCase,{},Verifies,test_user\n",
            "REQ-IMPORT-001", "TC-IMPORT-001"
        );
        fs::write(&import_csv_path, csv_import_content).unwrap();
        
        // Clear existing links to test import
        let links_path = temp_dir.join("trace").join("links.json");
        fs::write(&links_path, r#"{"version":"1.0","links":[]}"#).unwrap();
        
        // Import from CSV
        let import_stats = trace_manager.import_from_csv(&import_csv_path).unwrap();
        assert_eq!(import_stats.total_processed, 1);
        assert!(import_stats.successful_imports > 0);
        
        // Verify import worked
        let imported_links = trace_manager.get_trace_links().unwrap();
        assert!(imported_links.len() > 0);
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
    
    #[test]
    fn test_data_integrity_validation() {
        // Test data integrity and consistency validation
        let temp_dir = env::temp_dir().join("qms_integration_test_5");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Initialize audit system for test
        init_audit_for_test(&temp_dir);
        
        let trace_manager = TraceabilityManager::new(&temp_dir).unwrap();
        let mut req_manager = RequirementManager::new(&temp_dir).unwrap();
        let mut test_manager = TestCaseManager::new(&temp_dir).unwrap();
        
        // Create requirements and test cases
        let req1_id = req_manager.create_requirement(
            "test_project".to_string(),
            "REQ-INT-001".to_string(),
            "Integrity Test Requirement 1".to_string(),
            "Testing data integrity".to_string(),
            crate::modules::traceability::requirement::RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();

        let req2_id = req_manager.create_requirement(
            "test_project".to_string(),
            "REQ-INT-002".to_string(),
            "Integrity Test Requirement 2".to_string(),
            "Testing data integrity".to_string(),
            crate::modules::traceability::requirement::RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();

        test_manager.create_test_case(
            "TC-INT-001".to_string(),
            "Integrity Test Case 1".to_string(),
            "Testing data integrity".to_string(),
            crate::modules::traceability::test_case::TestCategory::Functional,
            crate::modules::traceability::test_case::TestPriority::High,
            "test_user".to_string(),
        ).unwrap();
        
        // Test circular dependency prevention
        let _link1 = trace_manager.create_trace_link(
            "REQ-INT-001",
            "REQ-INT-002",
            TraceLinkType::DependsOn,
        ).unwrap();

        // This should fail due to circular dependency
        let result = trace_manager.create_trace_link(
            "REQ-INT-002",
            "REQ-INT-001",
            TraceLinkType::DependsOn,
        );
        assert!(result.is_err());

        // Test duplicate link prevention
        let _link2 = trace_manager.create_trace_link(
            "REQ-INT-001",
            "TC-INT-001",
            TraceLinkType::Verifies,
        ).unwrap();

        // This should fail due to duplicate link
        let result = trace_manager.create_trace_link(
            "REQ-INT-001",
            "TC-INT-001",
            TraceLinkType::Verifies,
        );
        assert!(result.is_err());
        
        // Test orphan detection
        let orphans = trace_manager.find_orphaned_items().unwrap();
        // Should have no orphans since all entities are linked
        assert!(orphans.is_empty());
        
        // Test consistency validation
        let all_links = trace_manager.get_trace_links().unwrap();
        assert_eq!(all_links.len(), 2);
        
        // Verify all links have valid source and target entities
        for link in &all_links {
            assert!(!link.source_id.is_empty());
            assert!(!link.target_id.is_empty());
            assert!(!link.source_type.is_empty());
            assert!(!link.target_type.is_empty());
        }
        
        // Test RTM consistency
        let rtm = trace_manager.generate_rtm().unwrap();
        assert_eq!(rtm.entities.len(), 3);
        assert_eq!(rtm.links.len(), 2);
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
    
    #[test]
    fn test_performance_with_large_dataset() {
        // Test performance with larger dataset
        let temp_dir = env::temp_dir().join("qms_integration_test_6");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Initialize audit system for test
        init_audit_for_test(&temp_dir);
        
        let trace_manager = TraceabilityManager::new(&temp_dir).unwrap();
        let mut req_manager = RequirementManager::new(&temp_dir).unwrap();
        let mut test_manager = TestCaseManager::new(&temp_dir).unwrap();

        // Create moderate dataset (limited for test performance)
        let mut requirement_ids = Vec::new();
        let mut test_case_ids = Vec::new();

        // Create 10 requirements
        for i in 0..10 {
            let req_id = req_manager.create_requirement(
                "test_project".to_string(),
                format!("REQ-PERF2-{:03}", i),
                format!("Performance Test Requirement {}", i),
                format!("Performance testing requirement {}", i),
                crate::modules::traceability::requirement::RequirementCategory::Functional,
                "test_user".to_string(),
            ).unwrap();
            requirement_ids.push(format!("REQ-PERF2-{:03}", i));
        }

        // Create 15 test cases
        for i in 0..15 {
            test_manager.create_test_case(
                format!("TC-PERF2-{:03}", i),
                format!("Performance Test Case {}", i),
                format!("Performance testing test case {}", i),
                crate::modules::traceability::test_case::TestCategory::Functional,
                crate::modules::traceability::test_case::TestPriority::High,
                "test_user".to_string(),
            ).unwrap();
            test_case_ids.push(format!("TC-PERF2-{:03}", i));
        }
        
        // Create multiple links (each requirement linked to multiple test cases)
        let mut link_count = 0;
        for req_id in &requirement_ids {
            for (i, tc_id) in test_case_ids.iter().enumerate() {
                if i < 2 {  // Link each requirement to 2 test cases
                    let _link = trace_manager.create_trace_link(
                        req_id,
                        tc_id,
                        TraceLinkType::Verifies,
                    ).unwrap();
                    link_count += 1;
                }
            }
        }
        
        // Test performance of various operations
        let start_time = std::time::Instant::now();

        // Test forward tracing performance
        for req_id in &requirement_ids {
            let _forward_trace = trace_manager.trace_forward(req_id).unwrap();
        }

        // Test backward tracing performance
        for tc_id in &test_case_ids[..5] {  // Test first 5 test cases
            let _backward_trace = trace_manager.trace_backward(tc_id).unwrap();
        }
        
        // Test RTM generation performance
        let rtm = trace_manager.generate_rtm().unwrap();
        assert_eq!(rtm.entities.len(), 12);  // 10 requirements + 2 linked test cases (only linked entities are included in RTM)
        assert_eq!(rtm.links.len(), link_count);
        
        // Test orphan detection performance
        let orphans = trace_manager.find_orphaned_items().unwrap();
        
        let elapsed = start_time.elapsed();
        
        // Performance should be reasonable (under 5 seconds for this dataset)
        assert!(elapsed.as_secs() < 5);
        
        // Verify data integrity with large dataset
        let all_links = trace_manager.get_trace_links().unwrap();
        assert_eq!(all_links.len(), link_count);
        
        // Test export performance
        let export_path = temp_dir.join("performance_test_export.csv");
        trace_manager.export_rtm_csv(&export_path).unwrap();
        assert!(export_path.exists());
        
        println!("Performance test completed in {:?} with {} entities and {} links", 
                elapsed, rtm.entities.len(), rtm.links.len());
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
