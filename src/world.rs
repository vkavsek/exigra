//! Generic world entities.
//! Handles the initialization of the camera, the map, the decorations, etc.
use bevy::prelude::*;
use rand::Rng;

use crate::prelude::*;
use crate::resources::GlobTextAtlases;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Init), spawn_world_decor);
    }
}

#[derive(Component)]
#[require(Transform, Sprite)]
struct Decor;

fn spawn_world_decor(mut commands: Commands, text_atlases: Res<GlobTextAtlases>) {
    let mut rng = rand::thread_rng();

    let decor = (0..WORLD_DECOR_NUM)
        .map(|_| {
            let layout = text_atlases.common.clone().unwrap().layout;
            let image = text_atlases.common.clone().unwrap().image;
            let index = if rng.gen_bool(0.5) { 8 } else { 9 };

            let whalf = WORLD_SIZE * 0.5;
            let x = rng.gen_range(-whalf..whalf);
            let y = rng.gen_range(-whalf..whalf);
            // lower entities get rendered in front of the entities above to give perception of depth
            // returns 1..=2, entities lower on the map get a number closer to 2.
            let z_offset = -(-WORLD_SIZE + y - whalf) / 1000.0;

            (
                Sprite::from_atlas_image(image, TextureAtlas { layout, index }),
                Transform::from_xyz(x, y, 10. + z_offset),
                Decor,
            )
        })
        .collect::<Vec<_>>();

    commands.spawn_batch(decor);
}
