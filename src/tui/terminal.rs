//! Cross-platform Terminal Control for QMS TUI
//! 
//! This module provides terminal control using standard library components
//! and ANSI escape codes for cross-platform compatibility. It handles
//! terminal initialization, cursor control, colors, and input/output.

use crate::prelude::*;
use crate::tui::theme::{Color, Style};
use std::io::{self, Write, Read, Stdout, Stdin};
use std::sync::{Arc, Mutex};

/// Terminal capabilities detection
#[derive(Debug, Clone)]
pub struct TerminalCapabilities {
    pub supports_color: bool,
    pub supports_mouse: bool,
    pub supports_alternate_screen: bool,
    pub width: u16,
    pub height: u16,
}

impl TerminalCapabilities {
    /// Detect terminal capabilities
    pub fn detect() -> Self {
        let (width, height) = Self::get_terminal_size();
        
        Self {
            supports_color: Self::supports_ansi_colors(),
            supports_mouse: Self::supports_mouse_input(),
            supports_alternate_screen: Self::supports_alternate_screen_buffer(),
            width,
            height,
        }
    }

    /// Get terminal size using standard methods
    fn get_terminal_size() -> (u16, u16) {
        // Try to get size from environment variables first
        if let (Ok(cols), Ok(rows)) = (std::env::var("COLUMNS"), std::env::var("LINES")) {
            if let (Ok(cols), Ok(rows)) = (cols.parse::<u16>(), rows.parse::<u16>()) {
                return (cols, rows);
            }
        }

        // Default fallback size
        (80, 24)
    }

    /// Check if terminal supports ANSI colors
    fn supports_ansi_colors() -> bool {
        // Check common environment variables that indicate color support
        if let Ok(term) = std::env::var("TERM") {
            return term.contains("color") || term.contains("256") || term == "xterm" || term == "screen";
        }
        
        // Check if we're not in a pipe or redirect
        #[cfg(unix)]
        {
            unsafe {
                libc::isatty(libc::STDOUT_FILENO) != 0
            }
        }
        
        #[cfg(windows)]
        {
            // On Windows, assume color support for modern terminals
            true
        }
        
        #[cfg(not(any(unix, windows)))]
        {
            false
        }
    }

    /// Check if terminal supports mouse input
    fn supports_mouse_input() -> bool {
        // Most modern terminals support mouse input
        // We'll enable it and let the terminal ignore if not supported
        true
    }

    /// Check if terminal supports alternate screen buffer
    fn supports_alternate_screen_buffer() -> bool {
        // Most terminals support alternate screen
        true
    }
}

/// Terminal controller for TUI operations
pub struct Terminal {
    capabilities: TerminalCapabilities,
    stdout: Arc<Mutex<Stdout>>,
    stdin: Arc<Mutex<Stdin>>,
    raw_mode_enabled: bool,
    alternate_screen_enabled: bool,
    cursor_visible: bool,
}

impl Terminal {
    /// Create new terminal controller
    pub fn new() -> QmsResult<Self> {
        let capabilities = TerminalCapabilities::detect();
        
        Ok(Self {
            capabilities,
            stdout: Arc::new(Mutex::new(io::stdout())),
            stdin: Arc::new(Mutex::new(io::stdin())),
            raw_mode_enabled: false,
            alternate_screen_enabled: false,
            cursor_visible: true,
        })
    }

    /// Get terminal size
    pub fn get_size(&self) -> QmsResult<(u16, u16)> {
        Ok((self.capabilities.width, self.capabilities.height))
    }

    /// Resize terminal (called when terminal is resized)
    pub fn resize(&mut self, width: u16, height: u16) -> QmsResult<()> {
        self.capabilities.width = width;
        self.capabilities.height = height;
        Ok(())
    }

    /// Enter alternate screen buffer
    pub fn enter_alternate_screen(&mut self) -> QmsResult<()> {
        if self.capabilities.supports_alternate_screen && !self.alternate_screen_enabled {
            self.write_escape_sequence("\x1b[?1049h")?;
            self.alternate_screen_enabled = true;
        }
        Ok(())
    }

    /// Exit alternate screen buffer
    pub fn exit_alternate_screen(&mut self) -> QmsResult<()> {
        if self.alternate_screen_enabled {
            self.write_escape_sequence("\x1b[?1049l")?;
            self.alternate_screen_enabled = false;
        }
        Ok(())
    }

    /// Enable raw mode for character-by-character input
    pub fn enable_raw_mode(&mut self) -> QmsResult<()> {
        if !self.raw_mode_enabled {
            #[cfg(unix)]
            {
                self.enable_raw_mode_unix()?;
            }
            
            #[cfg(windows)]
            {
                self.enable_raw_mode_windows()?;
            }
            
            self.raw_mode_enabled = true;
        }
        Ok(())
    }

    /// Disable raw mode
    pub fn disable_raw_mode(&mut self) -> QmsResult<()> {
        if self.raw_mode_enabled {
            #[cfg(unix)]
            {
                self.disable_raw_mode_unix()?;
            }
            
            #[cfg(windows)]
            {
                self.disable_raw_mode_windows()?;
            }
            
            self.raw_mode_enabled = false;
        }
        Ok(())
    }

