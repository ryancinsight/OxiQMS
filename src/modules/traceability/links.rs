// Traceability links functionality
// Implementation for Task 3.2.4 Traceability Link Management

use std::fs;
use std::path::Path;
use std::collections::HashSet;
use crate::prelude::*;
use crate::utils::{generate_uuid, current_timestamp};
use crate::modules::audit_logger::functions::audit_log_create;

#[derive(Debug, Clone)]
pub struct TraceabilityLink {
    pub id: String,
    pub source_type: String,
    pub source_id: String,
    pub target_type: String,
    pub target_id: String,
    pub link_type: TraceLinkType,
    pub created_at: String,
    pub created_by: String,
    pub verified: bool,
    pub verified_at: Option<String>,
    pub verified_by: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TraceabilityPath {
    pub entity_id: String,
    pub entity_type: String,
    pub path: Vec<TraceabilityPathNode>,
    pub depth: usize,
}

#[derive(Debug, Clone)]
pub struct TraceabilityPathNode {
    pub entity_id: String,
    pub entity_type: String,
    pub link_type: TraceLinkType,
    pub depth: usize,
    pub children: Vec<TraceabilityPathNode>,
}

#[derive(Debug, Clone)]
pub struct OrphanedItem {
    pub entity_id: String,
    pub entity_type: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub enum TraceLinkType {
    DerivedFrom,    // Target derives from source
    Implements,     // Target implements source
    Verifies,       // Target verifies source
    DependsOn,      // Target depends on source
    Conflicts,      // Target conflicts with source
    Duplicates,     // Target duplicates source
    Related,        // General relationship
}

impl TraceLinkType {
    pub fn from_str(s: &str) -> Result<TraceLinkType, QmsError> {
        match s.to_lowercase().as_str() {
            "derivedfrom" | "derived_from" | "derives" => Ok(TraceLinkType::DerivedFrom),
            "implements" | "implementation" => Ok(TraceLinkType::Implements),
            "verifies" | "verification" | "tests" => Ok(TraceLinkType::Verifies),
            "dependson" | "depends_on" | "depends" => Ok(TraceLinkType::DependsOn),
            "conflicts" | "conflict" => Ok(TraceLinkType::Conflicts),
            "duplicates" | "duplicate" => Ok(TraceLinkType::Duplicates),
            "related" | "relation" => Ok(TraceLinkType::Related),
            _ => Err(QmsError::validation_error(&format!("Invalid link type: {s}"))),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            TraceLinkType::DerivedFrom => "DerivedFrom".to_string(),
            TraceLinkType::Implements => "Implements".to_string(),
            TraceLinkType::Verifies => "Verifies".to_string(),
            TraceLinkType::DependsOn => "DependsOn".to_string(),
            TraceLinkType::Conflicts => "Conflicts".to_string(),
            TraceLinkType::Duplicates => "Duplicates".to_string(),
            TraceLinkType::Related => "Related".to_string(),
        }
    }
}

impl std::fmt::Display for TraceLinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceLinkType::DerivedFrom => write!(f, "DerivedFrom"),
            TraceLinkType::Implements => write!(f, "Implements"),
            TraceLinkType::Verifies => write!(f, "Verifies"),
            TraceLinkType::DependsOn => write!(f, "DependsOn"),
            TraceLinkType::Conflicts => write!(f, "Conflicts"),
            TraceLinkType::Duplicates => write!(f, "Duplicates"),
            TraceLinkType::Related => write!(f, "Related"),
        }
    }
}

impl TraceabilityLink {
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"id":"{}","source_type":"{}","source_id":"{}","target_type":"{}","target_id":"{}","link_type":"{}","created_at":"{}","created_by":"{}","verified":{},"verified_at":{},"verified_by":{}}}"#,
            self.id.replace('"', "\\\""),
            self.source_type.replace('"', "\\\""),
            self.source_id.replace('"', "\\\""),
            self.target_type.replace('"', "\\\""),
            self.target_id.replace('"', "\\\""),
            self.link_type.to_string(),
            self.created_at.replace('"', "\\\""),
            self.created_by.replace('"', "\\\""),
            self.verified,
            match &self.verified_at {
                Some(v) => format!("\"{}\"", v.replace('"', "\\\"")),
                None => "null".to_string(),
            },
            match &self.verified_by {
                Some(v) => format!("\"{}\"", v.replace('"', "\\\"")),
                None => "null".to_string(),
            }
        )
    }

    pub fn from_json(json: &str) -> Result<TraceabilityLink, QmsError> {
        // Manual JSON parsing for TraceabilityLink
        let json = json.trim();
        if !json.starts_with('{') || !json.ends_with('}') {
            return Err(QmsError::parse_error("Invalid JSON format"));
        }

        let mut id = String::new();
        let mut source_type = String::new();
        let mut source_id = String::new();
        let mut target_type = String::new();
        let mut target_id = String::new();
        let mut link_type_str = String::new();
        let mut created_at = String::new();
        let mut created_by = String::new();
        let mut verified = false;
        let mut verified_at = None;
        let mut verified_by = None;

        // Parse JSON fields
        let content = &json[1..json.len()-1]; // Remove braces
        let fields: Vec<&str> = content.split(',').collect();
        
        for field in fields {
            let field = field.trim();
            if let Some(colon_pos) = field.find(':') {
                let key = field[..colon_pos].trim().trim_matches('"');
                let value = field[colon_pos+1..].trim();
                
                match key {
                    "id" => id = value.trim_matches('"').to_string(),
                    "source_type" => source_type = value.trim_matches('"').to_string(),
                    "source_id" => source_id = value.trim_matches('"').to_string(),
                    "target_type" => target_type = value.trim_matches('"').to_string(),
                    "target_id" => target_id = value.trim_matches('"').to_string(),
                    "link_type" => link_type_str = value.trim_matches('"').to_string(),
                    "created_at" => created_at = value.trim_matches('"').to_string(),
                    "created_by" => created_by = value.trim_matches('"').to_string(),
                    "verified" => verified = value == "true",
                    "verified_at" => {
                        if value != "null" {
                            verified_at = Some(value.trim_matches('"').to_string());
                        }
                    },
                    "verified_by" => {
                        if value != "null" {
                            verified_by = Some(value.trim_matches('"').to_string());
                        }
                    },
                    _ => {}
                }
            }
        }

        let link_type = TraceLinkType::from_str(&link_type_str)?;

        Ok(TraceabilityLink {
            id,
            source_type,
            source_id,
            target_type,
            target_id,
            link_type,
            created_at,
            created_by,
            verified,
            verified_at,
            verified_by,
        })
    }
}

pub struct TraceabilityManager {
    project_root: PathBuf,
    links_path: PathBuf,
}

impl TraceabilityManager {
    pub fn new(project_root: &Path) -> QmsResult<Self> {
        let trace_dir = project_root.join("trace");
        let links_path = trace_dir.join("links.json");
        
        // Ensure trace directory exists
        if !trace_dir.exists() {
            fs::create_dir_all(&trace_dir)?;
        }
        
        // Initialize links file if it doesn't exist or is corrupted
        if !links_path.exists() {
            let empty_links = r#"{"version":"1.0","links":[]}"#;
            fs::write(&links_path, empty_links)?;
        } else {
            // Check if existing file is valid JSON, if not, reinitialize
            if let Ok(content) = fs::read_to_string(&links_path) {
                // Try to parse the JSON to detect corruption
                if !content.trim().starts_with('{') || !content.trim().ends_with('}') ||
                   !content.contains("\"version\"") || !content.contains("\"links\"") {
                    // File appears corrupted, reinitialize
                    let empty_links = r#"{"version":"1.0","links":[]}"#;
                    fs::write(&links_path, empty_links)?;
                }
            }
        }
        
        Ok(TraceabilityManager {
            project_root: project_root.to_path_buf(),
            links_path,
        })
    }

