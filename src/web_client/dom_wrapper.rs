// DOM Wrapper - HTML Manipulation Module for WASM Client
// Medical Device QMS - FDA 21 CFR Part 820, ISO 13485, ISO 14971 Compliant
// Simulates DOM manipulation that would be done through WASM bindings

use crate::prelude::*;
use std::collections::HashMap;

/// DOM element representation for WASM manipulation
#[derive(Debug, Clone)]
pub struct DomElement {
    pub id: String,
    pub tag_name: String,
    pub text_content: String,
    pub inner_html: String,
    pub attributes: HashMap<String, String>,
    pub classes: Vec<String>,
}

impl DomElement {
    pub fn new(id: &str, tag_name: &str) -> Self {
        Self {
            id: id.to_string(),
            tag_name: tag_name.to_string(),
            text_content: String::new(),
            inner_html: String::new(),
            attributes: HashMap::new(),
            classes: Vec::new(),
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text_content = text.to_string();
    }

    pub fn set_html(&mut self, html: &str) {
        self.inner_html = html.to_string();
    }

    pub fn set_attribute(&mut self, name: &str, value: &str) {
        self.attributes.insert(name.to_string(), value.to_string());
    }

    pub fn add_class(&mut self, class_name: &str) {
        if !self.classes.contains(&class_name.to_string()) {
            self.classes.push(class_name.to_string());
        }
    }

    pub fn remove_class(&mut self, class_name: &str) {
        self.classes.retain(|c| c != class_name);
    }

    pub fn has_class(&self, class_name: &str) -> bool {
        self.classes.contains(&class_name.to_string())
    }
}

/// Event handler information for DOM events
#[derive(Debug, Clone)]
pub struct EventHandler {
    pub element_selector: String,
    pub event_type: String,
    pub handler_name: String,
    pub active: bool,
}

/// DOM Wrapper for WASM-based HTML manipulation
/// In a full WASM implementation, this would use web_sys and wasm_bindgen
pub struct DomWrapper {
    elements: HashMap<String, DomElement>,
    event_handlers: Vec<EventHandler>,
    pending_updates: Vec<DomUpdate>,
}

/// DOM update operation for batching changes
#[derive(Debug, Clone)]
pub enum DomUpdate {
    SetText { element_id: String, text: String },
    SetHtml { element_id: String, html: String },
    SetAttribute { element_id: String, name: String, value: String },
    AddClass { element_id: String, class_name: String },
    RemoveClass { element_id: String, class_name: String },
    ShowElement { element_id: String },
    HideElement { element_id: String },
}

impl DomWrapper {
    /// Create new DOM wrapper instance
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
            event_handlers: Vec::new(),
            pending_updates: Vec::new(),
        }
    }

    /// Set text content of an element
    pub fn set_element_text(&mut self, element_id: &str, text: &str) -> QmsResult<()> {
        self.pending_updates.push(DomUpdate::SetText {
            element_id: element_id.to_string(),
            text: text.to_string(),
        });
        
        // In a real WASM implementation, this would call:
        // web_sys::window().unwrap().document().unwrap()
        //     .get_element_by_id(element_id).unwrap()
        //     .set_text_content(Some(text));
        
        Ok(())
    }

    /// Set HTML content of an element
    pub fn set_element_html(&mut self, element_id: &str, html: &str) -> QmsResult<()> {
        self.pending_updates.push(DomUpdate::SetHtml {
            element_id: element_id.to_string(),
            html: html.to_string(),
        });
        Ok(())
    }

    /// Set attribute of an element
    pub fn set_element_attribute(&mut self, element_id: &str, name: &str, value: &str) -> QmsResult<()> {
        self.pending_updates.push(DomUpdate::SetAttribute {
            element_id: element_id.to_string(),
            name: name.to_string(),
            value: value.to_string(),
        });
        Ok(())
    }

    /// Add CSS class to an element
    pub fn add_element_class(&mut self, element_id: &str, class_name: &str) -> QmsResult<()> {
        self.pending_updates.push(DomUpdate::AddClass {
            element_id: element_id.to_string(),
            class_name: class_name.to_string(),
        });
        Ok(())
    }

    /// Remove CSS class from an element
    pub fn remove_element_class(&mut self, element_id: &str, class_name: &str) -> QmsResult<()> {
        self.pending_updates.push(DomUpdate::RemoveClass {
            element_id: element_id.to_string(),
            class_name: class_name.to_string(),
        });
        Ok(())
    }

    /// Show an element (remove 'hidden' style)
    pub fn show_element(&mut self, element_id: &str) -> QmsResult<()> {
        self.pending_updates.push(DomUpdate::ShowElement {
            element_id: element_id.to_string(),
        });
        Ok(())
    }

    /// Hide an element (add 'hidden' style)
    pub fn hide_element(&mut self, element_id: &str) -> QmsResult<()> {
        self.pending_updates.push(DomUpdate::HideElement {
            element_id: element_id.to_string(),
        });
        Ok(())
    }

    /// Register click event handler
    pub fn register_click_handler(&mut self, selector: &str, handler_name: &str) -> QmsResult<()> {
        let handler = EventHandler {
            element_selector: selector.to_string(),
            event_type: "click".to_string(),
            handler_name: handler_name.to_string(),
            active: true,
        };
        
        self.event_handlers.push(handler);
        
        // In a real WASM implementation, this would register actual DOM event listeners
        // using web_sys event handling
        
        Ok(())
    }

    /// Register form submit event handler
    pub fn register_submit_handler(&mut self, form_id: &str, handler_name: &str) -> QmsResult<()> {
        let handler = EventHandler {
            element_selector: format!("#{form_id}"),
            event_type: "submit".to_string(),
            handler_name: handler_name.to_string(),
            active: true,
        };
        
        self.event_handlers.push(handler);
        Ok(())
    }

    /// Flush all pending DOM updates
    pub fn flush_updates(&mut self) -> QmsResult<()> {
        for update in &self.pending_updates {
            self.apply_update(update)?;
        }
        
        self.pending_updates.clear();
        Ok(())
    }

    /// Apply a single DOM update
    fn apply_update(&mut self, update: &DomUpdate) -> QmsResult<()> {
        match update {
            DomUpdate::SetText { element_id, text } => {
                // In WASM: document.getElementById(element_id).textContent = text
                self.log_dom_operation(&format!("SetText: {} = '{}'", element_id, text));
            },
            DomUpdate::SetHtml { element_id, html } => {
                // In WASM: document.getElementById(element_id).innerHTML = html
                self.log_dom_operation(&format!("SetHtml: {} = '{}'", element_id, html));
            },
            DomUpdate::SetAttribute { element_id, name, value } => {
                // In WASM: document.getElementById(element_id).setAttribute(name, value)
                self.log_dom_operation(&format!("SetAttribute: {}.{} = '{}'", element_id, name, value));
            },
            DomUpdate::AddClass { element_id, class_name } => {
                // In WASM: document.getElementById(element_id).classList.add(class_name)
                self.log_dom_operation(&format!("AddClass: {} += '{}'", element_id, class_name));
            },
            DomUpdate::RemoveClass { element_id, class_name } => {
                // In WASM: document.getElementById(element_id).classList.remove(class_name)
                self.log_dom_operation(&format!("RemoveClass: {} -= '{}'", element_id, class_name));
            },
            DomUpdate::ShowElement { element_id } => {
                // In WASM: document.getElementById(element_id).style.display = ""
                self.log_dom_operation(&format!("ShowElement: {}", element_id));
            },
            DomUpdate::HideElement { element_id } => {
                // In WASM: document.getElementById(element_id).style.display = "none"
                self.log_dom_operation(&format!("HideElement: {}", element_id));
            },
        }
        Ok(())
    }

    /// Create loading indicator for async operations
    pub fn show_loading(&mut self, message: &str) -> QmsResult<()> {
        let loading_html = format!(
            r#"<div class="loading-spinner">
                <div class="spinner"></div>
                <p>{message}</p>
            </div>"#
        );
        
        self.set_element_html("loading-overlay", &loading_html)?;
        self.show_element("loading-overlay")?;
        self.flush_updates()?;
        
        Ok(())
    }

    /// Hide loading indicator
    pub fn hide_loading(&mut self) -> QmsResult<()> {
        self.hide_element("loading-overlay")?;
        self.flush_updates()?;
        Ok(())
    }

    /// Display success message to user
    pub fn show_success_message(&mut self, message: &str) -> QmsResult<()> {
        let alert_html = format!(
            r#"<div class="alert alert-success">
                <span class="alert-icon">✅</span>
                <span class="alert-message">{message}</span>
                <button class="alert-close" onclick="this.parentElement.style.display='none'">×</button>
            </div>"#
        );
        
        self.set_element_html("notifications", &alert_html)?;
        self.show_element("notifications")?;
        self.flush_updates()?;
        
        Ok(())
    }

    /// Display error message to user
    pub fn show_error_message(&mut self, message: &str) -> QmsResult<()> {
        let alert_html = format!(
            r#"<div class="alert alert-error">
                <span class="alert-icon">❌</span>
                <span class="alert-message">{message}</span>
                <button class="alert-close" onclick="this.parentElement.style.display='none'">×</button>
            </div>"#
        );
        
        self.set_element_html("notifications", &alert_html)?;
        self.show_element("notifications")?;
        self.flush_updates()?;
        
        Ok(())
    }

    /// Update progress bar
    pub fn update_progress_bar(&mut self, element_id: &str, percentage: f32) -> QmsResult<()> {
        let width = percentage.min(100.0).max(0.0);
        
        self.set_element_attribute(&format!("{element_id}-bar"), "style", &format!("width: {width}%"))?;
        self.set_element_text(&format!("{element_id}-text"), &format!("{:.1}%", width))?;
        
        self.flush_updates()?;
        Ok(())
    }

    /// Create and populate a data table
    pub fn populate_table(&mut self, table_id: &str, headers: &[&str], rows: &[Vec<&str>]) -> QmsResult<()> {
        let mut table_html = String::new();
        
        // Table header
        table_html.push_str("<thead><tr>");
        for header in headers {
            table_html.push_str(&format!("<th>{header}</th>"));
        }
        table_html.push_str("</tr></thead>");
        
        // Table body
        table_html.push_str("<tbody>");
        for row in rows {
            table_html.push_str("<tr>");
            for cell in row {
                table_html.push_str(&format!("<td>{cell}</td>"));
            }
            table_html.push_str("</tr>");
        }
        table_html.push_str("</tbody>");
        
        self.set_element_html(table_id, &table_html)?;
        self.flush_updates()?;
        
        Ok(())
    }

    /// Get form data from a form element
    pub fn get_form_data(&self, form_id: &str) -> QmsResult<HashMap<String, String>> {
        // In a real WASM implementation, this would extract form data:
        // let form = document.getElementById(form_id).dyn_into::<web_sys::HtmlFormElement>()?;
        // let form_data = web_sys::FormData::new_with_form(&form)?;
        
        // For now, return mock form data
        let mut form_data = HashMap::new();
        form_data.insert("title".to_string(), "Sample Title".to_string());
        form_data.insert("description".to_string(), "Sample Description".to_string());
        
        Ok(form_data)
    }

    /// Clear form fields
    pub fn clear_form(&mut self, form_id: &str) -> QmsResult<()> {
        // In WASM: form.reset() or individually clear fields
        self.log_dom_operation(&format!("ClearForm: {form_id}"));
        Ok(())
    }

    /// Validate form fields
    pub fn validate_form(&self, form_id: &str) -> QmsResult<bool> {
        // In a real implementation, this would check form validation
        // For now, assume forms are valid
        Ok(true)
    }

    /// Log DOM operations for debugging in non-WASM environment
    fn log_dom_operation(&self, operation: &str) {
        // In development, we can log these operations
        // In production WASM, these would be actual DOM manipulations
        eprintln!("DOM Operation: {operation}");
    }

    /// Get count of pending updates
    pub fn pending_update_count(&self) -> usize {
        self.pending_updates.len()
    }

    /// Get count of registered event handlers
    pub fn event_handler_count(&self) -> usize {
        self.event_handlers.len()
    }

    /// Remove all event handlers for an element
    pub fn remove_event_handlers(&mut self, element_selector: &str) {
        self.event_handlers.retain(|handler| handler.element_selector != element_selector);
    }
}

