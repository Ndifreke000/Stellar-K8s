//! Tests for CVE handling functionality

#[cfg(test)]
mod tests {
    use crate::controller::cve::{
        CVECount, CVEDetectionResult, CVERolloutStatus, CanaryTestStatus, Vulnerability,
        VulnerabilitySeverity,
    };
    use crate::crd::CVEHandlingConfig;
    use chrono::Utc;

    #[test]
    fn test_cve_handling_config_defaults() {
        let config = CVEHandlingConfig::default();
        assert!(config.enabled);
        assert_eq!(config.scan_interval_secs, 3600);
        assert!(!config.critical_only);
        assert_eq!(config.canary_test_timeout_secs, 300);
        assert_eq!(config.canary_pass_rate_threshold, 100.0);
        assert!(config.enable_auto_rollback);
        assert_eq!(config.consensus_health_threshold, 0.95);
    }

    #[test]
    fn test_cve_detection_result_requires_patch() {
        let result_with_critical = CVEDetectionResult {
            current_image: "stellar/core:v21.0.0".to_string(),
            vulnerabilities: vec![Vulnerability {
                cve_id: "CVE-2024-1234".to_string(),
                severity: VulnerabilitySeverity::Critical,
                package: "openssl".to_string(),
                installed_version: "1.0.0".to_string(),
                fixed_version: Some("1.0.1".to_string()),
                description: "Critical vulnerability in OpenSSL".to_string(),
            }],
            patched_version: Some("stellar/core:v21.0.1".to_string()),
            scan_timestamp: Utc::now(),
            cve_count: CVECount {
                critical: 1,
                ..Default::default()
            },
            has_critical: true,
        };

        assert!(result_with_critical.requires_urgent_patch());
        assert!(result_with_critical.can_patch());
    }

    #[test]
    fn test_cve_count_total() {
        let count = CVECount {
            critical: 1,
            high: 2,
            medium: 3,
            low: 4,
            unknown: 5,
        };
        assert_eq!(count.total(), 15);
    }

    #[test]
    fn test_vulnerability_severity_ordering() {
        assert!(VulnerabilitySeverity::Critical > VulnerabilitySeverity::High);
        assert!(VulnerabilitySeverity::High > VulnerabilitySeverity::Medium);
        assert!(VulnerabilitySeverity::Medium > VulnerabilitySeverity::Low);
        assert!(VulnerabilitySeverity::Low > VulnerabilitySeverity::Unknown);
    }

    #[test]
    fn test_canary_test_status_string_repr() {
        assert_eq!(CanaryTestStatus::Pending.as_str(), "Pending");
        assert_eq!(CanaryTestStatus::Running.as_str(), "Running");
        assert_eq!(CanaryTestStatus::Passed.as_str(), "Passed");
        assert_eq!(CanaryTestStatus::Failed.as_str(), "Failed");
        assert_eq!(CanaryTestStatus::Timeout.as_str(), "Timeout");
    }

    #[test]
    fn test_cve_rollout_status_string_repr() {
        assert_eq!(CVERolloutStatus::Idle.as_str(), "Idle");
        assert_eq!(CVERolloutStatus::CanaryTesting.as_str(), "CanaryTesting");
        assert_eq!(CVERolloutStatus::Rolling.as_str(), "Rolling");
        assert_eq!(CVERolloutStatus::Complete.as_str(), "Complete");
        assert_eq!(CVERolloutStatus::RollingBack.as_str(), "RollingBack");
        assert_eq!(CVERolloutStatus::RolledBack.as_str(), "RolledBack");
        assert_eq!(CVERolloutStatus::Failed.as_str(), "Failed");
    }

    #[test]
    fn test_cve_config_critical_only() {
        let config = CVEHandlingConfig {
            enabled: true,
            scan_interval_secs: 3600,
            critical_only: true,
            canary_test_timeout_secs: 300,
            canary_pass_rate_threshold: 100.0,
            enable_auto_rollback: true,
            consensus_health_threshold: 0.95,
        };

        assert!(config.critical_only);
        assert!(config.enable_auto_rollback);
    }

    #[test]
    fn test_cve_detection_without_patch() {
        let result = CVEDetectionResult {
            current_image: "stellar/core:v21.0.0".to_string(),
            vulnerabilities: vec![],
            patched_version: None,
            scan_timestamp: Utc::now(),
            cve_count: CVECount {
                critical: 1,
                ..Default::default()
            },
            has_critical: true,
        };

        assert!(result.requires_urgent_patch());
        assert!(!result.can_patch()); // No patch available
    }

    #[test]
    fn test_cve_config_aggressive_patching() {
        let config = CVEHandlingConfig {
            enabled: true,
            scan_interval_secs: 1800,      // 30 minutes
            critical_only: false,          // Patch all levels
            canary_test_timeout_secs: 180, // 3 minutes
            canary_pass_rate_threshold: 100.0,
            enable_auto_rollback: true,
            consensus_health_threshold: 0.90, // Less strict
        };

        assert!(!config.critical_only);
        assert_eq!(config.scan_interval_secs, 1800);
        assert_eq!(config.canary_test_timeout_secs, 180);
        assert_eq!(config.consensus_health_threshold, 0.90);
    }

    #[test]
    fn test_cve_config_manual_rollback() {
        let config = CVEHandlingConfig {
            enabled: true,
            scan_interval_secs: 3600,
            critical_only: false,
            canary_test_timeout_secs: 300,
            canary_pass_rate_threshold: 100.0,
            enable_auto_rollback: false, // Disable auto-rollback
            consensus_health_threshold: 0.95,
        };

        assert!(!config.enable_auto_rollback);
    }
}
