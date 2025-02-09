//! Contains the [`Health`] and [`Damage`] components + the [`HealthPlugin`]
//!
//! Current implementation of [`HealthPlugin`] handles despawning entities
//! when their health reaches zero.

// TODO: Remove the health plugin, add health and dmg to components module
// Player and Enemy plugins should handle player and enemy health and despawning.
use bevy::prelude::*;

use crate::{player::Player, prelude::*};

/// Current implementation will despawn entities if their health reaches 0.
/// (`Last` in `Main` schedule)
pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Last,
            handle_non_player_zero_health.run_if(in_state(GameState::GameRun)),
        );
    }
}

#[derive(Component, Default, Debug, Clone)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

impl Health {
    pub fn new(n: u32) -> Self {
        Self { current: n, max: n }
    }

    /// apply heal
    pub fn heal(&mut self, val: u16) {
        // ensure we don't exceed max allowed health when healing.
        self.current = self.max.min(self.current + val as u32);
    }

    /// apply dmg
    pub fn dmg(&mut self, val: u32) {
        // ensure we don't overflow
        self.current = self.current.saturating_sub(val);
    }
}

#[derive(Component, Debug, Deref, DerefMut, Default, Clone)]
pub struct Damage(pub u32);

fn handle_non_player_zero_health(
    mut commands: Commands,
    ent_query: Query<(Entity, &Health), (Changed<Health>, Without<Player>)>,
) {
    for (ent, hp) in ent_query.iter() {
        if hp.current == 0 {
            commands.entity(ent).despawn_recursive();
        }
    }
}
