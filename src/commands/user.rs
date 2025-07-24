use crate::commands::cli_auth_helper::{get_cli_auth_helper, require_cli_authentication, get_authenticated_project_path};
use crate::modules::user_manager::{FileAuthManager, RoleManager, Permission};
use crate::utils::get_current_project_path;
use std::io::{self, Write};
use std::process;

/// Handle user management commands
pub fn handle_user_command(args: &[String]) -> Result<(), String> {
    if args.len() < 3 {
        print_user_help();
        return Ok(());
    }

    match args[2].as_str() {
        "add" => handle_user_add(&args[3..]),
        "list" => handle_user_list(&args[3..]),
        "login" => handle_user_login(&args[3..]),
        "logout" => handle_user_logout(&args[3..]),
        "passwd" => handle_user_passwd(&args[3..]),
        "assign-role" => handle_user_assign_role(&args[3..]),
        "remove-role" => handle_user_remove_role(&args[3..]),
        "roles" => handle_user_roles(&args[3..]),
        "permissions" => handle_user_permissions(&args[3..]),
        "session" => handle_user_session(&args[3..]),
        "--help" | "-h" => {
            print_user_help();
            Ok(())
        }
        _ => {
            eprintln!("Error: Unknown user command '{}'", args[2]);
            print_user_help();
            process::exit(1);
        }
    }
}

