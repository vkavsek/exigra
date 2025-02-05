use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use crate::prelude::*;
use crate::quadtree::quad_val::{AsQuadVal, QuadVal, Shape};
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
        .map(|(ent, transf, shape)| QuadCollider::new(ent, transf.translation.truncate(), **shape))
        .collect::<Vec<_>>();

    if !enemies.is_empty() {
        qtree.insert_many(&enemies);
    }
}

#[allow(clippy::type_complexity)]
fn collide_enemy_bullet(
    qtree: Res<EnemyQuadtree>,
    bullet_query: Query<(&Transform, &Damage, &ColliderShape), With<Bullet>>,
    mut enemy_query: Query<(&mut Health, &mut Sprite, &Transform), With<Enemy>>,
) {
    if bullet_query.is_empty() || enemy_query.is_empty() {
        return;
    }

    bullet_query
        .iter()
        .for_each(|(bullet_transf, bullet_dmg, bullet_shape)| {
            if let Some(near_enemy_collider) = qtree.nearest(bullet_transf.translation.truncate()) {
                if let Ok((mut enemy_hp, mut sprite, enemy_transf)) =
                    enemy_query.get_mut(near_enemy_collider.entity)
                {
                    sprite.color = Color::srgb(1., 0., 0.);
                    let enemy_quad_coll = QuadVal::new(
                        enemy_transf.translation.truncate(),
                        *near_enemy_collider.shape,
                    );
                    let bullet_quad_coll =
                        QuadVal::new(bullet_transf.translation.truncate(), **bullet_shape);
                    if enemy_quad_coll.intersects(bullet_quad_coll) {
                        enemy_hp.current = enemy_hp.current.saturating_sub(**bullet_dmg);
                    }
                }
            }

            // for (mut enemy_hp, enemy_transf) in enemy_query.iter_mut() {
            //     if bullet_transf
            //         .translation
            //         .truncate()
            //         .distance_squared(enemy_transf.translation.truncate())
            //         <= 80.
            //     {
            //         enemy_hp.current = enemy_hp.current.saturating_sub(**bullet_dmg);
            //     }
            // }
        });
}

#[derive(Resource, DerefMut, Deref)]
pub struct EnemyQuadtree(pub Quadtree<QuadCollider>);

#[derive(Clone, PartialEq)]
pub struct QuadCollider {
    pub entity: Entity,
    pub pos: Vec2,
    pub shape: ColliderShape,
}

#[derive(Component, Clone, Copy, PartialEq, Deref, DerefMut)]
pub struct ColliderShape(pub Shape);

impl QuadCollider {
    pub fn new(entity: Entity, pos: Vec2, shape: Shape) -> Self {
        let shape = ColliderShape(shape);
        QuadCollider { entity, pos, shape }
    }
}

impl AsQuadVal for QuadCollider {
    fn as_quad_val(&self) -> QuadVal {
        QuadVal {
            pos: self.pos,
            shape: *self.shape,
        }
    }
}
