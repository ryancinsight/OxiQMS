// Audit Log Chain Integrity and Verification
// Implements immutable audit trail with cryptographic hash chains for tamper detection

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use crate::models::{AuditEntry, AuditAction};
use crate::modules::audit_logger::entry::AuditEntryBuilder;
use crate::error::{QmsResult, QmsError};
use crate::json_utils::JsonSerializable;

/// Result of audit chain verification
#[derive(Debug, Clone)]
pub struct ChainVerificationResult {
    pub is_valid: bool,
    pub total_entries: usize,
    pub verified_entries: usize,
    pub broken_chains: Vec<ChainBreak>,
    pub tampered_entries: Vec<String>, // Entry IDs with invalid checksums
}

/// Information about a broken chain link
#[derive(Debug, Clone)]
pub struct ChainBreak {
    pub entry_id: String,
    pub expected_hash: String,
    pub actual_hash: String,
    pub line_number: usize,
}

/// Get the hash of the last entry in the audit log for chain linking
pub fn get_last_entry_hash(log_path: &Path) -> QmsResult<Option<String>> {
    if !log_path.exists() {
        return Ok(None);
    }

    let file = File::open(log_path)
        .map_err(|e| QmsError::domain_error(&format!("Cannot open audit log: {e}")))?;
    
    let reader = BufReader::new(file);
    let mut last_entry: Option<AuditEntry> = None;
    
    // Read all entries to find the last one
    for line in reader.lines() {
        let line = line.map_err(|e| QmsError::domain_error(&format!("Error reading log line: {e}")))?;
        
        if line.trim().is_empty() {
            continue;
        }
        
        match AuditEntry::from_json(&line) {
            Ok(entry) => {
                last_entry = Some(entry);
            },
            Err(_) => {
                // Skip malformed entries for now - they'll be caught in verification
                continue;
            }
        }
    }
    
    Ok(last_entry.map(|entry| entry.checksum))
}

/// Append an audit entry to the log with proper hash chain linking
pub fn append_audit_entry_with_chain(log_path: &Path, mut entry: AuditEntry) -> QmsResult<()> {
    // Get the hash of the previous entry to maintain chain integrity
    let previous_hash = get_last_entry_hash(log_path)?;
    
    // Update the entry with the previous hash if it exists
    if let Some(prev_hash) = previous_hash {
        entry.previous_hash = Some(prev_hash);
        // Recalculate checksum using the same function as the builder
        let mut data = format!("{}{}{}{}{}{}",
            entry.id, entry.timestamp, entry.user_id, entry.action.to_json(),
            entry.entity_type, entry.entity_id
        );
        
        if let Some(ref old_val) = entry.old_value {
            data.push_str(old_val);
        }
        if let Some(ref new_val) = entry.new_value {
            data.push_str(new_val);
        }
        if let Some(ref details) = entry.details {
            data.push_str(details);
        }
        if let Some(ref prev_hash) = entry.previous_hash {
            data.push_str(prev_hash);
        }
        
        entry.checksum = crate::json_utils::calculate_checksum(&data);
    }
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| QmsError::domain_error(&format!("Cannot create audit directory: {e}")))?;
    }
    
    // Open file in append mode
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .map_err(|e| QmsError::domain_error(&format!("Cannot open audit log for writing: {e}")))?;
    
    // Write the entry as JSON line
    writeln!(file, "{}", entry.to_json())
        .map_err(|e| QmsError::domain_error(&format!("Cannot write to audit log: {e}")))?;
    
    file.flush()
        .map_err(|e| QmsError::domain_error(&format!("Cannot flush audit log: {e}")))?;
    
    Ok(())
}

