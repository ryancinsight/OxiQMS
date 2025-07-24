//! Layout System for QMS TUI
//! 
//! This module provides layout management for arranging widgets in the
//! terminal interface, supporting flexible and responsive layouts.

use crate::prelude::*;

/// Represents a rectangular region in the terminal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Region {
    /// Create new region
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self { x, y, width, height }
    }

    /// Create region from terminal size
    pub fn from_terminal_size(width: u16, height: u16) -> Self {
        Self::new(0, 0, width, height)
    }

    /// Get the area (width * height) of the region
    pub fn area(&self) -> u32 {
        self.width as u32 * self.height as u32
    }

    /// Check if region contains a point
    pub fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    /// Get intersection with another region
    pub fn intersect(&self, other: &Region) -> Option<Region> {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width).min(other.x + other.width);
        let y2 = (self.y + self.height).min(other.y + other.height);

        if x1 < x2 && y1 < y2 {
            Some(Region::new(x1, y1, x2 - x1, y2 - y1))
        } else {
            None
        }
    }

    /// Split region horizontally
    pub fn split_horizontal(&self, split_point: u16) -> (Region, Region) {
        let split_point = split_point.min(self.height);
        
        let top = Region::new(self.x, self.y, self.width, split_point);
        let bottom = Region::new(
            self.x,
            self.y + split_point,
            self.width,
            self.height - split_point,
        );
        
        (top, bottom)
    }

    /// Split region vertically
    pub fn split_vertical(&self, split_point: u16) -> (Region, Region) {
        let split_point = split_point.min(self.width);
        
        let left = Region::new(self.x, self.y, split_point, self.height);
        let right = Region::new(
            self.x + split_point,
            self.y,
            self.width - split_point,
            self.height,
        );
        
        (left, right)
    }

    /// Add margin to region
    pub fn margin(&self, margin: u16) -> Region {
        let margin = margin.min(self.width / 2).min(self.height / 2);
        
        Region::new(
            self.x + margin,
            self.y + margin,
            self.width.saturating_sub(2 * margin),
            self.height.saturating_sub(2 * margin),
        )
    }

    /// Add padding to region
    pub fn padding(&self, padding: u16) -> Region {
        self.margin(padding)
    }
}

/// Layout direction for arranging widgets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Horizontal,
    Vertical,
}

/// Layout constraint for widget sizing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Constraint {
    /// Fixed size in characters
    Length(u16),
    /// Percentage of available space (0-100)
    Percentage(u16),
    /// Minimum size, takes remaining space
    Min(u16),
    /// Maximum size, takes available space up to limit
    Max(u16),
    /// Fill remaining space equally with other Fill constraints
    Fill,
}

impl Constraint {
    /// Calculate actual size given available space
    pub fn calculate_size(&self, available: u16, min_size: u16) -> u16 {
        match self {
            Constraint::Length(len) => *len,
            Constraint::Percentage(pct) => (available * pct / 100).max(min_size),
            Constraint::Min(min) => (*min).max(min_size),
            Constraint::Max(max) => available.min(*max).max(min_size),
            Constraint::Fill => available.max(min_size),
        }
    }
}

/// Layout configuration
#[derive(Debug, Clone)]
pub struct Layout {
    direction: Direction,
    constraints: Vec<Constraint>,
    margin: u16,
    spacing: u16,
}

impl Layout {
    /// Create new layout
    pub fn new(direction: Direction) -> Self {
        Self {
            direction,
            constraints: Vec::new(),
            margin: 0,
            spacing: 0,
        }
    }

    /// Set constraints for layout
    pub fn constraints(mut self, constraints: Vec<Constraint>) -> Self {
        self.constraints = constraints;
        self
    }

    /// Set margin around layout
    pub fn margin(mut self, margin: u16) -> Self {
        self.margin = margin;
        self
    }

