use std::time::Duration;

use crate::collision::ColliderShape;
use crate::components::Health;
use crate::prelude::*;
use crate::quadtree::quad_collider::Shape;
use crate::score::ScoreAccumulator;
use crate::{animation::AnimationTimer, resources::GlobTextAtlases};

use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameInit), spawn_player)
            .add_systems(
                Update,
                (handle_player_input, tick_player_iframes_timer)
                    .run_if(in_state(GameState::GameRun)),
            );
    }
}

// Components
#[derive(Component)]
#[require(
    Transform,
    Health(|| Health::new(50)),
    Sprite,
    AnimationTimer,
    PlayerState,
    ScoreAccumulator(|| ScoreAccumulator(0)),
    IFramesTimer(|| IFramesTimer::new_from_secs_f32(PLAYER_IFRAMES_DURATION_SECS)),
    ColliderShape(|| ColliderShape(Shape::Quad(Rectangle::new(11., 13.))))
)]
pub struct Player;

/// Used for player animation.
#[derive(Component, Default, PartialEq, Eq)]
pub enum PlayerState {
    #[default]
    Stop,
    Move,
}

#[derive(Component, DerefMut, Deref, Clone)]
pub struct IFramesTimer(pub Timer);
impl IFramesTimer {
    /// Cretes a new IFRAME timer, by default it is set to finished aka no IFRAMES active
    fn new_from_secs_f32(secs: f32) -> Self {
        let mut t = IFramesTimer(Timer::new(Duration::from_secs_f32(secs), TimerMode::Once));
        t.set_elapsed(Duration::from_secs_f32(secs));
        t
    }
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

fn tick_player_iframes_timer(mut iframe_query: Query<&mut IFramesTimer>, time: Res<Time>) {
    let mut iframe_timer = iframe_query.single_mut();
    iframe_timer.tick(time.delta());
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
