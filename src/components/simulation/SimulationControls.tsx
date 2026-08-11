import { Pause, Play, RotateCcw, Save, SkipForward, Square, Upload } from "lucide-react";
import { useState } from "react";
import type { GenerateMapPayload, MotorCommand } from "../../lib/types";
import type { SimulationController } from "./useSimulation";

const DEFAULT_MAP: GenerateMapPayload = {
  width: 20,
  height: 12,
  wallCount: 14,
  wallThickness: 0.15,
  minimumWallLength: 1,
  maximumWallLength: 4,
};

const SPEED_OPTIONS = [1, 4, 16, 64, 256, 1024];

export function SimulationControls({ controller }: { controller: SimulationController }) {
  const { snapshot } = controller;
  const [mapSettings, setMapSettings] = useState(DEFAULT_MAP);
  const motorCommand = snapshot?.robot.motorCommand ?? { left: 0, right: 0 };

  const updateMapSetting = (key: "width" | "height" | "wallCount", value: number) => {
    setMapSettings((current) => ({ ...current, [key]: value }));
  };

  const updateMotor = (side: keyof MotorCommand, value: number) => {
    void controller.setMotors({ ...motorCommand, [side]: value });
  };

  return (
    <div className="flex h-full flex-col gap-5 overflow-y-auto p-4 text-sm">
      <section>
        <h2 className="mb-3 text-xs font-semibold uppercase tracking-[0.16em] text-black/50">
          Map
        </h2>
        <div className="grid grid-cols-2 gap-2">
          <NumberField
            label="Width (m)"
            value={mapSettings.width}
            minimum={2}
            onChange={(value) => updateMapSetting("width", value)}
          />
          <NumberField
            label="Height (m)"
            value={mapSettings.height}
            minimum={2}
            onChange={(value) => updateMapSetting("height", value)}
          />
          <NumberField
            label="Walls"
            value={mapSettings.wallCount}
            minimum={0}
            integer
            onChange={(value) => updateMapSetting("wallCount", value)}
          />
        </div>
        <button
          className="primary-button mt-3 w-full"
          onClick={() => void controller.generate(mapSettings)}
        >
          Generate map
        </button>
        <div className="mt-2 grid grid-cols-2 gap-2">
          <button className="secondary-button" onClick={() => void controller.chooseAndLoad()}>
            <Upload size={14} /> Load
          </button>
          <button className="secondary-button" onClick={() => void controller.chooseAndSave()}>
            <Save size={14} /> Save
          </button>
        </div>
      </section>

      <section>
        <h2 className="mb-3 text-xs font-semibold uppercase tracking-[0.16em] text-black/50">
          Simulation
        </h2>
        <div className="grid grid-cols-3 gap-2">
          <button
            className="secondary-button"
            onClick={() => void controller.setRunning(!snapshot?.running)}
          >
            {snapshot?.running ? <Pause size={14} /> : <Play size={14} />}
            {snapshot?.running ? "Pause" : "Run"}
          </button>
          <button className="secondary-button" onClick={() => void controller.step()}>
            <SkipForward size={14} /> Step
          </button>
          <button className="secondary-button" onClick={() => void controller.reset()}>
            <RotateCcw size={14} /> Reset
          </button>
        </div>
        <label className="mt-3 block text-xs text-black/60">
          Steps per frame
          <select
            className="control-input mt-1 w-full"
            value={snapshot?.stepsPerSnapshot ?? 4}
            onChange={(event) => void controller.setSpeed(Number(event.target.value))}
          >
            {SPEED_OPTIONS.map((speed) => (
              <option value={speed} key={speed}>
                {speed}
              </option>
            ))}
          </select>
        </label>
      </section>

      <section>
        <div className="mb-3 flex items-center justify-between">
          <h2 className="text-xs font-semibold uppercase tracking-[0.16em] text-black/50">
            Motors
          </h2>
          <button
            className="flex items-center gap-1 text-xs font-medium text-red-700 hover:text-red-900"
            onClick={() => void controller.setMotors({ left: 0, right: 0 })}
          >
            <Square size={11} fill="currentColor" /> Stop
          </button>
        </div>
        <MotorSlider
          label="Left"
          value={motorCommand.left}
          onChange={(value) => updateMotor("left", value)}
        />
        <MotorSlider
          label="Right"
          value={motorCommand.right}
          onChange={(value) => updateMotor("right", value)}
        />
      </section>

      {snapshot && (
        <section className="mt-auto rounded-lg border border-black/10 bg-white/70 p-3 font-mono text-[11px] leading-5 text-black/60">
          <div className="flex justify-between">
            <span>Pose</span>
            <span>
              {snapshot.robot.pose.x.toFixed(2)}, {snapshot.robot.pose.y.toFixed(2)} m
            </span>
          </div>
          <div className="flex justify-between">
            <span>Heading</span>
            <span>{((snapshot.robot.pose.orientation * 180) / Math.PI).toFixed(1)}°</span>
          </div>
          <div className="flex justify-between">
            <span>Sim time</span>
            <span>{snapshot.elapsedSeconds.toFixed(2)} s</span>
          </div>
          <div className="flex justify-between">
            <span>Steps</span>
            <span>{snapshot.stepCount.toLocaleString()}</span>
          </div>
          <div className="flex justify-between">
            <span>Collision</span>
            <span className={snapshot.robot.collided ? "text-red-700" : "text-emerald-700"}>
              {snapshot.robot.collided ? "yes" : "no"}
            </span>
          </div>
        </section>
      )}
    </div>
  );
}

function NumberField({
  label,
  value,
  minimum,
  integer = false,
  onChange,
}: {
  label: string;
  value: number;
  minimum: number;
  integer?: boolean;
  onChange: (value: number) => void;
}) {
  return (
    <label className="text-xs text-black/60">
      {label}
      <input
        className="control-input mt-1 w-full"
        type="number"
        min={minimum}
        step={integer ? 1 : 0.5}
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
      />
    </label>
  );
}

function MotorSlider({
  label,
  value,
  onChange,
}: {
  label: string;
  value: number;
  onChange: (value: number) => void;
}) {
  return (
    <label className="mb-3 block">
      <span className="mb-1 flex justify-between text-xs text-black/60">
        <span>{label}</span>
        <span className="font-mono">{value.toFixed(2)}</span>
      </span>
      <input
        className="w-full accent-emerald-700"
        type="range"
        min={-1}
        max={1}
        step={0.01}
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
      />
    </label>
  );
}
