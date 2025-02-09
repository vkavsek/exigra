use bevy::{prelude::*, window::PrimaryWindow};

use crate::prelude::*;

/// Loads all the assets into `Resources` and advances the GameState,
/// then it keeps track of and updates all the `Resources`.
pub struct ResourcePlugin;

impl Plugin for ResourcePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobTextAtlases::default())
            .insert_resource(CursorPos(None))
            .insert_resource(ClearColor(BG_COLOR))
            .insert_resource(EnemyNum(0))
            .add_systems(OnEnter(GameState::AssetLoad), load_resources)
            .add_systems(
                Update,
                update_cursor_pos.run_if(in_state(GameState::GameRun)),
            );
    }
}

/// Tracks the number of enemies currently spawned in game.
#[derive(Resource, Debug, Default, DerefMut, Deref)]
pub struct EnemyNum(pub usize);

#[derive(Resource, Debug, Default)]
pub struct GlobTextAtlases {
    pub player: Option<TextureAtlasHandle>,
    pub common: Option<TextureAtlasHandle>,
    pub foliage: Option<TextureAtlasHandle>,
}

#[derive(Debug, Clone)]
pub struct TextureAtlasHandle {
    pub layout: Handle<TextureAtlasLayout>,
    pub image: Handle<Image>,
}
impl TextureAtlasHandle {
    fn new(layout: Handle<TextureAtlasLayout>, image: Handle<Image>) -> Self {
        TextureAtlasHandle { layout, image }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct CursorPos(pub Option<Vec2>);

fn load_resources(
    mut text_atlases: ResMut<GlobTextAtlases>,
    mut texture_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut next_state: ResMut<NextState<GameState>>,
    asset_serv: Res<AssetServer>,
) {
    let player_txtr = asset_serv.load(SPRITESH_PLAYER_PATH);
    let common_txtr = asset_serv.load(SPRITESH_COMMON_PATH);
    let foliage_txtr = asset_serv.load(SPRITESH_FOLIAGE_PATH);

    let player_layout = TextureAtlasLayout::from_grid(
        SPRITESH_PLAYER_TILESIZE,
        SPRITESH_PLAYER_COL,
        SPRITESH_PLAYER_ROW,
        None,
        None,
    );
    let player_ta_layout = texture_layouts.add(player_layout);
    let player_atlas_handle = TextureAtlasHandle::new(player_ta_layout, player_txtr);
    text_atlases.player = Some(player_atlas_handle);

    let common_layout = TextureAtlasLayout::from_grid(
        SPRITESH_COMMON_TILESIZE,
        SPRITESH_COMMON_COL,
        SPRITESH_COMMON_ROW,
        None,
        None,
    );
    let common_ta_layout = texture_layouts.add(common_layout);
    let common_atlas_handle = TextureAtlasHandle::new(common_ta_layout, common_txtr);
    text_atlases.common = Some(common_atlas_handle);

    let foliage_layout = TextureAtlasLayout::from_grid(
        SPRITESH_FOLIAGE_TILESIZE,
        SPRITESH_FOLIAGE_COL,
        SPRITESH_FOLIAGE_ROW,
        None,
        None,
    );
    let foliage_ta_layout = texture_layouts.add(foliage_layout);
    let foliage_atlas_handle = TextureAtlasHandle::new(foliage_ta_layout, foliage_txtr);
    text_atlases.foliage = Some(foliage_atlas_handle);

    next_state.set(GameState::MainMenu);
}

fn update_cursor_pos(
    mut cursor_pos: ResMut<CursorPos>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    cam_query: Query<(&Camera, &GlobalTransform)>,
) {
    let win = window_query.single();
    let (cam, cam_transform) = cam_query.single();

    let Some(win_cpos) = win
        .cursor_position()
        .and_then(|cursor| cam.viewport_to_world_2d(cam_transform, cursor).ok())
    else {
        return;
    };

    cursor_pos.0 = Some(win_cpos);
}
