/** Type-safe wrappers around the commands exposed by the Rust backend. */

import { invoke } from "@tauri-apps/api/core";

export interface GreetPayload {
  name: string;
}

export interface GreetResponse {
  message: string;
}

/**
 * Run the sample command. The backend also emits a `sample` event containing
 * the returned message.
 */
export async function greet(payload: GreetPayload): Promise<GreetResponse> {
  return await invoke<GreetResponse>("greet", { payload });
}

/** Force close the main application window. */
export async function forceCloseWindow(): Promise<void> {
  return await invoke<void>("force_close_window");
}
