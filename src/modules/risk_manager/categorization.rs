/*
 * QMS (Quality Management System)
 * Risk Categorization Module - Task 3.1.15
 * 
 * Implements comprehensive risk categorization and classification system
 * for medical device quality management per ISO 14971:2019
 * 
 * Categories: Safety, Security, Performance, Usability, Environmental
 * Classifications: By component, use case, user type, environment
 * 
 * Author: QMS Development Team
 * Date: January 2025
 * Version: 1.0.0
 */

use std::collections::HashMap;
use crate::models::AuditAction;
use crate::modules::risk_manager::risk::RiskManager;
use crate::modules::audit_logger::entry::log_action;

/// Primary risk categories for medical device risk management
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RiskCategory {
    Safety,      // Physical harm, device malfunction, clinical safety
    Security,    // Cybersecurity, data protection, access control
    Performance, // Functional performance, speed, reliability
    Usability,   // User interface, user experience, human factors
    Environmental, // Environmental conditions, sustainability, disposal
}

impl RiskCategory {
    /// Convert to string representation
    pub const fn as_str(&self) -> &'static str {
        match self {
            RiskCategory::Safety => "safety",
            RiskCategory::Security => "security", 
            RiskCategory::Performance => "performance",
            RiskCategory::Usability => "usability",
            RiskCategory::Environmental => "environmental",
        }
    }
    
    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "safety" => Ok(RiskCategory::Safety),
            "security" => Ok(RiskCategory::Security),
            "performance" => Ok(RiskCategory::Performance),
            "usability" => Ok(RiskCategory::Usability),
            "environmental" => Ok(RiskCategory::Environmental),
            _ => Err(format!("Invalid risk category: {s}")),
        }
    }
}

/// Risk classification dimensions
#[derive(Debug, Clone, PartialEq)]
pub enum ClassificationDimension {
    Component(String),    // Device component (e.g., "power_supply", "sensor", "software")
    UseCase(String),      // Use case scenario (e.g., "surgical_procedure", "home_monitoring")
    UserType(String),     // User type (e.g., "healthcare_professional", "patient", "caregiver")
    Environment(String),  // Environment (e.g., "hospital", "home", "emergency")
}

impl ClassificationDimension {
    /// Get the dimension type
    pub const fn dimension_type(&self) -> &'static str {
        match self {
            ClassificationDimension::Component(_) => "component",
            ClassificationDimension::UseCase(_) => "use_case",
            ClassificationDimension::UserType(_) => "user_type",
            ClassificationDimension::Environment(_) => "environment",
        }
    }
    
    /// Get the dimension value
    pub fn value(&self) -> &str {
        match self {
            ClassificationDimension::Component(v) |
            ClassificationDimension::UseCase(v) |
            ClassificationDimension::UserType(v) |
            ClassificationDimension::Environment(v) => v,
        }
    }
}

/// Risk subcategories for detailed classification
#[derive(Debug, Clone, PartialEq)]
pub struct RiskSubcategory {
    pub name: String,
    pub description: String,
    pub category: RiskCategory,
}

/// Complete risk categorization information
#[derive(Debug, Clone)]
#[allow(dead_code)] // Categorization fields for audit trail and reporting
pub struct RiskCategorization {
    pub risk_id: String,
    pub category: RiskCategory,
    pub subcategory: Option<RiskSubcategory>,
    pub classifications: Vec<ClassificationDimension>,
    pub tags: Vec<String>,
    pub assigned_by: String,
    pub assigned_date: String,
    pub rationale: String,
}

/// Statistics for risk categorization analysis
#[derive(Debug, Clone)]
pub struct CategoryStatistics {
    pub total_risks: usize,
    pub category_counts: HashMap<String, usize>,
    pub classification_counts: HashMap<String, HashMap<String, usize>>,
    pub high_risk_categories: Vec<(String, usize)>,
    pub coverage_analysis: HashMap<String, f64>,
}

/// Risk categorization manager
pub struct RiskCategorizationManager {
    categorizations: HashMap<String, RiskCategorization>,
    predefined_subcategories: HashMap<RiskCategory, Vec<RiskSubcategory>>,
}

impl Default for RiskCategorizationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RiskCategorizationManager {
    /// Create new categorization manager
    pub fn new() -> Self {
        let mut manager = Self {
            categorizations: HashMap::new(),
            predefined_subcategories: HashMap::new(),
        };
        
        manager.initialize_predefined_subcategories();
        manager
    }
    
