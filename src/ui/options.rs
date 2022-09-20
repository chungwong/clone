use bevy::prelude::*;

use crate::{
    state::{
        despawn_with, AppLooplessStateExt, ConditionSet, GameState, IntoConditionalSystem,
        NextState,
    },
    ui::menu::{
        button_interact, get_button_style, on_esc, BackButton, GameConfig, GameConfigSaveEvent,
        SelectedOption, NORMAL_BUTTON, TEXT_COLOR,
    },
};

#[derive(Component)]
struct OnOptionScreen;

#[derive(Component)]
struct OnAudioScreen;

#[derive(Component)]
struct AudioButton;

impl AudioButton {
    fn show(mut cmd: Commands) {
        cmd.insert_resource(NextState(GameState::AudioMenu));
    }
}

#[derive(Clone, Copy, Debug, Savefile)]
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

trait Volume {
    fn save(&self, _: &mut ResMut<GameConfig>);
}

#[derive(Clone, Copy, Component, Debug, Deref, DerefMut, PartialEq, Savefile)]
pub(crate) struct MasterVolume(pub(crate) f64);

impl Volume for MasterVolume {
    fn save(&self, game_config: &mut ResMut<GameConfig>) {
        game_config.audio.master_volume = *self;
    }
}

#[derive(Clone, Copy, Component, Debug, Deref, DerefMut, PartialEq, Savefile)]
pub(crate) struct MusicVolume(pub(crate) f64);

impl Volume for MusicVolume {
    fn save(&self, game_config: &mut ResMut<GameConfig>) {
        game_config.audio.music_volume = *self;
    }
}

#[derive(Clone, Copy, Component, Debug, Deref, DerefMut, PartialEq, Savefile)]
pub(crate) struct SoundVolume(pub(crate) f64);

impl Volume for SoundVolume {
    fn save(&self, game_config: &mut ResMut<GameConfig>) {
        game_config.audio.sound_volume = *self;
    }
}

fn setting_button<T: Component + PartialEq + Volume>(
    interaction_query: Query<
        (&Interaction, &T, Entity),
        (Changed<Interaction>, With<Button>, With<T>),
    >,
    mut selected_query: Query<(Entity, &mut UiColor), (With<SelectedOption>, With<T>)>,
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

pub struct OptionPlugin;

impl Plugin for OptionPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::OptionMenu, option_menu)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::OptionMenu)
                    .with_system(BackButton::to_main_menu.run_if(button_interact::<BackButton>))
                    .with_system(AudioButton::show.run_if(button_interact::<AudioButton>))
                    .with_system(on_esc)
                    .into(),
            )
            .add_exit_system(GameState::OptionMenu, despawn_with::<OnOptionScreen>)
            .add_enter_system(GameState::AudioMenu, audio_menu)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::AudioMenu)
                    .with_system(BackButton::to_option_menu.run_if(button_interact::<BackButton>))
                    .with_system(AudioButton::show.run_if(button_interact::<AudioButton>))
                    .with_system(setting_button::<MasterVolume>)
                    .with_system(setting_button::<SoundVolume>)
                    .with_system(setting_button::<MusicVolume>)
                    .with_system(on_esc)
                    .into(),
            )
            .add_exit_system(GameState::AudioMenu, despawn_with::<OnAudioScreen>);
    }
}

