use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::{
    audio::AudioSource,
    state::GameState,
};

#[derive(AssetCollection)]
pub(crate) struct MainMenuAssets {
    #[asset(key = "main_menu.bgm")]
    pub(crate) bgm: Handle<AudioSource>,

}

#[derive(AssetCollection)]
pub(crate) struct ImageAssets {
    #[asset(key = "image.player")]
    pub(crate) player: Handle<Image>,
}

#[derive(AssetCollection)]
pub(crate) struct AudioAssets {
    #[asset(key = "sounds.bgm")]
    pub(crate) bgm: Handle<AudioSource>,
}

pub(crate) struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_loading_state(
                LoadingState::new(GameState::MainMenuAssetLoading)
                .continue_to_state(GameState::MainMenu)
                .with_dynamic_collections::<StandardDynamicAssetCollection>(vec![
                    "dynamic_asset.assets",
                ])
                .with_collection::<MainMenuAssets>()
            )
            .add_loading_state(
                LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::InGame)
                .with_dynamic_collections::<StandardDynamicAssetCollection>(vec![
                    "dynamic_asset.assets",
                ])
                .with_collection::<ImageAssets>()
                .with_collection::<AudioAssets>(),
            );

    }
}
