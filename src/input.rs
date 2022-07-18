use bevy::prelude::*;
pub use leafwing_input_manager::prelude::{ActionState, Actionlike, InputManagerPlugin, InputMap};

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub(crate) enum PlayerAction {
    Jump,
    Left,
    Right,
    Up,
    Down,
    Attack,
}

pub(crate) type PlayerActionState = ActionState<PlayerAction>;
pub(crate) type PlayerInputMap = InputMap<PlayerAction>;

impl PlayerAction {
    pub(crate) fn get_input_map() -> PlayerInputMap {
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
pub(crate) struct PlayerInputManagerBundle {
    action_state: PlayerActionState,
    input_map: PlayerInputMap,
}

impl Default for PlayerInputManagerBundle {
    fn default() -> Self {
        Self {
            action_state: PlayerActionState::default(),
            input_map: PlayerAction::get_input_map(),
        }
    }
}

pub(crate) type MenuActionState = ActionState<MenuAction>;
pub(crate) type MenuInputMap = InputMap<MenuAction>;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub(crate) enum MenuAction {
    Pause,
}

impl MenuAction {
    pub(crate) fn get_input_map() -> MenuInputMap {
        InputMap::new([(KeyCode::Escape, Self::Pause)])
    }
}

#[derive(Bundle, Clone)]
pub(crate) struct MenuInputManagerBundle {
    action_state: MenuActionState,
    input_map: MenuInputMap,
}

impl Default for MenuInputManagerBundle {
    fn default() -> Self {
        Self {
            action_state: MenuActionState::default(),
            input_map: MenuAction::get_input_map(),
        }
    }
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<MenuAction>::default())
            .add_plugin(InputManagerPlugin::<PlayerAction>::default());
    }
}
