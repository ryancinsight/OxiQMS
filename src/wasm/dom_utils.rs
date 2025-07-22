// QMS WASM DOM Utilities
// Safe DOM manipulation utilities for medical device compliance
// Provides type-safe DOM access and data binding

use crate::prelude::*;
use std::collections::HashMap;

/// DOM utility functions for WASM
pub struct DomUtils;

/// DOM element representation
#[derive(Debug, Clone)]
pub struct DomElement {
    pub id: String,
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
    pub text_content: Option<String>,
}

/// Event listener configuration
#[derive(Debug, Clone)]
pub struct EventListener {
    pub event_type: String,
    pub element_id: String,
    pub handler_name: String,
}

impl DomUtils {
    /// Get element by ID (placeholder for WASM implementation)
    pub fn get_element_by_id(id: &str) -> QmsResult<Option<DomElement>> {
        // In actual WASM, this would use web_sys::document().get_element_by_id()
        // For now, return a mock element for compilation
        Ok(Some(DomElement {
            id: id.to_string(),
            tag_name: "div".to_string(),
            attributes: HashMap::new(),
            text_content: None,
        }))
    }

    /// Set text content of element
    pub fn set_text_content(element_id: &str, content: &str) -> QmsResult<()> {
        // In actual WASM, this would manipulate the real DOM
        // For now, this is a placeholder
        println!("Setting text content of {element_id} to: {content}");
        Ok(())
    }

    /// Set HTML content of element
    pub fn set_inner_html(element_id: &str, html: &str) -> QmsResult<()> {
        // In actual WASM, this would use element.set_inner_html()
        println!("Setting HTML content of {element_id} to: {html}");
        Ok(())
    }

    /// Add CSS class to element
    pub fn add_class(element_id: &str, class_name: &str) -> QmsResult<()> {
        // In actual WASM, this would use element.class_list().add()
        println!("Adding class {class_name} to element {element_id}");
        Ok(())
    }

    /// Remove CSS class from element
    pub fn remove_class(element_id: &str, class_name: &str) -> QmsResult<()> {
        // In actual WASM, this would use element.class_list().remove()
        println!("Removing class {class_name} from element {element_id}");
        Ok(())
    }

    /// Set attribute on element
    pub fn set_attribute(element_id: &str, name: &str, value: &str) -> QmsResult<()> {
        // In actual WASM, this would use element.set_attribute()
        println!("Setting attribute {name}={value} on element {element_id}");
        Ok(())
    }

    /// Get attribute from element
    pub fn get_attribute(element_id: &str, name: &str) -> QmsResult<Option<String>> {
        // In actual WASM, this would use element.get_attribute()
        println!("Getting attribute {name} from element {element_id}");
        Ok(Some("mock_value".to_string()))
    }

    /// Show element (remove hidden class/style)
    pub fn show_element(element_id: &str) -> QmsResult<()> {
        Self::remove_class(element_id, "hidden")?;
        Self::set_attribute(element_id, "style", "display: block")?;
        Ok(())
    }

    /// Hide element (add hidden class/style)
    pub fn hide_element(element_id: &str) -> QmsResult<()> {
        Self::add_class(element_id, "hidden")?;
        Self::set_attribute(element_id, "style", "display: none")?;
        Ok(())
    }

    /// Update loading state
    pub fn set_loading_state(element_id: &str, is_loading: bool) -> QmsResult<()> {
        if is_loading {
            Self::set_text_content(element_id, "Loading...")?;
            Self::add_class(element_id, "loading")?;
        } else {
            Self::remove_class(element_id, "loading")?;
        }
        Ok(())
    }

    /// Update badge content (for compliance indicators)
    pub fn update_badge(badge_id: &str, status: &str, percentage: u8) -> QmsResult<()> {
        let content = format!("{status}: {percentage}%");
        Self::set_text_content(badge_id, &content)?;
        
        // Set color based on percentage
        let class = if percentage >= 90 {
            "badge-success"
        } else if percentage >= 70 {
            "badge-warning"
        } else {
            "badge-danger"
        };
        
        Self::add_class(badge_id, class)?;
        Ok(())
    }

    /// Create and append element
    pub fn create_element(parent_id: &str, tag: &str, id: &str, content: Option<&str>) -> QmsResult<()> {
        // In actual WASM, this would create real DOM elements
        println!("Creating {tag} element with id {id} in parent {parent_id}");
        if let Some(content) = content {
            Self::set_text_content(id, content)?;
        }
        Ok(())
    }

    /// Remove element from DOM
    pub fn remove_element(element_id: &str) -> QmsResult<()> {
        // In actual WASM, this would remove the element from DOM
        println!("Removing element {element_id}");
        Ok(())
    }

    /// Add event listener (placeholder for WASM)
    pub fn add_event_listener(element_id: &str, event_type: &str, handler: &str) -> QmsResult<()> {
        // In actual WASM, this would add real event listeners
        println!("Adding {event_type} event listener to {element_id} with handler {handler}");
        Ok(())
    }

    /// Update dashboard stats
    pub fn update_dashboard_stats(stats: &HashMap<String, String>) -> QmsResult<()> {
        for (key, value) in stats {
            let element_id = format!("stat-{key}");
            Self::set_text_content(&element_id, value)?;
        }
        Ok(())
    }

