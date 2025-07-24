//! TUI Authentication Screens
//! 
//! This module provides the login, logout, and authentication-related screens
//! for the QMS TUI interface with proper security and user experience.

use crate::prelude::*;
use crate::tui::{Terminal, Theme};
use crate::tui::auth::{TuiAuthManager, TuiAuthResult};
use std::io::{self, Write};
use std::path::PathBuf;

/// Login screen for TUI authentication
pub struct TuiLoginScreen {
    auth_manager: TuiAuthManager,
    username: String,
    password: String,
    current_field: LoginField,
    message: Option<String>,
    is_error: bool,
    login_attempts: u32,
    max_attempts: u32,
}

/// Login form fields
#[derive(Debug, Clone, PartialEq)]
enum LoginField {
    Username,
    Password,
    Submit,
}

/// Authentication screen result
#[derive(Debug, Clone, PartialEq)]
pub enum AuthScreenResult {
    LoginSuccess(String), // username
    LoginFailed(String),  // error message
    Quit,
    Continue,
}

impl TuiLoginScreen {
    /// Create new login screen
    pub fn new(project_path: Option<PathBuf>) -> QmsResult<Self> {
        let auth_manager = TuiAuthManager::new(project_path)?;
        
        Ok(Self {
            auth_manager,
            username: String::new(),
            password: String::new(),
            current_field: LoginField::Username,
            message: None,
            is_error: false,
            login_attempts: 0,
            max_attempts: 3,
        })
    }

    /// Render the login screen
    pub fn render(&self, terminal: &Terminal, theme: &Theme) -> QmsResult<()> {
        terminal.clear_screen()?;

        // Header
        terminal.move_cursor(2, 2)?;
        terminal.set_color(theme.colors.primary, theme.colors.background)?;
        terminal.write_text("╔══════════════════════════════════════════════════════════════════════════════╗")?;
        
        terminal.move_cursor(3, 2)?;
        terminal.write_text("║                    QMS - Medical Device Quality Management System           ║")?;
        
        terminal.move_cursor(4, 2)?;
        terminal.write_text("║                     FDA 21 CFR Part 820, ISO 13485, ISO 14971 Compliant    ║")?;
        
        terminal.move_cursor(5, 2)?;
        terminal.write_text("╚══════════════════════════════════════════════════════════════════════════════╝")?;

        // Login form title
        terminal.move_cursor(8, 10)?;
        terminal.set_color(theme.colors.text_primary, theme.colors.background)?;
        terminal.write_text("🔐 User Authentication Required")?;

        // Security notice
        terminal.move_cursor(10, 10)?;
        terminal.set_color(theme.colors.text_secondary, theme.colors.background)?;
        terminal.write_text("This system contains confidential medical device information.")?;
        
        terminal.move_cursor(11, 10)?;
        terminal.write_text("Unauthorized access is prohibited and monitored.")?;

        // Username field
        terminal.move_cursor(14, 10)?;
        terminal.set_color(theme.colors.text_primary, theme.colors.background)?;
        terminal.write_text("Username:")?;
        
        terminal.move_cursor(15, 12)?;
        let username_style = if self.current_field == LoginField::Username {
            theme.styles.form_input_focused
        } else {
            theme.styles.form_input
        };
        terminal.set_color(username_style.0, username_style.1)?;
        
        let username_display = if self.username.is_empty() {
            "[Enter username]".to_string()
        } else {
            self.username.clone()
        };
        terminal.write_text(&format!("│ {:<30} │", username_display))?;

        // Password field
        terminal.move_cursor(17, 10)?;
        terminal.set_color(theme.colors.text_primary, theme.colors.background)?;
        terminal.write_text("Password:")?;
        
        terminal.move_cursor(18, 12)?;
        let password_style = if self.current_field == LoginField::Password {
            theme.styles.form_input_focused
        } else {
            theme.styles.form_input
        };
        terminal.set_color(password_style.0, password_style.1)?;
        
        let password_display = if self.password.is_empty() {
            "[Enter password]".to_string()
        } else {
            "*".repeat(self.password.len())
        };
        terminal.write_text(&format!("│ {:<30} │", password_display))?;

        // Submit button
        terminal.move_cursor(20, 10)?;
        let submit_style = if self.current_field == LoginField::Submit {
            theme.styles.button_focused
        } else {
            theme.styles.button_normal
        };
        terminal.set_color(submit_style.0, submit_style.1)?;
        terminal.write_text("[ Login ]")?;

        // Instructions
        terminal.move_cursor(23, 10)?;
        terminal.set_color(theme.colors.text_secondary, theme.colors.background)?;
        terminal.write_text("Instructions:")?;
        
        terminal.move_cursor(24, 12)?;
        terminal.write_text("• Use Tab/Shift+Tab to navigate fields")?;
        
        terminal.move_cursor(25, 12)?;
        terminal.write_text("• Type to enter username/password")?;
        
        terminal.move_cursor(26, 12)?;
        terminal.write_text("• Press Enter to login")?;
        
        terminal.move_cursor(27, 12)?;
        terminal.write_text("• Press 'q' to quit")?;

        // Status message
        if let Some(ref message) = self.message {
            let (rows, _cols) = terminal.get_size()?;
            terminal.move_cursor(rows - 3, 10)?;
            
            let color = if self.is_error {
                theme.colors.error
            } else {
                theme.colors.success
            };
            terminal.set_color(color, theme.colors.background)?;
            
            let icon = if self.is_error { "❌" } else { "✅" };
            terminal.write_text(&format!("{} {}", icon, message))?;
        }

        // Login attempts warning
        if self.login_attempts > 0 {
            let (rows, _cols) = terminal.get_size()?;
            terminal.move_cursor(rows - 2, 10)?;
            terminal.set_color(theme.colors.warning, theme.colors.background)?;
            terminal.write_text(&format!("⚠️  Login attempts: {}/{}", self.login_attempts, self.max_attempts))?;
        }

        terminal.reset_colors()?;
        terminal.flush()?;
        
        Ok(())
    }

