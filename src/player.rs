#![allow(clippy::forget_non_drop)]

use std::convert::TryFrom;
use std::time::Duration;

use bevy::{prelude::*, utils::Instant};

use move_vis::TrackMovement;

use crate::input::{Action, ActionState, InputMap};
use crate::physics::*;
use crate::state::{ConditionSet, GameState};
use crate::tilemap::{EntityInstance, FieldValue, LdtkEntity, LdtkIntCell, Worldly};

mod systems;

// how many pixels
// with damping applied, values are not accurate anymore
const JUMP_HEIGHT: f32 = 4.0;

// how long does it take to reach the maximum height of a jump?
// note: if "jump_power_coefficient" is not a multiple of "g" the maximum height is reached between frames
// second
const TIME_TO_APEX: f32 = 0.4;

const DEFAULT_GRAVITY_SCALE: f32 = 5.0;

#[derive(Clone, Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub(crate) struct Health {
    pub(crate) current: u32,
    pub(crate) max: u32,
}

impl Health {
    pub fn new(hp: u32) -> Self {
        Self {
            current: hp,
            max: hp,
        }
    }
}

impl From<EntityInstance> for Health {
    fn from(entity_instance: EntityInstance) -> Self {
        let field_instances = entity_instance
            .field_instances
            .iter()
            .find(|f| f.identifier == "HP")
            .unwrap_or_else(|| panic!("HP not set for {:?}", entity_instance));

        match field_instances.value {
            FieldValue::Int(Some(hp)) if hp > 0 => Self::new(hp as u32),
            FieldValue::Int(Some(hp)) if hp == 0 => {
                panic!("{}", &format!("HP cannot be 0 {:?}", field_instances))
            }
            _ => panic!("{}", &format!("Wrong HP type {:?}", field_instances)),
        }
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
pub(crate) struct DashInput {
    input_timer: Timer,
    direction: Direction,
}

impl Default for DashInput {
    fn default() -> Self {
        Self {
            input_timer: Timer::new(Duration::from_millis(200), false),
            direction: Direction::Neutral,
        }
    }
}

#[derive(Default)]
pub struct PlayerMovementSettings {
    // metre
    pub jump_height: f32,
    // second
    pub time_to_apex: f32,
    pub run_speed: f32,
    pub dash_speed: f32,
    // pub jump_impulse: f32,
    pub jump_power_coefficient: f32,
    pub coyote_time_ms: u64,
    // pub jump_power_coefficient: f32,
    pub slide_factor: f32,
    pub fall_factor: f32,
    pub jump_break_factor: f32,
    pub gravity_scale: f32,
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

#[derive(Bundle, Clone, Default, LdtkIntCell)]
pub struct PlayerPhysicsBundle {
    pub collider: Collider,
    pub collider_mass_properties: ColliderMassProperties,
    pub damping: Damping,
    pub external_impulse: ExternalImpulse,
    pub external_force: ExternalForce,
    pub gravity_scale: GravityScale,
    pub locked_axes: LockedAxes,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub ccd: Ccd,
}

impl From<EntityInstance> for PlayerPhysicsBundle {
    fn from(entity_instance: EntityInstance) -> PlayerPhysicsBundle {
        Self {
            collider: Collider::cuboid(
                entity_instance.width as f32 / 2.0,
                entity_instance.height as f32 / 2.0,
            ),
            collider_mass_properties: ColliderMassProperties::Density(1.0),
            damping: Damping {
                linear_damping: 10.0,
                ..default()
            },
            external_impulse: ExternalImpulse::default(),
            external_force: ExternalForce::default(),
            gravity_scale: GravityScale(1.0),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            rigid_body: RigidBody::Dynamic,
            velocity: Velocity::zero(),
            ccd: Ccd::enabled(),
        }
    }
}

#[derive(Bundle, Clone)]
pub struct InputManagerBundle {
    action_state: ActionState,
    input_map: InputMap,
}

impl Default for InputManagerBundle {
    fn default() -> Self {
        Self {
            action_state: ActionState::default(),
            input_map: Action::get_input_map(),
        }
    }
}

#[derive(Clone, Bundle, LdtkEntity)]
pub(crate) struct PlayerBundle {
    #[sprite_bundle("player.png")]
    #[bundle]
    pub sprite_bundle: SpriteBundle,

    #[bundle]
    input_manager: InputManagerBundle,

    #[from_entity_instance]
    #[bundle]
    pub player_physics_bundle: PlayerPhysicsBundle,

    pub track_movement: TrackMovement,

    pub(crate) player: Player,

    #[from_entity_instance]
    entity_instance: EntityInstance,

    #[worldly]
    pub worldly: Worldly,

    #[from_entity_instance]
    pub(crate) hp: Health,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, SystemLabel)]
pub enum Label {
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
                    .label(Label::CheckStanding)
                    .with_system(systems::check_standing)
                    .with_system(systems::set_facing_direction)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
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
                    .with_system(systems::hp_death)
                    .with_system(systems::fall_death)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .after(Label::DeathSystems)
                    .with_system(systems::process_death_event)
                    .into(),
            )
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(systems::boundary)
                    .into(),
            );
    }
}
