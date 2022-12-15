use bevy::{ecs::system::SystemState, prelude::*, tasks::IoTaskPool};
use std::{
    fs,
    io::{Cursor, Error, Write},
    path::{Path, PathBuf},
};

use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

use crate::{
    player::{Health, Player},
    state::{AppState, ConditionSet, IntoConditionalSystem},
};

const SAVE_DIR: &str = "saves";

#[derive(Clone, Debug, Default)]
pub(crate) struct Save {
    pub(crate) filename: String,
    pub(crate) path: Option<PathBuf>,
    pub(crate) data: Option<SaveData>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub(crate) struct SaveData {
    pub(crate) player_health: Health,
}

pub(crate) struct SaveEvent;

#[derive(Default, Deref, DerefMut)]
pub(crate) struct SaveSlots(pub(crate) Vec<Save>);

impl SaveSlots {
    pub(crate) fn new_save() -> Save {
        let current_saves = Self::get_saves();
        let path = PathBuf::from(format!("{}/save_{}.bin", SAVE_DIR, current_saves.len()));

        Save {
            filename: path.file_prefix().unwrap().to_str().unwrap().to_owned(),
            path: Some(path),
            data: None,
        }
    }

    pub(crate) fn get_saves() -> Self {
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

    pub(crate) fn new(length: usize) -> Self {
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

pub(crate) struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentSave>()
            .add_event::<SaveEvent>()
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(AppState::InGame)
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

                save_file(&tmp_file, 0, &save_data).expect("Error while saving");

                if fs::rename(tmp_file, savefile).is_err() {
                    error!("cannot rename tmp save file");
                }
            })
            .detach();
    } else {
        debug!("cannot save {:?}", current_save);
    }
}

pub(crate) fn save_file<P, T>(filepath: P, _version: u32, data: &T) -> Result<Vec<u8>, Error>
where
    T: Serialize,
    P: AsRef<Path>,
{
    let mut buf = vec![];
    data.serialize(&mut Serializer::new(&mut buf)).unwrap();
    fs::File::create(filepath).and_then(|mut file| file.write(&buf))?;
    Ok(buf)
}

pub(crate) fn load_file<'a, P, T>(filepath: P, _version: u32) -> anyhow::Result<T>
where
    T: Deserialize<'a>,
    P: AsRef<Path>,
{
    let buf = fs::read(filepath)?;

    let cur = Cursor::new(buf.as_slice());
    let mut de = Deserializer::new(cur);

    Ok(Deserialize::deserialize(&mut de)?)
}
