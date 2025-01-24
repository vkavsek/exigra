use bevy::prelude::*;

use crate::prelude::*;
use crate::{
    enemy::Enemy,
    gun::Gun,
    player::{Player, PlayerState},
    resources::CursorPos,
};

pub struct AnimPlugin;

impl Plugin for AnimPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            // tick first, then run all the animation systems
            (
                animation_timer_tick,
                (animate_player, animate_gun, animate_enemy),
            )
                .chain()
                .run_if(in_state(GameState::Running)),
        );
    }
}

#[derive(Component, Default, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);
impl AnimationTimer {
    pub fn new_from_secs(duration: f32) -> Self {
        AnimationTimer(Timer::from_seconds(duration, TimerMode::Repeating))
    }
}

fn animation_timer_tick(mut at_query: Query<&mut AnimationTimer>, time: Res<Time>) {
    // Should this be parallel?
    at_query.iter_mut().for_each(|mut at| {
        at.tick(time.delta());
    });
}

fn animate_player(
    mut player_query: Query<(&mut Sprite, &PlayerState, &Transform, &AnimationTimer), With<Player>>,
    cursor_pos: Res<CursorPos>,
) {
    if player_query.is_empty() {
        return;
    }

    let (mut player_sprite, player_state, player_transf, anim_timer) = player_query.single_mut();

    // Animate index
    if anim_timer.just_finished() {
        if let Some(ta) = player_sprite.texture_atlas.as_mut() {
            ta.index = match player_state {
                PlayerState::Stop => 0,
                PlayerState::Move => (ta.index + 1) % 8,
            }
        }
    }

    if let Some(cursor_pos) = cursor_pos.0 {
        let player_pos = player_transf.translation;
        player_sprite.flip_x = cursor_pos.x < player_pos.x;
    }
}

#[allow(clippy::type_complexity)]
fn animate_enemy(
    mut enemy_query: Query<
        (&mut Sprite, &Transform, &AnimationTimer),
        (With<Enemy>, Without<Player>),
    >,
    player_query: Query<&Transform, With<Player>>,
) {
    if enemy_query.is_empty() {
        return;
    }

    let player_pos = player_query.single().translation;

    enemy_query
        .iter_mut()
        .for_each(|(mut enemy_sprite, enemy_transf, anim_timer)| {
            if anim_timer.just_finished() {
                if let Some(ta) = enemy_sprite.texture_atlas.as_mut() {
                    ta.index = (ta.index + 1) % 4;
                }
            }

            let enemy_pos = enemy_transf.translation;
            enemy_sprite.flip_x = player_pos.x < enemy_pos.x;
        });
}

fn animate_gun(
    mut gun_query: Query<(&mut Sprite, &Transform), With<Gun>>,
    cursor_pos: Res<CursorPos>,
) {
    if gun_query.is_empty() {
        return;
    }

    let (mut gun_sprite, gun_transf) = gun_query.single_mut();
    if let Some(cursor_pos) = cursor_pos.0 {
        gun_sprite.flip_y = cursor_pos.x < gun_transf.translation.x;
    }
}