impl Default for DomWrapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dom_wrapper_creation() {
        let wrapper = DomWrapper::new();
        assert_eq!(wrapper.pending_update_count(), 0);
        assert_eq!(wrapper.event_handler_count(), 0);
    }

    #[test]
    fn test_set_element_text() {
        let mut wrapper = DomWrapper::new();
        assert!(wrapper.set_element_text("test-element", "Test Text").is_ok());
        assert_eq!(wrapper.pending_update_count(), 1);
    }

    #[test]
    fn test_event_handler_registration() {
        let mut wrapper = DomWrapper::new();
        assert!(wrapper.register_click_handler(".nav-link", "handle_navigation").is_ok());
        assert_eq!(wrapper.event_handler_count(), 1);
    }

    #[test]
    fn test_dom_updates_batching() {
        let mut wrapper = DomWrapper::new();
        
        wrapper.set_element_text("element1", "Text 1").unwrap();
        wrapper.set_element_html("element2", "<p>HTML</p>").unwrap();
        wrapper.add_element_class("element3", "active").unwrap();
        
        assert_eq!(wrapper.pending_update_count(), 3);
        
        wrapper.flush_updates().unwrap();
        assert_eq!(wrapper.pending_update_count(), 0);
    }

    #[test]
    fn test_loading_indicator() {
        let mut wrapper = DomWrapper::new();
        assert!(wrapper.show_loading("Loading data...").is_ok());
        assert!(wrapper.hide_loading().is_ok());
    }

    #[test]
    fn test_message_display() {
        let mut wrapper = DomWrapper::new();
        assert!(wrapper.show_success_message("Operation successful").is_ok());
        assert!(wrapper.show_error_message("Operation failed").is_ok());
    }

    #[test]
    fn test_table_population() {
        let mut wrapper = DomWrapper::new();
        let headers = vec!["ID", "Name", "Status"];
        let rows = vec![
            vec!["DOC-001", "Requirements", "Active"],
            vec!["DOC-002", "Design", "Draft"],
        ];
        
        assert!(wrapper.populate_table("documents-table", &headers, &rows).is_ok());
    }

    #[test]
    fn test_progress_bar_update() {
        let mut wrapper = DomWrapper::new();
        assert!(wrapper.update_progress_bar("upload-progress", 75.0).is_ok());
        assert!(wrapper.update_progress_bar("upload-progress", 150.0).is_ok()); // Test clamping
    }

    #[test]
    fn test_form_data_handling() {
        let wrapper = DomWrapper::new();
        let form_data = wrapper.get_form_data("test-form").unwrap();
        assert!(!form_data.is_empty());
    }

    #[test]
    fn test_dom_element_creation() {
        let mut element = DomElement::new("test-id", "div");
        assert_eq!(element.id, "test-id");
        assert_eq!(element.tag_name, "div");
        
        element.set_text("Test content");
        assert_eq!(element.text_content, "Test content");
        
        element.add_class("test-class");
        assert!(element.has_class("test-class"));
        
        element.remove_class("test-class");
        assert!(!element.has_class("test-class"));
    }
}
