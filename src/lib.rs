#![allow(clippy::too_many_arguments, clippy::type_complexity)]
mod camera;
mod physics;
mod player;
mod tilemap;
mod ui;

use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;

use move_vis::MoveVisPlugin;

use camera::CameraPlugin;
use physics::PhysicsPlugin;
use player::PlayerPlugin;
use tilemap::TilemapPlugin;
use ui::UiPlugin;

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(CameraPlugin)
        .add_plugin(TilemapPlugin)
        .add_plugin(MoveVisPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(UiPlugin)
        .run();
}
