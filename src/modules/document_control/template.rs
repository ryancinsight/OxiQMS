//! Document template management for QMS
//! Phase 2.1.9 - Document Templates
//! Provides predefined document templates with variable substitution

use crate::error::{QmsError, QmsResult};
use crate::modules::document_control::document::DocumentType;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Document template with variable substitution
#[derive(Debug, Clone)]
pub struct DocumentTemplate {
    pub name: String,
    pub description: String,
    pub doc_type: DocumentType,
    pub content: String,
    pub variables: Vec<String>,
}

/// Template variable substitution context
#[derive(Debug, Clone)]
pub struct TemplateContext {
    pub project_name: String,
    pub date: String,
    pub user: String,
    pub version: String,
    pub custom_variables: HashMap<String, String>,
}

impl TemplateContext {
    /// Create a new template context with standard variables
    pub fn new(project_name: String, user: String) -> Self {
        Self {
            project_name,
            date: crate::utils::current_date_string(),
            user,
            version: "1.0.0".to_string(),
            custom_variables: HashMap::new(),
        }
    }

    /// Add a custom variable to the context
    #[allow(dead_code)] // May be used in future features
    pub fn add_variable(&mut self, key: String, value: String) {
        self.custom_variables.insert(key, value);
    }

    /// Get variable value by name
    #[allow(dead_code)] // May be used in future features  
    pub fn get_variable(&self, name: &str) -> Option<String> {
        match name {
            "PROJECT_NAME" => Some(self.project_name.clone()),
            "DATE" => Some(self.date.clone()),
            "USER" => Some(self.user.clone()),
            "VERSION" => Some(self.version.clone()),
            custom => self.custom_variables.get(custom).cloned(),
        }
    }

    /// Get all available variables as a map
    pub fn get_all_variables(&self) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("PROJECT_NAME".to_string(), self.project_name.clone());
        vars.insert("DATE".to_string(), self.date.clone());
        vars.insert("USER".to_string(), self.user.clone());
        vars.insert("VERSION".to_string(), self.version.clone());
        vars.extend(self.custom_variables.clone());
        vars
    }
}

/// Template manager for handling document templates
pub struct TemplateManager {
    #[allow(dead_code)] // Used for path resolution in template operations
    project_path: PathBuf,
    templates_path: PathBuf,
}

impl TemplateManager {
    /// Create a new template manager
    pub fn new(project_path: PathBuf) -> Self {
        let templates_path = project_path.join("templates");
        Self {
            project_path,
            templates_path,
        }
    }

    /// Initialize template directory with default templates
    pub fn initialize_templates(&self) -> QmsResult<()> {
        // Create templates directory
        fs::create_dir_all(&self.templates_path)?;

        // Create default templates if they don't exist
        self.create_default_templates()?;

        Ok(())
    }

    /// Create default document templates
    fn create_default_templates(&self) -> QmsResult<()> {
        let default_templates = self.get_default_templates();

        for template in default_templates {
            let template_path = self.templates_path.join(format!("{}.md", template.name));
            if !template_path.exists() {
                fs::write(&template_path, &template.content)?;
            }
        }

        // Create template metadata file
        let metadata_path = self.templates_path.join("templates.json");
        if !metadata_path.exists() {
            let metadata = self.create_template_metadata();
            fs::write(&metadata_path, metadata)?;
        }

        Ok(())
    }

    /// Get list of default templates
    fn get_default_templates(&self) -> Vec<DocumentTemplate> {
        vec![
            // Software Requirements Specification Template
            DocumentTemplate {
                name: "srs_template".to_string(),
                description: "Software Requirements Specification template for medical devices".to_string(),
                doc_type: DocumentType::SoftwareRequirementsSpecification,
                content: self.create_srs_template(),
                variables: vec!["PROJECT_NAME".to_string(), "DATE".to_string(), "USER".to_string(), "VERSION".to_string()],
            },
            // Software Design Description Template
            DocumentTemplate {
                name: "sdd_template".to_string(),
                description: "Software Design Description template for medical devices".to_string(),
                doc_type: DocumentType::SoftwareDesignDescription,
                content: self.create_sdd_template(),
                variables: vec!["PROJECT_NAME".to_string(), "DATE".to_string(), "USER".to_string(), "VERSION".to_string()],
            },
            // Test Protocol Template
            DocumentTemplate {
                name: "test_protocol_template".to_string(),
                description: "Test Protocol template for verification and validation".to_string(),
                doc_type: DocumentType::TestProtocol,
                content: self.create_test_protocol_template(),
                variables: vec!["PROJECT_NAME".to_string(), "DATE".to_string(), "USER".to_string(), "VERSION".to_string()],
            },
            // Risk Management File Template
            DocumentTemplate {
                name: "rmf_template".to_string(),
                description: "Risk Management File template following ISO 14971".to_string(),
                doc_type: DocumentType::RiskManagementFile,
                content: self.create_rmf_template(),
                variables: vec!["PROJECT_NAME".to_string(), "DATE".to_string(), "USER".to_string(), "VERSION".to_string()],
            },
            // User Requirements Template
            DocumentTemplate {
                name: "user_requirements_template".to_string(),
                description: "User Requirements template for stakeholder needs".to_string(),
                doc_type: DocumentType::UserRequirements,
                content: self.create_user_requirements_template(),
                variables: vec!["PROJECT_NAME".to_string(), "DATE".to_string(), "USER".to_string(), "VERSION".to_string()],
            },
            // Generic Document Template
            DocumentTemplate {
                name: "generic_template".to_string(),
                description: "Generic document template for any document type".to_string(),
                doc_type: DocumentType::Other("Generic".to_string()),
                content: self.create_generic_template(),
                variables: vec!["PROJECT_NAME".to_string(), "DATE".to_string(), "USER".to_string(), "VERSION".to_string()],
            },
        ]
    }

