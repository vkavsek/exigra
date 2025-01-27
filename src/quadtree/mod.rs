//! An implementation of a [`Quadtree`].
//!
//! TODO:
//!     - test query and find_all_intersections
//!     - clear?
//!     - Error?
//!     - Shape instead of Rect. aka rework the Quad trait to allow for different primitive shapes (Circle,
//!     Rect, Capsule)

use bevy::math::{vec2, Rect};

pub mod quad;

use quad::Quad;

/// A `Quadtree` implementation using [`bevy`] compatible types.
///
/// Uses a [`Rect`] to determine the `Quadtree` bounds and which quadtree-node the value belongs to.
/// As a result all values that need to be stored in the `Quadtree` need to implement [`Quad`] helper trait.
///
/// Current implementation stores all values even if they don't fit in the bounding box of the `Quadtree`!
/// Values that are out of bounds are stored in the `root` node of the tree.
///
/// Quadrants are stored in counter-clockwise order.
/// In bevy this means:
/// BotLeft(0,0) -> BotRight(width, 0) -> TopRight(width, height) -> TopLeft(0, height)
#[derive(Debug)]
pub struct Quadtree<T>
where
    T: PartialEq + Quad + Clone,
{
    bounds: Rect,
    root: Box<QNode<T>>,
}

impl<T: PartialEq + Quad + Clone> Quadtree<T> {
    const THRESHOLD: usize = 16;
    const MAX_DEPTH: usize = 8;

    /// Initializes an empty `Quadtree` from the provided bounds.
    #[inline]
    pub fn new(bounds: Rect) -> Self {
        let bounds = bounds.as_quad();
        Quadtree {
            bounds,
            root: Box::new(QNode::new()),
        }
    }

    /// Inserts a new value to the `Quadtree`
    ///
    /// Panics if the provided values can't fit in the Quadtree!
    #[inline]
    pub fn insert(&mut self, val: T) {
        self.root.insert(self.bounds, 0, val);
    }

    /// Inserts many new values to the `Quadtree`
    #[inline]
    pub fn insert_many(&mut self, items: &[T]) {
        let items = items.to_vec();
        self.root.insert_many(self.bounds, 0, items);
    }

    /// Removes a value from the `Quadtree`
    ///
    /// Panics if the provided values can't fit in the Quadtree!
    #[inline]
    pub fn remove(&mut self, val: &T) {
        self.root.remove(self.bounds, val);
    }

    /// Queries for all the values that intersect the `query_bounds`.
    /// All the contained values are returned in a [`Vec`].
    ///
    /// Panics if provided `query_bounds` don't intersect with the `Quadtree`'s bounds.
    #[inline]
    pub fn query(&self, query_bounds: Rect) -> Vec<&T> {
        // reserve space for 256 items as a sane default
        let mut contained_values = Vec::with_capacity(256);
        self.root
            .query(self.bounds, query_bounds, &mut contained_values);
        contained_values
    }

    /// Finds all the intersecting values stored in the Quadtree.
    /// All intersection pairs are returned in a [`Vec`].
    pub fn find_all_intersections(&self) -> Vec<(&T, &T)> {
        // reserve space for 256 items as a sane default
        let mut intersections = Vec::with_capacity(256);
        self.root.find_all_intersections(&mut intersections);
        intersections
    }
}

/// A [`Quadtree`] node.
///
/// child 0 -> child 1  -> child 2  -> child 3
/// BotLeft -> BotRight -> TopRight -> TopLeft
#[derive(Debug)]
struct QNode<T: PartialEq + Quad + Clone> {
    children: [Option<Box<QNode<T>>>; 4],
    values: Vec<T>,
}

impl<T: PartialEq + Quad + Clone> QNode<T> {
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

