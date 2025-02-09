use std::time::Duration;

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use crate::player::{IFramesTimer, Player};
use crate::prelude::*;
use crate::quadtree::quad_collider::{AsQuadCollider, QuadCollider, Shape};
use crate::quadtree::Quadtree;
use crate::{
    components::{Damage, Health},
    enemy::Enemy,
    gun::Bullet,
};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemyQuadtree::default()).add_systems(
            Update,
            (
                collide_enemy_bullet,
                collide_enemy_player,
                update_enemy_quadtree.run_if(on_timer(Duration::from_secs_f32(
                    ENEMY_QUADTREE_REFRESH_RATE_SECS,
                ))),
            )
                .run_if(in_state(GameState::GameRun)),
        );
    }
}

#[derive(Resource, DerefMut, Deref)]
pub struct EnemyQuadtree(pub Quadtree<QuadVal>);

impl Default for EnemyQuadtree {
    fn default() -> Self {
        EnemyQuadtree(Quadtree::new(Rect::from_center_size(
            Vec2::ZERO,
            // TODO: change to WORLD_SIZE when the world gets 'closed'
            Vec2::splat(WORLD_SIZE + 500.),
        )))
    }
}

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

fn update_enemy_quadtree(
    mut qtree: ResMut<EnemyQuadtree>,
    enemy_query: Query<(Entity, &Transform, &ColliderShape), With<Enemy>>,
) {
    let enemies = enemy_query
        .iter()
        .map(|(ent, transf, shape)| QuadVal::new(ent, transf.translation.truncate(), **shape))
        .collect::<Vec<_>>();

    if !enemies.is_empty() {
        // reset the EnemyQuadtree
        *qtree = EnemyQuadtree::default();
        qtree.insert_many(&enemies);
    }
}

fn collide_enemy_player(
    mut player_query: Query<
        (&mut Health, &mut IFramesTimer, &Transform, &ColliderShape),
        With<Player>,
    >,
    enemy_query: Query<(&Transform, &Damage), With<Enemy>>,
    qtree: Res<EnemyQuadtree>,
) {
    if enemy_query.is_empty() {
        return;
    }

    let (mut player_hp, mut iframes_timer, player_transf, player_shape) = player_query.single_mut();
    // if player is invulnerable don't do any processing.
    if !iframes_timer.finished() {
        return;
    }

    // Query the quadtree in a 256px box around player.
    let near_enemy_colliders = qtree.query(Rect::from_center_size(
        player_transf.translation.truncate(),
        Vec2::splat(256.),
    ));

    for &near_enemy_collider in near_enemy_colliders.iter() {
        if let Ok((enemy_transf, enemy_damage)) = enemy_query.get(near_enemy_collider.entity) {
            let enemy_quad_coll = QuadCollider::new(
                enemy_transf.translation.truncate(),
                *near_enemy_collider.shape,
            );
            let player_quad_coll =
                QuadCollider::new(player_transf.translation.truncate(), **player_shape);
            if enemy_quad_coll.intersects(player_quad_coll) && iframes_timer.finished() {
                player_hp.dmg(**enemy_damage);
                iframes_timer.reset();
            }
        }
    }
}

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
                        enemy_hp.dmg(**bullet_dmg);
                    }
                }
            }
        });
}