    /// Create Software Requirements Specification template
    fn create_srs_template(&self) -> String {
        r#"# Software Requirements Specification

**Project:** {{PROJECT_NAME}}  
**Document Type:** Software Requirements Specification  
**Version:** {{VERSION}}  
**Date:** {{DATE}}  
**Author:** {{USER}}  
**Status:** Draft  

---

## 1. Introduction

### 1.1 Purpose
This Software Requirements Specification (SRS) defines the functional and non-functional requirements for the {{PROJECT_NAME}} medical device software.

### 1.2 Scope
This document covers all software requirements for {{PROJECT_NAME}} in accordance with IEC 62304 and FDA 21 CFR Part 820.

### 1.3 Definitions and Abbreviations
- **SRS**: Software Requirements Specification
- **IEC 62304**: Medical device software standard
- **FDA 21 CFR Part 820**: Quality System Regulation

### 1.4 References
- IEC 62304:2006 Medical device software — Software life cycle processes
- FDA 21 CFR Part 820 Quality System Regulation
- ISO 13485:2016 Medical devices — Quality management systems

---

## 2. Overall Description

### 2.1 Product Perspective
Describe the context and origin of the {{PROJECT_NAME}} software.

### 2.2 Product Functions
List the major functions that the software will perform.

### 2.3 User Classes and Characteristics
Describe the general characteristics of the intended users.

### 2.4 Operating Environment
Describe the environment in which the software will operate.

---

## 3. System Features

### 3.1 Functional Requirements

#### FR-001: [Requirement Name]
- **Description**: [Detailed description]
- **Priority**: High/Medium/Low
- **Source**: [Stakeholder/Standard]
- **Verification Method**: Test/Analysis/Inspection/Demonstration

#### FR-002: [Requirement Name]
- **Description**: [Detailed description]
- **Priority**: High/Medium/Low
- **Source**: [Stakeholder/Standard]
- **Verification Method**: Test/Analysis/Inspection/Demonstration

### 3.2 Non-Functional Requirements

#### NFR-001: Performance Requirements
- **Description**: [Performance criteria]

#### NFR-002: Safety Requirements
- **Description**: [Safety requirements per ISO 14971]

#### NFR-003: Security Requirements
- **Description**: [Security and privacy requirements]

---

## 4. External Interface Requirements

### 4.1 User Interfaces
Describe the user interface requirements.

### 4.2 Hardware Interfaces
Describe the hardware interface requirements.

### 4.3 Software Interfaces
Describe the software interface requirements.

### 4.4 Communication Interfaces
Describe the communication requirements.

---

## 5. Quality Attributes

### 5.1 Reliability
Define reliability requirements and metrics.

### 5.2 Usability
Define usability requirements and metrics.

### 5.3 Performance
Define performance requirements and metrics.

### 5.4 Supportability
Define maintainability and supportability requirements.

---

## 6. Other Requirements

### 6.1 Regulatory Requirements
List applicable regulatory requirements.

### 6.2 Standards Compliance
List applicable standards and their requirements.

---

## 7. Appendices

### Appendix A: Glossary
Define technical terms and acronyms.

### Appendix B: Analysis Models
Include relevant analysis models or diagrams.

---

**Document Control**
- Created: {{DATE}} by {{USER}}
- Version: {{VERSION}}
- Status: Draft
- Next Review: [Date]

**Approval**
- [ ] Technical Review Complete
- [ ] Quality Review Complete
- [ ] Management Approval
"#.to_string()
    }

