//! TUI Application Entry Point
//! 
//! This module provides the main entry point for the QMS TUI application,
//! integrating with the unified interface system and providing a complete
//! terminal user interface experience.

use crate::prelude::*;
use crate::tui::{Terminal, TerminalCapabilities};
use crate::tui::simple_app::SimpleTuiApp;
use crate::tui::authenticated_app::AuthenticatedTuiApp;
use std::path::PathBuf;

/// TUI Application Runner (Authenticated)
pub struct TuiRunner {
    project_path: Option<PathBuf>,
}

/// Simple TUI Application Runner (Demo mode)
pub struct SimpleTuiRunner {
    project_path: Option<PathBuf>,
}

impl TuiRunner {
    /// Create new TUI runner
    pub fn new(project_path: Option<PathBuf>) -> Self {
        Self { project_path }
    }

    /// Run the TUI application
    pub fn run(&self) -> QmsResult<()> {
        // Check terminal capabilities
        let capabilities = TerminalCapabilities::detect();
        
        if capabilities.width < 80 || capabilities.height < 24 {
            eprintln!("⚠️  Warning: Terminal size is {}x{}, recommended minimum is 80x24", 
                capabilities.width, capabilities.height);
            eprintln!("   Some features may not display correctly.");
        }

        // Display startup message
        self.display_startup_message(&capabilities)?;

        // Create and run TUI application based on configuration
        // For now, always use authenticated mode for security
        let mut app = AuthenticatedTuiApp::new(self.project_path.clone())?;

        // Set up signal handlers for graceful shutdown
        self.setup_signal_handlers()?;

        // Run the application
        match app.run() {
            Ok(()) => {
                println!("✅ QMS TUI application exited successfully");
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ QMS TUI application error: {}", e);
                Err(e)
            }
        }
    }

    /// Display startup message with system information
    fn display_startup_message(&self, capabilities: &TerminalCapabilities) -> QmsResult<()> {
        println!("🏥 QMS - Medical Device Quality Management System");
        println!("   Terminal User Interface (TUI)");
        println!();
        println!("📊 System Information:");
        println!("   Terminal Size: {}x{}", capabilities.width, capabilities.height);
        println!("   Color Support: {}", if capabilities.supports_color { "Yes" } else { "No" });
        println!("   Mouse Support: {}", if capabilities.supports_mouse { "Yes" } else { "No" });
        println!("   Alternate Screen: {}", if capabilities.supports_alternate_screen { "Yes" } else { "No" });
        
        if let Some(ref path) = self.project_path {
            println!("   Project Path: {}", path.display());
        } else {
            println!("   Project Path: Current directory");
        }
        
        println!();
        println!("🔧 Controls:");
        println!("   Ctrl+Q or Ctrl+C: Quit application");
        println!("   Escape: Go back/Cancel");
        println!("   Arrow Keys: Navigate");
        println!("   Enter/Space: Select");
        println!("   F1: Help");
        println!();
        println!("⚡ Starting TUI application...");
        
        // Small delay to let user read the information
        std::thread::sleep(std::time::Duration::from_millis(2000));
        
        Ok(())
    }

    /// Set up signal handlers for graceful shutdown
    fn setup_signal_handlers(&self) -> QmsResult<()> {
        // This would set up signal handlers for SIGINT, SIGTERM, etc.
        // For now, we'll rely on the application's built-in Ctrl+C handling
        Ok(())
    }
}

impl SimpleTuiRunner {
    /// Create new simple TUI runner
    pub fn new(project_path: Option<PathBuf>) -> Self {
        Self { project_path }
    }

    /// Run the simple TUI application
    pub fn run(&self) -> QmsResult<()> {
        // Create and run simple TUI application
        let mut app = SimpleTuiApp::new(self.project_path.clone())?;

        // Run the application
        match app.run() {
            Ok(()) => {
                println!("✅ QMS Simple TUI application exited successfully");
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ QMS Simple TUI application error: {}", e);
                Err(e)
            }
        }
    }
}

/// TUI Command Line Interface
pub struct TuiCli;

