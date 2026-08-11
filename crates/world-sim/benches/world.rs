use rand::{rngs::StdRng, SeedableRng};
use std::hint::black_box;
use std::time::Instant;
use world_sim::{Bounds, Map, MapConfig, Vec2};

fn main() {
    let mut rng = StdRng::seed_from_u64(7);
    let map = Map::generate(
        MapConfig {
            bounds: Bounds::new(50.0, 50.0),
            wall_count: 250,
            wall_thickness: 0.12,
            minimum_wall_length: 0.5,
            maximum_wall_length: 5.0,
        },
        &mut rng,
    )
    .expect("benchmark map should generate");

    let started = Instant::now();
    for index in 0..100_000 {
        let angle = index as f32 * 0.001;
        black_box(map.raycast(
            Vec2::new(25.0, 25.0),
            Vec2::new(angle.cos(), angle.sin()),
            30.0,
        ));
    }
    println!("100,000 indexed raycasts: {:?}", started.elapsed());

    let mut bytes = Vec::new();
    map.save(&mut bytes).expect("benchmark map should save");
    let started = Instant::now();
    for _ in 0..1_000 {
        black_box(Map::load(bytes.as_slice()).expect("benchmark map should load"));
    }
    println!("1,000 JSON map loads: {:?}", started.elapsed());
}
