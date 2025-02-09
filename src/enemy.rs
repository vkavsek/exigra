use std::f32::consts::PI;
use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use rand::Rng;

use crate::collision::ColliderShape;
use crate::prelude::*;
use crate::quadtree::quad_collider::Shape;
use crate::resources::EnemyNum;
use crate::score::{ScoreAccumulator, Worth};
use crate::{
    animation::AnimationTimer, components::Damage, components::Health, player::Player,
    resources::GlobTextAtlases,
};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        // track number of enemies first, to account for all the enemies that were despawned in
        // the previous iteration.
        app.add_systems(
            First,
            track_num_of_enemies.run_if(in_state(GameState::GameRun)),
        )
        .add_systems(
            Update,
            (
                spawn_enemies.run_if(on_timer(Duration::from_secs_f32(ENEMY_SPAWN_INTERVAL_SECS))),
                update_enemy_transform,
            )
                // spawn enemies first, then run all the updating systems
                .chain()
                .run_if(in_state(GameState::GameRun)),
        )
        .add_systems(
            Last,
            handle_enemy_death.run_if(in_state(GameState::GameRun)),
        );
    }
}

#[derive(Component)]
#[require(
    Transform,
    Sprite,
    AnimationTimer,
    Health(|| Health::new(10)),
    Damage(|| Damage(5)),
    Worth(|| Worth(1)),
    ColliderShape(|| ColliderShape( Shape::Quad( Rectangle::from_size(Vec2::splat(8.0)))))
)]
pub struct Enemy;

fn spawn_enemies(
    mut commands: Commands,
    mut num_of_enemies: ResMut<EnemyNum>,
    text_atlases: Res<GlobTextAtlases>,
    player_query: Query<&Transform, With<Player>>,
) {
    let num_enemies = **num_of_enemies;
    if num_enemies >= ENEMY_MAX_INSTANCES {
        return;
    }

    let enemy_spawn_count = (ENEMY_MAX_INSTANCES - num_enemies).min(ENEMY_SPAWN_PER_INTERVAL);
    **num_of_enemies += enemy_spawn_count;

    let player_pos = player_query.single().translation.truncate();
    let mut rng = rand::thread_rng();

    let mut get_random_around = |pos: Vec2| {
        let angle = rng.gen_range(0.0..PI * 2.0);
        let dist = rng.gen_range(200.0..2000.);

        let mut res = pos + Vec2::from_angle(angle) * dist;
        let whalf = WORLD_SIZE * 0.5;
        res.x = res.x.clamp(-whalf, whalf);
        res.y = res.y.clamp(-whalf, whalf);
        res
    };

    let enemy_entities = (0..enemy_spawn_count)
        .map(|_| {
            let layout = text_atlases.common.clone().unwrap().layout;
            let image = text_atlases.common.clone().unwrap().image;

            (
                Sprite::from_atlas_image(image, TextureAtlas { layout, index: 0 }),
                Transform::from_translation(get_random_around(player_pos).extend(100.0)),
                AnimationTimer::new_from_secs(ENEMY_ANIM_INTERVAL_SECS),
                Enemy,
            )
        })
        .collect::<Vec<_>>();

    commands.spawn_batch(enemy_entities);
}

fn update_enemy_transform(
    mut enemy_query: Query<&mut Transform, (With<Enemy>, Without<Player>)>,
    player_query: Query<&Transform, With<Player>>,
    time: Res<Time>,
) {
    if player_query.is_empty() || enemy_query.is_empty() {
        return;
    }

    let player_pos = player_query.single().translation.truncate();

    enemy_query.iter_mut().for_each(|mut etransf| {
        let dir = (player_pos - etransf.translation.truncate()).normalize_or_zero();

        let enemy_vel = dir.extend(0.0) * ENEMY_SPEED * time.delta_secs();
        etransf.translation += enemy_vel;
    });
}

fn track_num_of_enemies(mut num_of_enemies: ResMut<EnemyNum>, enemy_query: Query<&Enemy>) {
    **num_of_enemies = enemy_query.iter().len();
}

fn handle_enemy_death(
    mut commands: Commands,
    mut player_query: Query<&mut ScoreAccumulator, With<Player>>,
    enemy_query: Query<(Entity, &Health, &Worth), (Changed<Health>, With<Enemy>)>,
) {
    let mut player_score_accum = player_query.single_mut();
    for (ent, hp, worth) in enemy_query.iter() {
        if hp.current == 0 {
            **player_score_accum += **worth;
            commands.entity(ent).despawn();
        }
    }
}
