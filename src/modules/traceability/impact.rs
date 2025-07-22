use crate::prelude::*;
use crate::modules::traceability::links::{TraceabilityManager, TraceabilityLink};
use crate::modules::traceability::requirement::{Requirement, RequirementPriority};
use crate::audit::log_audit;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::fs;

/// Simple test case structure for impact analysis
#[derive(Debug, Clone)]
pub struct TestCase {
    #[allow(dead_code)]
    pub id: String,
    pub title: String,
    pub test_type: String,
    pub automation_level: String,
    #[allow(dead_code)]
    pub priority: String,
}

impl Default for TestCase {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            test_type: "Unit".to_string(),
            automation_level: "Manual".to_string(),
            priority: "Medium".to_string(),
        }
    }
}

/// Impact analysis types
#[derive(Debug, Clone, PartialEq)]
pub enum ImpactType {
    Requirement,
    TestCase,
    Design,
    Risk,
    Document,
}

/// Impact level severity
#[derive(Debug, Clone, PartialEq)]
pub enum ImpactLevel {
    Critical,  // High impact, requires immediate attention
    High,      // Significant impact, needs prompt action
    Medium,    // Moderate impact, should be addressed
    Low,       // Minor impact, can be handled later
}

/// Impact analysis result for a single entity
#[derive(Debug, Clone)]
pub struct ImpactItem {
    pub entity_id: String,
    pub entity_type: ImpactType,
    pub entity_title: String,
    pub impact_level: ImpactLevel,
    pub impact_description: String,
    #[allow(dead_code)]
    pub affected_attributes: Vec<String>,
    pub estimated_effort_hours: Option<u32>,
    #[allow(dead_code)]
    pub required_actions: Vec<String>,
    pub stakeholders: Vec<String>,
    #[allow(dead_code)]
    pub risk_factors: Vec<String>,
}

/// Comprehensive impact analysis result
#[derive(Debug, Clone)]
pub struct ImpactAnalysis {
    pub source_entity_id: String,
    pub change_description: String,
    pub analysis_timestamp: String,
    pub direct_impacts: Vec<ImpactItem>,
    pub indirect_impacts: Vec<ImpactItem>,
    pub total_effort_estimate: u32,
    pub critical_path_items: Vec<String>,
    pub recommended_actions: Vec<String>,
    pub risk_assessment: String,
    pub stakeholder_summary: HashMap<String, Vec<String>>,
}

/// Impact analysis engine
pub struct ImpactAnalyzer {
    traceability_manager: TraceabilityManager,
    project_root: PathBuf,
}

impl ImpactAnalyzer {
    /// Create new impact analyzer
    pub fn new(project_root: &Path) -> QmsResult<Self> {
        let traceability_manager = TraceabilityManager::new(project_root)?;
        Ok(Self {
            traceability_manager,
            project_root: project_root.to_path_buf(),
        })
    }

    /// Analyze impact of a requirement change
    pub fn analyze_requirement_impact(
        &self,
        requirement_id: &str,
        change_description: &str,
    ) -> QmsResult<ImpactAnalysis> {
        let timestamp = crate::utils::current_timestamp();
        
        // Get all links for this requirement
        let links = self.traceability_manager.get_links_for_entity(requirement_id)?;
        
        // Analyze direct impacts
        let direct_impacts = self.analyze_direct_impacts(requirement_id, &links)?;
        
        // Analyze indirect impacts (impacts on items linked to direct impacts)
        let indirect_impacts = self.analyze_indirect_impacts(&direct_impacts)?;
        
        // Calculate effort estimates
        let total_effort_estimate = self.calculate_total_effort(&direct_impacts, &indirect_impacts);
        
        // Identify critical path items
        let critical_path_items = self.identify_critical_path(&direct_impacts, &indirect_impacts);
        
        // Generate recommendations
        let recommended_actions = self.generate_recommendations(&direct_impacts, &indirect_impacts);
        
        // Assess overall risk
        let risk_assessment = self.assess_change_risk(&direct_impacts, &indirect_impacts);
        
        // Summarize stakeholders
        let stakeholder_summary = self.summarize_stakeholders(&direct_impacts, &indirect_impacts);
        
        let analysis = ImpactAnalysis {
            source_entity_id: requirement_id.to_string(),
            change_description: change_description.to_string(),
            analysis_timestamp: timestamp.to_string(),
            direct_impacts,
            indirect_impacts,
            total_effort_estimate,
            critical_path_items,
            recommended_actions,
            risk_assessment,
            stakeholder_summary,
        };
        
        // Log the analysis
        log_audit(&format!(
            "IMPACT_ANALYSIS: Analyzed impact for requirement {} - {} direct impacts, {} indirect impacts, {} hour estimate, Change: {}",
            requirement_id, 
            analysis.direct_impacts.len(),
            analysis.indirect_impacts.len(),
            analysis.total_effort_estimate,
            change_description
        ));
        
        Ok(analysis)
    }

