//! Contains [`ScorePlugin`] that provides a [`Score`] resource.
//! The player entity should have a [`ScoreAccumulator`] component associated with it.
//! It periodically looks at all entities that have a [`ScoreAccumulator`] and adds it to the
//! [`Score`], while reseting `ScoreAccumulator`.
//!
//! Also contains a [`Worth`] component that is intended to be added to all the things that should
//! be scored.

use bevy::prelude::*;

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Score::default())
            .add_systems(FixedUpdate, add_score_accum_to_score);
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct Score(pub u64);

#[derive(Component, Deref, DerefMut)]
pub struct Worth(pub u64);

#[derive(Component, Deref, DerefMut)]
pub struct ScoreAccumulator(pub u64);

fn add_score_accum_to_score(
    mut score_accum_query: Query<&mut ScoreAccumulator, Changed<ScoreAccumulator>>,
    mut score: ResMut<Score>,
) {
    if score_accum_query.is_empty() {
        return;
    }

    for mut add_to_score in score_accum_query.iter_mut() {
        **score += **add_to_score;
        **add_to_score = 0;
    }
}