    fn insert_many(&mut self, bounds: Rect, depth: usize, items: Vec<T>) {
        if self.is_leaf() {
            // if leaf and fits or if we are at max depth extend with items
            if self.values.len() + items.len() <= Quadtree::<T>::THRESHOLD
                || depth >= Quadtree::<T>::MAX_DEPTH
            {
                self.values.extend(items);
            } else {
                // values len is over the threshold limit
                // subdivide and try again
                self.subdivide(bounds);
                self.insert_many(bounds, depth, items);
            }
        } else {
            // non leaf
            let groups = group_by_quadrant(bounds, items);
            for (i, quadrant_items) in groups.into_iter().enumerate() {
                // if we find a child we are looking at one of the first 4 groups
                // we try to recursively insert an appropriate vector of items into each of the children
                if let Some(child) = self.children.get_mut(i) {
                    let child = child.as_deref_mut().expect("parent is not a leaf");
                    let child_bounds = compute_bounds(bounds, i);
                    if !quadrant_items.is_empty() {
                        child.insert_many(child_bounds, depth + 1, quadrant_items);
                    }
                // otherwise we are looking at the last group - values that don't fit
                // in any of the child quadrants - the parent should insert them.
                } else {
                    self.values.extend(quadrant_items);
                }
            }
        }
    }

    fn insert(&mut self, bounds: Rect, depth: usize, val: T) {
        let val_bounds = val.as_quad();
        let max_depth = Quadtree::<T>::MAX_DEPTH;
        let threshold = Quadtree::<T>::THRESHOLD;

        if self.is_leaf() {
            // insert the value in this node if possible
            if depth >= max_depth || self.values.len() < threshold {
                self.values.push(val);
            } else {
                // otherwise split and try again
                self.subdivide(bounds);
                self.insert(bounds, depth, val);
            }
        } else if let Some(idx) = find_quadrant(bounds, val_bounds) {
            // Add the value to a child if the value is entirely contained in it
            self.children[idx]
                .as_mut()
                .expect("isn't a leaf node")
                .insert(compute_bounds(bounds, idx), depth + 1, val);
        } else {
            // Otherwise add the value to the current node.
            self.values.push(val);
        }
    }

    fn subdivide(&mut self, bounds: Rect) {
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
            if let Some(idx) = find_quadrant(bounds, val.as_quad()) {
                let child_qnode = self.children[idx].as_deref_mut().expect("init above");
                child_qnode.values.push(val);
            // Otherwise keep in the current Node
            } else {
                new_values.push(val);
            }
        }