    /// Initialize predefined subcategories per ISO 14971
    fn initialize_predefined_subcategories(&mut self) {
        // Safety subcategories
        let safety_subcategories = vec![
            RiskSubcategory {
                name: "electrical_hazard".to_string(),
                description: "Electrical shock, electrocution, or electrical fire risks".to_string(),
                category: RiskCategory::Safety,
            },
            RiskSubcategory {
                name: "mechanical_hazard".to_string(),
                description: "Mechanical injury from moving parts or structural failure".to_string(),
                category: RiskCategory::Safety,
            },
            RiskSubcategory {
                name: "thermal_hazard".to_string(),
                description: "Burns or thermal injury from excessive heat or cold".to_string(),
                category: RiskCategory::Safety,
            },
            RiskSubcategory {
                name: "radiation_hazard".to_string(),
                description: "Ionizing or non-ionizing radiation exposure risks".to_string(),
                category: RiskCategory::Safety,
            },
            RiskSubcategory {
                name: "chemical_biological_hazard".to_string(),
                description: "Chemical toxicity or biological contamination risks".to_string(),
                category: RiskCategory::Safety,
            },
            RiskSubcategory {
                name: "misdiagnosis_risk".to_string(),
                description: "Risk of incorrect diagnosis or delayed treatment".to_string(),
                category: RiskCategory::Safety,
            },
        ];
        
        // Security subcategories  
        let security_subcategories = vec![
            RiskSubcategory {
                name: "data_breach".to_string(),
                description: "Unauthorized access to patient health information".to_string(),
                category: RiskCategory::Security,
            },
            RiskSubcategory {
                name: "device_tampering".to_string(),
                description: "Unauthorized modification of device functionality".to_string(),
                category: RiskCategory::Security,
            },
            RiskSubcategory {
                name: "network_vulnerability".to_string(),
                description: "Network-based attacks on connected devices".to_string(),
                category: RiskCategory::Security,
            },
            RiskSubcategory {
                name: "authentication_bypass".to_string(),
                description: "Circumvention of user authentication mechanisms".to_string(),
                category: RiskCategory::Security,
            },
        ];
        
        // Performance subcategories
        let performance_subcategories = vec![
            RiskSubcategory {
                name: "response_time_degradation".to_string(),
                description: "Unacceptable delay in device response time".to_string(),
                category: RiskCategory::Performance,
            },
            RiskSubcategory {
                name: "accuracy_deviation".to_string(),
                description: "Measurement or calculation accuracy outside specifications".to_string(),
                category: RiskCategory::Performance,
            },
            RiskSubcategory {
                name: "system_failure".to_string(),
                description: "Complete or partial system failure during operation".to_string(),
                category: RiskCategory::Performance,
            },
            RiskSubcategory {
                name: "data_corruption".to_string(),
                description: "Loss or corruption of critical data".to_string(),
                category: RiskCategory::Performance,
            },
        ];
        
        // Usability subcategories
        let usability_subcategories = vec![
            RiskSubcategory {
                name: "user_error".to_string(),
                description: "Risk of user error due to poor interface design".to_string(),
                category: RiskCategory::Usability,
            },
            RiskSubcategory {
                name: "training_inadequacy".to_string(),
                description: "Insufficient user training leading to misuse".to_string(),
                category: RiskCategory::Usability,
            },
            RiskSubcategory {
                name: "accessibility_barrier".to_string(),
                description: "Device not accessible to users with disabilities".to_string(),
                category: RiskCategory::Usability,
            },
            RiskSubcategory {
                name: "workflow_disruption".to_string(),
                description: "Device disrupts established clinical workflows".to_string(),
                category: RiskCategory::Usability,
            },
        ];
        
        // Environmental subcategories
        let environmental_subcategories = vec![
            RiskSubcategory {
                name: "temperature_sensitivity".to_string(),
                description: "Device performance affected by temperature extremes".to_string(),
                category: RiskCategory::Environmental,
            },
            RiskSubcategory {
                name: "humidity_sensitivity".to_string(),
                description: "Device performance affected by humidity levels".to_string(),
                category: RiskCategory::Environmental,
            },
            RiskSubcategory {
                name: "electromagnetic_interference".to_string(),
                description: "Device susceptible to electromagnetic interference".to_string(),
                category: RiskCategory::Environmental,
            },
            RiskSubcategory {
                name: "disposal_hazard".to_string(),
                description: "Environmental or safety risks during device disposal".to_string(),
                category: RiskCategory::Environmental,
            },
        ];
        
        self.predefined_subcategories.insert(RiskCategory::Safety, safety_subcategories);
        self.predefined_subcategories.insert(RiskCategory::Security, security_subcategories);
        self.predefined_subcategories.insert(RiskCategory::Performance, performance_subcategories);
        self.predefined_subcategories.insert(RiskCategory::Usability, usability_subcategories);
        self.predefined_subcategories.insert(RiskCategory::Environmental, environmental_subcategories);
    }
    