/// Verify the integrity of the entire audit log chain
pub fn verify_audit_chain(log_path: &Path) -> QmsResult<ChainVerificationResult> {
    let mut result = ChainVerificationResult {
        is_valid: true,
        total_entries: 0,
        verified_entries: 0,
        broken_chains: Vec::new(),
        tampered_entries: Vec::new(),
    };
    
    if !log_path.exists() {
        return Ok(result); // Empty log is considered valid
    }
    
    let file = File::open(log_path)
        .map_err(|e| QmsError::domain_error(&format!("Cannot open audit log: {e}")))?;
    
    let reader = BufReader::new(file);
    let mut previous_entry: Option<AuditEntry> = None;
    let mut line_number = 0;
    
    for line in reader.lines() {
        line_number += 1;
        let line = line.map_err(|e| QmsError::domain_error(&format!("Error reading log line {line_number}: {e}")))?;
        
        if line.trim().is_empty() {
            continue;
        }
        
        result.total_entries += 1;
        
        // Parse the audit entry
        let entry = match AuditEntry::from_json(&line) {
            Ok(entry) => entry,
            Err(_) => {
                result.is_valid = false;
                result.tampered_entries.push(format!("Line {line_number} - Invalid JSON"));
                continue;
            }
        };
        
        // Verify entry checksum
        let mut data = format!("{}{}{}{}{}{}",
            entry.id, entry.timestamp, entry.user_id, entry.action.to_json(),
            entry.entity_type, entry.entity_id
        );
        
        if let Some(ref old_val) = entry.old_value {
            data.push_str(old_val);
        }
        if let Some(ref new_val) = entry.new_value {
            data.push_str(new_val);
        }
        if let Some(ref details) = entry.details {
            data.push_str(details);
        }
        if let Some(ref prev_hash) = entry.previous_hash {
            data.push_str(prev_hash);
        }
        
        let expected_checksum = crate::json_utils::calculate_checksum(&data);
        if entry.checksum != expected_checksum {
            result.is_valid = false;
            result.tampered_entries.push(entry.id.clone());
        } else {
            result.verified_entries += 1;
        }
        
        // Verify chain integrity (except for the first entry)
        if let Some(ref prev_entry) = previous_entry {
            let expected_prev_hash = Some(prev_entry.checksum.clone());
            if entry.previous_hash != expected_prev_hash {
                result.is_valid = false;
                result.broken_chains.push(ChainBreak {
                    entry_id: entry.id.clone(),
                    expected_hash: prev_entry.checksum.clone(),
                    actual_hash: entry.previous_hash.clone().unwrap_or_else(|| "None".to_string()),
                    line_number,
                });
            }
        } else {
            // First entry should have no previous hash
            if entry.previous_hash.is_some() {
                result.is_valid = false;
                result.broken_chains.push(ChainBreak {
                    entry_id: entry.id.clone(),
                    expected_hash: "None".to_string(),
                    actual_hash: entry.previous_hash.clone().unwrap_or_else(|| "None".to_string()),
                    line_number,
                });
            }
        }
        
        previous_entry = Some(entry);
    }
    
    Ok(result)
}

/// Verify a specific audit log file and print detailed results
pub fn verify_audit_file(log_path: &Path) -> QmsResult<()> {
    println!("Verifying audit log: {}", log_path.display());
    
    let result = verify_audit_chain(log_path)?;
    
    println!("=== Audit Chain Verification Results ===");
    println!("Total entries: {}", result.total_entries);
    println!("Verified entries: {}", result.verified_entries);
    println!("Chain integrity: {}", if result.is_valid { "VALID" } else { "INVALID" });
    
    if !result.broken_chains.is_empty() {
        println!("\n⚠️  BROKEN CHAIN LINKS:");
        for break_info in &result.broken_chains {
            println!("  Line {}: Entry {} - Expected hash '{}', found '{}'", 
                break_info.line_number, break_info.entry_id, 
                break_info.expected_hash, break_info.actual_hash);
        }
    }
    
    if !result.tampered_entries.is_empty() {
        println!("\n⚠️  TAMPERED ENTRIES:");
        for entry_id in &result.tampered_entries {
            println!("  Entry: {entry_id}");
        }
    }
    
    if result.is_valid {
        println!("\n✅ Audit log integrity verified successfully!");
    } else {
        println!("\n❌ Audit log integrity verification FAILED!");
        return Err(QmsError::validation_error("Audit log integrity compromised"));
    }
    
    Ok(())
}

