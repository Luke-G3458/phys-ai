use serde::{Deserialize, Serialize};
use std::f32::consts::{PI, TAU};
use std::ops::{Add, Mul, Sub};

const EPSILON: f32 = 1.0e-6;

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn normalized(self) -> Option<Self> {
        let length = self.length();
        (length > EPSILON && length.is_finite()).then(|| self * (1.0 / length))
    }

    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn distance(self, other: Self) -> f32 {
        (self - other).length()
    }

    pub fn lerp(self, other: Self, amount: f32) -> Self {
        self + (other - self) * amount
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Pose2 {
    pub x: f32,
    pub y: f32,
    pub orientation: f32,
}

impl Pose2 {
    pub const fn new(x: f32, y: f32, orientation: f32) -> Self {
        Self { x, y, orientation }
    }

    pub fn position(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub fn is_finite(self) -> bool {
        self.position().is_finite() && self.orientation.is_finite()
    }

    pub fn interpolate(self, other: Self, amount: f32) -> Self {
        let position = self.position().lerp(other.position(), amount);
        let orientation =
            self.orientation + angle_delta(self.orientation, other.orientation) * amount;
        Self::new(position.x, position.y, normalize_angle(orientation))
    }
}

pub fn normalize_angle(angle: f32) -> f32 {
    (angle + PI).rem_euclid(TAU) - PI
}

pub(crate) fn angle_delta(from: f32, to: f32) -> f32 {
    normalize_angle(to - from)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Aabb {
    pub min: Vec2,
    pub max: Vec2,
}

impl Aabb {
    pub fn from_points(first: Vec2, second: Vec2, padding: f32) -> Self {
        Self {
            min: Vec2::new(
                first.x.min(second.x) - padding,
                first.y.min(second.y) - padding,
            ),
            max: Vec2::new(
                first.x.max(second.x) + padding,
                first.y.max(second.y) + padding,
            ),
        }
    }
}

pub(crate) fn ray_capsule_distance(
    origin: Vec2,
    direction: Vec2,
    start: Vec2,
    end: Vec2,
    radius: f32,
    maximum: f32,
) -> Option<f32> {
    let axis = end - start;
    let length = axis.length();
    if length <= EPSILON {
        return ray_circle_distance(origin, direction, start, radius, maximum);
    }

    if point_segment_distance_squared(origin, start, end) <= radius * radius {
        return Some(0.0);
    }

    let tangent = axis * (1.0 / length);
    let normal = Vec2::new(-tangent.y, tangent.x);
    let relative = origin - start;
    let local_origin = Vec2::new(relative.dot(tangent), relative.dot(normal));
    let local_direction = Vec2::new(direction.dot(tangent), direction.dot(normal));
    let mut closest = maximum;
    let mut hit = false;

    if local_direction.y.abs() > EPSILON {
        for side in [-radius, radius] {
            let distance = (side - local_origin.y) / local_direction.y;
            let along = local_origin.x + local_direction.x * distance;
            if distance >= 0.0 && distance <= closest && along >= 0.0 && along <= length {
                closest = distance;
                hit = true;
            }
        }
    }

    for center in [start, end] {
        if let Some(distance) = ray_circle_distance(origin, direction, center, radius, closest) {
            closest = distance;
            hit = true;
        }
    }

    hit.then_some(closest)
}

fn ray_circle_distance(
    origin: Vec2,
    direction: Vec2,
    center: Vec2,
    radius: f32,
    maximum: f32,
) -> Option<f32> {
    let offset = origin - center;
    let half_b = offset.dot(direction);
    let c = offset.length_squared() - radius * radius;
    let discriminant = half_b * half_b - c;
    if discriminant < 0.0 {
        return None;
    }

    let root = discriminant.sqrt();
    let near = -half_b - root;
    let far = -half_b + root;
    let distance = if near >= 0.0 { near } else { far };
    (distance >= 0.0 && distance <= maximum).then_some(distance)
}

pub(crate) fn oriented_rectangle_corners(pose: Pose2, length: f32, width: f32) -> [Vec2; 4] {
    let half_length = length * 0.5;
    let half_width = width * 0.5;
    let cosine = pose.orientation.cos();
    let sine = pose.orientation.sin();

    [
        Vec2::new(half_length, half_width),
        Vec2::new(half_length, -half_width),
        Vec2::new(-half_length, -half_width),
        Vec2::new(-half_length, half_width),
    ]
    .map(|local| {
        Vec2::new(
            pose.x + local.x * cosine - local.y * sine,
            pose.y + local.x * sine + local.y * cosine,
        )
    })
}

pub(crate) fn rectangle_aabb(pose: Pose2, length: f32, width: f32) -> Aabb {
    let corners = oriented_rectangle_corners(pose, length, width);
    let mut minimum = corners[0];
    let mut maximum = corners[0];
    for corner in corners.iter().skip(1) {
        minimum.x = minimum.x.min(corner.x);
        minimum.y = minimum.y.min(corner.y);
        maximum.x = maximum.x.max(corner.x);
        maximum.y = maximum.y.max(corner.y);
    }
    Aabb {
        min: minimum,
        max: maximum,
    }
}

pub(crate) fn rectangle_intersects_capsule(
    pose: Pose2,
    length: f32,
    width: f32,
    segment_start: Vec2,
    segment_end: Vec2,
    radius: f32,
) -> bool {
    let cosine = pose.orientation.cos();
    let sine = pose.orientation.sin();
    let to_local = |point: Vec2| {
        let relative = point - pose.position();
        Vec2::new(
            relative.x * cosine + relative.y * sine,
            -relative.x * sine + relative.y * cosine,
        )
    };
    let start = to_local(segment_start);
    let end = to_local(segment_end);
    let half_length = length * 0.5;
    let half_width = width * 0.5;

    if segment_intersects_aabb(start, end, half_length, half_width) {
        return true;
    }

    let mut minimum_squared = point_aabb_distance_squared(start, half_length, half_width)
        .min(point_aabb_distance_squared(end, half_length, half_width));
    for corner in [
        Vec2::new(half_length, half_width),
        Vec2::new(half_length, -half_width),
        Vec2::new(-half_length, half_width),
        Vec2::new(-half_length, -half_width),
    ] {
        minimum_squared = minimum_squared.min(point_segment_distance_squared(corner, start, end));
    }

    minimum_squared <= radius * radius
}

fn segment_intersects_aabb(start: Vec2, end: Vec2, half_x: f32, half_y: f32) -> bool {
    let delta = end - start;
    let mut lower = 0.0_f32;
    let mut upper = 1.0_f32;
    for (origin, direction, minimum, maximum) in [
        (start.x, delta.x, -half_x, half_x),
        (start.y, delta.y, -half_y, half_y),
    ] {
        if direction.abs() <= EPSILON {
            if origin < minimum || origin > maximum {
                return false;
            }
            continue;
        }
        let inverse = 1.0 / direction;
        let mut near = (minimum - origin) * inverse;
        let mut far = (maximum - origin) * inverse;
        if near > far {
            std::mem::swap(&mut near, &mut far);
        }
        lower = lower.max(near);
        upper = upper.min(far);
        if lower > upper {
            return false;
        }
    }
    true
}

fn point_aabb_distance_squared(point: Vec2, half_x: f32, half_y: f32) -> f32 {
    let dx = (point.x.abs() - half_x).max(0.0);
    let dy = (point.y.abs() - half_y).max(0.0);
    dx * dx + dy * dy
}

fn point_segment_distance_squared(point: Vec2, start: Vec2, end: Vec2) -> f32 {
    let segment = end - start;
    let length_squared = segment.length_squared();
    if length_squared <= EPSILON {
        return (point - start).length_squared();
    }
    let amount = ((point - start).dot(segment) / length_squared).clamp(0.0, 1.0);
    (point - start.lerp(end, amount)).length_squared()
}
