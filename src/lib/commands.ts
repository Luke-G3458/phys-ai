/** Type-safe wrappers around the commands exposed by the Rust backend. */

import { invoke } from "@tauri-apps/api/core";
import type { GenerateMapPayload, MotorCommand, SimulationSnapshot } from "./types";

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

export async function getSimulationSnapshot(): Promise<SimulationSnapshot> {
  return await invoke<SimulationSnapshot>("simulation_snapshot");
}

export async function generateMap(payload: GenerateMapPayload): Promise<SimulationSnapshot> {
  return await invoke<SimulationSnapshot>("generate_map", { payload });
}

export async function loadMap(path: string): Promise<SimulationSnapshot> {
  return await invoke<SimulationSnapshot>("load_map", { path });
}

export async function saveMap(path: string): Promise<void> {
  return await invoke<void>("save_map", { path });
}

export async function resetSimulation(): Promise<SimulationSnapshot> {
  return await invoke<SimulationSnapshot>("reset_simulation");
}

export async function setSimulationRunning(running: boolean): Promise<SimulationSnapshot> {
  return await invoke<SimulationSnapshot>("set_simulation_running", { running });
}

export async function stepSimulation(): Promise<SimulationSnapshot> {
  return await invoke<SimulationSnapshot>("step_simulation");
}

export async function setVisualizationSpeed(stepsPerSnapshot: number): Promise<SimulationSnapshot> {
  return await invoke<SimulationSnapshot>("set_visualization_speed", { stepsPerSnapshot });
}

export async function setMotorCommand(command: MotorCommand): Promise<SimulationSnapshot> {
  return await invoke<SimulationSnapshot>("set_motor_command", { command });
}
