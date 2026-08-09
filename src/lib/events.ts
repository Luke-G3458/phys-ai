/** Type-safe subscription for the sample event emitted by the Rust backend. */

import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface SampleEvent {
  data: string;
}

export const SAMPLE_EVENT = "sample";

export async function onSample(callback: (event: SampleEvent) => void): Promise<UnlistenFn> {
  return await listen<SampleEvent>(SAMPLE_EVENT, (event) => {
    callback(event.payload);
  });
}