    /// Create Software Design Description template
    fn create_sdd_template(&self) -> String {
        r#"# Software Design Description

**Project:** {{PROJECT_NAME}}  
**Document Type:** Software Design Description  
**Version:** {{VERSION}}  
**Date:** {{DATE}}  
**Author:** {{USER}}  
**Status:** Draft  

---

## 1. Introduction

### 1.1 Purpose
This Software Design Description (SDD) describes the software design for {{PROJECT_NAME}} medical device software.

### 1.2 Scope
This document covers the detailed design of all software components for {{PROJECT_NAME}}.

### 1.3 Design Approach
Describe the overall design approach and methodology.

---

## 2. System Overview

### 2.1 System Architecture
Describe the high-level system architecture.

### 2.2 Design Constraints
List any constraints that influence the design.

### 2.3 Design Principles
List the design principles followed.

---

## 3. Software Architecture

### 3.1 Architectural Design
Describe the overall software architecture.

### 3.2 Component Diagram
Include or reference component diagrams.

### 3.3 Interface Design
Describe the interfaces between components.

---

## 4. Detailed Design

### 4.1 Component Specifications

#### Component 1: [Component Name]
- **Purpose**: [Description]
- **Interfaces**: [Input/Output interfaces]
- **Algorithms**: [Key algorithms]
- **Data Structures**: [Data structures used]

#### Component 2: [Component Name]
- **Purpose**: [Description]
- **Interfaces**: [Input/Output interfaces]
- **Algorithms**: [Key algorithms]
- **Data Structures**: [Data structures used]

### 4.2 Data Design
Describe the data design and database schema.

### 4.3 Algorithm Design
Describe key algorithms and their implementation.

---

## 5. User Interface Design

### 5.1 User Interface Overview
Describe the user interface design approach.

### 5.2 Screen Specifications
Detail each user interface screen.

### 5.3 User Interaction Flow
Describe the user interaction workflows.

---

## 6. Security Design

### 6.1 Authentication
Describe authentication mechanisms.

### 6.2 Authorization
Describe authorization and access control.

### 6.3 Data Protection
Describe data protection measures.

---

## 7. Error Handling

### 7.1 Error Detection
Describe error detection mechanisms.

### 7.2 Error Reporting
Describe error reporting and logging.

### 7.3 Error Recovery
Describe error recovery procedures.

---

## 8. Traceability

### 8.1 Requirements Traceability
Map design elements to requirements.

### 8.2 Test Traceability
Map design elements to test cases.

---

**Document Control**
- Created: {{DATE}} by {{USER}}
- Version: {{VERSION}}
- Status: Draft
- Next Review: [Date]

**Approval**
- [ ] Design Review Complete
- [ ] Architecture Review Complete
- [ ] Management Approval
"#.to_string()
    }

    /// Create Test Protocol template
    fn create_test_protocol_template(&self) -> String {
        r#"# Test Protocol

**Project:** {{PROJECT_NAME}}  
**Document Type:** Test Protocol  
**Version:** {{VERSION}}  
**Date:** {{DATE}}  
**Author:** {{USER}}  
**Status:** Draft  

---

## 1. Introduction

### 1.1 Purpose
This Test Protocol defines the test procedures for verification and validation of {{PROJECT_NAME}}.

### 1.2 Scope
This document covers all test activities required for {{PROJECT_NAME}} software validation.

### 1.3 Test Approach
Describe the overall test approach and strategy.

---

## 2. Test Environment

### 2.1 Test Setup
Describe the test environment setup.

### 2.2 Test Equipment
List required test equipment and tools.

### 2.3 Test Data
Describe test data requirements.

---

## 3. Test Procedures

### 3.1 Unit Tests

#### TC-001: [Test Case Name]
- **Objective**: [Test objective]
- **Preconditions**: [Setup requirements]
- **Test Steps**:
  1. [Step 1]
  2. [Step 2]
  3. [Step 3]
- **Expected Results**: [Expected outcome]
- **Requirements Traced**: [REQ-XXX]

#### TC-002: [Test Case Name]
- **Objective**: [Test objective]
- **Preconditions**: [Setup requirements]
- **Test Steps**:
  1. [Step 1]
  2. [Step 2]
  3. [Step 3]
- **Expected Results**: [Expected outcome]
- **Requirements Traced**: [REQ-XXX]

### 3.2 Integration Tests

#### TC-101: [Integration Test Name]
- **Objective**: [Test objective]
- **Preconditions**: [Setup requirements]
- **Test Steps**:
  1. [Step 1]
  2. [Step 2]
  3. [Step 3]
- **Expected Results**: [Expected outcome]
- **Requirements Traced**: [REQ-XXX]

### 3.3 System Tests

#### TC-201: [System Test Name]
- **Objective**: [Test objective]
- **Preconditions**: [Setup requirements]
- **Test Steps**:
  1. [Step 1]
  2. [Step 2]
  3. [Step 3]
- **Expected Results**: [Expected outcome]
- **Requirements Traced**: [REQ-XXX]

---

## 4. Test Execution

### 4.1 Test Schedule
Define the test execution schedule.

### 4.2 Test Responsibilities
Define roles and responsibilities.

### 4.3 Test Reporting
Define test reporting procedures.

---

## 5. Test Results

### 5.1 Test Summary
Summarize test execution results.

### 5.2 Pass/Fail Criteria
Define pass/fail criteria for each test.

### 5.3 Defect Management
Describe defect tracking and resolution.

---

## 6. Traceability Matrix

| Test Case | Requirement | Status | Result |
|-----------|-------------|---------|---------|
| TC-001    | REQ-001     | [P/F]   | [Pass/Fail] |
| TC-002    | REQ-002     | [P/F]   | [Pass/Fail] |

---

**Document Control**
- Created: {{DATE}} by {{USER}}
- Version: {{VERSION}}
- Status: Draft
- Next Review: [Date]

**Approval**
- [ ] Test Review Complete
- [ ] Quality Review Complete
- [ ] Management Approval
"#.to_string()
    }

