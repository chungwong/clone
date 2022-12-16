use bevy::prelude::*;
use global_state::Transient;

use crate::{
    asset::FontAssets,
    state::{AppLooplessStateExt, ConditionSet, IntoConditionalSystem, MenuState},
    ui::{
        audio::{AudioButton, AudioPlugin},
        control::{ControlButton, ControlPlugin},
        menu::{
            button_interact, get_button_style, on_esc_main_menu, BackButton, NORMAL_BUTTON,
            TEXT_COLOR,
        },
    },
};

pub(crate) struct OptionPlugin;

impl Plugin for OptionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AudioPlugin)
            .add_plugin(ControlPlugin)
            .add_enter_system(MenuState::Options, options_menu)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(MenuState::Options)
                    .with_system(BackButton::to_main_menu.run_if(button_interact::<BackButton>))
                    .with_system(on_esc_main_menu)
                    .into(),
            );
    }
}

fn options_menu(mut cmd: Commands, font_assets: Res<FontAssets>) {
    let font = font_assets.monogram.clone();

    let button_style = get_button_style();

    let button_text_style = TextStyle {
        font: font.clone(),
        font_size: 40.0,
        color: TEXT_COLOR,
    };

    cmd.spawn((
        Name::new("Options Menu"),
        Transient,
        NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Auto),
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Percent(0.0),
                    right: Val::Percent(0.0),
                    top: Val::Percent(0.0),
                    bottom: Val::Percent(0.0),
                },
                ..default()
            },
            ..default()
        },
    ))
    .with_children(|parent| {
        parent
            .spawn((
                Name::new("Wrapper"),
                NodeBundle {
                    style: Style {
                        margin: UiRect::all(Val::Auto),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color: Color::CRIMSON.into(),
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent
                    .spawn((
                        Name::new("Options"),
                        NodeBundle {
                            style: Style {
                                margin: UiRect::all(Val::Auto),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: Color::CRIMSON.into(),
                            ..default()
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle {
                            style: Style {
                                margin: UiRect::all(Val::Px(50.0)),
                                ..default()
                            },
                            text: Text::from_section(
                                "Options",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 80.0,
                                    color: TEXT_COLOR,
                                },
                            ),
                            ..default()
                        });
                    });

                parent
                    .spawn((
                        Name::new("Audio"),
                        NodeBundle {
                            style: Style {
                                margin: UiRect::all(Val::Auto),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: Color::CRIMSON.into(),
                            ..default()
                        },
                    ))
                    .with_children(|parent| {
                        parent
                            .spawn(ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            })
                            .insert(AudioButton)
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_sections([TextSection::new(
                                    "Audio",
                                    button_text_style.clone(),
                                )]));
                            });
                    });

                parent
                    .spawn((
                        Name::new("Controls"),
                        NodeBundle {
                            style: Style {
                                margin: UiRect::all(Val::Auto),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: Color::CRIMSON.into(),
                            ..default()
                        },
                    ))
                    .with_children(|parent| {
                        parent
                            .spawn(ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            })
                            .insert(ControlButton)
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_sections([TextSection::new(
                                    "Controls",
                                    button_text_style.clone(),
                                )]));
                            });
                    });

                BackButton::spawn(parent, button_text_style.clone());
            });
    });
}
