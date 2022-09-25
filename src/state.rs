use std::env;

use bevy::prelude::*;
use global_state::{AddGlobalState, GlobalState};
pub(crate) use iyes_loopless::prelude::*;

// GlobalState will despawn all compoents on state exit, unless they are marked with Persistent
#[derive(Clone, Copy, Debug, Eq, Hash, GlobalState, PartialEq)]
pub(crate) enum GameState {
    AssetLoading,
    AudioMenu,
    InGame,
    MainMenu,
    MainMenuAssetLoading,
    OptionMenu,
    Paused,
    SaveMenu,
    Splash,
}

impl Default for GameState {
    fn default() -> Self {
        // game state can be controlled during cargo run
        //
        // ❯ GAMESTATE=InGame cargo run
        if let Ok(state) = env::var("GAMESTATE") {
            match state.as_ref() {
                "AudioMenu" => Self::AudioMenu,
                "AssetLoading" => Self::AssetLoading,
                "MainMenuAssetLoading" => Self::MainMenuAssetLoading,
                "OptionMenu" => Self::OptionMenu,
                "SaveMenu" => Self::SaveMenu,
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
        app.add_loopless_state(GameState::default())
            .add_global_state::<GameState>();
    }
}

pub(crate) fn despawn_with<T: Component>(mut cmd: Commands, q: Query<Entity, With<T>>) {
    for entity in q.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}