    /// Create Risk Management File template
    fn create_rmf_template(&self) -> String {
        r#"# Risk Management File

**Project:** {{PROJECT_NAME}}  
**Document Type:** Risk Management File  
**Version:** {{VERSION}}  
**Date:** {{DATE}}  
**Author:** {{USER}}  
**Status:** Draft  

---

## 1. Introduction

### 1.1 Purpose
This Risk Management File documents the risk management process for {{PROJECT_NAME}} in accordance with ISO 14971.

### 1.2 Scope
This document covers all identified risks associated with {{PROJECT_NAME}} throughout its lifecycle.

### 1.3 Risk Management Process
Describe the risk management process and methodology.

---

## 2. Risk Management Policy

### 2.1 Risk Acceptability Criteria
Define the criteria for acceptable risk levels.

### 2.2 Risk Management Responsibilities
Define roles and responsibilities for risk management.

### 2.3 Risk Management Activities
List the planned risk management activities.

---

## 3. Risk Analysis

### 3.1 Intended Use and Reasonably Foreseeable Misuse
Describe the intended use and potential misuse scenarios.

### 3.2 Risk Management Process Implementation
Describe how the risk management process is implemented.

### 3.3 Hazard Identification

#### HAZARD-001: [Hazard Name]
- **Hazard Description**: [Description of the hazard]
- **Hazardous Situation**: [Sequence of events leading to harm]
- **Harm**: [Type of harm that could result]
- **Severity**: [1-5 scale]
- **Occurrence**: [1-5 scale]
- **Detectability**: [1-5 scale]
- **RPN**: [Risk Priority Number]

#### HAZARD-002: [Hazard Name]
- **Hazard Description**: [Description of the hazard]
- **Hazardous Situation**: [Sequence of events leading to harm]
- **Harm**: [Type of harm that could result]
- **Severity**: [1-5 scale]
- **Occurrence**: [1-5 scale]
- **Detectability**: [1-5 scale]
- **RPN**: [Risk Priority Number]

---

## 4. Risk Evaluation

### 4.1 Risk Acceptability Assessment
Evaluate each identified risk against acceptability criteria.

### 4.2 Risk Control Measures
Document risk control measures for unacceptable risks.

---

## 5. Risk Control

### 5.1 Risk Control Measures

#### Control Measure CM-001
- **For Hazard**: [HAZARD-XXX]
- **Description**: [Control measure description]
- **Implementation**: [How it will be implemented]
- **Verification**: [How effectiveness will be verified]

#### Control Measure CM-002
- **For Hazard**: [HAZARD-XXX]
- **Description**: [Control measure description]
- **Implementation**: [How it will be implemented]
- **Verification**: [How effectiveness will be verified]

### 5.2 Residual Risk Analysis
Analyze residual risks after control measures.

### 5.3 Benefit-Risk Analysis
Perform benefit-risk analysis for remaining risks.

---

## 6. Risk Management Report

### 6.1 Risk Management Summary
Summarize the risk management activities.

### 6.2 Residual Risk Acceptability
Conclude on the acceptability of residual risks.

### 6.3 Risk Management Conclusion
Overall conclusion of risk management activities.

---

## 7. Post-Market Surveillance

### 7.1 Post-Market Data Collection
Plan for collecting post-market risk data.

### 7.2 Risk Management File Updates
Process for updating the risk management file.

---

**Document Control**
- Created: {{DATE}} by {{USER}}
- Version: {{VERSION}}
- Status: Draft
- Next Review: [Date]

**Approval**
- [ ] Risk Review Complete
- [ ] Quality Review Complete
- [ ] Management Approval
"#.to_string()
    }

