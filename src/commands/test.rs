use crate::modules::traceability::test_case::{TestCaseManager, TestCategory, TestPriority, TestStepStatus, TestExecutionStatus};
use crate::error::QmsResult;
use std::path::Path;

pub fn handle_test_command(args: Vec<String>) -> QmsResult<()> {
    if args.is_empty() {
        return handle_test_help();
    }
    
    let project_path = Path::new(".");
    let mut manager = TestCaseManager::new(project_path)?;
    
    match args[0].as_str() {
        "create" => handle_test_create(args, &mut manager),
        "add-step" => handle_test_add_step(args, &mut manager),
        "execute" => handle_test_execute(args, &mut manager),
        "record-result" => handle_test_record_result(args, &mut manager),
        "finalize" => handle_test_finalize(args, &mut manager),
        "list" => handle_test_list(args, &manager),
        "show" => handle_test_show(args, &manager),
        "summary" => handle_test_summary(args, &manager),
        "help" | "--help" => handle_test_help(),
        _ => {
            eprintln!("Unknown test command: {}", args[0]);
            handle_test_help()
        }
    }
}

fn handle_test_create(args: Vec<String>, manager: &mut TestCaseManager) -> QmsResult<()> {
    if args.len() < 7 {
        eprintln!("Usage: qms test create --id <test_id> --title <title> --desc <description> [--category <category>] [--priority <priority>]");
        return Ok(());
    }
    
    let mut test_id = String::new();
    let mut title = String::new();
    let mut description = String::new();
    let mut category = TestCategory::Functional;
    let mut priority = TestPriority::Medium;
    let created_by = "current_user".to_string(); // TODO: Get from session
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--id" => {
                if i + 1 < args.len() {
                    test_id = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --id requires a value");
                    return Ok(());
                }
            }
            "--title" => {
                if i + 1 < args.len() {
                    title = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --title requires a value");
                    return Ok(());
                }
            }
            "--desc" => {
                if i + 1 < args.len() {
                    description = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --desc requires a value");
                    return Ok(());
                }
            }
            "--category" => {
                if i + 1 < args.len() {
                    category = match args[i + 1].to_lowercase().as_str() {
                        "functional" => TestCategory::Functional,
                        "performance" => TestCategory::Performance,
                        "security" => TestCategory::Security,
                        "usability" => TestCategory::Usability,
                        "integration" => TestCategory::Integration,
                        "regression" => TestCategory::Regression,
                        "smoke" => TestCategory::Smoke,
                        "useracceptance" => TestCategory::UserAcceptance,
                        _ => {
                            eprintln!("Invalid category. Valid options: functional, performance, security, usability, integration, regression, smoke, useracceptance");
                            return Ok(());
                        }
                    };
                    i += 2;
                } else {
                    eprintln!("Error: --category requires a value");
                    return Ok(());
                }
            }
            "--priority" => {
                if i + 1 < args.len() {
                    priority = match args[i + 1].to_lowercase().as_str() {
                        "critical" => TestPriority::Critical,
                        "high" => TestPriority::High,
                        "medium" => TestPriority::Medium,
                        "low" => TestPriority::Low,
                        _ => {
                            eprintln!("Invalid priority. Valid options: critical, high, medium, low");
                            return Ok(());
                        }
                    };
                    i += 2;
                } else {
                    eprintln!("Error: --priority requires a value");
                    return Ok(());
                }
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                return Ok(());
            }
        }
    }
    
    if test_id.is_empty() || title.is_empty() || description.is_empty() {
        eprintln!("Error: --id, --title, and --desc are required");
        return Ok(());
    }
    
    match manager.create_test_case(test_id.clone(), title, description, category, priority, created_by) {
        Ok(_) => {
            println!("✅ Test case '{test_id}' created successfully");
        }
        Err(e) => {
            eprintln!("❌ Error creating test case: {e}");
        }
    }
    
    Ok(())
}

