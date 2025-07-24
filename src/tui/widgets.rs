//! Widget System for QMS TUI
//! 
//! This module provides reusable UI components (widgets) for building
//! consistent and professional terminal user interfaces.

use crate::prelude::*;
use crate::tui::{Terminal, Theme, Event, KeyEvent};
use crate::tui::layout::Region;
use std::collections::HashMap;

/// Base trait for all TUI widgets
pub trait Widget {
    /// Render the widget in the given region
    fn render(&self, terminal: &Terminal, region: &Region, theme: &Theme) -> QmsResult<()>;
    
    /// Handle events for this widget
    fn handle_event(&mut self, event: &Event) -> QmsResult<bool>;
    
    /// Get the minimum size required by this widget
    fn min_size(&self) -> (u16, u16);
    
    /// Check if widget can receive focus
    fn can_focus(&self) -> bool {
        false
    }
    
    /// Set focus state
    fn set_focus(&mut self, focused: bool) {
        // Default implementation does nothing
    }
    
    /// Check if widget is focused
    fn is_focused(&self) -> bool {
        false
    }
}

/// Widget renderer helper for common rendering operations
pub struct WidgetRenderer;

impl WidgetRenderer {
    /// Draw a border around a region
    pub fn draw_border(
        terminal: &Terminal,
        region: &Region,
        theme: &Theme,
        title: Option<&str>,
        focused: bool,
    ) -> QmsResult<()> {
        let style = if focused {
            theme.styles.border_focused
        } else {
            theme.styles.border_normal
        };
        
        terminal.set_color(style.0, style.1)?;
        
        // Top border
        terminal.move_cursor(region.y, region.x)?;
        terminal.write_text("┌")?;
        for _ in 1..region.width - 1 {
            terminal.write_text("─")?;
        }
        terminal.write_text("┐")?;
        
        // Title if provided
        if let Some(title) = title {
            let title_x = region.x + (region.width - title.len() as u16) / 2;
            terminal.move_cursor(region.y, title_x)?;
            terminal.write_text(&format!(" {} ", title))?;
        }
        
        // Side borders
        for row in 1..region.height - 1 {
            terminal.move_cursor(region.y + row, region.x)?;
            terminal.write_text("│")?;
            terminal.move_cursor(region.y + row, region.x + region.width - 1)?;
            terminal.write_text("│")?;
        }
        
        // Bottom border
        terminal.move_cursor(region.y + region.height - 1, region.x)?;
        terminal.write_text("└")?;
        for _ in 1..region.width - 1 {
            terminal.write_text("─")?;
        }
        terminal.write_text("┘")?;
        
        terminal.reset_colors()?;
        Ok(())
    }

    /// Clear a region
    pub fn clear_region(terminal: &Terminal, region: &Region) -> QmsResult<()> {
        for row in 0..region.height {
            terminal.move_cursor(region.y + row, region.x)?;
            for _ in 0..region.width {
                terminal.write_text(" ")?;
            }
        }
        Ok(())
    }

    /// Draw text with word wrapping
    pub fn draw_text_wrapped(
        terminal: &Terminal,
        region: &Region,
        text: &str,
        theme: &Theme,
        style: (crate::tui::theme::Color, crate::tui::theme::Color, crate::tui::theme::Style),
    ) -> QmsResult<()> {
        terminal.set_color(style.0, style.1)?;
        
        let mut current_row = 0;
        let mut current_col = 0;
        
        for word in text.split_whitespace() {
            if current_col + word.len() as u16 > region.width {
                current_row += 1;
                current_col = 0;
                
                if current_row >= region.height {
                    break;
                }
            }
            
            terminal.move_cursor(region.y + current_row, region.x + current_col)?;
            terminal.write_text(word)?;
            current_col += word.len() as u16 + 1; // +1 for space
        }
        
        terminal.reset_colors()?;
        Ok(())
    }
}

/// Menu widget for displaying selectable options
pub struct MenuWidget {
    items: Vec<MenuItem>,
    selected_index: usize,
    focused: bool,
    title: String,
}

/// Menu item data
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub shortcut: Option<char>,
}

impl MenuItem {
    /// Create new menu item
    pub fn new(id: String, label: String) -> Self {
        Self {
            id,
            label,
            description: None,
            enabled: true,
            shortcut: None,
        }
    }