impl TuiCli {
    /// Parse command line arguments for TUI
    pub fn parse_args(args: &[String]) -> QmsResult<TuiConfig> {
        let mut config = TuiConfig::default();
        let mut i = 0;

        while i < args.len() {
            match args[i].as_str() {
                "--project" | "-p" => {
                    if i + 1 < args.len() {
                        config.project_path = Some(PathBuf::from(&args[i + 1]));
                        i += 2;
                    } else {
                        return Err(QmsError::validation_error("Project path value missing"));
                    }
                }
                "--theme" | "-t" => {
                    if i + 1 < args.len() {
                        config.theme = args[i + 1].clone();
                        i += 2;
                    } else {
                        return Err(QmsError::validation_error("Theme value missing"));
                    }
                }
                "--no-color" => {
                    config.force_no_color = true;
                    i += 1;
                }
                "--high-contrast" => {
                    config.high_contrast = true;
                    i += 1;
                }
                "--simple" => {
                    config.use_simple_mode = true;
                    i += 1;
                }
                "--authenticated" => {
                    config.use_simple_mode = false;
                    i += 1;
                }
                "--help" | "-h" => {
                    Self::print_help();
                    std::process::exit(0);
                }
                "--version" | "-v" => {
                    println!("qms-tui v1.0.0 - QMS Terminal User Interface");
                    std::process::exit(0);
                }
                arg if arg.starts_with('-') => {
                    return Err(QmsError::validation_error(&format!("Unknown option: {}", arg)));
                }
                _ => {
                    // Assume it's a project path if no project path is set
                    if config.project_path.is_none() {
                        config.project_path = Some(PathBuf::from(&args[i]));
                    }
                    i += 1;
                }
            }
        }

        Ok(config)
    }

    /// Print help information
    fn print_help() {
        println!("QMS Terminal User Interface (TUI)");
        println!();
        println!("USAGE:");
        println!("    qms tui [OPTIONS] [PROJECT_PATH]");
        println!();
        println!("OPTIONS:");
        println!("    -p, --project <PATH>     Set project path");
        println!("    -t, --theme <THEME>      Set theme (default, high-contrast, monochrome)");
        println!("        --no-color           Disable color output");
        println!("        --high-contrast      Use high contrast theme");
        println!("        --simple             Use simple demo mode (no authentication)");
        println!("        --authenticated      Use authenticated mode (default)");
        println!("    -h, --help               Show this help message");
        println!("    -v, --version            Show version information");
        println!();
        println!("EXAMPLES:");
        println!("    qms tui                           # Start authenticated TUI in current directory");
        println!("    qms tui /path/to/project         # Start TUI with specific project");
        println!("    qms tui --theme high-contrast    # Start with high contrast theme");
        println!("    qms tui --simple                 # Start simple demo mode (no auth)");
        println!("    qms tui --authenticated          # Start with full authentication (default)");
        println!();
        println!("FEATURES:");
        println!("    ✅ Cross-platform terminal support");
        println!("    ✅ Keyboard and mouse navigation");
        println!("    ✅ Responsive layout system");
        println!("    ✅ Accessibility support");
        println!("    ✅ Medical device compliance");
        println!("    ✅ Unified authentication system");
        println!("    ✅ Audit trail integration");
        println!("    ✅ Role-based access control");
    }

    /// Run TUI with configuration
    pub fn run_with_config(config: TuiConfig) -> QmsResult<()> {
        if config.use_simple_mode {
            // Run simple TUI for demonstration
            let runner = SimpleTuiRunner::new(config.project_path);
            runner.run()
        } else {
            // Run authenticated TUI (default)
            let runner = TuiRunner::new(config.project_path);
            runner.run()
        }
    }
}

/// TUI Configuration
#[derive(Debug, Clone)]
pub struct TuiConfig {
    pub project_path: Option<PathBuf>,
    pub theme: String,
    pub force_no_color: bool,
    pub high_contrast: bool,
    pub use_simple_mode: bool,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            project_path: None,
            theme: "default".to_string(),
            force_no_color: false,
            high_contrast: false,
            use_simple_mode: false,
        }
    }
}

/// TUI Integration with Unified Interface System
pub struct TuiIntegration;

impl TuiIntegration {
    /// Create TUI interface manager using unified system (placeholder)
    pub fn create_interface_manager(_project_path: Option<PathBuf>) -> QmsResult<()> {
        // In full implementation, this would create a CLI interface manager
        // This ensures consistent behavior across all interfaces
        Ok(())
    }

