//! An implementation of a [`Quadtree`].

use bevy::math::{vec2, Rect};

pub mod as_rect;

use as_rect::AsRect;

/// A `Quadtree` implementation using [`bevy`] types.
///
/// Uses a [`Rect`] to determine the `Quadtree` bounds and which quadtree-node the value belongs to.
/// As a result all values that need to be stored in the `Quadtree` need to implement [`AsRect`]
/// helper trait.
///
/// FIXME: improve this.
/// Quadrants are stored in counter-clockwise order.
///
/// BotLeft(0,0) -> BotRight(width, 0) -> TopRight(width, height) -> TopLeft(0, height)
pub struct Quadtree<T>
where
    T: PartialEq + AsRect,
{
    bounds: Rect,
    root: Box<QNode<T>>,
}

impl<T: PartialEq + AsRect> Quadtree<T> {
    const THRESHOLD: usize = 16;
    const MAX_DEPTH: usize = 8;

    /// Initializes a `Quadtree` from the provided bounds.
    #[inline]
    pub fn new(bounds: impl AsRect) -> Self {
        let bounds = bounds.as_rect();
        Quadtree {
            bounds,
            root: Box::new(QNode::new()),
        }
    }

    /// Adds a new value to the `Quadtree`
    #[inline]
    pub fn add(&mut self, val: T) {
        self.root.add(self.bounds, 0, val);
    }

    /// Removes a value from the `Quadtree`
    #[inline]
    pub fn remove(&mut self, val: &T) {
        self.root.remove(self.bounds, val);
    }
}

/// A [`Quadtree`] node.
///
/// child 0 -> child 1  -> child 2  -> child 3
/// BotLeft -> BotRight -> TopRight -> TopLeft
struct QNode<T: PartialEq + AsRect> {
    children: [Option<Box<QNode<T>>>; 4],
    values: Vec<T>,
}

impl<T: PartialEq + AsRect> QNode<T> {
    #[inline]
    fn new() -> Self {
        let capacity = Quadtree::<T>::THRESHOLD;
        Self {
            children: [None, None, None, None],
            values: Vec::with_capacity(capacity),
        }
    }

    #[inline]
    #[must_use]
    fn is_leaf(&self) -> bool {
        self.children[0].is_none()
    }

    fn add(&mut self, bounds: Rect, depth: usize, val: T) {
        let val_bounds = val.as_rect();
        assert!(
            bounds.contains(val_bounds.min) && bounds.contains(val_bounds.max),
            "provided value is out of bounds!"
        );

        let max_depth = Quadtree::<T>::MAX_DEPTH;
        let threshold = Quadtree::<T>::THRESHOLD;

        if self.is_leaf() {
            // insert the value in this node if possible
            if depth >= max_depth || self.values.len() < threshold {
                self.values.push(val);
            } else {
                // otherwise split and try again
                self.split(bounds);
                self.add(bounds, depth, val);
            }
        } else if let Some(idx) = find_quadrant(bounds, val_bounds) {
            // Add the value to a child if the value is entirely contained in it
            self.children[idx].as_mut().expect("isn't a leaf node").add(
                compute_rect(bounds, idx),
                depth + 1,
                val,
            );
        } else {
            // Otherwise add the value to the current node.
            self.values.push(val);
        }
    }

    fn remove(&mut self, bounds: Rect, val: &T) {
        todo!()
    }

    fn split(&mut self, bounds: Rect) {
        assert!(self.is_leaf());
        // initialize children
        for child in self.children.iter_mut() {
            *child = Some(Box::new(QNode::new()));
        }

        let mut new_values = Vec::with_capacity(Quadtree::<T>::THRESHOLD);

        // Swap the current `values` for an empty `Vec`,
        // so we can take ownership of the current `values`
        let mut old_values = Vec::new();
        std::mem::swap(&mut self.values, &mut old_values);

        for val in old_values {
            // If we find the quadrant to insert, we insert
            if let Some(idx) = find_quadrant(bounds, val.as_rect()) {
                let child_qnode = self.children[idx].as_deref_mut().expect("init above");
                child_qnode.values.push(val);
            // Otherwise keep in the current Node
            } else {
                new_values.push(val);
            }
        }

        std::mem::swap(&mut self.values, &mut new_values)
    }
}

