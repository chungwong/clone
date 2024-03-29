use bevy::prelude::*;
pub use bevy_rapier2d::prelude::*;

use crate::{
    player::PlayerMovementSettings,
    state::{AppLooplessStateExt, AppState, ConditionSet},
    tilemap::LevelEvent,
};

// pub const SCALE: f32 = 100.0;
// pub const SCALE: f32 = 10.0;
pub(crate) const SCALE: f32 = 1.0;

pub(crate) struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(SCALE))
            .add_enter_system(AppState::InGame, setup)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(AppState::InGame)
                    .with_system(pause_physics_during_load)
                    .into(),
            );
    }
}

pub(crate) fn pause_physics(rapier_config: &mut ResMut<RapierConfiguration>) {
    rapier_config.physics_pipeline_active = false;
    rapier_config.query_pipeline_active = false;
}

pub(crate) fn resume_physics(rapier_config: &mut ResMut<RapierConfiguration>) {
    rapier_config.physics_pipeline_active = true;
    rapier_config.query_pipeline_active = true;
}

fn pause_physics_during_load(
    mut level_events: EventReader<LevelEvent>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    for event in level_events.iter() {
        match event {
            LevelEvent::SpawnTriggered(_) => pause_physics(&mut rapier_config),
            LevelEvent::Transformed(_) => resume_physics(&mut rapier_config),
            _ => (),
        }
    }
}

fn setup(
    mut rapier_config: ResMut<RapierConfiguration>,
    mut player_movement_settings: ResMut<PlayerMovementSettings>,
) {
    set_gravity(&mut rapier_config, &player_movement_settings);
    set_jump_power_coefficient(&rapier_config, &mut player_movement_settings);
}

/// what is the gravity that would allow jumping to a given height?
pub(crate) fn set_gravity(
    rapier_config: &mut ResMut<RapierConfiguration>,
    player_movement_settings: &PlayerMovementSettings,
) {
    rapier_config.gravity.y = -(2.0 * player_movement_settings.jump_height)
        / player_movement_settings.time_to_apex.powf(2.0);
}

/// what is the initial jump velocity?
/// 50 is a multiplier.  Say the expected value of jump_power_coefficient is 20,000 and
/// (2.0 * rapier_config.gravity.y.abs() * JUMP_HEIGHT).sqrt() gives 400.0
/// It is necessary to multiply 50.0 to reach to 20,000
pub(crate) fn set_jump_power_coefficient(
    rapier_config: &ResMut<RapierConfiguration>,
    player_movement_settings: &mut PlayerMovementSettings,
) {
    player_movement_settings.jump_power_coefficient =
        (2.0 * rapier_config.gravity.y.abs() * player_movement_settings.jump_height).sqrt();
    player_movement_settings.jump_power_coefficient *= 50.0 / SCALE.powf(2.0);
}
