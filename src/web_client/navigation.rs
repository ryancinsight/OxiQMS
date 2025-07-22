// Navigation Manager - Page Routing and Navigation for WASM Client
// Medical Device QMS - FDA 21 CFR Part 820, ISO 13485, ISO 14971 Compliant
// Handles single-page application routing and navigation state

use crate::prelude::*;
use std::collections::HashMap;

/// Navigation route definition
#[derive(Debug, Clone)]
pub struct NavigationRoute {
    pub name: String,
    pub path: String,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub required_permissions: Vec<String>,
    pub active: bool,
}

impl NavigationRoute {
    pub fn new(name: &str, path: &str, title: &str, icon: &str) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_string(),
            title: title.to_string(),
            description: String::new(),
            icon: icon.to_string(),
            required_permissions: Vec::new(),
            active: false,
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn with_permissions(mut self, permissions: Vec<&str>) -> Self {
        self.required_permissions = permissions.iter().map(|s| s.to_string()).collect();
        self
    }
}

/// Navigation history entry for back/forward functionality
#[derive(Debug, Clone)]
pub struct NavigationHistoryEntry {
    pub route_name: String,
    pub timestamp: u64,
    pub state: HashMap<String, String>,
}

/// Navigation event for tracking navigation changes
#[derive(Debug, Clone)]
pub struct NavigationEvent {
    pub from_route: Option<String>,
    pub to_route: String,
    pub timestamp: u64,
    pub user_initiated: bool,
}

/// Navigation Manager for WASM client routing
pub struct NavigationManager {
    routes: HashMap<String, NavigationRoute>,
    current_route: Option<String>,
    history: Vec<NavigationHistoryEntry>,
    max_history_size: usize,
    navigation_listeners: Vec<String>,
}

impl NavigationManager {
    /// Create new navigation manager with default QMS routes
    pub fn new() -> Self {
        let mut manager = Self {
            routes: HashMap::new(),
            current_route: None,
            history: Vec::new(),
            max_history_size: 50,
            navigation_listeners: Vec::new(),
        };

        manager.setup_default_routes();
        manager
    }

    /// Set up default QMS application routes
    fn setup_default_routes(&mut self) {
        // Dashboard route
        let dashboard = NavigationRoute::new("dashboard", "#dashboard", "Dashboard", "📊")
            .with_description("System overview and key metrics")
            .with_permissions(vec!["READ_DASHBOARD"]);
        self.routes.insert("dashboard".to_string(), dashboard);

        // Documents route
        let documents = NavigationRoute::new("documents", "#documents", "Documents", "📄")
            .with_description("Document management and control per FDA 21 CFR Part 820")
            .with_permissions(vec!["READ_DOCUMENTS"]);
        self.routes.insert("documents".to_string(), documents);

        // Risks route
        let risks = NavigationRoute::new("risks", "#risks", "Risk Management", "⚠️")
            .with_description("Risk assessment and management per ISO 14971")
            .with_permissions(vec!["READ_RISKS"]);
        self.routes.insert("risks".to_string(), risks);

        // Requirements route
        let requirements = NavigationRoute::new("requirements", "#requirements", "Requirements", "📋")
            .with_description("Requirements management and traceability")
            .with_permissions(vec!["READ_REQUIREMENTS"]);
        self.routes.insert("requirements".to_string(), requirements);

        // Audit trail route
        let audit = NavigationRoute::new("audit", "#audit", "Audit Trail", "🔍")
            .with_description("Audit trail and compliance monitoring per 21 CFR Part 11")
            .with_permissions(vec!["READ_AUDIT"]);
        self.routes.insert("audit".to_string(), audit);

        // Reports route
        let reports = NavigationRoute::new("reports", "#reports", "Reports", "📈")
            .with_description("Compliance reports and documentation")
            .with_permissions(vec!["GENERATE_REPORTS"]);
        self.routes.insert("reports".to_string(), reports);

        // User management route (admin only)
        let users = NavigationRoute::new("users", "#users", "User Management", "👥")
            .with_description("User and role management")
            .with_permissions(vec!["MANAGE_USERS"]);
        self.routes.insert("users".to_string(), users);

        // Settings route
        let settings = NavigationRoute::new("settings", "#settings", "Settings", "⚙️")
            .with_description("System configuration and preferences")
            .with_permissions(vec!["CONFIGURE_SYSTEM"]);
        self.routes.insert("settings".to_string(), settings);
    }

    /// Navigate to a specific route
    pub fn navigate_to(&mut self, route_name: &str) -> QmsResult<NavigationEvent> {
        // Validate route exists
        if !self.routes.contains_key(route_name) {
            return Err(QmsError::validation_error(&format!("Unknown route: {route_name}")));
        }

        // Check permissions (placeholder - would integrate with user management)
        if !self.check_route_permissions(route_name)? {
            return Err(QmsError::auth_error(&format!("Insufficient permissions for route: {route_name}")));
        }

        // Create navigation event
        let event = NavigationEvent {
            from_route: self.current_route.clone(),
            to_route: route_name.to_string(),
            timestamp: self.current_timestamp(),
            user_initiated: true,
        };

        // Update history
        self.add_to_history(route_name)?;

        // Update active route
        self.set_active_route(route_name)?;

        // Notify listeners
        self.notify_navigation_change(&event)?;

        Ok(event)
    }

