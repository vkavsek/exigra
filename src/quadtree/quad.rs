//! Contains a trait that indicates how to interpret a type as its axis-aligned bounding box [`Rect`].

use bevy::math::{Rect, Vec2, Vec3};

/// A trait that indicates how to interpret a type as its AABB [`Rect`].
pub trait Quad {
    fn as_quad(&self) -> Rect;
}

// `AsRect` implementations

impl Quad for Rect {
    #[inline]
    fn as_quad(&self) -> Rect {
        *self
    }
}

impl Quad for Vec2 {
    #[inline]
    fn as_quad(&self) -> Rect {
        Rect::from_center_size(*self, Vec2::ZERO)
    }
}

impl Quad for Vec3 {
    #[inline]
    fn as_quad(&self) -> Rect {
        Rect::from_center_size(self.truncate(), Vec2::ZERO)
    }
}
