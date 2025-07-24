//! Unified State Management for QMS Interfaces
//! 
//! This module provides common state management abstractions that can be shared
//! across CLI, web, and TUI interfaces, ensuring consistent state handling and
//! eliminating code duplication.

use crate::prelude::*;
use crate::interfaces::{InterfaceContext, CommandResult};
use crate::modules::user_manager::UserSession;
use crate::json_utils::{JsonValue, JsonSerializable};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// State Manager trait - abstraction for managing interface state
/// Implements Interface Segregation Principle with focused state concerns
pub trait StateManager: Send + Sync {
    /// Update state after command execution
    fn update_state(&self, context: &InterfaceContext, result: &CommandResult) -> QmsResult<()>;

    /// Set authenticated session
    fn set_authenticated_session(&self, context: &InterfaceContext) -> QmsResult<()>;

    /// Clear authenticated session
    fn clear_authenticated_session(&self, context: &InterfaceContext) -> QmsResult<()>;

    /// Get current state snapshot
    fn get_state_snapshot(&self) -> QmsResult<StateSnapshot>;

    /// Restore state from snapshot
    fn restore_state(&self, snapshot: StateSnapshot) -> QmsResult<()>;

    /// Persist state to storage
    fn persist_state(&self) -> QmsResult<()>;

    /// Load state from storage
    fn load_state(&self) -> QmsResult<()>;
}

/// State snapshot for backup and restore operations
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub timestamp: u64,
    pub interface_type: crate::interfaces::InterfaceType,
    pub user_session: Option<UserSession>,
    pub project_path: Option<PathBuf>,
    pub configuration: HashMap<String, String>,
    pub navigation_history: Vec<String>,
    pub last_command: Option<String>,
}

impl StateSnapshot {
    /// Create new state snapshot
    pub fn new(context: &InterfaceContext) -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            interface_type: context.interface_type.clone(),
            user_session: context.user_session.clone(),
            project_path: context.project_path.clone(),
            configuration: context.configuration.clone(),
            navigation_history: Vec::new(),
            last_command: None,
        }
    }

    /// Update with command execution
    pub fn with_command_execution(mut self, command: &str) -> Self {
        self.last_command = Some(command.to_string());
        self.navigation_history.push(command.to_string());

        // Keep only last 50 commands in history
        if self.navigation_history.len() > 50 {
            self.navigation_history.remove(0);
        }

        self
    }
}

impl JsonSerializable for StateSnapshot {
    fn to_json(&self) -> String {
        let mut json_obj = HashMap::new();

        json_obj.insert("timestamp".to_string(), JsonValue::Number(self.timestamp as f64));
        json_obj.insert("interface_type".to_string(), JsonValue::String(format!("{:?}", self.interface_type)));

        // Serialize user session if present
        if let Some(ref session) = self.user_session {
            let mut session_obj = HashMap::new();
            session_obj.insert("session_id".to_string(), JsonValue::String(session.session_id.clone()));
            session_obj.insert("username".to_string(), JsonValue::String(session.username.clone()));
            session_obj.insert("user_id".to_string(), JsonValue::String(session.user_id.clone()));
            session_obj.insert("login_time".to_string(), JsonValue::Number(session.login_time as f64));
            session_obj.insert("last_activity".to_string(), JsonValue::Number(session.last_activity as f64));
            session_obj.insert("expires_at".to_string(), JsonValue::Number(session.expires_at as f64));
            session_obj.insert("is_active".to_string(), JsonValue::Bool(session.is_active));

            // Serialize roles
            let roles: Vec<JsonValue> = session.roles.iter()
                .map(|r| JsonValue::String(r.name.clone()))
                .collect();
            session_obj.insert("roles".to_string(), JsonValue::Array(roles));

            json_obj.insert("user_session".to_string(), JsonValue::Object(session_obj));
        } else {
            json_obj.insert("user_session".to_string(), JsonValue::Null);
        }

        // Serialize project path
        if let Some(ref path) = self.project_path {
            json_obj.insert("project_path".to_string(), JsonValue::String(path.to_string_lossy().to_string()));
        } else {
            json_obj.insert("project_path".to_string(), JsonValue::Null);
        }

        // Serialize configuration
        let config_obj: HashMap<String, JsonValue> = self.configuration.iter()
            .map(|(k, v)| (k.clone(), JsonValue::String(v.clone())))
            .collect();
        json_obj.insert("configuration".to_string(), JsonValue::Object(config_obj));

        // Serialize navigation history
        let history: Vec<JsonValue> = self.navigation_history.iter()
            .map(|h| JsonValue::String(h.clone()))
            .collect();
        json_obj.insert("navigation_history".to_string(), JsonValue::Array(history));

        // Serialize last command
        if let Some(ref cmd) = self.last_command {
            json_obj.insert("last_command".to_string(), JsonValue::String(cmd.clone()));
        } else {
            json_obj.insert("last_command".to_string(), JsonValue::Null);
        }

        JsonValue::Object(json_obj).to_json()
    }

