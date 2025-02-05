//! Contains [`AsQuadVal`] helper trait, [`QuadVal`] & [`Shape`] structs, that are used to
//! determine in the quadtree where a value should be stored.
//! `QuadVal` has methods for collision detection.

use bevy::{
    math::{vec2, Rect, Vec2, Vec3},
    prelude::{Capsule2d, Circle, Rectangle},
};

pub trait AsQuadVal {
    /// How to convert from a given type to a [`QuadVal`].
    fn as_quad_val(&self) -> QuadVal;
}

impl AsQuadVal for QuadVal {
    #[inline]
    fn as_quad_val(&self) -> QuadVal {
        *self
    }
}

impl AsQuadVal for Rect {
    #[inline]
    fn as_quad_val(&self) -> QuadVal {
        QuadVal {
            pos: self.center(),
            shape: Shape::Quad(Rectangle::new(self.width(), self.height())),
        }
    }
}

impl AsQuadVal for Vec2 {
    #[inline]
    fn as_quad_val(&self) -> QuadVal {
        QuadVal {
            pos: *self,
            shape: Shape::Circle(Circle::new(0.0)),
        }
    }
}

impl AsQuadVal for Vec3 {
    #[inline]
    fn as_quad_val(&self) -> QuadVal {
        QuadVal {
            pos: self.truncate(),
            shape: Shape::Circle(Circle::new(0.0)),
        }
    }
}

// TODO: Add triangle

/// A [`Quadtree`] compatible value with handy collision detection methods.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuadVal {
    pub pos: Vec2,
    pub shape: Shape,
}

impl QuadVal {
    #[inline]
    pub fn new(pos: Vec2, shape: Shape) -> Self {
        Self { pos, shape }
    }

    /// Computes an axis-aligned bounding box from `self`
    #[inline]
    pub fn aabb(&self) -> Rect {
        match self.shape {
            Shape::Quad(rectangle) => Rect::from_center_half_size(self.pos, rectangle.half_size),
            Shape::Circle(circle) => {
                Rect::from_center_half_size(self.pos, Vec2::splat(circle.radius))
            }
            // width = 2 * radius , height = 2 * radius + 2 * half_length;
            Shape::Capsule(capsule) => Rect::from_center_half_size(
                self.pos,
                vec2(capsule.radius, capsule.half_length + capsule.radius),
            ),
        }
    }

    #[inline]
    pub fn center(&self) -> Vec2 {
        self.pos
    }

    /// Checks if `self` is entirely contained in `bounds`.
    #[inline]
    pub fn is_contained_by(self, bounds: Rect) -> bool {
        let self_aabb = self.aabb();
        bounds.contains(self_aabb.min) && bounds.contains(self_aabb.max)
    }

    /// Checks if `self` intersects with `other`.
    #[inline]
    pub fn intersects(self, other: impl AsQuadVal) -> bool {
        let QuadVal {
            pos: other_pos,
            shape: other_shape,
        } = other.as_quad_val();
        match self.shape {
            Shape::Quad(rectangle) => match other_shape {
                Shape::Quad(rectangle2) => {
                    rectangles_intersect(self.pos, rectangle, other_pos, rectangle2)
                }
                Shape::Circle(circle) => {
                    rectangle_circle_intersect(self.pos, rectangle, other_pos, circle.radius)
                }
                Shape::Capsule(capsule) => {
                    rectangle_capsule_intersect(self.pos, rectangle, other_pos, capsule)
                }
            },
            Shape::Circle(circle) => match other_shape {
                Shape::Quad(rectangle) => {
                    rectangle_circle_intersect(other_pos, rectangle, self.pos, circle.radius)
                }
                Shape::Circle(circle2) => {
                    circles_intersect(self.pos, circle.radius, other_pos, circle2.radius)
                }
                Shape::Capsule(capsule) => {
                    circle_capsule_intersect(self.pos, circle.radius, other_pos, capsule)
                }
            },
            Shape::Capsule(capsule) => match other_shape {
                Shape::Quad(rectangle) => {
                    rectangle_capsule_intersect(other_pos, rectangle, self.pos, capsule)
                }
                Shape::Circle(circle) => {
                    circle_capsule_intersect(other_pos, circle.radius, self.pos, capsule)
                }
                Shape::Capsule(capsule2) => {
                    capsules_intersect(self.pos, capsule, other_pos, capsule2)
                }
            },
        }
    }
}

