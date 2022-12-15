use bevy::{
    ecs::system::SystemParam,
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ButtonState},
    prelude::*,
};
use serde::{Deserialize, Serialize};

use crate::{
    input::{Actionlike, ControlAction, ControlInputMap, InputKind, UserInput},
    state::*,
    ui::menu::{
        button_interact, despawn, get_button_style, BackButton, Despawnable, GameConfig,
        GameConfigSaveEvent, NORMAL_BUTTON, TEXT_COLOR,
    },
};

pub(crate) struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(BindingState::None)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(MenuState::Controls)
                    .with_system(BackButton::to_options_menu.run_if(button_interact::<BackButton>))
                    .with_system(BackButton::on_esc_to_options_menu)
                    .with_system(BindingButton::show_popup.run_if(button_interact::<BindingButton>))
                    .with_system(binding_window_system)
                    .with_system(ResetButton::reset_inputs.run_if(button_interact::<ResetButton>))
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(MenuState::Options)
                    .with_system(ControlButton::show.run_if(button_interact::<ControlButton>))
                    .into(),
            )
            .add_enter_system(MenuState::Controls, control_menu)
            .add_exit_system(MenuState::Controls, cleanup)
            .add_system(load_control_input_map)
            .add_enter_system(BindingState::Conflict, BindingConflict::conflict_popup)
            .add_enter_system(BindingState::None, despawn::<BindingPopUp>)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(BindingState::Conflict)
                    .with_system(
                        ReplaceConflictButton::on_click
                            .run_if(button_interact::<ReplaceConflictButton>),
                    )
                    .with_system(
                        CancelConflictButton::on_click
                            .run_if(button_interact::<CancelConflictButton>),
                    )
                    .into(),
            );
    }
}

fn cleanup(mut cmd: Commands) {
    cmd.remove_resource::<ActiveBinding>();
    cmd.insert_resource(NextState(BindingState::None));
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Resource)]
pub(crate) enum BindingState {
    #[default]
    None,
    Conflict,
}

#[derive(Clone, Copy, Component, Debug, PartialEq)]
struct BindingButton(ControlAction, usize);

impl BindingButton {
    fn show_popup(
        mut cmd: Commands,
        asset_server: Res<AssetServer>,
        query: Query<(&Interaction, &BindingButton), Changed<Interaction>>,
    ) {
        let font = asset_server.load("fonts/monogram.ttf");

        for (interaction, button) in query.iter() {
            if *interaction == Interaction::Clicked {
                let BindingButton(action, index) = button;

                cmd.insert_resource(ActiveBinding::new(*action, *index));

                cmd.spawn((
                    Despawnable,
                    BindingPopUp,
                    NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                margin: UiRect::all(Val::Auto),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: Color::NAVY.into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent
                                .spawn(TextBundle::from_sections([
                                    TextSection::new(
                                        "Press any key now to bind",
                                        TextStyle {
                                            font: font.clone(),
                                            font_size: 30.0,
                                            color: TEXT_COLOR,
                                        },
                                    ),
                                    TextSection::new(
                                        format!(" {action:?} "),
                                        TextStyle {
                                            font: font.clone(),
                                            font_size: 30.0,
                                            color: TEXT_COLOR,
                                        },
                                    ),
                                    TextSection::new(
                                        "or Esc to cancel",
                                        TextStyle {
                                            font: font.clone(),
                                            font_size: 30.0,
                                            color: TEXT_COLOR,
                                        },
                                    ),
                                ]))
                                .insert(BindingPopUp);
                        });
                });
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Resource)]
struct ActiveBinding {
    action: ControlAction,
    index: usize,
    conflict: Option<BindingConflict>,
}
impl ActiveBinding {
    fn new(action: ControlAction, index: usize) -> Self {
        Self {
            action,
            index,
            conflict: None,
        }
    }
}

#[derive(Clone, Copy, Component, Debug, PartialEq)]
struct ResetButton;
impl ResetButton {
    fn spawn(parent: &mut ChildBuilder, button_text_style: TextStyle) {
        parent
            .spawn(ButtonBundle {
                style: get_button_style(),
                background_color: NORMAL_BUTTON.into(),
                ..default()
            })
            .insert(Self)
            .with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section("Reset", button_text_style),
                    ..default()
                });
            });
    }

    fn reset_inputs(
        mut cmd: Commands,
        game_config: Option<ResMut<GameConfig>>,
        mut config_save_event: EventWriter<GameConfigSaveEvent>,
        mut control_input_map: ResMut<ControlInputMap>,
    ) {
        if let Some(mut game_config) = game_config {
            *control_input_map = ControlAction::get_input_map();

            save_input_map(&control_input_map, &mut game_config, &mut config_save_event);
            cmd.insert_resource(NextState(MenuState::Controls));
        }
    }
}

