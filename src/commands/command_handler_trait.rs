/// SOLID Principles Enhancement: Command Handler Trait
/// 
/// This module implements the Command Pattern with SOLID principles:
/// - Single Responsibility: Each handler focuses on one command type
/// - Open/Closed: New handlers can be added without modifying existing code
/// - Interface Segregation: Small, focused interfaces
/// - Dependency Inversion: Handlers depend on abstractions

use crate::prelude::*;

/// Command Handler trait following Interface Segregation Principle
/// Each command type gets its own focused interface
pub trait CommandHandler: Send + Sync {
    /// Handle a specific command with arguments
    fn handle(&self, args: &[String]) -> QmsResult<()>;
    
    /// Get command name for routing
    fn command_name(&self) -> &'static str;
    
    /// Get help text for this command
    fn help_text(&self) -> &'static str;
    
    /// Validate command arguments before execution
    fn validate_args(&self, args: &[String]) -> QmsResult<()> {
        if args.is_empty() {
            return Err(QmsError::validation_error("No arguments provided"));
        }
        Ok(())
    }
}

/// Command Router implementing Open/Closed Principle
/// New command handlers can be registered without modifying this code
pub struct CommandRouter {
    handlers: std::collections::HashMap<String, Box<dyn CommandHandler>>,
}

impl CommandRouter {
    /// Create new command router
    pub fn new() -> Self {
        Self {
            handlers: std::collections::HashMap::new(),
        }
    }
    
    /// Register a command handler (Open/Closed Principle)
    pub fn register_handler(&mut self, handler: Box<dyn CommandHandler>) {
        let command_name = handler.command_name().to_string();
        self.handlers.insert(command_name, handler);
    }
    
    /// Route command to appropriate handler
    pub fn route_command(&self, command: &str, args: &[String]) -> QmsResult<()> {
        match self.handlers.get(command) {
            Some(handler) => {
                handler.validate_args(args)?;
                handler.handle(args)
            }
            None => Err(QmsError::validation_error(&format!("Unknown command: {}", command))),
        }
    }
    
    /// List all available commands
    pub fn list_commands(&self) -> Vec<&str> {
        self.handlers.keys().map(|s| s.as_str()).collect()
    }
    
    /// Get help for a specific command
    pub fn get_help(&self, command: &str) -> Option<&str> {
        self.handlers.get(command).map(|h| h.help_text())
    }
    
    /// Get help for all commands
    pub fn get_all_help(&self) -> Vec<(&str, &str)> {
        self.handlers
            .iter()
            .map(|(name, handler)| (name.as_str(), handler.help_text()))
            .collect()
    }
}

impl Default for CommandRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Base command handler with common functionality
/// Template Method Pattern implementation
pub trait BaseCommandHandler {
    /// Execute the command (template method)
    fn execute(&self, args: &[String]) -> QmsResult<()> {
        self.pre_execute(args)?;
        let result = self.do_execute(args);
        self.post_execute(args, &result)?;
        result
    }
    
    /// Pre-execution hook (can be overridden)
    fn pre_execute(&self, _args: &[String]) -> QmsResult<()> {
        Ok(())
    }
    
    /// Main execution logic (must be implemented)
    fn do_execute(&self, args: &[String]) -> QmsResult<()>;
    
    /// Post-execution hook (can be overridden)
    fn post_execute(&self, _args: &[String], _result: &QmsResult<()>) -> QmsResult<()> {
        Ok(())
    }
}

/// Command execution context for dependency injection
#[derive(Clone)]
pub struct CommandContext {
    pub project_path: std::path::PathBuf,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
}

impl CommandContext {
    pub fn new() -> QmsResult<Self> {
        let project_path = crate::utils::get_current_project_path()?;
        Ok(Self {
            project_path,
            user_id: None,
            session_id: None,
        })
    }
    
    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

impl Default for CommandContext {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            project_path: std::path::PathBuf::from("."),
            user_id: None,
            session_id: None,
        })
    }
}

/// Command factory for creating handlers with dependency injection
pub trait CommandFactory {
    fn create_handler(&self, context: &CommandContext) -> Box<dyn CommandHandler>;
}

/// Macro for creating command handlers with less boilerplate
#[macro_export]
macro_rules! impl_command_handler {
    ($handler:ty, $name:expr, $help:expr) => {
        impl CommandHandler for $handler {
            fn command_name(&self) -> &'static str {
                $name
            }
            
            fn help_text(&self) -> &'static str {
                $help
            }
            
            fn handle(&self, args: &[String]) -> QmsResult<()> {
                self.execute(args)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestHandler;
    
    impl CommandHandler for TestHandler {
        fn handle(&self, _args: &[String]) -> QmsResult<()> {
            Ok(())
        }
        
        fn command_name(&self) -> &'static str {
            "test"
        }
        
        fn help_text(&self) -> &'static str {
            "Test command"
        }
    }
    
    #[test]
    fn test_command_router() {
        let mut router = CommandRouter::new();
        router.register_handler(Box::new(TestHandler));
        
        assert!(router.route_command("test", &["arg1".to_string()]).is_ok());
        assert!(router.route_command("unknown", &["arg1".to_string()]).is_err());
    }
    
    #[test]
    fn test_command_context() {
        let context = CommandContext::default();
        assert!(context.project_path.exists() || context.project_path == std::path::PathBuf::from("."));
    }
}
