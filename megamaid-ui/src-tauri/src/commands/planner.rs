use megamaid::planner::{PlanGenerator, PlanWriter};
use megamaid::detector::DetectionResult;
use megamaid::models::{CleanupPlan, CleanupAction};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanConfig {
    pub base_path: String,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStats {
    pub total_entries: usize,
    pub delete_count: usize,
    pub review_count: usize,
    pub keep_count: usize,
    pub total_size: u64,
}

/// Generate a cleanup plan from detection results
#[tauri::command]
pub async fn generate_cleanup_plan(
    detections: Vec<DetectionResult>,
    config: PlanConfig,
) -> Result<CleanupPlan, String> {
    let base_path = PathBuf::from(&config.base_path);
    let generator = PlanGenerator::new(base_path);
    let plan = generator.generate(detections);

    Ok(plan)
}

/// Save a cleanup plan to disk
#[tauri::command]
pub async fn save_cleanup_plan(
    plan: CleanupPlan,
    output_path: String,
) -> Result<(), String> {
    let path = PathBuf::from(output_path);
    PlanWriter::write(&plan, &path)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Load a cleanup plan from disk
#[tauri::command]
pub async fn load_cleanup_plan(path: String) -> Result<CleanupPlan, String> {
    let plan_path = PathBuf::from(path);

    if !plan_path.exists() {
        return Err(format!("Plan file does not exist: {:?}", plan_path));
    }

    let content = std::fs::read_to_string(&plan_path)
        .map_err(|e| format!("Failed to read plan file: {}", e))?;

    let plan: CleanupPlan = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse plan file: {}", e))?;

    Ok(plan)
}

/// Get statistics from a cleanup plan
#[tauri::command]
pub async fn get_plan_stats(plan: CleanupPlan) -> Result<PlanStats, String> {
    let total_entries = plan.entries.len();
    let delete_count = plan.entries.iter()
        .filter(|e| e.action == CleanupAction::Delete)
        .count();
    let review_count = plan.entries.iter()
        .filter(|e| e.action == CleanupAction::Review)
        .count();
    let keep_count = plan.entries.iter()
        .filter(|e| e.action == CleanupAction::Keep)
        .count();
    let total_size: u64 = plan.entries.iter()
        .map(|e| e.size)
        .sum();

    Ok(PlanStats {
        total_entries,
        delete_count,
        review_count,
        keep_count,
        total_size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_config_serialization() {
        let config = PlanConfig {
            base_path: "/test/base".to_string(),
            output_path: "/test/output.yaml".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: PlanConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.base_path, deserialized.base_path);
        assert_eq!(config.output_path, deserialized.output_path);
    }
}