/// A collision shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    Quad(Rectangle),
    Circle(Circle),
    Capsule(Capsule2d),
}

// ——> Helper functions to test for intersection between common shapes
//
#[inline]
fn rectangle_circle_intersect(
    rect_pos: Vec2,
    rectangle: Rectangle,
    c_center: Vec2,
    c_radius: f32,
) -> bool {
    let rect = Rect::from_center_half_size(rect_pos, rectangle.half_size);
    // find a point on the rectangle closest to the circle
    let close_pt = vec2(
        rect.min.x.max(c_center.x.min(rect.max.x)),
        rect.min.y.max(c_center.y.min(rect.max.y)),
    );

    close_pt.distance(c_center) <= c_radius
}

#[inline]
fn rectangle_capsule_intersect(
    rect_pos: Vec2,
    rectangle: Rectangle,
    c_pos: Vec2,
    capsule: Capsule2d,
) -> bool {
    let c_internal_rect =
        Rect::from_center_half_size(c_pos, vec2(capsule.radius, capsule.half_length));
    let rect = Rect::from_center_half_size(rect_pos, rectangle.half_size);

    let c1 = c_pos + vec2(0.0, capsule.half_length);
    let c2 = c_pos - vec2(0.0, capsule.half_length);

    rects_intersect(rect, c_internal_rect)
        || [c1, c2]
            .into_iter()
            .any(|c| rectangle_circle_intersect(rect_pos, rectangle, c, capsule.radius))
}

#[inline]
fn rectangles_intersect(
    pos1: Vec2,
    rectangle1: Rectangle,
    pos2: Vec2,
    rectangle2: Rectangle,
) -> bool {
    let rect1 = Rect::from_center_half_size(pos1, rectangle1.half_size);
    let rect2 = Rect::from_center_half_size(pos2, rectangle2.half_size);

    rects_intersect(rect1, rect2)
}

#[inline]
fn rects_intersect(rect: Rect, other: Rect) -> bool {
    // check on the x-axis
    (rect.min.x <= other.max.x && other.min.x <= rect.max.x)
    // check on the y-axis 
        && (rect.min.y <= other.max.y && other.min.y <= rect.max.y)
}

#[inline]
fn circle_capsule_intersect(
    c_center: Vec2,
    c_radius: f32,
    cap_center: Vec2,
    capsule: Capsule2d,
) -> bool {
    let cap_intern_rect =
        Rect::from_center_half_size(cap_center, vec2(capsule.radius, capsule.half_length));

    let c1 = cap_center + vec2(0.0, capsule.half_length);
    let c2 = cap_center - vec2(0.0, capsule.half_length);

    rectangle_circle_intersect(
        cap_intern_rect.center(),
        Rectangle::new(cap_intern_rect.width(), cap_intern_rect.height()),
        c_center,
        c_radius,
    ) || [c1, c2]
        .into_iter()
        .any(|c| circles_intersect(c_center, c_radius, c, capsule.radius))
}

#[inline]
fn circles_intersect(c1: Vec2, r1: f32, c2: Vec2, r2: f32) -> bool {
    let dist = c1.distance(c2);
    let r_sum = r1 + r2;
    dist <= r_sum
}

