//! Screen Management for QMS TUI
//! 
//! This module provides screen management and individual screen implementations
//! for the QMS terminal user interface.

use crate::prelude::*;
use crate::tui::{Terminal, Theme, Event, KeyEvent};
use crate::tui::layout::{Region, Layout, QmsLayouts};
use crate::tui::widgets::{Widget, MenuWidget, FormWidget, MenuItem, FormField, FormFieldType};
use std::collections::HashMap;

// Placeholder for CommandResult when interfaces module is not available
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
}

/// Base trait for all screens
pub trait Screen {
    /// Get screen name/identifier
    fn name(&self) -> &str;
    
    /// Render the screen
    fn render(&mut self, terminal: &Terminal, region: &Region, theme: &Theme) -> QmsResult<()>;
    
    /// Handle events for this screen
    fn handle_event(&mut self, event: &Event) -> QmsResult<ScreenAction>;
    
    /// Called when screen becomes active
    fn on_enter(&mut self) -> QmsResult<()> {
        Ok(())
    }
    
    /// Called when screen becomes inactive
    fn on_exit(&mut self) -> QmsResult<()> {
        Ok(())
    }
    
    /// Check if screen requires authentication
    fn requires_auth(&self) -> bool {
        true
    }
}

/// Actions that screens can request
#[derive(Debug, Clone, PartialEq)]
pub enum ScreenAction {
    None,
    NavigateTo(String),
    NavigateBack,
    Quit,
    ExecuteCommand(String, Vec<String>),
    ShowMessage(String),
    ShowError(String),
}

/// Screen manager for handling screen navigation and lifecycle
pub struct ScreenManager {
    screens: HashMap<String, Box<dyn Screen>>,
    screen_stack: Vec<String>,
    current_screen: Option<String>,
}

impl ScreenManager {
    /// Create new screen manager
    pub fn new() -> Self {
        let mut manager = Self {
            screens: HashMap::new(),
            screen_stack: Vec::new(),
            current_screen: None,
        };
        
        // Register default screens
        manager.register_default_screens();
        
        manager
    }

    /// Register default QMS screens
    fn register_default_screens(&mut self) {
        self.add_screen(Box::new(LoginScreen::new()));
        self.add_screen(Box::new(MainMenuScreen::new()));
        self.add_screen(Box::new(ProjectScreen::new()));
        self.add_screen(Box::new(DocumentsScreen::new()));
        self.add_screen(Box::new(RisksScreen::new()));
        self.add_screen(Box::new(RequirementsScreen::new()));
        self.add_screen(Box::new(AuditScreen::new()));
        self.add_screen(Box::new(HelpScreen::new()));
    }

    /// Add screen to manager
    pub fn add_screen(&mut self, screen: Box<dyn Screen>) {
        let name = screen.name().to_string();
        self.screens.insert(name, screen);
    }

    /// Navigate to screen
    pub fn navigate_to(&mut self, screen_name: &str) -> QmsResult<()> {
        if !self.screens.contains_key(screen_name) {
            return Err(QmsError::validation_error(&format!("Screen '{}' not found", screen_name)));
        }

        // Exit current screen
        if let Some(ref current) = self.current_screen {
            if let Some(screen) = self.screens.get_mut(current) {
                screen.on_exit()?;
            }
            self.screen_stack.push(current.clone());
        }

        // Enter new screen
        self.current_screen = Some(screen_name.to_string());
        if let Some(screen) = self.screens.get_mut(screen_name) {
            screen.on_enter()?;
        }

        Ok(())
    }

    /// Navigate back to previous screen
    pub fn navigate_back(&mut self) -> QmsResult<()> {
        if let Some(previous) = self.screen_stack.pop() {
            // Exit current screen
            if let Some(ref current) = self.current_screen {
                if let Some(screen) = self.screens.get_mut(current) {
                    screen.on_exit()?;
                }
            }

            // Enter previous screen
            self.current_screen = Some(previous.clone());
            if let Some(screen) = self.screens.get_mut(&previous) {
                screen.on_enter()?;
            }
        }

        Ok(())
    }

    /// Get current screen
    pub fn current_screen(&mut self) -> Option<&mut Box<dyn Screen>> {
        self.current_screen.as_ref().and_then(|name| self.screens.get_mut(name))
    }

    /// Get current screen name
    pub fn current_screen_name(&self) -> Option<&str> {
        self.current_screen.as_deref()
    }
}

impl Default for ScreenManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Login screen for user authentication
pub struct LoginScreen {
    form: FormWidget,
    message: Option<String>,
    is_error: bool,
}

