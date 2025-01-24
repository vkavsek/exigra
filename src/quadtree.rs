use bevy::math::{vec2, Rect, Vec2};

pub struct Quadtree<T>
where
    T: PartialEq,
{
    rect: Rect,
    root: Option<Box<QNode<T>>>,
}

impl<T: PartialEq> Quadtree<T> {
    const THRESHOLD: usize = 16;
    const MAX_DEPTH: usize = 8;

    pub fn new(rect: Rect) -> Self {
        Quadtree { rect, root: None }
    }

    fn is_leaf(&self, node: &QNode<T>) -> bool {
        node.children[0].is_none()
    }
}

/// Quadrants are stored in counter-clockwise order.
///
/// 0          1           2           3
/// BotLeft -> BotRight -> TopRight -> TopLeft
struct QNode<T: PartialEq> {
    children: [Option<Box<QNode<T>>>; 4],
    values: Vec<T>,
}

/// A helper function that computes a [`Rect`] of a child based on the [`Rect`] of its parent and
/// the index of its quadrant.
/// Quadrants are stored in counter-clockwise order, see [`QNode`].
fn compute_rect(parent: &Rect, idx: u8) -> Rect {
    let origin = vec2(parent.min.x, parent.min.y);
    let child_size = parent.half_size();
    match idx {
        //(0,0)(1,1)
        0 => Rect::from_corners(origin, origin + child_size),
        //(1,0)(2,1)
        1 => Rect::from_corners(
            origin + vec2(child_size.x, 0.0),
            origin + vec2(child_size.x * 2.0, child_size.y),
        ),
        //(1,1)(2,2)
        2 => Rect::from_corners(origin + child_size, origin + child_size * 2.0),
        //(0,1)(1,2)
        3 => Rect::from_corners(
            origin + vec2(0.0, child_size.y),
            origin + vec2(child_size.x, child_size.y * 2.0),
        ),
        _ => unreachable!("received an index greater than 3. Invalid use of a Quadtree!"),
    }
}

/// A helper function that finds a quadrant for a given quad.
fn find_quadrant(bounds: &Rect, quad: &Rect) -> Option<u8> {
    let center = bounds.center();

    // Return early if the quad is out of bounds.
    if quad.max.x > bounds.max.x
        || quad.max.y > bounds.max.y
        || quad.min.x < bounds.min.x
        || quad.min.y < bounds.min.y
    {
        return None;
    }

    // Try to find the quadrant and return early if you do
    if quad.max.x < center.x {
        if quad.max.y < center.y {
            return Some(0);
        } else if quad.min.y >= center.y {
            return Some(3);
        }
    } else if quad.min.x >= center.x {
        if quad.max.y < center.y {
            return Some(1);
        } else if quad.min.y >= center.y {
            return Some(2);
        }
    }

    None
}

#[cfg(test)]
mod test {
    use bevy::math::{vec2, Rect};

    use crate::quadtree::find_quadrant;

    #[test]
    fn compute_rect_works() {
        todo!()
    }

    #[test]
    fn find_quadrant_works() {
        let bounds = Rect::from_corners(vec2(0.0, 0.0), vec2(4.0, 4.0));
        let test_cases = vec![
            // Test case 1: Point in the Bottom-Left quadrant (Quadrant 0)
            (
                bounds,
                Rect::from_corners(vec2(0.0, 0.0), vec2(1.9, 1.9)),
                Some(0),
            ),
            // Test case 2: Point in the Bottom-Right quadrant (Quadrant 1)
            (
                bounds,
                Rect::from_corners(vec2(2.0, 0.0), vec2(4.0, 1.9)),
                Some(1),
            ),
            // Test case 3: Point in the Top-Right quadrant (Quadrant 2)
            (
                bounds,
                Rect::from_corners(vec2(2.0, 2.0), vec2(4.0, 4.0)),
                Some(2),
            ),
            // Test case 4: Point in the Top-Left quadrant (Quadrant 3)
            (
                bounds,
                Rect::from_corners(vec2(0.0, 2.0), vec2(1.9, 4.0)),
                Some(3),
            ),
            // Test case 5: Point exactly at the center of the parent bounds (None)
            (
                bounds,
                Rect::from_corners(vec2(1.5, 1.5), vec2(2.5, 2.5)),
                None,
            ),
            // Test case 6: Quad outside bounds (right)
            (
                bounds,
                Rect::from_corners(vec2(4.1, 0.0), vec2(5.0, 2.0)),
                None,
            ),
            // Test case 7: Quad outside bounds (left)
            (
                bounds,
                Rect::from_corners(vec2(-1.0, 0.0), vec2(0.0, 2.0)),
                None,
            ),
            // Test case 8: Quad outside bounds (top)
            (
                bounds,
                Rect::from_corners(vec2(0.0, 4.1), vec2(2.0, 5.0)),
                None,
            ),
            // Test case 9: Quad outside bounds (bottom)
            (
                bounds,
                Rect::from_corners(vec2(0.0, -1.0), vec2(2.0, 0.0)),
                None,
            ),
            // Test case 10: Empty Quad (Quadrant 2)
            (
                bounds,
                Rect::from_corners(vec2(2.0, 2.0), vec2(2.0, 2.0)),
                Some(2),
            ),
        ];

        for (i, (bounds, quad, expected)) in test_cases.iter().enumerate() {
            let result = find_quadrant(bounds, quad);
            assert_eq!(
                result,
                *expected,
                "Test case {} failed: expected {:?}, got {:?}",
                i + 1,
                expected,
                result
            );
        }

        println!("All test cases passed!");
    }
}