fn handle_test_add_step(args: Vec<String>, manager: &mut TestCaseManager) -> QmsResult<()> {
    if args.len() < 6 {
        eprintln!("Usage: qms test add-step <test_id> --action <action> --expected <expected_result> [--notes <notes>]");
        return Ok(());
    }
    
    let test_id = &args[1];
    let mut action = String::new();
    let mut expected_result = String::new();
    let mut notes = None;
    
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--action" => {
                if i + 1 < args.len() {
                    action = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --action requires a value");
                    return Ok(());
                }
            }
            "--expected" => {
                if i + 1 < args.len() {
                    expected_result = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --expected requires a value");
                    return Ok(());
                }
            }
            "--notes" => {
                if i + 1 < args.len() {
                    notes = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --notes requires a value");
                    return Ok(());
                }
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                return Ok(());
            }
        }
    }
    
    if action.is_empty() || expected_result.is_empty() {
        eprintln!("Error: --action and --expected are required");
        return Ok(());
    }
    
    match manager.add_test_step(test_id, action, expected_result, notes) {
        Ok(_) => {
            println!("✅ Test step added to test case '{test_id}'");
        }
        Err(e) => {
            eprintln!("❌ Error adding test step: {e}");
        }
    }
    
    Ok(())
}

fn handle_test_execute(args: Vec<String>, manager: &mut TestCaseManager) -> QmsResult<()> {
    if args.len() < 2 {
        eprintln!("Usage: qms test execute <test_id> [--environment <environment>]");
        return Ok(());
    }
    
    let test_id = &args[1];
    let executed_by = "current_user".to_string(); // TODO: Get from session
    let mut environment = None;
    
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--environment" => {
                if i + 1 < args.len() {
                    environment = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --environment requires a value");
                    return Ok(());
                }
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                return Ok(());
            }
        }
    }
    
    match manager.execute_test(test_id, executed_by, environment) {
        Ok(execution_id) => {
            println!("✅ Test execution started for test case '{test_id}'");
            println!("📋 Execution ID: {execution_id}");
        }
        Err(e) => {
            eprintln!("❌ Error starting test execution: {e}");
        }
    }
    
    Ok(())
}

fn handle_test_record_result(args: Vec<String>, manager: &mut TestCaseManager) -> QmsResult<()> {
    if args.len() < 8 {
        eprintln!("Usage: qms test record-result <test_id> --execution <execution_id> --step <step_number> --status <status> --actual <actual_result> [--notes <notes>]");
        return Ok(());
    }
    
    let test_id = &args[1];
    let mut execution_id = String::new();
    let mut step_number = 0u32;
    let mut status = TestStepStatus::NotExecuted;
    let mut actual_result = String::new();
    let mut notes = None;
    
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--execution" => {
                if i + 1 < args.len() {
                    execution_id = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --execution requires a value");
                    return Ok(());
                }
            }
            "--step" => {
                if i + 1 < args.len() {
                    step_number = args[i + 1].parse().unwrap_or(0);
                    i += 2;
                } else {
                    eprintln!("Error: --step requires a value");
                    return Ok(());
                }
            }
            "--status" => {
                if i + 1 < args.len() {
                    status = match args[i + 1].to_lowercase().as_str() {
                        "passed" => TestStepStatus::Passed,
                        "failed" => TestStepStatus::Failed,
                        "blocked" => TestStepStatus::Blocked,
                        "skipped" => TestStepStatus::Skipped,
                        _ => {
                            eprintln!("Invalid status. Valid options: passed, failed, blocked, skipped");
                            return Ok(());
                        }
                    };
                    i += 2;
                } else {
                    eprintln!("Error: --status requires a value");
                    return Ok(());
                }
            }
            "--actual" => {
                if i + 1 < args.len() {
                    actual_result = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --actual requires a value");
                    return Ok(());
                }
            }
            "--notes" => {
                if i + 1 < args.len() {
                    notes = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --notes requires a value");
                    return Ok(());
                }
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                return Ok(());
            }
        }
    }
    
    if execution_id.is_empty() || step_number == 0 || actual_result.is_empty() {
        eprintln!("Error: --execution, --step, --status, and --actual are required");
        return Ok(());
    }
    
    match manager.record_step_result(test_id, &execution_id, step_number, status, actual_result, notes) {
        Ok(_) => {
            println!("✅ Test step result recorded for test case '{test_id}'");
        }
        Err(e) => {
            eprintln!("❌ Error recording test step result: {e}");
        }
    }
    
    Ok(())
}

