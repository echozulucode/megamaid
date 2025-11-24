use megamaid::executor::{ExecutionEngine, ExecutionConfig};
use megamaid::models::CleanupPlan;

/// Execute a cleanup plan
#[tauri::command]
pub async fn execute_cleanup_plan(
    plan: CleanupPlan,
    config: ExecutionConfig,
) -> Result<String, String> {
    let engine = ExecutionEngine::new(config);
    let result = engine
        .execute(&plan)
        .map_err(|e| e.to_string())?;

    // Convert result to JSON for now
    Ok(format!("Executed {} operations: {} successful, {} failed, {} skipped. Space freed: {} bytes",
        result.summary.total_operations,
        result.summary.successful,
        result.summary.failed,
        result.summary.skipped,
        result.summary.space_freed
    ))
}

/// Get default executor configuration
#[tauri::command]
pub async fn get_default_executor_config() -> Result<ExecutionConfig, String> {
    Ok(ExecutionConfig::default())
}

// Transaction log commands removed for Phase 4.1 - will be added in future phases