    /// Analyze direct impacts from traceability links
    fn analyze_direct_impacts(&self, requirement_id: &str, links: &[TraceabilityLink]) -> QmsResult<Vec<ImpactItem>> {
        let mut impacts = Vec::new();
        
        for link in links {
            let target_id = if link.source_id == requirement_id {
                &link.target_id
            } else {
                &link.source_id
            };
            
            // Determine entity type and analyze impact
            let impact_item = match self.determine_entity_type(target_id) {
                ImpactType::TestCase => self.analyze_test_case_impact(target_id)?,
                ImpactType::Design => self.analyze_design_impact(target_id)?,
                ImpactType::Risk => self.analyze_risk_impact(target_id)?,
                ImpactType::Document => self.analyze_document_impact(target_id)?,
                ImpactType::Requirement => self.analyze_requirement_dependency_impact(target_id)?,
            };
            
            impacts.push(impact_item);
        }
        
        Ok(impacts)
    }

    /// Analyze indirect impacts (second-degree impacts)
    fn analyze_indirect_impacts(&self, direct_impacts: &[ImpactItem]) -> QmsResult<Vec<ImpactItem>> {
        let mut indirect_impacts = Vec::new();
        let mut processed_entities = HashSet::new();
        
        for direct_impact in direct_impacts {
            let links = self.traceability_manager.get_links_for_entity(&direct_impact.entity_id)?;
            
            for link in links {
                let target_id = if link.source_id == direct_impact.entity_id {
                    &link.target_id
                } else {
                    &link.source_id
                };
                
                // Skip if already processed
                if processed_entities.contains(target_id) {
                    continue;
                }
                processed_entities.insert(target_id.clone());
                
                // Analyze indirect impact (with reduced severity)
                let mut impact_item = match self.determine_entity_type(target_id) {
                    ImpactType::TestCase => self.analyze_test_case_impact(target_id)?,
                    ImpactType::Design => self.analyze_design_impact(target_id)?,
                    ImpactType::Risk => self.analyze_risk_impact(target_id)?,
                    ImpactType::Document => self.analyze_document_impact(target_id)?,
                    ImpactType::Requirement => self.analyze_requirement_dependency_impact(target_id)?,
                };
                
                // Reduce impact level for indirect impacts
                impact_item.impact_level = match impact_item.impact_level {
                    ImpactLevel::Critical => ImpactLevel::High,
                    ImpactLevel::High => ImpactLevel::Medium,
                    ImpactLevel::Medium => ImpactLevel::Low,
                    ImpactLevel::Low => ImpactLevel::Low,
                };
                
                // Update description to indicate indirect impact
                impact_item.impact_description = format!(
                    "Indirect impact via {}: {}",
                    direct_impact.entity_id,
                    impact_item.impact_description
                );
                
                indirect_impacts.push(impact_item);
            }
        }
        
        Ok(indirect_impacts)
    }

