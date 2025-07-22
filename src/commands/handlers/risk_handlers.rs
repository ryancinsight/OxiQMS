/// SOLID Principles Enhancement: Risk Command Handlers
/// 
/// This module breaks down the monolithic risk command handler into focused,
/// single-responsibility handlers following SOLID principles.

use crate::prelude::*;
use crate::commands::command_handler_trait::{CommandHandler, BaseCommandHandler, CommandContext};
use crate::modules::risk_manager::{RiskManager, RiskSeverity, RiskOccurrence, RiskDetectability};
use crate::impl_command_handler;

/// Risk Creation Handler - Single Responsibility Principle
/// Focuses solely on creating new risks
pub struct RiskCreateHandler {
    context: CommandContext,
}

impl RiskCreateHandler {
    pub fn new(context: CommandContext) -> Self {
        Self { context }
    }
}

impl BaseCommandHandler for RiskCreateHandler {
    fn do_execute(&self, args: &[String]) -> QmsResult<()> {
        if args.len() < 6 {
            return Err(QmsError::validation_error(
                "Usage: create --hazard <hazard> --situation <situation> --harm <harm>"
            ));
        }
        
        let mut hazard = String::new();
        let mut situation = String::new();
        let mut harm = String::new();
        
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--hazard" if i + 1 < args.len() => {
                    hazard = args[i + 1].clone();
                    i += 2;
                }
                "--situation" if i + 1 < args.len() => {
                    situation = args[i + 1].clone();
                    i += 2;
                }
                "--harm" if i + 1 < args.len() => {
                    harm = args[i + 1].clone();
                    i += 2;
                }
                _ => i += 1,
            }
        }
        
        if hazard.is_empty() || situation.is_empty() || harm.is_empty() {
            return Err(QmsError::validation_error(
                "All fields (hazard, situation, harm) are required"
            ));
        }
        
        let mut risk_manager = RiskManager::new(&self.context.project_path)?;
        let risk_item = risk_manager.create_risk(&hazard, &situation, &harm)?;
        
        println!("✅ Risk created successfully!");
        println!("Risk ID: {}", risk_item.id);
        
        Ok(())
    }
}

impl_command_handler!(
    RiskCreateHandler,
    "create",
    "Create a new risk with hazard, situation, and harm description"
);

/// Risk Assessment Handler - Single Responsibility Principle
/// Focuses solely on risk assessment operations
pub struct RiskAssessHandler {
    context: CommandContext,
}

impl RiskAssessHandler {
    pub fn new(context: CommandContext) -> Self {
        Self { context }
    }
}

impl BaseCommandHandler for RiskAssessHandler {
    fn do_execute(&self, args: &[String]) -> QmsResult<()> {
        if args.len() < 7 {
            return Err(QmsError::validation_error(
                "Usage: assess <risk_id> --severity <1-5> --occurrence <1-5> --detectability <1-5>"
            ));
        }
        
        let risk_id = &args[0];
        let mut severity = None;
        let mut occurrence = None;
        let mut detectability = None;
        
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--severity" if i + 1 < args.len() => {
                    severity = args[i + 1].parse::<u8>().ok();
                    i += 2;
                }
                "--occurrence" if i + 1 < args.len() => {
                    occurrence = args[i + 1].parse::<u8>().ok();
                    i += 2;
                }
                "--detectability" if i + 1 < args.len() => {
                    detectability = args[i + 1].parse::<u8>().ok();
                    i += 2;
                }
                _ => i += 1,
            }
        }
        
        let severity = severity.ok_or_else(|| QmsError::validation_error("Invalid severity value"))?;
        let occurrence = occurrence.ok_or_else(|| QmsError::validation_error("Invalid occurrence value"))?;
        let detectability = detectability.ok_or_else(|| QmsError::validation_error("Invalid detectability value"))?;
        
        let severity_enum = match severity {
            1 => RiskSeverity::Negligible,
            2 => RiskSeverity::Minor,
            3 => RiskSeverity::Major,
            4 => RiskSeverity::Critical,
            5 => RiskSeverity::Catastrophic,
            _ => return Err(QmsError::validation_error("Severity must be 1-5")),
        };

        let occurrence_enum = match occurrence {
            1 => RiskOccurrence::Improbable,
            2 => RiskOccurrence::Remote,
            3 => RiskOccurrence::Occasional,
            4 => RiskOccurrence::Probable,
            5 => RiskOccurrence::Frequent,
            _ => return Err(QmsError::validation_error("Occurrence must be 1-5")),
        };
        
        let detectability_enum = match detectability {
            1 => RiskDetectability::VeryHigh,
            2 => RiskDetectability::High,
            3 => RiskDetectability::Moderate,
            4 => RiskDetectability::Low,
            5 => RiskDetectability::VeryLow,
            _ => return Err(QmsError::validation_error("Detectability must be 1-5")),
        };
        
        let mut risk_manager = RiskManager::new(&self.context.project_path)?;
        risk_manager.assess_risk(risk_id, Some(severity_enum), Some(occurrence_enum), Some(detectability_enum))?;
        
        println!("✅ Risk assessment completed for {}", risk_id);
        
        Ok(())
    }
}

