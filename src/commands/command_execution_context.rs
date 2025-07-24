// Command Execution Context - Unified context for CLI and Web command execution
// Follows SOLID, CUPID, GRASP, ACID, KISS, DRY, SOC, and YAGNI principles

use crate::prelude::*;
use crate::modules::user_manager::UserSession;
use std::collections::HashMap;
use std::path::PathBuf;
use std::io::{self, Write};

/// Execution mode for commands
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionMode {
    /// CLI mode - output to stdout/stderr, can exit process
    Cli,
    /// Web mode - capture output, return structured data
    Web,
    /// Test mode - capture output, no side effects
    Test,
}

/// Output capture for non-CLI execution modes
#[derive(Debug, Clone, Default)]
pub struct OutputCapture {
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
    pub data: HashMap<String, serde_json::Value>,
}

impl OutputCapture {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn add_stdout(&mut self, message: String) {
        self.stdout.push(message);
    }
    
    pub fn add_stderr(&mut self, message: String) {
        self.stderr.push(message);
    }
    
    pub fn add_data(&mut self, key: String, value: serde_json::Value) {
        self.data.insert(key, value);
    }
    
    pub fn get_combined_output(&self) -> String {
        let mut output = String::new();
        for line in &self.stdout {
            output.push_str(line);
            output.push('\n');
        }
        for line in &self.stderr {
            output.push_str("ERROR: ");
            output.push_str(line);
            output.push('\n');
        }
        output
    }
}

/// Command execution context - provides unified interface for CLI and Web execution
#[derive(Debug, Clone)]
pub struct CommandExecutionContext {
    pub mode: ExecutionMode,
    pub session: Option<UserSession>,
    pub project_path: Option<PathBuf>,
    pub working_directory: PathBuf,
    pub output_capture: OutputCapture,
    pub metadata: HashMap<String, String>,
}

