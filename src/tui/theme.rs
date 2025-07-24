//! Theme and Styling System for QMS TUI
//! 
//! This module provides consistent theming and styling for the TUI interface,
//! ensuring a professional appearance that aligns with medical device standards.

use crate::prelude::*;

/// Color enumeration for terminal colors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

/// Text styling options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Normal,
    Bold,
    Dim,
    Italic,
    Underline,
    Blink,
    Reverse,
    Strikethrough,
}

/// Color scheme for different UI elements
#[derive(Debug, Clone)]
pub struct ColorScheme {
    // Primary colors
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    
    // Status colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    
    // UI element colors
    pub background: Color,
    pub foreground: Color,
    pub border: Color,
    pub highlight: Color,
    pub selection: Color,
    
    // Text colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub text_inverse: Color,
}

impl ColorScheme {
    /// Create default QMS color scheme (professional medical device theme)
    pub fn default() -> Self {
        Self {
            // Primary colors - professional blue theme
            primary: Color::Blue,
            secondary: Color::Cyan,
            accent: Color::BrightBlue,
            
            // Status colors - standard conventions
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Cyan,
            
            // UI element colors
            background: Color::Black,
            foreground: Color::White,
            border: Color::BrightBlack,
            highlight: Color::BrightWhite,
            selection: Color::BrightBlue,
            
            // Text colors
            text_primary: Color::White,
            text_secondary: Color::BrightBlack,
            text_muted: Color::BrightBlack,
            text_inverse: Color::Black,
        }
    }

    /// Create high contrast color scheme for accessibility
    pub fn high_contrast() -> Self {
        Self {
            primary: Color::BrightWhite,
            secondary: Color::BrightYellow,
            accent: Color::BrightCyan,
            
            success: Color::BrightGreen,
            warning: Color::BrightYellow,
            error: Color::BrightRed,
            info: Color::BrightCyan,
            
            background: Color::Black,
            foreground: Color::BrightWhite,
            border: Color::BrightWhite,
            highlight: Color::BrightYellow,
            selection: Color::BrightWhite,
            
            text_primary: Color::BrightWhite,
            text_secondary: Color::White,
            text_muted: Color::BrightBlack,
            text_inverse: Color::Black,
        }
    }

    /// Create monochrome color scheme for terminals without color support
    pub fn monochrome() -> Self {
        Self {
            primary: Color::White,
            secondary: Color::BrightBlack,
            accent: Color::BrightWhite,
            
            success: Color::White,
            warning: Color::BrightBlack,
            error: Color::BrightWhite,
            info: Color::White,
            
            background: Color::Black,
            foreground: Color::White,
            border: Color::BrightBlack,
            highlight: Color::BrightWhite,
            selection: Color::BrightWhite,
            
            text_primary: Color::White,
            text_secondary: Color::BrightBlack,
            text_muted: Color::BrightBlack,
            text_inverse: Color::Black,
        }
    }
}

/// Complete theme configuration
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub colors: ColorScheme,
    pub styles: ThemeStyles,
}

/// Style definitions for different UI components
#[derive(Debug, Clone)]
pub struct ThemeStyles {
    // Header styles
    pub header_title: (Color, Color, Style),
    pub header_subtitle: (Color, Color, Style),
    
    // Menu styles
    pub menu_item: (Color, Color, Style),
    pub menu_item_selected: (Color, Color, Style),
    pub menu_item_disabled: (Color, Color, Style),
    
    // Form styles
    pub form_label: (Color, Color, Style),
    pub form_input: (Color, Color, Style),
    pub form_input_focused: (Color, Color, Style),
    pub form_input_error: (Color, Color, Style),
    
    // Table styles
    pub table_header: (Color, Color, Style),
    pub table_row: (Color, Color, Style),
    pub table_row_selected: (Color, Color, Style),
    pub table_row_alternate: (Color, Color, Style),
    
    // Status styles
    pub status_success: (Color, Color, Style),
    pub status_warning: (Color, Color, Style),
    pub status_error: (Color, Color, Style),
    pub status_info: (Color, Color, Style),
    