impl_command_handler!(
    RiskAssessHandler,
    "assess",
    "Assess a risk with severity, occurrence, and detectability ratings"
);

/// Risk List Handler - Single Responsibility Principle
/// Focuses solely on listing and filtering risks
pub struct RiskListHandler {
    context: CommandContext,
}

impl RiskListHandler {
    pub fn new(context: CommandContext) -> Self {
        Self { context }
    }
}

impl BaseCommandHandler for RiskListHandler {
    fn do_execute(&self, args: &[String]) -> QmsResult<()> {
        let risk_manager = RiskManager::new(&self.context.project_path)?;
        let risks = risk_manager.list_risks(None)?;
        
        if risks.is_empty() {
            println!("No risks found in the current project.");
            return Ok(());
        }
        
        // Apply filters if provided
        let filtered_risks = if args.is_empty() {
            risks
        } else {
            // Simple filtering implementation - can be enhanced
            risks.into_iter().filter(|risk| {
                // Filter logic based on args
                true // Placeholder - implement actual filtering
            }).collect()
        };
        
        println!("📋 Risk Register");
        println!("================");
        println!();
        
        for risk in filtered_risks {
            println!("🔸 {} - {}", risk.id, risk.description);
            println!("   Hazard ID: {}", risk.hazard_id);
            println!("   Severity: {:?}", risk.severity);
            println!("   RPN: {} | Status: {:?}", risk.rpn, risk.status);
            println!();
        }
        
        Ok(())
    }
}

impl_command_handler!(
    RiskListHandler,
    "list",
    "List all risks in the current project with optional filtering"
);

/// Risk View Handler - Single Responsibility Principle
/// Focuses solely on displaying detailed risk information
pub struct RiskViewHandler {
    context: CommandContext,
}

impl RiskViewHandler {
    pub fn new(context: CommandContext) -> Self {
        Self { context }
    }
}

impl BaseCommandHandler for RiskViewHandler {
    fn do_execute(&self, args: &[String]) -> QmsResult<()> {
        if args.is_empty() {
            return Err(QmsError::validation_error("Usage: view <risk_id>"));
        }
        
        let risk_id = &args[0];
        let risk_manager = RiskManager::new(&self.context.project_path)?;
        let risk = risk_manager.load_risk(risk_id)?;
        
        println!("🔍 Risk Details: {}", risk.id);
        println!("================");
        println!();
        println!("Hazard: {}", risk.hazard_description);
        println!("Situation: {}", risk.hazardous_situation);
        println!("Harm: {}", risk.harm);
        println!();
        println!("Assessment:");
        println!("  Severity: {:?} ({})", risk.severity, risk.severity.clone() as u8);
        println!("  Occurrence: {:?} ({})", risk.occurrence, risk.occurrence.clone() as u8);
        println!("  Detectability: {:?} ({})", risk.detectability, risk.detectability.clone() as u8);
        println!("  RPN: {}", risk.risk_priority_number);
        println!();
        println!("Status: {:?}", risk.risk_status);
        println!("Created: {}", risk.created_at);
        println!("Updated: {}", risk.updated_at);
        
        Ok(())
    }
}

impl_command_handler!(
    RiskViewHandler,
    "view",
    "View detailed information about a specific risk"
);

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    fn create_test_context() -> CommandContext {
        CommandContext {
            project_path: PathBuf::from("test"),
            user_id: Some("test_user".to_string()),
            session_id: Some("test_session".to_string()),
        }
    }
    
    #[test]
    fn test_risk_create_handler_validation() {
        let handler = RiskCreateHandler::new(create_test_context());
        let result = handler.do_execute(&[]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_risk_assess_handler_validation() {
        let handler = RiskAssessHandler::new(create_test_context());
        let result = handler.do_execute(&[]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_risk_view_handler_validation() {
        let handler = RiskViewHandler::new(create_test_context());
        let result = handler.do_execute(&[]);
        assert!(result.is_err());
    }
}
