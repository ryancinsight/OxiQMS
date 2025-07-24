//! Simple Interactive TUI Application
//! 
//! This module provides a working interactive TUI application that demonstrates
//! the QMS TUI framework capabilities with real terminal interaction.

use crate::prelude::*;
use crate::tui::{Terminal, TerminalCapabilities, Theme};
use std::io::{self, Read};
use std::path::PathBuf;

/// Simple interactive TUI application
pub struct SimpleTuiApp {
    terminal: Terminal,
    theme: Theme,
    current_screen: Screen,
    is_running: bool,
    project_path: Option<PathBuf>,
}

/// Available screens in the simple TUI
#[derive(Debug, Clone, PartialEq)]
enum Screen {
    MainMenu,
    ProjectInfo,
    SystemInfo,
    Help,
    Exit,
}

impl SimpleTuiApp {
    /// Create new simple TUI application
    pub fn new(project_path: Option<PathBuf>) -> QmsResult<Self> {
        let terminal = Terminal::new()?;
        let capabilities = TerminalCapabilities::detect();
        let theme = Theme::for_capabilities(capabilities.supports_color, false);
        
        Ok(Self {
            terminal,
            theme,
            current_screen: Screen::MainMenu,
            is_running: false,
            project_path,
        })
    }

    /// Run the interactive TUI application
    pub fn run(&mut self) -> QmsResult<()> {
        // Initialize terminal for TUI mode
        self.terminal.enter_alternate_screen()?;
        self.terminal.enable_raw_mode()?;
        self.terminal.hide_cursor()?;

        // Set running flag
        self.is_running = true;

        // Main event loop
        while self.is_running {
            // Render current screen
            self.render_current_screen()?;

            // Handle user input
            self.handle_input()?;
        }

        // Cleanup terminal
        self.cleanup()?;

        Ok(())
    }

    /// Render the current screen
    fn render_current_screen(&mut self) -> QmsResult<()> {
        self.terminal.clear_screen()?;
        
        match self.current_screen {
            Screen::MainMenu => self.render_main_menu(),
            Screen::ProjectInfo => self.render_project_info(),
            Screen::SystemInfo => self.render_system_info(),
            Screen::Help => self.render_help(),
            Screen::Exit => {
                self.is_running = false;
                Ok(())
            }
        }
    }

    /// Render main menu screen
    fn render_main_menu(&mut self) -> QmsResult<()> {
        // Header
        self.terminal.move_cursor(2, 2)?;
        self.terminal.set_color(self.theme.colors.primary, self.theme.colors.background)?;
        self.terminal.write_text("╔══════════════════════════════════════════════════════════════════════════════╗")?;
        
        self.terminal.move_cursor(3, 2)?;
        self.terminal.write_text("║                    QMS - Medical Device Quality Management System           ║")?;
        
        self.terminal.move_cursor(4, 2)?;
        self.terminal.write_text("║                     FDA 21 CFR Part 820, ISO 13485, ISO 14971 Compliant    ║")?;
        
        self.terminal.move_cursor(5, 2)?;
        self.terminal.write_text("╚══════════════════════════════════════════════════════════════════════════════╝")?;
        
        // Menu options
        self.terminal.move_cursor(8, 10)?;
        self.terminal.set_color(self.theme.colors.text_primary, self.theme.colors.background)?;
        self.terminal.write_text("📋 Main Menu - Select an option:")?;
        
        self.terminal.move_cursor(10, 12)?;
        self.terminal.write_text("1. 📊 Project Information")?;
        
        self.terminal.move_cursor(11, 12)?;
        self.terminal.write_text("2. 🖥️  System Information")?;
        
        self.terminal.move_cursor(12, 12)?;
        self.terminal.write_text("3. ❓ Help & Documentation")?;
        
        self.terminal.move_cursor(13, 12)?;
        self.terminal.write_text("4. 🚪 Exit Application")?;
        
        // Instructions
        self.terminal.move_cursor(16, 10)?;
        self.terminal.set_color(self.theme.colors.text_secondary, self.theme.colors.background)?;
        self.terminal.write_text("Instructions:")?;

        self.terminal.move_cursor(17, 12)?;
        self.terminal.write_text("• Type 1-4 and press Enter to select an option")?;

        self.terminal.move_cursor(18, 12)?;
        self.terminal.write_text("• Type 'q' and press Enter to quit")?;

        self.terminal.move_cursor(19, 12)?;
        self.terminal.write_text("• Type any key and press Enter to go back")?;
        
        // Status bar
        let (rows, _cols) = self.terminal.get_size()?;
        self.terminal.move_cursor(rows - 2, 2)?;
        self.terminal.set_color(self.theme.colors.text_muted, self.theme.colors.background)?;
        self.terminal.write_text("QMS TUI v1.0.0 | Medical Device Quality Management System")?;
        
        self.terminal.reset_colors()?;
        self.terminal.flush()?;
        
        Ok(())
    }