fn binding_window_system(
    mut cmd: Commands,
    mut input_events: InputEvents,
    active_binding: Option<ResMut<ActiveBinding>>,
    mut control_input_map: ResMut<ControlInputMap>,
    mut game_config: ResMut<GameConfig>,
    keyboard_input: Res<Input<KeyCode>>,
    mut config_save_event: EventWriter<GameConfigSaveEvent>,
    binding_state: Res<CurrentState<BindingState>>,
) {
    let mut active_binding = match active_binding {
        Some(active_binding) => active_binding,
        None => return,
    };

    if keyboard_input.just_pressed(KeyCode::Escape) {
        cmd.remove_resource::<ActiveBinding>();
        cmd.insert_resource(NextState(MenuState::Controls));
    } else if active_binding.conflict.is_some() {
        if binding_state.0 != BindingState::Conflict {
            cmd.insert_resource(NextState(BindingState::Conflict));
        }
    } else if let Some(input_button) = input_events.input_button() {
        let conflict_action = control_input_map.iter().find_map(|(inputs, action)| {
            if action != active_binding.action && inputs.contains(&input_button.into()) {
                return Some(action);
            }
            None
        });
        if let Some(action) = conflict_action {
            active_binding.conflict.replace(BindingConflict {
                action,
                input_button,
            });
        } else {
            control_input_map.insert_at(input_button, active_binding.action, active_binding.index);
            cmd.remove_resource::<ActiveBinding>();
            cmd.insert_resource(NextState(MenuState::Controls));
        }
        save_input_map(&control_input_map, &mut game_config, &mut config_save_event);
    }
}

#[derive(Component)]
struct ReplaceConflictButton;

impl ReplaceConflictButton {
    fn on_click(
        mut cmd: Commands,
        active_binding: Option<ResMut<ActiveBinding>>,
        mut control_input_map: ResMut<ControlInputMap>,
    ) {
        if let Some(active_binding) = active_binding {
            if let Some(conflict) = &active_binding.conflict {
                control_input_map.remove(conflict.action, conflict.input_button);

                control_input_map.insert_at(
                    conflict.input_button,
                    active_binding.action,
                    active_binding.index,
                );

                cmd.remove_resource::<ActiveBinding>();
            }
        }
        cmd.insert_resource(NextState(BindingState::None));
    }
}

#[derive(Component)]
struct CancelConflictButton;

impl CancelConflictButton {
    fn on_click(mut cmd: Commands) {
        cmd.remove_resource::<ActiveBinding>();
        cmd.insert_resource(NextState(BindingState::None));
    }
}

#[derive(Component)]
struct BindingPopUp;

#[derive(Clone, Copy, Debug)]
struct BindingConflict {
    action: ControlAction,
    input_button: InputKind,
}

impl BindingConflict {
    fn conflict_popup(
        mut cmd: Commands,
        asset_server: Res<AssetServer>,
        active_binding: Option<ResMut<ActiveBinding>>,
    ) {
        if let Some(active_binding) = active_binding {
            if active_binding.is_changed() {
                if let Some(conflict) = &active_binding.conflict {
                    let font = asset_server.load("fonts/monogram.ttf");
                    let button_text_style = TextStyle {
                        font: font.clone(),
                        font_size: 40.0,
                        color: TEXT_COLOR,
                    };

                    cmd.spawn((
                        Despawnable,
                        BindingPopUp,
                        NodeBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            ..default()
                        },
                    ))
                    .with_children(|parent| {
                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    margin: UiRect::all(Val::Auto),
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: Color::NAVY.into(),
                                ..default()
                            })
                            .with_children(|parent| {
                                parent.spawn(TextBundle {
                                    style: Style {
                                        margin: UiRect::all(Val::Px(50.0)),
                                        ..default()
                                    },
                                    text: Text::from_section(
                                        format!(
                                            "Input {:?} is already used by {:?}",
                                            conflict.input_button, conflict.action
                                        ),
                                        TextStyle {
                                            font: font.clone(),
                                            font_size: 20.0,
                                            color: TEXT_COLOR,
                                        },
                                    ),
                                    ..default()
                                });

                                parent
                                    .spawn(NodeBundle {
                                        style: Style {
                                            margin: UiRect::all(Val::Auto),
                                            flex_direction: FlexDirection::Row,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        ..default()
                                    })
                                    .with_children(|parent| {
                                        parent
                                            .spawn(ButtonBundle {
                                                background_color: NORMAL_BUTTON.into(),
                                                ..default()
                                            })
                                            .insert(ReplaceConflictButton)
                                            .with_children(|parent| {
                                                parent.spawn(TextBundle {
                                                    text: Text::from_section(
                                                        "Replace",
                                                        button_text_style.clone(),
                                                    ),
                                                    ..default()
                                                });
                                            });

                                        parent
                                            .spawn(ButtonBundle {
                                                background_color: NORMAL_BUTTON.into(),
                                                ..default()
                                            })
                                            .insert(CancelConflictButton)
                                            .with_children(|parent| {
                                                parent.spawn(TextBundle {
                                                    text: Text::from_section(
                                                        "Cancel",
                                                        button_text_style.clone(),
                                                    ),
                                                    ..default()
                                                });
                                            });
                                    });
                            });
                    });
                }
            }
        }
    }
}

