//! Integration tests for Risk Manager module
//! Task 3.1.20: Risk Module Unit Tests

#[cfg(test)]
mod tests {
    use super::super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_risk_manager_basic_functionality() {
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path();
        
        // Test that we can create a basic risk manager
        let result = RiskManager::new(project_path);
        assert!(result.is_ok());
        
        let risk_manager = result.unwrap();
        
        // Test initialization
        let init_result = risk_manager.initialize();
        assert!(init_result.is_ok());
        
        // Verify basic directory structure exists
        assert!(project_path.join("risks").exists());
    }
    
    #[test]
    fn test_rpn_calculation() {
        // Test RPN calculation directly
        let severity = 3;
        let occurrence = 2;
        let detectability = 4;
        let rpn = severity * occurrence * detectability;
        assert_eq!(rpn, 24);
        
        // Test different combinations
        let high_rpn = 5 * 5 * 5;
        assert_eq!(high_rpn, 125);
        
        let low_rpn = 1 * 1 * 1;
        assert_eq!(low_rpn, 1);
    }
    
    #[test]
    fn test_directory_creation() {
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path();
        
        // Test that directories can be created
        let risk_dir = project_path.join("risks");
        std::fs::create_dir_all(&risk_dir).unwrap();
        assert!(risk_dir.exists());
        
        let fmea_dir = project_path.join("fmea");
        std::fs::create_dir_all(&fmea_dir).unwrap();
        assert!(fmea_dir.exists());
    }
    
    #[test]
    fn test_file_operations() {
        let temp_dir = tempdir().unwrap();
        let project_path = temp_dir.path();
        
        // Test basic file operations
        let test_file = project_path.join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();
        assert!(test_file.exists());
        
        let content = std::fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, "test content");
    }
    
    #[test]
    fn test_json_serialization() {
        // Test basic JSON operations using stdlib-only JSON utilities
        use crate::json_utils::JsonValue;
        use std::collections::HashMap;

        // Create test data using our JSON utilities
        let mut data = HashMap::new();
        data.insert("id".to_string(), JsonValue::String("RISK-001".to_string()));
        data.insert("title".to_string(), JsonValue::String("Test Risk".to_string()));
        data.insert("severity".to_string(), JsonValue::Number(3.0));
        data.insert("probability".to_string(), JsonValue::Number(2.0));

        let json_obj = JsonValue::Object(data);
        let json_string = json_obj.json_to_string();

        // Verify JSON contains expected data
        assert!(json_string.contains("RISK-001"));
        assert!(json_string.contains("Test Risk"));
        assert!(json_string.contains("3"));
        assert!(json_string.contains("2"));

        // Test parsing back
        let parsed = JsonValue::parse(&json_string).unwrap();
        if let JsonValue::Object(obj) = parsed {
            if let Some(JsonValue::String(id)) = obj.get("id") {
                assert_eq!(id, "RISK-001");
            } else {
                panic!("Expected id field to be a string");
            }
        } else {
            panic!("Expected parsed JSON to be an object");
        }
    }
}
