//! Contains the [`Health`] and [`Damage`] components + the [`HealthPlugin`]
//!
//! Current implementation of [`HealthPlugin`] handles despawning entities
//! when their health reaches zero.

use bevy::prelude::*;

use crate::prelude::*;

/// Current implementation will despawn entities if their health reaches 0.
/// (`Last` in `Main` schedule)
pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Last,
            handle_zero_health.run_if(in_state(GameState::Running)),
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
}

#[derive(Component, Debug, Deref, DerefMut, Default, Clone)]
pub struct Damage(pub u32);

fn handle_zero_health(
    mut commands: Commands,
    ent_query: Query<(Entity, &Health), Changed<Health>>,
) {
    for (ent, hp) in ent_query.iter() {
        if hp.current == 0 {
            commands.entity(ent).despawn_recursive();
        }
    }
}