#[derive(SystemParam)]
struct InputEvents<'w, 's> {
    keys: EventReader<'w, 's, KeyboardInput>,
    mouse_buttons: EventReader<'w, 's, MouseButtonInput>,
    gamepad_events: EventReader<'w, 's, GamepadEvent>,
}

impl InputEvents<'_, '_> {
    fn input_button(&mut self) -> Option<InputKind> {
        if let Some(keyboard_input) = self.keys.iter().next() {
            if keyboard_input.state == ButtonState::Pressed {
                if let Some(key_code) = keyboard_input.key_code {
                    return Some(key_code.into());
                }
            }
        }
        if let Some(mouse_input) = self.mouse_buttons.iter().next() {
            if mouse_input.state == ButtonState::Pressed {
                return Some(mouse_input.button.into());
            }
        }
        if let Some(GamepadEvent {
            gamepad: _,
            event_type,
        }) = self.gamepad_events.iter().next()
        {
            if let GamepadEventType::ButtonChanged(button, strength) = event_type.to_owned() {
                if strength <= 0.5 {
                    return Some(button.into());
                }
            }
        }
        None
    }
}

fn control_menu(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    mut control_input_map: ResMut<ControlInputMap>,
) {
    if control_input_map.is_empty() {
        *control_input_map = ControlAction::get_input_map()
    }

    let font = asset_server.load("fonts/monogram.ttf");

    let button_text_style = TextStyle {
        font: font.clone(),
        font_size: 40.0,
        color: TEXT_COLOR,
    };

    cmd.spawn((
        Despawnable,
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        },
    ))
    .with_children(|parent| {
        parent
            .spawn(NodeBundle {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::CRIMSON.into(),
                ..default()
            })
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            margin: UiRect::all(Val::Auto),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::CRIMSON.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn(TextBundle {
                            style: Style {
                                margin: UiRect::all(Val::Px(50.0)),
                                ..default()
                            },
                            text: Text::from_section(
                                "Controls",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 80.0,
                                    color: TEXT_COLOR,
                                },
                            ),
                            ..default()
                        });
                    });

                for action in ControlAction::variants() {
                    let button_text = match control_input_map.get(action).get_at(0) {
                        Some(UserInput::Single(InputKind::GamepadButton(gamepad_button))) => {
                            format!("{gamepad_button:?}")
                        }
                        Some(UserInput::Single(InputKind::Keyboard(keycode))) => {
                            format!("{keycode:?}")
                        }
                        Some(UserInput::Single(InputKind::Mouse(mouse_button))) => {
                            format!("{mouse_button:?}")
                        }
                        _ => "Empty".to_string(),
                    };

                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: Color::CRIMSON.into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                format!("{action:?}  "),
                                button_text_style.clone(),
                            ));

                            parent
                                .spawn(ButtonBundle {
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                })
                                .insert(BindingButton(action, 0usize))
                                .with_children(|parent| {
                                    parent.spawn(TextBundle {
                                        text: Text::from_section(
                                            button_text,
                                            button_text_style.clone(),
                                        ),
                                        ..default()
                                    });
                                });
                        });
                }

                parent
                    .spawn(NodeBundle {
                        style: Style {
                            margin: UiRect::all(Val::Auto),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::CRIMSON.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        ResetButton::spawn(parent, button_text_style.clone());
                    });

                parent
                    .spawn(NodeBundle {
                        style: Style {
                            margin: UiRect::all(Val::Auto),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::CRIMSON.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        BackButton::spawn(parent, button_text_style.clone());
                    });
            });
    });
}

#[derive(Component)]
pub(crate) struct ControlButton;
impl ControlButton {
    pub(crate) fn show(mut cmd: Commands) {
        cmd.insert_resource(NextState(MenuState::Controls));
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct ControlConfig {
    input_map: ControlInputMap,
}

fn save_input_map(
    input_map: &ControlInputMap,
    game_config: &mut ResMut<GameConfig>,
    config_save_event: &mut EventWriter<GameConfigSaveEvent>,
) {
    game_config.control.input_map = input_map.clone();
    config_save_event.send(GameConfigSaveEvent);
}

fn load_control_input_map(
    mut control_input_map: ResMut<ControlInputMap>,
    game_config: Option<Res<GameConfig>>,
) {
    if let Some(game_config) = game_config {
        if game_config.is_changed() {
            *control_input_map = game_config.control.input_map.clone();
        }
    }
}
