import { open, save } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  generateMap,
  getSimulationSnapshot,
  loadMap,
  resetSimulation,
  saveMap,
  setMotorCommand,
  setSimulationRunning,
  setVisualizationSpeed,
  stepSimulation,
} from "../../lib/commands";
import type { GenerateMapPayload, MotorCommand, SimulationSnapshot } from "../../lib/types";

const SNAPSHOT_INTERVAL_MS = 1000 / 30;

export type SimulationController = {
  snapshot: SimulationSnapshot | null;
  error: string | null;
  isLoading: boolean;
  refresh: () => Promise<void>;
  generate: (payload: GenerateMapPayload) => Promise<void>;
  chooseAndLoad: () => Promise<void>;
  chooseAndSave: () => Promise<void>;
  reset: () => Promise<void>;
  setRunning: (running: boolean) => Promise<void>;
  step: () => Promise<void>;
  setSpeed: (stepsPerSnapshot: number) => Promise<void>;
  setMotors: (command: MotorCommand) => Promise<void>;
};

export function useSimulation(): SimulationController {
  const [snapshot, setSnapshot] = useState<SimulationSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const requestInFlight = useRef(false);

  const apply = useCallback(async (request: Promise<SimulationSnapshot>) => {
    try {
      setSnapshot(await request);
      setError(null);
    } catch (requestError) {
      setError(errorMessage(requestError));
    }
  }, []);

  const refresh = useCallback(async () => {
    if (requestInFlight.current) {
      return;
    }
    requestInFlight.current = true;
    try {
      setSnapshot(await getSimulationSnapshot());
      setError(null);
    } catch (requestError) {
      setError(errorMessage(requestError));
    } finally {
      requestInFlight.current = false;
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    const initialRequest = window.setTimeout(() => void refresh(), 0);
    const interval = window.setInterval(() => void refresh(), SNAPSHOT_INTERVAL_MS);
    return () => {
      window.clearTimeout(initialRequest);
      window.clearInterval(interval);
    };
  }, [refresh]);

  const generate = useCallback(
    async (payload: GenerateMapPayload) => apply(generateMap(payload)),
    [apply],
  );

  const chooseAndLoad = useCallback(async () => {
    const path = await open({
      multiple: false,
      filters: [{ name: "Physical AI map", extensions: ["json"] }],
    });
    if (typeof path === "string") {
      await apply(loadMap(path));
    }
  }, [apply]);

  const chooseAndSave = useCallback(async () => {
    const path = await save({
      defaultPath: "physical-ai-map.json",
      filters: [{ name: "Physical AI map", extensions: ["json"] }],
    });
    if (path) {
      try {
        await saveMap(path);
        setError(null);
      } catch (requestError) {
        setError(errorMessage(requestError));
      }
    }
  }, []);

  const reset = useCallback(async () => apply(resetSimulation()), [apply]);
  const setRunning = useCallback(
    async (running: boolean) => apply(setSimulationRunning(running)),
    [apply],
  );
  const step = useCallback(async () => apply(stepSimulation()), [apply]);
  const setSpeed = useCallback(
    async (stepsPerSnapshot: number) => apply(setVisualizationSpeed(stepsPerSnapshot)),
    [apply],
  );
  const setMotors = useCallback(
    async (command: MotorCommand) => apply(setMotorCommand(command)),
    [apply],
  );

  return {
    snapshot,
    error,
    isLoading,
    refresh,
    generate,
    chooseAndLoad,
    chooseAndSave,
    reset,
    setRunning,
    step,
    setSpeed,
    setMotors,
  };
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