    // Border styles
    pub border_normal: (Color, Color, Style),
    pub border_focused: (Color, Color, Style),
    pub border_error: (Color, Color, Style),
    
    // Button styles
    pub button_normal: (Color, Color, Style),
    pub button_focused: (Color, Color, Style),
    pub button_pressed: (Color, Color, Style),
    pub button_disabled: (Color, Color, Style),
}

impl Theme {
    /// Create default QMS theme
    pub fn default() -> Self {
        let colors = ColorScheme::default();
        
        Self {
            name: "QMS Default".to_string(),
            colors: colors.clone(),
            styles: ThemeStyles {
                // Header styles
                header_title: (colors.text_primary, colors.primary, Style::Bold),
                header_subtitle: (colors.text_secondary, colors.background, Style::Normal),
                
                // Menu styles
                menu_item: (colors.text_primary, colors.background, Style::Normal),
                menu_item_selected: (colors.text_inverse, colors.selection, Style::Bold),
                menu_item_disabled: (colors.text_muted, colors.background, Style::Dim),
                
                // Form styles
                form_label: (colors.text_primary, colors.background, Style::Bold),
                form_input: (colors.text_primary, colors.background, Style::Normal),
                form_input_focused: (colors.text_primary, colors.background, Style::Underline),
                form_input_error: (colors.error, colors.background, Style::Normal),
                
                // Table styles
                table_header: (colors.text_inverse, colors.primary, Style::Bold),
                table_row: (colors.text_primary, colors.background, Style::Normal),
                table_row_selected: (colors.text_inverse, colors.selection, Style::Normal),
                table_row_alternate: (colors.text_primary, colors.border, Style::Normal),
                
                // Status styles
                status_success: (colors.success, colors.background, Style::Bold),
                status_warning: (colors.warning, colors.background, Style::Bold),
                status_error: (colors.error, colors.background, Style::Bold),
                status_info: (colors.info, colors.background, Style::Bold),
                
                // Border styles
                border_normal: (colors.border, colors.background, Style::Normal),
                border_focused: (colors.accent, colors.background, Style::Normal),
                border_error: (colors.error, colors.background, Style::Normal),
                
                // Button styles
                button_normal: (colors.text_primary, colors.secondary, Style::Normal),
                button_focused: (colors.text_inverse, colors.accent, Style::Bold),
                button_pressed: (colors.text_inverse, colors.primary, Style::Bold),
                button_disabled: (colors.text_muted, colors.border, Style::Dim),
            },
        }
    }

    /// Create high contrast theme for accessibility
    pub fn high_contrast() -> Self {
        let colors = ColorScheme::high_contrast();
        
        Self {
            name: "High Contrast".to_string(),
            colors: colors.clone(),
            styles: ThemeStyles {
                header_title: (colors.text_primary, colors.primary, Style::Bold),
                header_subtitle: (colors.text_secondary, colors.background, Style::Normal),
                
                menu_item: (colors.text_primary, colors.background, Style::Normal),
                menu_item_selected: (colors.text_inverse, colors.selection, Style::Bold),
                menu_item_disabled: (colors.text_muted, colors.background, Style::Dim),
                
                form_label: (colors.text_primary, colors.background, Style::Bold),
                form_input: (colors.text_primary, colors.background, Style::Normal),
                form_input_focused: (colors.text_primary, colors.background, Style::Underline),
                form_input_error: (colors.error, colors.background, Style::Bold),
                
                table_header: (colors.text_inverse, colors.primary, Style::Bold),
                table_row: (colors.text_primary, colors.background, Style::Normal),
                table_row_selected: (colors.text_inverse, colors.selection, Style::Bold),
                table_row_alternate: (colors.text_primary, colors.border, Style::Normal),
                
                status_success: (colors.success, colors.background, Style::Bold),
                status_warning: (colors.warning, colors.background, Style::Bold),
                status_error: (colors.error, colors.background, Style::Bold),
                status_info: (colors.info, colors.background, Style::Bold),
                
                border_normal: (colors.border, colors.background, Style::Normal),
                border_focused: (colors.accent, colors.background, Style::Bold),
                border_error: (colors.error, colors.background, Style::Bold),
                
                button_normal: (colors.text_primary, colors.secondary, Style::Bold),
                button_focused: (colors.text_inverse, colors.accent, Style::Bold),
                button_pressed: (colors.text_inverse, colors.primary, Style::Bold),
                button_disabled: (colors.text_muted, colors.border, Style::Dim),
            },
        }
    }

