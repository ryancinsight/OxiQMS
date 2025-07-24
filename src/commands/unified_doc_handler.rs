// Unified Document Command Handler - Bridge between CLI and Web execution
// Follows SOLID, CUPID, GRASP, ACID, KISS, DRY, SOC, and YAGNI principles

use crate::prelude::*;
use crate::commands::command_execution_context::{CommandExecutionContext, UnifiedCommandHandler};
use crate::modules::document_control::service::DocumentService;
use crate::modules::document_control::document::DocumentType;
use std::collections::HashMap;

/// Unified document command handler that works in both CLI and Web contexts
pub struct UnifiedDocumentHandler;

impl UnifiedDocumentHandler {
    pub fn new() -> Self {
        Self
    }
    
    /// Execute document command with unified context
    pub fn execute_with_context(context: &mut CommandExecutionContext, args: &[String]) -> QmsResult<()> {
        if args.len() < 3 {
            Self::print_help(context);
            return Ok(());
        }

        match args[2].as_str() {
            "add" => Self::handle_add(context, &args[3..]),
            "list" => Self::handle_list(context, &args[3..]),
            "view" => Self::handle_view(context, &args[3..]),
            "update" => Self::handle_update(context, &args[3..]),
            "remove" => Self::handle_remove(context, &args[3..]),
            "search" => Self::handle_search(context, &args[3..]),
            "--help" | "-h" => {
                Self::print_help(context);
                Ok(())
            }
            _ => {
                let error_msg = format!("Unknown document command '{}'", args[2]);
                context.eprintln(&error_msg);
                Self::print_help(context);
                context.exit_with_error(&error_msg, 1)
            }
        }
    }
    
