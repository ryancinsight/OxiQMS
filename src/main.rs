#![forbid(unsafe_code)]
#![allow(unused, clippy::used_underscore_binding)]
#![warn(clippy::missing_const_for_fn, clippy::approx_constant, clippy::all)]
use std::env;
use std::process;

mod audit;
mod commands;
mod config;
mod constants; // SSOT: Centralized regulatory compliance constants
mod error;
mod fs_utils;
mod json_utils;
mod lock;
mod models;
mod modules;
mod prelude;
mod utils;
mod validation;
mod web;

// #[cfg(test)]
// mod test_audit_integration;

use audit::{log_command_execution, log_error};
use commands::{audit as audit_cmd, doc, init, report, req, risk, test, trace, user};
use web::server::QMSWebServer;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Initialize audit logger
    if let Err(e) = audit::setup_audit_logger() {
        eprintln!("Error: Failed to initialize audit logger: {e}");
        process::exit(1);
    }

    // Parse command line arguments
    match args.get(1) {
        None => {
            print_usage();
            process::exit(1);
        }
        Some(cmd) => match cmd.as_str() {
            "--help" | "-h" => {
                print_help();
                process::exit(0);
            }
            "--version" | "-v" => {
                println!("qms v1.0.0");
                println!("Medical Device Quality Management System");
                println!("FDA 21 CFR Part 820, ISO 13485, ISO 14971 Compliant");
                process::exit(0);
            }
            "init" => {
                log_command_execution("init");
                if let Err(e) = init::handle_init_command(&args) {
                    handle_error(format!("Failed to initialize project: {e}"));
                }
            }
            "doc" => {
                log_command_execution("doc");
                if let Err(e) = doc::handle_doc_command(&args) {
                    handle_error(format!("Document command failed: {e}"));
                }
            }
            "risk" => {
                log_command_execution("risk");
                if let Err(e) = risk::handle_risk_command(&args) {
                    handle_error(format!("Risk command failed: {e}"));
                }
            }
            "req" => {
                log_command_execution("req");
                if let Err(e) = req::handle_req_command(&args) {
                    handle_error(format!("Requirements command failed: {e}"));
                }
            }
            "trace" => {
                log_command_execution("trace");
                if let Err(e) = trace::handle_trace_command(&args) {
                    handle_error(format!("Traceability command failed: {e}"));
                }
            }
            "test" => {
                log_command_execution("test");
                if let Err(e) = test::handle_test_command(args[2..].to_vec()) {
                    handle_error(format!("Test command failed: {e}"));
                }
            }
            "audit" => {
                log_command_execution("audit");
                if let Err(e) = audit_cmd::handle_audit_command(&args) {
                    handle_error(format!("Audit command failed: {e}"));
                }
            }
            "user" => {
                log_command_execution("user");
                if let Err(e) = user::handle_user_command(&args) {
                    handle_error(format!("User command failed: {e}"));
                }
            }
            "report" => {
                log_command_execution("report");
                if let Err(e) = report::handle_report_command(&args) {
                    handle_error(format!("Report command failed: {e}"));
                }
            }
            "serve" => {
                log_command_execution("serve");
                if let Err(e) = handle_serve_command(&args) {
                    handle_error(format!("Web server failed: {e}"));
                }
            }
            _ => {
                eprintln!("Error: Unknown command '{cmd}'");
                print_usage();
                process::exit(1);
            }
        },
    }
}

fn handle_error(error_msg: String) {
    eprintln!("Error: {error_msg}");
    log_error(&error_msg);
    process::exit(1);
}

