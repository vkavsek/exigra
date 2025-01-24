use bevy::prelude::*;

use tutgame::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resizable: true,
                        focused: true,
                        present_mode: bevy::window::PresentMode::Immediate,
                        // FIXME: change from displaying on second monitor.
                        mode: bevy::window::WindowMode::BorderlessFullscreen(
                            MonitorSelection::Index(1),
                        ),

                        ..default()
                    }),
                    ..default()
                }),
        )
        // State
        .init_state::<GameState>()
        // Internal plugins
        .add_plugins((
            GuiPlugin,
            ResourcePlugin,
            WorldPlugin,
            CamPlugin,
            PlayerPlugin,
            EnemyPlugin,
            GunPlugin,
            AnimPlugin,
            HealthPlugin,
            CollisionPlugin,
        ))
        .run();
}