    pub fn create_trace_link(
        &self,
        source_id: &str,
        target_id: &str,
        link_type: TraceLinkType,
    ) -> QmsResult<TraceabilityLink> {
        // Validate entities exist
        self.validate_entities_exist(source_id, target_id)?;
        
        // Prevent circular dependencies
        self.prevent_circular_dependencies(source_id, target_id, &link_type)?;
        
        // Check for duplicate links
        self.check_duplicate_links(source_id, target_id, &link_type)?;
        
        let link = TraceabilityLink {
            id: generate_uuid(),
            source_type: self.get_entity_type(source_id)?,
            source_id: source_id.to_string(),
            target_type: self.get_entity_type(target_id)?,
            target_id: target_id.to_string(),
            link_type,
            created_at: current_timestamp().to_string(),
            created_by: "system".to_string(), // TODO: Get from session in Phase 4
            verified: false,
            verified_at: None,
            verified_by: None,
        };
        
        // Save link
        self.save_trace_link(&link)?;
        
        // Log audit entry
        let _ = audit_log_create("TraceabilityLink", &link.id, &link.to_json());
        
        Ok(link)
    }

    pub fn get_trace_links(&self) -> QmsResult<Vec<TraceabilityLink>> {
        let content = fs::read_to_string(&self.links_path)?;
        let links_data = self.parse_links_json(&content)?;
        Ok(links_data)
    }

    pub fn get_links_for_entity(&self, entity_id: &str) -> QmsResult<Vec<TraceabilityLink>> {
        let all_links = self.get_trace_links()?;
        let filtered_links: Vec<TraceabilityLink> = all_links
            .into_iter()
            .filter(|link| link.source_id == entity_id || link.target_id == entity_id)
            .collect();
        Ok(filtered_links)
    }

    pub fn delete_trace_link(&self, link_id: &str) -> QmsResult<()> {
        let mut links = self.get_trace_links()?;
        let initial_count = links.len();
        links.retain(|link| link.id != link_id);
        
        if links.len() == initial_count {
            return Err(QmsError::not_found(&format!("Link with ID {link_id} not found")));
        }
        
        self.save_all_links(&links)?;
        Ok(())
    }

    fn validate_entities_exist(&self, source_id: &str, target_id: &str) -> QmsResult<()> {
        // Check if source entity exists
        if !self.entity_exists(source_id)? {
            return Err(QmsError::not_found(&format!("Source entity {source_id} not found")));
        }
        
        // Check if target entity exists
        if !self.entity_exists(target_id)? {
            return Err(QmsError::not_found(&format!("Target entity {target_id} not found")));
        }
        
        Ok(())
    }

    fn entity_exists(&self, entity_id: &str) -> QmsResult<bool> {
        // Check different entity types based on ID prefix
        if entity_id.starts_with("REQ-") {
            // Check requirements - handle both with and without spaces in JSON
            let req_path = self.project_root.join("trace").join("requirements.json");
            if req_path.exists() {
                let content = fs::read_to_string(&req_path)?;
                return Ok(content.contains(&format!("\"req_id\": \"{entity_id}\"")) ||
                         content.contains(&format!("\"req_id\":\"{entity_id}\"")));
            }
        } else if entity_id.starts_with("TC-") {
            // Check test cases - handle both with and without spaces in JSON
            let test_path = self.project_root.join("trace").join("testcases.json");
            if test_path.exists() {
                let content = fs::read_to_string(&test_path)?;
                return Ok(content.contains(&format!("\"test_id\": \"{entity_id}\"")) ||
                         content.contains(&format!("\"test_id\":\"{entity_id}\"")));
            }
        } else if entity_id.starts_with("RISK-") {
            // Check risks - handle both with and without spaces in JSON
            let risk_path = self.project_root.join("risks").join("risks.json");
            if risk_path.exists() {
                let content = fs::read_to_string(&risk_path)?;
                return Ok(content.contains(&format!("\"id\": \"{entity_id}\"")) ||
                         content.contains(&format!("\"id\":\"{entity_id}\"")));
            }
        } else if entity_id.starts_with("DOC-") {
            // Check documents - handle both with and without spaces in JSON
            let doc_path = self.project_root.join("documents").join("documents.json");
            if doc_path.exists() {
                let content = fs::read_to_string(&doc_path)?;
                return Ok(content.contains(&format!("\"id\": \"{entity_id}\"")) ||
                         content.contains(&format!("\"id\":\"{entity_id}\"")));
            }
        }

        Ok(false)
    }

    fn get_entity_type(&self, entity_id: &str) -> QmsResult<String> {
        if entity_id.starts_with("REQ-") {
            Ok("Requirement".to_string())
        } else if entity_id.starts_with("TC-") {
            Ok("TestCase".to_string())
        } else if entity_id.starts_with("RISK-") {
            Ok("Risk".to_string())
        } else if entity_id.starts_with("DOC-") {
            Ok("Document".to_string())
        } else {
            Err(QmsError::validation_error(&format!("Unknown entity type for ID: {entity_id}")))
        }
    }

    fn prevent_circular_dependencies(
        &self,
        source_id: &str,
        target_id: &str,
        link_type: &TraceLinkType,
    ) -> QmsResult<()> {
        // Get existing links
        let existing_links = self.get_trace_links()?;
        
        // Check if creating this link would create a circular dependency
        let mut visited = std::collections::HashSet::new();
        if self.would_create_cycle(&existing_links, target_id, source_id, &mut visited) {
            return Err(QmsError::validation_error(&format!(
                "Creating link from {source_id} to {target_id} would create a circular dependency"
            )));
        }
        
        // Additional validation for conflicting link types
        if let TraceLinkType::Conflicts = link_type {
            // Conflicting links should not create verification chains
            if self.has_verification_chain(&existing_links, source_id, target_id) {
                return Err(QmsError::validation_error(
                    "Cannot create conflict link between entities with verification relationship"
                ));
            }
        }
        
        Ok(())
    }

    fn would_create_cycle(
        &self,
        links: &[TraceabilityLink],
        current: &str,
        target: &str,
        visited: &mut std::collections::HashSet<String>,
    ) -> bool {
        if current == target {
            return true;
        }
        
        if visited.contains(current) {
            return false;
        }
        
        visited.insert(current.to_string());
        
        // Find all entities that current depends on
        for link in links {
            if link.source_id == current {
                match link.link_type {
                    TraceLinkType::DependsOn | TraceLinkType::DerivedFrom => {
                        if self.would_create_cycle(links, &link.target_id, target, visited) {
                            return true;
                        }
                    },
                    _ => {}
                }
            }
        }
        
        false
    }

    fn has_verification_chain(&self, links: &[TraceabilityLink], source: &str, target: &str) -> bool {
        // Check if there's a verification relationship between source and target
        for link in links {
            if (link.source_id == source && link.target_id == target) ||
               (link.source_id == target && link.target_id == source) {
                match link.link_type {
                    TraceLinkType::Verifies | TraceLinkType::Implements => {
                        return true;
                    },
                    _ => {}
                }
            }
        }
        false
    }

    fn check_duplicate_links(
        &self,
        source_id: &str,
        target_id: &str,
        link_type: &TraceLinkType,
    ) -> QmsResult<()> {
        let existing_links = self.get_trace_links()?;
        
        for link in existing_links {
            if link.source_id == source_id && 
               link.target_id == target_id && 
               matches!((&link.link_type, link_type), 
                       (TraceLinkType::DerivedFrom, TraceLinkType::DerivedFrom) |
                       (TraceLinkType::Implements, TraceLinkType::Implements) |
                       (TraceLinkType::Verifies, TraceLinkType::Verifies) |
                       (TraceLinkType::DependsOn, TraceLinkType::DependsOn) |
                       (TraceLinkType::Conflicts, TraceLinkType::Conflicts) |
                       (TraceLinkType::Duplicates, TraceLinkType::Duplicates) |
                       (TraceLinkType::Related, TraceLinkType::Related)) {
                return Err(QmsError::validation_error(&format!(
                    "Duplicate link already exists from {} to {} with type {}",
                    source_id, target_id, link_type.to_string()
                )));
            }
        }
        
        Ok(())
    }

    fn save_trace_link(&self, link: &TraceabilityLink) -> QmsResult<()> {
        let mut links = self.get_trace_links()?;
        links.push(link.clone());
        self.save_all_links(&links)
    }

