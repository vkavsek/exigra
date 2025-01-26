//! Contains a trait that indicates how to interpret a type as its bounding box [`Rect`].

use bevy::math::{Rect, Vec2, Vec3};

/// A trait that indicates how to interpret a type as its bounding box [`Rect`].
pub trait AsRect {
    fn as_rect(&self) -> Rect;
}

// `AsRect` implementations

impl AsRect for Rect {
    #[inline]
    fn as_rect(&self) -> Rect {
        *self
    }
}

impl AsRect for Vec2 {
    #[inline]
    fn as_rect(&self) -> Rect {
        Rect::from_center_size(*self, Vec2::ZERO)
    }
}

impl AsRect for Vec3 {
    #[inline]
    fn as_rect(&self) -> Rect {
        Rect::from_center_size(self.truncate(), Vec2::ZERO)
    }
}
