//! Commands for creating, controlling, and inspecting the simulation.

use amr_sim::{Amr, AmrConfig, MotorCommand};
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use world_sim::{Bounds, Map, MapConfig, Pose2, Simulation, SimulationSnapshot};

const FIXED_STEP_SECONDS: f32 = 1.0 / 120.0;
const DEFAULT_STEPS_PER_SNAPSHOT: u32 = 4;
const MAX_STEPS_PER_SNAPSHOT: u32 = 4096;
const MAP_GENERATION_ATTEMPTS: usize = 32;
const VISUALIZATION_LIDAR_RAY_COUNT: usize = 16;
const VISUALIZATION_LIDAR_MAXIMUM_RANGE: f32 = 5.0;

pub struct AppSimulation {
    simulation: Simulation<Amr>,
    reset_map: Map,
    running: bool,
    steps_per_snapshot: u32,
}

impl AppSimulation {
    pub fn create_default() -> Result<Self, String> {
        let mut rng = thread_rng();
        for _ in 0..MAP_GENERATION_ATTEMPTS {
            let map = Map::generate(MapConfig::default(), &mut rng).map_err(stringify)?;
            if let Ok(simulation) = create_simulation(map.clone()) {
                return Ok(Self {
                    simulation,
                    reset_map: map,
                    running: false,
                    steps_per_snapshot: DEFAULT_STEPS_PER_SNAPSHOT,
                });
            }
        }
        Err("could not generate a map with room for the AMR".into())
    }

    fn install_map(&mut self, map: Map, remember_for_reset: bool) -> Result<(), String> {
        let simulation = create_simulation(map.clone())?;
        self.simulation = simulation;
        if remember_for_reset {
            self.reset_map = map;
        }
        self.running = false;
        Ok(())
    }