fn handle_test_finalize(args: Vec<String>, manager: &mut TestCaseManager) -> QmsResult<()> {
    if args.len() < 5 {
        eprintln!("Usage: qms test finalize <test_id> --execution <execution_id> --status <status> [--duration <seconds>] [--notes <notes>]");
        return Ok(());
    }
    
    let test_id = &args[1];
    let mut execution_id = String::new();
    let mut status = TestExecutionStatus::Incomplete;
    let mut duration_seconds = None;
    let mut notes = None;
    
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--execution" => {
                if i + 1 < args.len() {
                    execution_id = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --execution requires a value");
                    return Ok(());
                }
            }
            "--status" => {
                if i + 1 < args.len() {
                    status = match args[i + 1].to_lowercase().as_str() {
                        "passed" => TestExecutionStatus::Passed,
                        "failed" => TestExecutionStatus::Failed,
                        "blocked" => TestExecutionStatus::Blocked,
                        "incomplete" => TestExecutionStatus::Incomplete,
                        _ => {
                            eprintln!("Invalid status. Valid options: passed, failed, blocked, incomplete");
                            return Ok(());
                        }
                    };
                    i += 2;
                } else {
                    eprintln!("Error: --status requires a value");
                    return Ok(());
                }
            }
            "--duration" => {
                if i + 1 < args.len() {
                    duration_seconds = Some(args[i + 1].parse().unwrap_or(0));
                    i += 2;
                } else {
                    eprintln!("Error: --duration requires a value");
                    return Ok(());
                }
            }
            "--notes" => {
                if i + 1 < args.len() {
                    notes = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --notes requires a value");
                    return Ok(());
                }
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                return Ok(());
            }
        }
    }
    
    if execution_id.is_empty() {
        eprintln!("Error: --execution and --status are required");
        return Ok(());
    }
    
    match manager.finalize_execution(test_id, &execution_id, status, duration_seconds, notes) {
        Ok(_) => {
            println!("✅ Test execution finalized for test case '{test_id}'");
        }
        Err(e) => {
            eprintln!("❌ Error finalizing test execution: {e}");
        }
    }
    
    Ok(())
}

fn handle_test_list(args: Vec<String>, manager: &TestCaseManager) -> QmsResult<()> {
    let mut category_filter = None;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--category" => {
                if i + 1 < args.len() {
                    category_filter = Some(match args[i + 1].to_lowercase().as_str() {
                        "functional" => TestCategory::Functional,
                        "performance" => TestCategory::Performance,
                        "security" => TestCategory::Security,
                        "usability" => TestCategory::Usability,
                        "integration" => TestCategory::Integration,
                        "regression" => TestCategory::Regression,
                        "smoke" => TestCategory::Smoke,
                        "useracceptance" => TestCategory::UserAcceptance,
                        _ => {
                            eprintln!("Invalid category. Valid options: functional, performance, security, usability, integration, regression, smoke, useracceptance");
                            return Ok(());
                        }
                    });
                    i += 2;
                } else {
                    eprintln!("Error: --category requires a value");
                    return Ok(());
                }
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                return Ok(());
            }
        }
    }
    
    let test_cases = if let Some(category) = category_filter {
        manager.list_test_cases_by_category(&category)
    } else {
        manager.list_test_cases()
    };
    
    if test_cases.is_empty() {
        println!("📋 No test cases found");
        return Ok(());
    }
    
    println!("📋 Test Cases:");
    println!("{:-<80}", "");
    for test_case in test_cases {
        println!("🧪 {} - {}", test_case.test_id, test_case.title);
        println!("   Category: {}, Priority: {}", test_case.category, test_case.priority);
        println!("   Steps: {}, Executions: {}", test_case.steps.len(), test_case.execution_results.len());
        println!("   Created: {:?} by {}", test_case.created_date, test_case.created_by);
        println!();
    }
    
    Ok(())
}

