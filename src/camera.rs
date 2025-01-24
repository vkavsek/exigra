use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};

use crate::player::Player;
use crate::prelude::*;

pub struct CamPlugin;

impl Plugin for CamPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanCamPlugin)
            .add_systems(OnEnter(GameState::Init), spawn_cam)
            .add_systems(
                Update,
                cam_follow_player.run_if(in_state(GameState::Running)),
            );
    }
}

// Init
fn spawn_cam(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        PanCam {
            grab_buttons: vec![],
            ..default()
        },
        OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::WindowSize,
            scale: 0.25,
            ..OrthographicProjection::default_2d()
        },
        Msaa::Off,
    ));
}

/// Follow player in a smooth motion
fn cam_follow_player(
    mut cam_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    player_query: Query<&Transform, With<Player>>,
    time: Res<Time>,
) {
    let cam_pos = &mut cam_query.single_mut().translation;
    let player_pos = player_query.single().translation;
    let t = time.delta_secs();

    *cam_pos = cam_pos.lerp(player_pos.truncate().extend(cam_pos.z), t * 5.);
}
