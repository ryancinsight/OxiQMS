#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::error::QmsResult;
use crate::modules::audit_logger::entry::{AuditLogger, AuditConfig};
use crate::modules::traceability::requirement::Requirement;
use crate::modules::traceability::test_case::TestCase;
use crate::modules::traceability::links::TraceabilityLink;

/// Supported graph output formats
#[derive(Debug, Clone)]
pub enum GraphFormat {
    ASCII,
    SVG,
    DOT,
}

impl GraphFormat {
    pub const fn as_str(&self) -> &'static str {
        match self {
            GraphFormat::ASCII => "ascii",
            GraphFormat::SVG => "svg",
            GraphFormat::DOT => "dot",
        }
    }
}

impl std::fmt::Display for GraphFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Node type in the dependency graph
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeType {
    Requirement(String),
    TestCase(String),
    Design(String),
    Risk(String),
    Document(String),
}

impl NodeType {
    pub fn id(&self) -> &str {
        match self {
            NodeType::Requirement(id) => id,
            NodeType::TestCase(id) => id,
            NodeType::Design(id) => id,
            NodeType::Risk(id) => id,
            NodeType::Document(id) => id,
        }
    }

    pub const fn node_type(&self) -> &str {
        match self {
            NodeType::Requirement(_) => "requirement",
            NodeType::TestCase(_) => "testcase",
            NodeType::Design(_) => "design",
            NodeType::Risk(_) => "risk",
            NodeType::Document(_) => "document",
        }
    }
}

/// Node information for graph visualization
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: NodeType,
    pub attributes: HashMap<String, String>,
    pub children: Vec<String>,
}

/// Edge information for graph visualization
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub attributes: HashMap<String, String>,
}

/// Graph visualization generator
pub struct GraphVisualizer {
    _audit_logger: AuditLogger,
}

impl GraphVisualizer {
    pub fn new(project_path: &Path) -> QmsResult<Self> {
        let audit_config = AuditConfig {
            project_path: project_path.to_string_lossy().to_string(),
            retention_days: 2555,
            daily_rotation: true,
            max_file_size_mb: 100,
            require_checksums: true,
        };
        
        let audit_logger = AuditLogger::new(audit_config)?;
        
        Ok(GraphVisualizer {
            _audit_logger: audit_logger,
        })
    }

    /// Generate a dependency graph from requirements and links
    pub fn generate_graph(
        &self,
        requirements: &[Requirement],
        test_cases: &[TestCase],
        links: &[TraceabilityLink],
        format: GraphFormat,
        output_path: &Path,
    ) -> QmsResult<()> {
        // Build graph structure
        let (nodes, edges) = self.build_graph_structure(requirements, test_cases, links)?;
        
        // Generate output based on format
        match format {
            GraphFormat::ASCII => self.generate_ascii_graph(&nodes, &edges, output_path)?,
            GraphFormat::SVG => self.generate_svg_graph(&nodes, &edges, output_path)?,
            GraphFormat::DOT => self.generate_dot_graph(&nodes, &edges, output_path)?,
        }
        
        Ok(())
    }

    /// Build the graph structure from requirements and links
    fn build_graph_structure(
        &self,
        requirements: &[Requirement],
        test_cases: &[TestCase],
        links: &[TraceabilityLink],
    ) -> QmsResult<(Vec<GraphNode>, Vec<GraphEdge>)> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_map = HashMap::new();

        // Create requirement nodes
        for req in requirements {
            let node = GraphNode {
                id: req.req_id.clone(),
                label: format!("{} ({})", req.req_id, req.title),
                node_type: NodeType::Requirement(req.req_id.clone()),
                attributes: {
                    let mut attrs = HashMap::new();
                    attrs.insert("category".to_string(), format!("{:?}", req.category));
                    attrs.insert("priority".to_string(), format!("{:?}", req.priority));
                    attrs.insert("status".to_string(), format!("{:?}", req.status));
                    attrs
                },
                children: Vec::new(),
            };
            node_map.insert(req.req_id.clone(), node.clone());
            nodes.push(node);
        }

