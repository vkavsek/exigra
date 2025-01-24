use bevy::prelude::*;

use crate::prelude::*;
use crate::{
    enemy::Enemy,
    gun::Bullet,
    health::{Damage, Health},
};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            collide_enemy_bullet.run_if(in_state(GameState::Running)),
        );
    }
}

#[allow(clippy::type_complexity)]
fn collide_enemy_bullet(
    mut enemy_query: Query<(&mut Health, &Transform), (With<Enemy>, Without<Bullet>)>,
    mut bullet_query: Query<(&mut Health, &Transform, &Damage), With<Bullet>>,
) {
    if bullet_query.is_empty() || enemy_query.is_empty() {
        return;
    }

    bullet_query
        .iter_mut()
        .for_each(|(mut bullet_hp, bullet_transf, bullet_dmg)| {
            for (mut enemy_hp, enemy_transf) in enemy_query.iter_mut() {
                if bullet_transf
                    .translation
                    .truncate()
                    .distance_squared(enemy_transf.translation.truncate())
                    <= 80.
                {
                    enemy_hp.current = enemy_hp.current.saturating_sub(**bullet_dmg);
                    bullet_hp.current = 0;
                }
            }
        });
}
