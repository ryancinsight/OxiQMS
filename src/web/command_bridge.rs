// WebCommandBridge - Bridge pattern for CLI-Web integration
// Follows SOLID, CUPID, GRASP, ACID, KISS, DRY, SOC, and YAGNI principles

use crate::prelude::*;
use crate::commands::cli_auth_helper::{get_cli_session, require_cli_authentication};
use crate::modules::user_manager::UserSession;
use crate::web::{HttpRequest, HttpResponse};
use crate::web::response::HttpStatus;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Command execution context for web requests
/// Encapsulates authentication and project information
#[derive(Debug, Clone)]
pub struct WebCommandContext {
    pub session: UserSession,
    pub project_path: PathBuf,
    pub request_id: String,
    pub client_info: ClientInfo,
}

/// Client information extracted from HTTP request
#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_method: String,
    pub request_path: String,
}

impl ClientInfo {
    pub fn from_request(request: &HttpRequest) -> Self {
        Self {
            ip_address: request.get_header("x-forwarded-for")
                .or_else(|| request.get_header("x-real-ip"))
                .map(|s| s.to_string()),
            user_agent: request.get_header("user-agent").map(|s| s.to_string()),
            request_method: request.method.clone(),
            request_path: request.path().to_string(),
        }
    }
}

/// Simple JSON value representation
#[derive(Debug, Clone)]
pub enum JsonValue {
    String(String),
    Number(f64),
    Bool(bool),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
    Null,
}

impl JsonValue {
    pub fn to_string(&self) -> String {
        match self {
            JsonValue::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            JsonValue::Number(n) => n.to_string(),
            JsonValue::Bool(b) => b.to_string(),
            JsonValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                format!("[{}]", items.join(","))
            }
            JsonValue::Object(obj) => {
                let items: Vec<String> = obj.iter()
                    .map(|(k, v)| format!("\"{}\":{}", k, v.to_string()))
                    .collect();
                format!("{{{}}}", items.join(","))
            }
            JsonValue::Null => "null".to_string(),
        }
    }
}

/// Command execution result with structured data
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub data: JsonValue,
    pub message: String,
    pub metadata: HashMap<String, JsonValue>,
}

impl CommandResult {
    pub fn success(data: JsonValue) -> QmsResult<Self> {
        Ok(Self {
            success: true,
            data,
            message: "Command executed successfully".to_string(),
            metadata: HashMap::new(),
        })
    }

    pub fn success_with_message(data: JsonValue, message: String) -> QmsResult<Self> {
        Ok(Self {
            success: true,
            data,
            message,
            metadata: HashMap::new(),
        })
    }

    pub fn error(message: String) -> QmsResult<Self> {
        Ok(Self {
            success: false,
            data: JsonValue::Null,
            message,
            metadata: HashMap::new(),
        })
    }

    pub fn with_metadata(mut self, key: String, value: JsonValue) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Convert to HTTP response
    pub fn to_http_response(&self) -> HttpResponse {
        let mut json_obj = HashMap::new();
        json_obj.insert("success".to_string(), JsonValue::Bool(self.success));
        json_obj.insert("data".to_string(), self.data.clone());
        json_obj.insert("message".to_string(), JsonValue::String(self.message.clone()));
        json_obj.insert("metadata".to_string(), JsonValue::Object(self.metadata.clone()));

        let json = JsonValue::Object(json_obj);

        if self.success {
            HttpResponse::json(&json.to_string())
        } else {
            HttpResponse::new_with_body(HttpStatus::BadRequest, json.to_string())
        }
    }
}

/// Command argument parser for web requests
/// Converts HTTP request parameters to CLI-style arguments
pub struct CommandArgumentParser;

impl CommandArgumentParser {
    /// Parse query parameters into CLI arguments
    pub fn parse_query_params(query: &str) -> Vec<String> {
        let mut args = Vec::new();
        
        for param in query.split('&') {
            if let Some((key, value)) = param.split_once('=') {
                let decoded_key = urlencoding::decode(key).unwrap_or_else(|_| key.into());
                let decoded_value = urlencoding::decode(value).unwrap_or_else(|_| value.into());
                
                // Convert to CLI flag format
                if decoded_key.len() == 1 {
                    args.push(format!("-{}", decoded_key));
                } else {
                    args.push(format!("--{}", decoded_key));
                }
                args.push(decoded_value.to_string());
            }
        }
        
        args
    }
    
    /// Parse JSON body into CLI arguments (simplified implementation)
    pub fn parse_json_body(body: &str) -> QmsResult<Vec<String>> {
        // For now, return empty args - we'll enhance this later
        // In a full implementation, we'd parse JSON and convert to CLI args
        Ok(Vec::new())
    }
}

