use std::env;

use bevy::prelude::*;
pub(crate) use iyes_loopless::prelude::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum GameState {
    InGame,
    LoadingLevel,
    MainMenu,
    Paused,
    Splash,
}

impl Default for GameState {
    fn default() -> Self {
        // game state can be controlled during cargo run
        //
        // ❯ GAMESTATE=InGame cargo run
        if let Ok(state) = env::var("GAMESTATE") {
            match state.as_ref() {
                "InGame" => Self::InGame,
                "LoadingLevel" => Self::LoadingLevel,
                "MainMenu" => Self::MainMenu,
                "Splash" => Self::Splash,
                _ => panic!("unrecognised game state {}", state),
            }
        } else {
            Self::Splash
        }
    }
}

pub(crate) struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(GameState::default());
    }
}

pub(crate) fn despawn_with<T: Component>(mut cmd: Commands, q: Query<Entity, With<T>>) {
    for entity in q.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}