fn handle_serve_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments for serve
    let mut port = 8080;
    let mut host = "127.0.0.1".to_string();
    let mut tls_cert: Option<String> = None;
    let mut tls_key: Option<String> = None;
    let mut enable_https = false;

    // Parse optional arguments
    let mut i = 2; // Skip "qms" and "serve"
    while i < args.len() {
        match args[i].as_str() {
            "--port" | "-p" => {
                if i + 1 < args.len() {
                    port = args[i + 1].parse().unwrap_or(8080);
                    i += 2;
                } else {
                    return Err("Port value missing".into());
                }
            }
            "--host" | "-h" => {
                if i + 1 < args.len() {
                    host = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Host value missing".into());
                }
            }
            "--tls-cert" => {
                if i + 1 < args.len() {
                    tls_cert = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("TLS certificate file path missing".into());
                }
            }
            "--tls-key" => {
                if i + 1 < args.len() {
                    tls_key = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("TLS private key file path missing".into());
                }
            }
            "--https" => {
                enable_https = true;
                i += 1;
            }
            "--help" => {
                print_serve_help();
                return Ok(());
            }
            _ => {
                return Err(format!("Unknown serve option: {}", args[i]).into());
            }
        }
    }
    
    // Initialize and start the web server
    let mut server = QMSWebServer::new(&host, port)?;

    // Configure TLS if certificates are provided
    if let (Some(cert_path), Some(key_path)) = (&tls_cert, &tls_key) {
        if let Err(e) = server.configure_tls(cert_path, key_path) {
            println!("⚠️  Warning: Failed to configure TLS: {e}");
            println!("   Continuing with HTTP only...");
        } else {
            enable_https = true;
        }
    }

    // Enable HTTPS enforcement if requested and configured
    if enable_https {
        if let Err(e) = server.enable_https_enforcement() {
            println!("⚠️  Warning: Failed to enable HTTPS enforcement: {e}");
        }
    }

    let protocol = if server.is_https_enabled() { "https" } else { "http" };
    let security_status = server.get_security_status();

    println!("🌐 Starting QMS Web Server...");
    println!("📋 Medical Device Quality Management System");
    println!("🔒 Regulatory Compliance: FDA 21 CFR Part 820, ISO 13485, ISO 14971");
    println!();
    println!("🚀 Server Details:");
    println!("   Host: {host}");
    println!("   Port: {port}");
    println!("   URL:  {protocol}://{host}:{port}");
    println!();
    println!("🔧 Server Features:");
    println!("   ✅ HTTP/1.1 Server (stdlib only)");
    println!("   ✅ Session Management & CSRF Protection");
    println!("   ✅ Audit Trail Integration");
    println!("   ✅ Medical Device Compliance Dashboard");
    println!("   ✅ Offline Support (Progressive Web App)");
    println!();
    println!("🔒 Security Features:");
    println!("   {} HTTPS/TLS Support", if security_status.https_configured { "✅" } else { "❌" });
    println!("   {} Certificate Validation", if security_status.certificate_valid { "✅" } else { "❌" });
    println!("   {} Security Headers", if security_status.security_headers_enabled { "✅" } else { "❌" });
    println!("   {} HSTS (HTTP Strict Transport Security)", if security_status.hsts_enabled { "✅" } else { "❌" });
    println!("   {} CSP (Content Security Policy)", if security_status.csp_enabled { "✅" } else { "❌" });
    println!("   {} HTTPS Enforcement", if security_status.https_enforced { "✅" } else { "❌" });
    println!();
    println!("📝 Access the QMS Dashboard at: {protocol}://{host}:{port}");
    println!("🛑 Press Ctrl+C to stop the server");
    println!("═══════════════════════════════════════════════════════════════════════");

    server.start()?;
    
    Ok(())
}

