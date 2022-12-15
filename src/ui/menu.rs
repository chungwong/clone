use std::{fs, path::Path};

use bevy::{
    app::AppExit, hierarchy::ChildBuilder, prelude::*, tasks::IoTaskPool, window::close_on_esc,
};

use serde::{Deserialize, Serialize};

use crate::{
    asset::FontAssets,
    input::{UiAction, UiActionState},
    physics::{pause_physics, resume_physics, RapierConfiguration},
    save::{load_file, save_file},
    state::*,
    ui::{
        audio::AudioConfig,
        control::{BindingState, ControlConfig},
        options::OptionPlugin,
        save::SaveMenuPlugin,
    },
};

pub(crate) const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub(crate) const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub(crate) const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.25, 0.65, 0.25);
pub(crate) const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

pub(crate) const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

const CONFIG_DIR: &str = "data";
const CONFIG_FILENAME: &str = "config.bin";

#[derive(Component)]
pub(crate) struct Despawnable;

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
struct OptionsButton;

impl OptionsButton {
    fn show(mut cmd: Commands) {
        cmd.insert_resource(NextState(MenuState::Options));
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
            .spawn((
                Name::new("Quit"),
                Self,
                ButtonBundle {
                    style: get_button_style(),
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle {
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
        cmd.insert_resource(NextState(AppState::SaveMenu));
    }
}

#[derive(Component)]
struct ResumeButton;
impl ResumeButton {
    fn resume(mut rapier_config: ResMut<RapierConfiguration>, mut cmd: Commands) {
        resume_physics(&mut rapier_config);
        cmd.insert_resource(NextState(PauseState::Off));
    }
}

#[derive(Component)]
struct OptionsMenuButton;

#[derive(Component)]
pub(crate) struct MainMenuButton;
impl MainMenuButton {
    pub(crate) fn back_to_main_menu(mut cmd: Commands) {
        cmd.insert_resource(NextState(AppState::MainMenu));
    }
}

#[derive(Component)]
pub(crate) struct BackButton;
impl BackButton {
    pub(crate) fn to_main_menu(mut cmd: Commands) {
        cmd.insert_resource(NextState(MenuState::None));
    }

    pub(crate) fn to_options_menu(mut cmd: Commands) {
        cmd.insert_resource(NextState(MenuState::Options));
    }

    pub(crate) fn on_esc_to_options_menu(
        mut cmd: Commands,
        keys: Res<Input<KeyCode>>,
        menu_state: Res<CurrentState<MenuState>>,
        binding_state: Res<CurrentState<BindingState>>,
    ) {
        if keys.just_pressed(KeyCode::Escape)
            && menu_state.0 != MenuState::Options
            && binding_state.0 == BindingState::None
        {
            cmd.insert_resource(NextState(MenuState::Options));
        }
    }

    pub(crate) fn spawn(parent: &mut ChildBuilder, button_text_style: TextStyle) {
        parent
            .spawn((
                Name::new("Back"),
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
                    .spawn((
                        Name::new("Back Button"),
                        Self,
                        ButtonBundle {
                            style: get_button_style(),
                            background_color: NORMAL_BUTTON.into(),
                            ..default()
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle {
                            text: Text::from_section("Back", button_text_style),
                            ..default()
                        });
                    });
            });
    }
}

#[derive(Component, Debug)]
pub(crate) struct SelectedOption;

#[derive(Clone, Debug, Default, Resource, Deserialize, Serialize)]
pub(crate) struct GameConfig {
    pub(crate) audio: AudioConfig,
    pub(crate) control: ControlConfig,
}

impl GameConfig {
    fn load(mut cmd: Commands) {
        if let Ok(config) =
            load_file::<_, GameConfig>(Path::new(&format!("{CONFIG_DIR}/{CONFIG_FILENAME}")), 0)
        {
            info!("loaded save data {:?}", &config);
            cmd.insert_resource(config);
        } else {
            info!("config file not found");
        }
    }

    fn save(game_config: Res<GameConfig>) {
        let game_config = game_config.clone();

        IoTaskPool::get()
            .spawn(async move {
                let path = format!("{CONFIG_DIR}/{CONFIG_FILENAME}");
                let config_file = Path::new(&path);
                info!("saving {:?} to {:?}", game_config, config_file);
                fs::create_dir_all(CONFIG_DIR).expect("Error while creating the *data* folder");

                let tmp_file = format!("{}_tmp", config_file.display());

                save_file(&tmp_file, 0, &game_config).expect("Error while saving config file");

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
            .add_plugin(SaveMenuPlugin)
            .add_startup_system(GameConfig::load)
            .add_system(button_interact_visual)
            .add_system(GameConfig::save.run_on_event::<GameConfigSaveEvent>())
            .add_enter_system(AppState::MainMenu, setup_menu)
            .add_enter_system(AppState::InGame, init)
            .add_exit_system(AppState::InGame, clean_up)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(AppState::MainMenu)
                    .with_system(close_on_esc)
                    .with_system(QuitButton::exit.run_if(button_interact::<QuitButton>))
                    .with_system(StartGameButton::start.run_if(button_interact::<StartGameButton>))
                    .with_system(OptionsButton::show.run_if(button_interact::<OptionsButton>))
                    .into(),
            )
            .add_enter_system(PauseState::On, pause_menu)
            .add_exit_system(PauseState::On, despawn::<Despawnable>)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(PauseState::Off)
                    .with_system(pause)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(PauseState::On)
                    .with_system(ResumeButton::resume.run_if(button_interact::<ResumeButton>))
                    .with_system(
                        MainMenuButton::back_to_main_menu.run_if(button_interact::<MainMenuButton>),
                    )
                    .with_system(
                        BackButton::to_options_menu.run_if(button_interact::<OptionsMenuButton>),
                    )
                    .with_system(QuitButton::exit.run_if(button_interact::<QuitButton>))
                    .with_system(ResumeButton::resume.run_if(on_esc_pause_menu))
                    .into(),
            )
            .add_enter_system(MenuState::None, despawn::<Despawnable>)
            .add_exit_system(MenuState::Audio, despawn::<Despawnable>)
            .add_exit_system(MenuState::Controls, despawn::<Despawnable>)
            .add_exit_system(MenuState::Options, despawn::<Despawnable>)
            .add_exit_system(MenuState::Save, despawn::<Despawnable>);
    }
}

fn init(mut cmd: Commands) {
    cmd.insert_resource(NextState(PauseState::Off));
}

fn clean_up(mut cmd: Commands) {
    cmd.insert_resource(NextState(PauseState::None));
}

fn setup_menu(mut cmd: Commands, font_assets: Res<FontAssets>) {
    cmd.spawn(Camera2dBundle::default());

    let font = font_assets.monogram.clone();

    let button_style = get_button_style();

    let button_text_style = TextStyle {
        font: font.clone(),
        font_size: 40.0,
        color: TEXT_COLOR,
    };

    cmd.spawn((
        Name::new("Main Menu"),
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
        // Display the game name
        parent.spawn((
            Name::new("Title"),
            TextBundle {
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
            },
        ));

        // Main menu buttons
        parent
            .spawn((
                Name::new("StartGame"),
                StartGameButton,
                ButtonBundle {
                    style: button_style.clone(),
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section("Start Game", button_text_style.clone()),
                    ..default()
                });
            });

        parent
            .spawn((
                Name::new("Options"),
                ButtonBundle {
                    style: button_style.clone(),
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                },
                OptionsButton,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle {
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
        (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
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

    cmd.spawn((
        Name::new("Pause Menu"),
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
        Despawnable,
    ))
    .with_children(|parent| {
        parent
            .spawn((
                Name::new("Resume Button"),
                ResumeButton,
                ButtonBundle {
                    style: button_style.clone(),
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section("Resume", button_text_style.clone()),
                    ..default()
                });
            });

        parent
            .spawn((
                Name::new("Options Button"),
                OptionsMenuButton,
                ButtonBundle {
                    style: button_style.clone(),
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section("Options", button_text_style.clone()),
                    ..default()
                });
            });

        parent
            .spawn((
                Name::new("Main Menu Button"),
                MainMenuButton,
                ButtonBundle {
                    style: button_style.clone(),
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section("Main Menu", button_text_style.clone()),
                    ..default()
                });
            });

        QuitButton::spawn(parent, button_text_style.clone());
    });
}

/// Pressing escape on screens returns to Main Menu
pub(crate) fn on_esc_main_menu(
    mut cmd: Commands,
    keys: Res<Input<KeyCode>>,
    state: Res<CurrentState<MenuState>>,
) {
    if keys.just_pressed(KeyCode::Escape) && state.0 != MenuState::None {
        cmd.insert_resource(NextState(MenuState::None));
    }
}

/// Pressing escape in PauseState::On hides pause menu
fn on_esc_pause_menu(keys: Res<Input<KeyCode>>) -> bool {
    keys.just_pressed(KeyCode::Escape)
}

fn pause(
    mut rapier_config: ResMut<RapierConfiguration>,
    mut cmd: Commands,
    input: Query<&UiActionState>,
) {
    let input = input.single();
    if input.just_pressed(UiAction::Pause) {
        pause_physics(&mut rapier_config);
        cmd.insert_resource(NextState(PauseState::On));
    }
}

pub(crate) fn despawn<T: Component>(mut cmd: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        cmd.entity(entity).despawn_recursive();
    }
}

pub(crate) trait ConfigButton {
    fn save(&self, _: &mut ResMut<GameConfig>);
}

pub(crate) fn select_button<T: Component + PartialEq + ConfigButton>(
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
