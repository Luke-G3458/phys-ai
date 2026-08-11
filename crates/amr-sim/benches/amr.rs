use amr_sim::{Amr, AmrConfig, MotorCommand};
use rand::{rngs::StdRng, SeedableRng};
use std::hint::black_box;
use std::time::Instant;
use world_sim::{Bounds, Map, MapConfig, Pose2, Simulation};

fn main() {
    let mut rng = StdRng::seed_from_u64(11);
    let map = Map::generate(
        MapConfig {
            bounds: Bounds::new(50.0, 50.0),
            wall_count: 200,
            wall_thickness: 0.12,
            minimum_wall_length: 0.5,
            maximum_wall_length: 4.0,
        },
        &mut rng,
    )
    .expect("benchmark map should generate");
    let robot = Amr::new(AmrConfig::default(), Pose2::new(25.0, 25.0, 0.0))
        .expect("benchmark robot should be valid");
    let mut simulation = Simulation::new(map, robot).expect("benchmark robot should spawn");
    simulation
        .module_mut()
        .set_motor_command(MotorCommand {
            left: 0.6,
            right: 0.55,
        })
        .unwrap();

    let started = Instant::now();
    for _ in 0..100_000 {
        simulation.step(1.0 / 120.0).unwrap();
        black_box(simulation.module().observation());
    }
    println!("100,000 AMR steps with full lidar: {:?}", started.elapsed());
}