    /// Analyze test case impact
    fn analyze_test_case_impact(&self, test_case_id: &str) -> QmsResult<ImpactItem> {
        let test_case = self.load_test_case(test_case_id)?;
        
        let impact_level = match test_case.test_type.as_str() {
            "System" => ImpactLevel::Critical,
            "Integration" => ImpactLevel::High,
            "Unit" => ImpactLevel::Medium,
            _ => ImpactLevel::Low,
        };
        
        let effort_estimate = match test_case.automation_level.as_str() {
            "Manual" => 8,  // 8 hours for manual test update
            "SemiAutomatic" => 4,  // 4 hours for semi-automatic
            "FullyAutomatic" => 2,  // 2 hours for automated test update
            _ => 4,
        };
        
        Ok(ImpactItem {
            entity_id: test_case_id.to_string(),
            entity_type: ImpactType::TestCase,
            entity_title: test_case.title.clone(),
            impact_level,
            impact_description: format!(
                "Test case '{}' requires updates to test steps, expected results, and verification methods",
                test_case.title
            ),
            affected_attributes: vec![
                "Test Steps".to_string(),
                "Expected Results".to_string(),
                "Preconditions".to_string(),
                "Test Data".to_string(),
            ],
            estimated_effort_hours: Some(effort_estimate),
            required_actions: vec![
                "Review test case design".to_string(),
                "Update test steps".to_string(),
                "Modify expected results".to_string(),
                "Execute test validation".to_string(),
            ],
            stakeholders: vec![
                "Test Engineer".to_string(),
                "Quality Assurance".to_string(),
            ],
            risk_factors: vec![
                "Test coverage gaps".to_string(),
                "Regression risk".to_string(),
            ],
        })
    }

    /// Analyze design impact
    fn analyze_design_impact(&self, design_id: &str) -> QmsResult<ImpactItem> {
        Ok(ImpactItem {
            entity_id: design_id.to_string(),
            entity_type: ImpactType::Design,
            entity_title: format!("Design Element {design_id}"),
            impact_level: ImpactLevel::High,
            impact_description: "Design element requires architectural review and potential redesign".to_string(),
            affected_attributes: vec![
                "Architecture".to_string(),
                "Interfaces".to_string(),
                "Implementation".to_string(),
            ],
            estimated_effort_hours: Some(16),  // 16 hours for design changes
            required_actions: vec![
                "Conduct architectural review".to_string(),
                "Update design documentation".to_string(),
                "Review interface specifications".to_string(),
                "Validate design changes".to_string(),
            ],
            stakeholders: vec![
                "Software Architect".to_string(),
                "Development Team".to_string(),
                "System Engineer".to_string(),
            ],
            risk_factors: vec![
                "Integration issues".to_string(),
                "Performance impact".to_string(),
                "Compatibility risks".to_string(),
            ],
        })
    }

    /// Analyze risk impact
    fn analyze_risk_impact(&self, risk_id: &str) -> QmsResult<ImpactItem> {
        let risk_file = self.project_root.join("risks").join(format!("{risk_id}.json"));
        
        let impact_level = if risk_file.exists() {
            // Try to load the risk to determine severity
            if let Ok(content) = fs::read_to_string(&risk_file) {
                if content.contains("\"severity\": 5") || content.contains("\"severity\": 4") {
                    ImpactLevel::Critical
                } else if content.contains("\"severity\": 3") {
                    ImpactLevel::High
                } else {
                    ImpactLevel::Medium
                }
            } else {
                ImpactLevel::Medium
            }
        } else {
            ImpactLevel::Medium
        };
        
        Ok(ImpactItem {
            entity_id: risk_id.to_string(),
            entity_type: ImpactType::Risk,
            entity_title: format!("Risk {risk_id}"),
            impact_level,
            impact_description: "Risk assessment requires re-evaluation due to requirement changes".to_string(),
            affected_attributes: vec![
                "Risk Probability".to_string(),
                "Risk Impact".to_string(),
                "Mitigation Measures".to_string(),
                "Risk Priority Number".to_string(),
            ],
            estimated_effort_hours: Some(4),  // 4 hours for risk re-assessment
            required_actions: vec![
                "Re-assess risk probability".to_string(),
                "Evaluate risk impact".to_string(),
                "Review mitigation measures".to_string(),
                "Update risk register".to_string(),
            ],
            stakeholders: vec![
                "Risk Manager".to_string(),
                "Quality Engineer".to_string(),
                "Product Owner".to_string(),
            ],
            risk_factors: vec![
                "Increased system risk".to_string(),
                "Regulatory compliance impact".to_string(),
            ],
        })
    }

