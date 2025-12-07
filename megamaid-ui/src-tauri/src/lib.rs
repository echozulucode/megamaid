mod commands;

use crate::commands::scanner::ScanResult;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Application state shared across Tauri commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub current_scan_path: Option<String>,
    pub scan_in_progress: bool,
    pub last_scan_result: Option<ScanResult>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_scan_path: None,
            scan_in_progress: false,
            last_scan_result: None,
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = Arc::new(Mutex::new(AppState::default()));

    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Scanner commands
            commands::scan_directory,
            commands::get_scan_results,
            // Detector commands
            commands::detect_cleanup_candidates,
            commands::get_default_detector_config,
            // Planner commands
            commands::generate_cleanup_plan,
            commands::save_cleanup_plan,
            commands::load_cleanup_plan,
            commands::get_plan_stats,
            // Verifier commands
            commands::verify_cleanup_plan,
            commands::get_default_verifier_config,
            // Executor commands
            commands::execute_cleanup_plan,
            commands::get_default_executor_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