/// A helper function that computes a [`Rect`] of a child based on the [`Rect`] of its parent and
/// the index of its quadrant.
/// Quadrants are stored in counter-clockwise order, see [`QNode`].
fn compute_rect(parent: Rect, idx: usize) -> Rect {
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
fn find_quadrant(bounds: Rect, quad: impl AsRect) -> Option<usize> {
    let center = bounds.center();
    let quad = quad.as_rect();

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
    use super::*;
    use bevy::math::{vec2, Rect, Vec2};

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
            let result = find_quadrant(*bounds, *quad);
            assert_eq!(
                result,
                *expected,
                "Test case {} failed: expected {:?}, got {:?}",
                i + 1,
                expected,
                result
            );
        }
    }

    #[test]
    fn compute_rect_works() {
        let parent = Rect::from_corners(vec2(0.0, 0.0), vec2(4.0, 4.0));
        let child_size = parent.half_size();

        let test_cases = vec![
            // Quadrant 0
            (
                0,
                Rect::from_corners(vec2(0.0, 0.0), vec2(child_size.x, child_size.y)),
            ),
            // Quadrant 1
            (
                1,
                Rect::from_corners(
                    vec2(child_size.x, 0.0),
                    vec2(child_size.x * 2.0, child_size.y),
                ),
            ),
            // Quadrant 2
            (
                2,
                Rect::from_corners(
                    vec2(child_size.x, child_size.y),
                    vec2(child_size.x * 2.0, child_size.y * 2.0),
                ),
            ),
            // Quadrant 3
            (
                3,
                Rect::from_corners(
                    vec2(0.0, child_size.y),
                    vec2(child_size.x, child_size.y * 2.0),
                ),
            ),
        ];

        for (idx, expected) in test_cases {
            let result = compute_rect(parent, idx);
            assert_eq!(
                result, expected,
                "Quadrant {} failed: expected {:?}, got {:?}",
                idx, expected, result
            );
        }
    }

    #[test]
    fn is_leaf_works() {
        use crate::quadtree::QNode;

        let mut qnode = QNode::new();
        let bounds = Rect::from_corners(vec2(0., 0.), vec2(2.0, 2.0));

        assert!(qnode.is_leaf());

        let pts = [
            vec2(0.5, 0.5),
            vec2(2.0, 0.0),
            vec2(1.0, 1.0),
            vec2(0.0, 2.0),
        ];

        for pt in pts {
            qnode.add(bounds, 0, pt);
        }
        assert!(qnode.is_leaf());
        assert_eq!(qnode.values.len(), 4);

        qnode.split(bounds);

        assert!(!qnode.is_leaf());
        assert_eq!(qnode.values.len(), 0);

        for (idx, pt) in pts.into_iter().enumerate() {
            let child_qnode = qnode.children[idx].as_ref().unwrap();
            assert!(child_qnode.values.contains(&pt));
        }
    }

    #[test]
    fn quadtree_adding() {
        let mut qt = Quadtree::new(Rect::from_corners(vec2(0., 0.), vec2(8.0, 8.0)));

        // Points to add
        let pts = [
            vec2(1.0, 1.0), // Bottom-Left quadrant
            vec2(7.0, 7.0), // Top-Right quadrant
            vec2(3.0, 5.0), // Top-Left quadrant
            vec2(6.5, 1.5), // Bottom-Right quadrant
            vec2(4.0, 4.0), // Center, should remain in root
        ];

        for pt in pts {
            qt.add(pt);
        }

        // Initial assertions
        assert!(qt.root.is_leaf(), "Root should initially be a leaf node");
        assert_eq!(
            qt.root.values.len(),
            5,
            "All points should be in root initially"
        );

        // Add enough points to exceed the threshold and trigger a split
        for x in (1..5).flat_map(|x| (1..5).map(move |y| vec2(x as f32, y as f32))) {
            qt.add(x);
        }

        assert!(
            !qt.root.is_leaf(),
            "Root should no longer be a leaf node after exceeding the threshold"
        );
        assert_eq!(
            qt.root.values.len(),
            0,
            "All values should get distributed among children"
        );

        // Verify points are distributed among child nodes
        for (idx, child) in qt.root.children.iter().enumerate() {
            let child_qnode = child
                .as_ref()
                .expect("Child node should exist after splitting");
            let rect = compute_rect(qt.bounds, idx);
            assert!(
                child_qnode
                    .values
                    .iter()
                    .all(|val| rect.contains(val.as_rect().center())),
                "All values in quadrant {} should be within its bounds",
                idx
            );
        }

        // Test adding boundary points
        let boundary_pts = [
            vec2(0.0, 0.0), // Bottom-left corner
            vec2(8.0, 0.0), // Bottom-right corner
            vec2(8.0, 8.0), // Top-right corner
            vec2(0.0, 8.0), // Top-left corner
        ];
        for pt in boundary_pts {
            qt.add(pt);
        }

        // Verify boundary points are added correctly
        for pt in boundary_pts {
            let added = qt
                .root
                .children
                .iter()
                .any(|child| child.as_ref().is_some_and(|c| c.values.contains(&pt)));
            assert!(
                added,
                "Boundary point {:?} should be added to the correct quadrant",
                pt
            );
        }

        // Further test with a deeply nested tree structure
        let nested_pts = [
            vec2(0.1, 0.1), // Near bottom-left, to test deep nesting
            vec2(7.9, 7.9), // Near top-right, to test deep nesting
        ];
        for pt in nested_pts {
            qt.add(pt);
        }

        // Verify deep nesting for boundary points
        for pt in nested_pts {
            let mut current_node = &qt.root;
            let mut depth = 0;
            while !current_node.is_leaf() {
                depth += 1;
                let idx =
                    find_quadrant(qt.bounds, pt).expect("Point should belong to a valid quadrant");
                current_node = current_node.children[idx]
                    .as_ref()
                    .expect("Expected child node to exist");
            }
            assert!(
                current_node.values.contains(&pt),
                "Deeply nested point {:?} should be in the correct node",
                pt
            );
            assert!(
                depth <= Quadtree::<Vec2>::MAX_DEPTH,
                "Tree depth should not exceed MAX_DEPTH"
            );
        }
    }
}
