use bevy::prelude::*;
use iyes_loopless::prelude::AppLooplessStateExt;
pub use leafwing_input_manager::{
    prelude::{ActionState, Actionlike, InputManagerPlugin, InputMap, ToggleActions},
    user_input::{InputKind, UserInput},
};
use serde::{Deserialize, Serialize};

use crate::state::PauseState;

#[derive(Actionlike, Clone, Copy, Eq, Debug, Hash, PartialEq, Serialize, Deserialize)]
pub(crate) enum ControlAction {
    // Directions
    Down,
    Left,
    Right,
    Up,

    // Actions
    Jump,
    Attack,
}

pub(crate) type ControlActionState = ActionState<ControlAction>;
pub(crate) type ControlInputMap = InputMap<ControlAction>;

impl ControlAction {
    pub(crate) fn get_input_map() -> ControlInputMap {
        InputMap::new([
            (KeyCode::Space, Self::Jump),
            (KeyCode::A, Self::Left),
            (KeyCode::D, Self::Right),
            (KeyCode::W, Self::Up),
            (KeyCode::S, Self::Down),
            (KeyCode::J, Self::Attack),
        ])
    }
}

#[derive(Bundle, Clone)]
pub(crate) struct ControlInputManagerBundle {
    action_state: ControlActionState,
    input_map: ControlInputMap,
}

impl ControlInputManagerBundle {
    pub(crate) fn with_input_map(mut self, input_map: ControlInputMap) -> Self {
        self.input_map = input_map;
        self
    }
}

impl Default for ControlInputManagerBundle {
    fn default() -> Self {
        Self {
            action_state: ControlActionState::default(),
            input_map: ControlAction::get_input_map(),
        }
    }
}

pub(crate) type UiActionState = ActionState<UiAction>;
pub(crate) type MenuInputMap = InputMap<UiAction>;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub(crate) enum UiAction {
    Pause,
}

impl UiAction {
    pub(crate) fn get_input_map() -> MenuInputMap {
        InputMap::new([(KeyCode::Escape, Self::Pause)])
    }
}

#[derive(Bundle, Clone)]
pub(crate) struct MenuInputManagerBundle {
    action_state: UiActionState,
    input_map: MenuInputMap,
}

impl Default for MenuInputManagerBundle {
    fn default() -> Self {
        Self {
            action_state: UiActionState::default(),
            input_map: UiAction::get_input_map(),
        }
    }
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<UiAction>::default())
            .add_plugin(InputManagerPlugin::<ControlAction>::default())
            .add_enter_system(PauseState::On, pause_player_action)
            .add_exit_system(PauseState::On, resume_player_action)
            .add_startup_system(setup);
    }
}

fn pause_player_action(mut toggle_actions: ResMut<ToggleActions<ControlAction>>) {
    toggle_actions.enabled = false;
}

fn resume_player_action(mut toggle_actions: ResMut<ToggleActions<ControlAction>>) {
    toggle_actions.enabled = true;
}

fn setup(mut cmd: Commands) {
    cmd.insert_resource(ControlAction::get_input_map());
    cmd.insert_resource(ControlActionState::default());
}
