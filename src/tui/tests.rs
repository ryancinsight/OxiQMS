//! Tests for QMS TUI Framework
//! 
//! This module provides comprehensive tests for the TUI framework,
//! demonstrating functionality and ensuring reliability.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::tui::{
        terminal::{Terminal, TerminalCapabilities},
        theme::{Theme, ColorScheme, Color},
        layout::{Region, Layout, Direction, Constraint, QmsLayouts},
        widgets::{Widget, MenuWidget, FormWidget, MenuItem, FormField, FormFieldType},
        events::{Event, KeyEvent, EventHandler},
        screens::{Screen, ScreenManager, LoginScreen, MainMenuScreen, ScreenAction},
        app::{TuiConfig, TuiIntegration},
    };
    use std::path::PathBuf;

    #[test]
    fn test_terminal_capabilities_detection() {
        let capabilities = TerminalCapabilities::detect();
        
        // Basic sanity checks
        assert!(capabilities.width > 0);
        assert!(capabilities.height > 0);
        assert!(capabilities.width >= 40); // Minimum reasonable width
        assert!(capabilities.height >= 10); // Minimum reasonable height
        
        println!("Terminal capabilities: {:?}", capabilities);
    }

    #[test]
    fn test_theme_system() {
        // Test default theme
        let default_theme = Theme::default();
        assert_eq!(default_theme.name, "QMS Default");
        assert_eq!(default_theme.colors.primary, Color::Blue);
        assert_eq!(default_theme.colors.success, Color::Green);
        assert_eq!(default_theme.colors.error, Color::Red);

        // Test high contrast theme
        let high_contrast_theme = Theme::high_contrast();
        assert_eq!(high_contrast_theme.name, "High Contrast");
        assert_eq!(high_contrast_theme.colors.primary, Color::BrightWhite);

        // Test monochrome theme
        let monochrome_theme = Theme::monochrome();
        assert_eq!(monochrome_theme.name, "Monochrome");
        assert_eq!(monochrome_theme.colors.primary, Color::White);

        // Test theme selection based on capabilities
        let color_theme = Theme::for_capabilities(true, false);
        assert_eq!(color_theme.name, "QMS Default");

        let no_color_theme = Theme::for_capabilities(false, false);
        assert_eq!(no_color_theme.name, "Monochrome");

        let high_contrast_theme = Theme::for_capabilities(true, true);
        assert_eq!(high_contrast_theme.name, "High Contrast");
    }

    #[test]
    fn test_layout_system() {
        // Test region creation and manipulation
        let region = Region::new(10, 20, 100, 50);
        assert_eq!(region.x, 10);
        assert_eq!(region.y, 20);
        assert_eq!(region.width, 100);
        assert_eq!(region.height, 50);
        assert_eq!(region.area(), 5000);

        // Test region contains
        assert!(region.contains(50, 30));
        assert!(!region.contains(5, 15));

        // Test region splitting
        let (top, bottom) = region.split_horizontal(25);
        assert_eq!(top.height, 25);
        assert_eq!(bottom.height, 25);
        assert_eq!(top.y, 20);
        assert_eq!(bottom.y, 45);

        let (left, right) = region.split_vertical(40);
        assert_eq!(left.width, 40);
        assert_eq!(right.width, 60);
        assert_eq!(left.x, 10);
        assert_eq!(right.x, 50);

        // Test margin
        let margin_region = region.margin(5);
        assert_eq!(margin_region.x, 15);
        assert_eq!(margin_region.y, 25);
        assert_eq!(margin_region.width, 90);
        assert_eq!(margin_region.height, 40);
    }

    #[test]
    fn test_layout_constraints() {
        // Test constraint calculations
        assert_eq!(Constraint::Length(10).calculate_size(100, 5), 10);
        assert_eq!(Constraint::Percentage(50).calculate_size(100, 5), 50);
        assert_eq!(Constraint::Min(20).calculate_size(100, 5), 20);
        assert_eq!(Constraint::Max(30).calculate_size(100, 5), 30);
        assert_eq!(Constraint::Fill.calculate_size(100, 5), 100);

        // Test layout splitting
        let layout = Layout::new(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(10),
                Constraint::Fill,
                Constraint::Length(5),
            ]);

        let region = Region::new(0, 0, 100, 50);
        let regions = layout.split(&region);

        assert_eq!(regions.len(), 3);
        assert_eq!(regions[0].height, 10);
        assert_eq!(regions[2].height, 5);
        // Middle region gets remaining space: 50 - 10 - 5 = 35
        assert_eq!(regions[1].height, 35);
    }

    #[test]
    fn test_predefined_layouts() {
        let region = Region::new(0, 0, 100, 50);

        // Test main menu layout
        let main_menu_layout = QmsLayouts::main_menu();
        let regions = main_menu_layout.split(&region);
        assert_eq!(regions.len(), 3); // Header, Menu, Status

        // Test form layout
        let form_layout = QmsLayouts::form();
        let regions = form_layout.split(&region);
        assert_eq!(regions.len(), 4); // Header, Form, Buttons, Status

        // Test table layout
        let table_layout = QmsLayouts::table();
        let regions = table_layout.split(&region);
        assert_eq!(regions.len(), 3); // Header, Table, Status

        // Test split panel layout
        let split_layout = QmsLayouts::split_panel();
        let regions = split_layout.split(&region);
        assert_eq!(regions.len(), 2); // Left panel, Right content

        // Test dashboard layout
        let dashboard_layout = QmsLayouts::dashboard();
        let regions = dashboard_layout.split(&region);
        assert_eq!(regions.len(), 3); // Header, Main, Summary
    }

    #[test]
    fn test_menu_widget() {
        let mut menu = MenuWidget::new("Test Menu".to_string());
        
        // Add menu items
        menu.add_item(MenuItem::new("item1".to_string(), "Item 1".to_string())
            .with_shortcut('1'));
        menu.add_item(MenuItem::new("item2".to_string(), "Item 2".to_string())
            .with_shortcut('2'));
        menu.add_item(MenuItem::new("item3".to_string(), "Item 3".to_string())
            .with_shortcut('3')
            .with_enabled(false));

        // Test initial state
        assert_eq!(menu.selected_item().unwrap().id, "item1");

        // Test navigation
        menu.select_next();
        assert_eq!(menu.selected_item().unwrap().id, "item2");

        menu.select_next();
        assert_eq!(menu.selected_item().unwrap().id, "item3");

        menu.select_previous();
        assert_eq!(menu.selected_item().unwrap().id, "item2");

        // Test shortcut selection
        assert!(menu.select_by_shortcut('1'));
        assert_eq!(menu.selected_item().unwrap().id, "item1");

        // Test disabled item shortcut (should fail)
        assert!(!menu.select_by_shortcut('3'));

        // Test widget properties
        assert!(menu.can_focus());
        menu.set_focus(true);
        assert!(menu.is_focused());

        let (min_width, min_height) = menu.min_size();
        assert!(min_width > 0);
        assert!(min_height > 0);
    }

    #[test]
    fn test_form_widget() {
        let mut form = FormWidget::new("Test Form".to_string());
        
        // Add form fields
        form.add_field(FormField {
            id: "username".to_string(),
            label: "Username".to_string(),
            value: String::new(),
            field_type: FormFieldType::Text,
            required: true,
            error: None,
        });
        
        form.add_field(FormField {
            id: "email".to_string(),
            label: "Email".to_string(),
            value: String::new(),
            field_type: FormFieldType::Email,
            required: false,
            error: None,
        });

        // Test validation with empty required field
        assert!(!form.validate());

        // Test getting values
        let values = form.get_values();
        assert_eq!(values.get("username").unwrap(), "");
        assert_eq!(values.get("email").unwrap(), "");

        // Test widget properties
        assert!(form.can_focus());
        form.set_focus(true);
        assert!(form.is_focused());

        let (min_width, min_height) = form.min_size();
        assert!(min_width > 0);
        assert!(min_height > 0);
    }

    #[test]
    fn test_event_system() {
        let mut event_handler = EventHandler::new();
        
        // Test key event creation
        let key_event = KeyEvent::new("Enter".to_string());
        assert_eq!(key_event.key, "Enter");
        assert!(!key_event.ctrl);
        assert!(!key_event.alt);
        assert!(!key_event.shift);

        let key_event_with_modifiers = KeyEvent::with_modifiers("c".to_string(), true, false, false);
        assert_eq!(key_event_with_modifiers.key, "c");
        assert!(key_event_with_modifiers.ctrl);

        // Test key event properties
        let printable_key = KeyEvent::new("a".to_string());
        assert!(printable_key.is_printable());

        let navigation_key = KeyEvent::new("Up".to_string());
        assert!(navigation_key.is_navigation());

        let action_key = KeyEvent::new("Enter".to_string());
        assert!(action_key.is_action());

        // Test event polling (will return None in test environment)
        let event = event_handler.poll_event().unwrap();
        assert!(event.is_none());
    }

    #[test]
    fn test_screen_system() {
        let mut screen_manager = ScreenManager::new();
        
        // Test initial state
        assert!(screen_manager.current_screen_name().is_none());

        // Test navigation
        screen_manager.navigate_to("login").unwrap();
        assert_eq!(screen_manager.current_screen_name(), Some("login"));

        screen_manager.navigate_to("main_menu").unwrap();
        assert_eq!(screen_manager.current_screen_name(), Some("main_menu"));

        // Test navigation back
        screen_manager.navigate_back().unwrap();
        assert_eq!(screen_manager.current_screen_name(), Some("login"));

        // Test invalid screen navigation
        let result = screen_manager.navigate_to("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_login_screen() {
        let mut login_screen = LoginScreen::new();
        
        // Test screen properties
        assert_eq!(login_screen.name(), "login");
        assert!(!login_screen.requires_auth());

        // Test event handling
        let key_event = KeyEvent::new("Escape".to_string());
        let event = Event::Key(key_event);
        let action = login_screen.handle_event(&event).unwrap();
        assert_eq!(action, ScreenAction::Quit);
    }

    #[test]
    fn test_main_menu_screen() {
        let mut main_menu_screen = MainMenuScreen::new();
        
        // Test screen properties
        assert_eq!(main_menu_screen.name(), "main_menu");
        assert!(main_menu_screen.requires_auth());

        // Test navigation event
        let key_event = KeyEvent::new("Escape".to_string());
        let event = Event::Key(key_event);
        let action = main_menu_screen.handle_event(&event).unwrap();
        assert_eq!(action, ScreenAction::NavigateTo("login".to_string()));
    }

    #[test]
    fn test_tui_config() {
        // Test default config
        let default_config = TuiConfig::default();
        assert!(default_config.project_path.is_none());
        assert_eq!(default_config.theme, "default");
        assert!(!default_config.force_no_color);
        assert!(!default_config.high_contrast);

        // Test config with values
        let config = TuiConfig {
            project_path: Some(PathBuf::from("/test/path")),
            theme: "high-contrast".to_string(),
            force_no_color: true,
            high_contrast: true,
            use_simple_mode: false,
        };
        
        assert_eq!(config.project_path, Some(PathBuf::from("/test/path")));
        assert_eq!(config.theme, "high-contrast");
        assert!(config.force_no_color);
        assert!(config.high_contrast);
    }

    #[test]
    fn test_tui_integration() {
        // Test environment validation
        let validation_result = TuiIntegration::validate_environment();
        // This might fail in CI environments without a terminal, so we just test it runs
        println!("Environment validation result: {:?}", validation_result);

        // Test recommended theme
        let recommended_theme = TuiIntegration::get_recommended_theme();
        assert!(!recommended_theme.is_empty());
        println!("Recommended theme: {}", recommended_theme);

        // Test interface manager creation
        let interface_manager = TuiIntegration::create_interface_manager(None);
        assert!(interface_manager.is_ok());
    }

    #[test]
    fn test_tui_framework_integration() {
        // Test that TUI framework integrates properly with unified interface system
        let test_dir = std::env::temp_dir().join("qms_tui_integration_test");
        let _ = std::fs::create_dir_all(&test_dir);

        // Create interface manager
        let interface_manager = TuiIntegration::create_interface_manager(Some(test_dir.clone()));
        assert!(interface_manager.is_ok());

        let _manager = interface_manager.unwrap();

        // Test that interface manager was created successfully
        // (Authentication testing is done in auth_tests.rs)
        
        // Test that it can access the same commands
        // (These would require authentication, so we just test the interface exists)
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_tui_accessibility_features() {
        // Test high contrast theme
        let high_contrast_theme = Theme::high_contrast();
        assert_eq!(high_contrast_theme.colors.primary, Color::BrightWhite);
        assert_eq!(high_contrast_theme.colors.background, Color::Black);

        // Test monochrome theme for terminals without color
        let monochrome_theme = Theme::monochrome();
        assert_eq!(monochrome_theme.colors.primary, Color::White);
        assert_eq!(monochrome_theme.colors.secondary, Color::BrightBlack);

        // Test that widgets support focus
        let mut menu = MenuWidget::new("Test".to_string());
        assert!(menu.can_focus());
        
        let mut form = FormWidget::new("Test".to_string());
        assert!(form.can_focus());
    }

    #[test]
    fn test_tui_medical_device_compliance() {
        // Test that TUI maintains medical device compliance features
        
        // 1. Authentication is required for most screens
        let main_menu = MainMenuScreen::new();
        assert!(main_menu.requires_auth());
        
        let login_screen = LoginScreen::new();
        assert!(!login_screen.requires_auth()); // Login screen doesn't require auth
        
        // 2. Audit trail integration (through unified interface system)
        let interface_manager = TuiIntegration::create_interface_manager(None).unwrap();
        // The interface manager uses the same audit system as CLI/Web
        
        // 3. Proper error handling and user feedback
        let capabilities = TerminalCapabilities::detect();
        assert!(capabilities.width > 0); // Basic terminal validation
        
        // 4. Consistent theming for professional appearance
        let theme = Theme::default();
        assert_eq!(theme.name, "QMS Default");
        assert_eq!(theme.colors.primary, Color::Blue); // Professional blue theme
    }
}
