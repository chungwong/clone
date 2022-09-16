use bevy::{app::AppExit, prelude::*, window::close_on_esc};

use crate::state::*;

pub(crate) const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub(crate) const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub(crate) const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

pub(crate) const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

#[derive(Component)]
struct QuitButton;

#[derive(Component)]
struct StartGameButton;

#[derive(Component)]
struct ResumeButton;

#[derive(Component)]
struct OnMainMenuScreen;

#[derive(Component)]
struct OnPauseScreen;

pub(crate) struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(button_interact_visual)
            .add_enter_system(GameState::MainMenu, setup_menu)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::MainMenu)
                    .with_system(close_on_esc)
                    .with_system(button_exit.run_if(button_interact::<QuitButton>))
                    .with_system(button_game.run_if(button_interact::<StartGameButton>))
                    .into(),
            )
            .add_exit_system(GameState::MainMenu, despawn_with::<OnMainMenuScreen>)
            .add_enter_system(GameState::Paused, pause_menu)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::Paused)
                    .with_system(button_resume.run_if(button_interact::<ResumeButton>))
                    .with_system(button_exit.run_if(button_interact::<QuitButton>))
                    .into(),
            )
            .add_exit_system(GameState::Paused, despawn_with::<OnPauseScreen>);
    }
}

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/monogram.ttf");

    let button_style = Style {
        size: Size::new(Val::Px(250.0), Val::Px(65.0)),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font: font.clone(),
        font_size: 40.0,
        color: TEXT_COLOR,
    };

    commands
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
        .insert(OnMainMenuScreen)
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

            // Display three buttons for each action available from the main menu:
            // - new game
            // - quit
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
                    style: button_style,
                    color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .insert(QuitButton)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::from_section("Quit", button_text_style),
                        ..default()
                    });
                });
        });
}

// This system handles changing all buttons color based on mouse interaction
pub(crate) fn button_interact_visual(
    mut query: Query<(&Interaction, &mut UiColor), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut color) in query.iter_mut() {
        *color = match *interaction {
            Interaction::Clicked => PRESSED_BUTTON.into(),
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
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

/// Handler for the Exit Game button
fn button_exit(mut ev: EventWriter<AppExit>) {
    ev.send(AppExit);
}

/// Handler for the Start Game button
fn button_game(mut commands: Commands) {
    //commands.insert_resource(NextState(GameState::LoadingLevel));
    commands.insert_resource(NextState(GameState::SaveMenu));
}

/// Handler for the resume Game button
fn button_resume(mut cmd: Commands) {
    cmd.insert_resource(NextState(GameState::InGame));
}

fn pause_menu(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/monogram.ttf");

    let button_style = Style {
        size: Size::new(Val::Px(250.0), Val::Px(65.0)),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
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
    .insert(OnPauseScreen)
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
        parent
            .spawn_bundle(ButtonBundle {
                style: button_style,
                color: NORMAL_BUTTON.into(),
                ..default()
            })
            .insert(QuitButton)
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle {
                    text: Text::from_section("Quit", button_text_style),
                    ..default()
                });
            });
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