    /// Handle user input for login screen
    pub fn handle_input(&mut self) -> QmsResult<AuthScreenResult> {
        let mut input = String::new();
        print!("> ");
        io::stdout().flush()?;
        
        if io::stdin().read_line(&mut input).is_ok() {
            let input = input.trim();
            
            match input {
                "q" | "quit" | "exit" => return Ok(AuthScreenResult::Quit),
                "tab" => {
                    self.next_field();
                    return Ok(AuthScreenResult::Continue);
                }
                "shift+tab" => {
                    self.previous_field();
                    return Ok(AuthScreenResult::Continue);
                }
                "" => {
                    // Enter key - attempt login or move to next field
                    if self.current_field == LoginField::Submit || 
                       (!self.username.is_empty() && !self.password.is_empty()) {
                        return self.attempt_login();
                    } else {
                        self.next_field();
                        return Ok(AuthScreenResult::Continue);
                    }
                }
                _ => {
                    // Handle field input
                    match self.current_field {
                        LoginField::Username => {
                            self.username = input.to_string();
                            self.message = None;
                        }
                        LoginField::Password => {
                            self.password = input.to_string();
                            self.message = None;
                        }
                        LoginField::Submit => {
                            return self.attempt_login();
                        }
                    }
                }
            }
        }
        
        Ok(AuthScreenResult::Continue)
    }

    /// Attempt to login with current credentials
    fn attempt_login(&mut self) -> QmsResult<AuthScreenResult> {
        // Validate input
        if self.username.trim().is_empty() {
            self.message = Some("Username is required".to_string());
            self.is_error = true;
            self.current_field = LoginField::Username;
            return Ok(AuthScreenResult::Continue);
        }

        if self.password.trim().is_empty() {
            self.message = Some("Password is required".to_string());
            self.is_error = true;
            self.current_field = LoginField::Password;
            return Ok(AuthScreenResult::Continue);
        }

        // Check login attempts
        if self.login_attempts >= self.max_attempts {
            self.message = Some("Maximum login attempts exceeded. Please try again later.".to_string());
            self.is_error = true;
            return Ok(AuthScreenResult::LoginFailed("Max attempts exceeded".to_string()));
        }

        // Attempt authentication
        match self.auth_manager.login(&self.username, &self.password) {
            Ok(auth_result) => {
                if auth_result.success {
                    self.message = Some(auth_result.message.clone());
                    self.is_error = false;
                    
                    // Clear sensitive data
                    self.password.clear();
                    
                    Ok(AuthScreenResult::LoginSuccess(self.username.clone()))
                } else {
                    self.login_attempts += 1;
                    self.message = Some(auth_result.message);
                    self.is_error = true;
                    
                    // Clear password on failed attempt
                    self.password.clear();
                    self.current_field = LoginField::Password;
                    
                    Ok(AuthScreenResult::Continue)
                }
            }
            Err(e) => {
                self.login_attempts += 1;
                self.message = Some(format!("Authentication error: {}", e));
                self.is_error = true;
                
                // Clear password on error
                self.password.clear();
                self.current_field = LoginField::Password;
                
                Ok(AuthScreenResult::Continue)
            }
        }
    }

