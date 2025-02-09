//! All the modules except for [`components`], [`state`] and [`quadtree`] contain their own plugin.

#![allow(clippy::type_complexity)]

pub mod prelude;

// generic components
pub mod components;
// generic resources and asset loading
pub mod resources;
pub mod score;
pub mod state;
// world decorations etc.
pub mod world;

pub mod camera;
pub mod gui;

pub mod collision;
pub mod quadtree;

pub mod animation;
pub mod enemy;
pub mod gun;
pub mod player;