    /// Set spacing between widgets
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Calculate regions for widgets
    pub fn split(&self, region: &Region) -> Vec<Region> {
        if self.constraints.is_empty() {
            return vec![*region];
        }

        // Apply margin
        let work_region = region.margin(self.margin);
        
        match self.direction {
            Direction::Horizontal => self.split_vertical(&work_region),   // Horizontal direction = split vertically into columns
            Direction::Vertical => self.split_horizontal(&work_region),   // Vertical direction = split horizontally into rows
        }
    }

    /// Split region horizontally
    fn split_horizontal(&self, region: &Region) -> Vec<Region> {
        let mut regions = Vec::new();
        let available_height = region.height.saturating_sub(self.spacing * (self.constraints.len() as u16).saturating_sub(1));
        
        // Calculate sizes
        let sizes = self.calculate_sizes(available_height);
        
        let mut current_y = region.y;
        for size in sizes {
            regions.push(Region::new(region.x, current_y, region.width, size));
            current_y += size + self.spacing;
        }
        
        regions
    }

    /// Split region vertically
    fn split_vertical(&self, region: &Region) -> Vec<Region> {
        let mut regions = Vec::new();
        let available_width = region.width.saturating_sub(self.spacing * (self.constraints.len() as u16).saturating_sub(1));
        
        // Calculate sizes
        let sizes = self.calculate_sizes(available_width);
        
        let mut current_x = region.x;
        for size in sizes {
            regions.push(Region::new(current_x, region.y, size, region.height));
            current_x += size + self.spacing;
        }
        
        regions
    }

    /// Calculate actual sizes for constraints
    fn calculate_sizes(&self, available: u16) -> Vec<u16> {
        let mut sizes = vec![0; self.constraints.len()];
        let mut remaining = available;
        let mut fill_count = 0;

        // First pass: calculate fixed sizes and count fill constraints
        for (i, constraint) in self.constraints.iter().enumerate() {
            match constraint {
                Constraint::Length(len) => {
                    sizes[i] = (*len).min(remaining);
                    remaining = remaining.saturating_sub(sizes[i]);
                }
                Constraint::Percentage(pct) => {
                    sizes[i] = (available * pct / 100).min(remaining);
                    remaining = remaining.saturating_sub(sizes[i]);
                }
                Constraint::Min(min) => {
                    sizes[i] = (*min).min(remaining);
                    remaining = remaining.saturating_sub(sizes[i]);
                }
                Constraint::Max(max) => {
                    sizes[i] = remaining.min(*max);
                    remaining = remaining.saturating_sub(sizes[i]);
                }
                Constraint::Fill => {
                    fill_count += 1;
                }
            }
        }

        // Second pass: distribute remaining space to fill constraints
        if fill_count > 0 && remaining > 0 {
            let fill_size = remaining / fill_count;
            let mut extra = remaining % fill_count;

            for (i, constraint) in self.constraints.iter().enumerate() {
                if matches!(constraint, Constraint::Fill) {
                    sizes[i] = fill_size + if extra > 0 { 1 } else { 0 };
                    if extra > 0 {
                        extra -= 1;
                    }
                }
            }
        }
        sizes
    }
}

/// Layout manager for handling complex layouts
pub struct LayoutManager {
    layouts: std::collections::HashMap<String, Layout>,
    current_layout: Option<String>,
}

impl LayoutManager {
    /// Create new layout manager
    pub fn new() -> Self {
        Self {
            layouts: std::collections::HashMap::new(),
            current_layout: None,
        }
    }

    /// Add layout
    pub fn add_layout(&mut self, name: String, layout: Layout) {
        self.layouts.insert(name, layout);
    }

    /// Set current layout
    pub fn set_current_layout(&mut self, name: &str) -> QmsResult<()> {
        if self.layouts.contains_key(name) {
            self.current_layout = Some(name.to_string());
            Ok(())
        } else {
            Err(QmsError::validation_error(&format!("Layout '{}' not found", name)))
        }
    }

    /// Get current layout
    pub fn current_layout(&self) -> Option<&Layout> {
        self.current_layout.as_ref().and_then(|name| self.layouts.get(name))
    }

