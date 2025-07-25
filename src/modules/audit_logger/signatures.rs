//! Audit log digital signatures for integrity verification
//! Uses SHA-256 for cryptographically secure signatures

use crate::error::{QmsError, QmsResult};
use crate::modules::audit_logger::AuditEntry;
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};

/// Electronic signature data structure
#[derive(Debug, Clone)]
pub struct ElectronicSignature {
    pub id: String,                         // Unique signature ID
    pub user_id: String,                    // Signing user
    pub timestamp: String,                  // ISO 8601 timestamp
    pub meaning: String,                    // What the signature means
    pub signature_hash: String,             // Hash of signature data
    pub entity_type: String,                // Type of entity being signed
    pub entity_id: String,                  // ID of entity being signed
    pub signature_method: SignatureMethod,  // How signature was created
    pub certificate_info: Option<String>,   // Digital certificate info
    pub reason: Option<String>,             // Reason for signing
}

/// Methods for creating electronic signatures
#[derive(Debug, Clone)]
#[allow(dead_code)] // Future implementation variants
pub enum SignatureMethod {
    Password,           // Username/password authentication
    Biometric,          // Biometric verification (future)
    DigitalCertificate, // PKI certificate (future)
    TwoFactor,          // Two-factor authentication
}

/// Electronic signature requirements for different actions
pub struct SignatureRequirements {
    pub requirements: HashMap<String, SignaturePolicy>,
}

/// Policy defining when signatures are required
#[derive(Debug, Clone)]
pub struct SignaturePolicy {
    pub required: bool,
    pub meaning: String,
    pub minimum_method: SignatureMethod,
    pub requires_reason: bool,
}

/// Electronic signature manager
pub struct ElectronicSignatureManager {
    project_path: PathBuf,
    requirements: SignatureRequirements,
}

/// Signature verification result
#[derive(Debug)]
pub struct SignatureVerification {
    pub is_valid: bool,
    pub signature: ElectronicSignature,
    pub verification_details: String,
    pub verified_at: String,
}

impl ElectronicSignature {
    /// Create a new electronic signature
    pub fn new(
        user_id: String,
        meaning: String,
        entity_type: String,
        entity_id: String,
        signature_method: SignatureMethod,
        reason: Option<String>,
    ) -> QmsResult<Self> {
        let id = crate::utils::generate_uuid();
        let timestamp = crate::utils::current_timestamp().to_string();
        
        // Create signature hash using SHA-256
        let signature_data = format!("{user_id}|{timestamp}|{meaning}|{entity_type}|{entity_id}");
        let signature_hash = format!("{:x}", Sha256::digest(signature_data.as_bytes()));

        Ok(ElectronicSignature {
            id,
            user_id,
            timestamp,
            meaning,
            signature_hash,
            entity_type,
            entity_id,
            signature_method,
            certificate_info: None,
            reason,
        })
    }

    /// Verify the signature hash
    pub fn verify_hash(&self) -> bool {
        let signature_data = format!("{}|{}|{}|{}|{}", 
                                    self.user_id, self.timestamp, self.meaning, 
                                    self.entity_type, self.entity_id);
        let calculated_hash = format!("{:x}", Sha256::digest(signature_data.as_bytes()));
        calculated_hash == self.signature_hash
    }
}

impl JsonSerializable for ElectronicSignature {
    fn to_json(&self) -> String {
        format!(
            r#"{{"id": "{}", "user_id": "{}", "timestamp": "{}", "meaning": "{}", "signature_hash": "{}", "entity_type": "{}", "entity_id": "{}", "signature_method": "{:?}", "certificate_info": {}, "reason": {}}}"#,
            self.id,
            self.user_id,
            self.timestamp,
            self.meaning,
            self.signature_hash,
            self.entity_type,
            self.entity_id,
            self.signature_method,
            self.certificate_info.as_ref().map(|s| format!("\"{s}\"")).unwrap_or_else(|| "null".to_string()),
            self.reason.as_ref().map(|s| format!("\"{s}\"")).unwrap_or_else(|| "null".to_string())
        )
    }

    fn from_json(json: &str) -> Result<Self, JsonError> {
        let parsed = JsonValue::parse(json)?;
        
        if let JsonValue::Object(obj) = parsed {
            let extract_string_field = |obj: &HashMap<String, JsonValue>, field: &str| -> Result<String, JsonError> {
                match obj.get(field) {
                    Some(JsonValue::String(s)) => Ok(s.clone()),
                    Some(_) => Err(JsonError::InvalidFormat(format!("Field '{field}' is not a string"))),
                    None => Err(JsonError::InvalidFormat(format!("Missing required field '{field}'"))),
                }
            };
            
            let user_id = extract_string_field(&obj, "user_id")?;
            let timestamp = extract_string_field(&obj, "timestamp")?;
            let meaning = extract_string_field(&obj, "meaning")?;
            let signature_hash = extract_string_field(&obj, "signature_hash")?;
            let entity_type = extract_string_field(&obj, "entity_type")?;
            let entity_id = extract_string_field(&obj, "entity_id")?;

            Ok(ElectronicSignature {
                id: extract_string_field(&obj, "id")?,
                user_id,
                timestamp,
                meaning,
                signature_hash,
                entity_type,
                entity_id,
                signature_method: SignatureMethod::Password, // Default
                certificate_info: None,
                reason: None,
            })
        } else {
            Err(JsonError::InvalidFormat("Expected JSON object".to_string()))
        }
    }
}