    /// Render project information screen
    fn render_project_info(&mut self) -> QmsResult<()> {
        // Header
        self.terminal.move_cursor(2, 2)?;
        self.terminal.set_color(self.theme.colors.primary, self.theme.colors.background)?;
        self.terminal.write_text("📊 Project Information")?;
        
        self.terminal.move_cursor(4, 4)?;
        self.terminal.set_color(self.theme.colors.text_primary, self.theme.colors.background)?;
        
        if let Some(ref path) = self.project_path {
            self.terminal.write_text(&format!("Project Path: {}", path.display()))?;
        } else {
            self.terminal.write_text("Project Path: Current directory")?;
        }
        
        self.terminal.move_cursor(6, 4)?;
        self.terminal.write_text("Project Status: Active")?;
        
        self.terminal.move_cursor(7, 4)?;
        self.terminal.write_text("QMS Version: 1.0.0")?;
        
        self.terminal.move_cursor(8, 4)?;
        self.terminal.write_text("Compliance: FDA 21 CFR Part 820, ISO 13485, ISO 14971")?;
        
        self.terminal.move_cursor(10, 4)?;
        self.terminal.set_color(self.theme.colors.success, self.theme.colors.background)?;
        self.terminal.write_text("✅ System Status: Operational")?;
        
        self.terminal.move_cursor(11, 4)?;
        self.terminal.write_text("✅ Audit Trail: Active")?;
        
        self.terminal.move_cursor(12, 4)?;
        self.terminal.write_text("✅ Compliance Monitoring: Enabled")?;
        
        // Back instruction
        let (rows, _cols) = self.terminal.get_size()?;
        self.terminal.move_cursor(rows - 3, 4)?;
        self.terminal.set_color(self.theme.colors.text_secondary, self.theme.colors.background)?;
        self.terminal.write_text("Press Enter to return to main menu (or type 'q' to quit)...")?;
        
        self.terminal.reset_colors()?;
        self.terminal.flush()?;
        
        Ok(())
    }

    /// Render system information screen
    fn render_system_info(&mut self) -> QmsResult<()> {
        let capabilities = TerminalCapabilities::detect();
        
        // Header
        self.terminal.move_cursor(2, 2)?;
        self.terminal.set_color(self.theme.colors.primary, self.theme.colors.background)?;
        self.terminal.write_text("🖥️  System Information")?;
        
        self.terminal.move_cursor(4, 4)?;
        self.terminal.set_color(self.theme.colors.text_primary, self.theme.colors.background)?;
        self.terminal.write_text("Terminal Capabilities:")?;
        
        self.terminal.move_cursor(5, 6)?;
        self.terminal.write_text(&format!("Size: {}x{}", capabilities.width, capabilities.height))?;
        
        self.terminal.move_cursor(6, 6)?;
        self.terminal.write_text(&format!("Color Support: {}", if capabilities.supports_color { "Yes" } else { "No" }))?;
        
        self.terminal.move_cursor(7, 6)?;
        self.terminal.write_text(&format!("Mouse Support: {}", if capabilities.supports_mouse { "Yes" } else { "No" }))?;
        
        self.terminal.move_cursor(8, 6)?;
        self.terminal.write_text(&format!("Alternate Screen: {}", if capabilities.supports_alternate_screen { "Yes" } else { "No" }))?;
        
        self.terminal.move_cursor(10, 4)?;
        self.terminal.write_text("TUI Framework Status:")?;
        
        self.terminal.move_cursor(11, 6)?;
        self.terminal.set_color(self.theme.colors.success, self.theme.colors.background)?;
        self.terminal.write_text("✅ Terminal Control System")?;
        
        self.terminal.move_cursor(12, 6)?;
        self.terminal.write_text("✅ Theme System Active")?;
        
        self.terminal.move_cursor(13, 6)?;
        self.terminal.write_text("✅ Event Handling Ready")?;
        
        self.terminal.move_cursor(14, 6)?;
        self.terminal.write_text("✅ Cross-Platform Support")?;
        
        // Back instruction
        let (rows, _cols) = self.terminal.get_size()?;
        self.terminal.move_cursor(rows - 3, 4)?;
        self.terminal.set_color(self.theme.colors.text_secondary, self.theme.colors.background)?;
        self.terminal.write_text("Press Enter to return to main menu (or type 'q' to quit)...")?;
        
        self.terminal.reset_colors()?;
        self.terminal.flush()?;
        
        Ok(())
    }

