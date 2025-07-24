//! Authenticated TUI Application
//! 
//! This module provides a fully authenticated TUI application that integrates
//! with the QMS authentication system and provides secure access to QMS functionality.

use crate::prelude::*;
use crate::tui::{Terminal, TerminalCapabilities, Theme};
use crate::tui::auth::{TuiAuthManager, TuiSessionInfo};
use crate::tui::auth_screens::{TuiLoginScreen, TuiLogoutScreen, AuthScreenResult};
use std::path::PathBuf;

/// Authenticated TUI application with full security integration
pub struct AuthenticatedTuiApp {
    terminal: Terminal,
    theme: Theme,
    auth_manager: Option<TuiAuthManager>,
    current_screen: AppScreen,
    is_running: bool,
    project_path: Option<PathBuf>,
    login_screen: Option<TuiLoginScreen>,
}

/// Application screens with authentication states
#[derive(Debug, Clone, PartialEq)]
enum AppScreen {
    Login,
    MainMenu,
    ProjectManagement,
    DocumentControl,
    RiskManagement,
    RequirementsManagement,
    AuditCompliance,
    UserManagement,
    SystemInfo,
    SessionInfo,
    Help,
    Logout,
    Exit,
}

impl AuthenticatedTuiApp {
    /// Create new authenticated TUI application
    pub fn new(project_path: Option<PathBuf>) -> QmsResult<Self> {
        let terminal = Terminal::new()?;
        let capabilities = TerminalCapabilities::detect();
        let theme = Theme::for_capabilities(capabilities.supports_color, false);
        
        Ok(Self {
            terminal,
            theme,
            auth_manager: None,
            current_screen: AppScreen::Login,
            is_running: false,
            project_path,
            login_screen: None,
        })
    }

    /// Run the authenticated TUI application
    pub fn run(&mut self) -> QmsResult<()> {
        // Initialize terminal for TUI mode
        self.terminal.enter_alternate_screen()?;
        self.terminal.enable_raw_mode()?;
        self.terminal.hide_cursor()?;
        
        // Set running flag
        self.is_running = true;
        
        // Initialize login screen
        self.login_screen = Some(TuiLoginScreen::new(self.project_path.clone())?);
        
        // Main event loop
        while self.is_running {
            // Render current screen
            self.render_current_screen()?;
            
            // Handle user input based on current screen
            self.handle_screen_input()?;
        }
        
        // Cleanup terminal
        self.cleanup()?;
        
        Ok(())
    }

    /// Render the current screen
    fn render_current_screen(&mut self) -> QmsResult<()> {
        match self.current_screen {
            AppScreen::Login => {
                if let Some(ref login_screen) = self.login_screen {
                    login_screen.render(&self.terminal, &self.theme)?;
                }
            }
            AppScreen::MainMenu => self.render_authenticated_main_menu()?,
            AppScreen::ProjectManagement => self.render_project_management()?,
            AppScreen::DocumentControl => self.render_document_control()?,
            AppScreen::RiskManagement => self.render_risk_management()?,
            AppScreen::RequirementsManagement => self.render_requirements_management()?,
            AppScreen::AuditCompliance => self.render_audit_compliance()?,
            AppScreen::UserManagement => self.render_user_management()?,
            AppScreen::SystemInfo => self.render_system_info()?,
            AppScreen::SessionInfo => self.render_session_info()?,
            AppScreen::Help => self.render_help()?,
            AppScreen::Logout => self.render_logout_confirmation()?,
            AppScreen::Exit => {
                self.is_running = false;
            }
        }
        
        Ok(())
    }

    /// Handle input for current screen
    fn handle_screen_input(&mut self) -> QmsResult<()> {
        match self.current_screen {
            AppScreen::Login => {
                if let Some(ref mut login_screen) = self.login_screen {
                    match login_screen.handle_input()? {
                        AuthScreenResult::LoginSuccess(username) => {
                            // Move auth manager from login screen to main app
                            if let Some(login_screen) = self.login_screen.take() {
                                self.auth_manager = Some(login_screen.get_auth_manager());
                            }
                            self.current_screen = AppScreen::MainMenu;
                            
                            // Show welcome message
                            self.show_welcome_message(&username)?;
                        }
                        AuthScreenResult::LoginFailed(_) => {
                            // Stay on login screen
                        }
                        AuthScreenResult::Quit => {
                            self.is_running = false;
                        }
                        AuthScreenResult::Continue => {
                            // Continue on login screen
                        }
                    }
                }
            }
            _ => {
                // Handle authenticated screen input
                self.handle_authenticated_input()?;
            }
        }
        
        Ok(())
    }