    /// Analyze document impact
    fn analyze_document_impact(&self, document_id: &str) -> QmsResult<ImpactItem> {
        Ok(ImpactItem {
            entity_id: document_id.to_string(),
            entity_type: ImpactType::Document,
            entity_title: format!("Document {document_id}"),
            impact_level: ImpactLevel::Medium,
            impact_description: "Document requires updates to reflect requirement changes".to_string(),
            affected_attributes: vec![
                "Content".to_string(),
                "Version".to_string(),
                "Approval Status".to_string(),
            ],
            estimated_effort_hours: Some(2),  // 2 hours for document updates
            required_actions: vec![
                "Review document content".to_string(),
                "Update relevant sections".to_string(),
                "Increment version number".to_string(),
                "Obtain re-approval".to_string(),
            ],
            stakeholders: vec![
                "Technical Writer".to_string(),
                "Document Approver".to_string(),
            ],
            risk_factors: vec![
                "Documentation inconsistency".to_string(),
                "Approval delays".to_string(),
            ],
        })
    }

    /// Analyze requirement dependency impact
    fn analyze_requirement_dependency_impact(&self, requirement_id: &str) -> QmsResult<ImpactItem> {
        let requirement = self.load_requirement(requirement_id)?;
        
        let impact_level = match requirement.priority.as_str() {
            "Critical" => ImpactLevel::Critical,
            "High" => ImpactLevel::High,
            "Medium" => ImpactLevel::Medium,
            _ => ImpactLevel::Low,
        };
        
        Ok(ImpactItem {
            entity_id: requirement_id.to_string(),
            entity_type: ImpactType::Requirement,
            entity_title: requirement.title.clone(),
            impact_level,
            impact_description: format!(
                "Dependent requirement '{}' may need updates due to changes in linked requirement",
                requirement.title
            ),
            affected_attributes: vec![
                "Requirement Text".to_string(),
                "Acceptance Criteria".to_string(),
                "Priority".to_string(),
                "Status".to_string(),
            ],
            estimated_effort_hours: Some(6),  // 6 hours for requirement analysis and updates
            required_actions: vec![
                "Review requirement dependencies".to_string(),
                "Update requirement text if needed".to_string(),
                "Revise acceptance criteria".to_string(),
                "Validate requirement consistency".to_string(),
            ],
            stakeholders: vec![
                "Business Analyst".to_string(),
                "Product Owner".to_string(),
                "Quality Engineer".to_string(),
            ],
            risk_factors: vec![
                "Requirement conflicts".to_string(),
                "Scope creep".to_string(),
                "Delivery delays".to_string(),
            ],
        })
    }

    /// Determine entity type from ID
    fn determine_entity_type(&self, entity_id: &str) -> ImpactType {
        if entity_id.starts_with("REQ-") {
            ImpactType::Requirement
        } else if entity_id.starts_with("TC-") {
            ImpactType::TestCase
        } else if entity_id.starts_with("RISK-") {
            ImpactType::Risk
        } else if entity_id.starts_with("DOC-") {
            ImpactType::Document
        } else if entity_id.starts_with("DESIGN-") {
            ImpactType::Design
        } else {
            ImpactType::Design  // Default fallback
        }
    }

