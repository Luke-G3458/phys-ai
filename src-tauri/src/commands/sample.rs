//! Sample Tauri command for Phase 0

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::events::sample::{emit_sample, SampleEvent};

#[derive(Debug, Serialize, Deserialize)]
pub struct GreetPayload {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GreetResponse {
    pub message: String,
}

/// Sample command that demonstrates IPC
#[tauri::command]
pub fn greet(app: AppHandle, payload: GreetPayload) -> Result<GreetResponse, String> {
    let message = format!("Hello from Rust, {}! 🚀", payload.name);

    emit_sample(
        &app,
        SampleEvent {
            data: message.clone(),
        },
    )
    .map_err(|error| error.to_string())?;

    Ok(GreetResponse { message })
}
