//! Unified User Interaction Abstractions for QMS Interfaces
//! 
//! This module provides common user interaction patterns that can be shared
//! across CLI, web, and TUI interfaces, ensuring consistent user experience
//! and eliminating code duplication.

use crate::prelude::*;
use std::collections::HashMap;
use std::io::Read;

/// User Interaction Provider trait - abstraction for user interface operations
/// Implements Interface Segregation Principle with focused UI concerns
pub trait UserInteractionProvider: Send + Sync {
    /// Display a message to the user
    fn display_message(&self, message: &str, message_type: MessageType) -> QmsResult<()>;

    /// Prompt user for input
    fn prompt_input(&self, prompt: &str, input_type: InputType) -> QmsResult<String>;

    /// Display a confirmation dialog
    fn confirm_action(&self, message: &str) -> QmsResult<bool>;

    /// Display a selection menu
    fn display_menu(&self, title: &str, options: Vec<MenuOption>) -> QmsResult<usize>;

    /// Display tabular data
    fn display_table(&self, headers: Vec<String>, rows: Vec<Vec<String>>) -> QmsResult<()>;

    /// Display progress indicator
    fn display_progress(&self, message: &str, progress: f32) -> QmsResult<()>;

    /// Clear the display/screen
    fn clear_display(&self) -> QmsResult<()>;

    /// Display error with context
    fn display_error(&self, error: &QmsError, context: Option<&str>) -> QmsResult<()>;

    /// Display success message with optional data
    fn display_success(&self, message: &str, data: Option<&str>) -> QmsResult<()>;
}

/// Message types for different display contexts
#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    Info,
    Warning,
    Error,
    Success,
    Debug,
}

/// Input types for user prompts
#[derive(Debug, Clone, PartialEq)]
pub enum InputType {
    Text,
    Password,
    Number,
    Email,
    Path,
    MultiLine,
}

/// Menu option for selection menus
#[derive(Debug, Clone)]
pub struct MenuOption {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub enabled: bool,
}

impl MenuOption {
    /// Create new menu option
    pub fn new(id: String, label: String) -> Self {
        Self {
            id,
            label,
            description: None,
            enabled: true,
        }
    }

    /// Add description to menu option
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set enabled state
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// CLI User Interaction Implementation
/// Implements Single Responsibility Principle for command-line interface
pub struct CliUserInteraction;

impl CliUserInteraction {
    pub fn new() -> Self {
        Self
    }

    /// Get colored output for message type
    fn get_message_prefix(&self, message_type: &MessageType) -> &'static str {
        match message_type {
            MessageType::Info => "ℹ️ ",
            MessageType::Warning => "⚠️ ",
            MessageType::Error => "❌ ",
            MessageType::Success => "✅ ",
            MessageType::Debug => "🐛 ",
        }
    }
}

impl UserInteractionProvider for CliUserInteraction {
    fn display_message(&self, message: &str, message_type: MessageType) -> QmsResult<()> {
        let prefix = self.get_message_prefix(&message_type);
        println!("{}{}", prefix, message);
        Ok(())
    }

    fn prompt_input(&self, prompt: &str, input_type: InputType) -> QmsResult<String> {
        use std::io::{self, Write};

        print!("{}: ", prompt);
        io::stdout().flush()?;

        let mut input = String::new();
        
        match input_type {
            InputType::Password => {
                // For password input, we'd ideally use a crate like `rpassword`
                // For now, we'll use regular input with a warning
                println!("(Warning: Password will be visible)");
                io::stdin().read_line(&mut input)?;
            }
            InputType::MultiLine => {
                println!("(Enter multiple lines, press Ctrl+D when done)");
                io::stdin().read_to_string(&mut input)?;
            }
            _ => {
                io::stdin().read_line(&mut input)?;
            }
        }

        Ok(input.trim().to_string())
    }

    fn confirm_action(&self, message: &str) -> QmsResult<bool> {
        let response = self.prompt_input(&format!("{} (y/N)", message), InputType::Text)?;
        Ok(response.to_lowercase().starts_with('y'))
    }

