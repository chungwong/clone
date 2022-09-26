use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    asset::FontAssets,
    state::{AppLooplessStateExt, ConditionSet, GameState, IntoConditionalSystem, NextState},
    ui::{
        control::{ControlButton, ControlPlugin},
        menu::{
            button_interact, get_button_style, on_esc_main_menu, BackButton, GameConfig,
            GameConfigSaveEvent, SelectedOption, NORMAL_BUTTON, TEXT_COLOR,
        },
    },
};

// impl Default for AudioConfig {
//     fn default() -> Self {
//         Self {
//             master_volume: MasterVolume(0.5),
//             music_volume: MusicVolume(0.5),
//             sound_volume: SoundVolume(0.5),
//         }
//     }
// }

#[derive(Component)]
struct AudioButton;

impl AudioButton {
    fn show(mut cmd: Commands) {
        cmd.insert_resource(NextState(GameState::AudioMenu));
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub(crate) struct AudioConfig {
    pub(crate) master_volume: MasterVolume,
    pub(crate) music_volume: MusicVolume,
    pub(crate) sound_volume: SoundVolume,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: MasterVolume(0.5),
            music_volume: MusicVolume(0.5),
            sound_volume: SoundVolume(0.5),
        }
    }
}

pub(crate) trait ConfigButton {
    fn save(&self, _: &mut ResMut<GameConfig>);
}

#[derive(Clone, Copy, Component, Debug, Deref, DerefMut, PartialEq, Deserialize, Serialize)]
pub(crate) struct MasterVolume(pub(crate) f64);

impl ConfigButton for MasterVolume {
    fn save(&self, game_config: &mut ResMut<GameConfig>) {
        game_config.audio.master_volume = *self;
    }
}

#[derive(Clone, Copy, Component, Debug, Deref, DerefMut, PartialEq, Deserialize, Serialize)]
pub(crate) struct MusicVolume(pub(crate) f64);

impl ConfigButton for MusicVolume {
    fn save(&self, game_config: &mut ResMut<GameConfig>) {
        game_config.audio.music_volume = *self;
    }
}

#[derive(Clone, Copy, Component, Debug, Deref, DerefMut, PartialEq, Deserialize, Serialize)]
pub(crate) struct SoundVolume(pub(crate) f64);

impl ConfigButton for SoundVolume {
    fn save(&self, game_config: &mut ResMut<GameConfig>) {
        game_config.audio.sound_volume = *self;
    }
}

fn select_button<T: Component + PartialEq + ConfigButton>(
    interaction_query: Query<
        (&Interaction, &T, Entity),
        (Changed<Interaction>, With<Button>, With<T>),
    >,
    mut selected_query: Query<(Entity, &mut BackgroundColor), (With<SelectedOption>, With<T>)>,
    mut cmd: Commands,
    mut config_save_event: EventWriter<GameConfigSaveEvent>,
    mut game_config: ResMut<GameConfig>,
) {
    for (interaction, button, entity) in &interaction_query {
        if *interaction == Interaction::Clicked {
            if let Ok((previous_button, mut previous_color)) = selected_query.get_single_mut() {
                *previous_color = NORMAL_BUTTON.into();
                cmd.entity(previous_button).remove::<SelectedOption>();
                cmd.entity(entity).insert(SelectedOption);
                button.save(&mut game_config);
                config_save_event.send(GameConfigSaveEvent);
            } else {
                error!("setting button error");
            }
        }
    }
}

pub(crate) struct OptionPlugin;

impl Plugin for OptionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ControlPlugin)
            .add_enter_system(GameState::OptionMenu, option_menu)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::OptionMenu)
                    .with_system(BackButton::to_main_menu.run_if(button_interact::<BackButton>))
                    .with_system(AudioButton::show.run_if(button_interact::<AudioButton>))
                    .with_system(on_esc_main_menu)
                    .into(),
            )
            .add_enter_system(GameState::AudioMenu, audio_menu)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::AudioMenu)
                    .with_system(BackButton::to_option_menu.run_if(button_interact::<BackButton>))
                    .with_system(AudioButton::show.run_if(button_interact::<AudioButton>))
                    .with_system(select_button::<MasterVolume>)
                    .with_system(select_button::<SoundVolume>)
                    .with_system(select_button::<MusicVolume>)
                    .with_system(on_esc_main_menu)
                    .into(),
            );
    }
}

fn option_menu(mut cmd: Commands, font_assets: Res<FontAssets>) {
    cmd.spawn(Camera2dBundle::default());
    let font = font_assets.monogram.clone();

    let button_style = get_button_style();

    let button_text_style = TextStyle {
        font: font.clone(),
        font_size: 40.0,
        color: TEXT_COLOR,
    };

    cmd.spawn(NodeBundle {
        style: Style {
            margin: UiRect::all(Val::Auto),
            flex_direction: FlexDirection::Column,
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
}

fn audio_menu(mut cmd: Commands, game_config: Res<GameConfig>, font_assets: Res<FontAssets>) {
    cmd.spawn(Camera2dBundle::default());

    let audio_config = &game_config.audio;

    let font = font_assets.monogram.clone();

    let button_style = get_button_style();

    let button_text_style = TextStyle {
        font: font.clone(),
        font_size: 40.0,
        color: TEXT_COLOR,
    };

    cmd.spawn(NodeBundle {
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
                        "Audio",
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
                    "Master Volume",
                    button_text_style.clone(),
                ));
                for volume_setting in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0] {
                    let mut entity = parent.spawn(ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(30.0), Val::Px(65.0)),
                            ..button_style.clone()
                        },
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    });
                    entity.insert(MasterVolume(volume_setting));
                    if audio_config.master_volume == MasterVolume(volume_setting) {
                        entity.insert(SelectedOption);
                    }
                }
            });
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
                    "Music Volume",
                    button_text_style.clone(),
                ));
                for volume_setting in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0] {
                    let mut entity = parent.spawn(ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(30.0), Val::Px(65.0)),
                            ..button_style.clone()
                        },
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    });
                    entity.insert(MusicVolume(volume_setting));
                    if audio_config.music_volume == MusicVolume(volume_setting) {
                        entity.insert(SelectedOption);
                    }
                }
            });
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
                    "Sound Volume",
                    button_text_style.clone(),
                ));
                for volume_setting in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0] {
                    let mut entity = parent.spawn(ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(30.0), Val::Px(65.0)),
                            ..button_style.clone()
                        },
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    });
                    entity.insert(SoundVolume(volume_setting));
                    if audio_config.sound_volume == SoundVolume(volume_setting) {
                        entity.insert(SelectedOption);
                    }
                }
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
}