        std::mem::swap(&mut self.values, &mut new_values)
    }

    /// Recursively tries to remove a value from `QNode` and its children,
    /// and merging appropriate parent nodes with its children.
    ///
    /// Returns `true` if the `QNode`'s parent node should try to merge with its children.
    ///
    /// Panics if the value can't be contained in `QNode`'s bounds.
    fn remove(&mut self, bounds: Rect, val: &T) -> bool {
        if self.is_leaf() {
            self.remove_found_val(val);
            // if this qnode is a leaf and we removed a value we should try to merge
            true
        } else if let Some(idx) = find_quadrant(bounds, val.as_quad()) {
            if self.children[idx]
                .as_deref_mut()
                .expect("not a leaf")
                .remove(compute_bounds(bounds, idx), val)
            {
                self.try_merge()
            } else {
                unreachable!("value should always be contained in one of the quadrants")
            }
        } else {
            self.remove_found_val(val);
            // not a leaf, no need to merge
            false
        }
    }

    /// Removes a value that is EXPECTED to be contained in the `values` array of this `QNode`.
    /// Does nothing if the value isn't found in the array.
    fn remove_found_val(&mut self, val: &T) {
        if let Some(i) = self.values.iter().position(|v| val == v) {
            // swap if the value is not the last element of the array
            let last = self.values.len() - 1;
            if i != last {
                self.values.swap(i, last);
            }
            // remove the last element
            self.values.pop();
        }
    }

    /// Checks that all of the `QNode`'s children are leaves and that the total number of its values
    /// and the childrens values is lower than the threshold.
    ///
    /// If the node is merged, it returns `true` to signal that its parent should also try to merge.
    fn try_merge(&mut self) -> bool {
        assert!(!self.is_leaf(), "only interior nodes can be merged");

        let mut values_len = self.values.len();
        for child in self.children.iter() {
            let child = child.as_deref().expect("parent is not a leaf");
            if !child.is_leaf() {
                return false;
            }
            values_len += child.values.len();
        }

        if values_len <= Quadtree::<T>::THRESHOLD {
            for child in self.children.iter_mut() {
                // reset the child node to None
                let child_vals = child.take().expect("parent is not a leaf").values;
                // extend the values with child's values
                self.values.extend(child_vals);
            }
            true
        } else {
            false
        }
    }

    /// A spatial query.
    /// Recursively queries the `QNode` and its children for values that intersect with the
    /// provided `query_bounds`
    fn query<'qt>(
        &'qt self,
        quad_bounds: Rect,
        query_bounds: Rect,
        contained_values: &mut Vec<&'qt T>,
    ) {
        assert!(!quad_bounds.intersect(query_bounds).is_empty());
        for val in self.values.iter() {
            let val_quad = val.as_quad();
            if query_bounds.contains(val_quad.min) || query_bounds.contains(val_quad.max) {
                contained_values.push(val);
            }
        }

        if !self.is_leaf() {
            for i in 0..self.children.len() {
                let child_bounds = compute_bounds(quad_bounds, i);
                if !query_bounds.intersect(child_bounds).is_empty() {
                    self.children[i]
                        .as_deref()
                        .expect("parent is not leaf")
                        .query(child_bounds, query_bounds, contained_values);
                }
            }
        }
    }

    /// Recursively finds intersections between values stored in this node
    /// Makes sure to not report the same intersection twice
    fn find_all_intersections<'qt>(&'qt self, intersections: &mut Vec<(&'qt T, &'qt T)>) {
        // skip first value to avoid an empty check
        for (i, val_a) in self.values.iter().enumerate().skip(1) {
            for val_b in self.values[0..i].iter() {
                // if intersection isn't empty push the values into intersections.
                if !val_a.as_quad().intersect(val_b.as_quad()).is_empty() {
                    intersections.push((val_a, val_b));
                }
            }
        }

        // values in current node can intersect values in childs and their descendants
        if !self.is_leaf() {
            for child in self.children.iter() {
                let child = child.as_deref().expect("parent is not leaf");
                for val in self.values.iter() {
                    // find intersections with the current value in descendants of children and the child itself
                    child.find_intersections_in_descendants(val, intersections);
                }

                // recursively search each of the children for additional intersections
                child.find_all_intersections(intersections);
            }
        }
    }

    /// Recursively searches the current node and it's descendants for intersections with the provided `val`,
    /// and stores them in `intersections`.
    fn find_intersections_in_descendants<'qt>(
        &'qt self,
        val: &'qt T,
        intersections: &mut Vec<(&'qt T, &'qt T)>,
    ) {
        for other in self.values.iter() {
            if !val.as_quad().intersect(other.as_quad()).is_empty() {
                intersections.push((val, other));
            }
        }

        if !self.is_leaf() {
            for child in self.children.iter() {
                let child = child.as_deref().expect("parent is not leaf");
                child.find_intersections_in_descendants(val, intersections);
            }
        }
    }
}

/// Creates quadrant groups from the provided `items`.
///
/// It returns an array of 5 [`Vec`]'s, first four have indices corresponding to the indices of the
/// quadrants.
/// The 5th `Vec` stores the items that couldn't be stored in any of the child qudrants and should
/// therefore be stored by the parent
fn group_by_quadrant<T: PartialEq + Quad>(bounds: Rect, items: Vec<T>) -> [Vec<T>; 5] {
    // initialize the return array
    let mut res = [vec![], vec![], vec![], vec![], vec![]];

    for item in items {
        if let Some(idx) = find_quadrant(bounds, item.as_quad()) {
            res[idx].push(item);
        } else {
            res[4].push(item);
        }
    }

    res
}