/// Handle user add command
fn handle_user_add(args: &[String]) -> Result<(), String> {
    let mut username = String::new();
    let mut password = String::new();
    let mut roles = Vec::new();
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--username" | "-u" => {
                if i + 1 >= args.len() {
                    return Err("Missing username value".to_string());
                }
                username = args[i + 1].clone();
                i += 2;
            }
            "--password" | "-p" => {
                if i + 1 >= args.len() {
                    return Err("Missing password value".to_string());
                }
                password = args[i + 1].clone();
                i += 2;
            }
            "--role" | "-r" => {
                if i + 1 >= args.len() {
                    return Err("Missing role value".to_string());
                }
                roles.push(args[i + 1].clone());
                i += 2;
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }
    
    if username.is_empty() {
        print!("Enter username: ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut username).unwrap();
        username = username.trim().to_string();
    }
    
    if password.is_empty() {
        print!("Enter password: ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut password).unwrap();
        password = password.trim().to_string();
    }
    
    // Check if this is initial setup (no users exist) or requires authentication
    let auth_helper = get_cli_auth_helper()
        .map_err(|e| format!("Failed to initialize authentication: {e}"))?;

    if auth_helper.has_any_users() {
        // Require authentication if users already exist
        let _session = require_cli_authentication()
            .map_err(|e| format!("Authentication required: {e}"))?;
    } else {
        println!("🔧 Initial setup detected - creating first admin user without authentication");
    }

    // Use bootstrap mechanism for initial user creation
    if !auth_helper.has_any_users() {
        // Initial setup - create user directly
        auth_helper.create_user_if_not_exists(&username, &password)
            .map_err(|e| format!("Failed to create initial user: {e}"))?;
        return Ok(());
    }

    let project_path = get_authenticated_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;

    let auth_manager = FileAuthManager::from_project_path(&project_path)
        .map_err(|e| format!("Failed to initialize auth manager: {e}"))?;
    
    let role_manager = RoleManager::new(&project_path)
        .map_err(|e| format!("Failed to initialize role manager: {e}"))?;
    
    // Get roles
    let user_roles = if roles.is_empty() {
        None
    } else {
        let mut role_objects = Vec::new();
        for role_name in roles {
            let role = role_manager.get_role_by_name(&role_name)
                .map_err(|e| format!("Failed to get role '{role_name}': {e}"))?;
            role_objects.push(role);
        }
        Some(role_objects)
    };
    
    let user = auth_manager.add_user(&username, &password, user_roles)
        .map_err(|e| format!("Failed to add user: {e}"))?;
    
    println!("✅ User '{}' created successfully", user.username);
    println!("   Created: {}", format_timestamp(user.created_at));
    println!("   Roles: {}", user.roles.iter().map(|r| r.name.as_str()).collect::<Vec<_>>().join(", "));
    
    Ok(())
}

/// Handle user list command
fn handle_user_list(args: &[String]) -> Result<(), String> {
    let mut show_details = false;
    let mut show_permissions = false;
    
    for arg in args {
        match arg.as_str() {
            "--details" | "-d" => show_details = true,
            "--permissions" | "-p" => show_permissions = true,
            _ => return Err(format!("Unknown argument: {arg}")),
        }
    }
    
    // Require authentication to list users
    let _session = require_cli_authentication()
        .map_err(|e| format!("Authentication required: {e}"))?;

    let project_path = get_authenticated_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;

    let auth_manager = FileAuthManager::from_project_path(&project_path)
        .map_err(|e| format!("Failed to initialize auth manager: {e}"))?;
    
    let users = auth_manager.list_users()
        .map_err(|e| format!("Failed to list users: {e}"))?;
    
    println!("📋 User List ({})", users.len());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    for user in users {
        println!("👤 {}", user.username);
        
        if show_details {
            println!("   Created: {}", format_timestamp(user.created_at));
            if let Some(last_login) = user.last_login {
                println!("   Last Login: {}", format_timestamp(last_login));
            } else {
                println!("   Last Login: Never");
            }
        }
        
        println!("   Roles: {}", user.roles.iter().map(|r| r.name.as_str()).collect::<Vec<_>>().join(", "));
        
        if show_permissions {
            let mut all_permissions: Vec<Permission> = Vec::new();
            for role in &user.roles {
                for permission in &role.permissions {
                    if !all_permissions.contains(permission) {
                        all_permissions.push(permission.clone());
                    }
                }
            }
            println!("   Permissions: {}", all_permissions.len());
            for permission in all_permissions {
                println!("     • {}", permission_to_string(&permission));
            }
        }
        
        println!();
    }
    
    Ok(())
}

/// Handle user login command
fn handle_user_login(args: &[String]) -> Result<(), String> {
    let mut username = String::new();
    let mut password = String::new();
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--username" | "-u" => {
                if i + 1 >= args.len() {
                    return Err("Missing username value".to_string());
                }
                username = args[i + 1].clone();
                i += 2;
            }
            "--password" | "-p" => {
                if i + 1 >= args.len() {
                    return Err("Missing password value".to_string());
                }
                password = args[i + 1].clone();
                i += 2;
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }
    
    if username.is_empty() {
        print!("Username: ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut username).unwrap();
        username = username.trim().to_string();
    }
    
    if password.is_empty() {
        print!("Password: ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut password).unwrap();
        password = password.trim().to_string();
    }
    
    let auth_helper = get_cli_auth_helper()
        .map_err(|e| format!("Failed to initialize authentication: {e}"))?;

    let session = if username.is_empty() || password.is_empty() {
        // Interactive login
        auth_helper.interactive_login()
            .map_err(|e| format!("Login failed: {e}"))?
    } else {
        // Direct login
        auth_helper.login(&username, &password)
            .map_err(|e| format!("Login failed: {e}"))?
    };

    println!("✅ Login successful");
    println!("   User: {}", session.username);
    println!("   Session ID: {}", session.session_id);
    println!("   Session Type: {:?}", session.session_type);
    println!("   Roles: {}", session.roles.iter().map(|r| r.name.as_str()).collect::<Vec<_>>().join(", "));
    
    Ok(())
}

/// Handle user logout command
fn handle_user_logout(_args: &[String]) -> Result<(), String> {
    let auth_helper = get_cli_auth_helper()
        .map_err(|e| format!("Failed to initialize authentication: {e}"))?;

    auth_helper.logout()
        .map_err(|e| format!("Logout failed: {e}"))?;

    println!("✅ Logout successful");

    Ok(())
}

/// Handle user password change command
fn handle_user_passwd(args: &[String]) -> Result<(), String> {
    let mut username = String::new();
    let mut new_password = String::new();
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--username" | "-u" => {
                if i + 1 >= args.len() {
                    return Err("Missing username value".to_string());
                }
                username = args[i + 1].clone();
                i += 2;
            }
            "--password" | "-p" => {
                if i + 1 >= args.len() {
                    return Err("Missing password value".to_string());
                }
                new_password = args[i + 1].clone();
                i += 2;
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }
    
    if username.is_empty() {
        print!("Username: ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut username).unwrap();
        username = username.trim().to_string();
    }
    
    if new_password.is_empty() {
        print!("New password: ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut new_password).unwrap();
        new_password = new_password.trim().to_string();
    }
    
    println!("🔧 Password change functionality not yet implemented");
    println!("   Username: {username}");
    println!("   New password length: {}", new_password.len());
    println!("   This feature will be available in the next update.");
    
    Ok(())
}

/// Handle user role assignment command
fn handle_user_assign_role(args: &[String]) -> Result<(), String> {
    let mut username = String::new();
    let mut role_name = String::new();
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--username" | "-u" => {
                if i + 1 >= args.len() {
                    return Err("Missing username value".to_string());
                }
                username = args[i + 1].clone();
                i += 2;
            }
            "--role" | "-r" => {
                if i + 1 >= args.len() {
                    return Err("Missing role value".to_string());
                }
                role_name = args[i + 1].clone();
                i += 2;
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }
    
    if username.is_empty() {
        return Err("Username is required (--username)".to_string());
    }
    
    if role_name.is_empty() {
        return Err("Role name is required (--role)".to_string());
    }
    
    let project_path = get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;
    
    let role_manager = RoleManager::new(&project_path)
        .map_err(|e| format!("Failed to initialize role manager: {e}"))?;
    
    role_manager.assign_role(&username, &role_name)
        .map_err(|e| format!("Failed to assign role: {e}"))?;
    
    println!("✅ Role '{role_name}' assigned to user '{username}'");
    
    Ok(())
}

/// Handle user role removal command
fn handle_user_remove_role(args: &[String]) -> Result<(), String> {
    let mut username = String::new();
    let mut role_name = String::new();
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--username" | "-u" => {
                if i + 1 >= args.len() {
                    return Err("Missing username value".to_string());
                }
                username = args[i + 1].clone();
                i += 2;
            }
            "--role" | "-r" => {
                if i + 1 >= args.len() {
                    return Err("Missing role value".to_string());
                }
                role_name = args[i + 1].clone();
                i += 2;
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }
    
    if username.is_empty() {
        return Err("Username is required (--username)".to_string());
    }
    
    if role_name.is_empty() {
        return Err("Role name is required (--role)".to_string());
    }
    
    let project_path = get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;
    
    let role_manager = RoleManager::new(&project_path)
        .map_err(|e| format!("Failed to initialize role manager: {e}"))?;
    
    role_manager.remove_role(&username, &role_name)
        .map_err(|e| format!("Failed to remove role: {e}"))?;
    
    println!("✅ Role '{role_name}' removed from user '{username}'");
    
    Ok(())
}

/// Handle user roles command
fn handle_user_roles(args: &[String]) -> Result<(), String> {
    let mut username = String::new();
    let mut list_all = false;
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--username" | "-u" => {
                if i + 1 >= args.len() {
                    return Err("Missing username value".to_string());
                }
                username = args[i + 1].clone();
                i += 2;
            }
            "--list" | "-l" => {
                list_all = true;
                i += 1;
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }
    
    let project_path = get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;
    
    let role_manager = RoleManager::new(&project_path)
        .map_err(|e| format!("Failed to initialize role manager: {e}"))?;
    
    if list_all {
        // List all available roles
        let roles = role_manager.get_available_roles();
        let descriptions = role_manager.get_role_descriptions();
        
        println!("🔐 Available Roles");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        for role in roles {
            println!("📋 {}", role.name);
            if let Some(description) = descriptions.get(&role.name) {
                println!("   Description: {description}");
            }
            println!("   Permissions: {}", role.permissions.len());
            for permission in &role.permissions {
                println!("     • {}", permission_to_string(permission));
            }
            println!();
        }
    } else if !username.is_empty() {
        // List roles for specific user
        let user_roles = role_manager.get_user_roles(&username)
            .map_err(|e| format!("Failed to get user roles: {e}"))?;
        
        println!("👤 Roles for user '{username}'");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        for role in user_roles {
            println!("📋 {}", role.name);
            println!("   Permissions: {}", role.permissions.len());
            for permission in &role.permissions {
                println!("     • {}", permission_to_string(permission));
            }
            println!();
        }
    } else {
        return Err("Either --username or --list must be specified".to_string());
    }
    
    Ok(())
}

