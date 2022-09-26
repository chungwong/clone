use std::{fs, path::Path};

use bevy::{
    app::AppExit, hierarchy::ChildBuilder, prelude::*, tasks::IoTaskPool, window::close_on_esc,
};

use crate::{
    asset::FontAssets,
    state::*,
    ui::options::{AudioConfig, OptionPlugin},
};

pub(crate) const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub(crate) const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub(crate) const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.25, 0.65, 0.25);
pub(crate) const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

pub(crate) const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

const CONFIG_DIR: &str = "data";
const CONFIG_FILENAME: &str = "config.bin";

pub(crate) fn get_button_style() -> Style {
    Style {
        size: Size::new(Val::Px(250.0), Val::Px(65.0)),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

#[derive(Component, Debug)]
struct OptionButton;

impl OptionButton {
    fn show(mut cmd: Commands) {
        cmd.insert_resource(NextState(GameState::OptionMenu));
    }
}

#[derive(Component, Debug)]
struct QuitButton;

impl QuitButton {
    fn exit(mut ev: EventWriter<AppExit>) {
        ev.send(AppExit);
    }

    pub(crate) fn spawn(parent: &mut ChildBuilder, button_text_style: TextStyle) {
        parent
            .spawn_bundle(ButtonBundle {
                style: get_button_style(),
                color: NORMAL_BUTTON.into(),
                ..default()
            })
            .insert(Self)
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle {
                    text: Text::from_section("Quit", button_text_style),
                    ..default()
                });
            });
    }
}

#[derive(Component, Debug)]
struct StartGameButton;

impl StartGameButton {
    fn start(mut cmd: Commands) {
        cmd.insert_resource(NextState(GameState::SaveMenu));
    }
}

#[derive(Component)]
struct ResumeButton;
impl ResumeButton {
    fn resume(mut cmd: Commands) {
        cmd.insert_resource(NextState(GameState::InGame));
    }
}

#[derive(Component)]
pub(crate) struct BackButton;
impl BackButton {
    pub(crate) fn to_main_menu(mut cmd: Commands) {
        cmd.insert_resource(NextState(GameState::MainMenu));
    }

    pub(crate) fn to_option_menu(mut cmd: Commands) {
        cmd.insert_resource(NextState(GameState::OptionMenu));
    }

    pub(crate) fn spawn(parent: &mut ChildBuilder, button_text_style: TextStyle) {
        parent
            .spawn_bundle(ButtonBundle {
                style: get_button_style(),
                color: NORMAL_BUTTON.into(),
                ..default()
            })
            .insert(Self)
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle {
                    text: Text::from_section("Back", button_text_style),
                    ..default()
                });
            });
    }
}

#[derive(Component, Debug)]
pub(crate) struct SelectedOption;

#[derive(Clone, Copy, Debug, Default, Savefile)]
pub(crate) struct GameConfig {
    pub(crate) audio: AudioConfig,
}

impl GameConfig {
    fn load(mut cmd: Commands) {
        if let Ok(config) = savefile::load_file::<GameConfig, _>(
            Path::new(&format!("{CONFIG_DIR}/{CONFIG_FILENAME}")),
            0,
        ) {
            info!("loaded save data {:?}", &config);
            cmd.insert_resource(config);
        } else {
            info!("config file not found");
        }
    }

    fn save(game_config: Res<GameConfig>) {
        let game_config = *game_config;

        IoTaskPool::get()
            .spawn(async move {
                let path = format!("{CONFIG_DIR}/{CONFIG_FILENAME}");
                let config_file = Path::new(&path);
                info!("saving {:?} to {:?}", game_config, config_file);
                fs::create_dir_all(CONFIG_DIR).expect("Error while creating the *data* folder");

                let tmp_file = format!("{}_tmp", config_file.display());

                savefile::save_file(&tmp_file, 0, &game_config)
                    .expect("Error while saving config file");

                if fs::rename(tmp_file, config_file).is_err() {
                    error!("cannot rename tmp config file");
                }
            })
            .detach();
    }
}

