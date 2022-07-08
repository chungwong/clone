use bevy::prelude::*;

use crate::camera::Offscreen;
use crate::physics::*;
use crate::player::{Health, Player};

#[derive(Component, Debug, Default)]
pub(crate) struct Projectile {
    started_at: Vec2,
    max_travel_distance: Option<f32>,
}

impl Projectile {
    pub(crate) fn new(distance: Option<f32>, started_at: Vec2) -> Self {
        Self {
            max_travel_distance: distance,
            started_at,
        }
    }
}

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(despawn_projectiles);
    }
}

fn despawn_projectiles(
    mut cmd: Commands,
    projectiles: Query<(Entity, &Projectile, &CollidingEntities, &Transform)>,
    sensors: Query<Entity, With<Sensor>>,
    mut healths: Query<&mut Health, Without<Player>>,
) {
    for (entity, projectile, colliding_entities, transform) in projectiles.iter() {
        for colliding_entity in colliding_entities.iter() {
            // if projectile collides with a sensor, do nothing
            if sensors.get(colliding_entity).is_err() {
                cmd.entity(entity).despawn_recursive();
            }

            if let Ok(mut health) = healths.get_mut(colliding_entity) {
                health.0 = 0;
            }
        }

        // make sure projectile only travel for the desired distance
        if let Some(max_travel_distance) = projectile.max_travel_distance {
            if (transform.translation.x - projectile.started_at.x) >= max_travel_distance {
                cmd.entity(entity).despawn_recursive();
            }
        }
    }
}

pub(crate) fn spawn_projectile(cmd: &mut Commands, translation: &Vec3, player: &Player) {
    let dir = player.facing_direction.to_f32();
    let offset = Vec3::new(10.0 * dir, 0.0, 0.0);

    cmd.spawn_bundle(SpriteBundle {
        transform: Transform {
            translation: *translation + offset,
            scale: Vec3::new(2.0, 2.0, 0.0),
            ..default()
        },
        ..default()
    })
    .insert(RigidBody::KinematicVelocityBased)
    .insert(Sensor)
    .insert(Collider::ball(1.0))
    .insert(Velocity {
        linvel: Vec2::new(300.0 * dir, 0.0),
        ..default()
    })
    .insert(GravityScale(0.0))
    .insert(Ccd::enabled())
    .insert(ActiveEvents::COLLISION_EVENTS)
    .insert(
        ActiveCollisionTypes::DYNAMIC_KINEMATIC
            | ActiveCollisionTypes::KINEMATIC_KINEMATIC
            | ActiveCollisionTypes::KINEMATIC_STATIC,
    )
    .insert(CollidingEntities::default())
    .insert(Offscreen::default())
    .insert(Projectile::new(None, translation.truncate()));
}
