use bevy::prelude::*;
use std::fs;

use crate::{
    asset::FontAssets,
    save::{load_file, CurrentSave, Save, SaveData, SaveSlots},
    state::*,
    ui::menu::{
        button_interact, get_button_style, on_esc_main_menu, BackButton, MainMenuButton,
        NORMAL_BUTTON, TEXT_COLOR,
    },
};

pub(crate) struct SaveMenuPlugin;

impl Plugin for SaveMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppState::SaveMenu, save_menu)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(AppState::SaveMenu)
                    .with_system(NewSaveButton::create.run_if(button_interact::<NewSaveButton>))
                    .with_system(LoadSaveButton::load.run_if(button_interact::<LoadSaveButton>))
                    .with_system(
                        MainMenuButton::back_to_main_menu.run_if(button_interact::<BackButton>),
                    )
                    .with_system(DeleteButton::delete.run_if(button_interact::<DeleteButton>))
                    .with_system(
                        DeleteModeButton::toggle.run_if(button_interact::<DeleteModeButton>),
                    )
                    .with_system(update_menu)
                    .with_system(on_esc_main_menu)
                    .into(),
            );
    }
}

#[derive(Component)]
struct NewSaveButton;
impl NewSaveButton {
    fn create(mut cmd: Commands, mut current_save: ResMut<CurrentSave>) {
        cmd.insert_resource(NextState(AppState::InGameAssetLoading));
        current_save.0 = SaveSlots::new_save();
    }
}

#[derive(Component, Debug, Deref, DerefMut)]
struct LoadSaveButton(Save);
impl LoadSaveButton {
    fn load(
        mut cmd: Commands,
        query: Query<(&Interaction, &LoadSaveButton), (Changed<Interaction>, With<Button>)>,
        mut current_save: ResMut<CurrentSave>,
    ) {
        for (interaction, button) in query.iter() {
            if *interaction == Interaction::Clicked {
                current_save.0 = button.0.clone();
                debug!("use save {:?}", &current_save);

                if let Some(path) = &current_save.0.path {
                    let save_data: SaveData = load_file(path, 0).unwrap();
                    debug!("loaded save data {:?}", &save_data);
                    current_save.0.data = Some(save_data);
                }

                cmd.insert_resource(NextState(AppState::InGameAssetLoading));
            }
        }
    }
}

#[derive(Component, Debug)]
struct DeleteMode(bool);

#[derive(Component)]
struct DeleteModeButtonText;

#[derive(Component)]
struct DeleteButton(Save);

impl DeleteButton {
    fn delete(
        mut cmd: Commands,
        query: Query<(&Interaction, &DeleteButton), (Changed<Interaction>, With<Button>)>,
    ) {
        for (interaction, button) in query.iter() {
            if *interaction == Interaction::Clicked {
                if let Some(ref path) = button.0.path {
                    if fs::remove_file(path).is_ok() {
                        cmd.insert_resource(NextState(AppState::SaveMenu));
                    } else {
                        error!("cannot remove save {:?}", button.0);
                    }
                }
            }
        }
    }
}

#[derive(Component)]
struct DeleteModeButton;

impl DeleteModeButton {
    fn toggle(mut delete_mode: Query<&mut DeleteMode>) {
        if let Ok(mut delete_mode) = delete_mode.get_single_mut() {
            delete_mode.0 = !delete_mode.0;
        }
    }
}

fn update_menu(
    mut cmd: Commands,
    save_nodes: Query<(&Parent, &LoadSaveButton)>,
    mut delete_buttons: Query<Entity, With<DeleteButton>>,
    delete_mode: Query<&DeleteMode, Changed<DeleteMode>>,
    mut button_text: Query<&mut Text, With<DeleteModeButtonText>>,
    font_assets: Res<FontAssets>,
) {
    if let Ok(DeleteMode(delete_mode)) = delete_mode.get_single() {
        if let Ok(mut delete_text) = button_text.get_single_mut() {
            if *delete_mode {
                // update Delete Mode Button text
                delete_text.sections[0].value = "Cancel".to_string();

                // insert delete buttons
                let button_text_style = TextStyle {
                    font: font_assets.monogram.clone(),
                    font_size: 40.0,
                    color: TEXT_COLOR,
                };

                for (parent, save_button) in save_nodes.iter() {
                    let button_id = cmd
                        .spawn((
                            Name::new("Delete Button"),
                            DeleteButton(save_button.0.clone()),
                            ButtonBundle {
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle {
                                text: Text::from_section("Delete?", button_text_style.clone()),
                                ..default()
                            });
                        })
                        .id();

                    cmd.entity(parent.get()).push_children(&[button_id]);
                }
            } else {
                // update Delete Mode Button text
                delete_text.sections[0].value = "Delete Save".to_string();

                // remove delete buttons
                for entty in &mut delete_buttons {
                    cmd.entity(entty).despawn_recursive();
                }
            }
        }
    }
}

fn save_menu(mut cmd: Commands, font_assets: Res<FontAssets>) {
    cmd.spawn(Camera2dBundle::default());

    let font = font_assets.monogram.clone();

    let button_style = get_button_style();

    let button_text_style = TextStyle {
        font: font.clone(),
        font_size: 40.0,
        color: TEXT_COLOR,
    };

    cmd.spawn((
        Name::new("Save Menu"),
        DeleteMode(false),
        NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Auto),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            background_color: Color::CRIMSON.into(),
            ..default()
        },
    ))
    .with_children(|parent| {
        parent
            .spawn((
                Name::new("Saves"),
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
                SaveSlots::new(5).iter().enumerate().for_each(|(i, save)| {
                    parent
                        .spawn((
                            Name::new(format!("Save Slot {i}")),
                            NodeBundle {
                                style: Style {
                                    margin: UiRect::all(Val::Auto),
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: Color::CRIMSON.into(),
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            let mut button = parent.spawn(ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            });

                            if save.filename == "New Save" {
                                button.insert(NewSaveButton {});
                            } else {
                                button.insert(LoadSaveButton(save.clone()));
                            };

                            button.with_children(|parent| {
                                parent.spawn(TextBundle {
                                    text: Text::from_section(
                                        save.filename.to_owned(),
                                        button_text_style.clone(),
                                    ),
                                    ..default()
                                });
                            });
                        });
                });

                BackButton::spawn(parent, button_text_style.clone());
            });

        let SaveSlots(saves) = SaveSlots::get_saves();

        if !saves.is_empty() {
            parent
                .spawn((
                    Name::new("Actions"),
                    NodeBundle {
                        style: Style {
                            align_items: AlignItems::FlexStart,
                            ..default()
                        },
                        background_color: Color::CRIMSON.into(),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            Name::new("Delete Save"),
                            DeleteModeButton,
                            ButtonBundle {
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            parent
                                .spawn(TextBundle::from_sections([TextSection::new(
                                    "Delete Save",
                                    TextStyle {
                                        font,
                                        font_size: 20.0,
                                        color: Color::WHITE,
                                    },
                                )]))
                                .insert(DeleteModeButtonText);
                        });
                });
        }
    });
}