impl Default for SignatureRequirements {
    fn default() -> Self {
        let mut requirements = HashMap::new();
        
        // Document approval requires signature
        requirements.insert("document_approve".to_string(), SignaturePolicy {
            required: true,
            meaning: "Document approved".to_string(),
            minimum_method: SignatureMethod::Password,
            requires_reason: false,
        });
        
        // Document deletion requires signature
        requirements.insert("document_delete".to_string(), SignaturePolicy {
            required: true,
            meaning: "Document deletion authorized".to_string(),
            minimum_method: SignatureMethod::Password,
            requires_reason: true,
        });
        
        // Risk acceptance requires signature
        requirements.insert("risk_accept".to_string(), SignaturePolicy {
            required: true,
            meaning: "Risk acceptance authorized".to_string(),
            minimum_method: SignatureMethod::Password,
            requires_reason: true,
        });
        
        // System configuration changes require signature
        requirements.insert("system_config".to_string(), SignaturePolicy {
            required: true,
            meaning: "System configuration change authorized".to_string(),
            minimum_method: SignatureMethod::TwoFactor,
            requires_reason: true,
        });
        
        Self { requirements }
    }
}

impl ElectronicSignatureManager {
    /// Create new signature manager
    pub fn new(project_path: PathBuf) -> Self {
        Self {
            project_path,
            requirements: SignatureRequirements::default(),
        }
    }

    /// Check if an action requires electronic signature
    #[allow(dead_code)] // Future implementation
    pub fn requires_signature(&self, action: &str) -> bool {
        self.requirements.requirements.get(action)
            .map(|policy| policy.required)
            .unwrap_or(false)
    }

    /// Get signature policy for an action
    pub fn get_signature_policy(&self, action: &str) -> Option<&SignaturePolicy> {
        self.requirements.requirements.get(action)
    }

    /// Create electronic signature for an action
    pub fn create_signature(
        &self,
        user_id: String,
        action: &str,
        entity_type: String,
        entity_id: String,
        reason: Option<String>,
    ) -> QmsResult<ElectronicSignature> {
        // Get policy for this action
        let policy = self.get_signature_policy(action)
            .ok_or_else(|| QmsError::validation_error(&format!("No signature policy for action: {action}")))?;

        // Check if reason is required
        if policy.requires_reason && reason.is_none() {
            return Err(QmsError::validation_error("Reason is required for this signature"));
        }

        // Create the signature
        let signature = ElectronicSignature::new(
            user_id,
            policy.meaning.clone(),
            entity_type,
            entity_id,
            policy.minimum_method.clone(),
            reason,
        )?;

        // Store the signature
        self.store_signature(&signature)?;

        // Log signature creation in audit trail
        self.log_signature_creation(&signature)?;

        Ok(signature)
    }

    /// Verify an electronic signature
    pub fn verify_signature(&self, signature_id: &str) -> QmsResult<SignatureVerification> {
        let signature = self.load_signature(signature_id)?;
        let is_valid = signature.verify_hash();
        let verification_details = if is_valid {
            "Signature hash verification passed".to_string()
        } else {
            "Signature hash verification failed".to_string()
        };

        Ok(SignatureVerification {
            is_valid,
            signature,
            verification_details,
            verified_at: crate::utils::current_timestamp().to_string(),
        })
    }

    /// Store signature to file system
    fn store_signature(&self, signature: &ElectronicSignature) -> QmsResult<()> {
        let signatures_dir = self.project_path.join("signatures");
        std::fs::create_dir_all(&signatures_dir)?;

        let signature_file = signatures_dir.join(format!("{}.json", signature.id));
        let signature_json = signature.to_json();
        
        crate::fs_utils::atomic_write(&signature_file, &signature_json)?;
        
        Ok(())
    }

    /// Load signature from file system
    fn load_signature(&self, signature_id: &str) -> QmsResult<ElectronicSignature> {
        let signature_file = self.project_path.join("signatures").join(format!("{signature_id}.json"));
        
        if !signature_file.exists() {
            return Err(QmsError::not_found(&format!("Signature not found: {signature_id}")));
        }

        let signature_json = std::fs::read_to_string(&signature_file)?;
        ElectronicSignature::from_json(&signature_json)
            .map_err(|e| QmsError::validation_error(&e.to_string()))
    }

    /// Log signature creation in audit trail
    fn log_signature_creation(&self, signature: &ElectronicSignature) -> QmsResult<()> {
        use crate::modules::audit_logger::log_system_event;
        
        let _details = format!("Electronic signature created: {} for {} {}", 
                            signature.meaning, signature.entity_type, signature.entity_id);
        
        log_system_event("signature_create", &format!("Electronic signature created for {} by {}", signature.entity_id, signature.user_id))
    }