/// Main WebCommandBridge - orchestrates CLI command execution from web context
/// Implements Bridge Pattern, Command Pattern, and Adapter Pattern
pub struct WebCommandBridge {
    response_adapters: crate::web::json_response_adapters::ResponseAdapterRegistry,
}

impl WebCommandBridge {
    /// Create new WebCommandBridge
    pub fn new() -> Self {
        Self {
            response_adapters: crate::web::json_response_adapters::ResponseAdapterRegistry::new(),
        }
    }
    
    /// Create command context from HTTP request
    /// Handles authentication and project resolution
    pub fn create_context(&self, request: &HttpRequest) -> QmsResult<WebCommandContext> {
        // Use unified authentication context
        let auth_context = crate::web::unified_auth_context::UnifiedAuthContext::from_web_request(request)?;

        // Generate request ID for tracking
        let uuid_str = uuid::Uuid::new_v4().to_string();
        let request_id = format!("web_{}", &uuid_str[..8]);

        // Extract client information
        let client_info = ClientInfo::from_request(request);

        // Convert to WebCommandContext
        Ok(auth_context.to_web_command_context(request_id, client_info))
    }
    
    /// Execute CLI command in web context
    /// This is the main bridge method that delegates to CLI handlers
    pub fn execute_command(
        &self,
        context: &WebCommandContext,
        command: &str,
        subcommand: &str,
        args: Vec<String>,
    ) -> QmsResult<CommandResult> {
        // Build full CLI arguments
        let mut full_args = vec![
            "qms".to_string(),
            command.to_string(),
            subcommand.to_string(),
        ];
        full_args.extend(args);
        
        // Set up environment for CLI command execution
        self.setup_command_environment(context)?;
        
        // Execute the appropriate CLI command handler
        match command {
            "doc" => self.execute_doc_command(context, &full_args),
            "risk" => self.execute_risk_command(context, &full_args),
            "req" => self.execute_req_command(context, &full_args),
            "audit" => self.execute_audit_command(context, &full_args),
            "report" => self.execute_report_command(context, &full_args),
            _ => Err(QmsError::validation_error(&format!("Unknown command: {}", command))),
        }
    }
    

    
    /// Set up environment for CLI command execution
    fn setup_command_environment(&self, context: &WebCommandContext) -> QmsResult<()> {
        // Set current directory to project path if it exists
        if context.project_path.exists() {
            std::env::set_current_dir(&context.project_path)
                .map_err(|e| QmsError::io_error(&format!("Failed to set working directory: {}", e)))?;
        }
        
        Ok(())
    }

    /// Execute document command and convert output to JSON
    fn execute_doc_command(&self, context: &WebCommandContext, args: &[String]) -> QmsResult<CommandResult> {
        // Create unified execution context for web mode
        let mut exec_context = crate::commands::command_execution_context::CommandExecutionContext::web(
            context.session.clone(),
            context.project_path.clone()
        );

        // Execute using unified document handler
        match crate::commands::unified_doc_handler::UnifiedDocumentHandler::execute_with_context(&mut exec_context, args) {
            Ok(()) => {
                // Extract subcommand for response adaptation
                let subcommand = if args.len() >= 3 { &args[2] } else { "unknown" };

                // Use response adapter to convert output to structured JSON
                let adapted_response = self.response_adapters.adapt_response(
                    &exec_context.output_capture,
                    "doc",
                    subcommand
                )?;

                // Add metadata
                let mut final_response = HashMap::new();
                final_response.insert("user".to_string(), JsonValue::String(context.session.username.clone()));
                final_response.insert("executed_at".to_string(), JsonValue::String(self.get_timestamp()));
                final_response.insert("data".to_string(), self.convert_web_json_to_json_value(adapted_response));

                CommandResult::success(JsonValue::Object(final_response))
            }
            Err(e) => Err(QmsError::domain_error(&format!("Document command failed: {}", e))),
        }
    }

    /// Execute risk command and convert output to JSON
    fn execute_risk_command(&self, context: &WebCommandContext, args: &[String]) -> QmsResult<CommandResult> {
        match crate::commands::risk::handle_risk_command(args) {
            Ok(()) => {
                let mut data = HashMap::new();
                data.insert("command".to_string(), JsonValue::String("risk".to_string()));
                data.insert("user".to_string(), JsonValue::String(context.session.username.clone()));
                data.insert("executed_at".to_string(), JsonValue::String(self.get_timestamp()));

                CommandResult::success(JsonValue::Object(data))
            }
            Err(e) => Err(QmsError::domain_error(&format!("Risk command failed: {}", e))),
        }
    }

