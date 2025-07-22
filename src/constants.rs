//! Regulatory Compliance Constants
//! 
//! SSOT (Single Source of Truth) for all regulatory compliance constants
//! Centralizes FDA, ISO, and other medical device regulatory requirements
//! 
//! Medical Device Compliance: FDA 21 CFR Part 820, ISO 13485, ISO 14971

/// FDA 21 CFR Part 820 Quality System Regulation Constants
pub mod fda_qsr {
    /// FDA audit retention period (7 years as required by 21 CFR Part 820.180)
    pub const AUDIT_RETENTION_DAYS: u32 = 2555; // 7 years * 365 days
    
    /// Maximum document versions to maintain for design history file
    pub const MAX_DOCUMENT_VERSIONS: u32 = 50;
    
    /// Electronic signature requirement per 21 CFR Part 11
    pub const REQUIRE_ELECTRONIC_SIGNATURE: bool = true;
    
    /// Design control documentation retention (lifetime of device + 2 years minimum)
    pub const DESIGN_CONTROL_RETENTION_YEARS: u32 = 10;
    
    /// Quality system record retention (2 years minimum per 21 CFR 820.180)
    pub const QUALITY_RECORD_RETENTION_YEARS: u32 = 2;
}

/// ISO 14971 Risk Management Constants
pub mod iso_14971 {
    /// Risk Priority Number (RPN) thresholds for medical devices
    pub const RPN_UNACCEPTABLE_THRESHOLD: u32 = 100;
    pub const RPN_ALARP_THRESHOLD: u32 = 25;
    pub const RPN_ACCEPTABLE_THRESHOLD: u32 = 1;
    
    /// Maximum RPN value (5 * 5 * 5)
    pub const MAX_RPN_VALUE: u32 = 125;
    
    /// Risk assessment scale values
    pub const MIN_RISK_SCALE: u8 = 1;
    pub const MAX_RISK_SCALE: u8 = 5;
    
    /// Post-market surveillance review frequency (months)
    pub const SURVEILLANCE_REVIEW_FREQUENCY_MONTHS: u32 = 6;
}

/// ISO 13485 Quality Management System Constants
pub mod iso_13485 {
    /// Management review frequency (months)
    pub const MANAGEMENT_REVIEW_FREQUENCY_MONTHS: u32 = 12;
    
    /// Internal audit frequency (months)
    pub const INTERNAL_AUDIT_FREQUENCY_MONTHS: u32 = 12;
    
    /// Corrective action response time (days)
    pub const CORRECTIVE_ACTION_RESPONSE_DAYS: u32 = 30;
    
    /// Document control review cycle (months)
    pub const DOCUMENT_REVIEW_CYCLE_MONTHS: u32 = 24;
}

/// FDA 21 CFR Part 11 Electronic Records Constants
pub mod cfr_part_11 {
    /// Electronic signature validation requirements
    pub const REQUIRE_BIOMETRIC_VERIFICATION: bool = false; // Optional for Class II devices
    pub const REQUIRE_PASSWORD_COMPLEXITY: bool = true;
    pub const MIN_PASSWORD_LENGTH: usize = 8;
    
    /// Audit trail requirements
    pub const REQUIRE_AUDIT_TRAIL: bool = true;
    pub const AUDIT_TRAIL_IMMUTABLE: bool = true;
    
    /// Electronic record retention
    pub const ELECTRONIC_RECORD_BACKUP_REQUIRED: bool = true;
    pub const ELECTRONIC_RECORD_ENCRYPTION_REQUIRED: bool = true;
}

/// System Configuration Constants
pub mod system {
    /// Default system configuration values
    pub const DEFAULT_LOG_LEVEL: &str = "INFO";
    pub const DEFAULT_VERSION: &str = "1.0.0";
    pub const DEFAULT_PROJECT_DIR: &str = ".qms";
    
