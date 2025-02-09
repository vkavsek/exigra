//! Contains components that are reused across multiple plugins
//!
//! Currently contains HealthPlugin, to be removed
//! Current implementation of [`HealthPlugin`] handles despawning entities
//! when their health reaches zero.

use bevy::prelude::*;

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
