#![allow(clippy::forget_non_drop)]

use bevy::prelude::*;

use crate::{
    physics::*,
    player::Health,
    state::{ConditionSet, GameState},
    tilemap::{
        ldtk_pixel_coords_to_translation_pivoted, EntityInstance, FieldValue, LayerInstance,
        LdtkEntity, RegisterLdtkObjects, TilesetDefinition,
    },
};

#[derive(Clone, Component, Default)]
pub struct Enemy;

#[derive(Bundle, Clone, Default, LdtkEntity)]
pub struct MobPhysicsBundle {
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

impl From<EntityInstance> for MobPhysicsBundle {
    fn from(entity_instance: EntityInstance) -> Self {
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

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub(crate) struct MobBundle {
    #[sprite_bundle("player.png")]
    #[bundle]
    pub(crate) sprite_bundle: SpriteBundle,

    #[from_entity_instance]
    #[bundle]
    pub(crate) collider_bundle: MobPhysicsBundle,

    pub(crate) enemy: Enemy,

    #[from_entity_instance]
    pub(crate) hp: Health,

    #[ldtk_entity]
    pub patrol: Patrol,
}

#[derive(Clone, PartialEq, Debug, Default, Component)]
pub struct Patrol {
    pub points: Vec<Vec2>,
    pub index: usize,
    pub forward: bool,
}

impl LdtkEntity for Patrol {
    fn bundle_entity(
        entity_instance: &EntityInstance,
        layer_instance: &LayerInstance,
        _: Option<&Handle<Image>>,
        _: Option<&TilesetDefinition>,
        _: &AssetServer,
        _: &mut Assets<TextureAtlas>,
    ) -> Self {
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

        Patrol {
            points,
            index: 1,
            forward: true,
        }
    }
}

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_entity::<MobBundle>("Mob").add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(setup)
                .with_system(despawn)
                .with_system(patrol)
                .into(),
        );
    }
}

fn setup(mut enemies: Query<&mut Sprite, Added<Enemy>>) {
    for mut sprite in enemies.iter_mut() {
        sprite.color = Color::RED;
    }
}

fn despawn(mut cmd: Commands, enemies: Query<(Entity, &Health), (With<Enemy>, Changed<Health>)>) {
    for (entity, health) in enemies.iter() {
        if health.current == 0 {
            cmd.entity(entity).despawn_recursive();
        }
    }
}

pub fn patrol(mut query: Query<(&mut Transform, &mut Velocity, &mut Patrol)>) {
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

            new_velocity =
                (patrol.points[patrol.index] - transform.translation.truncate()).normalize() * 75.0;
        }

        velocity.linvel = new_velocity;
    }
}