    /// Performance and reliability constants
    pub const MAX_CONCURRENT_USERS: usize = 100;
    pub const SESSION_TIMEOUT_HOURS: u64 = 8;
    pub const MAX_FILE_SIZE_MB: usize = 100;
    
    /// Backup and recovery constants
    pub const BACKUP_ENABLED_DEFAULT: bool = true;
    pub const ENCRYPTION_ENABLED_DEFAULT: bool = true;
    pub const AUTO_BACKUP_FREQUENCY_HOURS: u64 = 24;
}

/// Validation Constants
pub mod validation {
    /// ID format validation
    pub const MIN_ID_LENGTH: usize = 3;
    pub const MAX_ID_LENGTH: usize = 50;
    pub const VALID_ID_PATTERN: &str = r"^[A-Z]{2,5}-\d{3,6}$";
    
    /// Username validation
    pub const MIN_USERNAME_LENGTH: usize = 3;
    pub const MAX_USERNAME_LENGTH: usize = 50;
    pub const VALID_USERNAME_PATTERN: &str = r"^[a-zA-Z0-9_.-]+$";
    
    /// Document validation
    pub const MIN_DOCUMENT_TITLE_LENGTH: usize = 5;
    pub const MAX_DOCUMENT_TITLE_LENGTH: usize = 200;
    pub const MAX_DOCUMENT_DESCRIPTION_LENGTH: usize = 1000;
}

/// Medical Device Classification Constants
pub mod device_classification {
    /// FDA Device Classes
    pub const CLASS_I: &str = "Class I";
    pub const CLASS_II: &str = "Class II";
    pub const CLASS_III: &str = "Class III";
    
    /// Risk categories per ISO 14971
    pub const RISK_CATEGORY_SAFETY: &str = "Safety";
    pub const RISK_CATEGORY_SECURITY: &str = "Security";
    pub const RISK_CATEGORY_PERFORMANCE: &str = "Performance";
    pub const RISK_CATEGORY_USABILITY: &str = "Usability";
}

/// Compliance Standards References
pub mod standards {
    /// Standard identifiers for traceability
    pub const FDA_21_CFR_PART_820: &str = "FDA_21_CFR_Part_820";
    pub const FDA_21_CFR_PART_11: &str = "21_CFR_Part_11";
    pub const ISO_13485: &str = "ISO_13485";
    pub const ISO_14971: &str = "ISO_14971";
    pub const ISO_62304: &str = "ISO_62304"; // Medical device software
    pub const IEC_62366: &str = "IEC_62366"; // Usability engineering
    
    /// Standard versions for reference
    pub const ISO_14971_VERSION: &str = "2019";
    pub const ISO_13485_VERSION: &str = "2016";
    pub const ISO_62304_VERSION: &str = "2006";
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fda_constants() {
        assert_eq!(fda_qsr::AUDIT_RETENTION_DAYS, 2555);
        assert_eq!(fda_qsr::MAX_DOCUMENT_VERSIONS, 50);
        assert!(fda_qsr::REQUIRE_ELECTRONIC_SIGNATURE);
    }
    
    #[test]
    fn test_iso_14971_constants() {
        assert_eq!(iso_14971::RPN_UNACCEPTABLE_THRESHOLD, 100);
        assert_eq!(iso_14971::RPN_ALARP_THRESHOLD, 25);
        assert_eq!(iso_14971::MAX_RPN_VALUE, 125);
    }
    
    #[test]
    fn test_validation_constants() {
        assert_eq!(validation::MIN_ID_LENGTH, 3);
        assert_eq!(validation::MAX_ID_LENGTH, 50);
        assert!(!validation::VALID_ID_PATTERN.is_empty());
    }
    
    #[test]
    fn test_standards_references() {
        assert_eq!(standards::FDA_21_CFR_PART_820, "FDA_21_CFR_Part_820");
        assert_eq!(standards::ISO_14971, "ISO_14971");
        assert_eq!(standards::ISO_14971_VERSION, "2019");
    }
}
