use bevy::math::{Rect, Vec2};

pub enum Shape {
    Quad(Rect),
    Circle { center: Vec2, radius: f32 },
    Capsule,
}
