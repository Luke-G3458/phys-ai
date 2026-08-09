//! Window management commands

use tauri::{AppHandle, Manager};

/// Force close the main window
#[tauri::command]
pub async fn force_close_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.destroy().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Main window not found".to_string())
    }
}
