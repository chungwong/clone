#![allow(clippy::forget_non_drop)]

use bevy::prelude::*;

use crate::{
    physics::*,
    player::Health,
    state::{ConditionSet, GameState},
    tilemap::{
        ldtk_pixel_coords_to_translation_pivoted, EntityInstance, FieldValue, LayerInstance,
        LdtkLevel,
    },
};

#[derive(Clone, Component, Default)]
pub(crate) struct Enemy;

impl Enemy {
    fn despawn(
        mut cmd: Commands,
        enemies: Query<(Entity, &Health), (With<Enemy>, Changed<Health>)>,
    ) {
        for (entity, health) in enemies.iter() {
            if health.current == 0 {
                cmd.entity(entity).despawn_recursive();
            }
        }
    }

    fn spawn_npc(
        mut cmd: Commands,
        entity_query: Query<(Entity, &Transform, &EntityInstance), Added<EntityInstance>>,
        level_query: Query<&Handle<LdtkLevel>>,
        levels: Res<Assets<LdtkLevel>>,
        asset_server: Res<AssetServer>,
    ) {
        for (entity, transform, entity_instance) in entity_query.iter() {
            if entity_instance.identifier == *"Mob" {
                let layer_instance: &LayerInstance = level_query
                    .iter()
                    .find_map(|handle| {
                        levels
                            .get(handle)
                            .expect("Level should be loaded by this point")
                            .level
                            .layer_instances
                            .iter()
                            .flatten()
                            .find(|layer| layer.identifier == "Mobs")
                    })
                    .expect("Mobs layer is not found in Ldtk file");

                cmd.entity(entity).insert_bundle(MobBundle {
                    sprite_bundle: SpriteBundle {
                        sprite: Sprite {
                            color: Color::RED,
                            ..default()
                        },
                        texture: asset_server.load("player.png"),
                        transform: *transform,
                        ..default()
                    },
                    collider_bundle: entity_instance.into(),
                    hp: entity_instance.into(),
                    patrol: Patrol::new(entity_instance, layer_instance),
                    ..default()
                });
            }
        }
    }
}

#[derive(Bundle, Clone, Default)]
pub(crate) struct MobPhysicsBundle {
    pub(crate) collider: Collider,
    pub(crate) collider_mass_properties: ColliderMassProperties,
    pub(crate) damping: Damping,
    pub(crate) external_impulse: ExternalImpulse,
    pub(crate) external_force: ExternalForce,
    pub(crate) gravity_scale: GravityScale,
    pub(crate) locked_axes: LockedAxes,
    pub(crate) rigid_body: RigidBody,
    pub(crate) sensor: Sensor,
    pub(crate) velocity: Velocity,
    pub(crate) ccd: Ccd,
}

impl From<&EntityInstance> for MobPhysicsBundle {
    fn from(entity_instance: &EntityInstance) -> Self {
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
            sensor: Sensor,
            velocity: Velocity::zero(),
            ccd: Ccd::enabled(),
        }
    }
}

#[derive(Clone, Default, Bundle)]
pub(crate) struct MobBundle {
    #[bundle]
    pub(crate) sprite_bundle: SpriteBundle,

    #[bundle]
    pub(crate) collider_bundle: MobPhysicsBundle,

    pub(crate) enemy: Enemy,

    pub(crate) hp: Health,
    pub(crate) patrol: Patrol,
}

#[derive(Clone, PartialEq, Debug, Default, Component)]
pub(crate) struct Patrol {
    pub(crate) points: Vec<Vec2>,
    pub(crate) index: usize,
    pub(crate) forward: bool,
}

impl Patrol {
    fn new(entity_instance: &EntityInstance, layer_instance: &LayerInstance) -> Self {
        let mut points = vec![ldtk_pixel_coords_to_translation_pivoted(
            entity_instance.px,
            layer_instance.c_hei * layer_instance.grid_size,
            IVec2::new(entity_instance.width, entity_instance.height),
            entity_instance.pivot,
        )];

        if let Some(ldtk_patrol) = entity_instance
            .field_instances
            .iter()
            .find(|f| f.identifier == *"Patrol")
        {
            if let FieldValue::Points(ldtk_points) = &ldtk_patrol.value {
                for ldtk_point in ldtk_points.iter().flatten() {
                    // The +1 is necessary here due to the pivot of the entities in the sample
                    // file.
                    // The patrols set up in the file look flat and grounded,
                    // but technically they're not if you consider the pivot,
                    // which is at the bottom-center for the mobs.
                    let pixel_coords = (ldtk_point.as_vec2() + Vec2::new(0.5, 1.))
                        * Vec2::splat(layer_instance.grid_size as f32);

                    points.push(ldtk_pixel_coords_to_translation_pivoted(
                        pixel_coords.as_ivec2(),
                        layer_instance.c_hei * layer_instance.grid_size,
                        IVec2::new(entity_instance.width, entity_instance.height),
                        entity_instance.pivot,
                    ));
                }
            }
        };

        Self {
            points,
            index: 1,
            forward: true,
        }
    }

    fn patrol(mut query: Query<(&mut Transform, &mut Velocity, &mut Patrol)>) {
        for (mut transform, mut velocity, mut patrol) in query.iter_mut() {
            if patrol.points.len() <= 1 {
                continue;
            }

            let mut new_velocity =
                (patrol.points[patrol.index] - transform.translation.truncate()).normalize() * 75.0;

            if new_velocity.dot(velocity.linvel) < 0. {
                if patrol.index == 0 {
                    patrol.forward = true;
                } else if patrol.index == patrol.points.len() - 1 {
                    patrol.forward = false;
                }

                transform.translation.x = patrol.points[patrol.index].x;
                transform.translation.y = patrol.points[patrol.index].y;

                if patrol.forward {
                    patrol.index += 1;
                } else {
                    patrol.index -= 1;
                }

                new_velocity = (patrol.points[patrol.index] - transform.translation.truncate())
                    .normalize()
                    * 75.0;
            }

            velocity.linvel = new_velocity;
        }
    }
}

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(Enemy::spawn_npc)
                .with_system(Enemy::despawn)
                .with_system(Patrol::patrol)
                .into(),
        );
    }
}