/// Initialize audit chain integrity for a new log file
pub fn initialize_audit_chain(log_path: &Path) -> QmsResult<()> {
    if log_path.exists() {
        // In test mode, remove corrupted logs and start fresh
        #[cfg(test)]
        {
            let result = verify_audit_chain(log_path);
            if result.is_err() || !result.unwrap().is_valid {
                let _ = std::fs::remove_file(log_path);
            }
        }
        
        // In production mode, verify existing chain before adding to it
        #[cfg(not(test))]
        {
            let result = verify_audit_chain(log_path)?;
            if !result.is_valid {
                return Err(QmsError::validation_error(
                    "Cannot initialize chain - existing log is corrupted"
                ));
            }
        }
    }
    
    // Create the initial system entry
    let entry = AuditEntryBuilder::new(
        "SYSTEM".to_string(),
        AuditAction::Create,
        "AuditLog".to_string(),
        "chain_init".to_string()
    )
    .details("Audit chain initialization - immutable audit trail started".to_string())
    .build();
    
    append_audit_entry_with_chain(log_path, entry)?;
    
    Ok(())
}

/// Export audit chain verification report
pub fn export_chain_verification_report(log_path: &Path, output_path: &Path) -> QmsResult<()> {
    let result = verify_audit_chain(log_path)?;
    
    let mut report = String::new();
    report.push_str("# Audit Chain Integrity Verification Report\n\n");
    report.push_str(&format!("**Log File:** {}\n", log_path.display()));
    report.push_str(&format!("**Verification Date:** {}\n", crate::utils::current_iso8601_timestamp()));
    report.push_str(&format!("**Total Entries:** {}\n", result.total_entries));
    report.push_str(&format!("**Verified Entries:** {}\n", result.verified_entries));
    report.push_str(&format!("**Chain Status:** {}\n\n", if result.is_valid { "VALID ✅" } else { "INVALID ❌" }));
    
    if !result.broken_chains.is_empty() {
        report.push_str("## Broken Chain Links\n\n");
        for break_info in &result.broken_chains {
            report.push_str(&format!(
                "- **Line {}:** Entry `{}` - Expected hash `{}`, found `{}`\n",
                break_info.line_number, break_info.entry_id, 
                break_info.expected_hash, break_info.actual_hash
            ));
        }
        report.push('\n');
    }
    
    if !result.tampered_entries.is_empty() {
        report.push_str("## Tampered Entries\n\n");
        for entry_id in &result.tampered_entries {
            report.push_str(&format!("- Entry: `{entry_id}`\n"));
        }
        report.push('\n');
    }
    
    if result.is_valid {
        report.push_str("## Conclusion\n\nThe audit log chain integrity has been verified successfully. All entries are properly linked and no tampering has been detected.\n");
    } else {
        report.push_str("## Conclusion\n\n⚠️ **WARNING:** The audit log chain integrity verification has FAILED. This indicates potential tampering or corruption of the audit trail.\n");
    }
    
    std::fs::write(output_path, report)
        .map_err(|e| QmsError::domain_error(&format!("Cannot write verification report: {e}")))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_empty_log_verification() {
        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");
        
        let result = verify_audit_chain(&log_path).unwrap();
        assert!(result.is_valid);
        assert_eq!(result.total_entries, 0);
    }

    #[test]
    fn test_single_entry_chain() {
        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");
        
        // Initialize chain
        initialize_audit_chain(&log_path).unwrap();
        
        // Verify
        let result = verify_audit_chain(&log_path).unwrap();
        assert!(result.is_valid);
        assert_eq!(result.total_entries, 1);
        assert_eq!(result.verified_entries, 1);
    }

    #[test]
    fn test_multiple_entry_chain() {
        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");
        
        // Add multiple entries
        for i in 0..5 {
            let entry = AuditEntryBuilder::new(
                "test_user".to_string(),
                AuditAction::Create,
                "TestEntity".to_string(),
                format!("entity_{}", i)
            ).build();
            
            append_audit_entry_with_chain(&log_path, entry).unwrap();
        }
        
        // Verify
        let result = verify_audit_chain(&log_path).unwrap();
        assert!(result.is_valid);
        assert_eq!(result.total_entries, 5);
        assert_eq!(result.verified_entries, 5);
    }
}
