use std::env;

use bevy::prelude::*;
use global_state::{AddGlobalState, GlobalState};
pub(crate) use iyes_loopless::prelude::*;

// GlobalState will despawn all compoents on state exit, unless they are marked with Persistent
#[derive(Clone, Copy, Debug, Eq, Hash, GlobalState, PartialEq)]
pub(crate) enum GameState {
    AudioMenu,
    InGame,
    InGameAssetLoading,
    MainMenu,
    MainMenuAssetLoading,
    OptionMenu,
    Paused,
    SaveMenu,
    Splash,
    SplashAssetLoading,
}

impl Default for GameState {
    fn default() -> Self {
        // game state can be controlled during cargo run
        //
        // ❯ GAMESTATE=InGame cargo run
        if let Ok(state) = env::var("GAMESTATE") {
            match state.as_ref() {
                "AudioMenu" => Self::AudioMenu,
                "InGameAssetLoading" => Self::InGameAssetLoading,
                "MainMenuAssetLoading" => Self::MainMenuAssetLoading,
                "OptionMenu" => Self::OptionMenu,
                "SaveMenu" => Self::SaveMenu,
                "Splash" => Self::Splash,
                "SplashAssetLoading" => Self::SplashAssetLoading,
                _ => panic!("unrecognised game state {}", state),
            }
        } else {
            Self::SplashAssetLoading
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
