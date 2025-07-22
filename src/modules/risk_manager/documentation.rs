//! Risk Documentation Generation Module
//! Task 3.1.14: Risk Documentation Implementation
//! 
//! This module provides comprehensive documentation generation capabilities for 
//! risk management per ISO 14971 requirements. Generates templates, summaries,
//! and traceability reports required for medical device regulatory compliance.

use crate::prelude::*;
use crate::modules::risk_manager::risk::{RiskManager, RiskItem};
use crate::modules::risk_manager::template_registry::TemplateRegistry; // OCP: Registry pattern
use crate::modules::audit_logger::log_action;
use crate::models::AuditAction;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Documentation manager for risk management templates and reports
/// Generates standardized documentation required for regulatory compliance
/// OCP Enhancement: Uses template registry instead of switch statements
pub struct DocumentationManager {
    #[allow(dead_code)] // Path used for file operations and future features
    project_path: PathBuf,
    risk_manager: RiskManager,
    template_registry: TemplateRegistry, // OCP: Registry for extensible template generation
}

/// Template types available for generation
/// OCP Enhancement: Added Hash and Eq for registry pattern support
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TemplateType {
    RiskAssessment,         // Risk assessment template per ISO 14971
    FMEA,                   // Failure Mode and Effects Analysis template
    RiskManagementPlan,     // Risk management plan template
    ControlEffectiveness,   // Risk control effectiveness template
    PostMarketSurveillance, // Post-market surveillance plan template
}

/// Report types for auto-generation
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Report types for future enhanced reporting features
pub enum ReportType {
    RiskSummary,           // Executive risk summary
    RiskRegister,          // Complete risk register
    FMEATable,             // FMEA analysis table
    TraceabilityMatrix,    // Risk-to-requirement traceability
    ComplianceChecklist,   // ISO 14971 compliance status
    ResidualRiskSummary,   // Summary of residual risks
}

/// Output format for generated documents
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    HTML,     // HTML format for web viewing and conversion
    CSV,      // CSV format for data analysis
    Markdown, // Markdown format for documentation
    JSON,     // JSON format for data exchange
}

/// Template metadata and configuration
#[derive(Debug, Clone)]
#[allow(dead_code)] // Configuration fields for template customization
pub struct TemplateConfig {
    pub title: String,
    pub version: String,
    pub author: String,
    pub organization: String,
    pub device_name: String,
    pub device_version: String,
    pub date_generated: String,
    pub regulatory_basis: Vec<String>,
}

/// Risk traceability entry linking risks to other QMS elements
#[derive(Debug, Clone)]
pub struct TraceabilityEntry {
    pub risk_id: String,
    pub risk_description: String,
    pub requirements: Vec<String>,
    pub design_controls: Vec<String>,
    pub verification_activities: Vec<String>,
    pub validation_activities: Vec<String>,
}

impl DocumentationManager {
    /// Create new documentation manager
    /// OCP Enhancement: Initializes with template registry for extensible template generation
    pub fn new(project_path: &Path) -> QmsResult<Self> {
        let risk_manager = RiskManager::new(project_path)?;
        let template_registry = TemplateRegistry::default(); // OCP: Use registry pattern

        Ok(DocumentationManager {
            project_path: project_path.to_path_buf(),
            risk_manager,
            template_registry, // OCP: Registry enables extension without modification
        })
    }

    /// Generate template document
    /// OCP Enhancement: Uses registry pattern instead of switch statements
    pub fn generate_template(
        &self,
        template_type: TemplateType,
        output_path: &Path,
        config: TemplateConfig,
        format: OutputFormat,
    ) -> QmsResult<()> {
        // OCP: Use registry instead of switch statement - open for extension, closed for modification
        let content = self.template_registry.generate_template(&template_type, &config)?;

        // Format conversion (could also be registry-based in future enhancement)
        let formatted_content = match format {
            OutputFormat::HTML => content,
            OutputFormat::Markdown => self.html_to_markdown(&content)?,
            OutputFormat::CSV => return Err(QmsError::validation_error("CSV format not supported for templates")),
            OutputFormat::JSON => return Err(QmsError::validation_error("JSON format not supported for templates")),
        };

        // Ensure parent directory exists before writing
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        crate::fs_utils::atomic_write(output_path, &formatted_content)?;
        
        // Log document generation
        log_action(
            "system",
            AuditAction::Create,
            "risk_documentation",
            &format!("Generated {} template: {}", 
                template_type.display_name(), 
                output_path.display()
            )
        )?;

        Ok(())
    }

    /// Generate risk summary report
    pub fn generate_risk_summary_report(
        &self,
        output_path: &Path,
        config: TemplateConfig,
        format: OutputFormat,
    ) -> QmsResult<()> {
        let index_entries = self.risk_manager.list_risks(None)?;
        
        // Load full risk items for detailed reporting
        let mut risks = Vec::new();
        for entry in &index_entries {
            if let Ok(risk_item) = self.risk_manager.load_risk(&entry.id) {
                risks.push(risk_item);
            }
        }
        
        let content = match format {
            OutputFormat::HTML => self.generate_risk_summary_html(&risks, &config)?,
            OutputFormat::CSV => self.generate_risk_summary_csv(&risks)?,
            OutputFormat::Markdown => self.generate_risk_summary_markdown(&risks, &config)?,
            OutputFormat::JSON => self.generate_risk_summary_json(&risks)?,
        };

        crate::fs_utils::atomic_write(output_path, &content)?;
        
        // Log report generation
        log_action(
            "system",
            AuditAction::Create,
            "risk_documentation",
            &format!("Generated risk summary report: {}", output_path.display())
        )?;

        Ok(())
    }

    /// Generate FMEA table from risk data
    pub fn generate_fmea_table(
        &self,
        output_path: &Path,
        config: TemplateConfig,
        format: OutputFormat,
    ) -> QmsResult<()> {
        let index_entries = self.risk_manager.list_risks(None)?;
        
        // Load full risk items for detailed reporting
        let mut risks = Vec::new();
        for entry in &index_entries {
            if let Ok(risk_item) = self.risk_manager.load_risk(&entry.id) {
                risks.push(risk_item);
            }
        }
        
        let content = match format {
            OutputFormat::HTML => self.generate_fmea_table_html(&risks, &config)?,
            OutputFormat::CSV => self.generate_fmea_table_csv(&risks)?,
            OutputFormat::Markdown => self.generate_fmea_table_markdown(&risks, &config)?,
            OutputFormat::JSON => self.generate_fmea_table_json(&risks)?,
        };

        crate::fs_utils::atomic_write(output_path, &content)?;
        
        // Log FMEA generation
        log_action(
            "system",
            AuditAction::Create,
            "risk_documentation",
            &format!("Generated FMEA table: {}", output_path.display())
        )?;

        Ok(())
    }

    /// Generate traceability report linking risks to requirements
    pub fn generate_traceability_report(
        &self,
        output_path: &Path,
        config: TemplateConfig,
        format: OutputFormat,
    ) -> QmsResult<()> {
        let traceability_data = self.build_traceability_matrix()?;
        
        let content = match format {
            OutputFormat::HTML => self.generate_traceability_html(&traceability_data, &config)?,
            OutputFormat::CSV => self.generate_traceability_csv(&traceability_data)?,
            OutputFormat::Markdown => self.generate_traceability_markdown(&traceability_data, &config)?,
            OutputFormat::JSON => self.generate_traceability_json(&traceability_data)?,
        };

        crate::fs_utils::atomic_write(output_path, &content)?;
        
        // Log traceability generation
        log_action(
            "system",
            AuditAction::Create,
            "risk_documentation",
            &format!("Generated traceability report: {}", output_path.display())
        )?;

        Ok(())
    }

    /// Generate compliance checklist report
    pub fn generate_compliance_report(
        &self,
        output_path: &Path,
        config: TemplateConfig,
        format: OutputFormat,
    ) -> QmsResult<()> {
        let compliance_data = self.assess_iso14971_compliance()?;
        
        let content = match format {
            OutputFormat::HTML => self.generate_compliance_html(&compliance_data, &config)?,
            OutputFormat::CSV => self.generate_compliance_csv(&compliance_data)?,
            OutputFormat::Markdown => self.generate_compliance_markdown(&compliance_data, &config)?,
            OutputFormat::JSON => self.generate_compliance_json(&compliance_data)?,
        };

        crate::fs_utils::atomic_write(output_path, &content)?;
        
        // Log compliance report generation
        log_action(
            "system",
            AuditAction::Create,
            "risk_documentation",
            &format!("Generated compliance report: {}", output_path.display())
        )?;

        Ok(())
    }

    // Template Generation Methods
    
