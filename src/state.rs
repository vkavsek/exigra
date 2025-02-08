use bevy::prelude::States;

/// Represents the current state of the game.
/// `AssetLoad` —> `Init` —> `Running`
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum GameState {
    #[default]
    AssetLoad,
    MainMenu,
    GameInit,
    GameRun,
}
