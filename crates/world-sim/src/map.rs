use crate::geometry::{ray_capsule_distance, Aabb};
use crate::Vec2;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use thiserror::Error;

const MAP_FORMAT_VERSION: u32 = 1;
const MIN_CELL_SIZE: f32 = 0.25;
const MAX_CELL_SIZE: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Bounds {
    pub width: f32,
    pub height: f32,
}

impl Bounds {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn contains(self, point: Vec2) -> bool {
        point.x >= 0.0 && point.y >= 0.0 && point.x <= self.width && point.y <= self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Wall {
    pub start: Vec2,
    pub end: Vec2,
    pub thickness: f32,
}

impl Wall {
    pub const fn new(start: Vec2, end: Vec2, thickness: f32) -> Self {
        Self {
            start,
            end,
            thickness,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MapConfig {
    pub bounds: Bounds,
    pub wall_count: usize,
    pub wall_thickness: f32,
    pub minimum_wall_length: f32,
    pub maximum_wall_length: f32,
}

impl Default for MapConfig {
    fn default() -> Self {
        Self {
            bounds: Bounds::new(20.0, 12.0),
            wall_count: 14,
            wall_thickness: 0.15,
            minimum_wall_length: 1.0,
            maximum_wall_length: 4.0,
        }
    }
}

#[derive(Debug, Error)]
pub enum MapError {
    #[error("invalid map: {0}")]
    Invalid(String),
    #[error("map I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("map JSON failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported map format version {0}")]
    UnsupportedVersion(u32),
    #[error("could not place all requested walls within the map")]
    GenerationFailed,
}

#[derive(Debug, Clone)]
pub struct Map {
    bounds: Bounds,
    walls: Vec<Wall>,
    index: GridIndex,
    minimum_wall_thickness: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MapFile {
    version: u32,
    bounds: Bounds,
    walls: Vec<Wall>,
}

impl Map {
    pub fn new(bounds: Bounds, walls: Vec<Wall>) -> Result<Self, MapError> {
        validate_bounds(bounds)?;
        for (index, wall) in walls.iter().enumerate() {
            validate_wall(bounds, *wall)
                .map_err(|message| MapError::Invalid(format!("wall {index}: {message}")))?;
        }

        let minimum_wall_thickness = walls
            .iter()
            .map(|wall| wall.thickness)
            .reduce(f32::min)
            .unwrap_or(0.1);
        let index = GridIndex::build(bounds, &walls);
        Ok(Self {
            bounds,
            walls,
            index,
            minimum_wall_thickness,
        })
    }

    pub fn generate<R: Rng + ?Sized>(config: MapConfig, rng: &mut R) -> Result<Self, MapError> {
        validate_config(config)?;
        let mut walls = Vec::with_capacity(config.wall_count);
        let maximum_attempts = config.wall_count.saturating_mul(100).max(100);

        for _ in 0..maximum_attempts {
            if walls.len() == config.wall_count {
                break;
            }
            let length = rng.gen_range(config.minimum_wall_length..=config.maximum_wall_length);
            let orientation = rng.gen_range(0.0..std::f32::consts::TAU);
            let half = length * 0.5;
            let radius = config.wall_thickness * 0.5;
            let half_x = orientation.cos().abs() * half + radius;
            let half_y = orientation.sin().abs() * half + radius;
            if half_x * 2.0 > config.bounds.width || half_y * 2.0 > config.bounds.height {
                continue;
            }
            let center = Vec2::new(
                rng.gen_range(half_x..=config.bounds.width - half_x),
                rng.gen_range(half_y..=config.bounds.height - half_y),
            );
            let offset = Vec2::new(orientation.cos() * half, orientation.sin() * half);
            walls.push(Wall::new(
                center - offset,
                center + offset,
                config.wall_thickness,
            ));
        }

        if walls.len() != config.wall_count {
            return Err(MapError::GenerationFailed);
        }
        Self::new(config.bounds, walls)
    }

    pub fn load<R: Read>(reader: R) -> Result<Self, MapError> {
        let file: MapFile = serde_json::from_reader(reader)?;
        if file.version != MAP_FORMAT_VERSION {
            return Err(MapError::UnsupportedVersion(file.version));
        }
        Self::new(file.bounds, file.walls)
    }

    pub fn save<W: Write>(&self, writer: W) -> Result<(), MapError> {
        #[derive(Serialize)]
        struct MapFileRef<'a> {
            version: u32,
            bounds: Bounds,
            walls: &'a [Wall],
        }
        serde_json::to_writer(
            writer,
            &MapFileRef {
                version: MAP_FORMAT_VERSION,
                bounds: self.bounds,
                walls: &self.walls,
            },
        )?;
        Ok(())
    }

    pub fn bounds(&self) -> Bounds {
        self.bounds
    }

    pub fn walls(&self) -> &[Wall] {
        &self.walls
    }

    pub fn raycast(&self, origin: Vec2, direction: Vec2, maximum: f32) -> Option<RayHit> {
        if !origin.is_finite()
            || !self.bounds.contains(origin)
            || !maximum.is_finite()
            || maximum < 0.0
        {
            return None;
        }
        let direction = direction.normalized()?;
        let boundary_distance = ray_boundary_distance(origin, direction, self.bounds);
        let traversal_limit = maximum.min(boundary_distance.unwrap_or(maximum));
        let mut closest = boundary_distance
            .filter(|distance| *distance <= maximum)
            .map(|distance| (distance, None));

        self.index
            .visit_ray(origin, direction, traversal_limit, |wall_index| {
                let wall = self.walls[wall_index];
                let current_limit = closest.map(|hit| hit.0).unwrap_or(maximum);
                if let Some(distance) = ray_capsule_distance(
                    origin,
                    direction,
                    wall.start,
                    wall.end,
                    wall.thickness * 0.5,
                    current_limit,
                ) {
                    closest = Some((distance, Some(wall_index)));
                }
                closest.map(|hit| hit.0).unwrap_or(traversal_limit)
            });

        closest.map(|(distance, wall_index)| RayHit {
            distance,
            point: origin + direction * distance,
            wall_index,
        })
    }

    pub(crate) fn minimum_feature_size(&self) -> f32 {
        self.minimum_wall_thickness
    }

    pub(crate) fn visit_walls_in_aabb<F>(&self, area: Aabb, visitor: F)
    where
        F: FnMut(usize) -> bool,
    {
        self.index.visit_aabb(area, visitor);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RayHit {
    pub distance: f32,
    pub point: Vec2,
    pub wall_index: Option<usize>,
}

fn validate_bounds(bounds: Bounds) -> Result<(), MapError> {
    if !bounds.width.is_finite() || !bounds.height.is_finite() {
        return Err(MapError::Invalid("map bounds must be finite".into()));
    }
    if bounds.width <= 0.0 || bounds.height <= 0.0 {
        return Err(MapError::Invalid("map bounds must be positive".into()));
    }
    Ok(())
}

fn validate_wall(bounds: Bounds, wall: Wall) -> Result<(), String> {
    if !wall.start.is_finite() || !wall.end.is_finite() || !wall.thickness.is_finite() {
        return Err("values must be finite".into());
    }
    if wall.thickness <= 0.0 {
        return Err("thickness must be positive".into());
    }
    if wall.start.distance(wall.end) <= f32::EPSILON {
        return Err("endpoints must be different".into());
    }
    let radius = wall.thickness * 0.5;
    let area = Aabb::from_points(wall.start, wall.end, radius);
    if area.min.x < 0.0
        || area.min.y < 0.0
        || area.max.x > bounds.width
        || area.max.y > bounds.height
    {
        return Err("thick wall must remain inside map bounds".into());
    }
    Ok(())
}

fn validate_config(config: MapConfig) -> Result<(), MapError> {
    validate_bounds(config.bounds)?;
    for (name, value) in [
        ("wall thickness", config.wall_thickness),
        ("minimum wall length", config.minimum_wall_length),
        ("maximum wall length", config.maximum_wall_length),
    ] {
        if !value.is_finite() || value <= 0.0 {
            return Err(MapError::Invalid(format!(
                "{name} must be finite and positive"
            )));
        }
    }
    if config.minimum_wall_length > config.maximum_wall_length {
        return Err(MapError::Invalid(
            "minimum wall length cannot exceed maximum wall length".into(),
        ));
    }
    Ok(())
}

fn ray_boundary_distance(origin: Vec2, direction: Vec2, bounds: Bounds) -> Option<f32> {
    let mut closest = f32::INFINITY;
    if direction.x > 0.0 {
        closest = closest.min((bounds.width - origin.x) / direction.x);
    } else if direction.x < 0.0 {
        closest = closest.min(-origin.x / direction.x);
    }
    if direction.y > 0.0 {
        closest = closest.min((bounds.height - origin.y) / direction.y);
    } else if direction.y < 0.0 {
        closest = closest.min(-origin.y / direction.y);
    }
    closest.is_finite().then_some(closest.max(0.0))
}

#[derive(Debug, Clone)]
struct GridIndex {
    bounds: Bounds,
    cell_size: f32,
    columns: usize,
    rows: usize,
    cells: Vec<Vec<usize>>,
}

impl GridIndex {
    fn build(bounds: Bounds, walls: &[Wall]) -> Self {
        let ideal = if walls.is_empty() {
            1.0
        } else {
            (bounds.width * bounds.height / (walls.len() as f32 * 4.0)).sqrt()
        };
        let cell_size = ideal.clamp(MIN_CELL_SIZE, MAX_CELL_SIZE);
        let columns = (bounds.width / cell_size).ceil().max(1.0) as usize;
        let rows = (bounds.height / cell_size).ceil().max(1.0) as usize;
        let mut index = Self {
            bounds,
            cell_size,
            columns,
            rows,
            cells: vec![Vec::new(); columns * rows],
        };

        for (wall_index, wall) in walls.iter().enumerate() {
            let area = Aabb::from_points(wall.start, wall.end, wall.thickness * 0.5);
            let (minimum_x, maximum_x, minimum_y, maximum_y) = index.cell_range(area);
            for y in minimum_y..=maximum_y {
                for x in minimum_x..=maximum_x {
                    index.cells[y * columns + x].push(wall_index);
                }
            }
        }
        index
    }

    fn visit_aabb<F>(&self, area: Aabb, mut visitor: F)
    where
        F: FnMut(usize) -> bool,
    {
        let (minimum_x, maximum_x, minimum_y, maximum_y) = self.cell_range(area);
        for y in minimum_y..=maximum_y {
            for x in minimum_x..=maximum_x {
                for &wall_index in &self.cells[y * self.columns + x] {
                    if !visitor(wall_index) {
                        return;
                    }
                }
            }
        }
    }

    fn visit_ray<F>(&self, origin: Vec2, direction: Vec2, maximum: f32, mut visitor: F)
    where
        F: FnMut(usize) -> f32,
    {
        let mut cell_x = self.x_index(origin.x) as isize;
        let mut cell_y = self.y_index(origin.y) as isize;
        let step_x = direction.x.signum() as isize;
        let step_y = direction.y.signum() as isize;
        let mut next_x = next_grid_distance(origin.x, direction.x, cell_x, self.cell_size);
        let mut next_y = next_grid_distance(origin.y, direction.y, cell_y, self.cell_size);
        let delta_x = if direction.x == 0.0 {
            f32::INFINITY
        } else {
            self.cell_size / direction.x.abs()
        };
        let delta_y = if direction.y == 0.0 {
            f32::INFINITY
        } else {
            self.cell_size / direction.y.abs()
        };
        let mut limit = maximum;

        loop {
            if cell_x < 0
                || cell_y < 0
                || cell_x >= self.columns as isize
                || cell_y >= self.rows as isize
            {
                break;
            }
            for &wall_index in &self.cells[cell_y as usize * self.columns + cell_x as usize] {
                limit = limit.min(visitor(wall_index));
            }
            let next = next_x.min(next_y);
            if next > limit || next > maximum {
                break;
            }
            if (next_x - next_y).abs() <= f32::EPSILON {
                cell_x += step_x;
                next_x += delta_x;
                cell_y += step_y;
                next_y += delta_y;
            } else if next_x < next_y {
                cell_x += step_x;
                next_x += delta_x;
            } else {
                cell_y += step_y;
                next_y += delta_y;
            }
        }
    }

    fn cell_range(&self, area: Aabb) -> (usize, usize, usize, usize) {
        (
            self.x_index(area.min.x),
            self.x_index(area.max.x),
            self.y_index(area.min.y),
            self.y_index(area.max.y),
        )
    }

    fn x_index(&self, x: f32) -> usize {
        ((x.clamp(0.0, self.bounds.width) / self.cell_size).floor() as usize).min(self.columns - 1)
    }

    fn y_index(&self, y: f32) -> usize {
        ((y.clamp(0.0, self.bounds.height) / self.cell_size).floor() as usize).min(self.rows - 1)
    }
}

fn next_grid_distance(origin: f32, direction: f32, cell: isize, cell_size: f32) -> f32 {
    if direction > 0.0 {
        ((cell + 1) as f32 * cell_size - origin) / direction
    } else if direction < 0.0 {
        (cell as f32 * cell_size - origin) / direction
    } else {
        f32::INFINITY
    }
}