    fn display_menu(&self, title: &str, options: Vec<MenuOption>) -> QmsResult<usize> {
        println!("\n{}", title);
        println!("{}", "=".repeat(title.len()));

        for (index, option) in options.iter().enumerate() {
            let status = if option.enabled { "" } else { " (disabled)" };
            println!("{}. {}{}", index + 1, option.label, status);
            
            if let Some(ref description) = option.description {
                println!("   {}", description);
            }
        }

        loop {
            let input = self.prompt_input("Select option", InputType::Number)?;
            
            if let Ok(choice) = input.parse::<usize>() {
                if choice > 0 && choice <= options.len() {
                    let selected_option = &options[choice - 1];
                    if selected_option.enabled {
                        return Ok(choice - 1);
                    } else {
                        println!("❌ Option {} is disabled", choice);
                    }
                } else {
                    println!("❌ Invalid option. Please select 1-{}", options.len());
                }
            } else {
                println!("❌ Please enter a valid number");
            }
        }
    }

    fn display_table(&self, headers: Vec<String>, rows: Vec<Vec<String>>) -> QmsResult<()> {
        if headers.is_empty() {
            return Ok(());
        }

        // Calculate column widths
        let mut col_widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
        
        for row in &rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_widths.len() {
                    col_widths[i] = col_widths[i].max(cell.len());
                }
            }
        }

        // Print headers
        for (i, header) in headers.iter().enumerate() {
            print!("{:<width$}", header, width = col_widths[i] + 2);
        }
        println!();

        // Print separator
        for width in &col_widths {
            print!("{}", "-".repeat(width + 2));
        }
        println!();

        // Print rows
        for row in &rows {
            for (i, cell) in row.iter().enumerate() {
                if i < col_widths.len() {
                    print!("{:<width$}", cell, width = col_widths[i] + 2);
                }
            }
            println!();
        }

        Ok(())
    }

    fn display_progress(&self, message: &str, progress: f32) -> QmsResult<()> {
        let progress_clamped = progress.clamp(0.0, 1.0);
        let bar_width = 40;
        let filled = (progress_clamped * bar_width as f32) as usize;
        let empty = bar_width - filled;

        print!("\r{}: [{}{}] {:.1}%", 
            message,
            "█".repeat(filled),
            "░".repeat(empty),
            progress_clamped * 100.0
        );
        
        use std::io::{self, Write};
        io::stdout().flush()?;
        
        if progress_clamped >= 1.0 {
            println!(); // New line when complete
        }

        Ok(())
    }

    fn clear_display(&self) -> QmsResult<()> {
        // Clear screen using ANSI escape codes
        print!("\x1B[2J\x1B[1;1H");
        use std::io::{self, Write};
        io::stdout().flush()?;
        Ok(())
    }

    fn display_error(&self, error: &QmsError, context: Option<&str>) -> QmsResult<()> {
        if let Some(ctx) = context {
            println!("❌ Error in {}: {}", ctx, error);
        } else {
            println!("❌ Error: {}", error);
        }
        Ok(())
    }

    fn display_success(&self, message: &str, data: Option<&str>) -> QmsResult<()> {
        println!("✅ {}", message);
        if let Some(data) = data {
            println!("   {}", data);
        }
        Ok(())
    }
}

/// Web User Interaction Implementation
/// Implements Single Responsibility Principle for web interface
pub struct WebUserInteraction;

impl WebUserInteraction {
    pub fn new() -> Self {
        Self
    }
}

impl UserInteractionProvider for WebUserInteraction {
    fn display_message(&self, message: &str, message_type: MessageType) -> QmsResult<()> {
        // For web interface, messages are typically returned as JSON responses
        // This is a placeholder implementation
        println!("Web Message ({:?}): {}", message_type, message);
        Ok(())
    }

    fn prompt_input(&self, prompt: &str, _input_type: InputType) -> QmsResult<String> {
        // Web interface handles input through forms and AJAX requests
        // This is a placeholder that would be handled by the frontend
        Err(QmsError::InvalidOperation(
            format!("Web interface input prompts are handled by frontend: {}", prompt)
        ))
    }

    fn confirm_action(&self, message: &str) -> QmsResult<bool> {
        // Web interface handles confirmations through JavaScript dialogs or modals
        // This is a placeholder
        Err(QmsError::InvalidOperation(
            format!("Web interface confirmations are handled by frontend: {}", message)
        ))
    }

