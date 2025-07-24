// JSON Response Adapters - Convert CLI outputs to structured JSON for web API consumption
// Follows SOLID, CUPID, GRASP, ACID, KISS, DRY, SOC, and YAGNI principles

use crate::prelude::*;
use crate::commands::command_execution_context::{OutputCapture, serde_json::Value as JsonValue};
use crate::web::command_bridge::JsonValue as WebJsonValue;
use std::collections::HashMap;

/// Response adapter trait for converting CLI outputs to web-friendly JSON
pub trait ResponseAdapter: Send + Sync {
    /// Convert CLI output to structured JSON response
    fn adapt_response(&self, output: &OutputCapture, command: &str, subcommand: &str) -> QmsResult<WebJsonValue>;
    
    /// Get supported command
    fn supported_command(&self) -> &'static str;
}

/// Document response adapter - converts document command outputs to JSON
pub struct DocumentResponseAdapter;

impl DocumentResponseAdapter {
    pub fn new() -> Self {
        Self
    }
    
    /// Adapt document list response
    fn adapt_list_response(&self, output: &OutputCapture) -> QmsResult<WebJsonValue> {
        let mut response = HashMap::new();
        
        // Extract documents from structured data
        if let Some(JsonValue::Array(docs)) = output.data.get("documents") {
            let web_docs: Vec<WebJsonValue> = docs.iter()
                .map(|doc| self.convert_json_value(doc.clone()))
                .collect();
            response.insert("documents".to_string(), WebJsonValue::Array(web_docs));
        } else {
            response.insert("documents".to_string(), WebJsonValue::Array(Vec::new()));
        }
        
        // Extract count
        if let Some(JsonValue::Number(count)) = output.data.get("count") {
            response.insert("count".to_string(), WebJsonValue::Number(*count));
        } else {
            response.insert("count".to_string(), WebJsonValue::Number(0.0));
        }
        
        // Add output messages
        if !output.stdout.is_empty() {
            let messages: Vec<WebJsonValue> = output.stdout.iter()
                .map(|msg| WebJsonValue::String(msg.clone()))
                .collect();
            response.insert("messages".to_string(), WebJsonValue::Array(messages));
        }
        
        Ok(WebJsonValue::Object(response))
    }
    
    /// Adapt document view response
    fn adapt_view_response(&self, output: &OutputCapture) -> QmsResult<WebJsonValue> {
        let mut response = HashMap::new();
        
        // Extract document from structured data
        if let Some(doc_data) = output.data.get("document") {
            response.insert("document".to_string(), self.convert_json_value(doc_data.clone()));
        }
        
        // Add output messages for CLI compatibility
        if !output.stdout.is_empty() {
            let messages: Vec<WebJsonValue> = output.stdout.iter()
                .map(|msg| WebJsonValue::String(msg.clone()))
                .collect();
            response.insert("messages".to_string(), WebJsonValue::Array(messages));
        }
        
        Ok(WebJsonValue::Object(response))
    }
    
    /// Adapt document create response
    fn adapt_create_response(&self, output: &OutputCapture) -> QmsResult<WebJsonValue> {
        let mut response = HashMap::new();
        
        // Extract created document from structured data
        if let Some(doc_data) = output.data.get("document") {
            response.insert("document".to_string(), self.convert_json_value(doc_data.clone()));
        }
        
        // Add success message
        response.insert("success".to_string(), WebJsonValue::Bool(true));
        response.insert("message".to_string(), WebJsonValue::String("Document created successfully".to_string()));
        
        // Add CLI output for debugging
        if !output.stdout.is_empty() {
            let messages: Vec<WebJsonValue> = output.stdout.iter()
                .map(|msg| WebJsonValue::String(msg.clone()))
                .collect();
            response.insert("cli_output".to_string(), WebJsonValue::Array(messages));
        }
        
        Ok(WebJsonValue::Object(response))
    }
    
