#![feature(trait_alias)]
#![feature(path_file_prefix)]
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod asset;
mod audio;
mod camera;
mod input;
mod npc;
mod physics;
mod player;
mod save;
mod splash_screen;
mod state;
mod tilemap;
mod ui;
mod weapon;

use bevy::prelude::*;

pub fn run() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        //state should be loaded first as there are a lot of plugins depend on `AppState`
        .add_plugin(state::StatePlugin)
        .add_plugin(asset::AssetPlugin)
        .add_plugin(audio::AudioPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(tilemap::TilemapPlugin)
        .add_plugin(input::InputPlugin)
        .add_plugin(npc::NpcPlugin)
        .add_plugin(physics::PhysicsPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(save::SavePlugin)
        .add_plugin(splash_screen::SplashScreenPlugin)
        .add_plugin(ui::UiPlugin)
        .add_plugin(weapon::WeaponPlugin);

    #[cfg(feature = "debug")]
    app.add_plugin(move_vis::MoveVisPlugin);

    #[cfg(feature = "debug")]
    app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new());

    app.run();
}