    /// Execute requirements command and convert output to JSON
    fn execute_req_command(&self, context: &WebCommandContext, args: &[String]) -> QmsResult<CommandResult> {
        match crate::commands::req::handle_req_command(args) {
            Ok(()) => {
                let mut data = HashMap::new();
                data.insert("command".to_string(), JsonValue::String("req".to_string()));
                data.insert("user".to_string(), JsonValue::String(context.session.username.clone()));
                data.insert("executed_at".to_string(), JsonValue::String(self.get_timestamp()));

                CommandResult::success(JsonValue::Object(data))
            }
            Err(e) => Err(QmsError::domain_error(&format!("Requirements command failed: {}", e))),
        }
    }

    /// Execute audit command and convert output to JSON
    fn execute_audit_command(&self, context: &WebCommandContext, args: &[String]) -> QmsResult<CommandResult> {
        match crate::commands::audit::handle_audit_command(args) {
            Ok(()) => {
                let mut data = HashMap::new();
                data.insert("command".to_string(), JsonValue::String("audit".to_string()));
                data.insert("user".to_string(), JsonValue::String(context.session.username.clone()));
                data.insert("executed_at".to_string(), JsonValue::String(self.get_timestamp()));

                CommandResult::success(JsonValue::Object(data))
            }
            Err(e) => Err(QmsError::domain_error(&format!("Audit command failed: {}", e))),
        }
    }

    /// Execute report command and convert output to JSON
    fn execute_report_command(&self, context: &WebCommandContext, args: &[String]) -> QmsResult<CommandResult> {
        match crate::commands::report::handle_report_command(args) {
            Ok(()) => {
                let mut data = HashMap::new();
                data.insert("command".to_string(), JsonValue::String("report".to_string()));
                data.insert("user".to_string(), JsonValue::String(context.session.username.clone()));
                data.insert("executed_at".to_string(), JsonValue::String(self.get_timestamp()));

                CommandResult::success(JsonValue::Object(data))
            }
            Err(e) => Err(QmsError::domain_error(&format!("Report command failed: {}", e))),
        }
    }

    /// Get current timestamp as string
    fn get_timestamp(&self) -> String {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_secs().to_string(),
            Err(_) => "unknown".to_string(),
        }
    }

    /// Convert serde_json::Value to JsonValue
    fn convert_serde_json_to_json_value(&self, value: crate::commands::command_execution_context::serde_json::Value) -> JsonValue {
        match value {
            crate::commands::command_execution_context::serde_json::Value::String(s) => JsonValue::String(s),
            crate::commands::command_execution_context::serde_json::Value::Number(n) => JsonValue::Number(n),
            crate::commands::command_execution_context::serde_json::Value::Bool(b) => JsonValue::Bool(b),
            crate::commands::command_execution_context::serde_json::Value::Array(arr) => {
                let converted: Vec<JsonValue> = arr.into_iter()
                    .map(|v| self.convert_serde_json_to_json_value(v))
                    .collect();
                JsonValue::Array(converted)
            }
            crate::commands::command_execution_context::serde_json::Value::Object(obj) => {
                let converted: HashMap<String, JsonValue> = obj.into_iter()
                    .map(|(k, v)| (k, self.convert_serde_json_to_json_value(v)))
                    .collect();
                JsonValue::Object(converted)
            }
            crate::commands::command_execution_context::serde_json::Value::Null => JsonValue::Null,
        }
    }

    /// Convert WebJsonValue to JsonValue
    fn convert_web_json_to_json_value(&self, value: JsonValue) -> JsonValue {
        // Since both are the same type, this is a no-op, but kept for consistency
        value
    }
}

impl Default for WebCommandBridge {
    fn default() -> Self {
        Self::new()
    }
}

// Module for URL encoding/decoding (simple implementation)
mod urlencoding {
    use std::borrow::Cow;
    
    pub fn decode(input: &str) -> Result<Cow<str>, ()> {
        // Simple URL decoding - replace %20 with space, etc.
        let decoded = input.replace("%20", " ")
            .replace("%21", "!")
            .replace("%22", "\"")
            .replace("%23", "#")
            .replace("%24", "$")
            .replace("%25", "%")
            .replace("%26", "&")
            .replace("%27", "'")
            .replace("%28", "(")
            .replace("%29", ")")
            .replace("%2A", "*")
            .replace("%2B", "+")
            .replace("%2C", ",")
            .replace("%2D", "-")
            .replace("%2E", ".")
            .replace("%2F", "/");
        
        Ok(Cow::Owned(decoded))
    }
}

// Simple UUID generation (avoiding external dependencies)
mod uuid {
    pub struct Uuid;
    
    impl Uuid {
        pub fn new_v4() -> Self {
            Self
        }
        
        pub fn to_string(&self) -> String {
            use std::time::{SystemTime, UNIX_EPOCH};
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            format!("{:x}", timestamp)
        }
    }
}
