//! Fast, controller-agnostic 2D world simulation.

mod geometry;
mod map;
mod simulation;

pub use geometry::{normalize_angle, Pose2, Vec2};
pub use map::{Bounds, Map, MapConfig, MapError, RayHit, Wall};
pub use simulation::{
    BodyId, BodySnapshot, InitContext, MoveResult, RectangleBody, Simulation, SimulationError,
    SimulationModule, SimulationSnapshot, StepContext,
};

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};
    use std::io::Cursor;

    fn test_map() -> Map {
        Map::new(
            Bounds::new(10.0, 8.0),
            vec![Wall::new(Vec2::new(6.0, 1.0), Vec2::new(6.0, 7.0), 0.2)],
        )
        .unwrap()
    }

    #[test]
    fn map_json_round_trip_preserves_geometry() {
        let map = test_map();
        let mut json = Vec::new();
        map.save(&mut json).unwrap();
        let loaded = Map::load(Cursor::new(json)).unwrap();
        assert_eq!(loaded.bounds(), map.bounds());
        assert_eq!(loaded.walls(), map.walls());
    }

    #[test]
    fn generated_map_obeys_requested_shape() {
        let mut rng = StdRng::seed_from_u64(42);
        let config = MapConfig {
            wall_count: 20,
            ..MapConfig::default()
        };
        let map = Map::generate(config, &mut rng).unwrap();
        assert_eq!(map.bounds(), config.bounds);
        assert_eq!(map.walls().len(), 20);
    }

    #[test]
    fn raycast_returns_wall_surface_before_boundary() {
        let hit = test_map()
            .raycast(Vec2::new(2.0, 4.0), Vec2::new(1.0, 0.0), 20.0)
            .unwrap();
        assert!((hit.distance - 3.9).abs() < 0.001);
        assert_eq!(hit.wall_index, Some(0));
    }

    #[test]
    fn raycast_returns_map_boundary_when_no_wall_is_hit() {
        let hit = test_map()
            .raycast(Vec2::new(2.0, 4.0), Vec2::new(-1.0, 0.0), 20.0)
            .unwrap();
        assert!((hit.distance - 2.0).abs() < 0.001);
        assert_eq!(hit.wall_index, None);
    }

    struct MovingModule {
        body: Option<BodyId>,
        initial_pose: Pose2,
    }

    impl SimulationModule for MovingModule {
        fn initialize(&mut self, context: &mut InitContext<'_>) -> Result<(), SimulationError> {
            self.body = Some(context.register_rectangle(RectangleBody::new(
                self.initial_pose,
                1.0,
                0.6,
            ))?);
            Ok(())
        }

        fn step(&mut self, context: &mut StepContext<'_>, dt_seconds: f32) {
            let body = self.body.unwrap();
            let pose = context.body_pose(body).unwrap();
            context
                .try_move(
                    body,
                    Pose2::new(pose.x + 10.0 * dt_seconds, pose.y, pose.orientation),
                )
                .unwrap();
        }
    }

    #[test]
    fn generic_module_moves_but_cannot_cross_a_wall() {
        let mut simulation = Simulation::new(
            test_map(),
            MovingModule {
                body: None,
                initial_pose: Pose2::new(2.0, 4.0, 0.0),
            },
        )
        .unwrap();
        simulation.run_steps(10, 0.1).unwrap();
        let pose = simulation.snapshot().bodies[0].pose;
        assert!(pose.x > 5.3 && pose.x < 5.41, "unexpected x: {}", pose.x);
        assert_eq!(simulation.step_count(), 10);
    }

    #[test]
    fn registration_rejects_a_body_outside_the_map() {
        let result = Simulation::new(
            test_map(),
            MovingModule {
                body: None,
                initial_pose: Pose2::new(-1.0, 4.0, 0.0),
            },
        );
        assert!(matches!(result, Err(SimulationError::BodyCollision)));
    }
}