    /// Categorize a risk
    pub fn categorize_risk(
        &mut self,
        risk_id: &str,
        category: RiskCategory,
        subcategory_name: Option<&str>,
        classifications: Vec<ClassificationDimension>,
        user_id: &str,
        rationale: &str,
    ) -> Result<(), String> {
        // Validate subcategory if provided
        let subcategory = if let Some(name) = subcategory_name {
            self.find_subcategory(&category, name)?
        } else {
            None
        };
        
        let categorization = RiskCategorization {
            risk_id: risk_id.to_string(),
            category: category.clone(),
            subcategory,
            classifications,
            tags: Vec::new(),
            assigned_by: user_id.to_string(),
            assigned_date: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string(),
            rationale: rationale.to_string(),
        };
        
        self.categorizations.insert(risk_id.to_string(), categorization);
        
        // Log the categorization
        let _ = log_action(
            user_id,
            AuditAction::Create,
            "risk_categorization",
            risk_id,
        );
        
        Ok(())
    }
    
    /// Find subcategory by name within a category
    fn find_subcategory(&self, category: &RiskCategory, name: &str) -> Result<Option<RiskSubcategory>, String> {
        if let Some(subcategories) = self.predefined_subcategories.get(category) {
            for subcategory in subcategories {
                if subcategory.name == name {
                    return Ok(Some(subcategory.clone()));
                }
            }
            Err(format!("Subcategory '{}' not found in category '{}'", name, category.as_str()))
        } else {
            Err(format!("Category '{}' not found", category.as_str()))
        }
    }
    
    /// Filter risks by category
    pub fn filter_by_category(&self, category: &RiskCategory) -> Vec<String> {
        self.categorizations.iter()
            .filter(|(_, cat)| &cat.category == category)
            .map(|(risk_id, _)| risk_id.clone())
            .collect()
    }
    
    /// Filter risks by classification dimension
    #[allow(dead_code)] // Future feature for advanced filtering
    pub fn filter_by_classification(&self, dimension_type: &str, value: &str) -> Vec<String> {
        self.categorizations.iter()
            .filter(|(_, cat)| {
                cat.classifications.iter().any(|dim| {
                    dim.dimension_type() == dimension_type && dim.value() == value
                })
            })
            .map(|(risk_id, _)| risk_id.clone())
            .collect()
    }
    
    /// Get categorization for a risk
    pub fn get_categorization(&self, risk_id: &str) -> Option<&RiskCategorization> {
        self.categorizations.get(risk_id)
    }
    
    /// Get all predefined subcategories for a category
    #[allow(dead_code)] // API method for frontend integration
    pub fn get_subcategories(&self, category: &RiskCategory) -> Option<&Vec<RiskSubcategory>> {
        self.predefined_subcategories.get(category)
    }
    
    /// Generate category statistics
    pub fn category_statistics(&self, risk_manager: &RiskManager) -> CategoryStatistics {
        let mut category_counts = HashMap::new();
        let mut classification_counts = HashMap::new();
        let total_risks = self.categorizations.len();
        
        // Count risks by category
        for categorization in self.categorizations.values() {
            let category_name = categorization.category.as_str().to_string();
            *category_counts.entry(category_name).or_insert(0) += 1;
            
            // Count by classifications
            for classification in &categorization.classifications {
                let dim_type = classification.dimension_type().to_string();
                let dim_value = classification.value().to_string();
                
                classification_counts.entry(dim_type)
                    .or_insert_with(HashMap::new)
                    .entry(dim_value)
                    .and_modify(|e| *e += 1)
                    .or_insert(1);
            }
        }
        
        // Find high-risk categories (categories with high RPN risks)
        let mut high_risk_categories = Vec::new();
        for category_name in category_counts.keys() {
            // Get risks in this category and check their RPN
            let category = RiskCategory::from_str(category_name).unwrap_or(RiskCategory::Safety);
            let risk_ids = self.filter_by_category(&category);
            
            let high_risk_count = risk_ids.iter()
                .filter_map(|id| risk_manager.load_risk(id).ok())
                .filter(|risk| risk.risk_priority_number > 125) // High risk threshold
                .count();
                
            if high_risk_count > 0 {
                high_risk_categories.push((category_name.clone(), high_risk_count));
            }
        }
        
        // Sort by high risk count
        high_risk_categories.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Calculate coverage analysis (percentage of risks in each category)
        let mut coverage_analysis = HashMap::new();
        for (category_name, count) in &category_counts {
            let coverage = if total_risks > 0 {
                (*count as f64 / total_risks as f64) * 100.0
            } else {
                0.0
            };
            coverage_analysis.insert(category_name.clone(), coverage);
        }
        
        CategoryStatistics {
            total_risks,
            category_counts,
            classification_counts,
            high_risk_categories,
            coverage_analysis,
        }
    }
    
