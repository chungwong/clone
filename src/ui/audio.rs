use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    asset::FontAssets,
    state::*,
    ui::menu::{
        button_interact, get_button_style, select_button, BackButton, ConfigButton, Despawnable,
        GameConfig, SelectedOption, NORMAL_BUTTON, TEXT_COLOR,
    },
};

pub(crate) struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(MenuState::Audio, audio_menu)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(MenuState::Options)
                    .with_system(AudioButton::show.run_if(button_interact::<AudioButton>))
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(MenuState::Audio)
                    .with_system(BackButton::to_options_menu.run_if(button_interact::<BackButton>))
                    .with_system(BackButton::on_esc_to_options_menu)
                    .with_system(AudioButton::show.run_if(button_interact::<AudioButton>))
                    .with_system(select_button::<MasterVolume>)
                    .with_system(select_button::<SoundVolume>)
                    .with_system(select_button::<MusicVolume>)
                    .into(),
            );
    }
}

#[derive(Component)]
pub(crate) struct AudioButton;

impl AudioButton {
    fn show(mut cmd: Commands) {
        cmd.insert_resource(NextState(MenuState::Audio));
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

fn audio_menu(mut cmd: Commands, game_config: Res<GameConfig>, font_assets: Res<FontAssets>) {
    let audio_config = &game_config.audio;

    let font = font_assets.monogram.clone();

    let button_style = get_button_style();

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
                        for volume_setting in
                            [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
                        {
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
                        for volume_setting in
                            [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
                        {
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
                        for volume_setting in
                            [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
                        {
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
    });
}