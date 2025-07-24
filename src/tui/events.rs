//! Event Handling System for QMS TUI
//! 
//! This module provides event handling for keyboard, mouse, and terminal
//! resize events using standard library components with cross-platform support.

use crate::prelude::*;
use std::io::{self, Read};
use std::time::{Duration, Instant};

/// Event types that can occur in the TUI
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16), // width, height
}

/// Keyboard event information
#[derive(Debug, Clone, PartialEq)]
pub struct KeyEvent {
    pub key: String,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl KeyEvent {
    /// Create new key event
    pub fn new(key: String) -> Self {
        Self {
            key,
            ctrl: false,
            alt: false,
            shift: false,
        }
    }

    /// Create key event with modifiers
    pub fn with_modifiers(key: String, ctrl: bool, alt: bool, shift: bool) -> Self {
        Self {
            key,
            ctrl,
            alt,
            shift,
        }
    }

    /// Check if this is a printable character
    pub fn is_printable(&self) -> bool {
        self.key.len() == 1 && !self.ctrl && !self.alt
    }

    /// Check if this is a navigation key
    pub fn is_navigation(&self) -> bool {
        matches!(self.key.as_str(), "Up" | "Down" | "Left" | "Right" | "Home" | "End" | "PageUp" | "PageDown")
    }

    /// Check if this is an action key
    pub fn is_action(&self) -> bool {
        matches!(self.key.as_str(), "Enter" | "Space" | "Tab" | "Escape" | "Delete" | "Backspace")
    }
}

/// Mouse event information
#[derive(Debug, Clone, PartialEq)]
pub struct MouseEvent {
    pub x: u16,
    pub y: u16,
    pub button: MouseButton,
    pub action: MouseAction,
}

/// Mouse button types
#[derive(Debug, Clone, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    WheelUp,
    WheelDown,
}

/// Mouse action types
#[derive(Debug, Clone, PartialEq)]
pub enum MouseAction {
    Press,
    Release,
    Move,
    Drag,
}

/// Event handler for processing terminal input
pub struct EventHandler {
    last_poll: Instant,
    poll_timeout: Duration,
}

impl EventHandler {
    /// Create new event handler
    pub fn new() -> Self {
        Self {
            last_poll: Instant::now(),
            poll_timeout: Duration::from_millis(100),
        }
    }

    /// Poll for events (non-blocking)
    pub fn poll_event(&mut self) -> QmsResult<Option<Event>> {
        // Check if enough time has passed since last poll
        if self.last_poll.elapsed() < self.poll_timeout {
            return Ok(None);
        }

        self.last_poll = Instant::now();

        // Try to read input
        if let Some(event) = self.read_input()? {
            return Ok(Some(event));
        }

        // Check for terminal resize
        if let Some(event) = self.check_resize()? {
            return Ok(Some(event));
        }

        Ok(None)
    }