impl LoginScreen {
    pub fn new() -> Self {
        let mut form = FormWidget::new("QMS Login".to_string());
        
        form.add_field(FormField {
            id: "username".to_string(),
            label: "Username".to_string(),
            value: String::new(),
            field_type: FormFieldType::Text,
            required: true,
            error: None,
        });
        
        form.add_field(FormField {
            id: "password".to_string(),
            label: "Password".to_string(),
            value: String::new(),
            field_type: FormFieldType::Password,
            required: true,
            error: None,
        });

        Self {
            form,
            message: None,
            is_error: false,
        }
    }
}

impl Screen for LoginScreen {
    fn name(&self) -> &str {
        "login"
    }

    fn render(&mut self, terminal: &Terminal, region: &Region, theme: &Theme) -> QmsResult<()> {
        let layout = QmsLayouts::form();
        let regions = layout.split(region);

        // Header
        if let Some(header_region) = regions.get(0) {
            terminal.move_cursor(header_region.y + 1, header_region.x + 2)?;
            terminal.set_color(theme.colors.primary, theme.colors.background)?;
            terminal.write_text("🔐 Please log in to access QMS")?;
            terminal.reset_colors()?;
        }

        // Form
        if let Some(form_region) = regions.get(1) {
            self.form.set_focus(true);
            self.form.render(terminal, form_region, theme)?;
        }

        // Instructions
        if let Some(button_region) = regions.get(2) {
            terminal.move_cursor(button_region.y, button_region.x + 2)?;
            terminal.set_color(theme.colors.text_secondary, theme.colors.background)?;
            terminal.write_text("Press Enter to login, Escape to quit")?;
            terminal.reset_colors()?;
        }

        // Status message
        if let Some(status_region) = regions.get(3) {
            if let Some(ref message) = self.message {
                terminal.move_cursor(status_region.y, status_region.x + 2)?;
                let color = if self.is_error { theme.colors.error } else { theme.colors.success };
                terminal.set_color(color, theme.colors.background)?;
                terminal.write_text(message)?;
                terminal.reset_colors()?;
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> QmsResult<ScreenAction> {
        match event {
            Event::Key(key_event) => {
                match key_event.key.as_str() {
                    "Enter" => {
                        if self.form.validate() {
                            let values = self.form.get_values();
                            let username = values.get("username").unwrap_or(&String::new()).clone();
                            let password = values.get("password").unwrap_or(&String::new()).clone();
                            
                            // Return command to authenticate
                            return Ok(ScreenAction::ExecuteCommand("login".to_string(), vec![username, password]));
                        } else {
                            self.message = Some("Please fill in all required fields".to_string());
                            self.is_error = true;
                        }
                    }
                    "Escape" => {
                        return Ok(ScreenAction::Quit);
                    }
                    _ => {
                        // Pass event to form
                        self.form.handle_event(event)?;
                        // Clear message on input
                        self.message = None;
                    }
                }
            }
            _ => {
                self.form.handle_event(event)?;
            }
        }

        Ok(ScreenAction::None)
    }

    fn requires_auth(&self) -> bool {
        false
    }
}

/// Main menu screen
pub struct MainMenuScreen {
    menu: MenuWidget,
}

impl MainMenuScreen {
    pub fn new() -> Self {
        let mut menu = MenuWidget::new("QMS Main Menu".to_string());
        
        menu.add_item(MenuItem::new("projects".to_string(), "Project Management".to_string())
            .with_description("Manage QMS projects and configurations".to_string())
            .with_shortcut('1'));
        
        menu.add_item(MenuItem::new("documents".to_string(), "Document Control".to_string())
            .with_description("Manage controlled documents and procedures".to_string())
            .with_shortcut('2'));
        
        menu.add_item(MenuItem::new("risks".to_string(), "Risk Management".to_string())
            .with_description("ISO 14971 risk analysis and management".to_string())
            .with_shortcut('3'));
        
        menu.add_item(MenuItem::new("requirements".to_string(), "Requirements Management".to_string())
            .with_description("Manage system and user requirements".to_string())
            .with_shortcut('4'));
        
        menu.add_item(MenuItem::new("audit".to_string(), "Audit & Compliance".to_string())
            .with_description("Audit logs and compliance reporting".to_string())
            .with_shortcut('5'));
        
        menu.add_item(MenuItem::new("help".to_string(), "Help & Documentation".to_string())
            .with_description("User guide and system information".to_string())
            .with_shortcut('h'));
        
        menu.add_item(MenuItem::new("logout".to_string(), "Logout".to_string())
            .with_description("Log out of QMS system".to_string())
            .with_shortcut('q'));

        Self { menu }
    }
}

impl Screen for MainMenuScreen {
    fn name(&self) -> &str {
        "main_menu"
    }

    fn render(&mut self, terminal: &Terminal, region: &Region, theme: &Theme) -> QmsResult<()> {
        let layout = QmsLayouts::main_menu();
        let regions = layout.split(region);

        // Header
        if let Some(header_region) = regions.get(0) {
            terminal.move_cursor(header_region.y + 1, header_region.x + 2)?;
            terminal.set_color(theme.colors.primary, theme.colors.background)?;
            terminal.write_text("🏥 QMS - Medical Device Quality Management System")?;
            
            terminal.move_cursor(header_region.y + 2, header_region.x + 2)?;
            terminal.set_color(theme.colors.text_secondary, theme.colors.background)?;
            terminal.write_text("FDA 21 CFR Part 820, ISO 13485, ISO 14971 Compliant")?;
            terminal.reset_colors()?;
        }

        // Menu
        if let Some(menu_region) = regions.get(1) {
            self.menu.set_focus(true);
            self.menu.render(terminal, menu_region, theme)?;
        }

        // Status
        if let Some(status_region) = regions.get(2) {
            terminal.move_cursor(status_region.y, status_region.x + 2)?;
            terminal.set_color(theme.colors.text_muted, theme.colors.background)?;
            terminal.write_text("Use arrow keys to navigate, Enter to select, Escape to go back")?;
            terminal.reset_colors()?;
        }

        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> QmsResult<ScreenAction> {
        match event {
            Event::Key(key_event) => {
                match key_event.key.as_str() {
                    "Enter" | "Space" => {
                        if let Some(item) = self.menu.selected_item() {
                            match item.id.as_str() {
                                "logout" => return Ok(ScreenAction::ExecuteCommand("logout".to_string(), vec![])),
                                screen_id => return Ok(ScreenAction::NavigateTo(screen_id.to_string())),
                            }
                        }
                    }
                    "Escape" => {
                        return Ok(ScreenAction::NavigateTo("login".to_string()));
                    }
                    _ => {
                        self.menu.handle_event(event)?;
                    }
                }
            }
            _ => {
                self.menu.handle_event(event)?;
            }
        }

        Ok(ScreenAction::None)
    }
}

// Placeholder screen implementations
macro_rules! impl_placeholder_screen {
    ($name:ident, $screen_name:expr, $title:expr) => {
        pub struct $name {
            message: String,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    message: format!("Welcome to {}", $title),
                }
            }
        }

        impl Screen for $name {
            fn name(&self) -> &str {
                $screen_name
            }

            fn render(&mut self, terminal: &Terminal, region: &Region, theme: &Theme) -> QmsResult<()> {
                terminal.move_cursor(region.y + 2, region.x + 2)?;
                terminal.set_color(theme.colors.primary, theme.colors.background)?;
                terminal.write_text(&format!("📋 {}", $title))?;
                
                terminal.move_cursor(region.y + 4, region.x + 2)?;
                terminal.set_color(theme.colors.text_primary, theme.colors.background)?;
                terminal.write_text(&self.message)?;
                
                terminal.move_cursor(region.y + 6, region.x + 2)?;
                terminal.set_color(theme.colors.text_secondary, theme.colors.background)?;
                terminal.write_text("This screen is under development.")?;
                
                terminal.move_cursor(region.y + 8, region.x + 2)?;
                terminal.write_text("Press Escape to return to main menu.")?;
                
                terminal.reset_colors()?;
                Ok(())
            }

            fn handle_event(&mut self, event: &Event) -> QmsResult<ScreenAction> {
                match event {
                    Event::Key(key_event) => {
                        match key_event.key.as_str() {
                            "Escape" => Ok(ScreenAction::NavigateBack),
                            _ => Ok(ScreenAction::None),
                        }
                    }
                    _ => Ok(ScreenAction::None),
                }
            }
        }
    };
}

impl_placeholder_screen!(ProjectScreen, "projects", "Project Management");
impl_placeholder_screen!(DocumentsScreen, "documents", "Document Control");
impl_placeholder_screen!(RisksScreen, "risks", "Risk Management");
impl_placeholder_screen!(RequirementsScreen, "requirements", "Requirements Management");
impl_placeholder_screen!(AuditScreen, "audit", "Audit & Compliance");
impl_placeholder_screen!(HelpScreen, "help", "Help & Documentation");