    /// Create User Requirements template
    fn create_user_requirements_template(&self) -> String {
        r#"# User Requirements

**Project:** {{PROJECT_NAME}}  
**Document Type:** User Requirements  
**Version:** {{VERSION}}  
**Date:** {{DATE}}  
**Author:** {{USER}}  
**Status:** Draft  

---

## 1. Introduction

### 1.1 Purpose
This document defines the user requirements for {{PROJECT_NAME}} from the stakeholder perspective.

### 1.2 Scope
This document covers all user needs and expectations for {{PROJECT_NAME}}.

### 1.3 Stakeholders
List the key stakeholders and their roles.

---

## 2. User Needs

### 2.1 Primary Users
Describe the primary users and their characteristics.

### 2.2 User Workflows
Describe the typical user workflows and use cases.

### 2.3 User Environment
Describe the environment in which users will operate.

---

## 3. Functional User Requirements

### 3.1 Core Functionality

#### UR-001: [User Requirement Name]
- **Description**: [User need description]
- **Rationale**: [Why this is needed]
- **Source**: [Stakeholder/regulation]
- **Priority**: High/Medium/Low
- **Acceptance Criteria**: [How to verify satisfaction]

#### UR-002: [User Requirement Name]
- **Description**: [User need description]
- **Rationale**: [Why this is needed]
- **Source**: [Stakeholder/regulation]
- **Priority**: High/Medium/Low
- **Acceptance Criteria**: [How to verify satisfaction]

### 3.2 User Interface Requirements

#### UR-101: [UI Requirement Name]
- **Description**: [UI requirement description]
- **Rationale**: [Why this is needed]
- **Source**: [Stakeholder/usability study]
- **Priority**: High/Medium/Low
- **Acceptance Criteria**: [How to verify satisfaction]

---

## 4. Non-Functional User Requirements

### 4.1 Performance Requirements

#### UR-201: Response Time
- **Description**: System shall respond within X seconds
- **Rationale**: User productivity and satisfaction
- **Acceptance Criteria**: 95% of operations complete in X seconds

### 4.2 Usability Requirements

#### UR-301: Ease of Use
- **Description**: System shall be intuitive for target users
- **Rationale**: Minimize training and errors
- **Acceptance Criteria**: Users can complete tasks without training

### 4.3 Safety Requirements

#### UR-401: Patient Safety
- **Description**: System shall not compromise patient safety
- **Rationale**: Medical device safety requirements
- **Acceptance Criteria**: Risk analysis shows acceptable risk level

---

## 5. Regulatory Requirements

### 5.1 FDA Requirements
List applicable FDA requirements and their implications.

### 5.2 ISO Requirements
List applicable ISO standard requirements.

### 5.3 Other Regulatory Requirements
List any other applicable regulatory requirements.

---

## 6. User Acceptance Criteria

### 6.1 Acceptance Testing
Define the user acceptance testing approach.

### 6.2 Success Criteria
Define the criteria for user acceptance.

### 6.3 User Training
Define user training requirements.

---

**Document Control**
- Created: {{DATE}} by {{USER}}
- Version: {{VERSION}}
- Status: Draft
- Next Review: [Date]

**Approval**
- [ ] User Review Complete
- [ ] Stakeholder Sign-off
- [ ] Management Approval
"#.to_string()
    }

    /// Create generic template
    fn create_generic_template(&self) -> String {
        r#"# {{PROJECT_NAME}} Document

**Project:** {{PROJECT_NAME}}  
**Document Type:** [Document Type]  
**Version:** {{VERSION}}  
**Date:** {{DATE}}  
**Author:** {{USER}}  
**Status:** Draft  

---

## 1. Introduction

### 1.1 Purpose
[State the purpose of this document]

### 1.2 Scope
[Define the scope of this document]

### 1.3 Audience
[Identify the intended audience]

---

## 2. Overview

### 2.1 Background
[Provide background information]

### 2.2 Objectives
[List the objectives]

### 2.3 Approach
[Describe the approach taken]

---

## 3. Main Content

### 3.1 Section 1
[Main content section 1]

### 3.2 Section 2
[Main content section 2]

### 3.3 Section 3
[Main content section 3]

---

## 4. Implementation

### 4.1 Timeline
[Implementation timeline]

### 4.2 Resources
[Required resources]

### 4.3 Dependencies
[Dependencies and prerequisites]

---

## 5. Conclusion

### 5.1 Summary
[Summarize the key points]

### 5.2 Next Steps
[Define next steps]

### 5.3 References
[List references and related documents]

---

**Document Control**
- Created: {{DATE}} by {{USER}}
- Version: {{VERSION}}
- Status: Draft
- Next Review: [Date]

**Approval**
- [ ] Technical Review Complete
- [ ] Quality Review Complete
- [ ] Management Approval
"#.to_string()
    }

