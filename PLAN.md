# Physical AI

> A learning project for building and training simulated autonomous mobile robots (AMRs).

The project consists of three Rust crates and a Tauri visualization:

- `world-sim`: map generation, persistence, geometry, and simulation runtime.
- `amr-sim`: a differential-drive robot that runs as a `world-sim` module.
- Robot controller model: a later, user-led project using `rete`.
- Tauri app: a thin UI over the Rust simulation crates.

The current goal is the simulation environment. Controller training, rewards, goals, and episodes are intentionally deferred.

## Shared conventions

- The simulation is 2D. Distances and dimensions use meters, angles use radians, and time uses seconds.
- Runtime numeric data uses `f32` unless measurement shows that greater precision is necessary.
- A pose is `(x, y, orientation)`. An object's origin is its geometric center; zero orientation points along the positive x-axis.
- The simulation loop never sleeps or waits for real time. Headless runs execute as quickly as possible; visualization pacing is separate.
- Hot paths should reuse memory, avoid serialization and IPC, and be benchmarked with representative maps before optimizing further.

## 1. World simulator

### Responsibilities and data

- Represent a finite rectangular map and its walls. The map boundary is impassable.
- A wall is a line segment between two 2D points with a positive thickness. There are no other static obstacle types initially.
- Generate random maps from bounds and wall-generation parameters.
- Save and load maps only; runtime objects and simulation state are not persisted.
- Run an external simulation module without depending on its crate.
- Provide fast collision and ray-cast queries. A spatial index is built when a map is created or loaded and remains an internal implementation detail.

Maps use a versioned Serde JSON format containing map dimensions and walls. JSON is the initial choice because it is portable and inspectable; map loading and representative large files must be benchmarked before it is retained. The public persistence API must allow a future binary format without changing simulation APIs.

### Public API and behavior

The exact Rust names may evolve, but the public boundary must provide these operations:

```rust
Map::new(bounds, walls) -> Result<Map, MapError>
Map::generate(config, rng) -> Result<Map, MapError>
Map::load(reader) -> Result<Map, MapError>
map.save(writer) -> Result<(), MapError>

Simulation::new(map, module) -> Result<Simulation<M>, SimulationError>
simulation.step(dt_seconds) -> Result<(), SimulationError>
simulation.run_steps(step_count, dt_seconds) -> Result<(), SimulationError>
simulation.snapshot() -> SimulationSnapshot
simulation.module() / simulation.module_mut()
```

- `Map` rejects non-finite values, invalid bounds, zero/negative wall thickness, and walls outside the map.
- Generation config includes map size, wall count, thickness, and length range. Generated walls remain inside the bounds; connectivity guarantees are out of scope.
- Saving and loading round-trip the same map geometry. Files carry a schema version and invalid or unsupported data returns an error.
- `step` advances by the caller-provided duration. A run normally uses a constant step size, but reproducibility is not an initial requirement.
- `run_steps` is the high-throughput, headless path and performs no rendering, IPC, logging, or real-time pacing.
- `snapshot` returns only lightweight render data and is not required on every step.

The world crate defines the module interface:

```rust
trait SimulationModule {
    fn initialize(&mut self, context: &mut InitContext) -> Result<(), SimulationError>;
    fn step(&mut self, context: &mut StepContext, dt_seconds: f32);
}
```

`Simulation<M>` owns both the world and one generic module. Static dispatch keeps the hot loop small, while any crate can participate by implementing `SimulationModule`. The contexts allow a module to register a body, read its pose, ray-cast against walls, and request movement without exposing unrestricted world mutation.

A movement request returns the accepted pose and collision information. Motion is swept from the current pose toward the requested pose, stops at the first collision, and never permits wall penetration or escape from the map. Body-to-body collision is deferred.

## 2. AMR robot simulator

### Model

- One rectangular robot with configurable length, width, track width, and maximum wheel speed.
- The robot pose is its center `(x, y)` and orientation.
- Differential drive is controlled by independent left and right motor commands in the range `[-1.0, 1.0]`. Negative values reverse that side; the robot converts each value to meters per second using its configured maximum wheel speed.
- Each step converts wheel velocities to linear and angular motion, then asks the world to apply the resulting pose. Collision response comes entirely from `world-sim`.
- A centered, ideal 360-degree lidar uses a configurable number of evenly spaced rays. Ray `0` points forward and indices progress counterclockwise. Each reading is a distance in meters; no hit returns the configured maximum range.

The default remains 64 rays, giving 5.625-degree spacing, while individual AMRs can reduce the count and range for simpler controller observation contracts. The visualization uses 8 rays with a 3-meter maximum range. Noise is deferred.

### Public API and behavior

```rust
Amr::new(config, initial_pose) -> Result<Amr, AmrError>
amr.set_motor_command(MotorCommand { left, right }) -> Result<(), AmrError>
amr.motor_command() -> MotorCommand
amr.pose() -> Pose2
amr.lidar() -> &[f32]
amr.observation() -> &AmrObservation

impl SimulationModule for Amr
```

- Initialization registers the rectangular body and fails if its starting pose is invalid or colliding.
- Finite motor values outside `[-1.0, 1.0]` are clamped; non-finite values return an error. Setting a command does not advance time.
- `AmrObservation` contains the current pose, collision status, and configurable lidar distances. It is valid after initialization and refreshed after each step, with storage reused between steps.
- Each step applies the current motor command, resolves movement, and then scans from the accepted pose. Only the world's `step` and `run_steps` advance time.
- Manual tests and future controllers use the same boundary: read `observation` or `lidar`, produce a `MotorCommand`, call `set_motor_command`, and advance the world. A future neural-network controller may perform this loop and handle model loading without adding a `rete` dependency to `amr-sim`.

## 3. Tauri visualization

The Tauri backend owns a `Simulation<Amr>` instance and delegates all map, physics, and sensing work to the Rust crates. The frontend only sends commands and renders snapshots.

The first UI includes:

- Generate, load, save, and reset map.
- Run, pause, single-step, and visualization-speed controls.
- Two manual motor controls covering `[-1.0, 1.0]`, their current values, and an immediate stop action.
- A top-down canvas whose content is limited to the finite map and can be panned with the mouse. It renders the robot, its lidar rays and hit points, and collision state so the simulator can be checked without a controller model.

Simulation speed and presentation rate remain independent. Rust may execute many steps between snapshots, while the UI requests or receives snapshots at a capped visual rate. Training and other headless runs do not pass through Tauri.

## 4. Build order and completion criteria

1. Build `world-sim`: map validation, generation, JSON round trips, ray casting, swept wall collision, module runtime, tests, and benchmarks.
2. Build `amr-sim`: differential-drive motion, rectangular footprint, fixed lidar, module integration, tests, and benchmarks.
3. Connect the proven APIs to the minimal Tauri visualization.
4. Develop controller models and training workflows later without moving simulation behavior into the controller or UI.

Before visualization work begins, tests must cover map persistence, boundary/wall collision, ray-cast distances, drivetrain motion, lidar ordering, and running an AMR through the generic module API. Benchmarks must measure headless steps, collision queries, full lidar scans, and map loading on representative map sizes; performance decisions should be based on those results rather than an arbitrary real-time target.