    fn display_menu(&self, _title: &str, _options: Vec<MenuOption>) -> QmsResult<usize> {
        // Web interface handles menus through HTML/JavaScript
        // This is a placeholder
        Err(QmsError::InvalidOperation(
            "Web interface menus are handled by frontend".to_string()
        ))
    }

    fn display_table(&self, _headers: Vec<String>, _rows: Vec<Vec<String>>) -> QmsResult<()> {
        // Web interface handles tables through HTML
        // This is a placeholder
        Ok(())
    }

    fn display_progress(&self, _message: &str, _progress: f32) -> QmsResult<()> {
        // Web interface handles progress through JavaScript/CSS
        // This is a placeholder
        Ok(())
    }

    fn clear_display(&self) -> QmsResult<()> {
        // Web interface handles display clearing through DOM manipulation
        // This is a placeholder
        Ok(())
    }

    fn display_error(&self, error: &QmsError, context: Option<&str>) -> QmsResult<()> {
        // Web interface handles errors through JSON responses
        println!("Web Error{}: {}", 
            context.map(|c| format!(" in {}", c)).unwrap_or_default(), 
            error
        );
        Ok(())
    }

    fn display_success(&self, message: &str, data: Option<&str>) -> QmsResult<()> {
        // Web interface handles success messages through JSON responses
        println!("Web Success: {}{}", 
            message,
            data.map(|d| format!(" - {}", d)).unwrap_or_default()
        );
        Ok(())
    }
}

/// TUI User Interaction Implementation
/// Implements Single Responsibility Principle for terminal user interface
pub struct TuiUserInteraction;

impl TuiUserInteraction {
    pub fn new() -> Self {
        Self
    }
}

impl UserInteractionProvider for TuiUserInteraction {
    fn display_message(&self, message: &str, message_type: MessageType) -> QmsResult<()> {
        // TUI would use a library like ratatui or cursive for rich terminal UI
        // This is a placeholder implementation
        let prefix = match message_type {
            MessageType::Info => "[INFO]",
            MessageType::Warning => "[WARN]",
            MessageType::Error => "[ERROR]",
            MessageType::Success => "[SUCCESS]",
            MessageType::Debug => "[DEBUG]",
        };
        println!("{} {}", prefix, message);
        Ok(())
    }

    fn prompt_input(&self, prompt: &str, input_type: InputType) -> QmsResult<String> {
        // TUI would handle input through terminal widgets
        // For now, fall back to CLI-style input
        let cli_interaction = CliUserInteraction::new();
        cli_interaction.prompt_input(prompt, input_type)
    }

    fn confirm_action(&self, message: &str) -> QmsResult<bool> {
        // TUI would show a modal dialog for confirmation
        // For now, fall back to CLI-style confirmation
        let cli_interaction = CliUserInteraction::new();
        cli_interaction.confirm_action(message)
    }

    fn display_menu(&self, title: &str, options: Vec<MenuOption>) -> QmsResult<usize> {
        // TUI would show an interactive menu with keyboard navigation
        // For now, fall back to CLI-style menu
        let cli_interaction = CliUserInteraction::new();
        cli_interaction.display_menu(title, options)
    }

    fn display_table(&self, headers: Vec<String>, rows: Vec<Vec<String>>) -> QmsResult<()> {
        // TUI would show a scrollable table widget
        // For now, fall back to CLI-style table
        let cli_interaction = CliUserInteraction::new();
        cli_interaction.display_table(headers, rows)
    }

    fn display_progress(&self, message: &str, progress: f32) -> QmsResult<()> {
        // TUI would show a progress bar widget
        // For now, fall back to CLI-style progress
        let cli_interaction = CliUserInteraction::new();
        cli_interaction.display_progress(message, progress)
    }

    fn clear_display(&self) -> QmsResult<()> {
        // TUI would clear the terminal screen and redraw widgets
        let cli_interaction = CliUserInteraction::new();
        cli_interaction.clear_display()
    }

    fn display_error(&self, error: &QmsError, context: Option<&str>) -> QmsResult<()> {
        // TUI would show error in a status bar or modal
        let cli_interaction = CliUserInteraction::new();
        cli_interaction.display_error(error, context)
    }

    fn display_success(&self, message: &str, data: Option<&str>) -> QmsResult<()> {
        // TUI would show success in a status bar or notification
        let cli_interaction = CliUserInteraction::new();
        cli_interaction.display_success(message, data)
    }
}
