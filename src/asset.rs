use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::{
    audio::AudioSource,
    state::{AppLooplessStateExt, AppState},
};

#[derive(AssetCollection, Resource)]
pub(crate) struct MainMenuAssets {
    #[asset(key = "main_menu.bgm")]
    pub(crate) bgm: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub(crate) struct ImageAssets {
    #[asset(key = "image.player")]
    pub(crate) player: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub(crate) struct AudioAssets {
    #[asset(key = "sounds.bgm")]
    pub(crate) bgm: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub(crate) struct FontAssets {
    #[asset(key = "font.monogram")]
    pub(crate) monogram: Handle<Font>,
}

pub(crate) struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::SplashAssetLoading)
                .continue_to_state(AppState::Splash)
                .with_dynamic_collections::<StandardDynamicAssetCollection>(vec![
                    "dynamic_asset.assets",
                ])
                .with_collection::<FontAssets>(),
        )
        .add_loading_state(
            LoadingState::new(AppState::MainMenuAssetLoading)
                .continue_to_state(AppState::MainMenu)
                .with_dynamic_collections::<StandardDynamicAssetCollection>(vec![
                    "dynamic_asset.assets",
                ])
                .with_collection::<FontAssets>()
                .with_collection::<MainMenuAssets>(),
        )
        .add_loading_state(
            LoadingState::new(AppState::InGameAssetLoading)
                .continue_to_state(AppState::InGame)
                .with_dynamic_collections::<StandardDynamicAssetCollection>(vec![
                    "dynamic_asset.assets",
                ])
                .with_collection::<FontAssets>()
                .with_collection::<ImageAssets>()
                .with_collection::<AudioAssets>(),
        )
        .add_enter_system(AppState::MainMenuAssetLoading, loading_screen)
        .add_enter_system(AppState::InGameAssetLoading, loading_screen);
    }
}

fn loading_screen(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn(Camera2dBundle::default());

    let font = asset_server.load("fonts/monogram.ttf");

    cmd.spawn((
        Name::new("Loading"),
        Text2dBundle {
            text: Text::from_section(
                "Loading".to_owned(),
                TextStyle {
                    font,
                    font_size: 68.0,
                    color: Color::WHITE,
                },
            )
            .with_alignment(TextAlignment {
                horizontal: HorizontalAlign::Center,
                vertical: VerticalAlign::Center,
            }),
            ..default()
        },
    ));

    cmd.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Vec2::new(160., 12.).into(),
            color: Color::BLACK,
            ..default()
        },
        ..default()
    })
    .insert(Transform::from_xyz(0., -70., 0.));

    cmd.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Vec2::new(160., 12.).into(),
            color: Color::WHITE,
            ..default()
        },
        ..default()
    })
    .insert(Transform::from_xyz(0., -70., 0.).with_scale(Vec3::new(0., 1., 0.)));
}
