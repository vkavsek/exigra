use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::{prelude::GameState, resources::EnemyNum};

const FONT_SIZE: f32 = 30.0;

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(OnEnter(GameState::Init), spawn_debug_text)
            .add_systems(
                FixedPostUpdate,
                update_debug_text.run_if(in_state(GameState::Running)),
            );
    }
}

#[derive(Component)]
#[require(TextSpan)]
struct FpsText;

#[derive(Component)]
#[require(TextSpan)]
struct EnemyNumText;

fn spawn_debug_text(mut commands: Commands) {
    commands
        .spawn((
            Text::new("FPS: "),
            TextFont::default().with_font_size(FONT_SIZE),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(10.),
                ..default()
            },
        ))
        .with_child((
            TextSpan::default(),
            TextFont::default().with_font_size(FONT_SIZE),
            FpsText,
        ));

    commands
        .spawn((
            Text::new("ENEMIES: "),
            TextFont::default().with_font_size(FONT_SIZE),
        ))
        .with_child((
            TextSpan::default(),
            TextFont::default().with_font_size(FONT_SIZE),
            EnemyNumText,
        ));
}

fn update_debug_text(
    mut fps_text_query: Query<&mut TextSpan, With<FpsText>>,
    mut enemy_text_query: Query<&mut TextSpan, (With<EnemyNumText>, Without<FpsText>)>,
    num_of_enemies: Res<EnemyNum>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let mut fps_span = fps_text_query.single_mut();
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps) = fps.smoothed() {
            **fps_span = format!("{fps:.2}")
        }
    }

    let mut enemy_num_span = enemy_text_query.single_mut();
    **enemy_num_span = num_of_enemies.to_string()
}