    /// Load test case from file
    fn load_test_case(&self, test_case_id: &str) -> QmsResult<TestCase> {
        let test_case_file = self.project_root.join("trace").join("test_cases").join(format!("{test_case_id}.json"));
        
        if !test_case_file.exists() {
            // Return default test case if file doesn't exist
            return Ok(TestCase {
                id: test_case_id.to_string(),
                title: format!("Test Case {test_case_id}"),
                test_type: "Unit".to_string(),
                automation_level: "Manual".to_string(),
                priority: "Medium".to_string(),
                ..Default::default()
            });
        }
        
        let content = fs::read_to_string(&test_case_file)?;
        
        // Simple JSON parsing - extract basic fields
        let id = test_case_id.to_string();
        let title = if content.contains("\"title\":") {
            // Extract title from JSON-like content
            content.lines()
                .find(|line| line.contains("\"title\":"))
                .and_then(|line| {
                    line.split("\"title\":").nth(1)?
                        .split("\"").nth(1)
                })
                .unwrap_or(&format!("Test Case {test_case_id}"))
                .to_string()
        } else {
            format!("Test Case {test_case_id}")
        };
        
        let test_type = if content.contains("\"test_type\":") {
            content.lines()
                .find(|line| line.contains("\"test_type\":"))
                .and_then(|line| {
                    line.split("\"test_type\":").nth(1)?
                        .split("\"").nth(1)
                })
                .unwrap_or("Unit")
                .to_string()
        } else {
            "Unit".to_string()
        };
        
        let automation_level = if content.contains("\"automation_level\":") {
            content.lines()
                .find(|line| line.contains("\"automation_level\":"))
                .and_then(|line| {
                    line.split("\"automation_level\":").nth(1)?
                        .split("\"").nth(1)
                })
                .unwrap_or("Manual")
                .to_string()
        } else {
            "Manual".to_string()
        };
        
        let priority = if content.contains("\"priority\":") {
            content.lines()
                .find(|line| line.contains("\"priority\":"))
                .and_then(|line| {
                    line.split("\"priority\":").nth(1)?
                        .split("\"").nth(1)
                })
                .unwrap_or("Medium")
                .to_string()
        } else {
            "Medium".to_string()
        };
        
        Ok(TestCase {
            id,
            title,
            test_type,
            automation_level,
            priority,
        })
    }

    /// Load requirement from file
    fn load_requirement(&self, requirement_id: &str) -> QmsResult<Requirement> {
        let req_file = self.project_root.join("trace").join("requirements").join(format!("{requirement_id}.json"));
        
        if !req_file.exists() {
            // Return default requirement if file doesn't exist
            return Ok(Requirement {
                id: requirement_id.to_string(),
                title: format!("Requirement {requirement_id}"),
                priority: RequirementPriority::Medium,
                ..Default::default()
            });
        }
        
        let content = fs::read_to_string(&req_file)?;
        
        // Simple JSON parsing - extract basic fields
        let id = requirement_id.to_string();
        let title = if content.contains("\"title\":") {
            content.lines()
                .find(|line| line.contains("\"title\":"))
                .and_then(|line| {
                    line.split("\"title\":").nth(1)?
                        .split("\"").nth(1)
                })
                .unwrap_or(&format!("Requirement {requirement_id}"))
                .to_string()
        } else {
            format!("Requirement {requirement_id}")
        };
        
        let priority = if content.contains("\"priority\":") {
            let priority_str = content.lines()
                .find(|line| line.contains("\"priority\":"))
                .and_then(|line| {
                    line.split("\"priority\":").nth(1)?
                        .split("\"").nth(1)
                })
                .unwrap_or("Medium");
            RequirementPriority::from_str(priority_str)
        } else {
            RequirementPriority::Medium
        };
        
        Ok(Requirement {
            id,
            title,
            priority,
            ..Default::default()
        })
    }

    /// Calculate total effort estimate
    fn calculate_total_effort(&self, direct_impacts: &[ImpactItem], indirect_impacts: &[ImpactItem]) -> u32 {
        let direct_effort: u32 = direct_impacts.iter()
            .map(|item| item.estimated_effort_hours.unwrap_or(0))
            .sum();
        
        let indirect_effort: u32 = indirect_impacts.iter()
            .map(|item| item.estimated_effort_hours.unwrap_or(0))
            .sum();
        
        direct_effort + indirect_effort
    }