    fn save_all_links(&self, links: &[TraceabilityLink]) -> QmsResult<()> {
        let mut json_content = String::new();
        json_content.push_str(r#"{"version":"1.0","links":["#);
        
        for (i, link) in links.iter().enumerate() {
            if i > 0 {
                json_content.push(',');
            }
            json_content.push_str(&link.to_json());
        }
        
        json_content.push_str("]}");
        
        fs::write(&self.links_path, json_content)?;
        Ok(())
    }

    fn parse_links_json(&self, json_content: &str) -> QmsResult<Vec<TraceabilityLink>> {
        let json = json_content.trim();
        if !json.starts_with('{') || !json.ends_with('}') {
            return Err(QmsError::parse_error("Invalid JSON format"));
        }

        // Find the links array
        let links_start = json.find(r#""links":["#);
        if links_start.is_none() {
            return Ok(Vec::new());
        }

        let links_start = links_start.unwrap() + 8; // Skip `"links":[`
        let links_end = json.rfind("]");
        if links_end.is_none() {
            return Err(QmsError::parse_error("Invalid links array format"));
        }

        let links_content = &json[links_start..links_end.unwrap()];
        if links_content.trim().is_empty() {
            return Ok(Vec::new());
        }



        let mut links = Vec::new();
        let mut brace_count = 0;
        let mut current_link = String::new();
        let mut in_string = false;
        let mut escape_next = false;

        for ch in links_content.chars() {
            if escape_next {
                current_link.push(ch);
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => {
                    current_link.push(ch);
                    escape_next = true;
                },
                '"' => {
                    in_string = !in_string;
                    current_link.push(ch);
                },
                '{' if !in_string => {
                    brace_count += 1;
                    current_link.push(ch);
                },
                '}' if !in_string => {
                    brace_count -= 1;
                    current_link.push(ch);
                    if brace_count == 0 {
                        let link = TraceabilityLink::from_json(&current_link)?;
                        links.push(link);
                        current_link.clear();
                    }
                },
                ',' if !in_string && brace_count == 0 => {
                    // Skip comma between links
                },
                '[' | ']' if !in_string && brace_count == 0 => {
                    // Skip array brackets when not inside an object
                },
                ' ' | '\n' | '\t' if !in_string && brace_count == 0 => {
                    // Skip whitespace when not inside an object
                },
                _ if brace_count > 0 => {
                    // Only add characters when we're inside an object
                    current_link.push(ch);
                },
                _ => {
                    // Skip other characters when not inside an object
                },
            }
        }

        Ok(links)
    }

    /// Trace forward from a source entity to all connected targets
    pub fn trace_forward(&self, entity_id: &str) -> QmsResult<TraceabilityPath> {
        let entity_type = self.get_entity_type(entity_id)?;
        let links = self.get_trace_links()?;
        
        let mut path = TraceabilityPath {
            entity_id: entity_id.to_string(),
            entity_type: entity_type.clone(),
            path: Vec::new(),
            depth: 0,
        };

        let mut visited = HashSet::new();
        self.build_forward_path(&links, entity_id, &mut path.path, &mut visited, 0)?;
        
        // Update depth based on deepest node
        path.depth = self.calculate_max_depth(&path.path);
        
        Ok(path)
    }

    /// Trace backward from a target entity to all connected sources
    pub fn trace_backward(&self, entity_id: &str) -> QmsResult<TraceabilityPath> {
        let entity_type = self.get_entity_type(entity_id)?;
        let links = self.get_trace_links()?;
        
        let mut path = TraceabilityPath {
            entity_id: entity_id.to_string(),
            entity_type: entity_type.clone(),
            path: Vec::new(),
            depth: 0,
        };

        let mut visited = HashSet::new();
        self.build_backward_path(&links, entity_id, &mut path.path, &mut visited, 0)?;
        
        // Update depth based on deepest node
        path.depth = self.calculate_max_depth(&path.path);
        
        Ok(path)
    }

    /// Find all orphaned items (entities with no links)
    pub fn find_orphaned_items(&self) -> QmsResult<Vec<OrphanedItem>> {
        let mut orphans = Vec::new();
        let links = self.get_trace_links()?;
        
        // Get all linked entity IDs
        let mut linked_entities = HashSet::new();
        for link in &links {
            linked_entities.insert(link.source_id.clone());
            linked_entities.insert(link.target_id.clone());
        }
        
        // Check each entity type for orphans
        orphans.extend(self.find_orphaned_requirements(&linked_entities)?);
        orphans.extend(self.find_orphaned_test_cases(&linked_entities)?);
        orphans.extend(self.find_orphaned_risks(&linked_entities)?);
        orphans.extend(self.find_orphaned_documents(&linked_entities)?);
        
        Ok(orphans)
    }

    // Helper functions for building traceability paths
    fn build_forward_path(
        &self,
        links: &[TraceabilityLink],
        entity_id: &str,
        path: &mut Vec<TraceabilityPathNode>,
        visited: &mut HashSet<String>,
        depth: usize,
    ) -> QmsResult<()> {
        if visited.contains(entity_id) {
            return Ok(()); // Prevent infinite loops
        }
        visited.insert(entity_id.to_string());

        // Find all outgoing links from this entity
        for link in links {
            if link.source_id == entity_id {
                let target_type = self.get_entity_type(&link.target_id)?;
                let mut node = TraceabilityPathNode {
                    entity_id: link.target_id.clone(),
                    entity_type: target_type,
                    link_type: link.link_type.clone(),
                    depth: depth + 1,
                    children: Vec::new(),
                };

                // Recursively build forward path for target
                self.build_forward_path(links, &link.target_id, &mut node.children, visited, depth + 1)?;
                path.push(node);
            }
        }

        visited.remove(entity_id);
        Ok(())
    }

    fn build_backward_path(
        &self,
        links: &[TraceabilityLink],
        entity_id: &str,
        path: &mut Vec<TraceabilityPathNode>,
        visited: &mut HashSet<String>,
        depth: usize,
    ) -> QmsResult<()> {
        if visited.contains(entity_id) {
            return Ok(()); // Prevent infinite loops
        }
        visited.insert(entity_id.to_string());

        // Find all incoming links to this entity
        for link in links {
            if link.target_id == entity_id {
                let source_type = self.get_entity_type(&link.source_id)?;
                let mut node = TraceabilityPathNode {
                    entity_id: link.source_id.clone(),
                    entity_type: source_type,
                    link_type: link.link_type.clone(),
                    depth: depth + 1,
                    children: Vec::new(),
                };

                // Recursively build backward path for source
                self.build_backward_path(links, &link.source_id, &mut node.children, visited, depth + 1)?;
                path.push(node);
            }
        }

        visited.remove(entity_id);
        Ok(())
    }

    fn calculate_max_depth(&self, nodes: &[TraceabilityPathNode]) -> usize {
        let mut max_depth = 0;
        for node in nodes {
            max_depth = max_depth.max(node.depth);
            max_depth = max_depth.max(self.calculate_max_depth(&node.children));
        }
        max_depth
    }

    // Helper functions for finding orphaned items by type
    fn find_orphaned_requirements(&self, linked_entities: &HashSet<String>) -> QmsResult<Vec<OrphanedItem>> {
        let mut orphans = Vec::new();
        let req_path = self.project_root.join("trace").join("requirements.json");
        
        if req_path.exists() {
            let content = fs::read_to_string(&req_path)?;
            if let Ok(requirements) = self.parse_requirements_json(&content) {
                for req in requirements {
                    if !linked_entities.contains(&req) {
                        orphans.push(OrphanedItem {
                            entity_id: req.clone(),
                            entity_type: "Requirement".to_string(),
                            reason: "No traceability links found".to_string(),
                        });
                    }
                }
            }
        }
        
        Ok(orphans)
    }

    fn find_orphaned_test_cases(&self, linked_entities: &HashSet<String>) -> QmsResult<Vec<OrphanedItem>> {
        let mut orphans = Vec::new();
        let test_path = self.project_root.join("trace").join("testcases.json");
        
        if test_path.exists() {
            let content = fs::read_to_string(&test_path)?;
            if let Ok(test_cases) = self.parse_test_cases_json(&content) {
                for test_case in test_cases {
                    if !linked_entities.contains(&test_case) {
                        orphans.push(OrphanedItem {
                            entity_id: test_case.clone(),
                            entity_type: "TestCase".to_string(),
                            reason: "No traceability links found".to_string(),
                        });
                    }
                }
            }
        }
        
        Ok(orphans)
    }

    fn find_orphaned_risks(&self, linked_entities: &HashSet<String>) -> QmsResult<Vec<OrphanedItem>> {
        let mut orphans = Vec::new();
        let risk_path = self.project_root.join("risks").join("risks.json");
        
        if risk_path.exists() {
            let content = fs::read_to_string(&risk_path)?;
            if let Ok(risks) = self.parse_risks_json(&content) {
                for risk in risks {
                    if !linked_entities.contains(&risk) {
                        orphans.push(OrphanedItem {
                            entity_id: risk.clone(),
                            entity_type: "Risk".to_string(),
                            reason: "No traceability links found".to_string(),
                        });
                    }
                }
            }
        }
        
        Ok(orphans)
    }

    fn find_orphaned_documents(&self, linked_entities: &HashSet<String>) -> QmsResult<Vec<OrphanedItem>> {
        let mut orphans = Vec::new();
        let doc_path = self.project_root.join("documents").join("documents.json");
        
        if doc_path.exists() {
            let content = fs::read_to_string(&doc_path)?;
            if let Ok(documents) = self.parse_documents_json(&content) {
                for document in documents {
                    if !linked_entities.contains(&document) {
                        orphans.push(OrphanedItem {
                            entity_id: document.clone(),
                            entity_type: "Document".to_string(),
                            reason: "No traceability links found".to_string(),
                        });
                    }
                }
            }
        }
        
        Ok(orphans)
    }

    // JSON parsing helpers for orphan detection
    fn parse_requirements_json(&self, content: &str) -> Result<Vec<String>, QmsError> {
        let mut req_ids = Vec::new();

        // Simple JSON parsing for requirement IDs using string search (handle both with and without spaces)
        if content.contains("\"req_id\":") || content.contains("\"req_id\": ") {
            // Try both patterns: with and without space after colon
            for pattern in &["\"req_id\":\"", "\"req_id\": \""] {
                let mut parts = content.split(pattern);
                parts.next(); // Skip first part

                for part in parts {
                    if let Some(end_pos) = part.find('"') {
                        let id = &part[..end_pos];
                        if id.starts_with("REQ-") && !req_ids.contains(&id.to_string()) {
                            req_ids.push(id.to_string());
                        }
                    }
                }
            }
        }

        Ok(req_ids)
    }

    fn parse_test_cases_json(&self, content: &str) -> Result<Vec<String>, QmsError> {
        let mut test_ids = Vec::new();

        // Simple JSON parsing for test case IDs (handle both with and without spaces)
        if content.contains("\"test_id\":") || content.contains("\"test_id\": ") {
            // Try both patterns: with and without space after colon
            for pattern in &["\"test_id\":\"", "\"test_id\": \""] {
                let mut parts = content.split(pattern);
                parts.next(); // Skip first part

                for part in parts {
                    if let Some(end_pos) = part.find('"') {
                        let id = &part[..end_pos];
                        if id.starts_with("TC-") && !test_ids.contains(&id.to_string()) {
                            test_ids.push(id.to_string());
                        }
                    }
                }
            }
        }

        Ok(test_ids)
    }

    fn parse_risks_json(&self, content: &str) -> Result<Vec<String>, QmsError> {
        let mut risk_ids = Vec::new();
        
        // Simple JSON parsing for risk IDs
        if content.contains("\"id\":") {
            let mut parts = content.split("\"id\":\"");
            parts.next(); // Skip first part
            
            for part in parts {
                if let Some(end_pos) = part.find('"') {
                    let id = &part[..end_pos];
                    if id.starts_with("RISK-") {
                        risk_ids.push(id.to_string());
                    }
                }
            }
        }
        
        Ok(risk_ids)
    }

    fn parse_documents_json(&self, content: &str) -> Result<Vec<String>, QmsError> {
        let mut doc_ids = Vec::new();
        
        // Simple JSON parsing for document IDs
        if content.contains("\"id\":") {
            let mut parts = content.split("\"id\":\"");
            parts.next(); // Skip first part
            
            for part in parts {
                if let Some(end_pos) = part.find('"') {
                    let id = &part[..end_pos];
                    if id.starts_with("DOC-") {
                        doc_ids.push(id.to_string());
                    }
                }
            }
        }
        
        Ok(doc_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_trace_link_type_parsing() {
        assert!(matches!(TraceLinkType::from_str("verifies"), Ok(TraceLinkType::Verifies)));
        assert!(matches!(TraceLinkType::from_str("implements"), Ok(TraceLinkType::Implements)));
        assert!(matches!(TraceLinkType::from_str("depends_on"), Ok(TraceLinkType::DependsOn)));
        assert!(TraceLinkType::from_str("invalid").is_err());
    }

    #[test]
    fn test_trace_link_json_serialization() {
        let link = TraceabilityLink {
            id: "test-link-1".to_string(),
            source_type: "Requirement".to_string(),
            source_id: "REQ-001".to_string(),
            target_type: "TestCase".to_string(),
            target_id: "TC-001".to_string(),
            link_type: TraceLinkType::Verifies,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            created_by: "test_user".to_string(),
            verified: false,
            verified_at: None,
            verified_by: None,
        };

        let json = link.to_json();
        let parsed_link = TraceabilityLink::from_json(&json).unwrap();
        
        assert_eq!(parsed_link.id, link.id);
        assert_eq!(parsed_link.source_id, link.source_id);
        assert_eq!(parsed_link.target_id, link.target_id);
        assert!(matches!(parsed_link.link_type, TraceLinkType::Verifies));
    }

    #[test]
    fn test_traceability_manager_creation() {
        let temp_dir = env::temp_dir().join("qms_test_trace");
        let _manager = TraceabilityManager::new(&temp_dir).unwrap();
        
        assert!(temp_dir.join("trace").exists());
        assert!(temp_dir.join("trace").join("links.json").exists());
    }

    #[test]
    fn test_entity_type_detection() {
        let temp_dir = env::temp_dir().join("qms_test_entity");
        let manager = TraceabilityManager::new(&temp_dir).unwrap();
        
        assert_eq!(manager.get_entity_type("REQ-001").unwrap(), "Requirement");
        assert_eq!(manager.get_entity_type("TC-001").unwrap(), "TestCase");
        assert_eq!(manager.get_entity_type("RISK-001").unwrap(), "Risk");
        assert_eq!(manager.get_entity_type("DOC-001").unwrap(), "Document");
        assert!(manager.get_entity_type("INVALID-001").is_err());
    }

    #[test]
    fn test_circular_dependency_detection() {
        let temp_dir = env::temp_dir().join("qms_test_circular");
        let manager = TraceabilityManager::new(&temp_dir).unwrap();
        
        let links = vec![
            TraceabilityLink {
                id: "link1".to_string(),
                source_type: "Requirement".to_string(),
                source_id: "REQ-001".to_string(),
                target_type: "Requirement".to_string(),
                target_id: "REQ-002".to_string(),
                link_type: TraceLinkType::DependsOn,
                created_at: "2025-01-01T00:00:00Z".to_string(),
                created_by: "test".to_string(),
                verified: false,
                verified_at: None,
                verified_by: None,
            },
        ];
        
        let mut visited = std::collections::HashSet::new();
        // This should detect that REQ-002 -> REQ-001 would create a cycle
        // We check if there's a path from target (REQ-001) to source (REQ-002), which there is
        assert!(manager.would_create_cycle(&links, "REQ-001", "REQ-002", &mut visited));
    }

    #[test]
    fn test_link_type_string_conversion() {
        let verifies = TraceLinkType::Verifies;
        assert_eq!(verifies.to_string(), "Verifies");
        
        let implements = TraceLinkType::Implements;
        assert_eq!(implements.to_string(), "Implements");
        
        let depends_on = TraceLinkType::DependsOn;
        assert_eq!(depends_on.to_string(), "DependsOn");
    }

    #[test]
    fn test_trace_forward_basic() {
        let temp_dir = env::temp_dir().join("qms_test_forward");
        let manager = TraceabilityManager::new(&temp_dir).unwrap();
        
        // Create test links
        let link1 = TraceabilityLink {
            id: "link1".to_string(),
            source_type: "Requirement".to_string(),
            source_id: "REQ-001".to_string(),
            target_type: "TestCase".to_string(),
            target_id: "TC-001".to_string(),
            link_type: TraceLinkType::Verifies,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            created_by: "test".to_string(),
            verified: false,
            verified_at: None,
            verified_by: None,
        };
        
        let link2 = TraceabilityLink {
            id: "link2".to_string(),
            source_type: "Requirement".to_string(),
            source_id: "REQ-001".to_string(),
            target_type: "TestCase".to_string(),
            target_id: "TC-002".to_string(),
            link_type: TraceLinkType::Verifies,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            created_by: "test".to_string(),
            verified: false,
            verified_at: None,
            verified_by: None,
        };
        
        manager.save_all_links(&[link1, link2]).unwrap();
        
        // Test forward tracing
        let path = manager.trace_forward("REQ-001").unwrap();
        assert_eq!(path.entity_id, "REQ-001");
        assert_eq!(path.entity_type, "Requirement");
        assert_eq!(path.path.len(), 2);
        assert!(path.path.iter().any(|node| node.entity_id == "TC-001"));
        assert!(path.path.iter().any(|node| node.entity_id == "TC-002"));
    }

    #[test]
    fn test_trace_backward_basic() {
        let temp_dir = env::temp_dir().join("qms_test_backward");
        let manager = TraceabilityManager::new(&temp_dir).unwrap();
        
        // Create test links
        let link1 = TraceabilityLink {
            id: "link1".to_string(),
            source_type: "Requirement".to_string(),
            source_id: "REQ-001".to_string(),
            target_type: "TestCase".to_string(),
            target_id: "TC-001".to_string(),
            link_type: TraceLinkType::Verifies,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            created_by: "test".to_string(),
            verified: false,
            verified_at: None,
            verified_by: None,
        };
        
        let link2 = TraceabilityLink {
            id: "link2".to_string(),
            source_type: "Requirement".to_string(),
            source_id: "REQ-002".to_string(),
            target_type: "TestCase".to_string(),
            target_id: "TC-001".to_string(),
            link_type: TraceLinkType::Verifies,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            created_by: "test".to_string(),
            verified: false,
            verified_at: None,
            verified_by: None,
        };
        
        manager.save_all_links(&[link1, link2]).unwrap();
        
        // Test backward tracing
        let path = manager.trace_backward("TC-001").unwrap();
        assert_eq!(path.entity_id, "TC-001");
        assert_eq!(path.entity_type, "TestCase");
        assert_eq!(path.path.len(), 2);
        assert!(path.path.iter().any(|node| node.entity_id == "REQ-001"));
        assert!(path.path.iter().any(|node| node.entity_id == "REQ-002"));
    }

    #[test]
    fn test_trace_circular_dependency_prevention() {
        let temp_dir = env::temp_dir().join("qms_test_circular_trace");
        let manager = TraceabilityManager::new(&temp_dir).unwrap();
        
        // Create circular links
        let link1 = TraceabilityLink {
            id: "link1".to_string(),
            source_type: "Requirement".to_string(),
            source_id: "REQ-001".to_string(),
            target_type: "Requirement".to_string(),
            target_id: "REQ-002".to_string(),
            link_type: TraceLinkType::DependsOn,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            created_by: "test".to_string(),
            verified: false,
            verified_at: None,
            verified_by: None,
        };
        
        let link2 = TraceabilityLink {
            id: "link2".to_string(),
            source_type: "Requirement".to_string(),
            source_id: "REQ-002".to_string(),
            target_type: "Requirement".to_string(),
            target_id: "REQ-001".to_string(),
            link_type: TraceLinkType::DependsOn,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            created_by: "test".to_string(),
            verified: false,
            verified_at: None,
            verified_by: None,
        };
        
        manager.save_all_links(&[link1, link2]).unwrap();
        
        // Test forward/backward tracing handles circular dependencies
        let forward_path = manager.trace_forward("REQ-001").unwrap();
        let backward_path = manager.trace_backward("REQ-001").unwrap();
        
        // Should not get stuck in infinite loop
        assert!(forward_path.depth < 10);
        assert!(backward_path.depth < 10);
    }

    #[test]
    fn test_orphaned_items_detection() {
        let temp_dir = env::temp_dir().join("qms_test_orphans");
        let manager = TraceabilityManager::new(&temp_dir).unwrap();
        
        // Create requirements file with orphaned items
        let req_dir = temp_dir.join("trace");
        std::fs::create_dir_all(&req_dir).unwrap();
        let req_file = req_dir.join("requirements.json");
        std::fs::write(&req_file, r#"{"version":"1.0","requirements":[{"req_id":"REQ-001","title":"Test Req 1"},{"req_id":"REQ-002","title":"Test Req 2"}]}"#).unwrap();
        
        // Create link for only REQ-001
        let link = TraceabilityLink {
            id: "link1".to_string(),
            source_type: "Requirement".to_string(),
            source_id: "REQ-001".to_string(),
            target_type: "TestCase".to_string(),
            target_id: "TC-001".to_string(),
            link_type: TraceLinkType::Verifies,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            created_by: "test".to_string(),
            verified: false,
            verified_at: None,
            verified_by: None,
        };
        
        manager.save_all_links(&[link]).unwrap();
        
        // Test orphan detection
        let orphans = manager.find_orphaned_items().unwrap();
        
        // REQ-002 should be orphaned
        assert!(orphans.iter().any(|o| o.entity_id == "REQ-002"));
        assert!(orphans.iter().any(|o| o.entity_type == "Requirement"));
    }

    #[test]
    fn test_multi_level_traceability() {
        let temp_dir = env::temp_dir().join("qms_test_multilevel");
        let manager = TraceabilityManager::new(&temp_dir).unwrap();
        
        // Create multi-level link chain: REQ-001 -> TC-001 -> TC-002
        let link1 = TraceabilityLink {
            id: "link1".to_string(),
            source_type: "Requirement".to_string(),
            source_id: "REQ-001".to_string(),
            target_type: "TestCase".to_string(),
            target_id: "TC-001".to_string(),
            link_type: TraceLinkType::Verifies,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            created_by: "test".to_string(),
            verified: false,
            verified_at: None,
            verified_by: None,
        };
        
        let link2 = TraceabilityLink {
            id: "link2".to_string(),
            source_type: "TestCase".to_string(),
            source_id: "TC-001".to_string(),
            target_type: "TestCase".to_string(),
            target_id: "TC-002".to_string(),
            link_type: TraceLinkType::DependsOn,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            created_by: "test".to_string(),
            verified: false,
            verified_at: None,
            verified_by: None,
        };
        
        manager.save_all_links(&[link1, link2]).unwrap();
        
        // Test forward tracing shows multi-level chain
        let path = manager.trace_forward("REQ-001").unwrap();
        assert_eq!(path.entity_id, "REQ-001");
        assert!(path.depth >= 2);
        
        // Should have TC-001 at level 1 and TC-002 at level 2
        assert!(path.path.iter().any(|node| node.entity_id == "TC-001" && node.depth == 1));
        assert!(path.path.iter().any(|node| 
            node.entity_id == "TC-001" && 
            node.children.iter().any(|child| child.entity_id == "TC-002")
        ));
    }
    
    #[test]
    fn test_rtm_generation() {
        let temp_dir = env::temp_dir().join("qms_test_rtm");
        let manager = TraceabilityManager::new(&temp_dir).unwrap();
        
        // Create test links
        let link1 = TraceabilityLink {
            id: "link1".to_string(),
            source_type: "Requirement".to_string(),
            source_id: "REQ-001".to_string(),
            target_type: "TestCase".to_string(),
            target_id: "TC-001".to_string(),
            link_type: TraceLinkType::Verifies,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            created_by: "test".to_string(),
            verified: false,
            verified_at: None,
            verified_by: None,
        };
        
        let link2 = TraceabilityLink {
            id: "link2".to_string(),
            source_type: "Requirement".to_string(),
            source_id: "REQ-002".to_string(),
            target_type: "TestCase".to_string(),
            target_id: "TC-002".to_string(),
            link_type: TraceLinkType::Verifies,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            created_by: "test".to_string(),
            verified: false,
            verified_at: None,
            verified_by: None,
        };
        
        manager.save_all_links(&[link1, link2]).unwrap();
        
        // Test RTM generation
        let rtm = manager.generate_rtm().unwrap();
        assert_eq!(rtm.entities.len(), 4); // REQ-001, REQ-002, TC-001, TC-002
        assert_eq!(rtm.links.len(), 2);
        
        // Verify entities have correct linked entities
        let req_001 = rtm.entities.iter().find(|e| e.entity_id == "REQ-001").unwrap();
        assert!(req_001.linked_entities.contains(&"TC-001".to_string()));
    }
    
    #[test]
    fn test_rtm_csv_export() {
        let temp_dir = env::temp_dir().join("qms_test_rtm_csv");
        let manager = TraceabilityManager::new(&temp_dir).unwrap();
        
        // Create test links
        let link = TraceabilityLink {
            id: "link1".to_string(),
            source_type: "Requirement".to_string(),
            source_id: "REQ-001".to_string(),
            target_type: "TestCase".to_string(),
            target_id: "TC-001".to_string(),
            link_type: TraceLinkType::Verifies,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            created_by: "test".to_string(),
            verified: false,
            verified_at: None,
            verified_by: None,
        };
        
        manager.save_all_links(&[link]).unwrap();
        
        // Test CSV export
        let output_path = temp_dir.join("test_export.csv");
        manager.export_rtm_csv(&output_path).unwrap();
        
        // Verify CSV file was created and contains expected content
        assert!(output_path.exists());
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("Entity ID,Entity Type,Title,Status,Linked Entities"));
        assert!(content.contains("REQ-001"));
        assert!(content.contains("TC-001"));
    }
    
    #[test]
    fn test_csv_import() {
        let temp_dir = env::temp_dir().join("qms_test_csv_import");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Initialize audit system for test
        let audit_dir = temp_dir.join("audit");
        let _ = std::fs::create_dir_all(&audit_dir);
        let config = crate::modules::audit_logger::AuditConfig {
            project_path: temp_dir.to_string_lossy().to_string(),
            retention_days: 30,
            daily_rotation: false,
            max_file_size_mb: 10,
            require_checksums: false,
        };
        let _ = crate::modules::audit_logger::initialize_audit_system(config);

        let manager = TraceabilityManager::new(&temp_dir).unwrap();

        // Create the entities first (requirements and test cases must exist for import to work)
        let mut req_manager = crate::modules::traceability::requirement::RequirementManager::new(&temp_dir).unwrap();
        let mut test_manager = crate::modules::traceability::test_case::TestCaseManager::new(&temp_dir).unwrap();

        // Create requirements
        req_manager.create_requirement(
            "test_project".to_string(),
            "REQ-001".to_string(),
            "Test Requirement 1".to_string(),
            "Test requirement for CSV import".to_string(),
            crate::modules::traceability::requirement::RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();

        req_manager.create_requirement(
            "test_project".to_string(),
            "REQ-002".to_string(),
            "Test Requirement 2".to_string(),
            "Test requirement for CSV import".to_string(),
            crate::modules::traceability::requirement::RequirementCategory::Functional,
            "test_user".to_string(),
        ).unwrap();

        // Create test cases
        test_manager.create_test_case(
            "TC-001".to_string(),
            "Test Case 1".to_string(),
            "Test case for CSV import".to_string(),
            crate::modules::traceability::test_case::TestCategory::Functional,
            crate::modules::traceability::test_case::TestPriority::High,
            "test_user".to_string(),
        ).unwrap();

        test_manager.create_test_case(
            "TC-002".to_string(),
            "Test Case 2".to_string(),
            "Test case for CSV import".to_string(),
            crate::modules::traceability::test_case::TestCategory::Functional,
            crate::modules::traceability::test_case::TestPriority::High,
            "test_user".to_string(),
        ).unwrap();

        // Create test CSV content
        let csv_content = "SourceType,SourceID,TargetType,TargetID,LinkType,CreatedBy\nRequirement,REQ-001,TestCase,TC-001,Verifies,test_user\nRequirement,REQ-002,TestCase,TC-002,Verifies,test_user\n";
        let csv_path = temp_dir.join("test_import.csv");
        std::fs::write(&csv_path, csv_content).unwrap();
        
        // Test CSV import
        let stats = manager.import_from_csv(&csv_path).unwrap();
        assert_eq!(stats.total_processed, 2);
        assert_eq!(stats.successful_imports, 2);
        assert_eq!(stats.failed_imports, 0);
        
        // Verify links were created
        let links = manager.get_trace_links().unwrap();
        assert_eq!(links.len(), 2);
        assert!(links.iter().any(|l| l.source_id == "REQ-001" && l.target_id == "TC-001"));
        assert!(links.iter().any(|l| l.source_id == "REQ-002" && l.target_id == "TC-002"));
    }
}

// Import/Export functionality for Task 3.2.11

#[derive(Debug, Clone)]
pub struct TraceabilityMatrix {
    pub entities: Vec<TraceabilityEntity>,
    pub links: Vec<TraceabilityLink>,
    pub generated_at: String,
    pub generated_by: String,
}

#[derive(Debug, Clone)]
pub struct TraceabilityEntity {
    pub entity_id: String,
    pub entity_type: String,
    pub title: String,
    pub status: String,
    pub linked_entities: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ImportStats {
    pub total_processed: usize,
    pub successful_imports: usize,
    pub failed_imports: usize,
    pub duplicates_found: usize,
    pub validation_errors: Vec<String>,
}

impl TraceabilityManager {
    // Generate Requirements Traceability Matrix (RTM)
    pub fn generate_rtm(&self) -> QmsResult<TraceabilityMatrix> {
        let links = self.get_trace_links()?;
        let mut entities = Vec::new();
        let mut entity_map = std::collections::HashMap::new();
        
        // Collect all unique entities
        for link in &links {
            entity_map.insert(link.source_id.clone(), link.source_type.clone());
            entity_map.insert(link.target_id.clone(), link.target_type.clone());
        }
        
        // Build entity information with linked entities
        for (entity_id, entity_type) in entity_map {
            let mut linked_entities = Vec::new();

            // Find all entities linked to this one
            for link in &links {
                if link.source_id == entity_id {
                    linked_entities.push(link.target_id.clone());
                }
                if link.target_id == entity_id {
                    linked_entities.push(link.source_id.clone());
                }
            }

            // Get actual title from the entity data
            let title = self.get_entity_title(&entity_id, &entity_type).unwrap_or_else(|_| format!("Entity {entity_id}"));
            let status = self.get_entity_status(&entity_id, &entity_type).unwrap_or_else(|_| "Active".to_string());

            entities.push(TraceabilityEntity {
                entity_id: entity_id.clone(),
                entity_type: entity_type.clone(),
                title,
                status,
                linked_entities,
            });
        }
        
        Ok(TraceabilityMatrix {
            entities,
            links,
            generated_at: current_timestamp().to_string(),
            generated_by: "system".to_string(),
        })
    }

    // Helper method to get entity title from the appropriate manager
    fn get_entity_title(&self, entity_id: &str, entity_type: &str) -> QmsResult<String> {
        match entity_type {
            "Requirement" => {
                // Read requirements.json and extract title
                let req_path = self.project_root.join("trace").join("requirements.json");
                if req_path.exists() {
                    let content = fs::read_to_string(&req_path)?;
                    if let Ok(title) = self.extract_requirement_title(&content, entity_id) {
                        return Ok(title);
                    }
                }
            },
            "TestCase" => {
                // Read testcases.json and extract title
                let test_path = self.project_root.join("trace").join("testcases.json");
                if test_path.exists() {
                    let content = fs::read_to_string(&test_path)?;
                    if let Ok(title) = self.extract_test_case_title(&content, entity_id) {
                        return Ok(title);
                    }
                }
            },
            _ => {}
        }
        Ok(format!("Entity {entity_id}"))
    }

    // Helper method to get entity status from the appropriate manager
    fn get_entity_status(&self, entity_id: &str, entity_type: &str) -> QmsResult<String> {
        match entity_type {
            "Requirement" => {
                let req_path = self.project_root.join("trace").join("requirements.json");
                if req_path.exists() {
                    let content = fs::read_to_string(&req_path)?;
                    if let Ok(status) = self.extract_requirement_status(&content, entity_id) {
                        return Ok(status);
                    }
                }
            },
            "TestCase" => {
                let test_path = self.project_root.join("trace").join("testcases.json");
                if test_path.exists() {
                    let content = fs::read_to_string(&test_path)?;
                    if let Ok(status) = self.extract_test_case_status(&content, entity_id) {
                        return Ok(status);
                    }
                }
            },
            _ => {}
        }
        Ok("Active".to_string())
    }

    // Helper methods to extract specific fields from JSON content
    fn extract_requirement_title(&self, content: &str, req_id: &str) -> Result<String, QmsError> {
        // Find the requirement object with matching req_id
        let req_pattern = format!("\"req_id\": \"{req_id}\"");
        if let Some(start_pos) = content.find(&req_pattern) {
            // Look for title field in the same object
            let remaining = &content[start_pos..];
            if let Some(title_start) = remaining.find("\"title\": \"") {
                let title_content = &remaining[title_start + 10..];
                if let Some(title_end) = title_content.find('"') {
                    return Ok(title_content[..title_end].to_string());
                }
            }
        }
        Err(QmsError::not_found("Title not found"))
    }

    fn extract_test_case_title(&self, content: &str, test_id: &str) -> Result<String, QmsError> {
        // Find the test case object with matching test_id
        let test_pattern = format!("\"test_id\": \"{test_id}\"");
        if let Some(start_pos) = content.find(&test_pattern) {
            // Look for title field in the same object
            let remaining = &content[start_pos..];
            if let Some(title_start) = remaining.find("\"title\": \"") {
                let title_content = &remaining[title_start + 10..];
                if let Some(title_end) = title_content.find('"') {
                    return Ok(title_content[..title_end].to_string());
                }
            }
        }
        Err(QmsError::not_found("Title not found"))
    }

    fn extract_requirement_status(&self, content: &str, req_id: &str) -> Result<String, QmsError> {
        let req_pattern = format!("\"req_id\": \"{req_id}\"");
        if let Some(start_pos) = content.find(&req_pattern) {
            let remaining = &content[start_pos..];
            if let Some(status_start) = remaining.find("\"status\": \"") {
                let status_content = &remaining[status_start + 11..];
                if let Some(status_end) = status_content.find('"') {
                    return Ok(status_content[..status_end].to_string());
                }
            }
        }
        Ok("Draft".to_string())
    }

    fn extract_test_case_status(&self, content: &str, test_id: &str) -> Result<String, QmsError> {
        let test_pattern = format!("\"test_id\": \"{test_id}\"");
        if let Some(start_pos) = content.find(&test_pattern) {
            let remaining = &content[start_pos..];
            if let Some(status_start) = remaining.find("\"status\": \"") {
                let status_content = &remaining[status_start + 11..];
                if let Some(status_end) = status_content.find('"') {
                    return Ok(status_content[..status_end].to_string());
                }
            }
        }
        Ok("Active".to_string())
    }

    // Export RTM to CSV format
    pub fn export_rtm_csv(&self, output_path: &Path) -> QmsResult<()> {
        let matrix = self.generate_rtm()?;
        let mut csv_content = String::new();
        
        // CSV Header
        csv_content.push_str("Entity ID,Entity Type,Title,Status,Linked Entities\n");
        
        // CSV Rows
        for entity in &matrix.entities {
            let linked_entities = entity.linked_entities.join("; ");
            csv_content.push_str(&format!(
                "{},{},{},{},{}\n",
                entity.entity_id,
                entity.entity_type,
                entity.title,
                entity.status,
                linked_entities
            ));
        }
        
        fs::write(output_path, csv_content)?;
        
        // Log export activity
        audit_log_create("export", "rtm_csv", &format!("RTM exported to CSV: {}", output_path.display()))?;
        
        Ok(())
    }
    
    // Export RTM to JSON format
    pub fn export_rtm_json(&self, output_path: &Path) -> QmsResult<()> {
        let matrix = self.generate_rtm()?;
        
        // Simple JSON serialization
        let mut json_content = String::new();
        json_content.push_str("{\n");
        json_content.push_str(&format!("  \"generated_at\": \"{}\",\n", matrix.generated_at));
        json_content.push_str(&format!("  \"generated_by\": \"{}\",\n", matrix.generated_by));
        json_content.push_str("  \"entities\": [\n");
        
        for (i, entity) in matrix.entities.iter().enumerate() {
            json_content.push_str("    {\n");
            json_content.push_str(&format!("      \"entity_id\": \"{}\",\n", entity.entity_id));
            json_content.push_str(&format!("      \"entity_type\": \"{}\",\n", entity.entity_type));
            json_content.push_str(&format!("      \"title\": \"{}\",\n", entity.title));
            json_content.push_str(&format!("      \"status\": \"{}\",\n", entity.status));
            json_content.push_str("      \"linked_entities\": [");
            
            for (j, linked_id) in entity.linked_entities.iter().enumerate() {
                json_content.push_str(&format!("\"{linked_id}\""));
                if j < entity.linked_entities.len() - 1 {
                    json_content.push_str(", ");
                }
            }
            
            json_content.push_str("]\n    }");
            if i < matrix.entities.len() - 1 {
                json_content.push(',');
            }
            json_content.push('\n');
        }
        
        json_content.push_str("  ],\n");
        json_content.push_str("  \"links\": [\n");
        
        for (i, link) in matrix.links.iter().enumerate() {
            json_content.push_str("    {\n");
            json_content.push_str(&format!("      \"id\": \"{}\",\n", link.id));
            json_content.push_str(&format!("      \"source_type\": \"{}\",\n", link.source_type));
            json_content.push_str(&format!("      \"source_id\": \"{}\",\n", link.source_id));
            json_content.push_str(&format!("      \"target_type\": \"{}\",\n", link.target_type));
            json_content.push_str(&format!("      \"target_id\": \"{}\",\n", link.target_id));
            json_content.push_str(&format!("      \"link_type\": \"{}\",\n", link.link_type));
            json_content.push_str(&format!("      \"created_at\": \"{}\",\n", link.created_at));
            json_content.push_str(&format!("      \"created_by\": \"{}\",\n", link.created_by));
            json_content.push_str(&format!("      \"verified\": {}", link.verified));
            json_content.push_str("\n    }");
            if i < matrix.links.len() - 1 {
                json_content.push(',');
            }
            json_content.push('\n');
        }
        
        json_content.push_str("  ]\n");
        json_content.push_str("}\n");
        
        fs::write(output_path, json_content)?;
        
        // Log export activity
        audit_log_create("export", "rtm_json", &format!("RTM exported to JSON: {}", output_path.display()))?;
        
        Ok(())
    }
    
    // Import traceability links from CSV
    pub fn import_from_csv(&self, file_path: &Path) -> QmsResult<ImportStats> {
        let content = fs::read_to_string(file_path)?;
        let lines: Vec<&str> = content.lines().collect();
        
        if lines.is_empty() {
            return Err(QmsError::validation_error("CSV file is empty"));
        }
        
        let mut stats = ImportStats {
            total_processed: 0,
            successful_imports: 0,
            failed_imports: 0,
            duplicates_found: 0,
            validation_errors: Vec::new(),
        };
        
        // Skip header line
        for (line_num, line) in lines.iter().skip(1).enumerate() {
            stats.total_processed += 1;
            let line_number = line_num + 2; // Account for header and 0-based indexing
            
            let fields: Vec<&str> = line.split(',').collect();
            if fields.len() < 6 {
                stats.failed_imports += 1;
                stats.validation_errors.push(format!("Line {line_number}: Invalid CSV format, expected 6 fields"));
                continue;
            }
            
            let source_type = fields[0].trim();
            let source_id = fields[1].trim();
            let target_type = fields[2].trim();
            let target_id = fields[3].trim();
            let link_type_str = fields[4].trim();
            let _created_by = fields[5].trim();
            
            // Validate fields
            if source_type.is_empty() || source_id.is_empty() || target_type.is_empty() || target_id.is_empty() {
                stats.failed_imports += 1;
                stats.validation_errors.push(format!("Line {line_number}: Empty required fields"));
                continue;
            }
            
            // Parse link type
            let link_type = match link_type_str {
                "Verifies" => TraceLinkType::Verifies,
                "Implements" => TraceLinkType::Implements,
                "DependsOn" => TraceLinkType::DependsOn,
                "DerivedFrom" => TraceLinkType::DerivedFrom,
                "Related" => TraceLinkType::Related,
                "Conflicts" => TraceLinkType::Conflicts,
                "Duplicates" => TraceLinkType::Duplicates,
                _ => {
                    stats.failed_imports += 1;
                    stats.validation_errors.push(format!("Line {line_number}: Unknown link type '{link_type_str}'"));
                    continue;
                }
            };
            
            // Check for duplicates
            if self.link_exists(source_id, target_id, &link_type)? {
                stats.duplicates_found += 1;
                continue;
            }
            
            // Create the link
            match self.create_trace_link(source_id, target_id, link_type) {
                Ok(_) => stats.successful_imports += 1,
                Err(e) => {
                    stats.failed_imports += 1;
                    stats.validation_errors.push(format!("Line {line_number}: Failed to create link: {e}"));
                }
            }
        }
        
        // Log import activity
        audit_log_create("import", "csv", &format!("CSV import completed: {} successful, {} failed, {} duplicates", 
            stats.successful_imports, stats.failed_imports, stats.duplicates_found))?;
        
        Ok(stats)
    }
    
    // Import traceability links from JSON
    pub fn import_from_json(&self, file_path: &Path) -> QmsResult<ImportStats> {
        let content = fs::read_to_string(file_path)?;
        let mut stats = ImportStats {
            total_processed: 0,
            successful_imports: 0,
            failed_imports: 0,
            duplicates_found: 0,
            validation_errors: Vec::new(),
        };
        
        // Simple JSON parsing (basic implementation)
        let lines: Vec<&str> = content.lines().collect();
        let mut in_links_section = false;
        let mut in_link_object = false;
        let mut current_link = TraceabilityLink {
            id: String::new(),
            source_type: String::new(),
            source_id: String::new(),
            target_type: String::new(),
            target_id: String::new(),
            link_type: TraceLinkType::Related,
            created_at: current_timestamp().to_string(),
            created_by: "import".to_string(),
            verified: false,
            verified_at: None,
            verified_by: None,
        };
        
        for line in lines {
            let line = line.trim();
            
            if line.contains("\"links\":") {
                in_links_section = true;
                continue;
            }
            
            if !in_links_section {
                continue;
            }
            
            if line == "{" {
                in_link_object = true;
                stats.total_processed += 1;
                continue;
            }
            
            if line == "}" || line == "}," {
                if in_link_object {
                    // Process completed link
                    if !current_link.source_id.is_empty() && !current_link.target_id.is_empty() {
                        if self.link_exists(&current_link.source_id, &current_link.target_id, &current_link.link_type)? {
                            stats.duplicates_found += 1;
                        } else {
                            match self.create_trace_link(
                                &current_link.source_id,
                                &current_link.target_id,
                                current_link.link_type.clone(),
                            ) {
                                Ok(_) => stats.successful_imports += 1,
                                Err(e) => {
                                    stats.failed_imports += 1;
                                    stats.validation_errors.push(format!("Failed to create link: {e}"));
                                }
                            }
                        }
                    } else {
                        stats.failed_imports += 1;
                        stats.validation_errors.push("Incomplete link data".to_string());
                    }
                    
                    // Reset for next link
                    current_link = TraceabilityLink {
                        id: String::new(),
                        source_type: String::new(),
                        source_id: String::new(),
                        target_type: String::new(),
                        target_id: String::new(),
                        link_type: TraceLinkType::Related,
                        created_at: current_timestamp().to_string(),
                        created_by: "import".to_string(),
                        verified: false,
                        verified_at: None,
                        verified_by: None,
                    };
                    in_link_object = false;
                }
                continue;
            }
            
            if in_link_object {
                // Parse JSON fields
                if line.contains("\"source_type\":") {
                    current_link.source_type = extract_json_string_value(line);
                } else if line.contains("\"source_id\":") {
                    current_link.source_id = extract_json_string_value(line);
                } else if line.contains("\"target_type\":") {
                    current_link.target_type = extract_json_string_value(line);
                } else if line.contains("\"target_id\":") {
                    current_link.target_id = extract_json_string_value(line);
                } else if line.contains("\"link_type\":") {
                    let link_type_str = extract_json_string_value(line);
                    current_link.link_type = match link_type_str.as_str() {
                        "Verifies" => TraceLinkType::Verifies,
                        "Implements" => TraceLinkType::Implements,
                        "DependsOn" => TraceLinkType::DependsOn,
                        "DerivedFrom" => TraceLinkType::DerivedFrom,
                        "Related" => TraceLinkType::Related,
                        "Conflicts" => TraceLinkType::Conflicts,
                        "Duplicates" => TraceLinkType::Duplicates,
                        _ => TraceLinkType::Related,
                    };
                } else if line.contains("\"created_by\":") {
                    current_link.created_by = extract_json_string_value(line);
                }
            }
        }
        
        // Log import activity
        audit_log_create("import", "json", &format!("JSON import completed: {} successful, {} failed, {} duplicates", 
            stats.successful_imports, stats.failed_imports, stats.duplicates_found))?;
        
        Ok(stats)
    }
    
    // Helper method to check if a link already exists
    fn link_exists(&self, source_id: &str, target_id: &str, link_type: &TraceLinkType) -> QmsResult<bool> {
        let links = self.get_trace_links()?;
        Ok(links.iter().any(|link| 
            link.source_id == source_id && 
            link.target_id == target_id && 
            std::mem::discriminant(&link.link_type) == std::mem::discriminant(link_type)
        ))
    }
    
    // Export dependency graph in DOT format
    pub fn export_dependency_graph(&self, output_path: &Path) -> QmsResult<()> {
        let links = self.get_trace_links()?;
        let mut dot_content = String::new();
        
        dot_content.push_str("digraph TraceabilityGraph {\n");
        dot_content.push_str("    rankdir=TB;\n");
        dot_content.push_str("    node [shape=box];\n");
        
        // Add nodes
        let mut entities = std::collections::HashSet::new();
        for link in &links {
            entities.insert(format!("\"{}\"", link.source_id));
            entities.insert(format!("\"{}\"", link.target_id));
        }
        
        for entity in &entities {
            dot_content.push_str(&format!("    {entity};\n"));
        }
        
        // Add edges
        for link in &links {
            dot_content.push_str(&format!("    \"{}\" -> \"{}\" [label=\"{}\"];\n", 
                link.source_id, link.target_id, link.link_type));
        }
        
        dot_content.push_str("}\n");
        
        fs::write(output_path, dot_content)?;
        
        // Log export activity
        audit_log_create("export", "dot", &format!("Dependency graph exported to DOT: {}", output_path.display()))?;
        
        Ok(())
    }
}

// Helper function to extract string values from JSON
fn extract_json_string_value(line: &str) -> String {
    // Simple extraction: find content between quotes
    let parts: Vec<&str> = line.split('"').collect();
    if parts.len() >= 4 {
        parts[3].to_string()
    } else {
        String::new()
    }
}
