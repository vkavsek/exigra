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

struct QNode<T: PartialEq> {
    children: [Option<Box<QNode<T>>>; 4],
    values: Vec<T>,
}