    fn from_json(s: &str) -> Result<Self, crate::json_utils::JsonError> {
        let json_value = JsonValue::parse(s)?;

        if let JsonValue::Object(obj) = json_value {
            let timestamp = obj.get("timestamp")
                .and_then(|v| if let JsonValue::Number(n) = v { Some(*n as u64) } else { None })
                .unwrap_or(0);

            let interface_type = obj.get("interface_type")
                .and_then(|v| if let JsonValue::String(s) = v {
                    match s.as_str() {
                        "CLI" => Some(crate::interfaces::InterfaceType::CLI),
                        "Web" => Some(crate::interfaces::InterfaceType::Web),
                        "TUI" => Some(crate::interfaces::InterfaceType::TUI),
                        _ => None,
                    }
                } else { None })
                .unwrap_or(crate::interfaces::InterfaceType::CLI);

            // Deserialize user session
            let user_session = obj.get("user_session")
                .and_then(|v| if let JsonValue::Object(session_obj) = v {
                    let session_id = session_obj.get("session_id")?.as_string()?.clone();
                    let username = session_obj.get("username")?.as_string()?.clone();
                    let user_id = session_obj.get("user_id")?.as_string()?.clone();
                    let login_time = session_obj.get("login_time")?.as_number().map(|n| n as u64)?;
                    let last_activity = session_obj.get("last_activity")?.as_number().map(|n| n as u64)?;
                    let expires_at = session_obj.get("expires_at")?.as_number().map(|n| n as u64)?;
                    let is_active = session_obj.get("is_active")?.as_bool()?;

                    let role_names = session_obj.get("roles")
                        .and_then(|v| if let JsonValue::Array(arr) = v {
                            Some(arr.iter().filter_map(|r| r.as_string().cloned()).collect::<Vec<String>>())
                        } else { None })
                        .unwrap_or_default();

                    // Convert role names back to Role objects (simplified - just create basic roles)
                    let roles: Vec<crate::models::Role> = role_names.iter().map(|name| {
                        crate::models::Role {
                            name: name.clone(),
                            permissions: Vec::new(), // Simplified - would need proper role lookup
                        }
                    }).collect();

                    Some(UserSession {
                        session_id,
                        user_id,
                        username,
                        roles,
                        permissions: Vec::new(), // Simplified
                        login_time,
                        last_activity,
                        expires_at,
                        ip_address: None, // Not serialized for security
                        user_agent: None,
                        csrf_token: String::new(), // Simplified
                        is_active,
                        session_type: crate::modules::user_manager::interfaces::SessionType::CLI, // Default
                        data: std::collections::HashMap::new(), // Simplified
                    })
                } else { None });

            // Deserialize project path
            let project_path = obj.get("project_path")
                .and_then(|v| if let JsonValue::String(s) = v { Some(PathBuf::from(s)) } else { None });

            // Deserialize configuration
            let configuration = obj.get("configuration")
                .and_then(|v| if let JsonValue::Object(config_obj) = v {
                    Some(config_obj.iter()
                        .filter_map(|(k, v)| v.as_string().map(|s| (k.clone(), s.clone())))
                        .collect())
                } else { None })
                .unwrap_or_default();

            // Deserialize navigation history
            let navigation_history = obj.get("navigation_history")
                .and_then(|v| if let JsonValue::Array(arr) = v {
                    Some(arr.iter().filter_map(|h| h.as_string().cloned()).collect())
                } else { None })
                .unwrap_or_default();

            // Deserialize last command
            let last_command = obj.get("last_command")
                .and_then(|v| if let JsonValue::String(s) = v { Some(s.clone()) } else { None });

            Ok(StateSnapshot {
                timestamp,
                interface_type,
                user_session,
                project_path,
                configuration,
                navigation_history,
                last_command,
            })
        } else {
            Err(crate::json_utils::JsonError::InvalidFormat("Expected JSON object".to_string()))
        }
    }
}

/// File-based state manager for CLI and TUI interfaces
/// Implements Single Responsibility Principle for file-based state persistence
pub struct FileStateManager {
    project_path: Option<PathBuf>,
    state_file: PathBuf,
    current_state: Arc<Mutex<StateSnapshot>>,
}

