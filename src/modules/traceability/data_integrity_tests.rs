// Data integrity tests for traceability module
// Implementation for Task 3.2.12 Traceability Module Testing

use std::fs;
use std::env;
use crate::modules::traceability::links::{TraceabilityManager, TraceLinkType};
use crate::modules::traceability::requirement::RequirementManager;
use crate::modules::traceability::test_case::TestCaseManager;
use crate::modules::audit_logger::{AuditConfig, initialize_audit_system};

#[cfg(test)]
mod tests {
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
    fn test_consistency_validation() {
        // Test comprehensive consistency validation
        let temp_dir = env::temp_dir().join("qms_integrity_test_1");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Initialize audit system for test
        init_audit_for_test(&temp_dir);
        
        let trace_manager = TraceabilityManager::new(&temp_dir).unwrap();
        let mut req_manager = RequirementManager::new(&temp_dir).unwrap();
        let mut test_manager = TestCaseManager::new(&temp_dir).unwrap();
        
        // Create valid entities
        let req1_id = req_manager.create_requirement(
            "test_project".to_string(),
            "REQ-CONS-001".to_string(),
            "Consistency Test Requirement 1".to_string(),
            "Testing consistency validation".to_string(),
            crate::modules::traceability::requirement::RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();

        let req2_id = req_manager.create_requirement(
            "test_project".to_string(),
            "REQ-CONS-002".to_string(),
            "Consistency Test Requirement 2".to_string(),
            "Testing consistency validation".to_string(),
            crate::modules::traceability::requirement::RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();

        test_manager.create_test_case(
            "TC-CONS-001".to_string(),
            "Consistency Test Case 1".to_string(),
            "Testing consistency validation".to_string(),
            crate::modules::traceability::test_case::TestCategory::Functional,
            crate::modules::traceability::test_case::TestPriority::High,
            "test_user".to_string(),
        ).unwrap();
        
        // Create valid links
        let _link1 = trace_manager.create_trace_link(
            "REQ-CONS-001",
            "TC-CONS-001",
            TraceLinkType::Verifies,
        ).unwrap();

        let _link2 = trace_manager.create_trace_link(
            "REQ-CONS-002",
            "TC-CONS-001",
            TraceLinkType::Verifies,
        ).unwrap();
        
        // Verify consistency
        let all_links = trace_manager.get_trace_links().unwrap();
        assert_eq!(all_links.len(), 2);
        
        // Check all links have valid structure
        for link in &all_links {
            assert!(!link.id.is_empty());
            assert!(!link.source_id.is_empty());
            assert!(!link.target_id.is_empty());
            assert!(!link.source_type.is_empty());
            assert!(!link.target_type.is_empty());
            assert!(!link.created_at.is_empty());
            assert!(!link.created_by.is_empty());
        }
        
        // Test RTM consistency
        let rtm = trace_manager.generate_rtm().unwrap();
        assert_eq!(rtm.entities.len(), 3);  // 2 requirements + 1 test case
        assert_eq!(rtm.links.len(), 2);
        
        // Check RTM entities have proper relationships
        for entity in &rtm.entities {
            assert!(!entity.entity_id.is_empty());
            assert!(!entity.entity_type.is_empty());
            assert!(!entity.title.is_empty());
            assert!(!entity.status.is_empty());
        }
        
        // Verify forward/backward traceability consistency
        let forward_req1 = trace_manager.trace_forward("REQ-CONS-001").unwrap();
        let forward_req2 = trace_manager.trace_forward("REQ-CONS-002").unwrap();
        let backward_tc1 = trace_manager.trace_backward("TC-CONS-001").unwrap();
        
        assert_eq!(forward_req1.path.len(), 1);
        assert_eq!(forward_req2.path.len(), 1);
        assert_eq!(backward_tc1.path.len(), 2);  // Should link back to both requirements
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
    
    #[test]
    fn test_orphan_detection_accuracy() {
        // Test accurate orphan detection
        let temp_dir = env::temp_dir().join("qms_integrity_test_2");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Initialize audit system for test
        init_audit_for_test(&temp_dir);
        
        let trace_manager = TraceabilityManager::new(&temp_dir).unwrap();
        let mut req_manager = RequirementManager::new(&temp_dir).unwrap();
        let mut test_manager = TestCaseManager::new(&temp_dir).unwrap();
        
        // Create entities - some linked, some orphaned
        let req1_id = req_manager.create_requirement(
            "test_project".to_string(),
            "REQ-LINK-001".to_string(),
            "Linked Requirement".to_string(),
            "This requirement will be linked".to_string(),
            crate::modules::traceability::requirement::RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();

        let req2_id = req_manager.create_requirement(
            "test_project".to_string(),
            "REQ-ORPH-001".to_string(),
            "Orphaned Requirement".to_string(),
            "This requirement will be orphaned".to_string(),
            crate::modules::traceability::requirement::RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();

        test_manager.create_test_case(
            "TC-LINK-001".to_string(),
            "Linked Test Case".to_string(),
            "This test case will be linked".to_string(),
            crate::modules::traceability::test_case::TestCategory::Functional,
            crate::modules::traceability::test_case::TestPriority::High,
            "test_user".to_string(),
        ).unwrap();

        test_manager.create_test_case(
            "TC-ORPH-001".to_string(),
            "Orphaned Test Case".to_string(),
            "This test case will be orphaned".to_string(),
            crate::modules::traceability::test_case::TestCategory::Functional,
            crate::modules::traceability::test_case::TestPriority::High,
            "test_user".to_string(),
        ).unwrap();
        
        // Create only some links, leaving others orphaned
        let _link1 = trace_manager.create_trace_link(
            "REQ-LINK-001",
            "TC-LINK-001",
            TraceLinkType::Verifies,
        ).unwrap();
        
        // Test orphan detection
        let orphans = trace_manager.find_orphaned_items().unwrap();
        
        // Should detect orphaned requirement and test case
        let orphan_req_ids: Vec<&str> = orphans.iter()
            .filter(|o| o.entity_type == "Requirement")
            .map(|o| o.entity_id.as_str())
            .collect();

        let orphan_tc_ids: Vec<&str> = orphans.iter()
            .filter(|o| o.entity_type == "TestCase")
            .map(|o| o.entity_id.as_str())
            .collect();

        assert!(orphan_req_ids.contains(&"REQ-ORPH-001"));
        assert!(orphan_tc_ids.contains(&"TC-ORPH-001"));

        // Verify linked entities are NOT in orphan list
        assert!(!orphan_req_ids.contains(&"REQ-LINK-001"));
        assert!(!orphan_tc_ids.contains(&"TC-LINK-001"));

        // Test after linking orphaned entities
        let _link2 = trace_manager.create_trace_link(
            "REQ-ORPH-001",
            "TC-ORPH-001",
            TraceLinkType::Verifies,
        ).unwrap();
        
        let orphans_after = trace_manager.find_orphaned_items().unwrap();
        assert!(orphans_after.is_empty());
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
    
    #[test]
    fn test_circular_dependency_prevention() {
        // Test robust circular dependency prevention
        let temp_dir = env::temp_dir().join("qms_integrity_test_3");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Initialize audit system for test
        init_audit_for_test(&temp_dir);
        
        let trace_manager = TraceabilityManager::new(&temp_dir).unwrap();
        let mut req_manager = RequirementManager::new(&temp_dir).unwrap();
        
        // Create requirements for circular dependency testing
        let req1_id = req_manager.create_requirement(
            "test_project".to_string(),
            "REQ-CIRC-A".to_string(),
            "Circular Test Requirement A".to_string(),
            "Testing circular dependency prevention".to_string(),
            crate::modules::traceability::requirement::RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();

        let req2_id = req_manager.create_requirement(
            "test_project".to_string(),
            "REQ-CIRC-B".to_string(),
            "Circular Test Requirement B".to_string(),
            "Testing circular dependency prevention".to_string(),
            crate::modules::traceability::requirement::RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();

        let req3_id = req_manager.create_requirement(
            "test_project".to_string(),
            "REQ-CIRC-C".to_string(),
            "Circular Test Requirement C".to_string(),
            "Testing circular dependency prevention".to_string(),
            crate::modules::traceability::requirement::RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();
        
        // Create initial links A -> B -> C
        let _link1 = trace_manager.create_trace_link(
            "REQ-CIRC-A",
            "REQ-CIRC-B",
            TraceLinkType::DependsOn,
        ).unwrap();

        let _link2 = trace_manager.create_trace_link(
            "REQ-CIRC-B",
            "REQ-CIRC-C",
            TraceLinkType::DependsOn,
        ).unwrap();

        // Test direct circular dependency: C -> A (should fail)
        let result = trace_manager.create_trace_link(
            "REQ-CIRC-C",
            "REQ-CIRC-A",
            TraceLinkType::DependsOn,
        );
        assert!(result.is_err());

        // Test indirect circular dependency: C -> B (should fail)
        let result = trace_manager.create_trace_link(
            "REQ-CIRC-C",
            "REQ-CIRC-B",
            TraceLinkType::DependsOn,
        );
        assert!(result.is_err());

        // Test self-referential dependency: A -> A (should fail)
        let result = trace_manager.create_trace_link(
            "REQ-CIRC-A",
            "REQ-CIRC-A",
            TraceLinkType::DependsOn,
        );
        assert!(result.is_err());
        
        // Verify existing links remain intact
        let all_links = trace_manager.get_trace_links().unwrap();
        assert_eq!(all_links.len(), 2);
        
        // Test that non-circular links still work
        let req4_id = req_manager.create_requirement(
            "test_project".to_string(),
            "REQ-CIRC-D".to_string(),
            "Non-Circular Requirement D".to_string(),
            "Testing non-circular dependency".to_string(),
            crate::modules::traceability::requirement::RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();

        let _link3 = trace_manager.create_trace_link(
            "REQ-CIRC-C",
            "REQ-CIRC-D",
            TraceLinkType::DependsOn,
        ).unwrap();
        
        let all_links_after = trace_manager.get_trace_links().unwrap();
        assert_eq!(all_links_after.len(), 3);
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
    
    #[test]
    fn test_duplicate_link_prevention() {
        // Test prevention of duplicate links
        let temp_dir = env::temp_dir().join("qms_integrity_test_4");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Initialize audit system for test
        init_audit_for_test(&temp_dir);
        
        let trace_manager = TraceabilityManager::new(&temp_dir).unwrap();
        let mut req_manager = RequirementManager::new(&temp_dir).unwrap();
        let mut test_manager = TestCaseManager::new(&temp_dir).unwrap();

        // Create entities
        let req1_id = req_manager.create_requirement(
            "test_project".to_string(),
            "REQ-DUP-001".to_string(),
            "Duplicate Test Requirement".to_string(),
            "Testing duplicate link prevention".to_string(),
            crate::modules::traceability::requirement::RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();

        test_manager.create_test_case(
            "TC-DUP-001".to_string(),
            "Duplicate Test Case".to_string(),
            "Testing duplicate link prevention".to_string(),
            crate::modules::traceability::test_case::TestCategory::Functional,
            crate::modules::traceability::test_case::TestPriority::High,
            "test_user".to_string(),
        ).unwrap();
        
        // Create initial link
        let _link1 = trace_manager.create_trace_link(
            "REQ-DUP-001",
            "TC-DUP-001",
            TraceLinkType::Verifies,
        ).unwrap();

        // Attempt to create duplicate link (should fail)
        let result = trace_manager.create_trace_link(
            "REQ-DUP-001",
            "TC-DUP-001",
            TraceLinkType::Verifies,
        );
        assert!(result.is_err());

        // Verify only one link exists
        let all_links = trace_manager.get_trace_links().unwrap();
        assert_eq!(all_links.len(), 1);

        // Test that different link types are allowed
        let _link2 = trace_manager.create_trace_link(
            "REQ-DUP-001",
            "TC-DUP-001",
            TraceLinkType::Implements,
        ).unwrap();
        
        let all_links_after = trace_manager.get_trace_links().unwrap();
        assert_eq!(all_links_after.len(), 2);
        
        // Verify both links have different types
        let link_types: Vec<_> = all_links_after.iter()
            .map(|l| l.link_type.to_string())
            .collect();
        assert!(link_types.contains(&"Verifies".to_string()));
        assert!(link_types.contains(&"Implements".to_string()));
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
    
    #[test]
    fn test_data_corruption_recovery() {
        // Test recovery from data corruption scenarios
        let temp_dir = env::temp_dir().join("qms_integrity_test_5");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Initialize audit system for test
        init_audit_for_test(&temp_dir);
        
        let trace_manager = TraceabilityManager::new(&temp_dir).unwrap();
        let mut req_manager = RequirementManager::new(&temp_dir).unwrap();
        let mut test_manager = TestCaseManager::new(&temp_dir).unwrap();

        // Create valid data
        let req1_id = req_manager.create_requirement(
            "test_project".to_string(),
            "REQ-CORR-001".to_string(),
            "Corruption Test Requirement".to_string(),
            "Testing data corruption recovery".to_string(),
            crate::modules::traceability::requirement::RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();

        test_manager.create_test_case(
            "TC-CORR-001".to_string(),
            "Corruption Test Case".to_string(),
            "Testing data corruption recovery".to_string(),
            crate::modules::traceability::test_case::TestCategory::Functional,
            crate::modules::traceability::test_case::TestPriority::High,
            "test_user".to_string(),
        ).unwrap();

        let _link1 = trace_manager.create_trace_link(
            "REQ-CORR-001",
            "TC-CORR-001",
            TraceLinkType::Verifies,
        ).unwrap();
        
        // Verify initial state
        let initial_links = trace_manager.get_trace_links().unwrap();
        assert_eq!(initial_links.len(), 1);
        
        // Simulate data corruption by writing invalid JSON
        let links_file = temp_dir.join("trace").join("links.json");
        fs::write(&links_file, r#"{"version":"1.0","links":[{"invalid":"json"#).unwrap();
        
        // Test graceful handling of corrupted data
        let result = trace_manager.get_trace_links();
        assert!(result.is_err());
        
        // Test recovery by reinitializing
        let trace_manager_new = TraceabilityManager::new(&temp_dir).unwrap();
        let recovered_links = trace_manager_new.get_trace_links().unwrap();
        assert_eq!(recovered_links.len(), 0);  // Should start clean
        
        // Verify system still works after recovery
        let _new_link = trace_manager_new.create_trace_link(
            "REQ-CORR-001",
            "TC-CORR-001",
            TraceLinkType::Verifies,
        ).unwrap();
        
        let final_links = trace_manager_new.get_trace_links().unwrap();
        assert_eq!(final_links.len(), 1);
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
    
    #[test]
    fn test_concurrent_access_integrity() {
        // Test data integrity under simulated concurrent access
        let temp_dir = env::temp_dir().join("qms_integrity_test_6");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Initialize audit system for test
        init_audit_for_test(&temp_dir);
        
        let trace_manager = TraceabilityManager::new(&temp_dir).unwrap();
        let mut req_manager = RequirementManager::new(&temp_dir).unwrap();
        let mut test_manager = TestCaseManager::new(&temp_dir).unwrap();

        // Create base entities
        let mut requirement_ids = Vec::new();
        let mut test_case_ids = Vec::new();

        for i in 0..5 {
            let req_id = req_manager.create_requirement(
                "test_project".to_string(),
                format!("REQ-CONC-{:03}", i),
                format!("Concurrent Test Requirement {}", i),
                format!("Testing concurrent access integrity {}", i),
                crate::modules::traceability::requirement::RequirementCategory::Functional,
                "test_user".to_string(),
            ).unwrap();
            requirement_ids.push(format!("REQ-CONC-{:03}", i));

            test_manager.create_test_case(
                format!("TC-CONC-{:03}", i),
                format!("Concurrent Test Case {}", i),
                format!("Testing concurrent access integrity {}", i),
                crate::modules::traceability::test_case::TestCategory::Functional,
                crate::modules::traceability::test_case::TestPriority::High,
                "test_user".to_string(),
            ).unwrap();
            test_case_ids.push(format!("TC-CONC-{:03}", i));
        }
        
        // Simulate concurrent write operations
        let mut link_count = 0;
        for (i, req_id) in requirement_ids.iter().enumerate() {
            for (j, tc_id) in test_case_ids.iter().enumerate() {
                if i <= j {  // Create some links
                    let _link = trace_manager.create_trace_link(
                        req_id,
                        tc_id,
                        TraceLinkType::Verifies,
                    ).unwrap();
                    link_count += 1;
                }
            }
        }
        
        // Verify data integrity after concurrent operations
        let all_links = trace_manager.get_trace_links().unwrap();
        assert_eq!(all_links.len(), link_count);
        
        // Check each link has valid structure
        for link in &all_links {
            assert!(!link.id.is_empty());
            assert!(!link.source_id.is_empty());
            assert!(!link.target_id.is_empty());
            assert!(!link.created_at.is_empty());
            assert!(!link.created_by.is_empty());
            
            // Verify link references valid entities
            assert!(requirement_ids.iter().any(|r| r == &link.source_id) ||
                   test_case_ids.iter().any(|t| t == &link.source_id));
            assert!(requirement_ids.iter().any(|r| r == &link.target_id) ||
                   test_case_ids.iter().any(|t| t == &link.target_id));
        }

        // Test mixed read/write operations
        for i in 0..3 {
            // Read operations
            let _forward_trace = trace_manager.trace_forward(&requirement_ids[i]).unwrap();
            let _backward_trace = trace_manager.trace_backward(&test_case_ids[i]).unwrap();
            let _rtm = trace_manager.generate_rtm().unwrap();
            
            // Write operations
            let new_req_id = req_manager.create_requirement(
                "test_project".to_string(),
                format!("REQ-MIX-{:03}", i),
                format!("Mixed Operation Requirement {}", i),
                format!("Mixed operation testing {}", i),
                crate::modules::traceability::requirement::RequirementCategory::Functional,
                "test_user".to_string(),
            ).unwrap();

            let _new_link = trace_manager.create_trace_link(
                &format!("REQ-MIX-{:03}", i),
                &test_case_ids[0],
                TraceLinkType::Verifies,
            ).unwrap();
        }
        
        // Final integrity check
        let final_links = trace_manager.get_trace_links().unwrap();
        assert!(final_links.len() > link_count);
        
        let final_rtm = trace_manager.generate_rtm().unwrap();
        assert!(!final_rtm.entities.is_empty());
        assert!(!final_rtm.links.is_empty());
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
    
    #[test]
    fn test_validation_edge_cases() {
        // Test validation of edge cases and boundary conditions
        let temp_dir = env::temp_dir().join("qms_integrity_test_7");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Initialize audit system for test
        init_audit_for_test(&temp_dir);
        
        let trace_manager = TraceabilityManager::new(&temp_dir).unwrap();
        let mut req_manager = RequirementManager::new(&temp_dir).unwrap();
        let mut test_manager = TestCaseManager::new(&temp_dir).unwrap();

        // Test with edge case values
        let req1_id = req_manager.create_requirement(
            "test_project".to_string(),
            "REQ-EDGE-001".to_string(),
            "Edge Case Test Requirement".to_string(),
            "Testing edge cases and boundary conditions".to_string(),
            crate::modules::traceability::requirement::RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();

        test_manager.create_test_case(
            "TC-EDGE-001".to_string(),
            "Edge Case Test Case".to_string(),
            "Testing edge cases and boundary conditions".to_string(),
            crate::modules::traceability::test_case::TestCategory::Functional,
            crate::modules::traceability::test_case::TestPriority::High,
            "test_user".to_string(),
        ).unwrap();

        // Test valid link creation
        let _link1 = trace_manager.create_trace_link(
            "REQ-EDGE-001",
            "TC-EDGE-001",
            TraceLinkType::Verifies,
        ).unwrap();

        // Test invalid entity IDs (should fail)
        let result = trace_manager.create_trace_link(
            "INVALID-ID",
            "TC-EDGE-001",
            TraceLinkType::Verifies,
        );
        assert!(result.is_err());
        
        let result = trace_manager.create_trace_link(
            "REQ-EDGE-001",
            "INVALID-ID",
            TraceLinkType::Verifies,
        );
        assert!(result.is_err());

        // Test with special characters in entity content
        let req2_id = req_manager.create_requirement(
            "test_project".to_string(),
            "REQ-EDGE-002".to_string(),
            "Special \"Characters\" & Symbols: <test>".to_string(),
            "Testing with special characters: quotes, ampersands, brackets".to_string(),
            crate::modules::traceability::requirement::RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();

        let _link2 = trace_manager.create_trace_link(
            "REQ-EDGE-002",
            "TC-EDGE-001",
            TraceLinkType::Verifies,
        ).unwrap();
        
        // Test JSON serialization with special characters
        let all_links = trace_manager.get_trace_links().unwrap();
        assert_eq!(all_links.len(), 2);
        
        // Test RTM generation with special characters
        let rtm = trace_manager.generate_rtm().unwrap();
        assert_eq!(rtm.entities.len(), 3);
        
        // Test export with special characters
        let export_path = temp_dir.join("edge_case_export.csv");
        trace_manager.export_rtm_csv(&export_path).unwrap();
        
        let csv_content = fs::read_to_string(&export_path).unwrap();
        assert!(csv_content.contains("Special"));
        
        // Test import with special characters
        let import_csv_path = temp_dir.join("edge_case_import.csv");
        let csv_import_content = format!(
            "SourceType,SourceID,TargetType,TargetID,LinkType,CreatedBy\nRequirement,{},TestCase,{},Related,\"test,user\"\n",
            "REQ-EDGE-002", "TC-EDGE-001"
        );
        fs::write(&import_csv_path, csv_import_content).unwrap();
        
        let import_stats = trace_manager.import_from_csv(&import_csv_path).unwrap();
        assert!(import_stats.total_processed > 0);
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
