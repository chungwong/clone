use bevy::{ecs::system::SystemState, prelude::*, tasks::IoTaskPool};
use std::{fs, path::PathBuf};

use crate::{
    asset::FontAssets,
    player::{Health, Player},
    state::{AppLooplessStateExt, ConditionSet, GameState, IntoConditionalSystem, NextState},
    ui::menu::{button_interact, get_button_style, on_esc, BackButton, NORMAL_BUTTON, TEXT_COLOR},
};

const SAVE_DIR: &str = "saves";

#[derive(Component)]
struct NewSaveButton;
impl NewSaveButton {
    fn create(mut cmd: Commands, mut current_save: ResMut<CurrentSave>) {
        cmd.insert_resource(NextState(GameState::InGameAssetLoading));
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
                    let save_data: SaveData = savefile::load_file(path, 0).unwrap();
                    debug!("loaded save data {:?}", &save_data);
                    current_save.0.data = Some(save_data);
                }

                cmd.insert_resource(NextState(GameState::InGameAssetLoading));
            }
        }
    }
}

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
                        cmd.insert_resource(NextState(GameState::SaveMenu));
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

#[derive(Component, Debug)]
struct DeleteMode(bool);

#[derive(Component)]
struct DeleteModeButtonText;

#[derive(Clone, Debug, Default)]
pub(crate) struct Save {
    pub(crate) filename: String,
    pub(crate) path: Option<PathBuf>,
    pub(crate) data: Option<SaveData>,
}

#[derive(Clone, Copy, Debug, Savefile)]
pub(crate) struct SaveData {
    pub(crate) player_health: Health,
}

pub(crate) struct SaveEvent;

#[derive(Default, Deref, DerefMut)]
struct SaveSlots(Vec<Save>);

impl SaveSlots {
    fn new_save() -> Save {
        let current_saves = Self::get_saves();
        let path = PathBuf::from(format!("{}/save_{}.bin", SAVE_DIR, current_saves.len()));

        Save {
            filename: path.file_prefix().unwrap().to_str().unwrap().to_owned(),
            path: Some(path),
            data: None,
        }
    }

    fn get_saves() -> Self {
        let saves = if let Ok(entries) = fs::read_dir(SAVE_DIR) {
            let mut saves: Vec<PathBuf> = entries
                .filter(|e| e.is_ok())
                .map(|e| e.unwrap().path())
                .filter(|e| e.is_file())
                .collect();

            saves.sort();

            saves
                .into_iter()
                .map(|path| Save {
                    filename: path.file_prefix().unwrap().to_str().unwrap().to_owned(),
                    path: Some(path),
                    data: None,
                })
                .collect()
        } else {
            vec![]
        };

        Self(saves)
    }

    fn new(length: usize) -> Self {
        let mut saves = Self::get_saves();

        let mut new_saves: Vec<Save> = (saves.len()..length)
            .map(|_| Save {
                filename: "New Save".to_string(),
                path: None,
                data: None,
            })
            .collect();

        saves.append(&mut new_saves);

        saves
    }
}

#[derive(Debug, Default, Deref, DerefMut, Resource)]
pub(crate) struct CurrentSave(pub(crate) Save);

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentSave>()
            .add_event::<SaveEvent>()
            .add_enter_system(GameState::SaveMenu, save_menu)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::SaveMenu)
                    .with_system(NewSaveButton::create.run_if(button_interact::<NewSaveButton>))
                    .with_system(LoadSaveButton::load.run_if(button_interact::<LoadSaveButton>))
                    .with_system(BackButton::to_main_menu.run_if(button_interact::<BackButton>))
                    .with_system(DeleteButton::delete.run_if(button_interact::<DeleteButton>))
                    .with_system(
                        DeleteModeButton::toggle.run_if(button_interact::<DeleteModeButton>),
                    )
                    .with_system(update_menu)
                    .with_system(on_esc)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(save_system.run_if(on_save_event))
                    .into(),
            );
    }
}

fn on_save_event(save_events: EventReader<SaveEvent>) -> bool {
    if save_events.is_empty() {
        false
    } else {
        save_events.clear();
        true
    }
}

fn save_system(world: &mut World) {
    let mut system_state: SystemState<(Res<CurrentSave>, Query<&Health, With<Player>>)> =
        SystemState::new(world);

    let (current_save, player_query) = system_state.get(world);

    if let Some(savefile) = current_save.path.clone() {
        let player_health = player_query.single();

        let save_data = SaveData {
            player_health: *player_health,
        };
        IoTaskPool::get()
            .spawn(async move {
                debug!("saving {:?} to {:?}", save_data, savefile);
                fs::create_dir_all(SAVE_DIR).expect("Error while creating the *saves* folder");

                let tmp_file = format!("{}_tmp", savefile.display());

                savefile::save_file(&tmp_file, 0, &save_data).expect("Error while saving");

                if fs::rename(tmp_file, savefile).is_err() {
                    error!("cannot rename tmp save file");
                }
            })
            .detach();
    } else {
        debug!("cannot save {:?}", current_save);
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

    cmd.spawn(NodeBundle {
        style: Style {
            margin: UiRect::all(Val::Auto),
            flex_direction: FlexDirection::Row,
            ..default()
        },
        background_color: Color::CRIMSON.into(),
        ..default()
    })
    .insert(DeleteMode(false))
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
                SaveSlots::new(5).iter().for_each(|save| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                margin: UiRect::all(Val::Auto),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: Color::CRIMSON.into(),
                            ..default()
                        })
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
                .spawn(NodeBundle {
                    style: Style {
                        align_items: AlignItems::FlexEnd,
                        ..default()
                    },
                    background_color: Color::CRIMSON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(ButtonBundle {
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        })
                        .insert(DeleteModeButton)
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
                        .spawn(ButtonBundle {
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        })
                        .insert(DeleteButton(save_button.0.clone()))
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
