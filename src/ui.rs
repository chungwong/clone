mod menu;

use bevy::prelude::*;

use bevy_egui::EguiPlugin;
#[cfg(feature = "debug")]
use bevy_egui::{egui, EguiContext};

#[cfg(feature = "debug")]
use move_vis::make_slider;

#[cfg(feature = "debug")]
use crate::{
    physics::{set_gravity, set_jump_power_coefficient, RapierConfiguration},
    player::PlayerMovementSettings,
};

use crate::{
    input::{MenuAction, MenuActionState},
    state::{ConditionSet, GameState, NextState},
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .add_plugin(menu::MenuPlugin)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(pause)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Paused)
                    .with_system(resume)
                    .into(),
            );

        #[cfg(feature = "debug")]
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(movement_ui)
                .into(),
        );
    }
}

#[cfg(feature = "debug")]
fn movement_ui(
    mut egui_context: ResMut<EguiContext>,
    mut player_movement_settings: ResMut<PlayerMovementSettings>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    egui::Window::new("Physical Properties Tweaking").show(egui_context.ctx_mut(), |ui| {
        let player_movement_settings = &mut *player_movement_settings;

        ui.add(make_slider(
            "Jump Height",
            &mut player_movement_settings.jump_height,
            1.0..=20.0,
        ));
        ui.add(make_slider(
            "Time To Apex",
            &mut player_movement_settings.time_to_apex,
            0.1..=1.0,
        ));
        ui.add(make_slider(
            "Run Speed",
            &mut player_movement_settings.run_speed,
            100.0..=2000.0,
        ));
        ui.add(make_slider(
            "Dash Speed",
            &mut player_movement_settings.dash_speed,
            5000.0..=20000.0,
        ));
        // ui.add(make_slider(
        //     "Jump Impulse",
        //     &mut player_movement_settings.jump_impulse,
        //     10000.0..=50000.0,
        // ));
        ui.add(make_slider(
            "Coyote Time(ms)",
            &mut player_movement_settings.coyote_time_ms,
            10..=200,
        ));
        ui.add(make_slider(
            "Slide Factor",
            &mut player_movement_settings.slide_factor,
            0.0..=-1000.0,
        ));
        ui.add(make_slider(
            "Fall Factor",
            &mut player_movement_settings.fall_factor,
            50.0..=200.0,
        ));
        ui.add(make_slider(
            "Jump Break Factor",
            &mut player_movement_settings.jump_break_factor,
            100.0..=400.0,
        ));
        ui.add(make_slider(
            "Gravity Scale",
            &mut player_movement_settings.gravity_scale,
            1.0..=20.0,
        ));

        set_gravity(&mut rapier_config, player_movement_settings);
        set_jump_power_coefficient(&rapier_config, &mut *player_movement_settings);
    });
}

/// Transition game to pause state
fn pause(mut cmd: Commands, input: Query<&MenuActionState>) {
    let input = input.single();
    if input.just_pressed(MenuAction::Pause) {
        cmd.insert_resource(NextState(GameState::Paused));
    }
}

// Transition game out of paused state
fn resume(mut cmd: Commands, input: Query<&MenuActionState>) {
    let input = input.single();
    if input.just_pressed(MenuAction::Pause) {
        cmd.insert_resource(NextState(GameState::InGame));
    }
}
