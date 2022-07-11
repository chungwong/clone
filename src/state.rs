use bevy::prelude::*;
use iyes_loopless::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    MainMenu,
    InGame,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(GameState::MainMenu)
        )
    }
}
