use megamaid::detector::{DetectionEngine, DetectionResult, SizeThresholdRule, BuildArtifactRule, ScanContext};
use megamaid::models::FileEntry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorConfig {
    pub size_threshold_mb: Option<u64>,
    pub enable_build_artifacts: bool,
}

/// Detect cleanup candidates from file entries
#[tauri::command]
pub async fn detect_cleanup_candidates(
    entries: Vec<FileEntry>,
    config: DetectorConfig,
) -> Result<Vec<DetectionResult>, String> {
    let mut engine = DetectionEngine::empty();

    // Add size threshold rule if configured
    if let Some(threshold_mb) = config.size_threshold_mb {
        let threshold_bytes = threshold_mb * 1024 * 1024;
        engine.add_rule(Box::new(SizeThresholdRule {
            threshold_bytes,
        }));
    }

    // Add build artifact rule if enabled
    if config.enable_build_artifacts {
        engine.add_rule(Box::new(BuildArtifactRule::default()));
    }

    let context = ScanContext::default();
    let results = engine.analyze(&entries, &context);

    Ok(results)
}

/// Get default detector configuration
#[tauri::command]
pub async fn get_default_detector_config() -> Result<DetectorConfig, String> {
    Ok(DetectorConfig {
        size_threshold_mb: Some(100),
        enable_build_artifacts: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_config_serialization() {
        let config = DetectorConfig {
            size_threshold_mb: Some(100),
            enable_build_artifacts: true,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: DetectorConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.size_threshold_mb, deserialized.size_threshold_mb);
        assert_eq!(config.enable_build_artifacts, deserialized.enable_build_artifacts);
    }
}
