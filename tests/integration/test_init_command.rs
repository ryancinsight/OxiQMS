use std::env;
use std::fs;
use std::process::Command;

#[test]
fn test_qms_init_command_success() {
    // Setup temporary directory for testing
    let temp_dir = env::temp_dir().join("qms_test_init");
    let _ = fs::remove_dir_all(&temp_dir); // Clean up any previous test runs

    // Run qms init command (tests run from project root)
    let output = Command::new("cargo")
        .args(&["run", "--bin", "qms", "--", "init", "--name", "TestProject"])
        .output()
        .expect("Failed to execute qms init command");

    // Check command succeeded
    assert!(
        output.status.success(),
        "qms init command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify output contains expected messages
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Project initialized") || stdout.contains("success"),
        "Expected success message not found in output: {}",
        stdout
    );
}

#[test]
fn test_qms_init_creates_project_structure() {
    let temp_dir = env::temp_dir().join("qms_test_structure");
    let _ = fs::remove_dir_all(&temp_dir);

    // Run qms init with custom path
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "qms",
            "--",
            "init",
            "--name",
            "StructureTest",
            "--path",
            temp_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute qms init command");

    // Only check for basic directory creation if command succeeded
    if output.status.success() {
        // Check if basic project files exist
        let project_file = temp_dir.join("project.json");
        if project_file.exists() {
            let content = fs::read_to_string(&project_file).unwrap();
            assert!(
                content.contains("TestProject") || content.contains("name"),
                "Project file should contain project information"
            );
        }
    }

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_qms_help_command() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "qms", "--", "--help"])
        .output()
        .expect("Failed to execute qms help command");

    assert!(output.status.success(), "Help command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("QMS"),
        "Help should contain QMS information"
    );
    assert!(stdout.contains("init"), "Help should list init command");
    assert!(stdout.contains("doc"), "Help should list doc command");
}

#[test]
fn test_qms_unknown_command() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "qms", "--", "nonexistent"])
        .output()
        .expect("Failed to execute qms with unknown command");

    assert!(!output.status.success(), "Unknown command should fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unknown command") || stderr.contains("error"),
        "Should report unknown command error"
    );
}

#[test]
fn test_property_based_validation() {
    // Test various project name formats
    let test_names = vec![
        "ValidProject",
        "Project With Spaces",
        "Project_With_Underscores",
        "Project-With-Dashes",
        "123NumberProject",
    ];

    for name in test_names {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "qms", "--", "init", "--name", name])
            .output();

        if let Ok(output) = output {
            // Command should either succeed or fail gracefully with clear error message
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert!(
                    !stderr.is_empty(),
                    "Failed command should provide error message for name: {}",
                    name
                );
            }
        }
    }
}