    /// Wait for next event (blocking)
    pub fn wait_for_event(&mut self) -> QmsResult<Event> {
        loop {
            if let Some(event) = self.poll_event()? {
                return Ok(event);
            }
            
            // Small sleep to prevent busy waiting
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// Read input from stdin
    fn read_input(&self) -> QmsResult<Option<Event>> {
        // This is a simplified implementation
        // In a full implementation, we'd use platform-specific non-blocking I/O
        
        // For now, we'll simulate some common key events
        // In practice, this would read from stdin and parse ANSI escape sequences
        
        Ok(None)
    }

    /// Check for terminal resize
    fn check_resize(&self) -> QmsResult<Option<Event>> {
        // This would check if the terminal has been resized
        // For now, we'll return None (no resize detected)
        Ok(None)
    }

    /// Parse ANSI escape sequence into key event
    fn parse_escape_sequence(&self, sequence: &[u8]) -> Option<KeyEvent> {
        match sequence {
            // Arrow keys
            b"\x1b[A" => Some(KeyEvent::new("Up".to_string())),
            b"\x1b[B" => Some(KeyEvent::new("Down".to_string())),
            b"\x1b[C" => Some(KeyEvent::new("Right".to_string())),
            b"\x1b[D" => Some(KeyEvent::new("Left".to_string())),
            
            // Function keys
            b"\x1bOP" => Some(KeyEvent::new("F1".to_string())),
            b"\x1bOQ" => Some(KeyEvent::new("F2".to_string())),
            b"\x1bOR" => Some(KeyEvent::new("F3".to_string())),
            b"\x1bOS" => Some(KeyEvent::new("F4".to_string())),
            
            // Home/End
            b"\x1b[H" => Some(KeyEvent::new("Home".to_string())),
            b"\x1b[F" => Some(KeyEvent::new("End".to_string())),
            
            // Page Up/Down
            b"\x1b[5~" => Some(KeyEvent::new("PageUp".to_string())),
            b"\x1b[6~" => Some(KeyEvent::new("PageDown".to_string())),
            
            // Delete
            b"\x1b[3~" => Some(KeyEvent::new("Delete".to_string())),
            
            _ => None,
        }
    }

    /// Parse regular character into key event
    fn parse_character(&self, ch: u8) -> KeyEvent {
        match ch {
            // Control characters
            0x01..=0x1A => {
                let key = char::from(ch + b'a' - 1).to_string();
                KeyEvent::with_modifiers(key, true, false, false)
            }
            
            // Escape
            0x1B => KeyEvent::new("Escape".to_string()),
            
            // Backspace
            0x7F | 0x08 => KeyEvent::new("Backspace".to_string()),
            
            // Tab
            0x09 => KeyEvent::new("Tab".to_string()),
            
            // Enter
            0x0A | 0x0D => KeyEvent::new("Enter".to_string()),
            
            // Space
            0x20 => KeyEvent::new("Space".to_string()),
            
            // Printable characters
            0x21..=0x7E => {
                let key = char::from(ch).to_string();
                KeyEvent::new(key)
            }
            
            // Other characters
            _ => KeyEvent::new(format!("Unknown({})", ch)),
        }
    }

    /// Set poll timeout
    pub fn set_poll_timeout(&mut self, timeout: Duration) {
        self.poll_timeout = timeout;
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Event dispatcher for routing events to appropriate handlers
pub struct EventDispatcher {
    handlers: Vec<Box<dyn EventHandlerTrait>>,
}

impl EventDispatcher {
    /// Create new event dispatcher
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Add event handler
    pub fn add_handler(&mut self, handler: Box<dyn EventHandlerTrait>) {
        self.handlers.push(handler);
    }

    /// Dispatch event to all handlers
    pub fn dispatch(&mut self, event: &Event) -> QmsResult<bool> {
        for handler in &mut self.handlers {
            if handler.handle_event(event)? {
                return Ok(true); // Event was consumed
            }
        }
        Ok(false) // Event was not consumed
    }
}

/// Trait for objects that can handle events
pub trait EventHandlerTrait {
    /// Handle an event, return true if event was consumed
    fn handle_event(&mut self, event: &Event) -> QmsResult<bool>;
}

/// Global key bindings for the TUI
pub struct KeyBindings {
    bindings: std::collections::HashMap<String, String>,
}

impl KeyBindings {
    /// Create default key bindings
    pub fn default() -> Self {
        let mut bindings = std::collections::HashMap::new();
        
        // Global bindings
        bindings.insert("Ctrl+q".to_string(), "quit".to_string());
        bindings.insert("Ctrl+c".to_string(), "quit".to_string());
        bindings.insert("Escape".to_string(), "back".to_string());
        bindings.insert("F1".to_string(), "help".to_string());
        
        // Navigation bindings
        bindings.insert("Up".to_string(), "up".to_string());
        bindings.insert("Down".to_string(), "down".to_string());
        bindings.insert("Left".to_string(), "left".to_string());
        bindings.insert("Right".to_string(), "right".to_string());
        bindings.insert("Enter".to_string(), "select".to_string());
        bindings.insert("Space".to_string(), "select".to_string());
        bindings.insert("Tab".to_string(), "next".to_string());
        bindings.insert("Shift+Tab".to_string(), "previous".to_string());
        
        // Menu bindings
        bindings.insert("1".to_string(), "menu_1".to_string());
        bindings.insert("2".to_string(), "menu_2".to_string());
        bindings.insert("3".to_string(), "menu_3".to_string());
        bindings.insert("4".to_string(), "menu_4".to_string());
        bindings.insert("5".to_string(), "menu_5".to_string());
        bindings.insert("6".to_string(), "menu_6".to_string());
        bindings.insert("7".to_string(), "menu_7".to_string());
        bindings.insert("8".to_string(), "menu_8".to_string());
        bindings.insert("9".to_string(), "menu_9".to_string());
        
        Self { bindings }
    }

    /// Get action for key event
    pub fn get_action(&self, key_event: &KeyEvent) -> Option<&str> {
        let key_string = self.key_event_to_string(key_event);
        self.bindings.get(&key_string).map(|s| s.as_str())
    }

    /// Convert key event to string representation
    fn key_event_to_string(&self, key_event: &KeyEvent) -> String {
        let mut result = String::new();
        
        if key_event.ctrl {
            result.push_str("Ctrl+");
        }
        if key_event.alt {
            result.push_str("Alt+");
        }
        if key_event.shift {
            result.push_str("Shift+");
        }
        
        result.push_str(&key_event.key);
        result
    }

    /// Add custom key binding
    pub fn add_binding(&mut self, key: String, action: String) {
        self.bindings.insert(key, action);
    }

    /// Remove key binding
    pub fn remove_binding(&mut self, key: &str) {
        self.bindings.remove(key);
    }
}