    /// Create template metadata JSON
    fn create_template_metadata(&self) -> String {
        r#"{
  "version": "1.0",
  "templates": [
    {
      "name": "srs_template",
      "display_name": "Software Requirements Specification",
      "description": "Template for software requirements specification following IEC 62304",
      "doc_type": "SoftwareRequirementsSpecification",
      "variables": ["PROJECT_NAME", "DATE", "USER", "VERSION"],
      "category": "requirements"
    },
    {
      "name": "sdd_template",
      "display_name": "Software Design Description",
      "description": "Template for software design documentation",
      "doc_type": "SoftwareDesignDescription", 
      "variables": ["PROJECT_NAME", "DATE", "USER", "VERSION"],
      "category": "design"
    },
    {
      "name": "test_protocol_template",
      "display_name": "Test Protocol",
      "description": "Template for test protocols and verification procedures",
      "doc_type": "TestProtocol",
      "variables": ["PROJECT_NAME", "DATE", "USER", "VERSION"],
      "category": "testing"
    },
    {
      "name": "rmf_template",
      "display_name": "Risk Management File",
      "description": "Template for risk management documentation per ISO 14971",
      "doc_type": "RiskManagementFile",
      "variables": ["PROJECT_NAME", "DATE", "USER", "VERSION"],
      "category": "risk"
    },
    {
      "name": "user_requirements_template",
      "display_name": "User Requirements",
      "description": "Template for capturing user needs and requirements",
      "doc_type": "UserRequirements",
      "variables": ["PROJECT_NAME", "DATE", "USER", "VERSION"],
      "category": "requirements"
    },
    {
      "name": "generic_template",
      "display_name": "Generic Document",
      "description": "Generic template for any document type",
      "doc_type": "Other",
      "variables": ["PROJECT_NAME", "DATE", "USER", "VERSION"],
      "category": "general"
    }
  ]
}"#.to_string()
    }

    /// List available templates
    pub fn list_templates(&self) -> QmsResult<Vec<DocumentTemplate>> {
        if !self.templates_path.exists() {
            self.initialize_templates()?;
        }

        let metadata_path = self.templates_path.join("templates.json");
        if !metadata_path.exists() {
            return Ok(self.get_default_templates());
        }

        // Load templates from metadata file
        let metadata_content = fs::read_to_string(&metadata_path)?;
        let metadata = crate::json_utils::JsonValue::parse(&metadata_content)?;

        let mut templates = Vec::new();
        
        if let crate::json_utils::JsonValue::Object(obj) = metadata {
            if let Some(crate::json_utils::JsonValue::Array(template_array)) = obj.get("templates") {
                for template_value in template_array {
                    if let crate::json_utils::JsonValue::Object(template_obj) = template_value {
                        if let Some(template) = self.parse_template_metadata(template_obj)? {
                            templates.push(template);
                        }
                    }
                }
            }
        }

        Ok(templates)
    }

    /// Parse template metadata from JSON object
    fn parse_template_metadata(&self, obj: &HashMap<String, crate::json_utils::JsonValue>) -> QmsResult<Option<DocumentTemplate>> {
        let name = match obj.get("name") {
            Some(crate::json_utils::JsonValue::String(s)) => s.clone(),
            _ => return Ok(None),
        };

        let description = match obj.get("description") {
            Some(crate::json_utils::JsonValue::String(s)) => s.clone(),
            _ => "No description".to_string(),
        };

        let doc_type_str = match obj.get("doc_type") {
            Some(crate::json_utils::JsonValue::String(s)) => s.clone(),
            _ => "Other".to_string(),
        };

        let doc_type = self.parse_document_type(&doc_type_str);

        let variables = match obj.get("variables") {
            Some(crate::json_utils::JsonValue::Array(arr)) => {
                arr.iter()
                    .filter_map(|v| if let crate::json_utils::JsonValue::String(s) = v { Some(s.clone()) } else { None })
                    .collect()
            }
            _ => vec!["PROJECT_NAME".to_string(), "DATE".to_string(), "USER".to_string(), "VERSION".to_string()],
        };

        // Load template content
        let template_path = self.templates_path.join(format!("{name}.md"));
        let content = if template_path.exists() {
            fs::read_to_string(&template_path)?
        } else {
            "# Template not found\n\nTemplate content could not be loaded.".to_string()
        };

        Ok(Some(DocumentTemplate {
            name,
            description,
            doc_type,
            content,
            variables,
        }))
    }

    /// Parse document type string
    fn parse_document_type(&self, type_str: &str) -> DocumentType {
        match type_str {
            "SoftwareRequirementsSpecification" => DocumentType::SoftwareRequirementsSpecification,
            "SoftwareDesignDescription" => DocumentType::SoftwareDesignDescription,
            "VerificationAndValidation" => DocumentType::VerificationAndValidation,
            "RiskManagementFile" => DocumentType::RiskManagementFile,
            "DesignHistoryFile" => DocumentType::DesignHistoryFile,
            "UserRequirements" => DocumentType::UserRequirements,
            "TestProtocol" => DocumentType::TestProtocol,
            "TestReport" => DocumentType::TestReport,
            other => DocumentType::Other(other.to_string()),
        }
    }

    /// Get a specific template by name
    pub fn get_template(&self, template_name: &str) -> QmsResult<DocumentTemplate> {
        let templates = self.list_templates()?;
        
        templates
            .into_iter()
            .find(|t| t.name == template_name)
            .ok_or_else(|| QmsError::not_found(&format!("Template not found: {template_name}")))
    }

    /// Create document from template with variable substitution
    pub fn create_document_from_template(
        &self,
        template_name: &str,
        _title: String,
        context: TemplateContext,
    ) -> QmsResult<(String, DocumentType)> {
        let template = self.get_template(template_name)?;
        let content = self.substitute_variables(&template.content, &context)?;
        
        Ok((content, template.doc_type))
    }

    /// Substitute variables in template content
    pub fn substitute_variables(&self, content: &str, context: &TemplateContext) -> QmsResult<String> {
        let mut result = content.to_string();
        
        // Get all variables from context
        let variables = context.get_all_variables();
        
        // Replace each variable
        for (var_name, var_value) in variables {
            let placeholder = format!("{{{{{var_name}}}}}");
            result = result.replace(&placeholder, &var_value);
        }
        
        Ok(result)
    }

    /// Validate template file
    #[allow(dead_code)] // Future feature for template validation
    pub fn validate_template(&self, template_path: &Path) -> QmsResult<Vec<String>> {
        if !template_path.exists() {
            return Err(QmsError::not_found(&format!("Template file not found: {}", template_path.display())));
        }

        let content = fs::read_to_string(template_path)?;
        let variables = self.extract_variables(&content);
        
        Ok(variables)
    }

    /// Extract variables from template content
    #[allow(dead_code)] // Future feature for template validation
    pub fn extract_variables(&self, content: &str) -> Vec<String> {
        let mut variables = Vec::new();
        let mut chars = content.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '{'
                && chars.peek() == Some(&'{') {
                    chars.next(); // consume second '{'
                    
                    // Extract variable name
                    let mut var_name = String::new();
                    let mut found_closing = false;
                    
                    while let Some(ch) = chars.next() {
                        if ch == '}' && chars.peek() == Some(&'}') {
                            chars.next(); // consume second '}'
                            found_closing = true;
                            break;
                        }
                        var_name.push(ch);
                    }
                    
                    if found_closing && !var_name.trim().is_empty() {
                        let trimmed_name = var_name.trim().to_string();
                        if !variables.contains(&trimmed_name) {
                            variables.push(trimmed_name);
                        }
                    }
                }
        }
        
        variables
    }

    /// Save custom template
    #[allow(dead_code)] // Future feature for custom templates
    pub fn save_template(&self, template: &DocumentTemplate) -> QmsResult<()> {
        if !self.templates_path.exists() {
            fs::create_dir_all(&self.templates_path)?;
        }

        // Save template content
        let template_path = self.templates_path.join(format!("{}.md", template.name));
        fs::write(&template_path, &template.content)?;

        // Update metadata
        self.update_template_metadata(template)?;

        Ok(())
    }

    /// Update template metadata
    #[allow(dead_code)] // Future feature for custom templates
    fn update_template_metadata(&self, template: &DocumentTemplate) -> QmsResult<()> {
        let metadata_path = self.templates_path.join("templates.json");
        
        // Load existing metadata or create new
        let mut metadata = if metadata_path.exists() {
            let content = fs::read_to_string(&metadata_path)?;
            crate::json_utils::JsonValue::parse(&content)?
        } else {
            crate::json_utils::JsonValue::Object(HashMap::new())
        };

        // Extract templates array
        let mut templates_array = if let crate::json_utils::JsonValue::Object(ref mut obj) = metadata {
            match obj.get("templates") {
                Some(crate::json_utils::JsonValue::Array(arr)) => arr.clone(),
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        };

        // Remove existing template with same name
        templates_array.retain(|t| {
            if let crate::json_utils::JsonValue::Object(obj) = t {
                if let Some(crate::json_utils::JsonValue::String(name)) = obj.get("name") {
                    name != &template.name
                } else {
                    true
                }
            } else {
                true
            }
        });

        // Add new template metadata
        let mut template_obj = HashMap::new();
        template_obj.insert("name".to_string(), crate::json_utils::JsonValue::String(template.name.clone()));
        template_obj.insert("display_name".to_string(), crate::json_utils::JsonValue::String(template.name.clone()));
        template_obj.insert("description".to_string(), crate::json_utils::JsonValue::String(template.description.clone()));
        template_obj.insert("doc_type".to_string(), crate::json_utils::JsonValue::String(format!("{:?}", template.doc_type)));
        
        let variables_array = template.variables.iter()
            .map(|v| crate::json_utils::JsonValue::String(v.clone()))
            .collect();
        template_obj.insert("variables".to_string(), crate::json_utils::JsonValue::Array(variables_array));
        template_obj.insert("category".to_string(), crate::json_utils::JsonValue::String("custom".to_string()));

        templates_array.push(crate::json_utils::JsonValue::Object(template_obj));

        // Update metadata
        if let crate::json_utils::JsonValue::Object(ref mut obj) = metadata {
            obj.insert("templates".to_string(), crate::json_utils::JsonValue::Array(templates_array));
        }

        // Save updated metadata
        let json_string = metadata.json_to_string();
        fs::write(&metadata_path, json_string)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn create_test_dir() -> PathBuf {
        let mut temp_dir = env::temp_dir();
        temp_dir.push(format!("qms_template_test_{}", crate::utils::generate_uuid()));
        fs::create_dir_all(&temp_dir).unwrap();
        temp_dir
    }

    fn cleanup_test_dir(dir: &PathBuf) {
        if dir.exists() {
            fs::remove_dir_all(dir).unwrap_or_else(|e| {
                eprintln!("Warning: Failed to cleanup test directory: {}", e);
            });
        }
    }

    #[test]
    fn test_template_context_creation() {
        let context = TemplateContext::new("Test Project".to_string(), "test_user".to_string());
        
        assert_eq!(context.project_name, "Test Project");
        assert_eq!(context.user, "test_user");
        assert_eq!(context.version, "1.0.0");
        assert!(!context.date.is_empty());
    }

    #[test]
    fn test_template_context_variables() {
        let mut context = TemplateContext::new("Test Project".to_string(), "test_user".to_string());
        context.add_variable("CUSTOM_VAR".to_string(), "custom_value".to_string());
        
        assert_eq!(context.get_variable("PROJECT_NAME"), Some("Test Project".to_string()));
        assert_eq!(context.get_variable("USER"), Some("test_user".to_string()));
        assert_eq!(context.get_variable("CUSTOM_VAR"), Some("custom_value".to_string()));
        assert_eq!(context.get_variable("NONEXISTENT"), None);
    }

    #[test]
    fn test_template_manager_initialization() {
        let test_dir = create_test_dir();
        let manager = TemplateManager::new(test_dir.clone());
        
        let result = manager.initialize_templates();
        assert!(result.is_ok());
        
        // Check that templates directory was created
        assert!(test_dir.join("templates").exists());
        
        // Check that some template files were created
        assert!(test_dir.join("templates").join("srs_template.md").exists());
        assert!(test_dir.join("templates").join("templates.json").exists());
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_list_templates() {
        let test_dir = create_test_dir();
        let manager = TemplateManager::new(test_dir.clone());
        
        let templates = manager.list_templates().unwrap();
        
        assert!(!templates.is_empty());
        assert!(templates.iter().any(|t| t.name == "srs_template"));
        assert!(templates.iter().any(|t| t.name == "generic_template"));
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_get_template() {
        let test_dir = create_test_dir();
        let manager = TemplateManager::new(test_dir.clone());
        
        let template = manager.get_template("srs_template").unwrap();
        
        assert_eq!(template.name, "srs_template");
        assert!(!template.content.is_empty());
        assert!(template.content.contains("{{PROJECT_NAME}}"));
        assert!(template.variables.contains(&"PROJECT_NAME".to_string()));
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_get_nonexistent_template() {
        let test_dir = create_test_dir();
        let manager = TemplateManager::new(test_dir.clone());
        
        let result = manager.get_template("nonexistent_template");
        assert!(result.is_err());
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_variable_substitution() {
        let test_dir = create_test_dir();
        let manager = TemplateManager::new(test_dir.clone());
        
        let context = TemplateContext::new("Test Project".to_string(), "test_user".to_string());
        let content = "Project: {{PROJECT_NAME}}, Author: {{USER}}, Date: {{DATE}}";
        
        let result = manager.substitute_variables(content, &context).unwrap();
        
        assert!(result.contains("Project: Test Project"));
        assert!(result.contains("Author: test_user"));
        assert!(result.contains("Date:"));
        assert!(!result.contains("{{"));
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_extract_variables() {
        let test_dir = create_test_dir();
        let manager = TemplateManager::new(test_dir.clone());
        
        let content = "{{PROJECT_NAME}} document by {{USER}} on {{DATE}} version {{VERSION}}";
        let variables = manager.extract_variables(content);
        
        assert_eq!(variables.len(), 4);
        assert!(variables.contains(&"PROJECT_NAME".to_string()));
        assert!(variables.contains(&"USER".to_string()));
        assert!(variables.contains(&"DATE".to_string()));
        assert!(variables.contains(&"VERSION".to_string()));
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_extract_variables_edge_cases() {
        let test_dir = create_test_dir();
        let manager = TemplateManager::new(test_dir.clone());
        
        // Test with malformed variables
        let content = "{{VALID}} {INVALID} {{ANOTHER_VALID}} {{{ MALFORMED";
        let variables = manager.extract_variables(content);
        
        assert_eq!(variables.len(), 2);
        assert!(variables.contains(&"VALID".to_string()));
        assert!(variables.contains(&"ANOTHER_VALID".to_string()));
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_create_document_from_template() {
        let test_dir = create_test_dir();
        let manager = TemplateManager::new(test_dir.clone());
        
        let context = TemplateContext::new("Test Project".to_string(), "test_user".to_string());
        let title = "{{PROJECT_NAME}} SRS";
        
        let (content, doc_type) = manager.create_document_from_template("srs_template", title.to_string(), context).unwrap();
        
        assert!(!content.contains("{{PROJECT_NAME}}"));
        assert!(content.contains("Test Project"));
        assert!(content.contains("test_user"));
        assert!(matches!(doc_type, DocumentType::SoftwareRequirementsSpecification));
        
        cleanup_test_dir(&test_dir);
    }

    #[test]
    fn test_save_custom_template() {
        let test_dir = create_test_dir();
        let manager = TemplateManager::new(test_dir.clone());
        
        let custom_template = DocumentTemplate {
            name: "custom_test".to_string(),
            description: "Custom test template".to_string(),
            doc_type: DocumentType::Other("Custom".to_string()),
            content: "# {{PROJECT_NAME}}\n\nCustom template by {{USER}}".to_string(),
            variables: vec!["PROJECT_NAME".to_string(), "USER".to_string()],
        };
        
        let result = manager.save_template(&custom_template);
        assert!(result.is_ok());
        
        // Verify template was saved
        let template_path = test_dir.join("templates").join("custom_test.md");
        assert!(template_path.exists());
        
        // Verify template can be retrieved
        let retrieved = manager.get_template("custom_test").unwrap();
        assert_eq!(retrieved.name, "custom_test");
        assert_eq!(retrieved.description, "Custom test template");
        
        cleanup_test_dir(&test_dir);
    }
}