        // Create test case nodes
        for tc in test_cases {
            let node = GraphNode {
                id: tc.test_id.clone(),
                label: format!("{} ({})", tc.test_id, tc.title),
                node_type: NodeType::TestCase(tc.test_id.clone()),
                attributes: {
                    let mut attrs = HashMap::new();
                    attrs.insert("category".to_string(), format!("{:?}", tc.category));
                    attrs.insert("priority".to_string(), format!("{:?}", tc.priority));
                    attrs.insert("created_by".to_string(), tc.created_by.clone());
                    attrs
                },
                children: Vec::new(),
            };
            node_map.insert(tc.test_id.clone(), node.clone());
            nodes.push(node);
        }

        // Create edges from links
        for link in links {
            let edge = GraphEdge {
                from: link.source_id.clone(),
                to: link.target_id.clone(),
                label: Some(format!("{:?}", link.link_type)),
                attributes: {
                    let mut attrs = HashMap::new();
                    attrs.insert("verified".to_string(), link.verified.to_string());
                    attrs.insert("created_at".to_string(), link.created_at.to_string());
                    attrs
                },
            };
            edges.push(edge);

            // Update parent-child relationships
            if let Some(parent_node) = node_map.get_mut(&link.source_id) {
                parent_node.children.push(link.target_id.clone());
            }
        }