    /// Move to next field
    fn next_field(&mut self) {
        self.current_field = match self.current_field {
            LoginField::Username => LoginField::Password,
            LoginField::Password => LoginField::Submit,
            LoginField::Submit => LoginField::Username,
        };
    }

    /// Move to previous field
    fn previous_field(&mut self) {
        self.current_field = match self.current_field {
            LoginField::Username => LoginField::Submit,
            LoginField::Password => LoginField::Username,
            LoginField::Submit => LoginField::Password,
        };
    }

    /// Get authentication manager (for use after successful login)
    pub fn get_auth_manager(self) -> TuiAuthManager {
        self.auth_manager
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.auth_manager.is_authenticated()
    }
}

/// Logout confirmation screen
pub struct TuiLogoutScreen {
    auth_manager: TuiAuthManager,
    confirmed: bool,
}

impl TuiLogoutScreen {
    /// Create new logout screen
    pub fn new(auth_manager: TuiAuthManager) -> Self {
        Self {
            auth_manager,
            confirmed: false,
        }
    }

    /// Render logout confirmation
    pub fn render(&self, terminal: &Terminal, theme: &Theme) -> QmsResult<()> {
        terminal.clear_screen()?;

        // Header
        terminal.move_cursor(8, 10)?;
        terminal.set_color(theme.colors.warning, theme.colors.background)?;
        terminal.write_text("🚪 Logout Confirmation")?;

        if let Some(session_info) = self.auth_manager.get_session_info() {
            terminal.move_cursor(10, 10)?;
            terminal.set_color(theme.colors.text_primary, theme.colors.background)?;
            terminal.write_text(&format!("Current User: {}", session_info.username))?;
            
            terminal.move_cursor(11, 10)?;
            terminal.write_text(&format!("Session Duration: {} seconds", session_info.session_duration()))?;
            
            terminal.move_cursor(12, 10)?;
            terminal.write_text(&format!("Roles: {}", session_info.roles.join(", ")))?;
        }

        terminal.move_cursor(15, 10)?;
        terminal.set_color(theme.colors.text_secondary, theme.colors.background)?;
        terminal.write_text("Are you sure you want to logout?")?;

        terminal.move_cursor(17, 10)?;
        terminal.write_text("Type 'yes' to confirm logout, or 'no' to cancel:")?;

        terminal.reset_colors()?;
        terminal.flush()?;
        
        Ok(())
    }

    /// Handle logout confirmation input
    pub fn handle_input(&mut self) -> QmsResult<AuthScreenResult> {
        let mut input = String::new();
        print!("> ");
        io::stdout().flush()?;
        
        if io::stdin().read_line(&mut input).is_ok() {
            let input = input.trim().to_lowercase();
            
            match input.as_str() {
                "yes" | "y" => {
                    match self.auth_manager.logout() {
                        Ok(result) => {
                            if result.success {
                                Ok(AuthScreenResult::LoginFailed(result.message))
                            } else {
                                Ok(AuthScreenResult::Continue)
                            }
                        }
                        Err(e) => {
                            Ok(AuthScreenResult::LoginFailed(format!("Logout error: {}", e)))
                        }
                    }
                }
                "no" | "n" | "cancel" => {
                    Ok(AuthScreenResult::Continue)
                }
                "q" | "quit" => {
                    Ok(AuthScreenResult::Quit)
                }
                _ => {
                    Ok(AuthScreenResult::Continue)
                }
            }
        } else {
            Ok(AuthScreenResult::Continue)
        }
    }
}