    /// Create monochrome theme for terminals without color support
    pub fn monochrome() -> Self {
        let colors = ColorScheme::monochrome();
        
        Self {
            name: "Monochrome".to_string(),
            colors: colors.clone(),
            styles: ThemeStyles {
                header_title: (colors.text_primary, colors.background, Style::Bold),
                header_subtitle: (colors.text_secondary, colors.background, Style::Normal),
                
                menu_item: (colors.text_primary, colors.background, Style::Normal),
                menu_item_selected: (colors.text_inverse, colors.selection, Style::Reverse),
                menu_item_disabled: (colors.text_muted, colors.background, Style::Dim),
                
                form_label: (colors.text_primary, colors.background, Style::Bold),
                form_input: (colors.text_primary, colors.background, Style::Normal),
                form_input_focused: (colors.text_primary, colors.background, Style::Underline),
                form_input_error: (colors.text_primary, colors.background, Style::Reverse),
                
                table_header: (colors.text_primary, colors.background, Style::Bold),
                table_row: (colors.text_primary, colors.background, Style::Normal),
                table_row_selected: (colors.text_primary, colors.background, Style::Reverse),
                table_row_alternate: (colors.text_secondary, colors.background, Style::Normal),
                
                status_success: (colors.text_primary, colors.background, Style::Bold),
                status_warning: (colors.text_primary, colors.background, Style::Reverse),
                status_error: (colors.text_primary, colors.background, Style::Reverse),
                status_info: (colors.text_primary, colors.background, Style::Bold),
                
                border_normal: (colors.text_secondary, colors.background, Style::Normal),
                border_focused: (colors.text_primary, colors.background, Style::Bold),
                border_error: (colors.text_primary, colors.background, Style::Reverse),
                
                button_normal: (colors.text_primary, colors.background, Style::Normal),
                button_focused: (colors.text_primary, colors.background, Style::Reverse),
                button_pressed: (colors.text_primary, colors.background, Style::Bold),
                button_disabled: (colors.text_muted, colors.background, Style::Dim),
            },
        }
    }

    /// Get theme based on terminal capabilities
    pub fn for_capabilities(supports_color: bool, high_contrast: bool) -> Self {
        match (supports_color, high_contrast) {
            (true, true) => Self::high_contrast(),
            (true, false) => Self::default(),
            (false, _) => Self::monochrome(),
        }
    }
}

/// Theme manager for handling theme selection and application
pub struct ThemeManager {
    current_theme: Theme,
    available_themes: Vec<Theme>,
}

impl ThemeManager {
    /// Create new theme manager with default theme
    pub fn new() -> Self {
        let default_theme = Theme::default();
        
        Self {
            current_theme: default_theme.clone(),
            available_themes: vec![
                default_theme,
                Theme::high_contrast(),
                Theme::monochrome(),
            ],
        }
    }

    /// Get current theme
    pub fn current_theme(&self) -> &Theme {
        &self.current_theme
    }

    /// Set theme by name
    pub fn set_theme(&mut self, name: &str) -> QmsResult<()> {
        if let Some(theme) = self.available_themes.iter().find(|t| t.name == name) {
            self.current_theme = theme.clone();
            Ok(())
        } else {
            Err(QmsError::validation_error(&format!("Theme '{}' not found", name)))
        }
    }

    /// Get list of available theme names
    pub fn available_themes(&self) -> Vec<&str> {
        self.available_themes.iter().map(|t| t.name.as_str()).collect()
    }

    /// Auto-select theme based on terminal capabilities
    pub fn auto_select_theme(&mut self, supports_color: bool, high_contrast: bool) {
        self.current_theme = Theme::for_capabilities(supports_color, high_contrast);
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}
