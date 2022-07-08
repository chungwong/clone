use bevy::prelude::*;
pub use leafwing_input_manager::prelude::{
    ActionState as OriginalActionState, Actionlike, InputManagerPlugin,
    InputMap as OriginalInputMap,
};

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub(crate) enum Action {
    Jump,
    Left,
    Right,
    Up,
    Down,
    Attack,
}

pub(crate) type ActionState = OriginalActionState<Action>;
pub(crate) type InputMap = OriginalInputMap<Action>;

impl Action {
    pub(crate) fn get_input_map() -> InputMap {
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

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<Action>::default());
    }
}