fn option_menu(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/monogram.ttf");

    let button_style = get_button_style();

    let button_text_style = TextStyle {
        font: font.clone(),
        font_size: 40.0,
        color: TEXT_COLOR,
    };

    cmd.spawn_bundle(NodeBundle {
        style: Style {
            margin: UiRect::all(Val::Auto),
            flex_direction: FlexDirection::ColumnReverse,
            ..default()
        },
        color: Color::CRIMSON.into(),
        ..default()
    })
    .insert(OnOptionScreen)
    .with_children(|parent| {
        parent
            .spawn_bundle(NodeBundle {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    flex_direction: FlexDirection::ColumnReverse,
                    align_items: AlignItems::Center,
                    ..default()
                },
                color: Color::CRIMSON.into(),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle {
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
            .spawn_bundle(NodeBundle {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    flex_direction: FlexDirection::ColumnReverse,
                    align_items: AlignItems::Center,
                    ..default()
                },
                color: Color::CRIMSON.into(),
                ..default()
            })
            .with_children(|parent| {
                parent
                    .spawn_bundle(ButtonBundle {
                        style: button_style.clone(),
                        color: NORMAL_BUTTON.into(),
                        ..default()
                    })
                    .insert(AudioButton)
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle::from_sections([TextSection::new(
                            "Audio",
                            button_text_style,
                        )]));
                    });
            });

        parent
            .spawn_bundle(NodeBundle {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    flex_direction: FlexDirection::ColumnReverse,
                    align_items: AlignItems::Center,
                    ..default()
                },
                color: Color::CRIMSON.into(),
                ..default()
            })
            .with_children(|parent| {
                BackButton::spawn(parent, &asset_server);
            });
    });
}

fn audio_menu(mut cmd: Commands, asset_server: Res<AssetServer>, game_config: Res<GameConfig>) {
    let audio_config = &game_config.audio;

    let font = asset_server.load("fonts/monogram.ttf");

    let button_style = get_button_style();

    let button_text_style = TextStyle {
        font: font.clone(),
        font_size: 40.0,
        color: TEXT_COLOR,
    };

    cmd.spawn_bundle(NodeBundle {
        style: Style {
            margin: UiRect::all(Val::Auto),
            flex_direction: FlexDirection::ColumnReverse,
            align_items: AlignItems::Center,
            ..default()
        },
        color: Color::CRIMSON.into(),
        ..default()
    })
    .insert(OnAudioScreen)
    .with_children(|parent| {
        parent
            .spawn_bundle(NodeBundle {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    flex_direction: FlexDirection::ColumnReverse,
                    align_items: AlignItems::Center,
                    ..default()
                },
                color: Color::CRIMSON.into(),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle {
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
            .spawn_bundle(NodeBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    ..default()
                },
                color: Color::CRIMSON.into(),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle::from_section(
                    "Master Volume",
                    button_text_style.clone(),
                ));
                for volume_setting in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0] {
                    let mut entity = parent.spawn_bundle(ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(30.0), Val::Px(65.0)),
                            ..button_style.clone()
                        },
                        color: NORMAL_BUTTON.into(),
                        ..default()
                    });
                    entity.insert(MasterVolume(volume_setting));
                    if audio_config.master_volume == MasterVolume(volume_setting) {
                        entity.insert(SelectedOption);
                    }
                }
            });
        parent
            .spawn_bundle(NodeBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    ..default()
                },
                color: Color::CRIMSON.into(),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle::from_section(
                    "Music Volume",
                    button_text_style.clone(),
                ));
                for volume_setting in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0] {
                    let mut entity = parent.spawn_bundle(ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(30.0), Val::Px(65.0)),
                            ..button_style.clone()
                        },
                        color: NORMAL_BUTTON.into(),
                        ..default()
                    });
                    entity.insert(MusicVolume(volume_setting));
                    if audio_config.music_volume == MusicVolume(volume_setting) {
                        entity.insert(SelectedOption);
                    }
                }
            });
        parent
            .spawn_bundle(NodeBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    ..default()
                },
                color: Color::CRIMSON.into(),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle::from_section(
                    "Sound Volume",
                    button_text_style.clone(),
                ));
                for volume_setting in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0] {
                    let mut entity = parent.spawn_bundle(ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(30.0), Val::Px(65.0)),
                            ..button_style.clone()
                        },
                        color: NORMAL_BUTTON.into(),
                        ..default()
                    });
                    entity.insert(SoundVolume(volume_setting));
                    if audio_config.sound_volume == SoundVolume(volume_setting) {
                        entity.insert(SelectedOption);
                    }
                }
            });

        parent
            .spawn_bundle(NodeBundle {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    flex_direction: FlexDirection::ColumnReverse,
                    align_items: AlignItems::Center,
                    ..default()
                },
                color: Color::CRIMSON.into(),
                ..default()
            })
            .with_children(|parent| {
                BackButton::spawn(parent, &asset_server);
            });
    });
}
