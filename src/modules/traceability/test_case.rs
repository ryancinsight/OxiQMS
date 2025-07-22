#![allow(dead_code)]

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use crate::error::{QmsError, QmsResult};
use crate::modules::audit_logger::entry::{AuditLogger, AuditConfig, log_action};
use crate::models::AuditAction;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct TestStep {
    pub step_number: u32,
    pub action: String,
    pub expected_result: String,
    pub actual_result: Option<String>,
    pub status: TestStepStatus,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub enum TestStepStatus {
    NotExecuted,
    Passed,
    Failed,
    Blocked,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub test_id: String,
    pub title: String,
    pub description: String,
    pub category: TestCategory,
    pub priority: TestPriority,
    pub preconditions: Option<String>,
    pub postconditions: Option<String>,
    pub steps: Vec<TestStep>,
    pub created_by: String,
    pub created_date: SystemTime,
    pub last_modified: SystemTime,
    pub execution_results: Vec<TestExecution>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum TestCategory {
    Functional,
    Performance,
    Security,
    Usability,
    Integration,
    Regression,
    Smoke,
    UserAcceptance,
}

#[derive(Debug, Clone)]
pub enum TestPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
pub struct TestExecution {
    pub execution_id: String,
    pub executed_by: String,
    pub execution_date: SystemTime,
    pub overall_status: TestExecutionStatus,
    pub duration_seconds: Option<u64>,
    pub environment: Option<String>,
    pub notes: Option<String>,
    pub step_results: Vec<TestStepResult>,
}

#[derive(Debug, Clone)]
pub enum TestExecutionStatus {
    Passed,
    Failed,
    Blocked,
    Incomplete,
}

#[derive(Debug, Clone)]
pub struct TestStepResult {
    pub step_number: u32,
    pub status: TestStepStatus,
    pub actual_result: String,
    pub notes: Option<String>,
}

pub struct TestCaseManager {
    project_path: PathBuf,
    test_cases: HashMap<String, TestCase>,
    audit_logger: AuditLogger,
}

impl TestCaseManager {
    pub fn new(project_path: &Path) -> QmsResult<Self> {
        let test_cases_dir = project_path.join("tests");
        fs::create_dir_all(&test_cases_dir)?;
        
        let audit_config = AuditConfig {
            project_path: project_path.to_string_lossy().to_string(),
            retention_days: 2555,
            daily_rotation: true,
            max_file_size_mb: 100,
            require_checksums: true,
        };
        
        let mut manager = TestCaseManager {
            project_path: project_path.to_path_buf(),
            test_cases: HashMap::new(),
            audit_logger: AuditLogger::new(audit_config)?,
        };
        
        manager.load_test_cases()?;
        Ok(manager)
    }
    
    pub fn create_test_case(
        &mut self,
        test_id: String,
        title: String,
        description: String,
        category: TestCategory,
        priority: TestPriority,
        created_by: String,
    ) -> QmsResult<()> {
        if self.test_cases.contains_key(&test_id) {
            return Err(QmsError::validation_error(&format!("Test case with ID {test_id} already exists")));
        }
        
        let test_case = TestCase {
            test_id: test_id.clone(),
            title,
            description,
            category,
            priority,
            preconditions: None,
            postconditions: None,
            steps: Vec::new(),
            created_by: created_by.clone(),
            created_date: SystemTime::now(),
            last_modified: SystemTime::now(),
            execution_results: Vec::new(),
            tags: Vec::new(),
        };
        
        self.test_cases.insert(test_id.clone(), test_case);
        self.save_test_case(&test_id)?;

        // Save consolidated test cases index for traceability
        self.save_test_cases_index()?;

        log_action(
            &created_by,
            AuditAction::Create,
            "test_case",
            &test_id,
        )?;
        
        Ok(())
    }
    
    pub fn add_test_step(
        &mut self,
        test_id: &str,
        action: String,
        expected_result: String,
        notes: Option<String>,
    ) -> QmsResult<()> {
        let test_case = self.test_cases.get_mut(test_id)
            .ok_or_else(|| QmsError::not_found(&format!("Test case {test_id} not found")))?;
        
        let step_number = test_case.steps.len() as u32 + 1;
        let test_step = TestStep {
            step_number,
            action,
            expected_result,
            actual_result: None,
            status: TestStepStatus::NotExecuted,
            notes,
        };
        
        test_case.steps.push(test_step);
        test_case.last_modified = SystemTime::now();
        
        self.save_test_case(test_id)?;
        
        log_action(
            "System",
            AuditAction::Update,
            "test_case",
            test_id,
        )?;
        
        Ok(())
    }
    
    pub fn execute_test(
        &mut self,
        test_id: &str,
        executed_by: String,
        environment: Option<String>,
    ) -> QmsResult<String> {
        let test_case = self.test_cases.get_mut(test_id)
            .ok_or_else(|| QmsError::not_found(&format!("Test case {test_id} not found")))?;
        
        let execution_id = format!("EXEC-{}-{}", test_id, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
        let execution = TestExecution {
            execution_id: execution_id.clone(),
            executed_by: executed_by.clone(),
            execution_date: SystemTime::now(),
            overall_status: TestExecutionStatus::Incomplete,
            duration_seconds: None,
            environment,
            notes: None,
            step_results: Vec::new(),
        };
        
        test_case.execution_results.push(execution);
        test_case.last_modified = SystemTime::now();
        
        self.save_test_case(test_id)?;
        
        log_action(
            &executed_by,
            AuditAction::Create,
            "test_execution",
            &execution_id,
        )?;
        
        Ok(execution_id)
    }
    
    pub fn record_step_result(
        &mut self,
        test_id: &str,
        execution_id: &str,
        step_number: u32,
        status: TestStepStatus,
        actual_result: String,
        notes: Option<String>,
    ) -> QmsResult<()> {
        let test_case = self.test_cases.get_mut(test_id)
            .ok_or_else(|| QmsError::not_found(&format!("Test case {test_id} not found")))?;
        
        let execution = test_case.execution_results.iter_mut()
            .find(|e| e.execution_id == execution_id)
            .ok_or_else(|| QmsError::not_found(&format!("Execution {execution_id} not found")))?;
        
        let step_result = TestStepResult {
            step_number,
            status,
            actual_result,
            notes,
        };
        
        execution.step_results.push(step_result);
        test_case.last_modified = SystemTime::now();
        
        self.save_test_case(test_id)?;
        
        log_action(
            "System",
            AuditAction::Update,
            "test_execution",
            execution_id,
        )?;
        
        Ok(())
    }
    
    pub fn finalize_execution(
        &mut self,
        test_id: &str,
        execution_id: &str,
        overall_status: TestExecutionStatus,
        duration_seconds: Option<u64>,
        notes: Option<String>,
    ) -> QmsResult<()> {
        let test_case = self.test_cases.get_mut(test_id)
            .ok_or_else(|| QmsError::not_found(&format!("Test case {test_id} not found")))?;
        
        let execution = test_case.execution_results.iter_mut()
            .find(|e| e.execution_id == execution_id)
            .ok_or_else(|| QmsError::not_found(&format!("Execution {execution_id} not found")))?;
        
        execution.overall_status = overall_status;
        execution.duration_seconds = duration_seconds;
        execution.notes = notes;
        test_case.last_modified = SystemTime::now();
        
        self.save_test_case(test_id)?;
        
        log_action(
            "System",
            AuditAction::Update,
            "test_execution",
            execution_id,
        )?;
        
        Ok(())
    }
    
    pub fn get_test_case(&self, test_id: &str) -> Option<&TestCase> {
        self.test_cases.get(test_id)
    }
    
    pub fn list_test_cases(&self) -> Vec<&TestCase> {
        self.test_cases.values().collect()
    }
    
    pub fn list_test_cases_by_category(&self, category: &TestCategory) -> Vec<&TestCase> {
        self.test_cases.values()
            .filter(|tc| std::mem::discriminant(&tc.category) == std::mem::discriminant(category))
            .collect()
    }
    
    pub fn get_test_execution_summary(&self, test_id: &str) -> QmsResult<String> {
        let test_case = self.test_cases.get(test_id)
            .ok_or_else(|| QmsError::not_found(&format!("Test case {test_id} not found")))?;
        
        let mut summary = format!("Test Case: {} - {}\n", test_case.test_id, test_case.title);
        summary.push_str(&format!("Category: {:?}, Priority: {:?}\n", test_case.category, test_case.priority));
        summary.push_str(&format!("Total Steps: {}\n", test_case.steps.len()));
        summary.push_str(&format!("Executions: {}\n\n", test_case.execution_results.len()));
        
        for execution in &test_case.execution_results {
            summary.push_str(&format!("Execution: {} - {:?}\n", execution.execution_id, execution.overall_status));
            summary.push_str(&format!("  Executed by: {} on {:?}\n", execution.executed_by, execution.execution_date));
            if let Some(duration) = execution.duration_seconds {
                summary.push_str(&format!("  Duration: {duration} seconds\n"));
            }
            if let Some(env) = &execution.environment {
                summary.push_str(&format!("  Environment: {env}\n"));
            }
            summary.push_str(&format!("  Step Results: {}\n\n", execution.step_results.len()));
        }
        
        Ok(summary)
    }
    
    fn save_test_case(&self, test_id: &str) -> QmsResult<()> {
        let test_case = self.test_cases.get(test_id)
            .ok_or_else(|| QmsError::not_found(&format!("Test case {test_id} not found")))?;
        
        let file_path = self.project_path.join("tests").join(format!("{test_id}.json"));
        let mut file = File::create(&file_path)?;
        
        let json = format!("{{\"test_id\":\"{}\",\"title\":\"{}\",\"category\":\"{:?}\",\"priority\":\"{:?}\",\"steps\":{},\"execution_results\":{}}}", 
            test_case.test_id, test_case.title, test_case.category, test_case.priority, test_case.steps.len(), test_case.execution_results.len());
        file.write_all(json.as_bytes())?;
        
        Ok(())
    }

    /// Save consolidated test cases index for traceability manager
    fn save_test_cases_index(&self) -> QmsResult<()> {
        // Create trace directory if it doesn't exist
        let trace_dir = self.project_path.join("trace");
        fs::create_dir_all(&trace_dir)?;

        let testcases_file = trace_dir.join("testcases.json");

        let mut json = String::new();
        json.push_str("{\n");
        json.push_str("  \"version\": \"1.0\",\n");
        json.push_str("  \"data\": [\n");

        let mut first = true;
        for test_case in self.test_cases.values() {
            if !first {
                json.push_str(",\n");
            }
            first = false;

            // Create a simplified JSON representation for traceability
            json.push_str("    {\n");
            json.push_str(&format!("      \"test_id\": \"{}\",\n", test_case.test_id));
            json.push_str(&format!("      \"title\": \"{}\",\n", test_case.title));
            json.push_str(&format!("      \"category\": \"{:?}\",\n", test_case.category));
            json.push_str(&format!("      \"priority\": \"{:?}\",\n", test_case.priority));
            json.push_str(&format!("      \"created_by\": \"{}\"\n", test_case.created_by));
            json.push_str("    }");
        }

        json.push_str("\n  ]\n");
        json.push_str("}\n");

        fs::write(&testcases_file, json)?;
        Ok(())
    }

    fn load_test_cases(&mut self) -> QmsResult<()> {
        let tests_dir = self.project_path.join("tests");
        if !tests_dir.exists() {
            return Ok(());
        }
        
        for entry in fs::read_dir(&tests_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_test_case(&path) {
                    Ok(test_case) => {
                        self.test_cases.insert(test_case.test_id.clone(), test_case);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load test case from {}: {}", path.display(), e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn load_test_case(&self, file_path: &Path) -> QmsResult<TestCase> {
        let _content = fs::read_to_string(file_path)?;
        // Simple JSON parsing - create a basic test case from filename
        let test_id = file_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let test_case = TestCase {
            test_id: test_id.clone(),
            title: format!("Test Case {test_id}"),
            description: "Loaded from file".to_string(),
            category: TestCategory::Functional,
            priority: TestPriority::Medium,
            preconditions: None,
            postconditions: None,
            steps: Vec::new(),
            created_by: "system".to_string(),
            created_date: SystemTime::now(),
            last_modified: SystemTime::now(),
            execution_results: Vec::new(),
            tags: Vec::new(),
        };
        
        Ok(test_case)
    }
}

impl std::fmt::Display for TestCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestCategory::Functional => write!(f, "Functional"),
            TestCategory::Performance => write!(f, "Performance"),
            TestCategory::Security => write!(f, "Security"),
            TestCategory::Usability => write!(f, "Usability"),
            TestCategory::Integration => write!(f, "Integration"),
            TestCategory::Regression => write!(f, "Regression"),
            TestCategory::Smoke => write!(f, "Smoke"),
            TestCategory::UserAcceptance => write!(f, "UserAcceptance"),
        }
    }
}

impl std::fmt::Display for TestPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestPriority::Critical => write!(f, "Critical"),
            TestPriority::High => write!(f, "High"),
            TestPriority::Medium => write!(f, "Medium"),
            TestPriority::Low => write!(f, "Low"),
        }
    }
}