pub(crate) struct GameConfigSaveEvent;

pub(crate) struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameConfig>()
            .add_event::<GameConfigSaveEvent>()
            .add_plugin(OptionPlugin)
            .add_startup_system(GameConfig::load)
            .add_system(button_interact_visual)
            .add_system(GameConfig::save.run_on_event::<GameConfigSaveEvent>())
            .add_enter_system(GameState::MainMenu, setup_menu)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::MainMenu)
                    .with_system(close_on_esc)
                    .with_system(QuitButton::exit.run_if(button_interact::<QuitButton>))
                    .with_system(StartGameButton::start.run_if(button_interact::<StartGameButton>))
                    .with_system(OptionButton::show.run_if(button_interact::<OptionButton>))
                    .into(),
            )
            .add_enter_system(GameState::Paused, pause_menu)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Paused)
                    .with_system(ResumeButton::resume.run_if(button_interact::<ResumeButton>))
                    .with_system(QuitButton::exit.run_if(button_interact::<QuitButton>))
                    .into(),
            );
    }
}

fn setup_menu(mut cmd: Commands, font_assets: Res<FontAssets>) {
    cmd.spawn_bundle(Camera2dBundle::default());

    let font = font_assets.monogram.clone();

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
    .with_children(|parent| {
        // Display the game name
        parent.spawn_bundle(TextBundle {
            style: Style {
                margin: UiRect::all(Val::Px(50.0)),
                ..default()
            },
            text: Text::from_section(
                "Reckoning",
                TextStyle {
                    font: font.clone(),
                    font_size: 80.0,
                    color: TEXT_COLOR,
                },
            ),
            ..default()
        });

        // Main menu buttons
        parent
            .spawn_bundle(ButtonBundle {
                style: button_style.clone(),
                color: NORMAL_BUTTON.into(),
                ..default()
            })
            .insert(StartGameButton)
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle {
                    text: Text::from_section("Start Game", button_text_style.clone()),
                    ..default()
                });
            });

        parent
            .spawn_bundle(ButtonBundle {
                style: button_style.clone(),
                color: NORMAL_BUTTON.into(),
                ..default()
            })
            .insert(OptionButton)
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle {
                    text: Text::from_section("Option", button_text_style.clone()),
                    ..default()
                });
            });

        QuitButton::spawn(parent, button_text_style);
    });
}

// This system handles changing all buttons color based on mouse interaction
pub(crate) fn button_interact_visual(
    mut query: Query<
        (&Interaction, &mut UiColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, selected) in query.iter_mut() {
        *color = match (*interaction, selected) {
            (Interaction::Clicked, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}

/// Condition to help with handling multiple buttons
///
/// Returns true when a button identified by a given component is clicked.
pub(crate) fn button_interact<B: Component>(
    query: Query<&Interaction, (Changed<Interaction>, With<Button>, With<B>)>,
) -> bool {
    for interaction in query.iter() {
        if *interaction == Interaction::Clicked {
            return true;
        }
    }

    false
}

fn pause_menu(mut cmd: Commands, font_assets: Res<FontAssets>) {
    let font = font_assets.monogram.clone();

    let button_style = get_button_style();

    let button_text_style = TextStyle {
        font,
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
    .with_children(|parent| {
        parent
            .spawn_bundle(ButtonBundle {
                style: button_style.clone(),
                color: NORMAL_BUTTON.into(),
                ..default()
            })
            .insert(ResumeButton)
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle {
                    text: Text::from_section("Resume", button_text_style.clone()),
                    ..default()
                });
            });

        QuitButton::spawn(parent, button_text_style.clone());
    });
}

pub(crate) fn on_esc(
    mut cmd: Commands,
    keys: Res<Input<KeyCode>>,
    state: Res<CurrentState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Escape) && state.0 != GameState::MainMenu {
        cmd.insert_resource(NextState(GameState::MainMenu));
    }
}