/// Handle user permissions command
fn handle_user_permissions(args: &[String]) -> Result<(), String> {
    let mut username = String::new();
    let mut list_all = false;
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--username" | "-u" => {
                if i + 1 >= args.len() {
                    return Err("Missing username value".to_string());
                }
                username = args[i + 1].clone();
                i += 2;
            }
            "--list" | "-l" => {
                list_all = true;
                i += 1;
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }
    
    let project_path = get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;
    
    let role_manager = RoleManager::new(&project_path)
        .map_err(|e| format!("Failed to initialize role manager: {e}"))?;
    
    if list_all {
        // List all available permissions
        let descriptions = role_manager.get_permission_descriptions();
        
        println!("🔑 Available Permissions");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        for (permission, description) in descriptions {
            println!("🔐 {permission}");
            println!("   {description}");
            println!();
        }
    } else if !username.is_empty() {
        // List permissions for specific user
        let permissions = role_manager.get_user_permissions(&username)
            .map_err(|e| format!("Failed to get user permissions: {e}"))?;
        
        println!("👤 Permissions for user '{username}'");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        for permission in permissions {
            println!("🔑 {}", permission_to_string(&permission));
        }
    } else {
        return Err("Either --username or --list must be specified".to_string());
    }
    
    Ok(())
}