    /// Handle input for authenticated screens
    fn handle_authenticated_input(&mut self) -> QmsResult<()> {
        use std::io::{self, Write};
        
        let mut input = String::new();
        print!("> ");
        io::stdout().flush()?;
        
        if io::stdin().read_line(&mut input).is_ok() {
            let input = input.trim();
            
            match input {
                "q" | "quit" => {
                    self.current_screen = AppScreen::Logout;
                }
                "1" => self.current_screen = AppScreen::ProjectManagement,
                "2" => self.current_screen = AppScreen::DocumentControl,
                "3" => self.current_screen = AppScreen::RiskManagement,
                "4" => self.current_screen = AppScreen::RequirementsManagement,
                "5" => self.current_screen = AppScreen::AuditCompliance,
                "6" => self.current_screen = AppScreen::UserManagement,
                "7" => self.current_screen = AppScreen::SystemInfo,
                "8" => self.current_screen = AppScreen::SessionInfo,
                "9" => self.current_screen = AppScreen::Help,
                "0" | "menu" => self.current_screen = AppScreen::MainMenu,
                "logout" => self.current_screen = AppScreen::Logout,
                "" => {
                    // Enter key - return to main menu from info screens
                    if matches!(self.current_screen, 
                        AppScreen::ProjectManagement | AppScreen::DocumentControl | 
                        AppScreen::RiskManagement | AppScreen::RequirementsManagement |
                        AppScreen::AuditCompliance | AppScreen::UserManagement |
                        AppScreen::SystemInfo | AppScreen::SessionInfo | AppScreen::Help
                    ) {
                        self.current_screen = AppScreen::MainMenu;
                    }
                }
                _ => {
                    // Handle screen-specific commands
                    self.handle_screen_specific_input(input)?;
                }
            }
        }
        
        Ok(())
    }

    /// Handle screen-specific input commands
    fn handle_screen_specific_input(&mut self, input: &str) -> QmsResult<()> {
        match self.current_screen {
            AppScreen::Logout => {
                match input.to_lowercase().as_str() {
                    "yes" | "y" => {
                        if let Some(ref mut auth_manager) = self.auth_manager {
                            let _ = auth_manager.logout();
                        }
                        self.current_screen = AppScreen::Login;
                        self.login_screen = Some(TuiLoginScreen::new(self.project_path.clone())?);
                        self.auth_manager = None;
                    }
                    "no" | "n" => {
                        self.current_screen = AppScreen::MainMenu;
                    }
                    _ => {
                        // Invalid input, stay on logout screen
                    }
                }
            }
            _ => {
                // No specific handling for other screens yet
            }
        }
        
        Ok(())
    }

    /// Show welcome message after successful login
    fn show_welcome_message(&mut self, username: &str) -> QmsResult<()> {
        self.terminal.clear_screen()?;
        
        // Welcome header
        self.terminal.move_cursor(8, 10)?;
        self.terminal.set_color(self.theme.colors.success, self.theme.colors.background)?;
        self.terminal.write_text(&format!("✅ Welcome, {}!", username))?;
        
        self.terminal.move_cursor(10, 10)?;
        self.terminal.set_color(self.theme.colors.text_primary, self.theme.colors.background)?;
        self.terminal.write_text("You have successfully logged into the QMS system.")?;
        
        self.terminal.move_cursor(11, 10)?;
        self.terminal.write_text("All actions will be logged for compliance and audit purposes.")?;
        
        self.terminal.move_cursor(13, 10)?;
        self.terminal.set_color(self.theme.colors.text_secondary, self.theme.colors.background)?;
        self.terminal.write_text("Press Enter to continue to the main menu...")?;
        
        self.terminal.reset_colors()?;
        self.terminal.flush()?;
        
        // Wait for user to press Enter
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
        
        Ok(())
    }

