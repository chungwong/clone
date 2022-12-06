#![allow(clippy::forget_non_drop)]

use std::convert::TryFrom;
use std::time::Duration;

use bevy::{prelude::*, utils::Instant};

use move_vis::TrackMovement;

use crate::{
    input::PlayerInputManagerBundle,
    physics::*,
    state::{ConditionSet, GameState, PauseState},
    tilemap::{EntityInstance, FieldValue, Worldly},
};

pub(crate) mod systems;

// how many pixels
// with damping applied, values are not accurate anymore
const JUMP_HEIGHT: f32 = 4.0;

// how long does it take to reach the maximum height of a jump?
// note: if "jump_power_coefficient" is not a multiple of "g" the maximum height is reached between frames
// second
const TIME_TO_APEX: f32 = 0.4;

const DEFAULT_GRAVITY_SCALE: f32 = 5.0;

#[derive(Clone, Component, Copy, Debug, Default, Reflect, Savefile)]
#[reflect(Component)]
pub(crate) struct Health {
    pub(crate) current: u32,
    pub(crate) max: u32,
}

impl Health {
    pub(crate) fn new(hp: u32) -> Self {
        Self {
            current: hp,
            max: hp,
        }
    }
}

impl From<&EntityInstance> for Health {
    fn from(entity_instance: &EntityInstance) -> Self {
        let field_instances = entity_instance
            .field_instances
            .iter()
            .find(|f| f.identifier == "HP")
            .unwrap_or_else(|| panic!("HP not set for {entity_instance:?}"));

        match field_instances.value {
            FieldValue::Int(Some(hp)) if hp > 0 => Self::new(hp as u32),
            FieldValue::Int(Some(hp)) if hp == 0 => {
                panic!("HP cannot be 0 {field_instances:?}")
            }
            _ => panic!("Wrong HP type {field_instances:?}"),
        }
    }
}

impl From<EntityInstance> for Health {
    fn from(entity_instance: EntityInstance) -> Self {
        Self::from(&entity_instance)
    }
}

#[derive(Debug)]
pub(crate) struct DeathEvent(Entity);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Direction {
    Left = -1,
    Neutral = 0,
    Right = 1,
}

impl Direction {
    pub(crate) fn to_f32(self) -> f32 {
        match self {
            Self::Left => -1.0,
            Self::Right => 1.0,
            Self::Neutral => 0.0,
        }
    }
}

impl Default for Direction {
    fn default() -> Self {
        Direction::Neutral
    }
}

impl TryFrom<f32> for Direction {
    type Error = ();

    fn try_from(v: f32) -> Result<Self, Self::Error> {
        match v as isize {
            -1 => Ok(Direction::Left),
            1 => Ok(Direction::Right),
            0 => Ok(Direction::Neutral),
            _ => Err(()),
        }
    }
}

#[derive(Resource)]
pub(crate) struct DashInput {
    input_timer: Timer,
    direction: Direction,
}

impl Default for DashInput {
    fn default() -> Self {
        Self {
            input_timer: Timer::new(Duration::from_millis(200), TimerMode::Once),
            direction: Direction::Neutral,
        }
    }
}

#[derive(Default, Resource)]
pub(crate) struct PlayerMovementSettings {
    // metre
    pub(crate) jump_height: f32,
    // second
    pub(crate) time_to_apex: f32,
    pub(crate) run_speed: f32,
    pub(crate) dash_speed: f32,
    // pub(crate) jump_impulse: f32,
    pub(crate) jump_power_coefficient: f32,
    pub(crate) coyote_time_ms: u64,
    // pub(crate) jump_power_coefficient: f32,
    pub(crate) slide_factor: f32,
    pub(crate) fall_factor: f32,
    pub(crate) jump_break_factor: f32,
    pub(crate) gravity_scale: f32,
}

#[derive(Clone, Debug)]
pub(crate) enum JumpStatus {
    CanJump,
    InitiateJump,
    GoingUp,
    StoppingUp,
    Falling,
    WallSliding,
}

#[derive(Clone, Component, Debug)]
pub(crate) struct Player {
    pub(crate) dashing: bool,
    pub(crate) facing_direction: Direction,
    pub(crate) rising: bool,
    pub(crate) is_jumping: bool,
    pub(crate) wall_sliding: bool,
    pub(crate) last_stood_normal: Vec2,
    pub(crate) last_stood_time: Option<Instant>,
    pub(crate) jump_status: JumpStatus,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            dashing: false,
            facing_direction: Direction::Right,
            rising: false,
            is_jumping: false,
            wall_sliding: false,
            last_stood_normal: Vec2::Y,
            last_stood_time: None,
            jump_status: JumpStatus::CanJump,
        }
    }
}

#[derive(Bundle, Clone, Default)]
pub(crate) struct PlayerPhysicsBundle {
    pub(crate) collider: Collider,
    pub(crate) collider_mass_properties: ColliderMassProperties,
    pub(crate) damping: Damping,
    pub(crate) external_impulse: ExternalImpulse,
    pub(crate) external_force: ExternalForce,
    pub(crate) gravity_scale: GravityScale,
    pub(crate) locked_axes: LockedAxes,
    pub(crate) rigid_body: RigidBody,
    pub(crate) velocity: Velocity,
    pub(crate) ccd: Ccd,
}

#[derive(Clone, Bundle, Default)]
pub(crate) struct PlayerBundle {
    #[bundle]
    pub(crate) sprite_bundle: SpriteBundle,

    #[bundle]
    input_manager: PlayerInputManagerBundle,

    #[bundle]
    pub(crate) player_physics_bundle: PlayerPhysicsBundle,

    pub(crate) track_movement: TrackMovement,

    pub(crate) player: Player,

    pub(crate) entity_instance: EntityInstance,

    pub(crate) worldly: Worldly,

    pub(crate) hp: Health,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, SystemLabel)]
pub(crate) enum Label {
    Initial,
    CheckStanding,
    DeathSystems,
    Movement,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DashInput>()
            .insert_resource(PlayerMovementSettings {
                jump_height: JUMP_HEIGHT,
                time_to_apex: TIME_TO_APEX,
                run_speed: 500.0,
                dash_speed: 10000.0,
                // jump_impulse: 20000.0,
                jump_power_coefficient: 20000.0,
                coyote_time_ms: 100,
                slide_factor: 60.0,
                fall_factor: 100.0,
                jump_break_factor: 200.0,
                gravity_scale: DEFAULT_GRAVITY_SCALE,
            })
            .add_event::<DeathEvent>()
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .run_if_not(PauseState::is_paused)
                    .label(Label::Initial)
                    .with_system(systems::spawn_player)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .run_if_not(PauseState::is_paused)
                    .label(Label::CheckStanding)
                    .after(Label::Initial)
                    .with_system(systems::check_standing)
                    .with_system(systems::set_facing_direction)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .run_if_not(PauseState::is_paused)
                    .label(Label::Movement)
                    .after(Label::CheckStanding)
                    .with_system(systems::dash)
                    .with_system(systems::run)
                    .with_system(systems::jump)
                    .with_system(systems::attack)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .label(Label::DeathSystems)
                    .run_in_state(GameState::InGame)
                    .run_if_not(PauseState::is_paused)
                    .with_system(systems::hp_death)
                    .with_system(systems::fall_death)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .run_if_not(PauseState::is_paused)
                    .after(Label::DeathSystems)
                    .with_system(systems::process_death_event)
                    .into(),
            )
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .run_if_not(PauseState::is_paused)
                    .with_system(systems::boundary)
                    .into(),
            );
    }
}
