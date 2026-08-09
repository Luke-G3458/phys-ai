//! Sample event emitted after the sample `greet` command runs.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

/// Event name used by both the backend emitter and frontend listener.
pub const SAMPLE_EVENT: &str = "sample";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleEvent {
    pub data: String,
}

/// Emit the sample event to all open webviews.
pub fn emit_sample(app: &AppHandle, event: SampleEvent) -> Result<(), tauri::Error> {
    app.emit(SAMPLE_EVENT, event)
}