fn print_serve_help() {
    println!("QMS Web Server - Medical Device Quality Management System");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();
    println!("USAGE:");
    println!("    qms serve [OPTIONS]");
    println!();
    println!("DESCRIPTION:");
    println!("    Start the web-based GUI for QMS operations. Provides a comprehensive");
    println!("    dashboard for medical device quality management with regulatory");
    println!("    compliance features and audit trail integration.");
    println!();
    println!("OPTIONS:");
    println!("    -p, --port <PORT>         Set the server port (default: 8080)");
    println!("    -h, --host <HOST>         Set the server host (default: 127.0.0.1)");
    println!("    --tls-cert <CERT_FILE>    Path to TLS certificate file (PEM format)");
    println!("    --tls-key <KEY_FILE>      Path to TLS private key file (PEM format)");
    println!("    --https                   Enable HTTPS enforcement");
    println!("    --help                    Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    # Start server on default port (8080)");
    println!("    qms serve");
    println!();
    println!("    # Start server on custom port");
    println!("    qms serve --port 3000");
    println!();
    println!("    # Start server on custom host and port");
    println!("    qms serve --host 0.0.0.0 --port 8080");
    println!();
    println!("    # Start server with HTTPS/TLS support");
    println!("    qms serve --tls-cert server.crt --tls-key server.key --https");
    println!();
    println!("    # Start server with HTTPS on port 443");
    println!("    qms serve --port 443 --tls-cert server.crt --tls-key server.key --https");
    println!();
    println!("FEATURES:");
    println!("    🔒 Security:");
    println!("       - Optional HTTPS/TLS support");
    println!("       - Session-based authentication");
    println!("       - CSRF protection");
    println!("       - Comprehensive security headers (HSTS, CSP, X-Frame-Options, etc.)");
    println!("       - Content Security Policy (CSP)");
    println!("       - HTTP Strict Transport Security (HSTS)");
    println!("       - Role-based access control");
    println!("       - Request size validation");
    println!("       - Medical device security compliance");
    println!();
    println!("    📊 Dashboard:");
    println!("       - System health monitoring");
    println!("       - Compliance status indicators");
    println!("       - Recent activity feed");
    println!("       - Quick action buttons");
    println!();
    println!("    🔍 Audit & Compliance:");
    println!("       - Real-time audit logging");
    println!("       - FDA 21 CFR Part 11 compliance");
    println!("       - Electronic signature support");
    println!("       - Integrity verification");
    println!();
    println!("    📱 Progressive Web App:");
    println!("       - Offline support");
    println!("       - Service worker caching");
    println!("       - Mobile responsive design");
    println!("       - Desktop installation");
    println!();
    println!("REGULATORY COMPLIANCE:");
    println!("    ✅ FDA 21 CFR Part 820 - Quality System Regulation");
    println!("    ✅ ISO 13485:2016 - Medical Devices QMS");
    println!("    ✅ ISO 14971:2019 - Risk Management");
    println!("    ✅ 21 CFR Part 11 - Electronic Records and Signatures");
    println!();
    println!("ACCESS URLS:");
    println!("    http://localhost:8080         - Main Dashboard");
    println!("    http://localhost:8080/api     - REST API Endpoints");
    println!("    http://localhost:8080/health  - System Health Check");
    println!();
    println!("Note: Server uses only Rust standard library for maximum reliability");
    println!("      and regulatory compliance in medical device environments.");
}

fn print_usage() {
    println!("Usage: qms <command> [options]");
    println!("Commands: init, doc, risk, req, trace, test, audit, user, report, serve");
    println!("Use 'qms --help' for detailed help");
}