impl FileStateManager {
    /// Create new file state manager
    pub fn new(project_path: Option<PathBuf>) -> Self {
        let state_file = match &project_path {
            Some(path) => path.join(".qms").join("interface_state.json"),
            None => {
                let home_dir = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home_dir).join(".qms").join("interface_state.json")
            }
        };

        let initial_context = crate::interfaces::InterfaceContext::new(
            crate::interfaces::InterfaceType::CLI
        );
        let current_state = Arc::new(Mutex::new(StateSnapshot::new(&initial_context)));

        Self {
            project_path,
            state_file,
            current_state,
        }
    }

    /// Ensure state directory exists
    fn ensure_state_directory(&self) -> QmsResult<()> {
        if let Some(parent) = self.state_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    /// Save state to file
    fn save_state_to_file(&self, snapshot: &StateSnapshot) -> QmsResult<()> {
        self.ensure_state_directory()?;

        let json_data = snapshot.to_json();
        std::fs::write(&self.state_file, json_data)?;
        Ok(())
    }

    /// Load state from file
    fn load_state_from_file(&self) -> QmsResult<StateSnapshot> {
        if !self.state_file.exists() {
            // Return default state if file doesn't exist
            let initial_context = crate::interfaces::InterfaceContext::new(
                crate::interfaces::InterfaceType::CLI
            );
            return Ok(StateSnapshot::new(&initial_context));
        }

        let json_data = std::fs::read_to_string(&self.state_file)?;
        let snapshot = StateSnapshot::from_json(&json_data)
            .map_err(|e| QmsError::validation_error(&format!("Failed to deserialize state: {}", e)))?;

        Ok(snapshot)
    }
}

impl StateManager for FileStateManager {
    fn update_state(&self, context: &InterfaceContext, result: &CommandResult) -> QmsResult<()> {
        let mut state = self.current_state.lock().unwrap();
        
        // Update state with command result
        if let Some(ref last_command) = result.next_action {
            *state = state.clone().with_command_execution(last_command);
        }
        
        // Update context information
        state.user_session = context.user_session.clone();
        state.project_path = context.project_path.clone();
        state.configuration = context.configuration.clone();
        
        // Persist to file
        self.save_state_to_file(&state)?;
        
        Ok(())
    }

    fn set_authenticated_session(&self, context: &InterfaceContext) -> QmsResult<()> {
        let mut state = self.current_state.lock().unwrap();
        state.user_session = context.user_session.clone();
        
        // Persist authentication state
        self.save_state_to_file(&state)?;
        
        Ok(())
    }

    fn clear_authenticated_session(&self, _context: &InterfaceContext) -> QmsResult<()> {
        let mut state = self.current_state.lock().unwrap();
        state.user_session = None;
        
        // Persist cleared authentication state
        self.save_state_to_file(&state)?;
        
        Ok(())
    }

    fn get_state_snapshot(&self) -> QmsResult<StateSnapshot> {
        let state = self.current_state.lock().unwrap();
        Ok(state.clone())
    }

    fn restore_state(&self, snapshot: StateSnapshot) -> QmsResult<()> {
        let mut state = self.current_state.lock().unwrap();
        *state = snapshot.clone();
        
        // Persist restored state
        self.save_state_to_file(&snapshot)?;
        
        Ok(())
    }

    fn persist_state(&self) -> QmsResult<()> {
        let state = self.current_state.lock().unwrap();
        self.save_state_to_file(&state)
    }

    fn load_state(&self) -> QmsResult<()> {
        let loaded_state = self.load_state_from_file()?;
        let mut state = self.current_state.lock().unwrap();
        *state = loaded_state;
        
        Ok(())
    }
}

/// Session-based state manager for web interface
/// Implements Single Responsibility Principle for session-based state management
pub struct SessionStateManager {
    sessions: Arc<Mutex<HashMap<String, StateSnapshot>>>,
}

impl SessionStateManager {
    /// Create new session state manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get session ID from context
    fn get_session_id(context: &InterfaceContext) -> Option<String> {
        context.user_session.as_ref().map(|s| s.session_id.clone())
    }

    /// Get or create session state
    fn get_or_create_session_state(&self, context: &InterfaceContext) -> StateSnapshot {
        let session_id = Self::get_session_id(context);
        let mut sessions = self.sessions.lock().unwrap();
        
        match session_id {
            Some(id) => {
                sessions.get(&id).cloned().unwrap_or_else(|| {
                    let snapshot = StateSnapshot::new(context);
                    sessions.insert(id, snapshot.clone());
                    snapshot
                })
            }
            None => StateSnapshot::new(context),
        }
    }