    /// Handle document add command
    fn handle_add(context: &mut CommandExecutionContext, args: &[String]) -> QmsResult<()> {
        // Parse arguments
        let mut title = None;
        let mut doc_type = None;
        let mut content = None;
        let mut author = None;
        
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--title" | "-t" => {
                    if i + 1 < args.len() {
                        title = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        return context.exit_with_error("--title requires a value", 1);
                    }
                }
                "--type" => {
                    if i + 1 < args.len() {
                        doc_type = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        return context.exit_with_error("--type requires a value", 1);
                    }
                }
                "--content" | "-c" => {
                    if i + 1 < args.len() {
                        content = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        return context.exit_with_error("--content requires a value", 1);
                    }
                }
                "--author" | "-a" => {
                    if i + 1 < args.len() {
                        author = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        return context.exit_with_error("--author requires a value", 1);
                    }
                }
                _ => {
                    let error_msg = format!("Unknown argument '{}'", args[i]);
                    return context.exit_with_error(&error_msg, 1);
                }
            }
        }
        
        // Validate required arguments
        let title = title.ok_or_else(|| QmsError::validation_error("Title is required (--title)"))?;
        let doc_type = doc_type.unwrap_or_else(|| "General".to_string());
        let content = content.unwrap_or_else(|| "".to_string());
        
        // Use session author if available, otherwise use provided author
        let author = if let Ok(session) = context.get_session() {
            session.username.clone()
        } else {
            author.unwrap_or_else(|| "Unknown".to_string())
        };
        
        // Create document service
        let project_path = if let Ok(path) = context.get_project_path() {
            path.clone()
        } else {
            context.working_directory.clone()
        };

        let doc_service = DocumentService::new(project_path);
        
        // Create document
        let document_type = Self::parse_document_type(&doc_type)?;
        let document = doc_service.create_document(title.clone(), content.clone(), document_type, author.clone())?;

        // Output result
        context.println(&format!("✅ Document created successfully"));
        context.println(&format!("   ID: {}", document.id));
        context.println(&format!("   Title: {}", title));
        context.println(&format!("   Type: {}", doc_type));
        context.println(&format!("   Author: {}", author));

        // Add structured data for web context
        if context.is_web_mode() {
            let mut doc_data = HashMap::new();
            doc_data.insert("id".to_string(), crate::commands::command_execution_context::serde_json::Value::String(document.id));
            doc_data.insert("title".to_string(), crate::commands::command_execution_context::serde_json::Value::String(title));
            doc_data.insert("type".to_string(), crate::commands::command_execution_context::serde_json::Value::String(doc_type));
            doc_data.insert("author".to_string(), crate::commands::command_execution_context::serde_json::Value::String(author));

            context.add_data("document", crate::commands::command_execution_context::serde_json::Value::Object(doc_data));
        }
        
        Ok(())
    }
    
    /// Handle document list command
    fn handle_list(context: &mut CommandExecutionContext, args: &[String]) -> QmsResult<()> {
        // Parse filters
        let mut doc_type_filter = None;
        let mut author_filter = None;
        
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--type" => {
                    if i + 1 < args.len() {
                        doc_type_filter = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        return context.exit_with_error("--type requires a value", 1);
                    }
                }
                "--author" => {
                    if i + 1 < args.len() {
                        author_filter = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        return context.exit_with_error("--author requires a value", 1);
                    }
                }
                _ => {
                    let error_msg = format!("Unknown argument '{}'", args[i]);
                    return context.exit_with_error(&error_msg, 1);
                }
            }
        }
        
        // Create document service
        let project_path = if let Ok(path) = context.get_project_path() {
            path.clone()
        } else {
            context.working_directory.clone()
        };

        let doc_service = DocumentService::new(project_path);
        
        // List documents
        let documents = doc_service.list_documents()?;
        
        // Apply filters
        let filtered_docs: Vec<_> = documents.into_iter()
            .filter(|doc| {
                if let Some(ref type_filter) = doc_type_filter {
                    if doc.doc_type != *type_filter {
                        return false;
                    }
                }
                if let Some(ref author_filter) = author_filter {
                    if doc.author != *author_filter {
                        return false;
                    }
                }
                true
            })
            .collect();
        
        // Output results
        if filtered_docs.is_empty() {
            context.println("No documents found");
        } else {
            context.println(&format!("Found {} document(s):", filtered_docs.len()));
            context.println("");
            
            for doc in &filtered_docs {
                context.println(&format!("📄 {} ({})", doc.title, doc.id));
                context.println(&format!("   Type: {}", doc.doc_type));
                context.println(&format!("   Author: {}", doc.author));
                context.println(&format!("   Version: {}", doc.version));
                context.println(&format!("   Status: {}", doc.status));
                context.println("");
            }
        }
        
        // Add structured data for web context
        if context.is_web_mode() {
            let docs_array: Vec<_> = filtered_docs.iter().map(|doc| {
                let mut doc_obj = HashMap::new();
                doc_obj.insert("id".to_string(), crate::commands::command_execution_context::serde_json::Value::String(doc.id.clone()));
                doc_obj.insert("title".to_string(), crate::commands::command_execution_context::serde_json::Value::String(doc.title.clone()));
                doc_obj.insert("type".to_string(), crate::commands::command_execution_context::serde_json::Value::String(doc.doc_type.clone()));
                doc_obj.insert("author".to_string(), crate::commands::command_execution_context::serde_json::Value::String(doc.author.clone()));
                doc_obj.insert("version".to_string(), crate::commands::command_execution_context::serde_json::Value::String(doc.version.clone()));
                doc_obj.insert("status".to_string(), crate::commands::command_execution_context::serde_json::Value::String(doc.status.clone()));
                crate::commands::command_execution_context::serde_json::Value::Object(doc_obj)
            }).collect();
            
            context.add_data("documents", crate::commands::command_execution_context::serde_json::Value::Array(docs_array));
            context.add_data("count", crate::commands::command_execution_context::serde_json::Value::Number(filtered_docs.len() as f64));
        }
        
        Ok(())
    }
    
    /// Handle document view command
    fn handle_view(context: &mut CommandExecutionContext, args: &[String]) -> QmsResult<()> {
        if args.is_empty() {
            return context.exit_with_error("Document ID is required", 1);
        }
        
        let doc_id = &args[0];
        
        // Create document service
        let project_path = if let Ok(path) = context.get_project_path() {
            path.clone()
        } else {
            context.working_directory.clone()
        };

        let doc_service = DocumentService::new(project_path);
        
        // Get document
        let document = doc_service.read_document(doc_id)?;
        
        // Output document details
        context.println(&format!("📄 Document: {}", document.title));
        context.println(&format!("   ID: {}", document.id));
        context.println(&format!("   Type: {:?}", document.doc_type));
        context.println(&format!("   Author: {}", document.created_by));
        context.println(&format!("   Version: {}", document.version));
        context.println(&format!("   Status: {:?}", document.status));
        context.println(&format!("   Created: {}", document.created_at));
        context.println(&format!("   Updated: {}", document.updated_at));
        context.println("");
        context.println("Content:");
        context.println(&"─".repeat(50));
        context.println(&document.content);
        context.println(&"─".repeat(50));
        
        // Add structured data for web context
        if context.is_web_mode() {
            let mut doc_data = HashMap::new();
            doc_data.insert("id".to_string(), crate::commands::command_execution_context::serde_json::Value::String(document.id));
            doc_data.insert("title".to_string(), crate::commands::command_execution_context::serde_json::Value::String(document.title));
            doc_data.insert("type".to_string(), crate::commands::command_execution_context::serde_json::Value::String(format!("{:?}", document.doc_type)));
            doc_data.insert("author".to_string(), crate::commands::command_execution_context::serde_json::Value::String(document.created_by));
            doc_data.insert("version".to_string(), crate::commands::command_execution_context::serde_json::Value::String(document.version));
            doc_data.insert("status".to_string(), crate::commands::command_execution_context::serde_json::Value::String(format!("{:?}", document.status)));
            doc_data.insert("content".to_string(), crate::commands::command_execution_context::serde_json::Value::String(document.content));
            doc_data.insert("created_at".to_string(), crate::commands::command_execution_context::serde_json::Value::String(document.created_at));
            doc_data.insert("updated_at".to_string(), crate::commands::command_execution_context::serde_json::Value::String(document.updated_at));

            context.add_data("document", crate::commands::command_execution_context::serde_json::Value::Object(doc_data));
        }
        
        Ok(())
    }
    
    /// Handle document update command (placeholder)
    fn handle_update(context: &mut CommandExecutionContext, _args: &[String]) -> QmsResult<()> {
        context.println("Document update functionality not yet implemented");
        Ok(())
    }
    
    /// Handle document remove command (placeholder)
    fn handle_remove(context: &mut CommandExecutionContext, _args: &[String]) -> QmsResult<()> {
        context.println("Document remove functionality not yet implemented");
        Ok(())
    }
    
    /// Handle document search command (placeholder)
    fn handle_search(context: &mut CommandExecutionContext, _args: &[String]) -> QmsResult<()> {
        context.println("Document search functionality not yet implemented");
        Ok(())
    }
    
    /// Print help message
    fn print_help(context: &mut CommandExecutionContext) {
        context.println("QMS Document Management");
        context.println("");
        context.println("Usage: qms doc <command> [options]");
        context.println("");
        context.println("Commands:");
        context.println("  add     Add a new document");
        context.println("  list    List documents");
        context.println("  view    View document details");
        context.println("  update  Update a document");
        context.println("  remove  Remove a document");
        context.println("  search  Search documents");
        context.println("");
        context.println("Examples:");
        context.println("  qms doc add --title \"User Manual\" --type Manual --content \"Content here\"");
        context.println("  qms doc list --type Manual");
        context.println("  qms doc view DOC-001");
    }
    
    /// Parse document type from string
    fn parse_document_type(type_str: &str) -> QmsResult<DocumentType> {
        match type_str.to_lowercase().as_str() {
            "srs" | "softwarerequirementsspecification" => Ok(DocumentType::SoftwareRequirementsSpecification),
            "sdd" | "softwaredesigndescription" => Ok(DocumentType::SoftwareDesignDescription),
            "vv" | "verificationandvalidation" => Ok(DocumentType::VerificationAndValidation),
            "rmf" | "riskmanagementfile" => Ok(DocumentType::RiskManagementFile),
            "dhf" | "designhistoryfile" => Ok(DocumentType::DesignHistoryFile),
            "ur" | "userrequirements" => Ok(DocumentType::UserRequirements),
            "tp" | "testprotocol" => Ok(DocumentType::TestProtocol),
            "tr" | "testreport" => Ok(DocumentType::TestReport),
            _ => Ok(DocumentType::Other(type_str.to_string())),
        }
    }
}

impl UnifiedCommandHandler for UnifiedDocumentHandler {
    fn execute_unified(&self, context: &mut CommandExecutionContext, args: &[String]) -> QmsResult<()> {
        Self::execute_with_context(context, args)
    }
    
    fn command_name(&self) -> &'static str {
        "doc"
    }
    
    fn help_text(&self) -> &'static str {
        "Document management commands"
    }
}

impl Default for UnifiedDocumentHandler {
    fn default() -> Self {
        Self::new()
    }
}
