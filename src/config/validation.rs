//! Configuration validation.

use super::schema::{CustomRule, MegamaidConfig};
use anyhow::{Context, Result};

/// Validates a configuration.
pub fn validate_config(config: &MegamaidConfig) -> Result<()> {
    validate_scanner(&config.scanner)?;
    validate_detector(&config.detector)?;
    validate_executor(&config.executor)?;
    validate_output(&config.output)?;
    Ok(())
}

fn validate_scanner(scanner: &super::schema::ScannerConfig) -> Result<()> {
    if let Some(depth) = scanner.max_depth {
        if depth == 0 {
            anyhow::bail!("scanner.max_depth must be at least 1 (or null for unlimited)");
        }
        if depth > 1000 {
            anyhow::bail!("scanner.max_depth cannot exceed 1000 (got {})", depth);
        }
    }

    if scanner.thread_count > 256 {
        anyhow::bail!(
            "scanner.thread_count cannot exceed 256 (got {})",
            scanner.thread_count
        );
    }

    Ok(())
}

fn validate_detector(detector: &super::schema::DetectorConfig) -> Result<()> {
    // Validate size threshold
    if detector.rules.size_threshold.threshold_mb == 0 {
        anyhow::bail!("detector.rules.size_threshold.threshold_mb must be greater than 0");
    }

    if detector.rules.size_threshold.threshold_mb > 1_000_000 {
        anyhow::bail!(
            "detector.rules.size_threshold.threshold_mb cannot exceed 1,000,000 MB (got {})",
            detector.rules.size_threshold.threshold_mb
        );
    }

    // Validate custom rules
    for rule in &detector.custom_rules {
        validate_custom_rule(rule)
            .context(format!("Invalid custom rule: {}", rule.name))?;
    }

    Ok(())
}

fn validate_custom_rule(rule: &CustomRule) -> Result<()> {
    if rule.name.is_empty() {
        anyhow::bail!("Custom rule name cannot be empty");
    }

    if rule.description.is_empty() {
        anyhow::bail!("Custom rule description cannot be empty");
    }

    // At least one matching criterion must be specified
    if rule.pattern.is_none()
        && rule.extensions.is_none()
        && rule.min_age_days.is_none()
        && rule.min_size_mb.is_none()
    {
        anyhow::bail!(
            "Custom rule '{}' must specify at least one matching criterion (pattern, extensions, min_age_days, or min_size_mb)",
            rule.name
        );
    }

    // Validate min_age_days
    if let Some(age) = rule.min_age_days {
        if age == 0 {
            anyhow::bail!("min_age_days must be greater than 0");
        }
        if age > 36500 {
            // ~100 years
            anyhow::bail!("min_age_days cannot exceed 36500 (got {})", age);
        }
    }

    // Validate min_size_mb
    if let Some(size) = rule.min_size_mb {
        if size == 0 {
            anyhow::bail!("min_size_mb must be greater than 0");
        }
    }

    // Validate extensions format
    if let Some(ref exts) = rule.extensions {
        if exts.is_empty() {
            anyhow::bail!("extensions list cannot be empty");
        }
        for ext in exts {
            if !ext.starts_with('.') {
                anyhow::bail!(
                    "Extension '{}' must start with a dot (e.g., '.txt')",
                    ext
                );
            }
        }
    }

    Ok(())
}

fn validate_executor(executor: &super::schema::ExecutorConfig) -> Result<()> {
    if executor.batch_size == 0 {
        anyhow::bail!("executor.batch_size must be greater than 0");
    }

    if executor.batch_size > 10000 {
        anyhow::bail!(
            "executor.batch_size cannot exceed 10000 (got {})",
            executor.batch_size
        );
    }

    Ok(())
}

fn validate_output(output: &super::schema::OutputConfig) -> Result<()> {
    if output.plan_file.is_empty() {
        anyhow::bail!("output.plan_file cannot be empty");
    }

    if output.log_file.is_empty() {
        anyhow::bail!("output.log_file cannot be empty");
    }

    if output.drift_report.is_empty() {
        anyhow::bail!("output.drift_report cannot be empty");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::*;
    use crate::models::CleanupAction;

    #[test]
    fn test_validate_valid_config() {
        let config = MegamaidConfig::default();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_scanner_max_depth_zero() {
        let mut config = MegamaidConfig::default();
        config.scanner.max_depth = Some(0);

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least 1"));
    }

    #[test]
    fn test_validate_scanner_max_depth_too_large() {
        let mut config = MegamaidConfig::default();
        config.scanner.max_depth = Some(2000);

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("1000"));
    }

    #[test]
    fn test_validate_scanner_thread_count_too_large() {
        let mut config = MegamaidConfig::default();
        config.scanner.thread_count = 300;

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("256"));
    }

    #[test]
    fn test_validate_size_threshold_zero() {
        let mut config = MegamaidConfig::default();
        config.detector.rules.size_threshold.threshold_mb = 0;

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be greater than 0"));
    }

    #[test]
    fn test_validate_size_threshold_too_large() {
        let mut config = MegamaidConfig::default();
        config.detector.rules.size_threshold.threshold_mb = 2_000_000;

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("1,000,000"));
    }

    #[test]
    fn test_validate_batch_size_zero() {
        let mut config = MegamaidConfig::default();
        config.executor.batch_size = 0;

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be greater than 0"));
    }

    #[test]
    fn test_validate_batch_size_too_large() {
        let mut config = MegamaidConfig::default();
        config.executor.batch_size = 20000;

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("10000"));
    }

    #[test]
    fn test_validate_custom_rule_valid() {
        let rule = CustomRule {
            name: "test".to_string(),
            description: "Test rule".to_string(),
            pattern: Some("*.log".to_string()),
            extensions: None,
            min_age_days: Some(30),
            min_size_mb: Some(10),
            action: CleanupAction::Delete,
        };

        assert!(validate_custom_rule(&rule).is_ok());
    }

    #[test]
    fn test_validate_custom_rule_empty_name() {
        let rule = CustomRule {
            name: "".to_string(),
            description: "Test".to_string(),
            pattern: Some("*.log".to_string()),
            extensions: None,
            min_age_days: None,
            min_size_mb: None,
            action: CleanupAction::Delete,
        };

        let result = validate_custom_rule(&rule);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name cannot be empty"));
    }

    #[test]
    fn test_validate_custom_rule_no_criteria() {
        let rule = CustomRule {
            name: "test".to_string(),
            description: "Test".to_string(),
            pattern: None,
            extensions: None,
            min_age_days: None,
            min_size_mb: None,
            action: CleanupAction::Delete,
        };

        let result = validate_custom_rule(&rule);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least one matching criterion"));
    }

    #[test]
    fn test_validate_custom_rule_invalid_extension() {
        let rule = CustomRule {
            name: "test".to_string(),
            description: "Test".to_string(),
            pattern: None,
            extensions: Some(vec!["txt".to_string()]), // Missing dot
            min_age_days: None,
            min_size_mb: None,
            action: CleanupAction::Delete,
        };

        let result = validate_custom_rule(&rule);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must start with a dot"));
    }

    #[test]
    fn test_validate_custom_rule_age_too_large() {
        let rule = CustomRule {
            name: "test".to_string(),
            description: "Test".to_string(),
            pattern: Some("*.log".to_string()),
            extensions: None,
            min_age_days: Some(50000),
            min_size_mb: None,
            action: CleanupAction::Delete,
        };

        let result = validate_custom_rule(&rule);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("36500"));
    }

    #[test]
    fn test_validate_output_empty_filenames() {
        let mut config = MegamaidConfig::default();
        config.output.plan_file = "".to_string();

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }
}