    /// Set active route and update navigation state
    pub fn set_active_section(&mut self, section: &str) -> QmsResult<()> {
        self.navigate_to(section)?;
        Ok(())
    }

    /// Get current active route
    pub fn get_current_route(&self) -> Option<&String> {
        self.current_route.as_ref()
    }

    /// Get route information
    pub fn get_route(&self, route_name: &str) -> Option<&NavigationRoute> {
        self.routes.get(route_name)
    }

    /// Get all available routes for current user
    pub fn get_available_routes(&self) -> Vec<&NavigationRoute> {
        self.routes.values()
            .filter(|route| self.check_route_permissions(&route.name).unwrap_or(false))
            .collect()
    }

    /// Navigate back to previous route
    pub fn navigate_back(&mut self) -> QmsResult<Option<NavigationEvent>> {
        if self.history.len() < 2 {
            return Ok(None); // No previous route
        }

        // Get previous route (skip current)
        let previous_entry = &self.history[self.history.len() - 2];
        let previous_route = previous_entry.route_name.clone();

        // Navigate to previous route
        let event = self.navigate_to(&previous_route)?;
        Ok(Some(event))
    }

    /// Navigate forward (if available)
    pub fn navigate_forward(&mut self) -> QmsResult<Option<NavigationEvent>> {
        // In a full implementation, this would maintain forward history
        // For now, this is a placeholder
        Ok(None)
    }

    /// Get navigation history
    pub fn get_history(&self) -> &Vec<NavigationHistoryEntry> {
        &self.history
    }

    /// Clear navigation history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Check if user has permissions for a route
    fn check_route_permissions(&self, route_name: &str) -> QmsResult<bool> {
        let route = self.routes.get(route_name)
            .ok_or_else(|| QmsError::not_found(&format!("Route not found: {route_name}")))?;

        // For now, assume all permissions are granted
        // In a full implementation, this would check user roles and permissions
        if route.required_permissions.is_empty() {
            return Ok(true);
        }

        // TODO: Integrate with user management system
        // let user = get_current_user()?;
        // for permission in &route.required_permissions {
        //     if !user.has_permission(permission) {
        //         return Ok(false);
        //     }
        // }

        Ok(true) // Temporary - allow all navigation
    }

    /// Set active route and update UI state
    fn set_active_route(&mut self, route_name: &str) -> QmsResult<()> {
        // Clear all active states
        for route in self.routes.values_mut() {
            route.active = false;
        }

        // Set new active route
        if let Some(route) = self.routes.get_mut(route_name) {
            route.active = true;
            self.current_route = Some(route_name.to_string());
        }

        Ok(())
    }

    /// Add navigation to history
    fn add_to_history(&mut self, route_name: &str) -> QmsResult<()> {
        let entry = NavigationHistoryEntry {
            route_name: route_name.to_string(),
            timestamp: self.current_timestamp(),
            state: HashMap::new(), // Could store route-specific state
        };

        self.history.push(entry);

        // Limit history size
        if self.history.len() > self.max_history_size {
            self.history.remove(0);
        }

        Ok(())
    }

    /// Notify navigation change listeners
    fn notify_navigation_change(&self, event: &NavigationEvent) -> QmsResult<()> {
        // In a full WASM implementation, this would call registered callbacks
        // For now, just log the navigation event
        eprintln!("Navigation: {} -> {}", 
            event.from_route.as_deref().unwrap_or("none"), 
            event.to_route);
        Ok(())
    }

    /// Get current timestamp
    fn current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Add navigation listener
    pub fn add_navigation_listener(&mut self, listener_id: &str) {
        if !self.navigation_listeners.contains(&listener_id.to_string()) {
            self.navigation_listeners.push(listener_id.to_string());
        }
    }

    /// Remove navigation listener
    pub fn remove_navigation_listener(&mut self, listener_id: &str) {
        self.navigation_listeners.retain(|id| id != listener_id);
    }

    /// Get navigation breadcrumbs for current route
    pub fn get_breadcrumbs(&self) -> Vec<String> {
        let mut breadcrumbs = vec!["Home".to_string()];
        
        if let Some(current) = &self.current_route {
            if let Some(route) = self.routes.get(current) {
                breadcrumbs.push(route.title.clone());
            }
        }

        breadcrumbs
    }

    /// Generate navigation menu HTML
    pub fn generate_navigation_menu(&self) -> String {
        let mut menu_html = String::from(r#"<nav class="navigation-menu">"#);

        for route in self.get_available_routes() {
            let active_class = if route.active { " active" } else { "" };
            let menu_item = format!(
                r#"<a href="{}" class="nav-link{}" data-route="{}">
                    <span class="nav-icon">{}</span>
                    <span class="nav-title">{}</span>
                    <span class="nav-description">{}</span>
                </a>"#,
                route.path, active_class, route.name, route.icon, route.title, route.description
            );
            menu_html.push_str(&menu_item);
        }

        menu_html.push_str("</nav>");
        menu_html
    }

    /// Generate route information for debugging
    pub fn get_route_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::new();
        
        info.insert("current_route".to_string(), 
            self.current_route.as_deref().unwrap_or("none").to_string());
        info.insert("total_routes".to_string(), self.routes.len().to_string());
        info.insert("history_size".to_string(), self.history.len().to_string());
        info.insert("listeners".to_string(), self.navigation_listeners.len().to_string());

        info
    }

