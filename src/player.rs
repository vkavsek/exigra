use crate::prelude::*;
use crate::{animation::AnimationTimer, resources::GlobTextAtlases};

use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameInit), spawn_player)
            .add_systems(
                Update,
                handle_player_input.run_if(in_state(GameState::GameRun)),
            );
    }
}

// Components
#[derive(Component)]
#[require(Transform, Sprite, AnimationTimer, PlayerState)]
pub struct Player;

#[derive(Component, Default, PartialEq, Eq)]
pub enum PlayerState {
    #[default]
    Stop,
    Move,
}

fn spawn_player(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    text_atlases: Res<GlobTextAtlases>,
) {
    let image = text_atlases.player.clone().unwrap().image;
    let layout = text_atlases.player.clone().unwrap().layout;

    // Player
    commands.spawn((
        Sprite::from_atlas_image(image, TextureAtlas { layout, index: 0 }),
        Transform::from_translation(Vec3::new(0., 0., 50.)),
        AnimationTimer::new_from_secs(PLAYER_ANIM_INTERVAL_SECS),
        Player,
    ));

    next_state.set(GameState::GameRun)
}

fn handle_player_input(
    mut player_query: Query<(&mut Transform, &mut PlayerState), With<Player>>,
    kbd_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let (mut player_transf, mut player_state) = player_query.single_mut();

    let up = kbd_input.pressed(KeyCode::KeyW) || kbd_input.pressed(KeyCode::ArrowUp);
    let down = kbd_input.pressed(KeyCode::KeyS) || kbd_input.pressed(KeyCode::ArrowDown);
    let left = kbd_input.pressed(KeyCode::KeyA) || kbd_input.pressed(KeyCode::ArrowLeft);
    let right = kbd_input.pressed(KeyCode::KeyD) || kbd_input.pressed(KeyCode::ArrowRight);

    let mut dir_delta = Vec2::ZERO;
    if up {
        dir_delta.y += 1.;
    }
    if down {
        dir_delta.y -= 1.;
    }
    if left {
        dir_delta.x -= 1.;
    }
    if right {
        dir_delta.x += 1.;
    }
    dir_delta = dir_delta.normalize_or_zero();

    if dir_delta.length() > 0.0 {
        player_transf.translation +=
            Vec3::new(dir_delta.x, dir_delta.y, 0.) * Vec3::splat(PLAYER_SPEED) * time.delta_secs();

        *player_state = PlayerState::Move;
    } else {
        *player_state = PlayerState::Stop;
    }
}