fn capsules_intersect(c1: Vec2, capsule1: Capsule2d, c2: Vec2, capsule2: Capsule2d) -> bool {
    let intern_rects = [
        Rectangle::new(capsule1.radius * 2., capsule1.half_length * 2.),
        Rectangle::new(capsule2.radius * 2., capsule2.half_length * 2.),
    ];

    if rectangles_intersect(c1, intern_rects[0], c2, intern_rects[1]) {
        return true;
    }
    let c1c1 = c1 + vec2(0.0, capsule1.half_length);
    let c1c2 = c1 - vec2(0.0, capsule1.half_length);
    let c2c1 = c2 + vec2(0.0, capsule2.half_length);
    let c2c2 = c2 - vec2(0.0, capsule2.half_length);

    let centers1 = [c1c1, c1c2];
    let centers2 = [c2c1, c2c2];

    let r1 = capsule1.radius;
    let r2 = capsule2.radius;

    centers1.into_iter().any(|center_1| {
        if rectangle_circle_intersect(c2, intern_rects[1], center_1, r1) {
            return true;
        }

        centers2.into_iter().any(|center_2| {
            rectangle_circle_intersect(c1, intern_rects[0], center_2, r2)
                || circles_intersect(center_1, r1, center_2, r2)
        })
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use bevy::math::{vec2, Rect};

    #[test]
    fn intersect_helpers_work() {
        let rect = Rect::from_corners(vec2(0.0, 0.0), vec2(50.0, 50.0));
        let c_center = vec2(-1.0, 25.0);
        let c_radius = 4.0;
        let c_center2 = vec2(-4.0, 2.0);
        let c_radius2 = 4.0;

        assert!(rectangle_circle_intersect(
            rect.center(),
            Rectangle::new(rect.width(), rect.height()),
            c_center,
            c_radius
        ));
        assert!(rectangle_circle_intersect(
            rect.center(),
            Rectangle::new(rect.width(), rect.height()),
            c_center2,
            c_radius2
        ));

        let rect_49 = Rect::from_corners(Vec2::splat(49.), Vec2::splat(52.));
        let rect_centered = Rect::from_center_size(Vec2::splat(25.), Vec2::splat(5.));
        let rect_touching = Rect::from_corners(Vec2::splat(50.0), Vec2::splat(51.0));

        assert!(rects_intersect(rect, rect_49));
        assert!(rects_intersect(rect, rect_centered));
        assert!(rects_intersect(rect, rect_touching));
        assert!(!rects_intersect(rect_49, rect_centered));

        let c2 = vec2(4., 25.);
        let r2 = 1.0;
        let r3 = 10.0;
        let r4 = 0.3;

        assert!(circles_intersect(c_center, c_radius, c2, r2));
        assert!(circles_intersect(c_center, c_radius, c2, r3));
        assert!(!circles_intersect(c_center, c_radius, c2, r4));

        let cap = vec2(4., -3.);
        let capsule = Capsule2d::new(1., 6.);
        let capsule2 = Capsule2d::new(1., 4.);
        let capsule3 = Capsule2d::new(0.5, 4.);
        assert!(rectangle_capsule_intersect(
            rect.center(),
            Rectangle::new(rect.width(), rect.height()),
            cap,
            capsule
        ));
        assert!(rectangle_capsule_intersect(
            rect.center(),
            Rectangle::new(rect.width(), rect.height()),
            cap,
            capsule2
        ));
        assert!(!rectangle_capsule_intersect(
            rect.center(),
            Rectangle::new(rect.width(), rect.height()),
            cap,
            capsule3
        ));

        let c_pos = vec2(6.0, -3.0);
        let c_rad = 1.;
        let c_rad2 = 0.5;
        assert!(circle_capsule_intersect(c_pos, c_rad, cap, capsule));
        assert!(!circle_capsule_intersect(c_pos, c_rad2, cap, capsule));
        let c_pos = vec2(5.0, -7.0);
        let c_rad = 1.0;
        assert!(circle_capsule_intersect(c_pos, c_rad, cap, capsule));

        let cap2 = vec2(4.0, -10.);
        let capsule2 = Capsule2d::new(1.0, 4.0);
        let capsule3 = Capsule2d::new(5.0, 1.0);
        assert!(capsules_intersect(cap, capsule, cap2, capsule2));
        assert!(capsules_intersect(cap, capsule, cap2, capsule3));
    }

    #[test]
    fn shapes_work() {
        let field = Rect::from_corners(Vec2::splat(0.0), Vec2::splat(40.0));

        // let r = Shape::Quad(Rect::from_corners(Vec2::splat(0.0), Vec2::splat(8.0)));
        let r = QuadVal {
            pos: Vec2::splat(4.0),
            shape: Shape::Quad(Rectangle::new(8.0, 8.0)),
        };
        let cap = QuadVal {
            pos: Vec2::splat(10.),
            shape: Shape::Capsule(Capsule2d::new(1., 10.)),
        };
        let circ = QuadVal {
            pos: vec2(4.0, 9.0),
            shape: Shape::Circle(Circle::new(1.0)),
        };
        let circ2 = QuadVal {
            pos: vec2(11.0, 9.0),
            shape: Shape::Circle(Circle::new(2.0)),
        };

        assert!(r.intersects(circ));
        assert!(cap.intersects(circ2));
        assert!(!r.intersects(circ2));
        assert!(!cap.intersects(r));

        [r, cap, circ, circ2]
            .into_iter()
            .for_each(|shape| assert!(shape.is_contained_by(field)));
    }
}