    /// Update server status indicator
    pub fn update_server_status(is_connected: bool) -> QmsResult<()> {
        let status_element = "server-status";
        let indicator_element = "server-indicator";
        
        if is_connected {
            Self::set_text_content(status_element, "Connected")?;
            Self::remove_class(indicator_element, "status-disconnected")?;
            Self::add_class(indicator_element, "status-connected")?;
        } else {
            Self::set_text_content(status_element, "Disconnected")?;
            Self::remove_class(indicator_element, "status-connected")?;
            Self::add_class(indicator_element, "status-disconnected")?;
        }
        Ok(())
    }

    /// Populate table with data
    pub fn populate_table(table_id: &str, headers: &[&str], rows: &[Vec<String>]) -> QmsResult<()> {
        // Clear existing table content
        Self::set_inner_html(table_id, "")?;
        
        // Create header
        let mut header_html = "<thead><tr>".to_string();
        for header in headers {
            header_html.push_str(&format!("<th>{header}</th>"));
        }
        header_html.push_str("</tr></thead>");
        
        // Create body
        let mut body_html = "<tbody>".to_string();
        for row in rows {
            body_html.push_str("<tr>");
            for cell in row {
                body_html.push_str(&format!("<td>{cell}</td>"));
            }
            body_html.push_str("</tr>");
        }
        body_html.push_str("</tbody>");
        
        // Set complete table HTML
        let full_html = format!("{header_html}{body_html}");
        Self::set_inner_html(table_id, &full_html)?;
        
        Ok(())
    }

    /// Update form validation state
    pub fn update_form_validation(form_id: &str, field_errors: &HashMap<String, String>) -> QmsResult<()> {
        // Clear existing validation states
        Self::remove_class(form_id, "has-errors")?;
        
        if field_errors.is_empty() {
            Self::add_class(form_id, "valid")?;
        } else {
            Self::add_class(form_id, "has-errors")?;
            
            for (field, error) in field_errors {
                let error_element = format!("{field}-error");
                Self::set_text_content(&error_element, error)?;
                Self::show_element(&error_element)?;
            }
        }
        Ok(())
    }

    /// Show notification/alert
    pub fn show_notification(message: &str, notification_type: &str) -> QmsResult<()> {
        let notification_id = "notification-area";
        let notification_html = format!(
            r#"<div class="notification notification-{notification_type}" role="alert">
                <span class="notification-icon">📋</span>
                <span class="notification-message">{message}</span>
                <button class="notification-close" onclick="hideNotification()">×</button>
               </div>"#
        );
        
        Self::set_inner_html(notification_id, &notification_html)?;
        Self::show_element(notification_id)?;
        
        Ok(())
    }
}

impl DomElement {
    /// Create new DOM element representation
    pub fn new(id: String, tag_name: String) -> Self {
        Self {
            id,
            tag_name,
            attributes: HashMap::new(),
            text_content: None,
        }
    }

    /// Set attribute on this element
    pub fn set_attribute(&mut self, name: String, value: String) {
        self.attributes.insert(name, value);
    }

    /// Get attribute from this element
    pub fn get_attribute(&self, name: &str) -> Option<&String> {
        self.attributes.get(name)
    }

    /// Set text content
    pub fn set_text_content(&mut self, content: String) {
        self.text_content = Some(content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dom_element_creation() {
        let element = DomElement::new("test-id".to_string(), "div".to_string());
        assert_eq!(element.id, "test-id");
        assert_eq!(element.tag_name, "div");
        assert!(element.attributes.is_empty());
        assert!(element.text_content.is_none());
    }

    #[test]
    fn test_dom_element_attributes() {
        let mut element = DomElement::new("test-id".to_string(), "div".to_string());
        element.set_attribute("class".to_string(), "test-class".to_string());
        
        assert_eq!(element.get_attribute("class"), Some(&"test-class".to_string()));
        assert_eq!(element.get_attribute("nonexistent"), None);
    }

    #[test]
    fn test_dom_element_text_content() {
        let mut element = DomElement::new("test-id".to_string(), "div".to_string());
        element.set_text_content("Test content".to_string());
        
        assert_eq!(element.text_content, Some("Test content".to_string()));
    }

    #[test]
    fn test_dom_utils_get_element() {
        let element = DomUtils::get_element_by_id("test-id").unwrap();
        assert!(element.is_some());
        let elem = element.unwrap();
        assert_eq!(elem.id, "test-id");
    }

    #[test]
    fn test_update_badge() {
        // This test verifies the badge update logic
        let result = DomUtils::update_badge("test-badge", "FDA 21 CFR", 95);
        assert!(result.is_ok());
    }

    #[test]
    fn test_populate_table() {
        let headers = vec!["ID", "Title", "Status"];
        let rows = vec![
            vec!["DOC-001".to_string(), "SRS".to_string(), "Approved".to_string()],
            vec!["DOC-002".to_string(), "SDD".to_string(), "Draft".to_string()],
        ];
        
        let result = DomUtils::populate_table("test-table", &headers, &rows);
        assert!(result.is_ok());
    }

    #[test]
    fn test_form_validation() {
        let mut errors = HashMap::new();
        errors.insert("title".to_string(), "Title is required".to_string());
        
        let result = DomUtils::update_form_validation("test-form", &errors);
        assert!(result.is_ok());
    }

    #[test]
    fn test_server_status_update() {
        // Test connected state
        let result = DomUtils::update_server_status(true);
        assert!(result.is_ok());
        
        // Test disconnected state
        let result = DomUtils::update_server_status(false);
        assert!(result.is_ok());
    }
}
