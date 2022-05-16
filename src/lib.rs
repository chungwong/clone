mod camera;
mod physics;
mod tilemap;

use bevy::prelude::*;

use camera::CameraPlugin;
use physics::PhysicsPlugin;
use tilemap::TileMapPlugin;

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(CameraPlugin)
        .add_plugin(TileMapPlugin)
        .add_plugin(PhysicsPlugin)
        .run();
}
