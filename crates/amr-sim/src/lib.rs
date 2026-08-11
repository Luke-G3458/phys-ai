//! Differential-drive AMR module for `world-sim`.

use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;
use thiserror::Error;
use world_sim::{
    normalize_angle, BodyId, InitContext, Pose2, RectangleBody, SimulationError, SimulationModule,
    StepContext, Vec2,
};

pub const LIDAR_RAY_COUNT: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AmrConfig {
    pub length: f32,
    pub width: f32,
    pub track_width: f32,
    pub maximum_wheel_speed: f32,
    pub lidar_maximum_range: f32,
}

impl Default for AmrConfig {
    fn default() -> Self {
        Self {
            length: 0.8,
            width: 0.55,
            track_width: 0.45,
            maximum_wheel_speed: 1.5,
            lidar_maximum_range: 15.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct MotorCommand {
    pub left: f32,
    pub right: f32,
}

impl MotorCommand {
    pub const STOPPED: Self = Self {
        left: 0.0,
        right: 0.0,
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct AmrObservation {
    pub pose: Pose2,
    pub collided: bool,
    pub lidar: [f32; LIDAR_RAY_COUNT],
}

#[derive(Debug, Error)]
pub enum AmrError {
    #[error("invalid AMR configuration: {0}")]
    InvalidConfig(String),
    #[error("motor commands must be finite")]
    NonFiniteMotorCommand,
}

pub struct Amr {
    config: AmrConfig,
    initial_pose: Pose2,
    body: Option<BodyId>,
    motor_command: MotorCommand,
    observation: AmrObservation,
}

impl Amr {
    pub fn new(config: AmrConfig, initial_pose: Pose2) -> Result<Self, AmrError> {
        validate_config(config)?;
        if !initial_pose.is_finite() {
            return Err(AmrError::InvalidConfig(
                "initial pose must contain finite values".into(),
            ));
        }
        Ok(Self {
            config,
            initial_pose,
            body: None,
            motor_command: MotorCommand::STOPPED,
            observation: AmrObservation {
                pose: initial_pose,
                collided: false,
                lidar: [config.lidar_maximum_range; LIDAR_RAY_COUNT],
            },
        })
    }

    pub fn set_motor_command(&mut self, command: MotorCommand) -> Result<(), AmrError> {
        if !command.left.is_finite() || !command.right.is_finite() {
            return Err(AmrError::NonFiniteMotorCommand);
        }
        self.motor_command = MotorCommand {
            left: command.left.clamp(-1.0, 1.0),
            right: command.right.clamp(-1.0, 1.0),
        };
        Ok(())
    }

    pub fn motor_command(&self) -> MotorCommand {
        self.motor_command
    }

    pub fn pose(&self) -> Pose2 {
        self.observation.pose
    }

    pub fn lidar(&self) -> &[f32; LIDAR_RAY_COUNT] {
        &self.observation.lidar
    }

    pub fn observation(&self) -> &AmrObservation {
        &self.observation
    }

    pub fn config(&self) -> AmrConfig {
        self.config
    }

    fn next_pose(&self, dt_seconds: f32) -> Pose2 {
        let left_velocity = self.motor_command.left * self.config.maximum_wheel_speed;
        let right_velocity = self.motor_command.right * self.config.maximum_wheel_speed;
        let linear_velocity = (left_velocity + right_velocity) * 0.5;
        let angular_velocity = (right_velocity - left_velocity) / self.config.track_width;
        let current = self.observation.pose;

        if angular_velocity.abs() <= 1.0e-6 {
            Pose2::new(
                current.x + linear_velocity * current.orientation.cos() * dt_seconds,
                current.y + linear_velocity * current.orientation.sin() * dt_seconds,
                current.orientation,
            )
        } else {
            let next_orientation = current.orientation + angular_velocity * dt_seconds;
            let radius = linear_velocity / angular_velocity;
            Pose2::new(
                current.x + radius * (next_orientation.sin() - current.orientation.sin()),
                current.y - radius * (next_orientation.cos() - current.orientation.cos()),
                normalize_angle(next_orientation),
            )
        }
    }

    fn scan_with_init_context(&mut self, context: &InitContext<'_>) {
        let pose = self.observation.pose;
        let maximum = self.config.lidar_maximum_range;
        for (index, reading) in self.observation.lidar.iter_mut().enumerate() {
            let angle = pose.orientation + index as f32 * TAU / LIDAR_RAY_COUNT as f32;
            let direction = Vec2::new(angle.cos(), angle.sin());
            *reading = context
                .raycast(pose.position(), direction, maximum)
                .map(|hit| hit.distance)
                .unwrap_or(maximum);
        }
    }

    fn scan_with_step_context(&mut self, context: &StepContext<'_>) {
        let pose = self.observation.pose;
        let maximum = self.config.lidar_maximum_range;
        for (index, reading) in self.observation.lidar.iter_mut().enumerate() {
            let angle = pose.orientation + index as f32 * TAU / LIDAR_RAY_COUNT as f32;
            let direction = Vec2::new(angle.cos(), angle.sin());
            *reading = context
                .raycast(pose.position(), direction, maximum)
                .map(|hit| hit.distance)
                .unwrap_or(maximum);
        }
    }
}

impl SimulationModule for Amr {
    fn initialize(&mut self, context: &mut InitContext<'_>) -> Result<(), SimulationError> {
        let body = context.register_rectangle(RectangleBody::new(
            self.initial_pose,
            self.config.length,
            self.config.width,
        ))?;
        self.body = Some(body);
        self.observation.pose = context.body_pose(body)?;
        self.observation.collided = false;
        self.scan_with_init_context(context);
        Ok(())
    }

    fn step(&mut self, context: &mut StepContext<'_>, dt_seconds: f32) {
        let body = self.body.expect("an AMR is initialized before stepping");
        let movement = context
            .try_move(body, self.next_pose(dt_seconds))
            .expect("an initialized AMR body remains registered");
        self.observation.pose = movement.pose;
        self.observation.collided = movement.collided;
        self.scan_with_step_context(context);
    }
}

fn validate_config(config: AmrConfig) -> Result<(), AmrError> {
    for (name, value) in [
        ("length", config.length),
        ("width", config.width),
        ("track width", config.track_width),
        ("maximum wheel speed", config.maximum_wheel_speed),
        ("lidar maximum range", config.lidar_maximum_range),
    ] {
        if !value.is_finite() || value <= 0.0 {
            return Err(AmrError::InvalidConfig(format!(
                "{name} must be finite and positive"
            )));
        }
    }
    if config.track_width > config.width {
        return Err(AmrError::InvalidConfig(
            "track width cannot exceed robot width".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use world_sim::{Bounds, Map, Simulation, Vec2, Wall};

    fn open_map() -> Map {
        Map::new(Bounds::new(10.0, 10.0), Vec::new()).unwrap()
    }

    #[test]
    fn motor_commands_are_clamped_and_non_finite_values_fail() {
        let mut robot = Amr::new(AmrConfig::default(), Pose2::new(5.0, 5.0, 0.0)).unwrap();
        robot
            .set_motor_command(MotorCommand {
                left: 2.0,
                right: -2.0,
            })
            .unwrap();
        assert_eq!(
            robot.motor_command(),
            MotorCommand {
                left: 1.0,
                right: -1.0
            }
        );
        assert!(robot
            .set_motor_command(MotorCommand {
                left: f32::NAN,
                right: 0.0,
            })
            .is_err());
    }

    #[test]
    fn equal_motor_commands_move_straight() {
        let robot = Amr::new(AmrConfig::default(), Pose2::new(5.0, 5.0, 0.0)).unwrap();
        let mut simulation = Simulation::new(open_map(), robot).unwrap();
        simulation
            .module_mut()
            .set_motor_command(MotorCommand {
                left: 0.5,
                right: 0.5,
            })
            .unwrap();
        simulation.step(1.0).unwrap();
        let pose = simulation.module().pose();
        assert!((pose.x - 5.75).abs() < 0.001);
        assert!((pose.y - 5.0).abs() < 0.001);
        assert!(pose.orientation.abs() < 0.001);
    }

    #[test]
    fn opposite_motor_commands_rotate_in_place() {
        let robot = Amr::new(AmrConfig::default(), Pose2::new(5.0, 5.0, 0.0)).unwrap();
        let mut simulation = Simulation::new(open_map(), robot).unwrap();
        simulation
            .module_mut()
            .set_motor_command(MotorCommand {
                left: -0.25,
                right: 0.25,
            })
            .unwrap();
        simulation.step(0.2).unwrap();
        let pose = simulation.module().pose();
        assert!((pose.x - 5.0).abs() < 0.001);
        assert!((pose.y - 5.0).abs() < 0.001);
        assert!(pose.orientation > 0.0);
    }

    #[test]
    fn lidar_ray_zero_points_forward_and_reports_surface_distance() {
        let map = Map::new(
            Bounds::new(10.0, 10.0),
            vec![Wall::new(Vec2::new(7.0, 0.1), Vec2::new(7.0, 9.9), 0.2)],
        )
        .unwrap();
        let robot = Amr::new(AmrConfig::default(), Pose2::new(5.0, 5.0, 0.0)).unwrap();
        let simulation = Simulation::new(map, robot).unwrap();
        assert!((simulation.module().lidar()[0] - 1.9).abs() < 0.001);
        assert!((simulation.module().lidar()[LIDAR_RAY_COUNT / 4] - 5.0).abs() < 0.001);
    }

    #[test]
    fn robot_stops_before_crossing_a_wall() {
        let map = Map::new(
            Bounds::new(10.0, 10.0),
            vec![Wall::new(Vec2::new(7.0, 0.1), Vec2::new(7.0, 9.9), 0.2)],
        )
        .unwrap();
        let robot = Amr::new(AmrConfig::default(), Pose2::new(5.0, 5.0, 0.0)).unwrap();
        let mut simulation = Simulation::new(map, robot).unwrap();
        simulation
            .module_mut()
            .set_motor_command(MotorCommand {
                left: 1.0,
                right: 1.0,
            })
            .unwrap();
        simulation.step(2.0).unwrap();
        assert!(simulation.module().observation().collided);
        assert!(simulation.module().pose().x < 6.51);
    }
}