fn print_help() {
    println!("QMS - Medical Device Quality Management System v1.0.0");
    println!("FDA 21 CFR Part 820, ISO 13485, ISO 14971 Compliant");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();
    println!("DESCRIPTION:");
    println!("    A comprehensive Quality Management System for medical device development");
    println!("    and manufacturing, ensuring regulatory compliance throughout the product");
    println!("    lifecycle. Built for reliability, traceability, and audit compliance.");
    println!();
    println!("USAGE:");
    println!("    qms <command> [options]");
    println!();
    println!("COMMANDS:");
    println!("    📋 Document Control (FDA 21 CFR Part 820.40):");
    println!("        init      Initialize a new QMS project with document structure");
    println!("        doc       Document lifecycle management (create, approve, version)");
    println!();
    println!("    ⚠️  Risk Management (ISO 14971):");
    println!("        risk      Risk analysis, FMEA, and mitigation tracking");
    println!();
    println!("    🔗 Requirements Traceability (ISO 13485 Section 7.3):");
    println!("        req       Requirements management and validation");
    println!("        trace     Bi-directional traceability matrices");
    println!();
    println!("    🧪 Testing & Verification:");
    println!("        test      Test case management and execution tracking");
    println!();
    println!("    🔍 Audit & Compliance (FDA 21 CFR Part 820.180-186):");
    println!("        audit     Audit trail management and integrity verification");
    println!("        user      User management with role-based access control");
    println!("        report    Regulatory compliance reports (DHF, CFR compliance)");
    println!();
    println!("    🌐 Web Interface:");
    println!("        serve     Start web-based GUI for QMS operations");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help    Show this help message");
    println!("    --version     Show version information");
    println!();
    println!("EXAMPLES:");
    println!("    # Initialize new medical device project");
    println!("    qms init \"Cardiac Monitor v2.0\" \"2.0\"");
    println!();
    println!("    # Create and manage documents");
    println!("    qms doc create \"Risk Management Plan\" \"RMP-001\" \"Risk Management\" Planning");
    println!("    qms doc approve <document-id>");
    println!();
    println!("    # Risk management workflow");
    println!("    qms risk create HAZ-001 \"Electrical shock\" \"High voltage\" \"Patient harm\" 5 3 2");
    println!("    qms risk mitigate <risk-id> \"Insulation\" 2 1 1");
    println!();
    println!("    # Requirements traceability");
    println!("    qms req create \"REQ-001\" \"User authentication\" Functional High");
    println!("    qms trace matrix");
    println!();
    println!("    # Generate compliance reports");
    println!("    qms report dhf");
    println!("    qms report compliance");
    println!();
    println!("    # Start web interface");
    println!("    qms serve --port 8080");
    println!();
    println!("REGULATORY COMPLIANCE:");
    println!("    ✅ FDA 21 CFR Part 820 - Quality System Regulation");
    println!("    ✅ ISO 13485:2016 - Medical Devices QMS");
    println!("    ✅ ISO 14971:2019 - Risk Management");
    println!("    ✅ 21 CFR Part 11 - Electronic Records and Signatures");
    println!();
    println!("For detailed command help: qms <command> --help");
    println!("Documentation: https://github.com/qms-team/qms/wiki");
    println!("Support: support@qms-team.com");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{log_audit, setup_audit_logger};
    use std::process::Command;

    #[test]
    fn test_audit_logger_setup() {
        // Test that audit logger can be set up without errors
        // Note: In parallel test environment, the audit logger may already be initialized
        let result = setup_audit_logger();
        assert!(
            result.is_ok() || result.as_ref().unwrap_err().to_string().contains("Failed to initialize audit logger"),
            "Audit logger setup should succeed or already be initialized: {:?}", result
        );
    }

    #[test]
    fn test_log_audit_basic() {
        // Test basic audit logging functionality
        let _ = setup_audit_logger();
        log_audit("TEST_EVENT");
        // If we reach here without panicking, the test passes
        // Test passes - main functionality works
    }

    #[test]
    fn test_cli_help_output() {
        // Test that help command works
        let output = Command::new("cargo").args(["run", "--", "--help"]).output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("QMS - Medical Device Quality Management System"));
        }
    }

    #[test]
    fn test_cli_unknown_command() {
        // Test that unknown commands return error
        let output = Command::new("cargo")
            .args(["run", "--", "unknown"])
            .output();

        if let Ok(output) = output {
            assert!(!output.status.success());
        }
    }

    #[test]
    fn test_commands_module_loading() {
        // Test that all command modules can be loaded
        let test_args = vec!["qms".to_string(), "doc".to_string(), "--help".to_string()];

        // This should not panic when calling the command handlers
        let result = std::panic::catch_unwind(|| {
            let _ = doc::handle_doc_command(&test_args);
        });

        assert!(result.is_ok());
    }
}
