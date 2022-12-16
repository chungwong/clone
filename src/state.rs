use std::env;

use bevy::prelude::*;
use global_state::{AddGlobalState, AddTransientState, GlobalState, TransientState};
pub(crate) use iyes_loopless::prelude::*;

// GlobalState will despawn all compoents on state exit, unless they are marked with Persistent
#[derive(Clone, Copy, Debug, Eq, Hash, GlobalState, PartialEq, Resource)]
pub(crate) enum AppState {
    InGame,
    InGameAssetLoading,
    MainMenu,
    MainMenuAssetLoading,
    SaveMenu,
    Splash,
    SplashAssetLoading,
}

impl Default for AppState {
    fn default() -> Self {
        // game state can be controlled during cargo run
        //
        // ❯ GAMESTATE=InGame cargo run
        if let Ok(state) = env::var("APPSTATE") {
            match state.as_ref() {
                "InGame" => Self::InGameAssetLoading,
                "InGameAssetLoading" => Self::InGameAssetLoading,
                "MainMenu" => Self::MainMenuAssetLoading,
                "MainMenuAssetLoading" => Self::MainMenuAssetLoading,
                "SaveMenu" => Self::SaveMenu,
                "Splash" => Self::SplashAssetLoading,
                "SplashAssetLoading" => Self::SplashAssetLoading,
                _ => panic!("unrecognised game state {state}"),
            }
        } else {
            Self::SplashAssetLoading
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Resource, TransientState)]
pub(crate) enum MenuState {
    Audio,
    Controls,
    #[default]
    None,
    Options,
    Save,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Resource, TransientState)]
pub(crate) enum PauseState {
    #[default]
    None,
    On,
    Off,
}

impl PauseState {
    pub(crate) fn is_paused(pause_state: Res<CurrentState<PauseState>>) -> bool {
        pause_state.0 == PauseState::On
    }
}

pub(crate) struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(AppState::default())
            .add_loopless_state(MenuState::default())
            .add_loopless_state(PauseState::default())
            .add_global_state::<AppState>()
            .add_transient_state::<MenuState>()
            .add_transient_state::<PauseState>();
    }
}
