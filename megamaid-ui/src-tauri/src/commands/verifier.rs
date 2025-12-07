use megamaid::verifier::{VerificationEngine, VerificationConfig, VerificationResult};
use megamaid::models::CleanupPlan;

/// Verify a cleanup plan for drift
#[tauri::command]
pub async fn verify_cleanup_plan(
    plan: CleanupPlan,
    config: VerificationConfig,
) -> Result<VerificationResult, String> {
    let verifier = VerificationEngine::new(config);
    let result = verifier
        .verify(&plan)
        .map_err(|e| e.to_string())?;

    Ok(result)
}

/// Get default verifier configuration
#[tauri::command]
pub async fn get_default_verifier_config() -> Result<VerificationConfig, String> {
    Ok(VerificationConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

#[test]
fn test_verifier_config_serialization() {
    let config = VerificationConfig {
        check_size: true,
        check_mtime: true,
        fail_fast: false,
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: VerificationConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.check_size, deserialized.check_size);
        assert_eq!(config.check_mtime, deserialized.check_mtime);
        assert_eq!(config.fail_fast, deserialized.fail_fast);
    }
}