fn handle_test_show(args: Vec<String>, manager: &TestCaseManager) -> QmsResult<()> {
    if args.len() < 2 {
        eprintln!("Usage: qms test show <test_id>");
        return Ok(());
    }
    
    let test_id = &args[1];
    
    match manager.get_test_case(test_id) {
        Some(test_case) => {
            println!("🧪 Test Case: {} - {}", test_case.test_id, test_case.title);
            println!("📝 Description: {}", test_case.description);
            println!("🏷️  Category: {}, Priority: {}", test_case.category, test_case.priority);
            println!("📅 Created: {:?} by {}", test_case.created_date, test_case.created_by);
            println!("🔄 Last Modified: {:?}", test_case.last_modified);
            
            if let Some(preconditions) = &test_case.preconditions {
                println!("⚙️  Preconditions: {preconditions}");
            }
            
            if let Some(postconditions) = &test_case.postconditions {
                println!("🏁 Postconditions: {postconditions}");
            }
            
            if !test_case.tags.is_empty() {
                println!("🏷️  Tags: {}", test_case.tags.join(", "));
            }
            
            println!("\n📋 Test Steps:");
            for step in &test_case.steps {
                println!("  {}. {}", step.step_number, step.action);
                println!("     Expected: {}", step.expected_result);
                println!("     Status: {}", step.status);
                if let Some(notes) = &step.notes {
                    println!("     Notes: {notes}");
                }
            }
            
            if !test_case.execution_results.is_empty() {
                println!("\n🔄 Execution Results:");
                for execution in &test_case.execution_results {
                    println!("  📋 {} - {}", execution.execution_id, execution.overall_status);
                    println!("     Executed by: {} on {:?}", execution.executed_by, execution.execution_date);
                    if let Some(duration) = execution.duration_seconds {
                        println!("     Duration: {duration} seconds");
                    }
                    if let Some(env) = &execution.environment {
                        println!("     Environment: {env}");
                    }
                    if let Some(notes) = &execution.notes {
                        println!("     Notes: {notes}");
                    }
                }
            }
        }
        None => {
            eprintln!("❌ Test case '{test_id}' not found");
        }
    }
    
    Ok(())
}

fn handle_test_summary(args: Vec<String>, manager: &TestCaseManager) -> QmsResult<()> {
    if args.len() < 2 {
        eprintln!("Usage: qms test summary <test_id>");
        return Ok(());
    }
    
    let test_id = &args[1];
    
    match manager.get_test_execution_summary(test_id) {
        Ok(summary) => {
            println!("{summary}");
        }
        Err(e) => {
            eprintln!("❌ Error generating test summary: {e}");
        }
    }
    
    Ok(())
}

fn handle_test_help() -> QmsResult<()> {
    println!("🧪 QMS Test Case Management Commands");
    println!("{:-<50}", "");
    println!();
    println!("📋 Basic Commands:");
    println!("  qms test create --id <id> --title <title> --desc <description>");
    println!("    Create a new test case");
    println!();
    println!("  qms test add-step <test_id> --action <action> --expected <expected>");
    println!("    Add a test step to a test case");
    println!();
    println!("  qms test execute <test_id> [--environment <env>]");
    println!("    Start test execution");
    println!();
    println!("  qms test record-result <test_id> --execution <exec_id> --step <num> --status <status> --actual <result>");
    println!("    Record test step result");
    println!();
    println!("  qms test finalize <test_id> --execution <exec_id> --status <status> [--duration <seconds>]");
    println!("    Finalize test execution");
    println!();
    println!("📊 Query Commands:");
    println!("  qms test list [--category <category>]");
    println!("    List all test cases or filter by category");
    println!();
    println!("  qms test show <test_id>");
    println!("    Show detailed test case information");
    println!();
    println!("  qms test summary <test_id>");
    println!("    Show test execution summary");
    println!();
    println!("📝 Options:");
    println!("  Categories: functional, performance, security, usability, integration, regression, smoke, useracceptance");
    println!("  Priorities: critical, high, medium, low");
    println!("  Step Status: passed, failed, blocked, skipped");
    println!("  Execution Status: passed, failed, blocked, incomplete");
    println!();
    println!("💡 Examples:");
    println!("  qms test create --id TC-001 --title \"Login Test\" --desc \"Test user login functionality\"");
    println!("  qms test add-step TC-001 --action \"Enter username\" --expected \"Username field accepts input\"");
    println!("  qms test execute TC-001 --environment \"QA Environment\"");
    println!("  qms test record-result TC-001 --execution EXEC-TC-001-123 --step 1 --status passed --actual \"Username accepted\"");
    println!();
    Ok(())
}