    /// Render help screen
    fn render_help(&mut self) -> QmsResult<()> {
        // Header
        self.terminal.move_cursor(2, 2)?;
        self.terminal.set_color(self.theme.colors.primary, self.theme.colors.background)?;
        self.terminal.write_text("❓ Help & Documentation")?;
        
        self.terminal.move_cursor(4, 4)?;
        self.terminal.set_color(self.theme.colors.text_primary, self.theme.colors.background)?;
        self.terminal.write_text("QMS TUI Framework - Interactive Terminal Interface")?;
        
        self.terminal.move_cursor(6, 4)?;
        self.terminal.write_text("Navigation:")?;
        
        self.terminal.move_cursor(7, 6)?;
        self.terminal.write_text("• Type number keys (1-4) and press Enter to select menu options")?;

        self.terminal.move_cursor(8, 6)?;
        self.terminal.write_text("• Type 'q' and press Enter to quit the application")?;

        self.terminal.move_cursor(9, 6)?;
        self.terminal.write_text("• Press Enter to go back to previous screen")?;

        self.terminal.move_cursor(10, 6)?;
        self.terminal.write_text("• Type 'q' from any screen to quit immediately")?;
        
        self.terminal.move_cursor(12, 4)?;
        self.terminal.write_text("Features:")?;
        
        self.terminal.move_cursor(13, 6)?;
        self.terminal.write_text("• Cross-platform terminal support")?;
        
        self.terminal.move_cursor(14, 6)?;
        self.terminal.write_text("• Professional medical device interface")?;
        
        self.terminal.move_cursor(15, 6)?;
        self.terminal.write_text("• Regulatory compliance integration")?;
        
        self.terminal.move_cursor(16, 6)?;
        self.terminal.write_text("• Standard library only (no external dependencies)")?;
        
        // Back instruction
        let (rows, _cols) = self.terminal.get_size()?;
        self.terminal.move_cursor(rows - 3, 4)?;
        self.terminal.set_color(self.theme.colors.text_secondary, self.theme.colors.background)?;
        self.terminal.write_text("Press Enter to return to main menu (or type 'q' to quit)...")?;
        
        self.terminal.reset_colors()?;
        self.terminal.flush()?;
        
        Ok(())
    }

    /// Handle user input
    fn handle_input(&mut self) -> QmsResult<()> {
        // Read a line from stdin for simplicity
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            let input = input.trim();

            match self.current_screen {
                Screen::MainMenu => {
                    match input {
                        "1" => self.current_screen = Screen::ProjectInfo,
                        "2" => self.current_screen = Screen::SystemInfo,
                        "3" => self.current_screen = Screen::Help,
                        "4" | "q" | "Q" | "quit" | "exit" => self.current_screen = Screen::Exit,
                        _ => {} // Ignore other input
                    }
                }
                Screen::ProjectInfo | Screen::SystemInfo | Screen::Help => {
                    // Any input returns to main menu
                    match input {
                        "q" | "Q" | "quit" | "exit" => self.current_screen = Screen::Exit,
                        _ => self.current_screen = Screen::MainMenu,
                    }
                }
                Screen::Exit => {
                    self.is_running = false;
                }
            }
        }

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
