#![allow(clippy::too_many_arguments, clippy::type_complexity)]
mod camera;
mod input;
mod menu;
mod npc;
mod physics;
mod player;
mod splash_screen;
mod state;
mod tilemap;
mod ui;
mod weapon;

use bevy::prelude::*;

pub fn run() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugin(state::StatePlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(tilemap::TilemapPlugin)
        .add_plugin(input::InputPlugin)
        .add_plugin(menu::MenuPlugin)
        .add_plugin(npc::NpcPlugin)
        .add_plugin(physics::PhysicsPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(splash_screen::SplashScreenPlugin)
        .add_plugin(ui::UiPlugin)
        .add_plugin(weapon::WeaponPlugin);

    #[cfg(feature = "debug")]
    app.add_plugin(move_vis::MoveVisPlugin);

    #[cfg(feature = "debug")]
    app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new());

    app.run();
}