    /// Split region using current layout
    pub fn split_region(&self, region: &Region) -> Vec<Region> {
        if let Some(layout) = self.current_layout() {
            layout.split(region)
        } else {
            vec![*region]
        }
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Predefined layouts for common QMS screens
pub struct QmsLayouts;

impl QmsLayouts {
    /// Create main menu layout
    pub fn main_menu() -> Layout {
        Layout::new(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(6),  // Header
                Constraint::Fill,       // Menu
                Constraint::Length(3),  // Status bar
            ])
            .margin(1)
            .spacing(1)
    }

    /// Create form layout
    pub fn form() -> Layout {
        Layout::new(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),  // Header
                Constraint::Fill,       // Form content
                Constraint::Length(3),  // Buttons
                Constraint::Length(1),  // Status
            ])
            .margin(1)
            .spacing(1)
    }

    /// Create table layout
    pub fn table() -> Layout {
        Layout::new(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),  // Header
                Constraint::Fill,       // Table content
                Constraint::Length(2),  // Status/pagination
            ])
            .margin(1)
            .spacing(1)
    }

    /// Create split layout (left panel + right content)
    pub fn split_panel() -> Layout {
        Layout::new(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(30), // Left panel
                Constraint::Fill,           // Right content
            ])
            .spacing(1)
    }

    /// Create dashboard layout
    pub fn dashboard() -> Layout {
        Layout::new(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),      // Header
                Constraint::Percentage(60), // Main content
                Constraint::Fill,           // Status/summary
            ])
            .margin(1)
            .spacing(1)
    }
}

/// Layout builder for creating custom layouts
pub struct LayoutBuilder {
    layout: Layout,
}

impl LayoutBuilder {
    /// Create new layout builder
    pub fn new(direction: Direction) -> Self {
        Self {
            layout: Layout::new(direction),
        }
    }

    /// Add constraint
    pub fn constraint(mut self, constraint: Constraint) -> Self {
        self.layout.constraints.push(constraint);
        self
    }

    /// Set margin
    pub fn margin(mut self, margin: u16) -> Self {
        self.layout = self.layout.margin(margin);
        self
    }

    /// Set spacing
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.layout = self.layout.spacing(spacing);
        self
    }

    /// Build the layout
    pub fn build(self) -> Layout {
        self.layout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_creation() {
        let region = Region::new(10, 20, 30, 40);
        assert_eq!(region.x, 10);
        assert_eq!(region.y, 20);
        assert_eq!(region.width, 30);
        assert_eq!(region.height, 40);
        assert_eq!(region.area(), 1200);
    }

    #[test]
    fn test_region_contains() {
        let region = Region::new(10, 20, 30, 40);
        assert!(region.contains(15, 25));
        assert!(region.contains(10, 20)); // Top-left corner
        assert!(!region.contains(40, 60)); // Bottom-right corner (exclusive)
        assert!(!region.contains(5, 15)); // Outside
    }

    #[test]
    fn test_region_split_horizontal() {
        let region = Region::new(0, 0, 100, 50);
        let (top, bottom) = region.split_horizontal(20);
        
        assert_eq!(top, Region::new(0, 0, 100, 20));
        assert_eq!(bottom, Region::new(0, 20, 100, 30));
    }

    #[test]
    fn test_region_split_vertical() {
        let region = Region::new(0, 0, 100, 50);
        let (left, right) = region.split_vertical(30);
        
        assert_eq!(left, Region::new(0, 0, 30, 50));
        assert_eq!(right, Region::new(30, 0, 70, 50));
    }

    #[test]
    fn test_constraint_calculation() {
        assert_eq!(Constraint::Length(10).calculate_size(100, 5), 10);
        assert_eq!(Constraint::Percentage(50).calculate_size(100, 5), 50);
        assert_eq!(Constraint::Min(20).calculate_size(100, 5), 20);
        assert_eq!(Constraint::Max(30).calculate_size(100, 5), 30);
        assert_eq!(Constraint::Fill.calculate_size(100, 5), 100);
    }

    #[test]
    fn test_layout_split() {
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
}