    /// Convert JsonValue to WebJsonValue
    fn convert_json_value(&self, value: JsonValue) -> WebJsonValue {
        match value {
            JsonValue::String(s) => WebJsonValue::String(s),
            JsonValue::Number(n) => WebJsonValue::Number(n),
            JsonValue::Bool(b) => WebJsonValue::Bool(b),
            JsonValue::Array(arr) => {
                let web_arr: Vec<WebJsonValue> = arr.into_iter()
                    .map(|v| self.convert_json_value(v))
                    .collect();
                WebJsonValue::Array(web_arr)
            }
            JsonValue::Object(obj) => {
                let web_obj: HashMap<String, WebJsonValue> = obj.into_iter()
                    .map(|(k, v)| (k, self.convert_json_value(v)))
                    .collect();
                WebJsonValue::Object(web_obj)
            }
            JsonValue::Null => WebJsonValue::Null,
        }
    }
}

impl ResponseAdapter for DocumentResponseAdapter {
    fn adapt_response(&self, output: &OutputCapture, _command: &str, subcommand: &str) -> QmsResult<WebJsonValue> {
        match subcommand {
            "list" => self.adapt_list_response(output),
            "view" => self.adapt_view_response(output),
            "add" => self.adapt_create_response(output),
            _ => {
                // Generic response for other subcommands
                let mut response = HashMap::new();
                response.insert("success".to_string(), WebJsonValue::Bool(true));
                response.insert("subcommand".to_string(), WebJsonValue::String(subcommand.to_string()));
                
                if !output.stdout.is_empty() {
                    let messages: Vec<WebJsonValue> = output.stdout.iter()
                        .map(|msg| WebJsonValue::String(msg.clone()))
                        .collect();
                    response.insert("output".to_string(), WebJsonValue::Array(messages));
                }
                
                // Include any structured data
                for (key, value) in &output.data {
                    response.insert(key.clone(), self.convert_json_value(value.clone()));
                }
                
                Ok(WebJsonValue::Object(response))
            }
        }
    }
    
    fn supported_command(&self) -> &'static str {
        "doc"
    }
}

/// Risk response adapter - converts risk command outputs to JSON
pub struct RiskResponseAdapter;

impl RiskResponseAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl ResponseAdapter for RiskResponseAdapter {
    fn adapt_response(&self, output: &OutputCapture, _command: &str, subcommand: &str) -> QmsResult<WebJsonValue> {
        let mut response = HashMap::new();
        response.insert("success".to_string(), WebJsonValue::Bool(true));
        response.insert("command".to_string(), WebJsonValue::String("risk".to_string()));
        response.insert("subcommand".to_string(), WebJsonValue::String(subcommand.to_string()));
        
        // Add CLI output
        if !output.stdout.is_empty() {
            let messages: Vec<WebJsonValue> = output.stdout.iter()
                .map(|msg| WebJsonValue::String(msg.clone()))
                .collect();
            response.insert("output".to_string(), WebJsonValue::Array(messages));
        }
        
        // Include any structured data (placeholder for future risk-specific data)
        for (key, value) in &output.data {
            response.insert(key.clone(), self.convert_json_value(value.clone()));
        }
        
        Ok(WebJsonValue::Object(response))
    }
    
    fn supported_command(&self) -> &'static str {
        "risk"
    }
}

impl RiskResponseAdapter {
    fn convert_json_value(&self, value: JsonValue) -> WebJsonValue {
        match value {
            JsonValue::String(s) => WebJsonValue::String(s),
            JsonValue::Number(n) => WebJsonValue::Number(n),
            JsonValue::Bool(b) => WebJsonValue::Bool(b),
            JsonValue::Array(arr) => {
                let web_arr: Vec<WebJsonValue> = arr.into_iter()
                    .map(|v| self.convert_json_value(v))
                    .collect();
                WebJsonValue::Array(web_arr)
            }
            JsonValue::Object(obj) => {
                let web_obj: HashMap<String, WebJsonValue> = obj.into_iter()
                    .map(|(k, v)| (k, self.convert_json_value(v)))
                    .collect();
                WebJsonValue::Object(web_obj)
            }
            JsonValue::Null => WebJsonValue::Null,
        }
    }
}

/// Requirements response adapter - converts requirements command outputs to JSON
pub struct RequirementsResponseAdapter;