    /// Validate TUI environment
    pub fn validate_environment() -> QmsResult<()> {
        // Check if we're running in a terminal
        if !Self::is_terminal() {
            return Err(QmsError::InvalidOperation(
                "TUI requires a terminal environment".to_string()
            ));
        }

        // Check terminal capabilities
        let capabilities = TerminalCapabilities::detect();
        
        if capabilities.width < 40 || capabilities.height < 10 {
            return Err(QmsError::InvalidOperation(
                format!("Terminal too small: {}x{}, minimum 40x10 required", 
                    capabilities.width, capabilities.height)
            ));
        }

        Ok(())
    }

    /// Check if running in a terminal
    fn is_terminal() -> bool {
        #[cfg(unix)]
        {
            unsafe {
                libc::isatty(libc::STDIN_FILENO) != 0 && libc::isatty(libc::STDOUT_FILENO) != 0
            }
        }
        
        #[cfg(windows)]
        {
            // On Windows, assume we're in a terminal if we can get console info
            // This is a simplified check
            true
        }
        
        #[cfg(not(any(unix, windows)))]
        {
            // For other platforms, assume terminal support
            true
        }
    }

    /// Get recommended theme based on terminal capabilities
    pub fn get_recommended_theme() -> String {
        let capabilities = TerminalCapabilities::detect();
        
        if !capabilities.supports_color {
            "monochrome".to_string()
        } else if std::env::var("TERM").unwrap_or_default().contains("256") {
            "default".to_string()
        } else {
            "high-contrast".to_string()
        }
    }
}

/// TUI Error Recovery
pub struct TuiErrorRecovery;

impl TuiErrorRecovery {
    /// Handle TUI application errors gracefully
    pub fn handle_error(error: &QmsError) -> QmsResult<()> {
        // Ensure terminal is in a clean state
        Self::cleanup_terminal()?;
        
        // Display error message
        eprintln!("❌ TUI Application Error: {}", error);
        
        // Provide recovery suggestions
        match error {
            QmsError::InvalidOperation(msg) if msg.contains("terminal") => {
                eprintln!("💡 Suggestion: Try running in a different terminal or check terminal settings");
            }
            QmsError::Authentication(_) => {
                eprintln!("💡 Suggestion: Check your credentials and try again");
            }
            QmsError::Io(_) => {
                eprintln!("💡 Suggestion: Check file permissions and disk space");
            }
            QmsError::Permission(_) => {
                eprintln!("💡 Suggestion: Check file permissions and access rights");
            }
            _ => {
                eprintln!("💡 Suggestion: Try restarting the application or check system resources");
            }
        }
        
        Ok(())
    }

    /// Clean up terminal state
    fn cleanup_terminal() -> QmsResult<()> {
        // Reset terminal to normal state
        print!("\x1b[?1049l"); // Exit alternate screen
        print!("\x1b[?25h");   // Show cursor
        print!("\x1b[0m");     // Reset colors
        
        use std::io::{self, Write};
        io::stdout().flush()?;
        
        Ok(())
    }
}

/// Main TUI entry point function
pub fn run_tui(args: &[String]) -> QmsResult<()> {
    // Parse command line arguments
    let config = TuiCli::parse_args(args)?;

    // Validate environment
    TuiIntegration::validate_environment()?;

    // Display startup message
    let capabilities = TerminalCapabilities::detect();

    if capabilities.width < 80 || capabilities.height < 24 {
        println!("⚠️  Warning: Terminal size is {}x{}, recommended minimum is 80x24",
            capabilities.width, capabilities.height);
        println!("   Some features may not display correctly.");
        println!();
    }

    println!("🏥 QMS - Medical Device Quality Management System");
    println!("   Terminal User Interface (TUI)");
    println!();
    println!("📊 System Information:");
    println!("   Terminal Size: {}x{}", capabilities.width, capabilities.height);
    println!("   Color Support: {}", if capabilities.supports_color { "Yes" } else { "No" });
    println!("   Mouse Support: {}", if capabilities.supports_mouse { "Yes" } else { "No" });
    println!("   Alternate Screen: {}", if capabilities.supports_alternate_screen { "Yes" } else { "No" });
    println!();
    println!("🔧 Controls:");
    println!("   Ctrl+Q or Ctrl+C: Quit application");
    println!("   Escape: Go back/Cancel");
    println!("   Arrow Keys: Navigate");
    println!("   Enter/Space: Select");
    println!("   F1: Help");
    println!();
    println!("⚡ Starting TUI application...");

    // Small delay to let user read the information
    std::thread::sleep(std::time::Duration::from_millis(2000));

    // Create and run TUI application
    let runner = TuiRunner::new(config.project_path);
    runner.run()
}
