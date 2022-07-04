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
            (Self::Jump, KeyCode::Space),
            (Self::Left, KeyCode::A),
            (Self::Right, KeyCode::D),
            (Self::Up, KeyCode::W),
            (Self::Down, KeyCode::S),
            (Self::Attack, KeyCode::J),
        ])
    }
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<Action>::default());
    }
}