impl RequirementsResponseAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl ResponseAdapter for RequirementsResponseAdapter {
    fn adapt_response(&self, output: &OutputCapture, _command: &str, subcommand: &str) -> QmsResult<WebJsonValue> {
        let mut response = HashMap::new();
        response.insert("success".to_string(), WebJsonValue::Bool(true));
        response.insert("command".to_string(), WebJsonValue::String("req".to_string()));
        response.insert("subcommand".to_string(), WebJsonValue::String(subcommand.to_string()));
        
        // Add CLI output
        if !output.stdout.is_empty() {
            let messages: Vec<WebJsonValue> = output.stdout.iter()
                .map(|msg| WebJsonValue::String(msg.clone()))
                .collect();
            response.insert("output".to_string(), WebJsonValue::Array(messages));
        }
        
        Ok(WebJsonValue::Object(response))
    }
    
    fn supported_command(&self) -> &'static str {
        "req"
    }
}

/// Audit response adapter - converts audit command outputs to JSON
pub struct AuditResponseAdapter;

impl AuditResponseAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl ResponseAdapter for AuditResponseAdapter {
    fn adapt_response(&self, output: &OutputCapture, _command: &str, subcommand: &str) -> QmsResult<WebJsonValue> {
        let mut response = HashMap::new();
        response.insert("success".to_string(), WebJsonValue::Bool(true));
        response.insert("command".to_string(), WebJsonValue::String("audit".to_string()));
        response.insert("subcommand".to_string(), WebJsonValue::String(subcommand.to_string()));
        
        // Add CLI output
        if !output.stdout.is_empty() {
            let messages: Vec<WebJsonValue> = output.stdout.iter()
                .map(|msg| WebJsonValue::String(msg.clone()))
                .collect();
            response.insert("output".to_string(), WebJsonValue::Array(messages));
        }
        
        Ok(WebJsonValue::Object(response))
    }
    
    fn supported_command(&self) -> &'static str {
        "audit"
    }
}

/// Response adapter registry - manages all response adapters
pub struct ResponseAdapterRegistry {
    adapters: HashMap<String, Box<dyn ResponseAdapter>>,
}

impl ResponseAdapterRegistry {
    /// Create new registry with default adapters
    pub fn new() -> Self {
        let mut registry = Self {
            adapters: HashMap::new(),
        };
        
        // Register default adapters
        registry.register(Box::new(DocumentResponseAdapter::new()));
        registry.register(Box::new(RiskResponseAdapter::new()));
        registry.register(Box::new(RequirementsResponseAdapter::new()));
        registry.register(Box::new(AuditResponseAdapter::new()));
        
        registry
    }
    
    /// Register a response adapter
    pub fn register(&mut self, adapter: Box<dyn ResponseAdapter>) {
        let command = adapter.supported_command().to_string();
        self.adapters.insert(command, adapter);
    }
    
    /// Get adapter for command
    pub fn get_adapter(&self, command: &str) -> Option<&dyn ResponseAdapter> {
        self.adapters.get(command).map(|adapter| adapter.as_ref())
    }
    
    /// Adapt response using appropriate adapter
    pub fn adapt_response(&self, output: &OutputCapture, command: &str, subcommand: &str) -> QmsResult<WebJsonValue> {
        if let Some(adapter) = self.get_adapter(command) {
            adapter.adapt_response(output, command, subcommand)
        } else {
            // Generic fallback adapter
            let mut response = HashMap::new();
            response.insert("success".to_string(), WebJsonValue::Bool(true));
            response.insert("command".to_string(), WebJsonValue::String(command.to_string()));
            response.insert("subcommand".to_string(), WebJsonValue::String(subcommand.to_string()));
            
            if !output.stdout.is_empty() {
                let messages: Vec<WebJsonValue> = output.stdout.iter()
                    .map(|msg| WebJsonValue::String(msg.clone()))
                    .collect();
                response.insert("output".to_string(), WebJsonValue::Array(messages));
            }
            
            Ok(WebJsonValue::Object(response))
        }
    }
}

impl Default for ResponseAdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}