    /// Add tags to a risk categorization
    #[allow(dead_code)] // Tagging system for enhanced organization
    pub fn add_tags(&mut self, risk_id: &str, tags: Vec<String>, user_id: &str) -> Result<(), String> {
        if let Some(categorization) = self.categorizations.get_mut(risk_id) {
            for tag in tags {
                if !categorization.tags.contains(&tag) {
                    categorization.tags.push(tag);
                }
            }
            
            let _ = log_action(
                user_id,
                AuditAction::Update,
                "risk_categorization",
                risk_id,
            );
            
            Ok(())
        } else {
            Err(format!("Risk categorization not found for: {risk_id}"))
        }
    }
    
    /// Update risk classification
    #[allow(dead_code)] // Classification update API for workflow management
    pub fn update_classifications(
        &mut self,
        risk_id: &str,
        classifications: Vec<ClassificationDimension>,
        user_id: &str,
    ) -> Result<(), String> {
        if let Some(categorization) = self.categorizations.get_mut(risk_id) {
            categorization.classifications = classifications;
            
            let _ = log_action(
                user_id,
                AuditAction::Update,
                "risk_categorization",
                risk_id,
            );
            
            Ok(())
        } else {
            Err(format!("Risk categorization not found for: {risk_id}"))
        }
    }
    
    /// Generate categorization report
    pub fn generate_categorization_report(&self, risk_manager: &RiskManager) -> String {
        let stats = self.category_statistics(risk_manager);
        let mut report = String::new();
        
        report.push_str("# Risk Categorization Report\n\n");
        report.push_str(&format!("**Total Categorized Risks**: {}\n\n", stats.total_risks));
        
        // Category distribution
        report.push_str("## Category Distribution\n\n");
        for (category, count) in &stats.category_counts {
            let percentage = stats.coverage_analysis.get(category).unwrap_or(&0.0);
            report.push_str(&format!("- **{category}**: {count} risks ({percentage:.1}%)\n"));
        }
        
        // High-risk categories
        if !stats.high_risk_categories.is_empty() {
            report.push_str("\n## High-Risk Categories\n\n");
            for (category, count) in &stats.high_risk_categories {
                report.push_str(&format!("- **{category}**: {count} high-risk items (RPN > 125)\n"));
            }
        }
        
        // Classification analysis
        report.push_str("\n## Classification Analysis\n\n");
        for (dimension, values) in &stats.classification_counts {
            report.push_str(&format!("### {}\n", dimension.replace('_', " ").to_uppercase()));
            for (value, count) in values {
                report.push_str(&format!("- {value}: {count} risks\n"));
            }
            report.push('\n');
        }
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_risk_category_conversion() {
        assert_eq!(RiskCategory::Safety.as_str(), "safety");
        assert_eq!(RiskCategory::from_str("security").unwrap(), RiskCategory::Security);
        assert!(RiskCategory::from_str("invalid").is_err());
    }
    
    #[test]
    fn test_classification_dimension() {
        let component = ClassificationDimension::Component("power_supply".to_string());
        assert_eq!(component.dimension_type(), "component");
        assert_eq!(component.value(), "power_supply");
    }
    
    #[test]
    fn test_categorization_manager() {
        let mut manager = RiskCategorizationManager::new();
        
        // Test categorization
        let result = manager.categorize_risk(
            "RISK-001",
            RiskCategory::Safety,
            Some("electrical_hazard"),
            vec![ClassificationDimension::Component("power_supply".to_string())],
            "test_user",
            "High voltage component requires safety categorization",
        );
        
        assert!(result.is_ok());
        
        // Test filtering
        let safety_risks = manager.filter_by_category(&RiskCategory::Safety);
        assert_eq!(safety_risks.len(), 1);
        assert_eq!(safety_risks[0], "RISK-001");
        
        // Test getting categorization
        let categorization = manager.get_categorization("RISK-001");
        assert!(categorization.is_some());
        assert_eq!(categorization.unwrap().category, RiskCategory::Safety);
    }
}
