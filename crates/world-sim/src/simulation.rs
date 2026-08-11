use crate::geometry::{
    angle_delta, oriented_rectangle_corners, rectangle_aabb, rectangle_intersects_capsule,
};
use crate::{Map, Pose2, RayHit, Vec2, Wall};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const CONTACT_SEARCH_ITERATIONS: usize = 14;

#[derive(Debug, Error)]
pub enum SimulationError {
    #[error("invalid simulation value: {0}")]
    Invalid(String),
    #[error("unknown body id {0}")]
    UnknownBody(usize),
    #[error("body intersects a wall or map boundary")]
    BodyCollision,
    #[error("simulation module failed to initialize: {0}")]
    ModuleInitialization(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BodyId(usize);

impl BodyId {
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RectangleBody {
    pub pose: Pose2,
    pub length: f32,
    pub width: f32,
}

impl RectangleBody {
    pub const fn new(pose: Pose2, length: f32, width: f32) -> Self {
        Self {
            pose,
            length,
            width,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MoveResult {
    pub pose: Pose2,
    pub collided: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulationSnapshot {
    pub bounds: crate::Bounds,
    pub walls: Vec<Wall>,
    pub bodies: Vec<BodySnapshot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BodySnapshot {
    pub id: BodyId,
    pub pose: Pose2,
    pub length: f32,
    pub width: f32,
}

pub trait SimulationModule {
    fn initialize(&mut self, context: &mut InitContext<'_>) -> Result<(), SimulationError>;
    fn step(&mut self, context: &mut StepContext<'_>, dt_seconds: f32);
}

pub struct Simulation<M> {
    world: World,
    module: M,
    elapsed_seconds: f64,
    step_count: u64,
}

impl<M: SimulationModule> Simulation<M> {
    pub fn new(map: Map, mut module: M) -> Result<Self, SimulationError> {
        let mut world = World::new(map);
        module.initialize(&mut InitContext { world: &mut world })?;
        Ok(Self {
            world,
            module,
            elapsed_seconds: 0.0,
            step_count: 0,
        })
    }

    pub fn step(&mut self, dt_seconds: f32) -> Result<(), SimulationError> {
        validate_positive("step duration", dt_seconds)?;
        self.module.step(
            &mut StepContext {
                world: &mut self.world,
            },
            dt_seconds,
        );
        self.elapsed_seconds += f64::from(dt_seconds);
        self.step_count = self.step_count.saturating_add(1);
        Ok(())
    }

    pub fn run_steps(&mut self, step_count: usize, dt_seconds: f32) -> Result<(), SimulationError> {
        validate_positive("step duration", dt_seconds)?;
        for _ in 0..step_count {
            self.module.step(
                &mut StepContext {
                    world: &mut self.world,
                },
                dt_seconds,
            );
        }
        self.elapsed_seconds += f64::from(dt_seconds) * step_count as f64;
        self.step_count = self.step_count.saturating_add(step_count as u64);
        Ok(())
    }

    pub fn snapshot(&self) -> SimulationSnapshot {
        self.world.snapshot()
    }

    pub fn module(&self) -> &M {
        &self.module
    }

    pub fn module_mut(&mut self) -> &mut M {
        &mut self.module
    }

    pub fn map(&self) -> &Map {
        &self.world.map
    }

    pub fn elapsed_seconds(&self) -> f64 {
        self.elapsed_seconds
    }

    pub fn step_count(&self) -> u64 {
        self.step_count
    }
}

pub struct InitContext<'a> {
    world: &'a mut World,
}

impl InitContext<'_> {
    pub fn register_rectangle(&mut self, body: RectangleBody) -> Result<BodyId, SimulationError> {
        self.world.register_rectangle(body)
    }

    pub fn body_pose(&self, id: BodyId) -> Result<Pose2, SimulationError> {
        self.world.body_pose(id)
    }

    pub fn raycast(&self, origin: Vec2, direction: Vec2, maximum: f32) -> Option<RayHit> {
        self.world.map.raycast(origin, direction, maximum)
    }
}

pub struct StepContext<'a> {
    world: &'a mut World,
}

impl StepContext<'_> {
    pub fn body_pose(&self, id: BodyId) -> Result<Pose2, SimulationError> {
        self.world.body_pose(id)
    }

    pub fn try_move(
        &mut self,
        id: BodyId,
        desired_pose: Pose2,
    ) -> Result<MoveResult, SimulationError> {
        self.world.try_move(id, desired_pose)
    }

    pub fn raycast(&self, origin: Vec2, direction: Vec2, maximum: f32) -> Option<RayHit> {
        self.world.map.raycast(origin, direction, maximum)
    }
}

struct World {
    map: Map,
    bodies: Vec<RectangleBody>,
}

impl World {
    fn new(map: Map) -> Self {
        Self {
            map,
            bodies: Vec::new(),
        }
    }

    fn register_rectangle(&mut self, body: RectangleBody) -> Result<BodyId, SimulationError> {
        validate_body(body)?;
        if self.pose_collides(body, body.pose) {
            return Err(SimulationError::BodyCollision);
        }
        let id = BodyId(self.bodies.len());
        self.bodies.push(body);
        Ok(id)
    }

    fn body_pose(&self, id: BodyId) -> Result<Pose2, SimulationError> {
        self.bodies
            .get(id.0)
            .map(|body| body.pose)
            .ok_or(SimulationError::UnknownBody(id.0))
    }

    fn try_move(&mut self, id: BodyId, desired_pose: Pose2) -> Result<MoveResult, SimulationError> {
        if !desired_pose.is_finite() {
            return Err(SimulationError::Invalid(
                "requested pose must be finite".into(),
            ));
        }
        let body = *self
            .bodies
            .get(id.0)
            .ok_or(SimulationError::UnknownBody(id.0))?;
        let current = body.pose;
        let translation = current.position().distance(desired_pose.position());
        let corner_radius = (body.length * body.length + body.width * body.width).sqrt() * 0.5;
        let rotation_travel =
            angle_delta(current.orientation, desired_pose.orientation).abs() * corner_radius;
        let maximum_increment = self
            .map
            .minimum_feature_size()
            .min(body.length.min(body.width))
            .max(0.001)
            * 0.4;
        let samples = ((translation + rotation_travel) / maximum_increment)
            .ceil()
            .max(1.0) as usize;
        let mut last_safe_amount = 0.0;

        for sample in 1..=samples {
            let amount = sample as f32 / samples as f32;
            let candidate = current.interpolate(desired_pose, amount);
            if self.pose_collides(body, candidate) {
                let mut lower = last_safe_amount;
                let mut upper = amount;
                for _ in 0..CONTACT_SEARCH_ITERATIONS {
                    let middle = (lower + upper) * 0.5;
                    if self.pose_collides(body, current.interpolate(desired_pose, middle)) {
                        upper = middle;
                    } else {
                        lower = middle;
                    }
                }
                let accepted = current.interpolate(desired_pose, lower);
                self.bodies[id.0].pose = accepted;
                return Ok(MoveResult {
                    pose: accepted,
                    collided: true,
                });
            }
            last_safe_amount = amount;
        }

        let accepted = Pose2::new(
            desired_pose.x,
            desired_pose.y,
            crate::normalize_angle(desired_pose.orientation),
        );
        self.bodies[id.0].pose = accepted;
        Ok(MoveResult {
            pose: accepted,
            collided: false,
        })
    }

    fn pose_collides(&self, body: RectangleBody, pose: Pose2) -> bool {
        let corners = oriented_rectangle_corners(pose, body.length, body.width);
        if corners
            .iter()
            .any(|corner| !self.map.bounds().contains(*corner))
        {
            return true;
        }

        let area = rectangle_aabb(pose, body.length, body.width);
        let mut collision = false;
        self.map.visit_walls_in_aabb(area, |wall_index| {
            let wall = self.map.walls()[wall_index];
            if rectangle_intersects_capsule(
                pose,
                body.length,
                body.width,
                wall.start,
                wall.end,
                wall.thickness * 0.5,
            ) {
                collision = true;
                false
            } else {
                true
            }
        });
        collision
    }

    fn snapshot(&self) -> SimulationSnapshot {
        SimulationSnapshot {
            bounds: self.map.bounds(),
            walls: self.map.walls().to_vec(),
            bodies: self
                .bodies
                .iter()
                .enumerate()
                .map(|(index, body)| BodySnapshot {
                    id: BodyId(index),
                    pose: body.pose,
                    length: body.length,
                    width: body.width,
                })
                .collect(),
        }
    }
}

fn validate_body(body: RectangleBody) -> Result<(), SimulationError> {
    if !body.pose.is_finite() {
        return Err(SimulationError::Invalid("body pose must be finite".into()));
    }
    validate_positive("body length", body.length)?;
    validate_positive("body width", body.width)
}

fn validate_positive(name: &str, value: f32) -> Result<(), SimulationError> {
    if !value.is_finite() || value <= 0.0 {
        Err(SimulationError::Invalid(format!(
            "{name} must be finite and positive"
        )))
    } else {
        Ok(())
    }
}