    /// Add description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Add keyboard shortcut
    pub fn with_shortcut(mut self, shortcut: char) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    /// Set enabled state
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl MenuWidget {
    /// Create new menu widget
    pub fn new(title: String) -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
            focused: false,
            title,
        }
    }

    /// Add menu item
    pub fn add_item(&mut self, item: MenuItem) {
        self.items.push(item);
    }

    /// Get selected item
    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.items.get(self.selected_index)
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = self.items.len().saturating_sub(1);
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.selected_index < self.items.len().saturating_sub(1) {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }

    /// Select item by shortcut
    pub fn select_by_shortcut(&mut self, shortcut: char) -> bool {
        for (index, item) in self.items.iter().enumerate() {
            if item.shortcut == Some(shortcut) && item.enabled {
                self.selected_index = index;
                return true;
            }
        }
        false
    }
}

impl Widget for MenuWidget {
    fn render(&self, terminal: &Terminal, region: &Region, theme: &Theme) -> QmsResult<()> {
        // Draw border
        WidgetRenderer::draw_border(terminal, region, theme, Some(&self.title), self.focused)?;
        
        // Calculate content area
        let content_region = Region {
            x: region.x + 1,
            y: region.y + 1,
            width: region.width - 2,
            height: region.height - 2,
        };
        
        // Render menu items
        for (index, item) in self.items.iter().enumerate() {
            if index as u16 >= content_region.height {
                break;
            }
            
            let row = content_region.y + index as u16;
            terminal.move_cursor(row, content_region.x)?;
            
            // Choose style based on selection and enabled state
            let style = if index == self.selected_index {
                theme.styles.menu_item_selected
            } else if !item.enabled {
                theme.styles.menu_item_disabled
            } else {
                theme.styles.menu_item
            };
            
            terminal.set_color(style.0, style.1)?;
            
            // Format menu item
            let mut item_text = String::new();
            
            // Add selection indicator
            if index == self.selected_index {
                item_text.push_str("► ");
            } else {
                item_text.push_str("  ");
            }
            
            // Add shortcut if available
            if let Some(shortcut) = item.shortcut {
                item_text.push_str(&format!("{}. ", shortcut));
            }
            
            // Add label
            item_text.push_str(&item.label);
            
            // Truncate if too long
            if item_text.len() > content_region.width as usize {
                item_text.truncate(content_region.width as usize - 3);
                item_text.push_str("...");
            }
            
            terminal.write_text(&item_text)?;
            
            // Add description if available and there's space
            if let Some(ref description) = item.description {
                let desc_x = content_region.x + item_text.len() as u16 + 2;
                if desc_x < content_region.x + content_region.width {
                    terminal.move_cursor(row, desc_x)?;
                    terminal.set_color(theme.colors.text_muted, style.1)?;
                    
                    let remaining_width = content_region.width - (desc_x - content_region.x);
                    let mut desc_text = description.clone();
                    if desc_text.len() > remaining_width as usize {
                        desc_text.truncate(remaining_width as usize - 3);
                        desc_text.push_str("...");
                    }
                    
                    terminal.write_text(&desc_text)?;
                }
            }
        }
        
        terminal.reset_colors()?;
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> QmsResult<bool> {
        if !self.focused {
            return Ok(false);
        }

        match event {
            Event::Key(key_event) => {
                match key_event.key.as_str() {
                    "Up" => {
                        self.select_previous();
                        Ok(true)
                    }
                    "Down" => {
                        self.select_next();
                        Ok(true)
                    }
                    key if key.len() == 1 => {
                        let ch = key.chars().next().unwrap();
                        Ok(self.select_by_shortcut(ch))
                    }
                    _ => Ok(false),
                }
            }
            _ => Ok(false),
        }
    }

    fn min_size(&self) -> (u16, u16) {
        let width = self.items.iter()
            .map(|item| item.label.len() + 4) // 4 for selection indicator and shortcut
            .max()
            .unwrap_or(10) as u16;
        let height = self.items.len() as u16 + 2; // +2 for border
        
        (width.max(self.title.len() as u16 + 4), height)
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn is_focused(&self) -> bool {
        self.focused
    }
}

/// Form widget for input fields
pub struct FormWidget {
    fields: Vec<FormField>,
    current_field: usize,
    focused: bool,
    title: String,
}

/// Form field data
#[derive(Debug, Clone)]
pub struct FormField {
    pub id: String,
    pub label: String,
    pub value: String,
    pub field_type: FormFieldType,
    pub required: bool,
    pub error: Option<String>,
}

/// Form field types
#[derive(Debug, Clone, PartialEq)]
pub enum FormFieldType {
    Text,
    Password,
    Number,
    Email,
    TextArea,
}

impl FormWidget {
    /// Create new form widget
    pub fn new(title: String) -> Self {
        Self {
            fields: Vec::new(),
            current_field: 0,
            focused: false,
            title,
        }
    }

    /// Add form field
    pub fn add_field(&mut self, field: FormField) {
        self.fields.push(field);
    }

    /// Get field values as HashMap
    pub fn get_values(&self) -> HashMap<String, String> {
        self.fields.iter()
            .map(|field| (field.id.clone(), field.value.clone()))
            .collect()
    }

    /// Validate all fields
    pub fn validate(&mut self) -> bool {
        let mut all_valid = true;
        
        for field in &mut self.fields {
            field.error = None;
            
            if field.required && field.value.trim().is_empty() {
                field.error = Some("This field is required".to_string());
                all_valid = false;
            }
            
            // Type-specific validation
            match field.field_type {
                FormFieldType::Email => {
                    if !field.value.is_empty() && !field.value.contains('@') {
                        field.error = Some("Invalid email format".to_string());
                        all_valid = false;
                    }
                }
                FormFieldType::Number => {
                    if !field.value.is_empty() && field.value.parse::<f64>().is_err() {
                        field.error = Some("Must be a valid number".to_string());
                        all_valid = false;
                    }
                }
                _ => {}
            }
        }
        
        all_valid
    }
}

impl Widget for FormWidget {
    fn render(&self, terminal: &Terminal, region: &Region, theme: &Theme) -> QmsResult<()> {
        // Draw border
        WidgetRenderer::draw_border(terminal, region, theme, Some(&self.title), self.focused)?;
        
        // Calculate content area
        let content_region = Region {
            x: region.x + 1,
            y: region.y + 1,
            width: region.width - 2,
            height: region.height - 2,
        };
        
        // Render form fields
        let mut current_row = 0;
        
        for (index, field) in self.fields.iter().enumerate() {
            if current_row >= content_region.height {
                break;
            }
            
            // Render label
            terminal.move_cursor(content_region.y + current_row, content_region.x)?;
            terminal.set_color(theme.styles.form_label.0, theme.styles.form_label.1)?;
            
            let label_text = if field.required {
                format!("{}*:", field.label)
            } else {
                format!("{}:", field.label)
            };
            
            terminal.write_text(&label_text)?;
            current_row += 1;
            
            if current_row >= content_region.height {
                break;
            }
            
            // Render input field
            terminal.move_cursor(content_region.y + current_row, content_region.x)?;
            
            let is_current = index == self.current_field && self.focused;
            let style = if field.error.is_some() {
                theme.styles.form_input_error
            } else if is_current {
                theme.styles.form_input_focused
            } else {
                theme.styles.form_input
            };
            
            terminal.set_color(style.0, style.1)?;
            
            // Display value (mask password fields)
            let display_value = if field.field_type == FormFieldType::Password {
                "*".repeat(field.value.len())
            } else {
                field.value.clone()
            };
            
            let input_width = content_region.width - 2;
            let mut input_text = format!(" {}", display_value);
            
            // Pad or truncate to fit
            if input_text.len() < input_width as usize {
                input_text.push_str(&" ".repeat(input_width as usize - input_text.len()));
            } else {
                input_text.truncate(input_width as usize);
            }
            
            terminal.write_text(&input_text)?;
            current_row += 1;
            
            // Render error if present
            if let Some(ref error) = field.error {
                if current_row < content_region.height {
                    terminal.move_cursor(content_region.y + current_row, content_region.x)?;
                    terminal.set_color(theme.colors.error, theme.colors.background)?;
                    terminal.write_text(&format!("  ❌ {}", error))?;
                    current_row += 1;
                }
            }
            
            current_row += 1; // Extra spacing between fields
        }
        
        terminal.reset_colors()?;
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> QmsResult<bool> {
        if !self.focused || self.fields.is_empty() {
            return Ok(false);
        }

        match event {
            Event::Key(key_event) => {
                match key_event.key.as_str() {
                    "Up" => {
                        if self.current_field > 0 {
                            self.current_field -= 1;
                        }
                        Ok(true)
                    }
                    "Down" | "Tab" => {
                        if self.current_field < self.fields.len() - 1 {
                            self.current_field += 1;
                        }
                        Ok(true)
                    }
                    "Backspace" => {
                        if let Some(field) = self.fields.get_mut(self.current_field) {
                            field.value.pop();
                            field.error = None; // Clear error on edit
                        }
                        Ok(true)
                    }
                    key if key.len() == 1 && key_event.is_printable() => {
                        if let Some(field) = self.fields.get_mut(self.current_field) {
                            field.value.push(key.chars().next().unwrap());
                            field.error = None; // Clear error on edit
                        }
                        Ok(true)
                    }
                    _ => Ok(false),
                }
            }
            _ => Ok(false),
        }
    }

    fn min_size(&self) -> (u16, u16) {
        let width = 40; // Reasonable default width for forms
        let height = (self.fields.len() * 3 + 2) as u16; // 3 rows per field + border
        (width, height)
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn is_focused(&self) -> bool {
        self.focused
    }
}

/// Table widget for displaying tabular data
pub struct TableWidget {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    selected_row: usize,
    focused: bool,
    title: String,
}

impl TableWidget {
    /// Create new table widget
    pub fn new(title: String, headers: Vec<String>) -> Self {
        Self {
            headers,
            rows: Vec::new(),
            selected_row: 0,
            focused: false,
            title,
        }
    }

    /// Add row to table
    pub fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    /// Get selected row
    pub fn selected_row(&self) -> Option<&Vec<String>> {
        self.rows.get(self.selected_row)
    }
}

impl Widget for TableWidget {
    fn render(&self, terminal: &Terminal, region: &Region, theme: &Theme) -> QmsResult<()> {
        // Implementation would render table with headers and rows
        // This is a simplified placeholder
        WidgetRenderer::draw_border(terminal, region, theme, Some(&self.title), self.focused)?;
        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> QmsResult<bool> {
        // Implementation would handle table navigation
        Ok(false)
    }

    fn min_size(&self) -> (u16, u16) {
        (50, 10) // Reasonable default for tables
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn is_focused(&self) -> bool {
        self.focused
    }
}

/// Status widget for displaying status information
pub struct StatusWidget {
    message: String,
    status_type: StatusType,
}

/// Status types
#[derive(Debug, Clone, PartialEq)]
pub enum StatusType {
    Success,
    Warning,
    Error,
    Info,
}

impl StatusWidget {
    /// Create new status widget
    pub fn new(message: String, status_type: StatusType) -> Self {
        Self {
            message,
            status_type,
        }
    }
}

impl Widget for StatusWidget {
    fn render(&self, terminal: &Terminal, region: &Region, theme: &Theme) -> QmsResult<()> {
        let style = match self.status_type {
            StatusType::Success => theme.styles.status_success,
            StatusType::Warning => theme.styles.status_warning,
            StatusType::Error => theme.styles.status_error,
            StatusType::Info => theme.styles.status_info,
        };

        terminal.move_cursor(region.y, region.x)?;
        terminal.set_color(style.0, style.1)?;
        
        let icon = match self.status_type {
            StatusType::Success => "✅",
            StatusType::Warning => "⚠️",
            StatusType::Error => "❌",
            StatusType::Info => "ℹ️",
        };
        
        let status_text = format!("{} {}", icon, self.message);
        terminal.write_text(&status_text)?;
        terminal.reset_colors()?;
        
        Ok(())
    }

    fn handle_event(&mut self, _event: &Event) -> QmsResult<bool> {
        Ok(false) // Status widgets don't handle events
    }

    fn min_size(&self) -> (u16, u16) {
        (self.message.len() as u16 + 3, 1) // +3 for icon and spaces
    }
}
