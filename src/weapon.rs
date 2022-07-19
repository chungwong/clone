use bevy::{prelude::*, utils::Duration};

use crate::{
    camera::Offscreen,
    physics::*,
    player::{Health, Player},
    state::{ConditionSet, GameState},
};

#[derive(Clone, Component, Debug, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub(crate) struct WeaponCooldown(pub(crate) Timer);

impl WeaponCooldown {
    fn new() -> Self {
        let duration = Duration::from_secs_f32(0.2);

        let mut timer = Timer::new(duration, false);
        timer.tick(duration);

        Self(timer)
    }
}

impl Default for WeaponCooldown {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Component, Default)]
pub(crate) struct Projectile {
    damage: u32,
    started_at: Vec2,
    max_travel_distance: Option<f32>,
}

impl Projectile {
    pub(crate) fn new(damage: u32, max_travel_distance: Option<f32>, started_at: Vec2) -> Self {
        Self {
            damage,
            max_travel_distance,
            started_at,
        }
    }
}

pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(despawn_projectiles)
                .with_system(add_weapon)
                .with_system(weapon_cooldown)
                .into(),
        );
    }
}

fn add_weapon(mut cmd: Commands, players: Query<Entity, (Added<Player>, Without<WeaponCooldown>)>) {
    for entity in players.iter() {
        cmd.entity(entity).insert(WeaponCooldown::default());
    }
}

fn weapon_cooldown(time: Res<Time>, mut timers: Query<&mut WeaponCooldown>) {
    for mut timer in timers.iter_mut() {
        if !timer.finished() {
            timer.tick(time.delta());
        }
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
                health.current = health.current.saturating_sub(projectile.damage);
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
    .insert(Projectile::new(1, None, translation.truncate()));
}
