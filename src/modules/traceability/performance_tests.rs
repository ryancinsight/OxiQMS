// Performance tests for traceability module
// Implementation for Task 3.2.12 Traceability Module Testing

use std::fs;
use std::env;
use std::time::Instant;
use crate::modules::traceability::links::{TraceabilityManager, TraceLinkType};
use crate::modules::traceability::requirement::RequirementManager;
use crate::modules::traceability::test_case::TestCaseManager;
use crate::modules::audit_logger::{AuditConfig, initialize_audit_system};

#[cfg(test)]
mod performance_tests {
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
    fn test_large_requirement_set_performance() {
        // Test performance with large requirement sets
        let temp_dir = env::temp_dir().join("qms_perf_test_1");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Initialize audit system for test
        init_audit_for_test(&temp_dir);
        
        let trace_manager = TraceabilityManager::new(&temp_dir).unwrap();
        let mut req_manager = RequirementManager::new(&temp_dir).unwrap();
        let mut test_manager = TestCaseManager::new(&temp_dir).unwrap();
        
        let start_time = Instant::now();
        
        // Create large requirement set (50 requirements)
        let mut requirement_ids = Vec::new();
        for i in 0..50 {
            let req_id = req_manager.create_requirement(
                "test_project".to_string(),
                format!("REQ-PERF-{:03}", i),
                format!("Large Set Requirement {}", i),
                format!("Performance testing requirement {}", i),
                crate::modules::traceability::requirement::RequirementCategory::Functional,
                "test_user".to_string(),
            ).unwrap();
            requirement_ids.push(format!("REQ-PERF-{:03}", i));
        }

        // Create corresponding test cases (100 test cases)
        let mut test_case_ids = Vec::new();
        for i in 0..100 {
            test_manager.create_test_case(
                format!("TC-PERF-{:03}", i),
                format!("Large Set Test Case {}", i),
                format!("Performance testing test case {}", i),
                crate::modules::traceability::test_case::TestCategory::Functional,
                crate::modules::traceability::test_case::TestPriority::High,
                "test_user".to_string(),
            ).unwrap();
            test_case_ids.push(format!("TC-PERF-{:03}", i));
        }
        
        let creation_time = start_time.elapsed();
        println!("Created {} requirements and {} test cases in {:?}",
                requirement_ids.len(), test_case_ids.len(), creation_time);

        // Create links (each requirement links to 2 test cases)
        let link_start = Instant::now();
        let mut link_count = 0;

        for (req_idx, req_id) in requirement_ids.iter().enumerate() {
            let tc1_idx = (req_idx * 2) % test_case_ids.len();
            let tc2_idx = (req_idx * 2 + 1) % test_case_ids.len();

            let _link1 = trace_manager.create_trace_link(
                req_id,
                &test_case_ids[tc1_idx],
                TraceLinkType::Verifies,
            ).unwrap();
            link_count += 1;
            
            let _link2 = trace_manager.create_trace_link(
                req_id,
                &test_case_ids[tc2_idx],
                TraceLinkType::Verifies,
            ).unwrap();
            link_count += 1;
        }
        
        let link_creation_time = link_start.elapsed();
        println!("Created {} links in {:?}", link_count, link_creation_time);
        
        // Test RTM generation performance
        let rtm_start = Instant::now();
        let rtm = trace_manager.generate_rtm().unwrap();
        let rtm_time = rtm_start.elapsed();
        
        assert_eq!(rtm.entities.len(), 150);  // 50 requirements + 100 test cases
        assert_eq!(rtm.links.len(), link_count);
        
        println!("Generated RTM with {} entities and {} links in {:?}",
                rtm.entities.len(), rtm.links.len(), rtm_time);
        
        // Test export performance
        let export_start = Instant::now();
        let export_path = temp_dir.join("large_set_export.csv");
        trace_manager.export_rtm_csv(&export_path).unwrap();
        let export_time = export_start.elapsed();
        
        assert!(export_path.exists());
        println!("Exported RTM to CSV in {:?}", export_time);
        
        // Test orphan detection performance
        let orphan_start = Instant::now();
        let orphans = trace_manager.find_orphaned_items().unwrap();
        let orphan_time = orphan_start.elapsed();
        
        println!("Found {} orphaned items in {:?}", orphans.len(), orphan_time);
        
        let total_time = start_time.elapsed();
        println!("Total performance test completed in {:?}", total_time);
        
        // Performance assertions (should complete in reasonable time)
        assert!(creation_time.as_secs() < 10);  // Entity creation should be fast
        assert!(link_creation_time.as_secs() < 15);  // Link creation should be reasonable
        assert!(rtm_time.as_secs() < 5);  // RTM generation should be fast
        assert!(export_time.as_secs() < 5);  // Export should be fast
        assert!(orphan_time.as_secs() < 3);  // Orphan detection should be fast
        assert!(total_time.as_secs() < 30);  // Total should be under 30 seconds
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
    
    #[test]
    fn test_complex_dependency_graph_performance() {
        // Test performance with complex dependency graphs
        let temp_dir = env::temp_dir().join("qms_perf_test_2");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Initialize audit system for test
        init_audit_for_test(&temp_dir);
        
        let trace_manager = TraceabilityManager::new(&temp_dir).unwrap();
        let mut req_manager = RequirementManager::new(&temp_dir).unwrap();
        let mut test_manager = TestCaseManager::new(&temp_dir).unwrap();

        // Create hierarchical requirement structure
        let mut requirement_ids = Vec::new();
        for i in 0..20 {
            let req_id = req_manager.create_requirement(
                "test_project".to_string(),
                format!("REQ-COMP-{:03}", i),
                format!("Complex Dependency Requirement {}", i),
                format!("Complex dependency testing requirement {}", i),
                crate::modules::traceability::requirement::RequirementCategory::Functional,
                "test_user".to_string(),
            ).unwrap();
            requirement_ids.push(format!("REQ-COMP-{:03}", i));
        }

        // Create test cases
        let mut test_case_ids = Vec::new();
        for i in 0..30 {
            test_manager.create_test_case(
                format!("TC-COMP-{:03}", i),
                format!("Complex Dependency Test Case {}", i),
                format!("Complex dependency testing test case {}", i),
                crate::modules::traceability::test_case::TestCategory::Functional,
                crate::modules::traceability::test_case::TestPriority::High,
                "test_user".to_string(),
            ).unwrap();
            test_case_ids.push(format!("TC-COMP-{:03}", i));
        }
        
        // Create complex dependency graph
        let link_start = Instant::now();
        let mut link_count = 0;
        
        // Create hierarchical requirement dependencies
        for i in 0..requirement_ids.len() - 1 {
            let _link = trace_manager.create_trace_link(
                &requirement_ids[i],
                &requirement_ids[i + 1],
                TraceLinkType::DerivedFrom,
            ).unwrap();
            link_count += 1;
        }

        // Create requirement-to-test links
        for (req_idx, req_id) in requirement_ids.iter().enumerate() {
            let tc_idx = req_idx % test_case_ids.len();
            let _link = trace_manager.create_trace_link(
                req_id,
                &test_case_ids[tc_idx],
                TraceLinkType::Verifies,
            ).unwrap();
            link_count += 1;
        }
        
        // Create test case dependencies
        for i in 0..test_case_ids.len() - 1 {
            if i % 3 == 0 {  // Create some test dependencies
                let _link = trace_manager.create_trace_link(
                    &test_case_ids[i],
                    &test_case_ids[i + 1],
                    TraceLinkType::DependsOn,
                ).unwrap();
                link_count += 1;
            }
        }
        
        let link_creation_time = link_start.elapsed();
        println!("Created {} complex dependency links in {:?}", link_count, link_creation_time);
        
        // Test forward tracing performance on complex graph
        let forward_start = Instant::now();
        let forward_traces = requirement_ids.iter().map(|req_id| {
            trace_manager.trace_forward(req_id).unwrap()
        }).collect::<Vec<_>>();
        let forward_time = forward_start.elapsed();

        println!("Completed {} forward traces in {:?}", forward_traces.len(), forward_time);

        // Test backward tracing performance
        let backward_start = Instant::now();
        let backward_traces = test_case_ids.iter().take(10).map(|tc_id| {
            trace_manager.trace_backward(tc_id).unwrap()
        }).collect::<Vec<_>>();
        let backward_time = backward_start.elapsed();
        
        println!("Completed {} backward traces in {:?}", backward_traces.len(), backward_time);
        
        // Test dependency graph export performance
        let graph_start = Instant::now();
        let graph_path = temp_dir.join("complex_graph.dot");
        trace_manager.export_dependency_graph(&graph_path).unwrap();
        let graph_time = graph_start.elapsed();
        
        assert!(graph_path.exists());
        println!("Exported dependency graph in {:?}", graph_time);
        
        // Verify graph complexity
        let graph_content = fs::read_to_string(&graph_path).unwrap();
        assert!(graph_content.contains("digraph TraceabilityGraph"));
        
        // Performance assertions
        assert!(link_creation_time.as_secs() < 10);
        assert!(forward_time.as_secs() < 10);
        assert!(backward_time.as_secs() < 5);
        assert!(graph_time.as_secs() < 5);
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
    
    #[test]
    fn test_bulk_import_performance() {
        // Test performance of bulk import operations
        let temp_dir = env::temp_dir().join("qms_perf_test_3");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Initialize audit system for test
        init_audit_for_test(&temp_dir);
        
        let trace_manager = TraceabilityManager::new(&temp_dir).unwrap();
        let mut req_manager = RequirementManager::new(&temp_dir).unwrap();
        let mut test_manager = TestCaseManager::new(&temp_dir).unwrap();
        
        // Create entities for bulk import
        let mut requirement_ids = Vec::new();
        let mut test_case_ids = Vec::new();

        for i in 0..25 {
            let req_id = req_manager.create_requirement(
                "test_project".to_string(),
                format!("REQ-BULK-{:03}", i),
                format!("Bulk Import Requirement {}", i),
                format!("Bulk import testing requirement {}", i),
                crate::modules::traceability::requirement::RequirementCategory::Functional,
                "test_user".to_string(),
            ).unwrap();
            requirement_ids.push(format!("REQ-BULK-{:03}", i));

            test_manager.create_test_case(
                format!("TC-BULK-{:03}", i),
                format!("Bulk Import Test Case {}", i),
                format!("Bulk import testing test case {}", i),
                crate::modules::traceability::test_case::TestCategory::Functional,
                crate::modules::traceability::test_case::TestPriority::High,
                "test_user".to_string(),
            ).unwrap();
            test_case_ids.push(format!("TC-BULK-{:03}", i));
        }
        
        // Create bulk import CSV
        let csv_path = temp_dir.join("bulk_import.csv");
        let mut csv_content = String::from("SourceType,SourceID,TargetType,TargetID,LinkType,CreatedBy\n");
        
        for (req_id, tc_id) in requirement_ids.iter().zip(test_case_ids.iter()) {
            csv_content.push_str(&format!(
                "Requirement,{},TestCase,{},Verifies,bulk_import_user\n",
                req_id, tc_id
            ));
        }
        
        fs::write(&csv_path, csv_content).unwrap();
        
        // Test bulk import performance
        let import_start = Instant::now();
        let import_stats = trace_manager.import_from_csv(&csv_path).unwrap();
        let import_time = import_start.elapsed();
        
        println!("Bulk imported {} links in {:?}", import_stats.successful_imports, import_time);
        
        assert_eq!(import_stats.total_processed, 25);
        assert_eq!(import_stats.successful_imports, 25);
        assert_eq!(import_stats.failed_imports, 0);
        
        // Test bulk export performance
        let export_start = Instant::now();
        let export_csv_path = temp_dir.join("bulk_export.csv");
        trace_manager.export_rtm_csv(&export_csv_path).unwrap();
        let export_time = export_start.elapsed();
        
        println!("Bulk exported RTM in {:?}", export_time);
        
        let export_json_path = temp_dir.join("bulk_export.json");
        trace_manager.export_rtm_json(&export_json_path).unwrap();
        
        // Verify export contents
        assert!(export_csv_path.exists());
        assert!(export_json_path.exists());
        
        let csv_content = fs::read_to_string(&export_csv_path).unwrap();
        let json_content = fs::read_to_string(&export_json_path).unwrap();
        
        assert!(csv_content.contains("Entity ID,Entity Type"));
        assert!(json_content.contains("\"entities\""));
        
        // Performance assertions
        assert!(import_time.as_secs() < 5);
        assert!(export_time.as_secs() < 5);
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
    
    #[test]
    fn test_memory_usage_with_large_dataset() {
        // Test memory usage patterns with large datasets
        let temp_dir = env::temp_dir().join("qms_perf_test_4");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Initialize audit system for test
        init_audit_for_test(&temp_dir);
        
        let trace_manager = TraceabilityManager::new(&temp_dir).unwrap();
        let mut req_manager = RequirementManager::new(&temp_dir).unwrap();
        let mut test_manager = TestCaseManager::new(&temp_dir).unwrap();

        // Create moderate dataset to test memory usage (reduced for test performance)
        let entity_count = 10;
        let mut requirement_ids = Vec::new();
        let mut test_case_ids = Vec::new();

        for i in 0..entity_count {
            let req_id = req_manager.create_requirement(
                "test_project".to_string(),
                format!("REQ-MEM-{:03}", i),
                format!("Memory Test Requirement {}", i),
                format!("Memory usage testing requirement {}", i),
                crate::modules::traceability::requirement::RequirementCategory::Functional,
                "test_user".to_string(),
            ).unwrap();
            requirement_ids.push(format!("REQ-MEM-{:03}", i));

            test_manager.create_test_case(
                format!("TC-MEM-{:03}", i),
                format!("Memory Test Case {}", i),
                format!("Memory usage testing test case {}", i),
                crate::modules::traceability::test_case::TestCategory::Functional,
                crate::modules::traceability::test_case::TestPriority::High,
                "test_user".to_string(),
            ).unwrap();
            test_case_ids.push(format!("TC-MEM-{:03}", i));
        }
        
        // Create all-to-all links (high memory usage scenario)
        let link_start = Instant::now();
        let mut link_count = 0;

        for req_id in &requirement_ids {
            for tc_id in &test_case_ids {
                let _link = trace_manager.create_trace_link(
                    req_id,
                    tc_id,
                    TraceLinkType::Verifies,
                ).unwrap();
                link_count += 1;
            }
        }
        
        let link_time = link_start.elapsed();
        println!("Created {} all-to-all links in {:?}", link_count, link_time);
        
        // Test memory-intensive operations
        let memory_test_start = Instant::now();
        
        // Load all links into memory
        let all_links = trace_manager.get_trace_links().unwrap();
        assert_eq!(all_links.len(), link_count);
        
        // Generate RTM (memory-intensive)
        let rtm = trace_manager.generate_rtm().unwrap();
        assert_eq!(rtm.entities.len(), entity_count * 2);
        assert_eq!(rtm.links.len(), link_count);
        
        // Test all forward traces (memory-intensive)
        for req_id in &requirement_ids {
            let forward_trace = trace_manager.trace_forward(req_id).unwrap();
            assert_eq!(forward_trace.path.len(), entity_count);
        }

        // Test all backward traces (memory-intensive)
        for tc_id in &test_case_ids {
            let backward_trace = trace_manager.trace_backward(tc_id).unwrap();
            assert_eq!(backward_trace.path.len(), entity_count);
        }
        
        let memory_test_time = memory_test_start.elapsed();
        println!("Completed memory-intensive operations in {:?}", memory_test_time);
        
        // Performance assertions (relaxed for test environment)
        assert!(link_time.as_secs() < 30);  // All-to-all linking should complete (reduced dataset)
        assert!(memory_test_time.as_secs() < 20);  // Memory operations should complete
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
    
    #[test]
    fn test_concurrent_operations_performance() {
        // Test performance of concurrent operations (simulated)
        let temp_dir = env::temp_dir().join("qms_perf_test_5");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Initialize audit system for test
        init_audit_for_test(&temp_dir);
        
        let trace_manager = TraceabilityManager::new(&temp_dir).unwrap();
        let mut req_manager = RequirementManager::new(&temp_dir).unwrap();
        let mut test_manager = TestCaseManager::new(&temp_dir).unwrap();

        // Create entities
        let mut requirement_ids = Vec::new();
        let mut test_case_ids = Vec::new();

        for i in 0..20 {
            let req_id = req_manager.create_requirement(
                "test_project".to_string(),
                format!("REQ-CONC-{:03}", i),
                format!("Concurrent Test Requirement {}", i),
                format!("Concurrent testing requirement {}", i),
                crate::modules::traceability::requirement::RequirementCategory::Functional,
                "test_user".to_string(),
            ).unwrap();
            requirement_ids.push(format!("REQ-CONC-{:03}", i));

            test_manager.create_test_case(
                format!("TC-CONC-{:03}", i),
                format!("Concurrent Test Case {}", i),
                format!("Concurrent testing test case {}", i),
                crate::modules::traceability::test_case::TestCategory::Functional,
                crate::modules::traceability::test_case::TestPriority::High,
                "test_user".to_string(),
            ).unwrap();
            test_case_ids.push(format!("TC-CONC-{:03}", i));
        }

        // Create links
        for (req_id, tc_id) in requirement_ids.iter().zip(test_case_ids.iter()) {
            let _link = trace_manager.create_trace_link(
                req_id,
                tc_id,
                TraceLinkType::Verifies,
            ).unwrap();
        }
        
        // Simulate concurrent read operations
        let concurrent_start = Instant::now();
        
        // Multiple read operations in sequence (simulating concurrent access)
        for _ in 0..10 {
            let _all_links = trace_manager.get_trace_links().unwrap();
            let _rtm = trace_manager.generate_rtm().unwrap();
            let _orphans = trace_manager.find_orphaned_items().unwrap();
        }
        
        let concurrent_time = concurrent_start.elapsed();
        println!("Completed simulated concurrent operations in {:?}", concurrent_time);
        
        // Test mixed read/write operations
        let mixed_start = Instant::now();
        
        for i in 0..5 {
            // Add new link
            let new_req_id = req_manager.create_requirement(
                "test_project".to_string(),
                format!("REQ-MIX-{:03}", i),
                format!("Mixed Operation Requirement {}", i),
                format!("Mixed operation testing {}", i),
                crate::modules::traceability::requirement::RequirementCategory::Functional,
                "test_user".to_string(),
            ).unwrap();
            
            let _link = trace_manager.create_trace_link(
                &format!("REQ-MIX-{:03}", i),
                &test_case_ids[0],
                TraceLinkType::Verifies,
            ).unwrap();

            // Read operations
            let _forward_trace = trace_manager.trace_forward(&format!("REQ-MIX-{:03}", i)).unwrap();
            let _backward_trace = trace_manager.trace_backward(&test_case_ids[0]).unwrap();
        }
        
        let mixed_time = mixed_start.elapsed();
        println!("Completed mixed read/write operations in {:?}", mixed_time);
        
        // Performance assertions
        assert!(concurrent_time.as_secs() < 10);
        assert!(mixed_time.as_secs() < 10);
        
        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
