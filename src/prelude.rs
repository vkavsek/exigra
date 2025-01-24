//! Contains re-exports & common constants.
//! Currently also holds the [`GameState`] and other components that don't fit anywhere else.

use bevy::{
    color::{Color, Srgba},
    math::UVec2,
    prelude::States,
};

// Re-export Plugins
pub use crate::{
    animation::AnimPlugin, camera::CamPlugin, collision::CollisionPlugin, enemy::EnemyPlugin,
    gui::GuiPlugin, gun::GunPlugin, health::HealthPlugin, player::PlayerPlugin,
    resources::ResourcePlugin, world::WorldPlugin,
};

// Colors
pub const BG_COLOR: Color = Color::Srgba(Srgba::new(0.078, 0.064, 0.15, 1.));

// Sprites
pub const SPRITESH_PLAYER_PATH: &str = "player_sprites.png";
pub const SPRITESH_PLAYER_COL: u32 = 4;
pub const SPRITESH_PLAYER_ROW: u32 = 2;
pub const SPRITESH_PLAYER_TILESIZE: UVec2 = UVec2::new(16, 32);

pub const SPRITESH_COMMON_PATH: &str = "combined_sprites.png";
pub const SPRITESH_COMMON_COL: u32 = 4;
pub const SPRITESH_COMMON_ROW: u32 = 4;
pub const SPRITESH_COMMON_TILESIZE: UVec2 = UVec2::splat(16);

// World
pub const WORLD_DECOR_NUM: u32 = 1500;
pub const WORLD_SIZE: f32 = 2000.;

// Player
pub const PLAYER_ANIM_INTERVAL_SECS: f32 = 0.1;
pub const PLAYER_SPEED: f32 = 100.;

// Enemy
pub const ENEMY_SPAWN_INTERVAL_SECS: f32 = 1.0;
pub const ENEMY_SPAWN_PER_SEC: usize = 10_000;
pub const ENEMY_ANIM_INTERVAL_SECS: f32 = 0.2;
pub const ENEMY_MAX_INSTANCES: usize = 500_000;
pub const ENEMY_SPEED: f32 = 30.;

// Gun
pub const BULLET_SPAWN_INTERVAL_SECS: f32 = 0.1;
pub const BULLET_LIFE_SECS: f32 = 2.0;
pub const BULLET_SPEED: f32 = 300.;

/// Represents the current state of the game.
/// `AssetLoad` —> `Init` —> `Running`
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum GameState {
    #[default]
    AssetLoad,
    Init,
    Running,
}
