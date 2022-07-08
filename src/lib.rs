#![allow(clippy::too_many_arguments, clippy::type_complexity)]
mod camera;
mod input;
mod npc;
mod physics;
mod player;
mod tilemap;
mod ui;
mod weapon;

use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;

use move_vis::MoveVisPlugin;

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(MoveVisPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(tilemap::TilemapPlugin)
        .add_plugin(input::InputPlugin)
        .add_plugin(npc::NpcPlugin)
        .add_plugin(physics::PhysicsPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(ui::UiPlugin)
        .add_plugin(weapon::WeaponPlugin)
        .run();
}