    /// Update session state
    fn update_session_state(&self, context: &InterfaceContext, snapshot: StateSnapshot) {
        if let Some(session_id) = Self::get_session_id(context) {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.insert(session_id, snapshot);
        }
    }
}

impl StateManager for SessionStateManager {
    fn update_state(&self, context: &InterfaceContext, result: &CommandResult) -> QmsResult<()> {
        let mut snapshot = self.get_or_create_session_state(context);
        
        // Update state with command result
        if let Some(ref last_command) = result.next_action {
            snapshot = snapshot.with_command_execution(last_command);
        }
        
        // Update context information
        snapshot.user_session = context.user_session.clone();
        snapshot.project_path = context.project_path.clone();
        snapshot.configuration = context.configuration.clone();
        
        self.update_session_state(context, snapshot);
        
        Ok(())
    }

    fn set_authenticated_session(&self, context: &InterfaceContext) -> QmsResult<()> {
        let mut snapshot = self.get_or_create_session_state(context);
        snapshot.user_session = context.user_session.clone();
        
        self.update_session_state(context, snapshot);
        
        Ok(())
    }

    fn clear_authenticated_session(&self, context: &InterfaceContext) -> QmsResult<()> {
        if let Some(session_id) = Self::get_session_id(context) {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.remove(&session_id);
        }
        
        Ok(())
    }

    fn get_state_snapshot(&self) -> QmsResult<StateSnapshot> {
        // For session-based state, we need a context to identify the session
        // This is a limitation of the current design - we'll return a default snapshot
        let initial_context = crate::interfaces::InterfaceContext::new(
            crate::interfaces::InterfaceType::Web
        );
        Ok(StateSnapshot::new(&initial_context))
    }

    fn restore_state(&self, _snapshot: StateSnapshot) -> QmsResult<()> {
        // Session-based state restoration is handled through session management
        // This is a no-op for web sessions
        Ok(())
    }

    fn persist_state(&self) -> QmsResult<()> {
        // Session state is persisted through the session management system
        // This is a no-op for web sessions
        Ok(())
    }

    fn load_state(&self) -> QmsResult<()> {
        // Session state is loaded through the session management system
        // This is a no-op for web sessions
        Ok(())
    }
}

/// Memory-based state manager for testing
/// Implements Single Responsibility Principle for in-memory state management
pub struct MemoryStateManager {
    current_state: Arc<Mutex<StateSnapshot>>,
}

impl MemoryStateManager {
    /// Create new memory state manager
    pub fn new() -> Self {
        let initial_context = crate::interfaces::InterfaceContext::new(
            crate::interfaces::InterfaceType::CLI
        );
        let current_state = Arc::new(Mutex::new(StateSnapshot::new(&initial_context)));

        Self { current_state }
    }
}

impl StateManager for MemoryStateManager {
    fn update_state(&self, context: &InterfaceContext, result: &CommandResult) -> QmsResult<()> {
        let mut state = self.current_state.lock().unwrap();
        
        // Update state with command result
        if let Some(ref last_command) = result.next_action {
            *state = state.clone().with_command_execution(last_command);
        }
        
        // Update context information
        state.user_session = context.user_session.clone();
        state.project_path = context.project_path.clone();
        state.configuration = context.configuration.clone();
        
        Ok(())
    }

    fn set_authenticated_session(&self, context: &InterfaceContext) -> QmsResult<()> {
        let mut state = self.current_state.lock().unwrap();
        state.user_session = context.user_session.clone();
        Ok(())
    }

    fn clear_authenticated_session(&self, _context: &InterfaceContext) -> QmsResult<()> {
        let mut state = self.current_state.lock().unwrap();
        state.user_session = None;
        Ok(())
    }

    fn get_state_snapshot(&self) -> QmsResult<StateSnapshot> {
        let state = self.current_state.lock().unwrap();
        Ok(state.clone())
    }

    fn restore_state(&self, snapshot: StateSnapshot) -> QmsResult<()> {
        let mut state = self.current_state.lock().unwrap();
        *state = snapshot;
        Ok(())
    }

    fn persist_state(&self) -> QmsResult<()> {
        // Memory state doesn't need persistence
        Ok(())
    }

    fn load_state(&self) -> QmsResult<()> {
        // Memory state doesn't need loading
        Ok(())
    }
}
