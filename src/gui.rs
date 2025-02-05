use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::{enemy::Enemy, gun::Bullet, prelude::GameState, resources::EnemyNum};

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

#[derive(Component)]
#[require(TextSpan)]
struct EnemyPosText;

#[derive(Component)]
#[require(TextSpan)]
struct BulletPosText;

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
        ))
        .with_child((
            TextSpan::default(),
            TextFont::default().with_font_size(FONT_SIZE),
            EnemyPosText,
        ));

    commands
        .spawn((
            Text::new("BULLETS: "),
            TextFont::default().with_font_size(FONT_SIZE),
        ))
        .with_child((
            TextSpan::default(),
            TextFont::default().with_font_size(FONT_SIZE),
            BulletPosText,
        ));
}

#[allow(clippy::type_complexity)]
fn update_debug_text(
    mut set: ParamSet<(
        Query<&mut TextSpan, With<FpsText>>,
        Query<&mut TextSpan, With<EnemyNumText>>,
        Query<&mut TextSpan, With<EnemyPosText>>,
        Query<&mut TextSpan, With<BulletPosText>>,
        Query<&Transform, With<Enemy>>,
        Query<&Transform, With<Bullet>>,
    )>,
    num_of_enemies: Res<EnemyNum>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let mut fps_span = set.p0();
    let mut fps_span = fps_span.single_mut();
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps) = fps.smoothed() {
            **fps_span = format!("{fps:.2}");
        }
    }

    let mut enemy_num_span = set.p1();
    let mut enemy_num_span = enemy_num_span.single_mut();
    **enemy_num_span = num_of_enemies.to_string();

    let enemy_pos = set.p4().get_single().copied().unwrap_or_default();
    let mut enemy_pos_span = set.p1();
    let mut enemy_pos_span = enemy_pos_span.single_mut();
    **enemy_pos_span = format!("{:?}", enemy_pos);

    let bullet_pos = set.p5().get_single().copied().unwrap_or_default();
    let mut bullet_pos_span = set.p3();
    let mut bullet_pos_span = bullet_pos_span.single_mut();
    **bullet_pos_span = format!("{bullet_pos:?}");
}