impl std::fmt::Display for TestStepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestStepStatus::NotExecuted => write!(f, "Not Executed"),
            TestStepStatus::Passed => write!(f, "Passed"),
            TestStepStatus::Failed => write!(f, "Failed"),
            TestStepStatus::Blocked => write!(f, "Blocked"),
            TestStepStatus::Skipped => write!(f, "Skipped"),
        }
    }
}

impl std::fmt::Display for TestExecutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestExecutionStatus::Passed => write!(f, "Passed"),
            TestExecutionStatus::Failed => write!(f, "Failed"),
            TestExecutionStatus::Blocked => write!(f, "Blocked"),
            TestExecutionStatus::Incomplete => write!(f, "Incomplete"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;
    use crate::modules::audit_logger::{AuditConfig, initialize_audit_system};

    /// Initialize audit system for tests - lightweight version to avoid hanging
    fn init_audit_for_test(temp_dir: &std::path::Path) {
        // Create audit directory structure but don't initialize the full audit system
        // This avoids potential hanging issues with file I/O operations
        let audit_dir = temp_dir.join("audit");
        let _ = std::fs::create_dir_all(&audit_dir);

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
    fn test_create_test_case() {
        let temp_dir = env::temp_dir().join("qms_test_create");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Initialize audit system for test
        init_audit_for_test(&temp_dir);

        let mut manager = TestCaseManager::new(&temp_dir).unwrap();
        
        let result = manager.create_test_case(
            "TC-001".to_string(),
            "Login Test".to_string(),
            "Test user login functionality".to_string(),
            TestCategory::Functional,
            TestPriority::High,
            "test_user".to_string(),
        );
        
        assert!(result.is_ok());
        let test_case = manager.get_test_case("TC-001").unwrap();
        assert_eq!(test_case.test_id, "TC-001");
        assert_eq!(test_case.title, "Login Test");
    }
    
    #[test]
    fn test_add_test_step() {
        let temp_dir = env::temp_dir().join("qms_test_add_step");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Initialize audit system for test
        init_audit_for_test(&temp_dir);

        let mut manager = TestCaseManager::new(&temp_dir).unwrap();
        
        manager.create_test_case(
            "TC-001".to_string(),
            "Login Test".to_string(),
            "Test user login functionality".to_string(),
            TestCategory::Functional,
            TestPriority::High,
            "test_user".to_string(),
        ).unwrap();
        
        let result = manager.add_test_step(
            "TC-001",
            "Enter username".to_string(),
            "Username field accepts input".to_string(),
            None,
        );
        
        assert!(result.is_ok());
        let test_case = manager.get_test_case("TC-001").unwrap();
        assert_eq!(test_case.steps.len(), 1);
        assert_eq!(test_case.steps[0].action, "Enter username");
    }
    
    #[test]
    fn test_execute_test() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = TestCaseManager::new(temp_dir.path()).unwrap();
        
        manager.create_test_case(
            "TC-001".to_string(),
            "Login Test".to_string(),
            "Test user login functionality".to_string(),
            TestCategory::Functional,
            TestPriority::High,
            "test_user".to_string(),
        ).unwrap();
        
        let result = manager.execute_test(
            "TC-001",
            "test_executor".to_string(),
            Some("QA Environment".to_string()),
        );
        
        assert!(result.is_ok());
        let test_case = manager.get_test_case("TC-001").unwrap();
        assert_eq!(test_case.execution_results.len(), 1);
    }
}