    /// Identify critical path items
    fn identify_critical_path(&self, direct_impacts: &[ImpactItem], indirect_impacts: &[ImpactItem]) -> Vec<String> {
        let mut critical_items = Vec::new();
        
        // Add critical and high impact items to critical path
        for item in direct_impacts.iter().chain(indirect_impacts.iter()) {
            if matches!(item.impact_level, ImpactLevel::Critical | ImpactLevel::High) {
                critical_items.push(item.entity_id.clone());
            }
        }
        
        critical_items
    }

    /// Generate recommendations
    fn generate_recommendations(&self, direct_impacts: &[ImpactItem], indirect_impacts: &[ImpactItem]) -> Vec<String> {
        let mut recommendations = Vec::new();
        let _total_impacts = direct_impacts.len() + indirect_impacts.len();
        
        // Priority-based recommendations
        let critical_count = direct_impacts.iter().chain(indirect_impacts.iter())
            .filter(|item| item.impact_level == ImpactLevel::Critical)
            .count();
        
        if critical_count > 0 {
            recommendations.push(format!(
                "URGENT: {critical_count} critical impact items require immediate attention"
            ));
        }
        
        // Effort-based recommendations
        let total_effort = self.calculate_total_effort(direct_impacts, indirect_impacts);
        if total_effort > 40 {
            recommendations.push("Consider phased implementation due to high effort estimate".to_string());
        }
        
        // Stakeholder coordination
        let stakeholder_count: HashSet<String> = direct_impacts.iter()
            .chain(indirect_impacts.iter())
            .flat_map(|item| &item.stakeholders)
            .cloned()
            .collect();
        
        if stakeholder_count.len() > 3 {
            recommendations.push("Establish stakeholder coordination meeting due to multiple affected parties".to_string());
        }
        
        // Risk mitigation
        let risk_count = direct_impacts.iter().chain(indirect_impacts.iter())
            .filter(|item| item.entity_type == ImpactType::Risk)
            .count();
        
        if risk_count > 0 {
            recommendations.push("Conduct risk assessment review due to affected risk items".to_string());
        }
        
        // General recommendations
        recommendations.extend(vec![
            "Update project timeline to account for impact analysis".to_string(),
            "Notify affected stakeholders of upcoming changes".to_string(),
            "Consider regression testing for all affected test cases".to_string(),
        ]);
        
        recommendations
    }

    /// Assess overall change risk
    fn assess_change_risk(&self, direct_impacts: &[ImpactItem], indirect_impacts: &[ImpactItem]) -> String {
        let total_impacts = direct_impacts.len() + indirect_impacts.len();
        let critical_count = direct_impacts.iter().chain(indirect_impacts.iter())
            .filter(|item| item.impact_level == ImpactLevel::Critical)
            .count();
        
        let total_effort = self.calculate_total_effort(direct_impacts, indirect_impacts);
        
        if critical_count > 2 || total_effort > 60 {
            "HIGH RISK: Significant impact on multiple critical systems. Consider careful planning and phased rollout.".to_string()
        } else if critical_count > 0 || total_effort > 30 || total_impacts > 10 {
            "MEDIUM RISK: Notable impact on system components. Requires coordination and thorough testing.".to_string()
        } else if total_impacts > 5 || total_effort > 15 {
            "LOW RISK: Limited impact on system. Standard change management processes apply.".to_string()
        } else {
            "MINIMAL RISK: Minor impact with limited scope. Can proceed with normal development practices.".to_string()
        }
    }

    /// Summarize stakeholders by impact area
    fn summarize_stakeholders(&self, direct_impacts: &[ImpactItem], indirect_impacts: &[ImpactItem]) -> HashMap<String, Vec<String>> {
        let mut stakeholder_map = HashMap::new();
        
        for item in direct_impacts.iter().chain(indirect_impacts.iter()) {
            for stakeholder in &item.stakeholders {
                stakeholder_map.entry(stakeholder.clone())
                    .or_insert_with(Vec::new)
                    .push(item.entity_id.clone());
            }
        }
        
        stakeholder_map
    }
}