    /// Clear the entire screen
    pub fn clear_screen(&self) -> QmsResult<()> {
        self.write_escape_sequence("\x1b[2J")?;
        self.move_cursor(1, 1)?;
        Ok(())
    }

    /// Move cursor to specific position (1-based)
    pub fn move_cursor(&self, row: u16, col: u16) -> QmsResult<()> {
        self.write_escape_sequence(&format!("\x1b[{};{}H", row, col))?;
        Ok(())
    }

    /// Hide cursor
    pub fn hide_cursor(&mut self) -> QmsResult<()> {
        if self.cursor_visible {
            self.write_escape_sequence("\x1b[?25l")?;
            self.cursor_visible = false;
        }
        Ok(())
    }

    /// Show cursor
    pub fn show_cursor(&mut self) -> QmsResult<()> {
        if !self.cursor_visible {
            self.write_escape_sequence("\x1b[?25h")?;
            self.cursor_visible = true;
        }
        Ok(())
    }

    /// Set foreground and background colors
    pub fn set_color(&self, fg: Color, bg: Color) -> QmsResult<()> {
        if self.capabilities.supports_color {
            let fg_code = self.color_to_ansi_fg(fg);
            let bg_code = self.color_to_ansi_bg(bg);
            self.write_escape_sequence(&format!("\x1b[{};{}m", fg_code, bg_code))?;
        }
        Ok(())
    }

    /// Reset colors to default
    pub fn reset_colors(&self) -> QmsResult<()> {
        if self.capabilities.supports_color {
            self.write_escape_sequence("\x1b[0m")?;
        }
        Ok(())
    }

    /// Write text to terminal
    pub fn write_text(&self, text: &str) -> QmsResult<()> {
        let mut stdout = self.stdout.lock().unwrap();
        stdout.write_all(text.as_bytes())?;
        Ok(())
    }

    /// Flush output buffer
    pub fn flush(&self) -> QmsResult<()> {
        let mut stdout = self.stdout.lock().unwrap();
        stdout.flush()?;
        Ok(())
    }

    /// Read a single character (non-blocking)
    pub fn read_char(&self) -> QmsResult<Option<char>> {
        // This is a simplified implementation
        // In a full implementation, we'd use platform-specific non-blocking I/O
        Ok(None)
    }

    /// Write ANSI escape sequence
    fn write_escape_sequence(&self, sequence: &str) -> QmsResult<()> {
        let mut stdout = self.stdout.lock().unwrap();
        stdout.write_all(sequence.as_bytes())?;
        stdout.flush()?;
        Ok(())
    }

    /// Convert Color enum to ANSI foreground color code
    fn color_to_ansi_fg(&self, color: Color) -> u8 {
        match color {
            Color::Black => 30,
            Color::Red => 31,
            Color::Green => 32,
            Color::Yellow => 33,
            Color::Blue => 34,
            Color::Magenta => 35,
            Color::Cyan => 36,
            Color::White => 37,
            Color::BrightBlack => 90,
            Color::BrightRed => 91,
            Color::BrightGreen => 92,
            Color::BrightYellow => 93,
            Color::BrightBlue => 94,
            Color::BrightMagenta => 95,
            Color::BrightCyan => 96,
            Color::BrightWhite => 97,
        }
    }

    /// Convert Color enum to ANSI background color code
    fn color_to_ansi_bg(&self, color: Color) -> u8 {
        match color {
            Color::Black => 40,
            Color::Red => 41,
            Color::Green => 42,
            Color::Yellow => 43,
            Color::Blue => 44,
            Color::Magenta => 45,
            Color::Cyan => 46,
            Color::White => 47,
            Color::BrightBlack => 100,
            Color::BrightRed => 101,
            Color::BrightGreen => 102,
            Color::BrightYellow => 103,
            Color::BrightBlue => 104,
            Color::BrightMagenta => 105,
            Color::BrightCyan => 106,
            Color::BrightWhite => 107,
        }
    }

    /// Platform-specific raw mode implementation for Unix
    #[cfg(unix)]
    fn enable_raw_mode_unix(&self) -> QmsResult<()> {
        // This would use termios to set raw mode
        // For now, we'll use a simplified approach
        Ok(())
    }

    #[cfg(unix)]
    fn disable_raw_mode_unix(&self) -> QmsResult<()> {
        // This would restore original termios settings
        Ok(())
    }

    /// Platform-specific raw mode implementation for Windows
    #[cfg(windows)]
    fn enable_raw_mode_windows(&self) -> QmsResult<()> {
        // For Windows, we'll use a simplified approach
        // In a full implementation, this would use the Windows Console API
        // to disable line buffering and echo
        Ok(())
    }

    #[cfg(windows)]
    fn disable_raw_mode_windows(&self) -> QmsResult<()> {
        // Restore original console settings
        Ok(())
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // Ensure we clean up terminal state
        let _ = self.show_cursor();
        let _ = self.disable_raw_mode();
        let _ = self.exit_alternate_screen();
        let _ = self.reset_colors();
    }
}