/// A helper function that computes an axis-aligned bounding box [`Rect`] of a child based on
/// the bounding box of its parent and the index of its quadrant.
/// Quadrants are stored in counter-clockwise order, see [`QNode`].
fn compute_bounds(parent: Rect, idx: usize) -> Rect {
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

/// A helper function that finds a quadrant for a given value.
fn find_quadrant(bounds: Rect, val: impl Quad) -> Option<usize> {
    let center = bounds.center();
    let quad = val.as_quad();

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

// —> TESTS
#[cfg(test)]
mod test {
    use super::*;
    use bevy::math::{vec2, Rect};

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
    fn compute_bounds_works() {
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
            let result = compute_bounds(parent, idx);
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
            qnode.insert(bounds, 0, pt);
        }
        assert!(qnode.is_leaf());
        assert_eq!(qnode.values.len(), 4);

        qnode.subdivide(bounds);

        assert!(!qnode.is_leaf());
        assert_eq!(qnode.values.len(), 0);

        for (idx, pt) in pts.into_iter().enumerate() {
            let child_qnode = qnode.children[idx].as_ref().unwrap();
            assert!(child_qnode.values.contains(&pt));
        }
    }

    #[test]
    fn quadtree_insert_remove_works() {
        let mut qtree = Quadtree::new(Rect::from_corners(vec2(0., 0.), vec2(8.0, 8.0)));

        // Points to add
        let pts = [
            vec2(1.0, 1.0), // Bottom-Left quadrant
            vec2(7.0, 7.0), // Top-Right quadrant
            vec2(3.0, 5.0), // Top-Left quadrant
            vec2(6.5, 1.5), // Bottom-Right quadrant
            vec2(4.0, 4.0), // Center, Top-Right quadrant
        ];

        qtree.insert_many(&pts);

        // Initial assertions
        assert!(qtree.root.is_leaf(), "Root should initially be a leaf node");
        assert_eq!(
            qtree.root.values.len(),
            5,
            "All points should be in root initially"
        );

        // Add enough points to exceed the threshold and trigger a split
        let threshold_pts = (1..5).flat_map(|x| (1..5).map(move |y| vec2(x as f32, y as f32)));
        for x in threshold_pts.clone() {
            qtree.insert(x);
        }

        assert!(
            !qtree.root.is_leaf(),
            "Root should no longer be a leaf node after exceeding the threshold"
        );
        assert_eq!(
            qtree.root.values.len(),
            0,
            "All values should get distributed among children"
        );

        // Verify points are distributed among child nodes
        for (idx, child) in qtree.root.children.iter().enumerate() {
            let child_qnode = child
                .as_ref()
                .expect("Child node should exist after splitting");
            let rect = compute_bounds(qtree.bounds, idx);
            assert!(
                child_qnode
                    .values
                    .iter()
                    .all(|val| rect.contains(val.as_quad().center())),
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
        qtree.insert_many(&boundary_pts);

        // Verify boundary points are added correctly
        for pt in boundary_pts {
            let added = qtree
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

        // Test removing points
        let remove_pts = [vec2(0.0, 0.0), vec2(8.0, 8.0)];
        for pt in remove_pts.into_iter().chain(threshold_pts) {
            qtree.remove(&pt);
        }

        // Verify removed points no longer exist in the tree
        for pt in remove_pts {
            let found = qtree
                .root
                .children
                .iter()
                .any(|child| child.as_ref().is_some_and(|c| c.values.contains(&pt)))
                || qtree.root.values.contains(&pt);
            assert!(!found, "Point {:?} should be removed from the quadtree", pt);
        }

        // Ensure tree rebalances if possible
        assert!(
            qtree.root.is_leaf(),
            "Tree should rebalance and root should be a leaf after removing points"
        );

        let oob_pts = (-4..0)
            .flat_map(|x| (-4..0).map(move |y| vec2(x as f32, y as f32)))
            .collect::<Vec<_>>();

        qtree.insert_many(&oob_pts);

        dbg!(&qtree);
        assert!(
            !qtree.root.is_leaf(),
            "out of bounds values get inserted into the root node, but the valid values get split amongst the child nodes"
        );

        assert_eq!(
            qtree.root.values.len(),
            16,
            "out of bounds values get inserted into the root node"
        );
    }

    // #[test]
    // fn quadtree_query_works() {
    //     let mut qtree: Quadtree<Vec2> =
    //         Quadtree::new(Rect::from_corners(vec2(0., 0.), vec2(8.0, 8.0)));
    //
    //     // all pts between (0.0, 0.0) and (3.5, 3.5) in increments of 0.5;
    //     // 36 points to insert
    //     // quadtree should split twice specifically the first quadrant
    //     let pts: Vec<_> = (0..4)
    //         .flat_map(|x| {
    //             (0..4).flat_map(move |y| {
    //                 [
    //                     vec2(x as f32, y as f32),
    //                     vec2(x as f32 + 0.5, y as f32 + 0.5),
    //                 ]
    //             })
    //         })
    //         .collect();
    //
    //     // qtree.insert_many(pts);
    //
    //     todo!()
    // }
    //
    // #[test]
    // fn quadtree_find_all_intersections_works() {
    //     todo!()
    // }
}