    /// Render authenticated main menu
    fn render_authenticated_main_menu(&mut self) -> QmsResult<()> {
        self.terminal.clear_screen()?;

        // Header with user info
        self.terminal.move_cursor(2, 2)?;
        self.terminal.set_color(self.theme.colors.primary, self.theme.colors.background)?;
        self.terminal.write_text("╔══════════════════════════════════════════════════════════════════════════════╗")?;
        
        self.terminal.move_cursor(3, 2)?;
        self.terminal.write_text("║                    QMS - Medical Device Quality Management System           ║")?;
        
        // Show current user
        if let Some(ref auth_manager) = self.auth_manager {
            if let Some(username) = auth_manager.current_username() {
                self.terminal.move_cursor(4, 2)?;
                self.terminal.write_text(&format!("║                              Logged in as: {:<20}           ║", username))?;
            }
        }
        
        self.terminal.move_cursor(5, 2)?;
        self.terminal.write_text("╚══════════════════════════════════════════════════════════════════════════════╝")?;

        // Menu options
        self.terminal.move_cursor(8, 10)?;
        self.terminal.set_color(self.theme.colors.text_primary, self.theme.colors.background)?;
        self.terminal.write_text("📋 Main Menu - Select an option:")?;
        
        self.terminal.move_cursor(10, 12)?;
        self.terminal.write_text("1. 📊 Project Management")?;
        
        self.terminal.move_cursor(11, 12)?;
        self.terminal.write_text("2. 📄 Document Control")?;
        
        self.terminal.move_cursor(12, 12)?;
        self.terminal.write_text("3. ⚠️  Risk Management (ISO 14971)")?;
        
        self.terminal.move_cursor(13, 12)?;
        self.terminal.write_text("4. 📋 Requirements Management")?;
        
        self.terminal.move_cursor(14, 12)?;
        self.terminal.write_text("5. 🔍 Audit & Compliance")?;
        
        self.terminal.move_cursor(15, 12)?;
        self.terminal.write_text("6. 👥 User Management")?;
        
        self.terminal.move_cursor(16, 12)?;
        self.terminal.write_text("7. 🖥️  System Information")?;
        
        self.terminal.move_cursor(17, 12)?;
        self.terminal.write_text("8. 🔐 Session Information")?;
        
        self.terminal.move_cursor(18, 12)?;
        self.terminal.write_text("9. ❓ Help & Documentation")?;

        // Instructions
        self.terminal.move_cursor(21, 10)?;
        self.terminal.set_color(self.theme.colors.text_secondary, self.theme.colors.background)?;
        self.terminal.write_text("Instructions:")?;
        
        self.terminal.move_cursor(22, 12)?;
        self.terminal.write_text("• Type 1-9 and press Enter to select an option")?;
        
        self.terminal.move_cursor(23, 12)?;
        self.terminal.write_text("• Type 'logout' to logout")?;
        
        self.terminal.move_cursor(24, 12)?;
        self.terminal.write_text("• Type 'q' to quit")?;

        // Status bar
        let (rows, _cols) = self.terminal.get_size()?;
        self.terminal.move_cursor(rows - 2, 2)?;
        self.terminal.set_color(self.theme.colors.text_muted, self.theme.colors.background)?;
        self.terminal.write_text("QMS TUI v1.0.0 | Medical Device Quality Management System | Authenticated Session")?;
        
        self.terminal.reset_colors()?;
        self.terminal.flush()?;
        
        Ok(())
    }

    /// Render session information screen
    fn render_session_info(&mut self) -> QmsResult<()> {
        self.terminal.clear_screen()?;
        
        self.terminal.move_cursor(2, 2)?;
        self.terminal.set_color(self.theme.colors.primary, self.theme.colors.background)?;
        self.terminal.write_text("🔐 Session Information")?;
        
        if let Some(ref auth_manager) = self.auth_manager {
            if let Some(session_info) = auth_manager.get_session_info() {
                self.terminal.move_cursor(4, 4)?;
                self.terminal.set_color(self.theme.colors.text_primary, self.theme.colors.background)?;
                self.terminal.write_text(&format!("Username: {}", session_info.username))?;
                
                self.terminal.move_cursor(5, 4)?;
                self.terminal.write_text(&format!("Session ID: {}", session_info.session_id))?;
                
                self.terminal.move_cursor(6, 4)?;
                self.terminal.write_text(&format!("Login Time: {}", session_info.format_login_time()))?;
                
                self.terminal.move_cursor(7, 4)?;
                self.terminal.write_text(&format!("Last Activity: {}", session_info.format_last_activity()))?;
                
                self.terminal.move_cursor(8, 4)?;
                self.terminal.write_text(&format!("Session Duration: {} seconds", session_info.session_duration()))?;
                
                self.terminal.move_cursor(10, 4)?;
                self.terminal.write_text("Roles:")?;
                for (i, role) in session_info.roles.iter().enumerate() {
                    self.terminal.move_cursor(11 + i as u16, 6)?;
                    self.terminal.write_text(&format!("• {}", role))?;
                }
                
                self.terminal.move_cursor(13 + session_info.roles.len() as u16, 4)?;
                self.terminal.write_text("Permissions:")?;
                for (i, permission) in session_info.permissions.iter().enumerate() {
                    self.terminal.move_cursor(14 + session_info.roles.len() as u16 + i as u16, 6)?;
                    self.terminal.write_text(&format!("• {}", permission))?;
                }
            }
        }
        
        // Back instruction
        let (rows, _cols) = self.terminal.get_size()?;
        self.terminal.move_cursor(rows - 3, 4)?;
        self.terminal.set_color(self.theme.colors.text_secondary, self.theme.colors.background)?;
        self.terminal.write_text("Press Enter to return to main menu...")?;
        
        self.terminal.reset_colors()?;
        self.terminal.flush()?;
        
        Ok(())
    }