    /// Generate risk assessment template
    fn generate_risk_assessment_template(&self, config: &TemplateConfig) -> QmsResult<String> {
        let mut html = String::new();
        
        html.push_str(&self.generate_document_header("Risk Assessment", config)?);
        
        html.push_str(r#"
<h2>1. Executive Summary</h2>
<p><em>Provide a high-level summary of the risk assessment including scope, key findings, and overall risk conclusion.</em></p>
<div class="section-content">
    <textarea placeholder="Enter executive summary here..." style="width: 100%; height: 100px;"></textarea>
</div>

<h2>2. Scope and Applicability</h2>
<p><em>Define the scope of this risk assessment, including device components, intended use, and applicable regulations.</em></p>
<div class="section-content">
    <h3>2.1 Device Description</h3>
    <textarea placeholder="Describe the medical device, its intended use, and key specifications..." style="width: 100%; height: 80px;"></textarea>
    
    <h3>2.2 Intended Use and Indications</h3>
    <textarea placeholder="Define intended use, target patient population, and clinical indications..." style="width: 100%; height: 80px;"></textarea>
    
    <h3>2.3 User Profile</h3>
    <textarea placeholder="Describe intended users, their training level, and use environment..." style="width: 100%; height: 80px;"></textarea>
</div>

<h2>3. Risk Management Process</h2>
<p><em>Document the risk management process per ISO 14971 requirements.</em></p>
<div class="section-content">
    <h3>3.1 Risk Analysis Methods</h3>
    <ul>
        <li><input type="checkbox"> Preliminary Hazard Analysis (PHA)</li>
        <li><input type="checkbox"> Failure Mode and Effects Analysis (FMEA)</li>
        <li><input type="checkbox"> Fault Tree Analysis (FTA)</li>
        <li><input type="checkbox"> Risk Assessment Matrix</li>
        <li><input type="checkbox"> Clinical Risk Assessment</li>
        <li><input type="checkbox"> Other: <input type="text" placeholder="Specify method"></li>
    </ul>
    
    <h3>3.2 Risk Acceptability Criteria</h3>
    <table border="1" style="width: 100%; border-collapse: collapse;">
        <tr>
            <th>Risk Level</th>
            <th>RPN Range</th>
            <th>Acceptability</th>
            <th>Required Actions</th>
        </tr>
        <tr>
            <td>Low</td>
            <td contenteditable="true">1-8</td>
            <td>Acceptable</td>
            <td contenteditable="true">No action required</td>
        </tr>
        <tr>
            <td>Medium</td>
            <td contenteditable="true">9-24</td>
            <td>ALARP</td>
            <td contenteditable="true">Risk reduction measures if reasonably practicable</td>
        </tr>
        <tr>
            <td>High</td>
            <td contenteditable="true">25-125</td>
            <td>Unacceptable</td>
            <td contenteditable="true">Mandatory risk reduction measures</td>
        </tr>
    </table>
</div>

<h2>4. Hazard Identification</h2>
<p><em>Systematic identification of potential hazards associated with the device.</em></p>
<div class="section-content">
    <table border="1" style="width: 100%; border-collapse: collapse;">
        <tr>
            <th>Hazard ID</th>
            <th>Hazard Source</th>
            <th>Hazard Description</th>
            <th>Potential Harm</th>
            <th>Harm Severity</th>
        </tr>
        <tr>
            <td contenteditable="true">H-001</td>
            <td contenteditable="true">Electrical</td>
            <td contenteditable="true">Electric shock from exposed conductors</td>
            <td contenteditable="true">Burns, cardiac arrhythmia, death</td>
            <td contenteditable="true">Catastrophic</td>
        </tr>
        <tr>
            <td contenteditable="true">H-002</td>
            <td contenteditable="true">Mechanical</td>
            <td contenteditable="true">Sharp edges causing cuts</td>
            <td contenteditable="true">Lacerations, bleeding</td>
            <td contenteditable="true">Minor</td>
        </tr>
        <!-- Add more rows as needed -->
    </table>
</div>

<h2>5. Risk Analysis</h2>
<p><em>Detailed analysis of identified risks including probability and severity assessment.</em></p>
<div class="section-content">
    <table border="1" style="width: 100%; border-collapse: collapse;">
        <tr>
            <th>Risk ID</th>
            <th>Hazard</th>
            <th>Failure Mode</th>
            <th>Cause</th>
            <th>Effect</th>
            <th>Severity</th>
            <th>Occurrence</th>
            <th>Detection</th>
            <th>RPN</th>
        </tr>
        <tr>
            <td contenteditable="true">R-001</td>
            <td contenteditable="true">H-001</td>
            <td contenteditable="true">Insulation failure</td>
            <td contenteditable="true">Wear, damage, manufacturing defect</td>
            <td contenteditable="true">Electric shock to user</td>
            <td contenteditable="true">5</td>
            <td contenteditable="true">2</td>
            <td contenteditable="true">3</td>
            <td contenteditable="true">30</td>
        </tr>
        <!-- Add more rows as needed -->
    </table>
</div>

<h2>6. Risk Control Measures</h2>
<p><em>Risk reduction and control measures implemented to achieve acceptable risk levels.</em></p>
<div class="section-content">
    <table border="1" style="width: 100%; border-collapse: collapse;">
        <tr>
            <th>Risk ID</th>
            <th>Control Measure</th>
            <th>Control Type</th>
            <th>Implementation</th>
            <th>Verification</th>
            <th>Residual RPN</th>
        </tr>
        <tr>
            <td contenteditable="true">R-001</td>
            <td contenteditable="true">Double insulation system</td>
            <td contenteditable="true">Inherent safety by design</td>
            <td contenteditable="true">Design specification REQ-001</td>
            <td contenteditable="true">Electrical safety testing per IEC 60601-1</td>
            <td contenteditable="true">6</td>
        </tr>
        <!-- Add more rows as needed -->
    </table>
</div>

<h2>7. Residual Risk Evaluation</h2>
<p><em>Assessment of remaining risks after control measures implementation.</em></p>
<div class="section-content">
    <h3>7.1 Overall Residual Risk Summary</h3>
    <textarea placeholder="Provide overall assessment of residual risks and their acceptability..." style="width: 100%; height: 100px;"></textarea>
    
    <h3>7.2 Benefit-Risk Analysis</h3>
    <textarea placeholder="Document benefit-risk analysis per ISO 14971 clause 7..." style="width: 100%; height: 100px;"></textarea>
</div>

<h2>8. Risk Management File</h2>
<p><em>Documentation references and risk management file contents.</em></p>
<div class="section-content">
    <ul>
        <li><input type="checkbox"> Risk Management Plan</li>
        <li><input type="checkbox"> Risk Analysis Records</li>
        <li><input type="checkbox"> Risk Control Implementation Records</li>
        <li><input type="checkbox"> Verification and Validation Reports</li>
        <li><input type="checkbox"> Production and Post-Production Information</li>
        <li><input type="checkbox"> Post-Market Surveillance Data</li>
    </ul>
</div>

<h2>9. Review and Approval</h2>
<div class="section-content">
    <table border="1" style="width: 100%; border-collapse: collapse;">
        <tr>
            <th>Role</th>
            <th>Name</th>
            <th>Signature</th>
            <th>Date</th>
        </tr>
        <tr>
            <td>Risk Manager</td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
        </tr>
        <tr>
            <td>Quality Manager</td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
        </tr>
        <tr>
            <td>Clinical Affairs</td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
        </tr>
    </table>
</div>
"#);

        html.push_str(&self.generate_document_footer()?);
        
        Ok(html)
    }

    /// Generate FMEA template
    fn generate_fmea_template(&self, config: &TemplateConfig) -> QmsResult<String> {
        let mut html = String::new();
        
        html.push_str(&self.generate_document_header("Failure Mode and Effects Analysis (FMEA)", config)?);
        
        html.push_str(r#"
<h2>1. FMEA Overview</h2>
<div class="section-content">
    <h3>1.1 Scope and Objectives</h3>
    <textarea placeholder="Define the scope of this FMEA including system/subsystem boundaries and analysis objectives..." style="width: 100%; height: 80px;"></textarea>
    
    <h3>1.2 FMEA Team</h3>
    <table border="1" style="width: 100%; border-collapse: collapse;">
        <tr>
            <th>Name</th>
            <th>Role/Discipline</th>
            <th>Responsibility</th>
        </tr>
        <tr>
            <td contenteditable="true"></td>
            <td contenteditable="true">Team Leader</td>
            <td contenteditable="true">FMEA coordination and completion</td>
        </tr>
        <tr>
            <td contenteditable="true"></td>
            <td contenteditable="true">Design Engineer</td>
            <td contenteditable="true">Technical design input</td>
        </tr>
        <tr>
            <td contenteditable="true"></td>
            <td contenteditable="true">Quality Engineer</td>
            <td contenteditable="true">Quality and regulatory input</td>
        </tr>
        <tr>
            <td contenteditable="true"></td>
            <td contenteditable="true">Clinical Specialist</td>
            <td contenteditable="true">Clinical use and safety input</td>
        </tr>
    </table>
</div>

<h2>2. Severity Ranking Criteria</h2>
<div class="section-content">
    <table border="1" style="width: 100%; border-collapse: collapse;">
        <tr>
            <th>Ranking</th>
            <th>Severity</th>
            <th>Description</th>
            <th>Clinical Impact</th>
        </tr>
        <tr>
            <td>5</td>
            <td>Catastrophic</td>
            <td contenteditable="true">Death or permanent disability</td>
            <td contenteditable="true">Life-threatening condition</td>
        </tr>
        <tr>
            <td>4</td>
            <td>Critical</td>
            <td contenteditable="true">Serious injury requiring medical intervention</td>
            <td contenteditable="true">Requires immediate medical attention</td>
        </tr>
        <tr>
            <td>3</td>
            <td>Major</td>
            <td contenteditable="true">Moderate injury requiring treatment</td>
            <td contenteditable="true">Delays treatment or diagnosis</td>
        </tr>
        <tr>
            <td>2</td>
            <td>Minor</td>
            <td contenteditable="true">Minor injury, first aid required</td>
            <td contenteditable="true">Inconvenience to patient/user</td>
        </tr>
        <tr>
            <td>1</td>
            <td>Negligible</td>
            <td contenteditable="true">No injury expected</td>
            <td contenteditable="true">No clinical impact</td>
        </tr>
    </table>
</div>

<h2>3. Occurrence Ranking Criteria</h2>
<div class="section-content">
    <table border="1" style="width: 100%; border-collapse: collapse;">
        <tr>
            <th>Ranking</th>
            <th>Occurrence</th>
            <th>Probability Description</th>
            <th>Failure Rate</th>
        </tr>
        <tr>
            <td>5</td>
            <td>Frequent</td>
            <td contenteditable="true">Very likely to occur repeatedly</td>
            <td contenteditable="true">> 1 in 10</td>
        </tr>
        <tr>
            <td>4</td>
            <td>Probable</td>
            <td contenteditable="true">Will occur several times</td>
            <td contenteditable="true">1 in 100 to 1 in 10</td>
        </tr>
        <tr>
            <td>3</td>
            <td>Occasional</td>
            <td contenteditable="true">Likely to occur sometime</td>
            <td contenteditable="true">1 in 1,000 to 1 in 100</td>
        </tr>
        <tr>
            <td>2</td>
            <td>Remote</td>
            <td contenteditable="true">Unlikely but possible</td>
            <td contenteditable="true">1 in 10,000 to 1 in 1,000</td>
        </tr>
        <tr>
            <td>1</td>
            <td>Improbable</td>
            <td contenteditable="true">So unlikely, assumed not to occur</td>
            <td contenteditable="true">< 1 in 10,000</td>
        </tr>
    </table>
</div>

<h2>4. Detection Ranking Criteria</h2>
<div class="section-content">
    <table border="1" style="width: 100%; border-collapse: collapse;">
        <tr>
            <th>Ranking</th>
            <th>Detection</th>
            <th>Description</th>
            <th>Detection Method</th>
        </tr>
        <tr>
            <td>5</td>
            <td>Very Low</td>
            <td contenteditable="true">Cannot detect or no controls in place</td>
            <td contenteditable="true">No detection method available</td>
        </tr>
        <tr>
            <td>4</td>
            <td>Low</td>
            <td contenteditable="true">Poor chance of detection</td>
            <td contenteditable="true">Manual inspection, difficult to detect</td>
        </tr>
        <tr>
            <td>3</td>
            <td>Moderate</td>
            <td contenteditable="true">Moderate chance of detection</td>
            <td contenteditable="true">Some controls, manual testing</td>
        </tr>
        <tr>
            <td>2</td>
            <td>High</td>
            <td contenteditable="true">Good chance of detection</td>
            <td contenteditable="true">Automated monitoring, good controls</td>
        </tr>
        <tr>
            <td>1</td>
            <td>Very High</td>
            <td contenteditable="true">Almost certain detection</td>
            <td contenteditable="true">Fail-safe design, automatic detection</td>
        </tr>
    </table>
</div>

<h2>5. FMEA Analysis Table</h2>
<div class="section-content">
    <table border="1" style="width: 100%; border-collapse: collapse; font-size: 12px;">
        <tr>
            <th>Item/Function</th>
            <th>Failure Mode</th>
            <th>Potential Effects</th>
            <th>Severity (S)</th>
            <th>Potential Causes</th>
            <th>Occurrence (O)</th>
            <th>Current Controls</th>
            <th>Detection (D)</th>
            <th>RPN</th>
            <th>Actions Required</th>
            <th>Responsibility</th>
            <th>Target Date</th>
        </tr>
        <tr>
            <td contenteditable="true">Power supply</td>
            <td contenteditable="true">No output voltage</td>
            <td contenteditable="true">Device does not operate</td>
            <td contenteditable="true">3</td>
            <td contenteditable="true">Component failure, fuse blown</td>
            <td contenteditable="true">2</td>
            <td contenteditable="true">Power indicator LED, self-test</td>
            <td contenteditable="true">2</td>
            <td contenteditable="true">12</td>
            <td contenteditable="true">Add backup power indicator</td>
            <td contenteditable="true">Design team</td>
            <td contenteditable="true">MM/DD/YYYY</td>
        </tr>
        <tr>
            <td contenteditable="true">Alarm system</td>
            <td contenteditable="true">False alarm</td>
            <td contenteditable="true">Unnecessary clinical intervention</td>
            <td contenteditable="true">2</td>
            <td contenteditable="true">Sensor noise, software bug</td>
            <td contenteditable="true">3</td>
            <td contenteditable="true">Alarm verification algorithm</td>
            <td contenteditable="true">3</td>
            <td contenteditable="true">18</td>
            <td contenteditable="true">Improve noise filtering</td>
            <td contenteditable="true">Software team</td>
            <td contenteditable="true">MM/DD/YYYY</td>
        </tr>
        <!-- Add more rows as needed -->
    </table>
</div>

<h2>6. Action Plan Summary</h2>
<div class="section-content">
    <table border="1" style="width: 100%; border-collapse: collapse;">
        <tr>
            <th>Priority</th>
            <th>RPN Range</th>
            <th>Action Required</th>
            <th>Timeline</th>
        </tr>
        <tr>
            <td>High</td>
            <td contenteditable="true">> 100</td>
            <td contenteditable="true">Immediate action required before release</td>
            <td contenteditable="true">Within 30 days</td>
        </tr>
        <tr>
            <td>Medium</td>
            <td contenteditable="true">50-100</td>
            <td contenteditable="true">Action recommended, schedule implementation</td>
            <td contenteditable="true">Within 90 days</td>
        </tr>
        <tr>
            <td>Low</td>
            <td contenteditable="true">< 50</td>
            <td contenteditable="true">Monitor and review in next assessment</td>
            <td contenteditable="true">Next review cycle</td>
        </tr>
    </table>
</div>

<h2>7. FMEA Review and Approval</h2>
<div class="section-content">
    <table border="1" style="width: 100%; border-collapse: collapse;">
        <tr>
            <th>Role</th>
            <th>Name</th>
            <th>Signature</th>
            <th>Date</th>
            <th>Comments</th>
        </tr>
        <tr>
            <td>FMEA Team Leader</td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
        </tr>
        <tr>
            <td>Design Manager</td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
        </tr>
        <tr>
            <td>Quality Manager</td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
        </tr>
    </table>
</div>
"#);

        html.push_str(&self.generate_document_footer()?);
        
        Ok(html)
    }

    /// Generate risk management plan template
    fn generate_risk_management_plan_template(&self, config: &TemplateConfig) -> QmsResult<String> {
        let mut html = String::new();
        
        html.push_str(&self.generate_document_header("Risk Management Plan", config)?);
        
        html.push_str(r#"
<h2>1. Introduction and Scope</h2>
<div class="section-content">
    <h3>1.1 Purpose</h3>
    <p>This Risk Management Plan establishes the framework for systematic identification, analysis, evaluation, and control of risks associated with the medical device throughout its lifecycle, in accordance with ISO 14971:2019.</p>
    
    <h3>1.2 Scope</h3>
    <textarea placeholder="Define the scope of risk management activities, including device boundaries, lifecycle phases, and exclusions..." style="width: 100%; height: 80px;"></textarea>
    
    <h3>1.3 Regulatory Requirements</h3>
    <ul>
        <li><input type="checkbox"> ISO 14971:2019 - Application of risk management to medical devices</li>
        <li><input type="checkbox"> ISO 13485:2016 - Quality management systems for medical devices</li>
        <li><input type="checkbox"> FDA 21 CFR Part 820 - Quality System Regulation</li>
        <li><input type="checkbox"> EU MDR 2017/745 - Medical Device Regulation</li>
        <li><input type="checkbox"> IEC 62304 - Medical device software lifecycle processes</li>
        <li><input type="checkbox"> Other: <input type="text" placeholder="Specify additional requirements"></li>
    </ul>
</div>

<h2>2. Risk Management Organization</h2>
<div class="section-content">
    <h3>2.1 Risk Management Team</h3>
    <table border="1" style="width: 100%; border-collapse: collapse;">
        <tr>
            <th>Role</th>
            <th>Name</th>
            <th>Responsibilities</th>
            <th>Qualifications</th>
        </tr>
        <tr>
            <td>Risk Manager</td>
            <td contenteditable="true"></td>
            <td contenteditable="true">Overall risk management coordination</td>
            <td contenteditable="true">Risk management training, medical device experience</td>
        </tr>
        <tr>
            <td>Design Engineer</td>
            <td contenteditable="true"></td>
            <td contenteditable="true">Technical risk identification and mitigation</td>
            <td contenteditable="true">Engineering degree, design experience</td>
        </tr>
        <tr>
            <td>Clinical Specialist</td>
            <td contenteditable="true"></td>
            <td contenteditable="true">Clinical risk assessment and use-related risks</td>
            <td contenteditable="true">Clinical background, device familiarity</td>
        </tr>
        <tr>
            <td>Quality Engineer</td>
            <td contenteditable="true"></td>
            <td contenteditable="true">Regulatory compliance and quality risks</td>
            <td contenteditable="true">Quality systems experience, regulatory knowledge</td>
        </tr>
    </table>
    
    <h3>2.2 Authorities and Responsibilities</h3>
    <textarea placeholder="Define decision-making authority, escalation procedures, and final approval responsibilities..." style="width: 100%; height: 100px;"></textarea>
</div>

<h2>3. Risk Management Process</h2>
<div class="section-content">
    <h3>3.1 Risk Management Activities</h3>
    <ol>
        <li><strong>Risk Analysis</strong>
            <ul>
                <li>Intended use and reasonably foreseeable misuse identification</li>
                <li>Hazard identification (electrical, mechanical, thermal, etc.)</li>
                <li>Risk estimation (severity and probability assessment)</li>
            </ul>
        </li>
        <li><strong>Risk Evaluation</strong>
            <ul>
                <li>Risk acceptability assessment</li>
                <li>Application of risk acceptability criteria</li>
            </ul>
        </li>
        <li><strong>Risk Control</strong>
            <ul>
                <li>Risk reduction measures implementation</li>
                <li>Risk control verification and validation</li>
                <li>Residual risk assessment</li>
            </ul>
        </li>
        <li><strong>Production and Post-Production</strong>
            <ul>
                <li>Production information review</li>
                <li>Post-market surveillance data analysis</li>
                <li>Risk management file updates</li>
            </ul>
        </li>
    </ol>
    
    <h3>3.2 Risk Assessment Methods</h3>
    <table border="1" style="width: 100%; border-collapse: collapse;">
        <tr>
            <th>Method</th>
            <th>Application</th>
            <th>When Used</th>
            <th>Responsible</th>
        </tr>
        <tr>
            <td>Preliminary Hazard Analysis</td>
            <td contenteditable="true">Initial hazard identification</td>
            <td contenteditable="true">Early design phase</td>
            <td contenteditable="true">Risk team</td>
        </tr>
        <tr>
            <td>FMEA</td>
            <td contenteditable="true">Component and system failure analysis</td>
            <td contenteditable="true">Design and process analysis</td>
            <td contenteditable="true">Design engineers</td>
        </tr>
        <tr>
            <td>Use-related Risk Analysis</td>
            <td contenteditable="true">User interface and usability risks</td>
            <td contenteditable="true">UI design and validation</td>
            <td contenteditable="true">Human factors engineer</td>
        </tr>
        <tr>
            <td>Software Risk Analysis</td>
            <td contenteditable="true">Software safety classification</td>
            <td contenteditable="true">Software development</td>
            <td contenteditable="true">Software engineer</td>
        </tr>
    </table>
</div>

<h2>4. Risk Acceptability Criteria</h2>
<div class="section-content">
    <h3>4.1 Risk Evaluation Matrix</h3>
    <table border="1" style="width: 100%; border-collapse: collapse;">
        <tr>
            <th rowspan="2">Severity</th>
            <th colspan="5">Occurrence</th>
        </tr>
        <tr>
            <th>Improbable (1)</th>
            <th>Remote (2)</th>
            <th>Occasional (3)</th>
            <th>Probable (4)</th>
            <th>Frequent (5)</th>
        </tr>
        <tr>
            <td>Catastrophic (5)</td>
            <td contenteditable="true" style="background-color: #ffcccc;">5</td>
            <td contenteditable="true" style="background-color: #ffcccc;">10</td>
            <td contenteditable="true" style="background-color: #ffcccc;">15</td>
            <td contenteditable="true" style="background-color: #ffcccc;">20</td>
            <td contenteditable="true" style="background-color: #ffcccc;">25</td>
        </tr>
        <tr>
            <td>Critical (4)</td>
            <td contenteditable="true" style="background-color: #ffffcc;">4</td>
            <td contenteditable="true" style="background-color: #ffffcc;">8</td>
            <td contenteditable="true" style="background-color: #ffcccc;">12</td>
            <td contenteditable="true" style="background-color: #ffcccc;">16</td>
            <td contenteditable="true" style="background-color: #ffcccc;">20</td>
        </tr>
        <tr>
            <td>Major (3)</td>
            <td contenteditable="true" style="background-color: #ccffcc;">3</td>
            <td contenteditable="true" style="background-color: #ffffcc;">6</td>
            <td contenteditable="true" style="background-color: #ffffcc;">9</td>
            <td contenteditable="true" style="background-color: #ffcccc;">12</td>
            <td contenteditable="true" style="background-color: #ffcccc;">15</td>
        </tr>
        <tr>
            <td>Minor (2)</td>
            <td contenteditable="true" style="background-color: #ccffcc;">2</td>
            <td contenteditable="true" style="background-color: #ccffcc;">4</td>
            <td contenteditable="true" style="background-color: #ffffcc;">6</td>
            <td contenteditable="true" style="background-color: #ffffcc;">8</td>
            <td contenteditable="true" style="background-color: #ffffcc;">10</td>
        </tr>
        <tr>
            <td>Negligible (1)</td>
            <td contenteditable="true" style="background-color: #ccffcc;">1</td>
            <td contenteditable="true" style="background-color: #ccffcc;">2</td>
            <td contenteditable="true" style="background-color: #ccffcc;">3</td>
            <td contenteditable="true" style="background-color: #ccffcc;">4</td>
            <td contenteditable="true" style="background-color: #ccffcc;">5</td>
        </tr>
    </table>
    
    <h3>4.2 Risk Acceptability Levels</h3>
    <ul>
        <li><span style="color: green;">■</span> <strong>Acceptable (Green)</strong>: Risk Priority Number ≤ 5 - No further action required</li>
        <li><span style="color: orange;">■</span> <strong>ALARP (Yellow)</strong>: Risk Priority Number 6-10 - Risk reduction if reasonably practicable</li>
        <li><span style="color: red;">■</span> <strong>Unacceptable (Red)</strong>: Risk Priority Number > 10 - Mandatory risk reduction required</li>
    </ul>
    
    <h3>4.3 Benefit-Risk Considerations</h3>
    <textarea placeholder="Document the process for benefit-risk analysis when risks remain above acceptable levels..." style="width: 100%; height: 80px;"></textarea>
</div>

<h2>5. Risk Control Hierarchy</h2>
<div class="section-content">
    <ol>
        <li><strong>Inherent Safety by Design</strong>
            <p>Eliminate or reduce risks through design features that prevent hazards</p>
            <textarea placeholder="Examples: fail-safe mechanisms, redundant systems, physical barriers..." style="width: 100%; height: 60px;"></textarea>
        </li>
        <li><strong>Protective Measures in the Device</strong>
            <p>Add safety features to detect and control hazardous situations</p>
            <textarea placeholder="Examples: alarms, automatic shutoffs, monitoring systems..." style="width: 100%; height: 60px;"></textarea>
        </li>
        <li><strong>Information for Safety</strong>
            <p>Provide warnings, contraindications, and usage instructions</p>
            <textarea placeholder="Examples: warning labels, user training, operator manuals..." style="width: 100%; height: 60px;"></textarea>
        </li>
    </ol>
</div>

<h2>6. Verification and Validation Activities</h2>
<div class="section-content">
    <table border="1" style="width: 100%; border-collapse: collapse;">
        <tr>
            <th>Risk Control Type</th>
            <th>Verification Method</th>
            <th>Validation Method</th>
            <th>Acceptance Criteria</th>
        </tr>
        <tr>
            <td>Design Controls</td>
            <td contenteditable="true">Design review, testing</td>
            <td contenteditable="true">Clinical evaluation</td>
            <td contenteditable="true">Meets safety requirements</td>
        </tr>
        <tr>
            <td>Protective Measures</td>
            <td contenteditable="true">Functional testing</td>
            <td contenteditable="true">Use simulation</td>
            <td contenteditable="true">Operates as intended</td>
        </tr>
        <tr>
            <td>Information for Safety</td>
            <td contenteditable="true">Document review</td>
            <td contenteditable="true">Usability testing</td>
            <td contenteditable="true">User comprehension</td>
        </tr>
    </table>
</div>

<h2>7. Production and Post-Production Activities</h2>
<div class="section-content">
    <h3>7.1 Production Information Review</h3>
    <ul>
        <li><input type="checkbox"> Manufacturing process validation</li>
        <li><input type="checkbox"> In-process quality control data</li>
        <li><input type="checkbox"> Final product testing results</li>
        <li><input type="checkbox"> Supplier quality data</li>
        <li><input type="checkbox"> Customer complaints</li>
        <li><input type="checkbox"> Return and repair data</li>
    </ul>
    
    <h3>7.2 Post-Market Surveillance</h3>
    <textarea placeholder="Define post-market surveillance plan including data sources, review frequency, and escalation triggers..." style="width: 100%; height: 100px;"></textarea>
    
    <h3>7.3 Risk Management File Updates</h3>
    <textarea placeholder="Describe the process for updating the risk management file based on production and post-production information..." style="width: 100%; height: 80px;"></textarea>
</div>

<h2>8. Review and Approval</h2>
<div class="section-content">
    <table border="1" style="width: 100%; border-collapse: collapse;">
        <tr>
            <th>Role</th>
            <th>Name</th>
            <th>Signature</th>
            <th>Date</th>
        </tr>
        <tr>
            <td>Risk Manager</td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
        </tr>
        <tr>
            <td>Design Manager</td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
        </tr>
        <tr>
            <td>Quality Manager</td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
        </tr>
        <tr>
            <td>General Manager</td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
            <td contenteditable="true"></td>
        </tr>
    </table>
</div>
"#);

        html.push_str(&self.generate_document_footer()?);
        
        Ok(html)
    }

    /// Generate control effectiveness template  
    fn generate_control_effectiveness_template(&self, config: &TemplateConfig) -> QmsResult<String> {
        let mut html = String::new();
        
        html.push_str(&self.generate_document_header("Risk Control Effectiveness Assessment", config)?);
        
        html.push_str(r#"
<h2>1. Assessment Overview</h2>
<div class="section-content">
    <p>This document evaluates the effectiveness of implemented risk control measures in reducing identified risks to acceptable levels per ISO 14971 requirements.</p>
    
    <h3>1.1 Assessment Scope</h3>
    <textarea placeholder="Define the scope of control effectiveness assessment..." style="width: 100%; height: 80px;"></textarea>
</div>

<h2>2. Risk Control Effectiveness Analysis</h2>
<div class="section-content">
    <table border="1" style="width: 100%; border-collapse: collapse;">
        <tr>
            <th>Risk ID</th>
            <th>Initial RPN</th>
            <th>Control Measures</th>
            <th>Implementation Status</th>
            <th>Verification Results</th>
            <th>Residual RPN</th>
            <th>Effectiveness</th>
        </tr>
        <tr>
            <td contenteditable="true">R-001</td>
            <td contenteditable="true">30</td>
            <td contenteditable="true">Double insulation system</td>
            <td contenteditable="true">Complete</td>
            <td contenteditable="true">Passed electrical safety testing</td>
            <td contenteditable="true">6</td>
            <td contenteditable="true">Effective</td>
        </tr>
        <!-- Add more rows as needed -->
    </table>
</div>
"#);

        html.push_str(&self.generate_document_footer()?);
        
        Ok(html)
    }

    /// Generate surveillance plan template
    fn generate_surveillance_plan_template(&self, config: &TemplateConfig) -> QmsResult<String> {
        let mut html = String::new();
        
        html.push_str(&self.generate_document_header("Post-Market Surveillance Plan", config)?);
        
        html.push_str(r#"
<h2>1. Surveillance Plan Overview</h2>
<div class="section-content">
    <p>This plan establishes systematic post-market surveillance activities to monitor device performance and identify emerging risks per ISO 14971 clause 9.</p>
    
    <h3>1.1 Objectives</h3>
    <ul>
        <li>Monitor device performance in real-world use</li>
        <li>Identify emerging risks not discovered during development</li>
        <li>Validate risk control effectiveness</li>
        <li>Support regulatory compliance requirements</li>
    </ul>
</div>

<h2>2. Data Collection Plan</h2>
<div class="section-content">
    <table border="1" style="width: 100%; border-collapse: collapse;">
        <tr>
            <th>Data Source</th>
            <th>Collection Method</th>
            <th>Frequency</th>
            <th>Responsible Party</th>
        </tr>
        <tr>
            <td contenteditable="true">Customer complaints</td>
            <td contenteditable="true">Support system database</td>
            <td contenteditable="true">Continuous</td>
            <td contenteditable="true">Customer service</td>
        </tr>
        <tr>
            <td contenteditable="true">Field service reports</td>
            <td contenteditable="true">Service database</td>
            <td contenteditable="true">Monthly</td>
            <td contenteditable="true">Field service team</td>
        </tr>
        <tr>
            <td contenteditable="true">Adverse event reports</td>
            <td contenteditable="true">Regulatory reporting system</td>
            <td contenteditable="true">Immediate</td>
            <td contenteditable="true">Regulatory affairs</td>
        </tr>
    </table>
</div>
"#);

        html.push_str(&self.generate_document_footer()?);
        
        Ok(html)
    }

    // Report Generation Methods

    /// Generate risk summary HTML report
    fn generate_risk_summary_html(&self, risks: &[RiskItem], config: &TemplateConfig) -> QmsResult<String> {
        let mut html = String::new();
        
        html.push_str(&self.generate_document_header("Risk Summary Report", config)?);
        
        // Calculate summary statistics
        let total_risks = risks.len();
        let high_risks = risks.iter().filter(|r| r.risk_priority_number > 50).count();
        let medium_risks = risks.iter().filter(|r| r.risk_priority_number > 20 && r.risk_priority_number <= 50).count();
        let low_risks = risks.iter().filter(|r| r.risk_priority_number <= 20).count();
        
        html.push_str(&format!(r#"
<h2>Executive Summary</h2>
<div class="section-content">
    <p>This report provides a comprehensive summary of {} identified risks for the device.</p>
    
    <h3>Risk Distribution</h3>
    <table border="1" style="width: 50%; border-collapse: collapse;">
        <tr>
            <th>Risk Level</th>
            <th>Count</th>
            <th>Percentage</th>
        </tr>
        <tr style="background-color: #ffcccc;">
            <td>High (RPN > 50)</td>
            <td>{}</td>
            <td>{:.1}%</td>
        </tr>
        <tr style="background-color: #ffffcc;">
            <td>Medium (RPN 21-50)</td>
            <td>{}</td>
            <td>{:.1}%</td>
        </tr>
        <tr style="background-color: #ccffcc;">
            <td>Low (RPN ≤ 20)</td>
            <td>{}</td>
            <td>{:.1}%</td>
        </tr>
        <tr>
            <td><strong>Total</strong></td>
            <td><strong>{}</strong></td>
            <td><strong>100.0%</strong></td>
        </tr>
    </table>
</div>

<h2>Detailed Risk List</h2>
<div class="section-content">
    <table border="1" style="width: 100%; border-collapse: collapse;">
        <tr>
            <th>Risk ID</th>
            <th>Description</th>
            <th>Severity</th>
            <th>Occurrence</th>
            <th>Detection</th>
            <th>RPN</th>
            <th>Status</th>
            <th>Mitigation</th>
        </tr>
"#, 
            total_risks,
            high_risks, 
            if total_risks > 0 { high_risks as f64 / total_risks as f64 * 100.0 } else { 0.0 },
            medium_risks,
            if total_risks > 0 { medium_risks as f64 / total_risks as f64 * 100.0 } else { 0.0 },
            low_risks,
            if total_risks > 0 { low_risks as f64 / total_risks as f64 * 100.0 } else { 0.0 },
            total_risks
        ));

        // Add risk details
        for risk in risks {
            let row_color = if risk.risk_priority_number > 50 {
                "background-color: #ffcccc;"
            } else if risk.risk_priority_number > 20 {
                "background-color: #ffffcc;"
            } else {
                "background-color: #ccffcc;"
            };

            html.push_str(&format!(r#"
        <tr style="{}">
            <td>{}</td>
            <td>{}</td>
            <td>{:?} ({})</td>
            <td>{:?} ({})</td>
            <td>{:?} ({})</td>
            <td>{}</td>
            <td>{:?}</td>
            <td>{}</td>
        </tr>
"#, 
                row_color,
                risk.id,
                risk.hazard_description,
                risk.severity, risk.severity.clone() as u32,
                risk.occurrence, risk.occurrence.clone() as u32,
                risk.detectability, risk.detectability.clone() as u32,
                risk.risk_priority_number,
                risk.risk_status,
                risk.mitigation_measures.iter().map(|m| m.description.as_str()).collect::<Vec<_>>().join("; ")
            ));
        }

        html.push_str("    </table>\n</div>\n");
        html.push_str(&self.generate_document_footer()?);
        
        Ok(html)
    }

    /// Generate risk summary CSV
    fn generate_risk_summary_csv(&self, risks: &[RiskItem]) -> QmsResult<String> {
        let mut csv = String::new();
        
        csv.push_str("Risk ID,Description,Severity,Occurrence,Detection,RPN,Status,Mitigation\n");
        
        for risk in risks {
            csv.push_str(&format!("{},{},{:?},{:?},{:?},{},{:?},\"{}\"\n",
                risk.id,
                risk.hazard_description.replace(',', " "),
                risk.severity,
                risk.occurrence,
                risk.detectability,
                risk.risk_priority_number,
                risk.risk_status,
                risk.mitigation_measures.iter().map(|m| m.description.as_str()).collect::<Vec<_>>().join("; ").replace('"', "\"\"")
            ));
        }
        
        Ok(csv)
    }

    /// Generate risk summary markdown
    fn generate_risk_summary_markdown(&self, risks: &[RiskItem], config: &TemplateConfig) -> QmsResult<String> {
        let mut md = String::new();
        
        md.push_str("# Risk Summary Report\n\n");
        md.push_str(&format!("**Device:** {}\n", config.device_name));
        md.push_str(&format!("**Version:** {}\n", config.device_version));
        md.push_str(&format!("**Generated:** {}\n\n", config.date_generated));
        
        md.push_str("## Risk Distribution\n\n");
        md.push_str("| Risk Level | Count | Percentage |\n");
        md.push_str("|------------|-------|------------|\n");
        
        let total_risks = risks.len();
        let high_risks = risks.iter().filter(|r| r.risk_priority_number > 50).count();
        let medium_risks = risks.iter().filter(|r| r.risk_priority_number > 20 && r.risk_priority_number <= 50).count();
        let low_risks = risks.iter().filter(|r| r.risk_priority_number <= 20).count();
        
        md.push_str(&format!("| High (RPN > 50) | {} | {:.1}% |\n", 
            high_risks,
            if total_risks > 0 { high_risks as f64 / total_risks as f64 * 100.0 } else { 0.0 }
        ));
        md.push_str(&format!("| Medium (RPN 21-50) | {} | {:.1}% |\n", 
            medium_risks,
            if total_risks > 0 { medium_risks as f64 / total_risks as f64 * 100.0 } else { 0.0 }
        ));
        md.push_str(&format!("| Low (RPN ≤ 20) | {} | {:.1}% |\n\n", 
            low_risks,
            if total_risks > 0 { low_risks as f64 / total_risks as f64 * 100.0 } else { 0.0 }
        ));
        
        md.push_str("## Detailed Risk List\n\n");
        md.push_str("| Risk ID | Description | Severity | Occurrence | Detection | RPN | Status |\n");
        md.push_str("|---------|-------------|----------|------------|-----------|-----|--------|\n");
        
        for risk in risks {
            md.push_str(&format!("| {} | {} | {:?} | {:?} | {:?} | {} | {:?} |\n",
                risk.id,
                risk.hazard_description,
                risk.severity,
                risk.occurrence,
                risk.detectability,
                risk.risk_priority_number,
                risk.risk_status
            ));
        }
        
        Ok(md)
    }

    /// Generate risk summary JSON
    fn generate_risk_summary_json(&self, risks: &[RiskItem]) -> QmsResult<String> {
        // Manual JSON serialization to avoid external dependencies
        let mut json = String::new();
        json.push_str("{\n");
        json.push_str(&format!("  \"total_risks\": {},\n", risks.len()));
        json.push_str("  \"risks\": [\n");
        
        for (i, risk) in risks.iter().enumerate() {
            json.push_str("    {\n");
            json.push_str(&format!("      \"id\": \"{}\",\n", risk.id));
            json.push_str(&format!("      \"description\": \"{}\",\n", risk.hazard_description.replace('"', "\\\"")));
            json.push_str(&format!("      \"severity\": \"{:?}\",\n", risk.severity));
            json.push_str(&format!("      \"occurrence\": \"{:?}\",\n", risk.occurrence));
            json.push_str(&format!("      \"detectability\": \"{:?}\",\n", risk.detectability));
            json.push_str(&format!("      \"rpn\": {},\n", risk.risk_priority_number));
            json.push_str(&format!("      \"status\": \"{:?}\"\n", risk.risk_status));
            
            if i < risks.len() - 1 {
                json.push_str("    },\n");
            } else {
                json.push_str("    }\n");
            }
        }
        
        json.push_str("  ]\n");
        json.push_str("}\n");
        
        Ok(json)
    }

    // FMEA Table Generation Methods

    /// Generate FMEA table HTML
    fn generate_fmea_table_html(&self, risks: &[RiskItem], config: &TemplateConfig) -> QmsResult<String> {
        let mut html = String::new();
        
        html.push_str(&self.generate_document_header("FMEA Analysis Table", config)?);
        
        html.push_str(r#"
<h2>FMEA Analysis Results</h2>
<div class="section-content">
    <table border="1" style="width: 100%; border-collapse: collapse; font-size: 12px;">
        <tr>
            <th>Item/Function</th>
            <th>Failure Mode</th>
            <th>Potential Effects</th>
            <th>Severity</th>
            <th>Potential Causes</th>
            <th>Occurrence</th>
            <th>Current Controls</th>
            <th>Detection</th>
            <th>RPN</th>
            <th>Risk Level</th>
        </tr>
"#);

        for risk in risks {
            let risk_level = if risk.risk_priority_number > 50 {
                "High"
            } else if risk.risk_priority_number > 20 {
                "Medium"  
            } else {
                "Low"
            };

            let row_color = if risk.risk_priority_number > 50 {
                "background-color: #ffcccc;"
            } else if risk.risk_priority_number > 20 {
                "background-color: #ffffcc;"
            } else {
                "background-color: #ccffcc;"
            };

            html.push_str(&format!(r#"
        <tr style="{}">
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
        </tr>
"#,
                row_color,
                "Component", // Generic component value
                risk.hazardous_situation,
                risk.hazard_description,
                risk.severity.clone() as u32,
                risk.harm, // Using harm as causes
                risk.occurrence.clone() as u32,
                risk.mitigation_measures.iter().map(|m| m.description.as_str()).collect::<Vec<_>>().join("; "), // existing controls
                risk.detectability.clone() as u32,
                risk.risk_priority_number,
                risk_level
            ));
        }

        html.push_str("    </table>\n</div>\n");
        html.push_str(&self.generate_document_footer()?);
        
        Ok(html)
    }

    /// Generate FMEA table CSV
    fn generate_fmea_table_csv(&self, risks: &[RiskItem]) -> QmsResult<String> {
        let mut csv = String::new();
        
        csv.push_str("Item/Function,Failure Mode,Potential Effects,Severity,Potential Causes,Occurrence,Current Controls,Detection,RPN,Risk Level\n");
        
        for risk in risks {
            let risk_level = if risk.risk_priority_number > 50 { "High" } 
                           else if risk.risk_priority_number > 20 { "Medium" } 
                           else { "Low" };

            csv.push_str(&format!("{},{},{},{},{},{},{},{},{},{}\n",
                "Component", // Generic component
                risk.hazardous_situation.replace(',', " "),
                risk.hazard_description.replace(',', " "),
                risk.severity.clone() as u32,
                risk.harm.replace(',', " "), // Using harm as causes
                risk.occurrence.clone() as u32,
                risk.mitigation_measures.iter().map(|m| m.description.as_str()).collect::<Vec<_>>().join("; ").replace(',', " "), // existing controls
                risk.detectability.clone() as u32,
                risk.risk_priority_number,
                risk_level
            ));
        }
        
        Ok(csv)
    }

    /// Generate FMEA table markdown
    fn generate_fmea_table_markdown(&self, risks: &[RiskItem], config: &TemplateConfig) -> QmsResult<String> {
        let mut md = String::new();
        
        md.push_str("# FMEA Analysis Table\n\n");
        md.push_str(&format!("**Device:** {}\n", config.device_name));
        md.push_str(&format!("**Generated:** {}\n\n", config.date_generated));
        
        md.push_str("| Item/Function | Failure Mode | Effects | S | Causes | O | Controls | D | RPN | Level |\n");
        md.push_str("|---------------|--------------|---------|---|--------|---|----------|---|-----|-------|\n");
        
        for risk in risks {
            let risk_level = if risk.risk_priority_number > 50 { "High" } 
                           else if risk.risk_priority_number > 20 { "Medium" } 
                           else { "Low" };

            md.push_str(&format!("| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
                "Component", // Generic component
                risk.hazardous_situation,
                risk.hazard_description,
                risk.severity.clone() as u32,
                risk.harm, // Using harm as causes
                risk.occurrence.clone() as u32,
                risk.mitigation_measures.iter().map(|m| m.description.as_str()).collect::<Vec<_>>().join("; "), // existing controls
                risk.detectability.clone() as u32,
                risk.risk_priority_number,
                risk_level
            ));
        }
        
        Ok(md)
    }

    /// Generate FMEA table JSON
    fn generate_fmea_table_json(&self, risks: &[RiskItem]) -> QmsResult<String> {
        let mut json = String::new();
        json.push_str("{\n");
        json.push_str("  \"fmea_analysis\": [\n");
        
        for (i, risk) in risks.iter().enumerate() {
            let risk_level = if risk.risk_priority_number > 50 { "High" } 
                           else if risk.risk_priority_number > 20 { "Medium" } 
                           else { "Low" };

            json.push_str("    {\n");
            json.push_str(&format!("      \"item_function\": \"{}\",\n", 
                "Component")); // Generic component
            json.push_str(&format!("      \"failure_mode\": \"{}\",\n", 
                risk.hazardous_situation.replace('"', "\\\"")));
            json.push_str(&format!("      \"potential_effects\": \"{}\",\n", 
                risk.hazard_description.replace('"', "\\\"")));
            json.push_str(&format!("      \"severity\": {},\n", risk.severity.clone() as u32));
            json.push_str(&format!("      \"potential_causes\": \"{}\",\n", 
                risk.harm.replace('"', "\\\"")));
            json.push_str(&format!("      \"occurrence\": {},\n", risk.occurrence.clone() as u32));
            json.push_str(&format!("      \"current_controls\": \"{}\",\n", 
                risk.mitigation_measures.iter().map(|m| m.description.as_str()).collect::<Vec<_>>().join("; ").replace('"', "\\\"")));
            json.push_str(&format!("      \"detection\": {},\n", risk.detectability.clone() as u32));
            json.push_str(&format!("      \"rpn\": {},\n", risk.risk_priority_number));
            json.push_str(&format!("      \"risk_level\": \"{risk_level}\"\n"));
            
            if i < risks.len() - 1 {
                json.push_str("    },\n");
            } else {
                json.push_str("    }\n");
            }
        }
        
        json.push_str("  ]\n");
        json.push_str("}\n");
        
        Ok(json)
    }

    // Traceability Methods

    /// Build traceability matrix data
    fn build_traceability_matrix(&self) -> QmsResult<Vec<TraceabilityEntry>> {
        let index_entries = self.risk_manager.list_risks(None)?;
        let mut traceability_data = Vec::new();
        
        // Load full risk items for detailed traceability
        for entry in &index_entries {
            if let Ok(risk) = self.risk_manager.load_risk(&entry.id) {
                let entry = TraceabilityEntry {
                    risk_id: risk.id.clone(),
                    risk_description: risk.hazard_description.clone(),
                    requirements: risk.regulatory_references.clone(),
                    design_controls: risk.mitigation_measures.iter().map(|m| m.description.clone()).collect(),
                    verification_activities: vec![risk.verification_method.clone()],
                    validation_activities: risk.verification_evidence.clone(),
                };
                traceability_data.push(entry);
            }
        }
        
        Ok(traceability_data)
    }

    /// Generate traceability HTML report
    fn generate_traceability_html(&self, data: &[TraceabilityEntry], config: &TemplateConfig) -> QmsResult<String> {
        let mut html = String::new();
        
        html.push_str(&self.generate_document_header("Risk Traceability Matrix", config)?);
        
        html.push_str(r#"
<h2>Risk-to-Requirement Traceability</h2>
<div class="section-content">
    <p>This matrix demonstrates traceability between identified risks and related QMS elements including requirements, design controls, and verification activities.</p>
    
    <table border="1" style="width: 100%; border-collapse: collapse;">
        <tr>
            <th>Risk ID</th>
            <th>Risk Description</th>
            <th>Requirements</th>
            <th>Design Controls</th>
            <th>Verification</th>
            <th>Validation</th>
        </tr>
"#);

        for entry in data {
            html.push_str(&format!(r#"
        <tr>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
        </tr>
"#,
                entry.risk_id,
                entry.risk_description,
                entry.requirements.join("<br>"),
                entry.design_controls.join("<br>"),
                entry.verification_activities.join("<br>"),
                entry.validation_activities.join("<br>")
            ));
        }

        html.push_str("    </table>\n</div>\n");
        html.push_str(&self.generate_document_footer()?);
        
        Ok(html)
    }

    /// Generate traceability CSV
    fn generate_traceability_csv(&self, data: &[TraceabilityEntry]) -> QmsResult<String> {
        let mut csv = String::new();
        
        csv.push_str("Risk ID,Risk Description,Requirements,Design Controls,Verification,Validation\n");
        
        for entry in data {
            csv.push_str(&format!("{},{},{},{},{},{}\n",
                entry.risk_id,
                entry.risk_description.replace(',', " "),
                entry.requirements.join("; ").replace(',', " "),
                entry.design_controls.join("; ").replace(',', " "),
                entry.verification_activities.join("; ").replace(',', " "),
                entry.validation_activities.join("; ").replace(',', " ")
            ));
        }
        
        Ok(csv)
    }

    /// Generate traceability markdown
    fn generate_traceability_markdown(&self, data: &[TraceabilityEntry], config: &TemplateConfig) -> QmsResult<String> {
        let mut md = String::new();
        
        md.push_str("# Risk Traceability Matrix\n\n");
        md.push_str(&format!("**Device:** {}\n", config.device_name));
        md.push_str(&format!("**Generated:** {}\n\n", config.date_generated));
        
        md.push_str("| Risk ID | Description | Requirements | Design Controls | Verification | Validation |\n");
        md.push_str("|---------|-------------|--------------|-----------------|--------------|------------|\n");
        
        for entry in data {
            md.push_str(&format!("| {} | {} | {} | {} | {} | {} |\n",
                entry.risk_id,
                entry.risk_description,
                entry.requirements.join("; "),
                entry.design_controls.join("; "),
                entry.verification_activities.join("; "),
                entry.validation_activities.join("; ")
            ));
        }
        
        Ok(md)
    }

    /// Generate traceability JSON
    fn generate_traceability_json(&self, data: &[TraceabilityEntry]) -> QmsResult<String> {
        let mut json = String::new();
        json.push_str("{\n");
        json.push_str("  \"traceability_matrix\": [\n");
        
        for (i, entry) in data.iter().enumerate() {
            json.push_str("    {\n");
            json.push_str(&format!("      \"risk_id\": \"{}\",\n", entry.risk_id));
            json.push_str(&format!("      \"description\": \"{}\",\n", entry.risk_description.replace('"', "\\\"")));
            json.push_str("      \"requirements\": [\n");
            for (j, req) in entry.requirements.iter().enumerate() {
                if j < entry.requirements.len() - 1 {
                    json.push_str(&format!("        \"{}\",\n", req.replace('"', "\\\"")));
                } else {
                    json.push_str(&format!("        \"{}\"\n", req.replace('"', "\\\"")));
                }
            }
            json.push_str("      ],\n");
            
            json.push_str("      \"design_controls\": [\n");
            for (j, control) in entry.design_controls.iter().enumerate() {
                if j < entry.design_controls.len() - 1 {
                    json.push_str(&format!("        \"{}\",\n", control.replace('"', "\\\"")));
                } else {
                    json.push_str(&format!("        \"{}\"\n", control.replace('"', "\\\"")));
                }
            }
            json.push_str("      ]\n");
            
            if i < data.len() - 1 {
                json.push_str("    },\n");
            } else {
                json.push_str("    }\n");
            }
        }
        
        json.push_str("  ]\n");
        json.push_str("}\n");
        
        Ok(json)
    }

    // Compliance Assessment Methods

    /// Assess ISO 14971 compliance
    fn assess_iso14971_compliance(&self) -> QmsResult<HashMap<String, bool>> {
        let mut compliance = HashMap::new();
        
        // Basic compliance checks based on available data
        let index_entries = self.risk_manager.list_risks(None)?;
        
        // Load full risk items for detailed compliance checking
        let mut risks = Vec::new();
        for entry in &index_entries {
            if let Ok(risk_item) = self.risk_manager.load_risk(&entry.id) {
                risks.push(risk_item);
            }
        }
        
        compliance.insert("4.1_Risk_Management_Plan".to_string(), true); // Assume plan exists
        compliance.insert("4.2_Risk_Management_File".to_string(), !risks.is_empty());
        compliance.insert("5.2_Intended_Use_Identification".to_string(), !risks.is_empty());
        compliance.insert("5.3_Risk_Identification".to_string(), !risks.is_empty());
        compliance.insert("5.4_Risk_Estimation".to_string(), risks.iter().all(|r| r.risk_priority_number > 0));
        compliance.insert("6.1_Risk_Evaluation".to_string(), risks.iter().any(|r| r.initial_risk_level != r.residual_risk_level));
        compliance.insert("7.1_Risk_Control_Analysis".to_string(), risks.iter().any(|r| !r.mitigation_measures.is_empty()));
        compliance.insert("7.2_Risk_Control_Implementation".to_string(), risks.iter().any(|r| !r.verification_evidence.is_empty()));
        compliance.insert("7.3_Residual_Risk_Evaluation".to_string(), risks.iter().any(|r| r.residual_risk_justification.is_some()));
        compliance.insert("8.1_Risk_Benefit_Analysis".to_string(), risks.iter().any(|r| r.residual_risk_approved));
        
        Ok(compliance)
    }

    /// Generate compliance HTML report
    fn generate_compliance_html(&self, compliance: &HashMap<String, bool>, config: &TemplateConfig) -> QmsResult<String> {
        let mut html = String::new();
        
        html.push_str(&self.generate_document_header("ISO 14971 Compliance Report", config)?);
        
        html.push_str(r#"
<h2>ISO 14971:2019 Compliance Assessment</h2>
<div class="section-content">
    <p>This report assesses compliance with ISO 14971:2019 requirements for risk management of medical devices.</p>
    
    <table border="1" style="width: 100%; border-collapse: collapse;">
        <tr>
            <th>ISO 14971 Clause</th>
            <th>Requirement</th>
            <th>Status</th>
            <th>Evidence</th>
        </tr>
"#);

        let requirements = vec![
            ("4.1", "Risk Management Plan", "Risk management plan document"),
            ("4.2", "Risk Management File", "Risk management file maintenance"),
            ("5.2", "Intended Use Identification", "Intended use and reasonably foreseeable misuse"),
            ("5.3", "Risk Identification", "Systematic hazard identification"),
            ("5.4", "Risk Estimation", "Risk analysis and estimation"),
            ("6.1", "Risk Evaluation", "Risk acceptability evaluation"),
            ("7.1", "Risk Control Analysis", "Risk control option analysis"),
            ("7.2", "Risk Control Implementation", "Risk control measure implementation"),
            ("7.3", "Residual Risk Evaluation", "Residual risk assessment"),
            ("8.1", "Risk Benefit Analysis", "Overall residual risk evaluation"),
        ];

        for (clause, requirement, evidence) in requirements {
            let key = format!("{}_{}",
                clause.replace('.', "_"),
                requirement.replace(' ', "_")
            );
            
            let status = compliance.get(&key).unwrap_or(&false);
            let status_text = if *status { "✅ Compliant" } else { "❌ Non-compliant" };
            let row_color = if *status { "background-color: #ccffcc;" } else { "background-color: #ffcccc;" };

            html.push_str(&format!(r#"
        <tr style="{row_color}">
            <td>{clause}</td>
            <td>{requirement}</td>
            <td>{status_text}</td>
            <td>{evidence}</td>
        </tr>
"#
            ));
        }

        html.push_str("    </table>\n</div>\n");
        html.push_str(&self.generate_document_footer()?);
        
        Ok(html)
    }

    /// Generate compliance CSV
    fn generate_compliance_csv(&self, compliance: &HashMap<String, bool>) -> QmsResult<String> {
        let mut csv = String::new();
        
        csv.push_str("Clause,Requirement,Status,Compliant\n");
        
        let requirements = vec![
            ("4.1", "Risk Management Plan"),
            ("4.2", "Risk Management File"),
            ("5.2", "Intended Use Identification"),
            ("5.3", "Risk Identification"),
            ("5.4", "Risk Estimation"),
            ("6.1", "Risk Evaluation"),
            ("7.1", "Risk Control Analysis"),
            ("7.2", "Risk Control Implementation"),
            ("7.3", "Residual Risk Evaluation"),
            ("8.1", "Risk Benefit Analysis"),
        ];

        for (clause, requirement) in requirements {
            let key = format!("{}_{}",
                clause.replace('.', "_"),
                requirement.replace(' ', "_")
            );
            
            let status = compliance.get(&key).unwrap_or(&false);
            let status_text = if *status { "Compliant" } else { "Non-compliant" };

            csv.push_str(&format!("{clause},{requirement},{status_text},{status}\n"
            ));
        }
        
        Ok(csv)
    }

    /// Generate compliance markdown
    fn generate_compliance_markdown(&self, compliance: &HashMap<String, bool>, config: &TemplateConfig) -> QmsResult<String> {
        let mut md = String::new();
        
        md.push_str("# ISO 14971 Compliance Report\n\n");
        md.push_str(&format!("**Device:** {}\n", config.device_name));
        md.push_str(&format!("**Generated:** {}\n\n", config.date_generated));
        
        md.push_str("| Clause | Requirement | Status |\n");
        md.push_str("|--------|-------------|--------|\n");
        
        let requirements = vec![
            ("4.1", "Risk Management Plan"),
            ("4.2", "Risk Management File"),
            ("5.2", "Intended Use Identification"),
            ("5.3", "Risk Identification"),
            ("5.4", "Risk Estimation"),
            ("6.1", "Risk Evaluation"),
            ("7.1", "Risk Control Analysis"),
            ("7.2", "Risk Control Implementation"),
            ("7.3", "Residual Risk Evaluation"),
            ("8.1", "Risk Benefit Analysis"),
        ];

        for (clause, requirement) in requirements {
            let key = format!("{}_{}",
                clause.replace('.', "_"),
                requirement.replace(' ', "_")
            );
            
            let status = compliance.get(&key).unwrap_or(&false);
            let status_icon = if *status { "✅" } else { "❌" };

            md.push_str(&format!("| {clause} | {requirement} | {status_icon} |\n"
            ));
        }
        
        Ok(md)
    }

    /// Generate compliance JSON
    fn generate_compliance_json(&self, compliance: &HashMap<String, bool>) -> QmsResult<String> {
        let mut json = String::new();
        json.push_str("{\n");
        json.push_str("  \"iso14971_compliance\": {\n");
        
        let mut items: Vec<_> = compliance.iter().collect();
        items.sort_by_key(|&(k, _)| k);
        
        for (i, (key, status)) in items.iter().enumerate() {
            if i < items.len() - 1 {
                json.push_str(&format!("    \"{key}\": {status},\n"));
            } else {
                json.push_str(&format!("    \"{key}\": {status}\n"));
            }
        }
        
        json.push_str("  }\n");
        json.push_str("}\n");
        
        Ok(json)
    }

    // Utility Methods

    /// Generate document header
    fn generate_document_header(&self, title: &str, config: &TemplateConfig) -> QmsResult<String> {
        Ok(format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            line-height: 1.6;
            margin: 40px;
            color: #333;
        }}
        .header {{
            border-bottom: 2px solid #003366;
            padding-bottom: 20px;
            margin-bottom: 30px;
        }}
        .header h1 {{
            color: #003366;
            margin: 0;
        }}
        .header-info {{
            margin-top: 10px;
            color: #666;
        }}
        .section-content {{
            margin: 20px 0;
            padding: 15px;
            border-left: 4px solid #0066cc;
            background-color: #f8f9fa;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
        }}
        th, td {{
            border: 1px solid #ddd;
            padding: 8px;
            text-align: left;
        }}
        th {{
            background-color: #f2f2f2;
            font-weight: bold;
        }}
        textarea, input[type="text"] {{
            font-family: Arial, sans-serif;
            font-size: 14px;
            border: 1px solid #ddd;
            padding: 5px;
        }}
        .footer {{
            margin-top: 50px;
            padding-top: 20px;
            border-top: 1px solid #ddd;
            color: #666;
            font-size: 12px;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>{}</h1>
        <div class="header-info">
            <strong>Device:</strong> {} | <strong>Version:</strong> {} | <strong>Organization:</strong> {}<br>
            <strong>Generated:</strong> {} | <strong>Author:</strong> {}
        </div>
    </div>
"#, 
            title,
            title,
            config.device_name,
            config.device_version,
            config.organization,
            config.date_generated,
            config.author
        ))
    }

    /// Generate document footer
    fn generate_document_footer(&self) -> QmsResult<String> {
        Ok(r#"
    <div class="footer">
        <p>This document was generated by QMS Risk Management System.</p>
        <p>For regulatory compliance, this document should be reviewed and approved by qualified personnel.</p>
    </div>
</body>
</html>
"#.to_string())
    }

    /// Convert HTML to Markdown (basic conversion)
    fn html_to_markdown(&self, html: &str) -> QmsResult<String> {
        // Basic HTML to Markdown conversion
        let mut markdown = html.to_string();
        
        // Remove HTML tags and convert basic elements
        markdown = markdown.replace("<!DOCTYPE html>", "");
        markdown = markdown.replace("<html lang=\"en\">", "");
        markdown = markdown.replace("<head>", "");
        markdown = markdown.replace("</head>", "");
        markdown = markdown.replace("<body>", "");
        markdown = markdown.replace("</body>", "");
        markdown = markdown.replace("</html>", "");
        
        // Remove style block
        if let Some(start) = markdown.find("<style>") {
            if let Some(end) = markdown.find("</style>") {
                markdown = format!("{}{}", &markdown[..start], &markdown[end + 8..]);
            }
        }
        
        // Convert headers
        markdown = markdown.replace("<h1>", "# ");
        markdown = markdown.replace("</h1>", "\n\n");
        markdown = markdown.replace("<h2>", "## ");
        markdown = markdown.replace("</h2>", "\n\n");
        markdown = markdown.replace("<h3>", "### ");
        markdown = markdown.replace("</h3>", "\n\n");
        
        // Convert basic elements
        markdown = markdown.replace("<p>", "");
        markdown = markdown.replace("</p>", "\n\n");
        markdown = markdown.replace("<br>", "\n");
        markdown = markdown.replace("<em>", "*");
        markdown = markdown.replace("</em>", "*");
        markdown = markdown.replace("<strong>", "**");
        markdown = markdown.replace("</strong>", "**");
        
        // Remove div tags
        markdown = markdown.replace("<div class=\"header\">", "");
        markdown = markdown.replace("<div class=\"header-info\">", "");
        markdown = markdown.replace("<div class=\"section-content\">", "");
        markdown = markdown.replace("<div class=\"footer\">", "");
        markdown = markdown.replace("</div>", "");
        
        // Clean up extra whitespace
        while markdown.contains("\n\n\n") {
            markdown = markdown.replace("\n\n\n", "\n\n");
        }
        
        Ok(markdown.trim().to_string())
    }
}

impl TemplateType {
    /// Get display name for template type
    pub const fn display_name(&self) -> &'static str {
        match self {
            TemplateType::RiskAssessment => "Risk Assessment",
            TemplateType::FMEA => "FMEA",
            TemplateType::RiskManagementPlan => "Risk Management Plan",
            TemplateType::ControlEffectiveness => "Control Effectiveness Assessment",
            TemplateType::PostMarketSurveillance => "Post-Market Surveillance Plan",
        }
    }

    /// Parse template type from string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "risk-assessment" | "assessment" => Ok(TemplateType::RiskAssessment),
            "fmea" | "failure-mode" => Ok(TemplateType::FMEA),
            "risk-plan" | "plan" | "risk-management-plan" => Ok(TemplateType::RiskManagementPlan),
            "control-effectiveness" | "effectiveness" => Ok(TemplateType::ControlEffectiveness),
            "surveillance" | "post-market" | "post-market-surveillance" => Ok(TemplateType::PostMarketSurveillance),
            _ => Err(format!("Unknown template type: {s}")),
        }
    }
}

impl OutputFormat {
    /// Parse output format from string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "html" => Ok(OutputFormat::HTML),
            "csv" => Ok(OutputFormat::CSV),
            "markdown" | "md" => Ok(OutputFormat::Markdown),
            "json" => Ok(OutputFormat::JSON),
            _ => Err(format!("Unknown output format: {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    /// Helper function to create a temporary project directory for testing
    fn create_temp_project() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_path = temp_dir.path().join("test_project");
        fs::create_dir_all(&project_path).expect("Failed to create project directory");
        
        // Create required subdirectories
        let risk_dir = project_path.join("risks");
        fs::create_dir_all(&risk_dir).expect("Failed to create risks directory");
        
        (temp_dir, project_path)
    }

    /// Test documentation manager initialization
    #[test]
    fn test_documentation_manager_creation() {
        let (_temp_dir, project_path) = create_temp_project();
        let result = DocumentationManager::new(&project_path);
        assert!(result.is_ok(), "DocumentationManager creation should succeed");
    }

    /// Test template type parsing from string
    #[test]
    fn test_template_type_parsing() {
        // Test valid template types
        assert_eq!(TemplateType::from_str("risk-assessment"), Ok(TemplateType::RiskAssessment));
        assert_eq!(TemplateType::from_str("fmea"), Ok(TemplateType::FMEA));
        assert_eq!(TemplateType::from_str("risk-management-plan"), Ok(TemplateType::RiskManagementPlan));
        assert_eq!(TemplateType::from_str("control-effectiveness"), Ok(TemplateType::ControlEffectiveness));
        assert_eq!(TemplateType::from_str("post-market-surveillance"), Ok(TemplateType::PostMarketSurveillance));
        
        // Test invalid template type
        assert!(TemplateType::from_str("invalid-template").is_err());
    }

    /// Test output format parsing from string
    #[test]
    fn test_output_format_parsing() {
        // Test valid output formats
        assert_eq!(OutputFormat::from_str("html"), Ok(OutputFormat::HTML));
        assert_eq!(OutputFormat::from_str("csv"), Ok(OutputFormat::CSV));
        assert_eq!(OutputFormat::from_str("markdown"), Ok(OutputFormat::Markdown));
        assert_eq!(OutputFormat::from_str("md"), Ok(OutputFormat::Markdown));
        assert_eq!(OutputFormat::from_str("json"), Ok(OutputFormat::JSON));
        
        // Test case insensitive parsing
        assert_eq!(OutputFormat::from_str("HTML"), Ok(OutputFormat::HTML));
        assert_eq!(OutputFormat::from_str("CSV"), Ok(OutputFormat::CSV));
        
        // Test invalid output format
        assert!(OutputFormat::from_str("invalid-format").is_err());
    }

    /// Test risk assessment template generation
    #[test]
    fn test_risk_assessment_template_generation() {
        let (_temp_dir, project_path) = create_temp_project();

        // Initialize audit system for the test - required for template generation logging
        let audit_config = crate::modules::audit_logger::AuditConfig {
            project_path: project_path.to_string_lossy().to_string(),
            retention_days: 30,
            daily_rotation: false,
            max_file_size_mb: 10,
            require_checksums: false,
        };
        let _ = crate::modules::audit_logger::initialize_audit_system(audit_config);

        let manager = DocumentationManager::new(&project_path).expect("Failed to create manager");

        let config = TemplateConfig {
            title: "Risk Assessment Template".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            organization: "Test Organization".to_string(),
            device_name: "Test Device".to_string(),
            device_version: "1.0.0".to_string(),
            date_generated: "2025-01-06".to_string(),
            regulatory_basis: vec!["ISO 14971".to_string()],
        };

        let temp_output = project_path.join("test_template.html");
        let result = manager.generate_template(TemplateType::RiskAssessment, &temp_output, config, OutputFormat::HTML);
        assert!(result.is_ok(), "Risk assessment template generation should succeed: {:?}", result.err());

        // Verify the file was created
        assert!(temp_output.exists(), "Template file should be created");

        // Read and verify content
        let content = std::fs::read_to_string(&temp_output).expect("Should read template file");
        assert!(content.contains("Risk Assessment"), "Template should contain title");
        assert!(content.contains("Test Author"), "Template should contain author");
        assert!(content.contains("ISO 14971"), "Template should reference ISO 14971");
    }

    /// Test FMEA template generation
    #[test]
    fn test_fmea_template_generation() {
        let (_temp_dir, project_path) = create_temp_project();

        // Initialize audit system for the test - required for template generation logging
        let audit_config = crate::modules::audit_logger::AuditConfig {
            project_path: project_path.to_string_lossy().to_string(),
            retention_days: 30,
            daily_rotation: false,
            max_file_size_mb: 10,
            require_checksums: false,
        };
        let _ = crate::modules::audit_logger::initialize_audit_system(audit_config);

        let manager = DocumentationManager::new(&project_path).expect("Failed to create manager");

        let config = TemplateConfig {
            title: "FMEA Analysis Template".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            organization: "Test Organization".to_string(),
            device_name: "Test Device".to_string(),
            device_version: "1.0.0".to_string(),
            date_generated: "2025-01-06".to_string(),
            regulatory_basis: vec!["ISO 14971".to_string()],
        };

        let temp_output = project_path.join("test_fmea.html");
        let result = manager.generate_template(TemplateType::FMEA, &temp_output, config, OutputFormat::HTML);
        assert!(result.is_ok(), "FMEA template generation should succeed");

        // Verify the file was created
        assert!(temp_output.exists(), "FMEA template file should be created");

        // Read and verify content
        let content = std::fs::read_to_string(&temp_output).expect("Should read FMEA template file");
        assert!(content.contains("FMEA"), "Template should contain FMEA title");
        assert!(content.contains("Failure Mode"), "Template should contain FMEA concepts");
        assert!(content.contains("RPN"), "Template should reference RPN");
    }

    /// Test risk management plan template generation
    #[test]
    fn test_risk_management_plan_template() {
        let (_temp_dir, project_path) = create_temp_project();

        // Initialize audit system for the test - required for template generation logging
        let audit_config = crate::modules::audit_logger::AuditConfig {
            project_path: project_path.to_string_lossy().to_string(),
            retention_days: 30,
            daily_rotation: false,
            max_file_size_mb: 10,
            require_checksums: false,
        };
        let _ = crate::modules::audit_logger::initialize_audit_system(audit_config);

        let manager = DocumentationManager::new(&project_path).expect("Failed to create manager");

        let config = TemplateConfig {
            title: "Risk Management Plan".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            organization: "Test Organization".to_string(),
            device_name: "Test Device".to_string(),
            device_version: "1.0.0".to_string(),
            date_generated: "2025-01-06".to_string(),
            regulatory_basis: vec!["ISO 14971".to_string()],
        };

        let temp_output = project_path.join("test_plan.html");
        let result = manager.generate_template(TemplateType::RiskManagementPlan, &temp_output, config, OutputFormat::HTML);
        assert!(result.is_ok(), "Risk management plan template generation should succeed");

        // Verify the file was created
        assert!(temp_output.exists(), "Risk management plan template file should be created");

        // Read and verify content
        let content = std::fs::read_to_string(&temp_output).expect("Should read plan template file");
        assert!(content.contains("Risk Management Plan"), "Template should contain title");
        assert!(content.contains("Risk Management Process"), "Template should contain process info");
        assert!(content.contains("Risk Acceptability"), "Template should contain acceptance criteria");
    }

    /// Test HTML output format generation
    #[test]
    fn test_html_output_format() {
        let (_temp_dir, project_path) = create_temp_project();

        // Initialize audit system for the test
        let audit_config = crate::modules::audit_logger::AuditConfig {
            project_path: project_path.to_string_lossy().to_string(),
            retention_days: 30,
            daily_rotation: false,
            max_file_size_mb: 10,
            require_checksums: false,
        };
        let _ = crate::modules::audit_logger::initialize_audit_system(audit_config);

        let manager = DocumentationManager::new(&project_path).expect("Failed to create manager");

        let config = TemplateConfig {
            title: "Risk Assessment".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            organization: "Test Organization".to_string(),
            device_name: "Test Device".to_string(),
            device_version: "1.0.0".to_string(),
            date_generated: "2025-01-06".to_string(),
            regulatory_basis: vec!["ISO 14971".to_string()],
        };

        let temp_output = project_path.join("test_html.html");
        let result = manager.generate_template(
            TemplateType::RiskAssessment,
            &temp_output,
            config,
            OutputFormat::HTML
        );
        assert!(result.is_ok(), "HTML document generation should succeed");

        // Verify the file was created and contains HTML
        assert!(temp_output.exists(), "HTML template file should be created");
        let content = std::fs::read_to_string(&temp_output).expect("Should read HTML file");
        assert!(content.contains("<html"), "HTML should contain HTML tags");
        assert!(content.contains("<head>"), "HTML should contain head section");
        assert!(content.contains("<body>"), "HTML should contain body section");
        assert!(content.contains("Risk Assessment"), "HTML should contain content");
    }

    /// Test CSV output format generation
    #[test]
    fn test_csv_output_format() {
        let (_temp_dir, project_path) = create_temp_project();
        let manager = DocumentationManager::new(&project_path).expect("Failed to create manager");

        let config = TemplateConfig {
            title: "FMEA Analysis".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            organization: "Test Organization".to_string(),
            device_name: "Test Device".to_string(),
            device_version: "1.0.0".to_string(),
            date_generated: "2025-01-06".to_string(),
            regulatory_basis: vec!["ISO 14971".to_string()],
        };

        let temp_output = project_path.join("test_csv.csv");
        let result = manager.generate_template(
            TemplateType::FMEA,
            &temp_output,
            config,
            OutputFormat::CSV
        );
        // CSV format is not supported for templates, so this should fail
        assert!(result.is_err(), "CSV template generation should fail as it's not supported");
    }

    /// Test JSON output format generation
    #[test]
    fn test_json_output_format() {
        let (_temp_dir, project_path) = create_temp_project();
        let manager = DocumentationManager::new(&project_path).expect("Failed to create manager");

        let config = TemplateConfig {
            title: "Risk Assessment".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            organization: "Test Organization".to_string(),
            device_name: "Test Device".to_string(),
            device_version: "1.0.0".to_string(),
            date_generated: "2025-01-06".to_string(),
            regulatory_basis: vec!["ISO 14971".to_string()],
        };

        let temp_output = project_path.join("test_json.json");
        let result = manager.generate_template(
            TemplateType::RiskAssessment,
            &temp_output,
            config,
            OutputFormat::JSON
        );
        // JSON format is not supported for templates, so this should fail
        assert!(result.is_err(), "JSON template generation should fail as it's not supported");
    }

    /// Test traceability matrix generation
    #[test]
    fn test_traceability_matrix_generation() {
        let (_temp_dir, project_path) = create_temp_project();
        let manager = DocumentationManager::new(&project_path).expect("Failed to create manager");

        let config = TemplateConfig {
            title: "Traceability Matrix".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            organization: "Test Organization".to_string(),
            device_name: "Test Device".to_string(),
            device_version: "1.0.0".to_string(),
            date_generated: "2025-01-06".to_string(),
            regulatory_basis: vec!["ISO 14971".to_string()],
        };

        // Create some test traceability data
        let test_data = vec![];
        let result = manager.generate_traceability_html(&test_data, &config);
        assert!(result.is_ok(), "Traceability matrix generation should succeed");

        let matrix = result.unwrap();
        assert!(matrix.contains("Risk ID"), "Matrix should contain risk ID column");
        assert!(matrix.contains("Requirements"), "Matrix should contain requirements column");
        assert!(matrix.contains("Design Controls"), "Matrix should contain design controls column");
        assert!(matrix.contains("Verification"), "Matrix should contain verification column");
    }

    /// Test template configuration validation
    #[test]
    fn test_template_config_validation() {
        let (_temp_dir, project_path) = create_temp_project();
        let manager = DocumentationManager::new(&project_path).expect("Failed to create manager");

        // Test with empty title (should still work as there's no validation)
        let config_with_empty_title = TemplateConfig {
            title: "".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            organization: "Test Organization".to_string(),
            device_name: "Test Device".to_string(),
            device_version: "1.0.0".to_string(),
            date_generated: "2025-01-06".to_string(),
            regulatory_basis: vec!["ISO 14971".to_string()],
        };

        let temp_output = project_path.join("test_empty_title.html");
        let result = manager.generate_template(TemplateType::RiskAssessment, &temp_output, config_with_empty_title, OutputFormat::HTML);
        assert!(result.is_ok(), "Template generation should succeed even with empty title");

        // Test with valid configuration (should succeed)
        let valid_config = TemplateConfig {
            title: "Valid Risk Assessment".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            organization: "Test Organization".to_string(),
            device_name: "Test Device".to_string(),
            device_version: "1.0.0".to_string(),
            date_generated: "2025-01-06".to_string(),
            regulatory_basis: vec!["ISO 14971".to_string()],
        };

        let temp_output2 = project_path.join("test_valid.html");
        let result = manager.generate_template(TemplateType::RiskAssessment, &temp_output2, valid_config, OutputFormat::HTML);
        assert!(result.is_ok(), "Template generation should succeed with valid config");
    }

    /// Test ISO 14971 compliance validation in templates
    #[test]
    fn test_iso14971_compliance_in_templates() {
        let (_temp_dir, project_path) = create_temp_project();

        // Initialize audit system for the test
        let audit_config = crate::modules::audit_logger::AuditConfig {
            project_path: project_path.to_string_lossy().to_string(),
            retention_days: 30,
            daily_rotation: false,
            max_file_size_mb: 10,
            require_checksums: false,
        };
        let _ = crate::modules::audit_logger::initialize_audit_system(audit_config);

        let manager = DocumentationManager::new(&project_path).expect("Failed to create manager");

        let config = TemplateConfig {
            title: "Medical Device Project".to_string(),
            organization: "Test Organization".to_string(),
            device_name: "Test Device".to_string(),
            device_version: "1.0".to_string(),
            date_generated: "2025-01-06".to_string(),
            regulatory_basis: vec!["ISO 14971:2019".to_string()],
            author: "Test Author".to_string(),
            version: "1.0.0".to_string(),
        };
        
        // Test that ISO 14971-specific templates reference ISO 14971
        // Note: FMEA is a general analysis technique and may not always reference ISO 14971
        let templates = vec![
            TemplateType::RiskAssessment,
            TemplateType::RiskManagementPlan,
            TemplateType::ControlEffectiveness,
            TemplateType::PostMarketSurveillance,
        ];
        
        for template_type in templates {
            let output_path = project_path.join("test_output.html");
            let result = manager.generate_template(template_type, &output_path, config.clone(), OutputFormat::HTML);
            assert!(result.is_ok(), "Template generation should succeed");

            // Read the generated file to check content
            let template_content = std::fs::read_to_string(&output_path).expect("Failed to read generated template");
            assert!(template_content.contains("ISO 14971"), "Template should reference ISO 14971");
        }
    }

    /// Test edge cases and error handling
    #[test]
    fn test_edge_cases_and_error_handling() {
        let (_temp_dir, project_path) = create_temp_project();

        // Initialize audit system for the test
        let audit_config = crate::modules::audit_logger::AuditConfig {
            project_path: project_path.to_string_lossy().to_string(),
            retention_days: 30,
            daily_rotation: false,
            max_file_size_mb: 10,
            require_checksums: false,
        };
        let _ = crate::modules::audit_logger::initialize_audit_system(audit_config);

        let manager = DocumentationManager::new(&project_path).expect("Failed to create manager");

        // Test with invalid output path - should fail when trying to write
        let config = TemplateConfig {
            title: "Test".to_string(),
            organization: "Test".to_string(),
            device_name: "Test".to_string(),
            device_version: "1.0".to_string(),
            date_generated: "2025-01-06".to_string(),
            regulatory_basis: vec!["ISO 14971".to_string()],
            author: "Test".to_string(),
            version: "1.0".to_string(),
        };

        // Use a path that will definitely fail on write (invalid characters on Windows)
        let invalid_output_path = if cfg!(windows) {
            PathBuf::from("C:\\invalid<>path\\test.html")
        } else {
            PathBuf::from("/root/test.html") // Requires root permissions
        };

        let result = manager.generate_template(TemplateType::RiskAssessment, &invalid_output_path, config, OutputFormat::HTML);
        assert!(result.is_err(), "Template generation should fail with invalid output path");

        // Test with valid template config
        let config = TemplateConfig {
            title: "Test Project".to_string(),
            organization: "Test Organization".to_string(),
            device_name: "Test Device".to_string(),
            device_version: "1.0.0".to_string(),
            date_generated: "2025-01-06".to_string(),
            regulatory_basis: vec!["ISO 14971:2019".to_string()],
            author: "Test Author".to_string(),
            version: "1.0.0".to_string(),
        };

        // Test template generation with valid config
        let output_path = project_path.join("test_output.html");
        let result = manager.generate_template(TemplateType::RiskAssessment, &output_path, config, OutputFormat::HTML);
        assert!(result.is_ok(), "Template generation should succeed with valid config");
    }
}