/// Handle user session command
fn handle_user_session(args: &[String]) -> Result<(), String> {
    let mut session_id = String::new();
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--session" | "-s" => {
                if i + 1 >= args.len() {
                    return Err("Missing session ID value".to_string());
                }
                session_id = args[i + 1].clone();
                i += 2;
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
    }
    
    if session_id.is_empty() {
        return Err("Session ID is required (--session)".to_string());
    }
    
    let project_path = get_current_project_path()
        .map_err(|e| format!("Failed to get project path: {e}"))?;
    
    let auth_manager = FileAuthManager::from_project_path(&project_path)
        .map_err(|e| format!("Failed to initialize auth manager: {e}"))?;
    
    let session = auth_manager.validate_session(&session_id)
        .map_err(|e| format!("Session validation failed: {e}"))?;
    
    println!("✅ Session Information");
    println!("   Session ID: {}", session.session_id);
    println!("   User: {}", session.username);
    println!("   Login Time: {}", format_timestamp(session.login_time));
    println!("   Last Activity: {}", format_timestamp(session.last_activity));
    println!("   Roles: {}", session.roles.iter().map(|r| r.name.as_str()).collect::<Vec<_>>().join(", "));
    
    Ok(())
}

/// Format timestamp for display
fn format_timestamp(timestamp: u64) -> String {
    use std::time::UNIX_EPOCH;
    
    let duration = std::time::Duration::from_secs(timestamp);
    let datetime = UNIX_EPOCH + duration;
    
    format!("{datetime:?}")
}

/// Convert permission to string
const fn permission_to_string(permission: &Permission) -> &str {
    match permission {
        Permission::ReadDocuments => "Read Documents",
        Permission::WriteDocuments => "Write Documents",
        Permission::DeleteDocuments => "Delete Documents",
        Permission::ReadRisks => "Read Risks",
        Permission::WriteRisks => "Write Risks",
        Permission::DeleteRisks => "Delete Risks",
        Permission::ReadTrace => "Read Traceability",
        Permission::WriteTrace => "Write Traceability",
        Permission::DeleteTrace => "Delete Traceability",
        Permission::ReadAudit => "Read Audit",
        Permission::ExportAudit => "Export Audit",
        Permission::ManageUsers => "Manage Users",
        Permission::GenerateReports => "Generate Reports",
        Permission::UserManagement => "User Management",
        Permission::ProjectManagement => "Project Management",
        Permission::DocumentManagement => "Document Management",
        Permission::RiskManagement => "Risk Management",
        Permission::AuditAccess => "Audit Access",
        Permission::SystemConfiguration => "System Configuration",
    }
}

/// Print user help
fn print_user_help() {
    println!("QMS User Management Commands");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("USAGE:");
    println!("  qms user <command> [options]");
    println!();
    println!("COMMANDS:");
    println!("  add                     Add new user");
    println!("  list                    List users");
    println!("  login                   Login user");
    println!("  logout                  Logout user");
    println!("  passwd                  Change password");
    println!("  assign-role             Assign role to user");
    println!("  remove-role             Remove role from user");
    println!("  roles                   Show roles");
    println!("  permissions             Show permissions");
    println!("  session                 Show session information");
    println!();
    println!("USER MANAGEMENT:");
    println!("  qms user add --username <name> --password <pass> [--role <role>]");
    println!("  qms user list [--details] [--permissions]");
    println!("  qms user login --username <name> --password <pass>");
    println!("  qms user logout --session <session-id>");
    println!();
    println!("ROLE MANAGEMENT:");
    println!("  qms user assign-role --username <name> --role <role>");
    println!("  qms user remove-role --username <name> --role <role>");
    println!("  qms user roles --list                  # List all available roles");
    println!("  qms user roles --username <name>       # List user's roles");
    println!("  qms user permissions --list            # List all permissions");
    println!("  qms user permissions --username <name> # List user's permissions");
    println!();
    println!("SESSION MANAGEMENT:");
    println!("  qms user session --session <session-id>");
    println!();
    println!("AVAILABLE ROLES:");
    println!("  Administrator          Full system access");
    println!("  QualityEngineer        Quality management functions");
    println!("  Developer              Development-focused access");
    println!("  Auditor                Read-only audit access");
    println!();
    println!("EXAMPLES:");
    println!("  qms user add --username jdoe --password secret123 --role QualityEngineer");
    println!("  qms user login --username jdoe --password secret123");
    println!("  qms user assign-role --username jdoe --role Administrator");
    println!("  qms user list --details --permissions");
    println!("  qms user roles --list");
    println!();
    println!("For more information, visit: https://qms.medical-device.com/docs/user-management");
}