    fn snapshot(&self) -> AppSnapshot {
        let robot = self.simulation.module();
        let config = robot.config();
        AppSnapshot {
            world: self.simulation.snapshot(),
            robot: RobotSnapshot {
                pose: robot.pose(),
                length: config.length,
                width: config.width,
                motor_command: robot.motor_command(),
                lidar: robot.lidar().to_vec(),
                lidar_maximum_range: config.lidar_maximum_range,
                collided: robot.observation().collided,
            },
            running: self.running,
            steps_per_snapshot: self.steps_per_snapshot,
            fixed_step_seconds: FIXED_STEP_SECONDS,
            elapsed_seconds: self.simulation.elapsed_seconds(),
            step_count: self.simulation.step_count(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub world: SimulationSnapshot,
    pub robot: RobotSnapshot,
    pub running: bool,
    pub steps_per_snapshot: u32,
    pub fixed_step_seconds: f32,
    pub elapsed_seconds: f64,
    pub step_count: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotSnapshot {
    pub pose: Pose2,
    pub length: f32,
    pub width: f32,
    pub motor_command: MotorCommand,
    pub lidar: Vec<f32>,
    pub lidar_maximum_range: f32,
    pub collided: bool,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateMapPayload {
    pub width: f32,
    pub height: f32,
    pub wall_count: usize,
    pub wall_thickness: f32,
    pub minimum_wall_length: f32,
    pub maximum_wall_length: f32,
}

impl From<GenerateMapPayload> for MapConfig {
    fn from(payload: GenerateMapPayload) -> Self {
        Self {
            bounds: Bounds::new(payload.width, payload.height),
            wall_count: payload.wall_count,
            wall_thickness: payload.wall_thickness,
            minimum_wall_length: payload.minimum_wall_length,
            maximum_wall_length: payload.maximum_wall_length,
        }
    }
}

#[tauri::command]
pub fn simulation_snapshot(state: State<'_, Mutex<AppSimulation>>) -> Result<AppSnapshot, String> {
    let mut state = lock(&state)?;
    if state.running {
        let step_count = state.steps_per_snapshot as usize;
        state
            .simulation
            .run_steps(step_count, FIXED_STEP_SECONDS)
            .map_err(stringify)?;
    }
    Ok(state.snapshot())
}

#[tauri::command]
pub fn generate_map(
    state: State<'_, Mutex<AppSimulation>>,
    payload: GenerateMapPayload,
) -> Result<AppSnapshot, String> {
    let mut state = lock(&state)?;
    let config = MapConfig::from(payload);
    let mut rng = thread_rng();
    for _ in 0..MAP_GENERATION_ATTEMPTS {
        let map = Map::generate(config, &mut rng).map_err(stringify)?;
        if state.install_map(map, true).is_ok() {
            return Ok(state.snapshot());
        }
    }
    Err("could not generate a map with room for the AMR".into())
}

#[tauri::command]
pub fn load_map(
    state: State<'_, Mutex<AppSimulation>>,
    path: PathBuf,
) -> Result<AppSnapshot, String> {
    let map = Map::load(BufReader::new(File::open(path).map_err(stringify)?)).map_err(stringify)?;
    let mut state = lock(&state)?;
    state.install_map(map, true)?;
    Ok(state.snapshot())
}

#[tauri::command]
pub fn save_map(state: State<'_, Mutex<AppSimulation>>, path: PathBuf) -> Result<(), String> {
    let state = lock(&state)?;
    state
        .simulation
        .map()
        .save(BufWriter::new(File::create(path).map_err(stringify)?))
        .map_err(stringify)
}

#[tauri::command]
pub fn reset_simulation(state: State<'_, Mutex<AppSimulation>>) -> Result<AppSnapshot, String> {
    let mut state = lock(&state)?;
    let map = state.reset_map.clone();
    state.install_map(map, false)?;
    Ok(state.snapshot())
}

#[tauri::command]
pub fn set_simulation_running(
    state: State<'_, Mutex<AppSimulation>>,
    running: bool,
) -> Result<AppSnapshot, String> {
    let mut state = lock(&state)?;
    state.running = running;
    Ok(state.snapshot())
}

#[tauri::command]
pub fn step_simulation(state: State<'_, Mutex<AppSimulation>>) -> Result<AppSnapshot, String> {
    let mut state = lock(&state)?;
    state.running = false;
    state
        .simulation
        .step(FIXED_STEP_SECONDS)
        .map_err(stringify)?;
    Ok(state.snapshot())
}

#[tauri::command]
pub fn set_visualization_speed(
    state: State<'_, Mutex<AppSimulation>>,
    steps_per_snapshot: u32,
) -> Result<AppSnapshot, String> {
    if steps_per_snapshot == 0 || steps_per_snapshot > MAX_STEPS_PER_SNAPSHOT {
        return Err(format!(
            "steps per snapshot must be between 1 and {MAX_STEPS_PER_SNAPSHOT}"
        ));
    }
    let mut state = lock(&state)?;
    state.steps_per_snapshot = steps_per_snapshot;
    Ok(state.snapshot())
}

#[tauri::command]
pub fn set_motor_command(
    state: State<'_, Mutex<AppSimulation>>,
    command: MotorCommand,
) -> Result<AppSnapshot, String> {
    let mut state = lock(&state)?;
    state
        .simulation
        .module_mut()
        .set_motor_command(command)
        .map_err(stringify)?;
    Ok(state.snapshot())
}

fn create_simulation(map: Map) -> Result<Simulation<Amr>, String> {
    let bounds = map.bounds();
    let config = AmrConfig {
        lidar_ray_count: VISUALIZATION_LIDAR_RAY_COUNT,
        lidar_maximum_range: VISUALIZATION_LIDAR_MAXIMUM_RANGE,
        ..AmrConfig::default()
    };
    let margin_x = config.length * 0.5 + 0.05;
    let margin_y = config.width * 0.5 + 0.05;
    let mut candidates = Vec::new();
    candidates.push(Pose2::new(bounds.width * 0.5, bounds.height * 0.5, 0.0));

    let mut y = margin_y;
    while y <= bounds.height - margin_y {
        let mut x = margin_x;
        while x <= bounds.width - margin_x {
            candidates.push(Pose2::new(x, y, 0.0));
            x += 0.5;
        }
        y += 0.5;
    }

    for pose in candidates {
        let robot = Amr::new(config, pose).map_err(stringify)?;
        if let Ok(simulation) = Simulation::new(map.clone(), robot) {
            return Ok(simulation);
        }
    }
    Err("the map has no collision-free AMR spawn position".into())
}

fn lock<'a>(
    state: &'a State<'_, Mutex<AppSimulation>>,
) -> Result<std::sync::MutexGuard<'a, AppSimulation>, String> {
    state
        .lock()
        .map_err(|_| "simulation state lock is poisoned".into())
}

fn stringify(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_a_robot_in_an_open_map() {
        let map = Map::new(Bounds::new(8.0, 6.0), Vec::new()).unwrap();
        let simulation = create_simulation(map).unwrap();
        assert_eq!(simulation.module().pose(), Pose2::new(4.0, 3.0, 0.0));
        assert_eq!(
            simulation.module().lidar().len(),
            VISUALIZATION_LIDAR_RAY_COUNT
        );
        assert_eq!(
            simulation.module().config().lidar_maximum_range,
            VISUALIZATION_LIDAR_MAXIMUM_RANGE
        );
    }

    #[test]
    fn snapshot_contains_controller_and_render_state() {
        let map = Map::new(Bounds::new(8.0, 6.0), Vec::new()).unwrap();
        let simulation = create_simulation(map.clone()).unwrap();
        let app = AppSimulation {
            simulation,
            reset_map: map,
            running: false,
            steps_per_snapshot: 4,
        };
        let snapshot = app.snapshot();
        assert_eq!(snapshot.world.bodies.len(), 1);
        assert_eq!(snapshot.robot.lidar.len(), VISUALIZATION_LIDAR_RAY_COUNT);
        assert_eq!(
            snapshot.robot.lidar_maximum_range,
            VISUALIZATION_LIDAR_MAXIMUM_RANGE
        );
        assert_eq!(snapshot.robot.motor_command, MotorCommand::STOPPED);
    }
}