    /// Validate navigation state
    pub fn validate_state(&self) -> QmsResult<bool> {
        // Check if current route exists
        if let Some(current) = &self.current_route {
            if !self.routes.contains_key(current) {
                return Err(QmsError::validation_error("Current route does not exist"));
            }
        }

        // Check history integrity
        for entry in &self.history {
            if !self.routes.contains_key(&entry.route_name) {
                return Err(QmsError::validation_error("History contains invalid route"));
            }
        }

        Ok(true)
    }

    /// Reset navigation to default state
    pub fn reset(&mut self) {
        self.current_route = None;
        self.history.clear();
        
        // Clear all active states
        for route in self.routes.values_mut() {
            route.active = false;
        }

        // Navigate to dashboard by default
        let _ = self.navigate_to("dashboard");
    }
}

impl Default for NavigationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_manager_creation() {
        let manager = NavigationManager::new();
        assert!(!manager.routes.is_empty());
        assert!(manager.routes.contains_key("dashboard"));
        assert!(manager.routes.contains_key("documents"));
        assert!(manager.routes.contains_key("risks"));
    }

    #[test]
    fn test_navigation_to_valid_route() {
        let mut manager = NavigationManager::new();
        let event = manager.navigate_to("dashboard").unwrap();
        
        assert_eq!(event.to_route, "dashboard");
        assert_eq!(manager.get_current_route(), Some(&"dashboard".to_string()));
        assert!(manager.routes.get("dashboard").unwrap().active);
    }

    #[test]
    fn test_navigation_to_invalid_route() {
        let mut manager = NavigationManager::new();
        let result = manager.navigate_to("invalid_route");
        assert!(result.is_err());
    }

    #[test]
    fn test_navigation_history() {
        let mut manager = NavigationManager::new();
        
        manager.navigate_to("dashboard").unwrap();
        manager.navigate_to("documents").unwrap();
        manager.navigate_to("risks").unwrap();
        
        assert_eq!(manager.history.len(), 3);
        assert_eq!(manager.history[0].route_name, "dashboard");
        assert_eq!(manager.history[1].route_name, "documents");
        assert_eq!(manager.history[2].route_name, "risks");
    }

    #[test]
    fn test_navigation_back() {
        let mut manager = NavigationManager::new();
        
        manager.navigate_to("dashboard").unwrap();
        manager.navigate_to("documents").unwrap();
        
        let back_event = manager.navigate_back().unwrap();
        assert!(back_event.is_some());
        assert_eq!(manager.get_current_route(), Some(&"dashboard".to_string()));
    }

    #[test]
    fn test_available_routes() {
        let manager = NavigationManager::new();
        let available = manager.get_available_routes();
        assert!(!available.is_empty());
        assert!(available.iter().any(|r| r.name == "dashboard"));
    }

    #[test]
    fn test_breadcrumbs() {
        let mut manager = NavigationManager::new();
        manager.navigate_to("documents").unwrap();
        
        let breadcrumbs = manager.get_breadcrumbs();
        assert_eq!(breadcrumbs.len(), 2);
        assert_eq!(breadcrumbs[0], "Home");
        assert_eq!(breadcrumbs[1], "Documents");
    }

    #[test]
    fn test_navigation_menu_generation() {
        let manager = NavigationManager::new();
        let menu_html = manager.generate_navigation_menu();
        
        assert!(menu_html.contains("navigation-menu"));
        assert!(menu_html.contains("dashboard"));
        assert!(menu_html.contains("documents"));
        assert!(menu_html.contains("📊")); // Dashboard icon
    }

    #[test]
    fn test_navigation_state_validation() {
        let manager = NavigationManager::new();
        assert!(manager.validate_state().unwrap());
    }

    #[test]
    fn test_navigation_reset() {
        let mut manager = NavigationManager::new();
        
        manager.navigate_to("documents").unwrap();
        manager.navigate_to("risks").unwrap();
        
        manager.reset();
        
        assert_eq!(manager.get_current_route(), Some(&"dashboard".to_string()));
        assert!(manager.routes.get("dashboard").unwrap().active);
        assert!(!manager.routes.get("documents").unwrap().active);
    }

    #[test]
    fn test_route_information() {
        let manager = NavigationManager::new();
        let dashboard_route = manager.get_route("dashboard").unwrap();
        
        assert_eq!(dashboard_route.name, "dashboard");
        assert_eq!(dashboard_route.title, "Dashboard");
        assert_eq!(dashboard_route.icon, "📊");
        assert!(dashboard_route.description.contains("overview"));
    }
}