        Ok((nodes, edges))
    }

    /// Generate ASCII text-based graph
    fn generate_ascii_graph(
        &self,
        nodes: &[GraphNode],
        edges: &[GraphEdge],
        output_path: &Path,
    ) -> QmsResult<()> {
        let mut content = String::new();
        content.push_str("Traceability Dependency Graph\n");
        content.push_str("═══════════════════════════════\n\n");

        // Build adjacency list
        let mut adjacency = HashMap::new();
        for edge in edges {
            adjacency.entry(edge.from.clone())
                .or_insert_with(Vec::new)
                .push(edge.to.clone());
        }

        // Find root nodes (nodes with no incoming edges)
        let mut has_incoming = HashSet::new();
        for edge in edges {
            has_incoming.insert(edge.to.clone());
        }

        let root_nodes: Vec<_> = nodes.iter()
            .filter(|node| !has_incoming.contains(&node.id))
            .collect();

        // Generate ASCII tree for each root node
        for root in root_nodes {
            content.push_str(&self.generate_ascii_tree(root, &adjacency, nodes, 0, &mut HashSet::new()));
            content.push('\n');
        }

        // Write to file
        let mut file = File::create(output_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    /// Generate ASCII tree structure recursively
    fn generate_ascii_tree(
        &self,
        node: &GraphNode,
        adjacency: &HashMap<String, Vec<String>>,
        all_nodes: &[GraphNode],
        depth: usize,
        visited: &mut HashSet<String>,
    ) -> String {
        let mut result = String::new();
        
        // Prevent infinite loops
        if visited.contains(&node.id) {
            return format!("{}↻ {} (circular)\n", "    ".repeat(depth), node.label);
        }
        visited.insert(node.id.clone());

        // Add current node
        let prefix = if depth == 0 {
            ""
        } else {
            "├── "
        };
        result.push_str(&format!("{}{}{}\n", "    ".repeat(depth), prefix, node.label));

        // Add children
        if let Some(children) = adjacency.get(&node.id) {
            for (i, child_id) in children.iter().enumerate() {
                if let Some(child_node) = all_nodes.iter().find(|n| n.id == *child_id) {
                    let child_prefix = if i == children.len() - 1 { "└── " } else { "├── " };
                    result.push_str(&format!("{}{}{}\n", "    ".repeat(depth + 1), child_prefix, child_node.label));
                    
                    // Recursively add grandchildren
                    let grandchild_result = self.generate_ascii_tree(child_node, adjacency, all_nodes, depth + 2, visited);
                    result.push_str(&grandchild_result);
                }
            }
        }

        visited.remove(&node.id);
        result
    }

    /// Generate SVG graph
    fn generate_svg_graph(
        &self,
        nodes: &[GraphNode],
        edges: &[GraphEdge],
        output_path: &Path,
    ) -> QmsResult<()> {
        let mut content = String::new();
        content.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        content.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 800 600\">\n");
        content.push_str("  <style>\n");
        content.push_str("    .node { stroke: #333; stroke-width: 1; fill: #f9f9f9; }\n");
        content.push_str("    .requirement { fill: #e1f5fe; }\n");
        content.push_str("    .testcase { fill: #f3e5f5; }\n");
        content.push_str("    .design { fill: #e8f5e8; }\n");
        content.push_str("    .risk { fill: #ffebee; }\n");
        content.push_str("    .edge { stroke: #666; stroke-width: 1; fill: none; }\n");
        content.push_str("    .edge-label { font-size: 10px; fill: #333; }\n");
        content.push_str("    .node-label { font-size: 12px; fill: #333; text-anchor: middle; }\n");
        content.push_str("  </style>\n");

        // Simple layout: arrange nodes in a grid
        let cols = (nodes.len() as f64).sqrt().ceil() as usize;
        let node_width = 120.0;
        let node_height = 40.0;
        let margin_x = 40.0;
        let margin_y = 40.0;

        // Calculate positions
        let mut node_positions = HashMap::new();
        for (i, node) in nodes.iter().enumerate() {
            let row = i / cols;
            let col = i % cols;
            let x = margin_x + col as f64 * (node_width + margin_x);
            let y = margin_y + row as f64 * (node_height + margin_y);
            node_positions.insert(node.id.clone(), (x, y));
        }

        // Add arrowhead marker
        content.push_str("  <defs>\n");
        content.push_str("    <marker id=\"arrowhead\" markerWidth=\"10\" markerHeight=\"7\" refX=\"9\" refY=\"3.5\" orient=\"auto\">\n");
        content.push_str("      <polygon points=\"0 0, 10 3.5, 0 7\" fill=\"#666\" />\n");
        content.push_str("    </marker>\n");
        content.push_str("  </defs>\n");

        // Draw edges first (so they appear behind nodes)
        for edge in edges {
            if let (Some((x1, y1)), Some((x2, y2))) = 
                (node_positions.get(&edge.from), node_positions.get(&edge.to)) {
                let (x1, y1) = (x1 + node_width / 2.0, y1 + node_height / 2.0);
                let (x2, y2) = (x2 + node_width / 2.0, y2 + node_height / 2.0);
                
                content.push_str(&format!(
                    "  <line x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" class=\"edge\" marker-end=\"url(#arrowhead)\" />\n"
                ));
            }
        }

        // Draw nodes
        for node in nodes {
            if let Some((x, y)) = node_positions.get(&node.id) {
                let class = match node.node_type {
                    NodeType::Requirement(_) => "node requirement",
                    NodeType::TestCase(_) => "node testcase",
                    NodeType::Design(_) => "node design",
                    NodeType::Risk(_) => "node risk",
                    NodeType::Document(_) => "node",
                };

                content.push_str(&format!(
                    "  <rect x=\"{x}\" y=\"{y}\" width=\"{node_width}\" height=\"{node_height}\" class=\"{class}\" />\n"
                ));
                content.push_str(&format!(
                    "  <text x=\"{}\" y=\"{}\" class=\"node-label\">{}</text>\n",
                    x + node_width / 2.0, y + node_height / 2.0 + 4.0, node.id
                ));
            }
        }

        content.push_str("</svg>\n");

        // Write to file
        let mut file = File::create(output_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    /// Generate DOT graph (Graphviz format)
    fn generate_dot_graph(
        &self,
        nodes: &[GraphNode],
        edges: &[GraphEdge],
        output_path: &Path,
    ) -> QmsResult<()> {
        let mut content = String::new();
        content.push_str("digraph TraceabilityGraph {\n");
        content.push_str("  rankdir=TB;\n");
        content.push_str("  node [shape=box, style=filled];\n");
        content.push_str("  \n");

        // Define nodes
        content.push_str("  // Nodes\n");
        for node in nodes {
            let color = match node.node_type {
                NodeType::Requirement(_) => "lightblue",
                NodeType::TestCase(_) => "lightgreen",
                NodeType::Design(_) => "lightyellow",
                NodeType::Risk(_) => "lightcoral",
                NodeType::Document(_) => "lightgray",
            };

            content.push_str(&format!(
                "  \"{}\" [label=\"{}\", fillcolor={}];\n",
                node.id, node.label, color
            ));
        }

        content.push_str("  \n");

        // Define edges
        content.push_str("  // Edges\n");
        for edge in edges {
            let label = edge.label.as_ref().map(|l| format!(" [label=\"{l}\"]")).unwrap_or_default();
            content.push_str(&format!(
                "  \"{}\" -> \"{}\"{};\n",
                edge.from, edge.to, label
            ));
        }

        content.push_str("}\n");

        // Write to file
        let mut file = File::create(output_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    /// Generate interactive HTML graph with clickable nodes
    pub fn generate_interactive_html(
        &self,
        nodes: &[GraphNode],
        edges: &[GraphEdge],
        output_path: &Path,
    ) -> QmsResult<()> {
        let mut content = String::new();
        content.push_str("<!DOCTYPE html>\n");
        content.push_str("<html>\n");
        content.push_str("<head>\n");
        content.push_str("    <title>Interactive Traceability Graph</title>\n");
        content.push_str("    <style>\n");
        content.push_str("        body { font-family: Arial, sans-serif; margin: 20px; }\n");
        content.push_str("        .graph-container { border: 1px solid #ccc; height: 600px; }\n");
        content.push_str("        .node-info { margin-top: 20px; padding: 10px; background: #f5f5f5; }\n");
        content.push_str("    </style>\n");
        content.push_str("</head>\n");
        content.push_str("<body>\n");
        content.push_str("    <h1>Interactive Traceability Graph</h1>\n");
        content.push_str("    <div class=\"graph-container\" id=\"graph\"></div>\n");
        content.push_str("    <div class=\"node-info\" id=\"node-info\">\n");
        content.push_str("        <h3>Node Information</h3>\n");
        content.push_str("        <p>Click on a node to view details</p>\n");
        content.push_str("    </div>\n");
        content.push_str("    \n");
        content.push_str("    <script>\n");
        content.push_str("        const nodes = [\n");

        // Add nodes data
        for (i, node) in nodes.iter().enumerate() {
            content.push_str(&format!(
                "            {{ id: '{}', label: '{}', type: '{}' }}{}",
                node.id, node.label, node.node_type.node_type(),
                if i < nodes.len() - 1 { ",\n" } else { "\n" }
            ));
        }

        content.push_str("        ];\n");
        content.push_str("        \n");
        content.push_str("        const edges = [\n");

        // Add edges data
        for (i, edge) in edges.iter().enumerate() {
            content.push_str(&format!(
                "            {{ from: '{}', to: '{}' }}{}",
                edge.from, edge.to,
                if i < edges.len() - 1 { ",\n" } else { "\n" }
            ));
        }

        content.push_str("        ];\n");
        content.push_str("        \n");
        content.push_str("        // Simple rendering (would use D3.js or similar in a real implementation)\n");
        content.push_str("        const graphDiv = document.getElementById('graph');\n");
        content.push_str("        graphDiv.innerHTML = '<p>Interactive graph would be rendered here using D3.js or similar library</p>';\n");
        content.push_str("        \n");
        content.push_str("        function showNodeInfo(nodeId) {\n");
        content.push_str("            const node = nodes.find(n => n.id === nodeId);\n");
        content.push_str("            if (node) {\n");
        content.push_str("                document.getElementById('node-info').innerHTML = `\n");
        content.push_str("                    <h3>Node Information</h3>\n");
        content.push_str("                    <p><strong>ID:</strong> ${node.id}</p>\n");
        content.push_str("                    <p><strong>Label:</strong> ${node.label}</p>\n");
        content.push_str("                    <p><strong>Type:</strong> ${node.type}</p>\n");
        content.push_str("                `;\n");
        content.push_str("            }\n");
        content.push_str("        }\n");
        content.push_str("    </script>\n");
        content.push_str("</body>\n");
        content.push_str("</html>\n");

        // Write to file
        let mut file = File::create(output_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_graph_format_display() {
        assert_eq!(GraphFormat::ASCII.to_string(), "ascii");
        assert_eq!(GraphFormat::SVG.to_string(), "svg");
        assert_eq!(GraphFormat::DOT.to_string(), "dot");
    }

    #[test]
    fn test_node_type_methods() {
        let req_node = NodeType::Requirement("REQ-001".to_string());
        assert_eq!(req_node.id(), "REQ-001");
        assert_eq!(req_node.node_type(), "requirement");

        let test_node = NodeType::TestCase("TC-001".to_string());
        assert_eq!(test_node.id(), "TC-001");
        assert_eq!(test_node.node_type(), "testcase");
    }

    #[test]
    fn test_visualizer_creation() {
        let temp_dir = TempDir::new().unwrap();
        let visualizer = GraphVisualizer::new(temp_dir.path());
        assert!(visualizer.is_ok());
    }
}