/// Format impact analysis as text report
pub fn format_impact_analysis(analysis: &ImpactAnalysis) -> String {
    let mut report = String::new();
    
    report.push_str("🔍 IMPACT ANALYSIS REPORT\n");
    report.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");
    
    report.push_str(&format!("📋 Source Entity: {}\n", analysis.source_entity_id));
    report.push_str(&format!("📝 Change Description: {}\n", analysis.change_description));
    report.push_str(&format!("🕒 Analysis Time: {}\n", analysis.analysis_timestamp));
    report.push_str(&format!("⏱️ Total Effort Estimate: {} hours\n", analysis.total_effort_estimate));
    report.push_str(&format!("🎯 Risk Assessment: {}\n\n", analysis.risk_assessment));
    
    if !analysis.direct_impacts.is_empty() {
        report.push_str("🎯 DIRECT IMPACTS\n");
        report.push_str("────────────────────────────────────────────────────────────────────────────────────────────────────\n");
        for (i, impact) in analysis.direct_impacts.iter().enumerate() {
            report.push_str(&format!("\n{}. {} ({})\n", i + 1, impact.entity_title, impact.entity_id));
            report.push_str(&format!("   Impact Level: {:?}\n", impact.impact_level));
            report.push_str(&format!("   Description: {}\n", impact.impact_description));
            report.push_str(&format!("   Effort Estimate: {} hours\n", impact.estimated_effort_hours.unwrap_or(0)));
            report.push_str(&format!("   Stakeholders: {}\n", impact.stakeholders.join(", ")));
        }
        report.push('\n');
    }
    
    if !analysis.indirect_impacts.is_empty() {
        report.push_str("🔗 INDIRECT IMPACTS\n");
        report.push_str("────────────────────────────────────────────────────────────────────────────────────────────────────\n");
        for (i, impact) in analysis.indirect_impacts.iter().enumerate() {
            report.push_str(&format!("\n{}. {} ({})\n", i + 1, impact.entity_title, impact.entity_id));
            report.push_str(&format!("   Impact Level: {:?}\n", impact.impact_level));
            report.push_str(&format!("   Description: {}\n", impact.impact_description));
            report.push_str(&format!("   Effort Estimate: {} hours\n", impact.estimated_effort_hours.unwrap_or(0)));
            report.push_str(&format!("   Stakeholders: {}\n", impact.stakeholders.join(", ")));
        }
        report.push('\n');
    }
    
    if !analysis.critical_path_items.is_empty() {
        report.push_str("⚠️  CRITICAL PATH ITEMS\n");
        report.push_str("────────────────────────────────────────────────────────────────────────────────────────────────────\n");
        for item in &analysis.critical_path_items {
            report.push_str(&format!("   • {item}\n"));
        }
        report.push('\n');
    }
    
    if !analysis.recommended_actions.is_empty() {
        report.push_str("💡 RECOMMENDED ACTIONS\n");
        report.push_str("────────────────────────────────────────────────────────────────────────────────────────────────────\n");
        for (i, action) in analysis.recommended_actions.iter().enumerate() {
            report.push_str(&format!("   {}. {}\n", i + 1, action));
        }
        report.push('\n');
    }
    
    if !analysis.stakeholder_summary.is_empty() {
        report.push_str("👥 STAKEHOLDER SUMMARY\n");
        report.push_str("────────────────────────────────────────────────────────────────────────────────────────────────────\n");
        for (stakeholder, entities) in &analysis.stakeholder_summary {
            report.push_str(&format!("   {}: {} affected items\n", stakeholder, entities.len()));
            for entity in entities {
                report.push_str(&format!("     • {entity}\n"));
            }
        }
        report.push('\n');
    }
    
    report.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    report.push_str("📊 SUMMARY\n");
    report.push_str(&format!("   Direct Impacts: {}\n", analysis.direct_impacts.len()));
    report.push_str(&format!("   Indirect Impacts: {}\n", analysis.indirect_impacts.len()));
    report.push_str(&format!("   Total Effort: {} hours\n", analysis.total_effort_estimate));
    report.push_str(&format!("   Critical Items: {}\n", analysis.critical_path_items.len()));
    report.push_str(&format!("   Stakeholders: {}\n", analysis.stakeholder_summary.len()));
    
    report
}
