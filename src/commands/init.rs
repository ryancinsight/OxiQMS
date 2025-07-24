use crate::commands::cli_auth_helper::{get_cli_auth_helper, require_cli_authentication};
use crate::modules::repository::project::{Repository, RepositoryError};
use crate::utils::format_timestamp;
use std::process;

pub fn handle_init_command(args: &[String]) -> Result<(), RepositoryError> {
    // Require authentication before creating projects
    let session = require_cli_authentication()
        .map_err(|e| RepositoryError::InvalidInput(format!("Authentication required: {e}")))?;

    let mut project_name = None;
    let mut custom_path = None;

    // Parse arguments
    let mut i = 2; // Skip "qms" and "init"
    while i < args.len() {
        match args[i].as_str() {
            "--name" => {
                if i + 1 < args.len() {
                    project_name = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err(RepositoryError::InvalidInput(
                        "--name requires a value".to_string(),
                    ));
                }
            }
            "--path" => {
                if i + 1 < args.len() {
                    custom_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err(RepositoryError::InvalidInput(
                        "--path requires a value".to_string(),
                    ));
                }
            }
            "--help" | "-h" => {
                print_init_help();
                process::exit(0);
            }
            _ => {
                return Err(RepositoryError::InvalidInput(format!(
                    "Unknown option '{}'",
                    args[i]
                )));
            }
        }
    }

    let name = match project_name {
        Some(n) => n,
        None => {
            return Err(RepositoryError::InvalidInput(
                "Project name is required. Use --name <name>".to_string(),
            ));
        }
    };

    // Get user-specific project directory
    let auth_helper = get_cli_auth_helper()
        .map_err(|e| RepositoryError::InvalidInput(format!("Failed to get auth helper: {e}")))?;

    let user_projects_dir = auth_helper.get_user_project_directory(&session.username)
        .map_err(|e| RepositoryError::InvalidInput(format!("Failed to get user project directory: {e}")))?;

    // Use custom path if provided, otherwise use user-specific directory
    let project_base_path = custom_path.as_deref().map(|p| std::path::Path::new(p).to_path_buf())
        .unwrap_or(user_projects_dir);

    match Repository::init_project_in_directory(&name, &project_base_path) {
        Ok(project) => {
            println!("✓ QMS project '{name}' initialized successfully");
            println!("  Project ID: {}", project.id);
            println!("  Location: {}", project.path.display());
            println!("  Created: {}", format_timestamp(project.created_at));
            println!("\nNext steps:");
            println!("  1. Add documents: qms doc add --file <file> --title <title>");
            println!("  2. Create requirements: qms req create --title <title>");
            println!("  3. Define risks: qms risk create --description <desc>");
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn print_init_help() {
    println!("Initialize a new QMS project\n");
    println!("USAGE:");
    println!("    qms init --name <name> [--path <path>]\n");
    println!("OPTIONS:");
    println!("    --name <name>    Project name (required, max 100 characters)");
    println!("    --path <path>    Custom project location (optional)");
    println!("    -h, --help       Show this help message\n");
    println!("EXAMPLES:");
    println!("    qms init --name \"Insulin Pump Device\"");
    println!("    qms init --name \"Test Project\" --path /custom/location");
}