    /// Render logout confirmation
    fn render_logout_confirmation(&mut self) -> QmsResult<()> {
        self.terminal.clear_screen()?;
        
        self.terminal.move_cursor(8, 10)?;
        self.terminal.set_color(self.theme.colors.warning, self.theme.colors.background)?;
        self.terminal.write_text("🚪 Logout Confirmation")?;
        
        if let Some(ref auth_manager) = self.auth_manager {
            if let Some(username) = auth_manager.current_username() {
                self.terminal.move_cursor(10, 10)?;
                self.terminal.set_color(self.theme.colors.text_primary, self.theme.colors.background)?;
                self.terminal.write_text(&format!("Current User: {}", username))?;
            }
        }
        
        self.terminal.move_cursor(12, 10)?;
        self.terminal.set_color(self.theme.colors.text_secondary, self.theme.colors.background)?;
        self.terminal.write_text("Are you sure you want to logout?")?;
        
        self.terminal.move_cursor(14, 10)?;
        self.terminal.write_text("Type 'yes' to confirm logout, or 'no' to cancel:")?;
        
        self.terminal.reset_colors()?;
        self.terminal.flush()?;
        
        Ok(())
    }

    /// Placeholder method for other screens
    fn render_project_management(&mut self) -> QmsResult<()> {
        self.render_placeholder_screen("📊 Project Management", "Manage QMS projects and configurations")
    }

    fn render_document_control(&mut self) -> QmsResult<()> {
        self.render_placeholder_screen("📄 Document Control", "Manage controlled documents and procedures")
    }

    fn render_risk_management(&mut self) -> QmsResult<()> {
        self.render_placeholder_screen("⚠️  Risk Management", "ISO 14971 risk analysis and management")
    }

    fn render_requirements_management(&mut self) -> QmsResult<()> {
        self.render_placeholder_screen("📋 Requirements Management", "Manage system and user requirements")
    }

    fn render_audit_compliance(&mut self) -> QmsResult<()> {
        self.render_placeholder_screen("🔍 Audit & Compliance", "Audit logs and compliance reporting")
    }

    fn render_user_management(&mut self) -> QmsResult<()> {
        self.render_placeholder_screen("👥 User Management", "Manage users, roles, and permissions")
    }

    fn render_system_info(&mut self) -> QmsResult<()> {
        self.render_placeholder_screen("🖥️  System Information", "System status and configuration")
    }

    fn render_help(&mut self) -> QmsResult<()> {
        self.render_placeholder_screen("❓ Help & Documentation", "User guide and system information")
    }

    /// Render placeholder screen for unimplemented features
    fn render_placeholder_screen(&mut self, title: &str, description: &str) -> QmsResult<()> {
        self.terminal.clear_screen()?;
        
        self.terminal.move_cursor(2, 2)?;
        self.terminal.set_color(self.theme.colors.primary, self.theme.colors.background)?;
        self.terminal.write_text(title)?;
        
        self.terminal.move_cursor(4, 4)?;
        self.terminal.set_color(self.theme.colors.text_primary, self.theme.colors.background)?;
        self.terminal.write_text(description)?;
        
        self.terminal.move_cursor(6, 4)?;
        self.terminal.set_color(self.theme.colors.text_secondary, self.theme.colors.background)?;
        self.terminal.write_text("This feature is under development.")?;
        
        self.terminal.move_cursor(8, 4)?;
        self.terminal.write_text("Available in future releases with full QMS functionality.")?;
        
        // Back instruction
        let (rows, _cols) = self.terminal.get_size()?;
        self.terminal.move_cursor(rows - 3, 4)?;
        self.terminal.write_text("Press Enter to return to main menu...")?;
        
        self.terminal.reset_colors()?;
        self.terminal.flush()?;
        
        Ok(())
    }

    /// Cleanup terminal state
    fn cleanup(&mut self) -> QmsResult<()> {
        self.terminal.show_cursor()?;
        self.terminal.disable_raw_mode()?;
        self.terminal.exit_alternate_screen()?;
        self.terminal.reset_colors()?;
        
        println!("👋 Thank you for using QMS Medical Device Quality Management System");
        
        Ok(())
    }
}
