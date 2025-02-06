use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use crate::prelude::*;
use crate::quadtree::quad_collider::{AsQuadCollider, QuadCollider, Shape};
use crate::quadtree::Quadtree;
use crate::{
    enemy::Enemy,
    gun::Bullet,
    health::{Damage, Health},
};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemyQuadtree(Quadtree::new(Rect::from_center_size(
            Vec2::ZERO,
            // TODO: change to WORLD_SIZE when the world gets 'closed'
            Vec2::splat(WORLD_SIZE + 500.),
        ))))
        .add_systems(
            Update,
            (
                collide_enemy_bullet,
                update_enemy_quadtree.run_if(on_timer(Duration::from_secs_f32(
                    ENEMY_QUADTREE_REFRESH_RATE_SECS,
                ))),
            )
                .run_if(in_state(GameState::Running)),
        );
    }
}

fn update_enemy_quadtree(
    mut qtree: ResMut<EnemyQuadtree>,
    enemy_query: Query<(Entity, &Transform, &ColliderShape), With<Enemy>>,
) {
    qtree.clear();
    let enemies = enemy_query
        .iter()
        .map(|(ent, transf, shape)| QuadVal::new(ent, transf.translation.truncate(), **shape))
        .collect::<Vec<_>>();

    if !enemies.is_empty() {
        qtree.insert_many(&enemies);
    }
}

#[allow(clippy::type_complexity)]
fn collide_enemy_bullet(
    qtree: Res<EnemyQuadtree>,
    bullet_query: Query<(&Transform, &Damage, &ColliderShape), With<Bullet>>,
    mut enemy_query: Query<(&mut Health, &Transform), With<Enemy>>,
) {
    if bullet_query.is_empty() || enemy_query.is_empty() {
        return;
    }

    bullet_query
        .iter()
        .for_each(|(bullet_transf, bullet_dmg, bullet_shape)| {
            // Query the quadtree in a 64px box around bullet.
            let near_enemy_colliders = qtree.query(Rect::from_center_size(
                bullet_transf.translation.truncate(),
                Vec2::splat(64.),
            ));

            for &near_enemy_collider in near_enemy_colliders.iter() {
                if let Ok((mut enemy_hp, enemy_transf)) =
                    enemy_query.get_mut(near_enemy_collider.entity)
                {
                    let enemy_quad_coll = QuadCollider::new(
                        enemy_transf.translation.truncate(),
                        *near_enemy_collider.shape,
                    );
                    let bullet_quad_coll =
                        QuadCollider::new(bullet_transf.translation.truncate(), **bullet_shape);
                    if enemy_quad_coll.intersects(bullet_quad_coll) {
                        enemy_hp.current = enemy_hp.current.saturating_sub(**bullet_dmg);
                    }
                }
            }
        });
}

#[derive(Resource, DerefMut, Deref)]
pub struct EnemyQuadtree(pub Quadtree<QuadVal>);

#[derive(Clone, PartialEq)]
pub struct QuadVal {
    pub entity: Entity,
    pub pos: Vec2,
    pub shape: ColliderShape,
}

#[derive(Component, Clone, Copy, PartialEq, Deref, DerefMut)]
pub struct ColliderShape(pub Shape);

impl QuadVal {
    pub fn new(entity: Entity, pos: Vec2, shape: Shape) -> Self {
        let shape = ColliderShape(shape);
        QuadVal { entity, pos, shape }
    }
}

impl AsQuadCollider for QuadVal {
    fn as_quad_collider(&self) -> QuadCollider {
        QuadCollider {
            pos: self.pos,
            shape: *self.shape,
        }
    }
}