    /// List all signatures for an entity
    pub fn list_signatures_for_entity(&self, entity_type: &str, entity_id: &str) -> QmsResult<Vec<ElectronicSignature>> {
        let signatures_dir = self.project_path.join("signatures");
        let mut signatures = Vec::new();

        if !signatures_dir.exists() {
            return Ok(signatures);
        }

        for entry in std::fs::read_dir(&signatures_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "json") {
                if let Ok(signature_json) = std::fs::read_to_string(&path) {
                    if let Ok(signature) = ElectronicSignature::from_json(&signature_json) {
                        if signature.entity_type == entity_type && signature.entity_id == entity_id {
                            signatures.push(signature);
                        }
                    }
                }
            }
        }

        // Sort by timestamp (most recent first)
        signatures.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(signatures)
    }

    /// Validate signature requirements are met for an action
    #[allow(dead_code)] // Future implementation  
    pub fn validate_signature_requirements(
        &self,
        action: &str,
        entity_type: &str,
        entity_id: &str,
    ) -> QmsResult<bool> {
        if !self.requires_signature(action) {
            return Ok(true); // No signature required
        }

        let signatures = self.list_signatures_for_entity(entity_type, entity_id)?;
        let policy = self.get_signature_policy(action)
            .ok_or_else(|| QmsError::validation_error("No signature policy found"))?;

        // Check if there's a valid signature with the required meaning
        let has_valid_signature = signatures.iter().any(|sig| {
            sig.meaning == policy.meaning && sig.verify_hash()
        });

        Ok(has_valid_signature)
    }

    /// Generate signature requirements report
    pub fn generate_requirements_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("ELECTRONIC SIGNATURE REQUIREMENTS\n");
        report.push_str("==================================\n\n");
        
        for (action, policy) in &self.requirements.requirements {
            report.push_str(&format!("Action: {action}\n"));
            report.push_str(&format!("  Required: {}\n", if policy.required { "Yes" } else { "No" }));
            report.push_str(&format!("  Meaning: {}\n", policy.meaning));
            report.push_str(&format!("  Method: {:?}\n", policy.minimum_method));
            report.push_str(&format!("  Reason Required: {}\n", if policy.requires_reason { "Yes" } else { "No" }));
            report.push('\n');
        }

        report.push_str("This report documents 21 CFR Part 11 electronic signature requirements.\n");
        report
    }
}

/// Format signature verification result
pub fn format_signature_verification(verification: &SignatureVerification) -> String {
    let mut output = String::new();
    
    output.push_str("ELECTRONIC SIGNATURE VERIFICATION\n");
    output.push_str("==================================\n\n");
    
    output.push_str(&format!("Signature ID: {}\n", verification.signature.id));
    output.push_str(&format!("Verification Status: {}\n", 
        if verification.is_valid { "✅ VALID" } else { "❌ INVALID" }));
    output.push_str(&format!("Verified At: {}\n\n", verification.verified_at));
    
    output.push_str("SIGNATURE DETAILS:\n");
    output.push_str(&format!("  User: {}\n", verification.signature.user_id));
    output.push_str(&format!("  Signed At: {}\n", verification.signature.timestamp));
    output.push_str(&format!("  Meaning: {}\n", verification.signature.meaning));
    output.push_str(&format!("  Entity: {} ({})\n", verification.signature.entity_type, verification.signature.entity_id));
    output.push_str(&format!("  Method: {:?}\n", verification.signature.signature_method));
    
    if let Some(ref reason) = verification.signature.reason {
        output.push_str(&format!("  Reason: {reason}\n"));
    }
    
    output.push_str(&format!("  Hash: {}\n", verification.signature.signature_hash));
    
    output.push_str(&format!("\nVerification Details: {}\n", verification.verification_details));
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_electronic_signature_creation() {
        let signature = ElectronicSignature::new(
            "testuser".to_string(),
            "Document approved".to_string(),
            "Document".to_string(),
            "DOC-001".to_string(),
            SignatureMethod::Password,
            None,
        ).expect("Should create signature");

        assert_eq!(signature.user_id, "testuser");
        assert_eq!(signature.meaning, "Document approved");
        assert!(signature.verify_hash());
    }

    #[test]
    fn test_signature_hash_verification() {
        let signature = ElectronicSignature::new(
            "testuser".to_string(),
            "Test signature".to_string(),
            "Document".to_string(),
            "DOC-001".to_string(),
            SignatureMethod::Password,
            None,
        ).expect("Should create signature");

        assert!(signature.verify_hash());
    }

    #[test]
    fn test_signature_requirements() {
        let requirements = SignatureRequirements::default();
        
        assert!(requirements.requirements.contains_key("document_approve"));
        assert!(requirements.requirements.contains_key("document_delete"));
        assert!(requirements.requirements.contains_key("risk_accept"));
        assert!(requirements.requirements.contains_key("system_config"));
    }

    #[test]
    fn test_signature_manager() {
        let temp_dir = std::path::PathBuf::from("/tmp/test_signatures");
        let manager = ElectronicSignatureManager::new(temp_dir);
        
        assert!(manager.requires_signature("document_approve"));
        assert!(!manager.requires_signature("unknown_action"));
    }
}