impl CommandExecutionContext {
    /// Create new CLI execution context
    pub fn cli() -> Self {
        Self {
            mode: ExecutionMode::Cli,
            session: None,
            project_path: None,
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            output_capture: OutputCapture::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Create new web execution context with session
    pub fn web(session: UserSession, project_path: PathBuf) -> Self {
        Self {
            mode: ExecutionMode::Web,
            session: Some(session),
            project_path: Some(project_path.clone()),
            working_directory: project_path,
            output_capture: OutputCapture::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Create new test execution context
    pub fn test() -> Self {
        Self {
            mode: ExecutionMode::Test,
            session: None,
            project_path: None,
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            output_capture: OutputCapture::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Print message to stdout (CLI) or capture (Web/Test)
    pub fn println(&mut self, message: &str) {
        match self.mode {
            ExecutionMode::Cli => println!("{}", message),
            ExecutionMode::Web | ExecutionMode::Test => {
                self.output_capture.add_stdout(message.to_string());
            }
        }
    }
    
    /// Print message to stderr (CLI) or capture (Web/Test)
    pub fn eprintln(&mut self, message: &str) {
        match self.mode {
            ExecutionMode::Cli => eprintln!("{}", message),
            ExecutionMode::Web | ExecutionMode::Test => {
                self.output_capture.add_stderr(message.to_string());
            }
        }
    }
    
    /// Print formatted message to stdout
    pub fn print_formatted(&mut self, message: &str) {
        self.println(message);
    }
    
    /// Exit with error code (CLI) or return error (Web/Test)
    pub fn exit_with_error(&mut self, message: &str, code: i32) -> QmsResult<()> {
        match self.mode {
            ExecutionMode::Cli => {
                eprintln!("Error: {}", message);
                std::process::exit(code);
            }
            ExecutionMode::Web | ExecutionMode::Test => {
                self.eprintln(message);
                Err(QmsError::domain_error(message))
            }
        }
    }
    
    /// Add structured data (for Web/Test modes)
    pub fn add_data(&mut self, key: &str, value: serde_json::Value) {
        self.output_capture.add_data(key.to_string(), value);
    }
    
    /// Get current user session
    pub fn get_session(&self) -> QmsResult<&UserSession> {
        self.session.as_ref()
            .ok_or_else(|| QmsError::Authentication("No user session available".to_string()))
    }
    
    /// Get project path
    pub fn get_project_path(&self) -> QmsResult<&PathBuf> {
        self.project_path.as_ref()
            .ok_or_else(|| QmsError::NotFound("No project path available".to_string()))
    }
    
    /// Check if running in CLI mode
    pub fn is_cli_mode(&self) -> bool {
        self.mode == ExecutionMode::Cli
    }
    
    /// Check if running in web mode
    pub fn is_web_mode(&self) -> bool {
        self.mode == ExecutionMode::Web
    }
    
    /// Check if running in test mode
    pub fn is_test_mode(&self) -> bool {
        self.mode == ExecutionMode::Test
    }
    
    /// Set metadata
    pub fn set_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
    
    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// Command execution result - unified result type for CLI and Web
#[derive(Debug, Clone)]
pub struct CommandExecutionResult {
    pub success: bool,
    pub output: OutputCapture,
    pub error_message: Option<String>,
    pub exit_code: i32,
    pub metadata: HashMap<String, String>,
}

impl CommandExecutionResult {
    /// Create successful result
    pub fn success(output: OutputCapture) -> Self {
        Self {
            success: true,
            output,
            error_message: None,
            exit_code: 0,
            metadata: HashMap::new(),
        }
    }
    
    /// Create error result
    pub fn error(message: String, exit_code: i32) -> Self {
        let mut output = OutputCapture::new();
        output.add_stderr(message.clone());
        
        Self {
            success: false,
            output,
            error_message: Some(message),
            exit_code,
            metadata: HashMap::new(),
        }
    }
    
    /// Create result from context
    pub fn from_context(context: CommandExecutionContext, success: bool) -> Self {
        Self {
            success,
            output: context.output_capture,
            error_message: if success { None } else { Some("Command failed".to_string()) },
            exit_code: if success { 0 } else { 1 },
            metadata: context.metadata,
        }
    }
    
    /// Convert to QmsResult for error propagation
    pub fn to_qms_result(self) -> QmsResult<OutputCapture> {
        if self.success {
            Ok(self.output)
        } else {
            Err(QmsError::domain_error(&self.error_message.unwrap_or_else(|| "Command failed".to_string())))
        }
    }
}

/// Trait for commands that can be executed in both CLI and Web contexts
pub trait UnifiedCommandHandler {
    /// Execute command with unified context
    fn execute_unified(&self, context: &mut CommandExecutionContext, args: &[String]) -> QmsResult<()>;
    
    /// Get command name
    fn command_name(&self) -> &'static str;
    
    /// Get help text
    fn help_text(&self) -> &'static str;
}

/// Macro to create unified command handlers that work in both CLI and Web contexts
#[macro_export]
macro_rules! unified_command_handler {
    ($handler_name:ident, $command:expr, $help:expr, $execute_fn:expr) => {
        pub struct $handler_name;
        
        impl UnifiedCommandHandler for $handler_name {
            fn execute_unified(&self, context: &mut CommandExecutionContext, args: &[String]) -> QmsResult<()> {
                $execute_fn(context, args)
            }
            
            fn command_name(&self) -> &'static str {
                $command
            }
            
            fn help_text(&self) -> &'static str {
                $help
            }
        }
    };
}

// Simple JSON implementation to avoid external dependencies
pub mod serde_json {
    use std::collections::HashMap;
    
    #[derive(Debug, Clone)]
    pub enum Value {
        String(String),
        Number(f64),
        Bool(bool),
        Array(Vec<Value>),
        Object(HashMap<String, Value>),
        Null,
    }
    
    impl Value {
        pub fn to_string(&self) -> String {
            match self {
                Value::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Array(arr) => {
                    let items: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                    format!("[{}]", items.join(","))
                }
                Value::Object(obj) => {
                    let items: Vec<String> = obj.iter()
                        .map(|(k, v)| format!("\"{}\":{}", k, v.to_string()))
                        .collect();
                    format!("{{{}}}", items.join(","))
                }
                Value::Null => "null".to_string(),
            }
        }
    }
}
