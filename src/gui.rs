#![allow(clippy::type_complexity)]

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::{
    components::Health, player::Player, prelude::GameState, resources::EnemyNum, score::Score,
};

const FONT_SIZE: f32 = 30.0;

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
            .add_systems(
                OnExit(GameState::MainMenu),
                despawn_entities::<OnMenuScreen>,
            )
            .add_systems(
                Update,
                (handle_button_color, handle_menu_button_action)
                    .run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(OnEnter(GameState::GameInit), spawn_debug_text)
            .add_systems(
                FixedPostUpdate,
                (update_debug_text.run_if(in_state(GameState::GameRun)),),
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
struct ScoreText;

#[derive(Component)]
#[require(TextSpan)]
struct PlayerHpText;

#[derive(Component)]
#[require(TextSpan)]
struct EnemyPosText;

#[derive(Component)]
#[require(TextSpan)]
struct BulletPosText;

#[derive(Component)]
struct OnGameScreen;

#[derive(Component)]
struct OnMenuScreen;

#[derive(Component)]
enum MenuButtonAction {
    Play,
    Exit,
}

const TITLE_BG_CD: Color = Color::srgb(0.32, 0.23, 0.42);
const PRESSED_BUTTON_BG: Color = Color::srgb(0.32, 0.23, 0.72);
const HOVERED_BUTTON_BG: Color = Color::srgb(0.05, 0.23, 0.62);
const BUTTON_BG: Color = Color::srgb(0.02, 0.23, 0.42);

fn spawn_main_menu(mut commands: Commands) {
    let button_node = Node {
        padding: UiRect::all(Val::Px(20.)),
        ..default()
    };
    let title_node = Node {
        padding: UiRect::all(Val::Px(20.)),
        ..default()
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceAround,
                ..default()
            },
            OnMenuScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn((BackgroundColor(TITLE_BG_CD), title_node))
                .with_child((
                    Text::new("MENU"),
                    TextFont::default().with_font_size(FONT_SIZE + 20.),
                    TextColor(Color::srgb(0.674, 0.229, 0.732)),
                ));

            parent
                .spawn((button_node.clone(), Button, MenuButtonAction::Play))
                .with_child((
                    Text::new("Play"),
                    TextFont::default().with_font_size(FONT_SIZE),
                ));

            parent
                .spawn((button_node, Button, MenuButtonAction::Exit))
                .with_child((
                    Text::new("Exit"),
                    TextFont::default().with_font_size(FONT_SIZE),
                ));
        });
}

fn spawn_debug_text(mut commands: Commands) {
    let fps_text = commands
        .spawn((
            Text::new("FPS: "),
            TextFont::default().with_font_size(FONT_SIZE),
            Node::default(),
        ))
        .with_child((TextFont::default().with_font_size(FONT_SIZE), FpsText))
        .id();

    let enemies_text = commands
        .spawn((
            Text::new("ENEMIES: "),
            TextFont::default().with_font_size(FONT_SIZE),
            Node::default(),
        ))
        .with_child((TextFont::default().with_font_size(FONT_SIZE), EnemyNumText))
        .id();

    let player_hp_text = commands
        .spawn((
            Text::new("PLAYER_HP: "),
            TextFont::default().with_font_size(FONT_SIZE),
            Node::default(),
        ))
        .with_child((TextFont::default().with_font_size(FONT_SIZE), PlayerHpText))
        .id();

    let score_text = commands
        .spawn((
            Text::new("SCORE: "),
            TextFont::default().with_font_size(FONT_SIZE),
            Node::default(),
        ))
        .with_child((TextFont::default().with_font_size(FONT_SIZE), ScoreText))
        .id();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::End,
                ..Default::default()
            },
            OnGameScreen,
        ))
        .add_children(&[fps_text, enemies_text, player_hp_text, score_text]);
}

fn update_debug_text(
    mut set: ParamSet<(
        Query<&mut TextSpan, With<FpsText>>,
        Query<&mut TextSpan, With<EnemyNumText>>,
        Query<&mut TextSpan, With<PlayerHpText>>,
        Query<&mut TextSpan, With<ScoreText>>,
    )>,
    player_query: Query<&Health, (With<Player>, Changed<Health>)>,
    num_of_enemies: Res<EnemyNum>,
    score: Res<Score>,
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

    if let Ok(player_hp) = player_query.get_single() {
        let mut hp_span = set.p2();
        let mut hp_span = hp_span.single_mut();
        **hp_span = format!("{} / {}", player_hp.current, player_hp.max);
    }

    let mut score_span = set.p3();
    let mut score_span = score_span.single_mut();
    **score_span = score.to_string();
}

// This system handles changing all buttons color based on mouse interaction
fn handle_button_color(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background_color) in interaction_query.iter_mut() {
        *background_color = match *interaction {
            Interaction::Pressed => PRESSED_BUTTON_BG.into(),
            Interaction::Hovered => HOVERED_BUTTON_BG.into(),
            Interaction::None => BUTTON_BG.into(),
        }
    }
}

fn handle_menu_button_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
    mut app_exit_event: EventWriter<AppExit>,
) {
    for (interaction, button_action) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            match button_action {
                MenuButtonAction::Play => game_state.set(GameState::GameInit),
                MenuButtonAction::Exit => {
                    app_exit_event.send(AppExit::Success);
                }
            };
        }
    }
}

/// Generic despawn entities function
/// Despawns all entities that have a `T` component.
fn despawn_entities<T: Component>(mut commands: Commands, entities: Query<Entity, With<T>>) {
    for ent in entities.iter() {
        commands.entity(ent).despawn_recursive();
    }
}
