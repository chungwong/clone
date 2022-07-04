use std::time::Duration;

use bevy::prelude::*;

use crate::input::{Action, ActionState};
use crate::physics::*;
use crate::player::{DashInput, Direction, JumpStatus, Player, PlayerMovementSettings};
use crate::tilemap::{EntityInstance, LevelSize};
use crate::weapon::spawn_projectile;

pub(crate) fn check_standing(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Player)>,
    rapier_context: Res<RapierContext>,
) {
    for (player_entity, mut player) in query.iter_mut() {
        if get_standing_normal(&rapier_context, &player_entity).is_some() {
            player.last_stood_time = time.last_update();
        }
    }
}

pub(crate) fn set_facing_direction(mut query: Query<(&mut Player, &ActionState)>) {
    for (mut player, action_state) in query.iter_mut() {
        if action_state.pressed(Action::Left) && !matches!(player.facing_direction, Direction::Left)
        {
            player.facing_direction = Direction::Left;
        } else if action_state.pressed(Action::Right)
            && !matches!(player.facing_direction, Direction::Right)
        {
            player.facing_direction = Direction::Right;
        }
    }
}

pub(crate) fn dash(
    time: Res<Time>,
    mut dash_input: Local<DashInput>,
    mut query: Query<(
        Entity,
        &mut Velocity,
        &mut GravityScale,
        &mut Player,
        &ActionState,
    )>,
    rapier_context: Res<RapierContext>,
    player_movement_settings: Res<PlayerMovementSettings>,
) {
    for (player_entity, mut velocity, mut gravity_scale, mut player, action_state) in
        query.iter_mut()
    {
        let dir = if action_state.just_pressed(Action::Left) {
            Direction::Left
        } else if action_state.just_pressed(Action::Right) {
            Direction::Right
        } else {
            Direction::Neutral
        };

        let standing_normal = get_standing_normal(&rapier_context, &player_entity);

        if let Some(normal) = standing_normal {
            // if it is standing on something and LEFT or Right is pressed
            if normal == Vec2::Y && matches!(&dir, Direction::Left | Direction::Right) {
                if dash_input.input_timer.finished() {
                    // store the input, to check if the consecutive inputs are in the same dir
                    dash_input.direction = dir;

                    // reset the timer
                    dash_input.input_timer.reset();

                    *gravity_scale = GravityScale(player_movement_settings.gravity_scale);
                } else if !dash_input.input_timer.finished() && dir == dash_input.direction {
                    velocity.linvel = get_run_velocity(
                        &velocity.linvel,
                        dir.to_f32() * player_movement_settings.dash_speed,
                        time.delta_seconds(),
                    );

                    player.dashing = true;

                    *gravity_scale = GravityScale(0.0);
                } else {
                    *gravity_scale = GravityScale(player_movement_settings.gravity_scale);
                }
            }
        }

        if dash_input.input_timer.finished() && player.dashing {
            player.dashing = false;

            // if dash is finished, reset gravity scale otherwise, when doing a dash jump
            // and holding space, the player will fly
            *gravity_scale = GravityScale(player_movement_settings.gravity_scale);
        }
    }
    if !dash_input.input_timer.finished() {
        dash_input.input_timer.tick(time.delta());
    }
}

pub(crate) fn get_run_velocity(velocity: &Vec2, speed: f32, time_delta: f32) -> Vec2 {
    let wall_jump_lerp = 10.;
    velocity.lerp(Vec2::new(speed, velocity.y), wall_jump_lerp * time_delta)
}

/// https://en.wikipedia.org/wiki/Normal_(geometry)
pub(crate) fn get_standing_normal(
    rapier_context: &Res<RapierContext>,
    player_entity: &Entity,
) -> Option<Vec2> {
    let mut standing_normal = rapier_context
        .contacts_with(*player_entity)
        .filter(|contact| contact.has_any_active_contacts())
        .flat_map(|contact| {
            contact
                .manifolds()
                .filter_map(|contact_manifold| {
                    if contact.collider1() == *player_entity {
                        Some(-contact_manifold.normal())
                    } else if contact.collider2() == *player_entity {
                        Some(contact_manifold.normal())
                    } else {
                        None
                    }
                })
                .max_by_key(|normal| float_ord::FloatOrd(normal.dot(Vec2::Y)))
        })
        .max_by_key(|normal| float_ord::FloatOrd(normal.dot(Vec2::Y)));

    if let Some(mut normal) = standing_normal {
        // truncate float number with a `long tail`
        if normal.y < 0.0001 {
            normal.y = 0.0;
        }

        standing_normal = Some(normal);
    }

    standing_normal
}
pub(crate) fn jump(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Velocity, &mut Player, &ActionState)>,
    player_movement_settings: Res<PlayerMovementSettings>,
    rapier_context: Res<RapierContext>,
    rapier_config: Res<RapierConfiguration>,
) {
    // let jump_impulse = 10000.0; // SCALE = 1.0
    // let jump_impulse = 100.0;  // SCALE = 10.0
    // let jump_impulse = 1.0;   // SCALE = 100.0
    // let jump_impulse = player_movement_settings.jump_impulse / SCALE.powf(2.0);

    for (player_entity, mut velocity, mut player, action_state) in query.iter_mut() {
        let pressed_jump = action_state.pressed(Action::Jump);

        // find a normal of the standing ground where the player stands on
        let mut standing_normal = get_standing_normal(&rapier_context, &player_entity);

        // coyote time if
        // 1. the Y normal is none(falling)
        // 2. player pressed_jump
        // 3. the player is not currently jumping
        if standing_normal.is_none() && pressed_jump && !player.is_jumping {
            let duration = Duration::from_millis(player_movement_settings.coyote_time_ms);
            match (player.last_stood_time, time.last_update()) {
                (Some(last_stood_time), Some(last_update))
                    if last_update - last_stood_time <= duration =>
                {
                    standing_normal = Some(player.last_stood_normal);
                }
                _ => (),
            };
        }

        player.wall_sliding = false;

        if let Some(normal) = standing_normal {
            // reset player.is_jumping when it is jump back on ground
            // 1. on a ground
            // 2. on wall grab
            if normal.x.abs() == 1.0 || normal.y == 1.0 {
                player.is_jumping = false;
            }
        }

        // determie the jump status of the player
        let jump_status = (|| {
            if let Some(normal) = standing_normal {
                player.last_stood_normal = normal;

                // // wall grab and slide
                if normal.x.abs() == 1.0
                    && normal.y == 0.0
                    && (action_state.pressed(Action::Right) || action_state.pressed(Action::Left))
                {
                    if action_state.just_pressed(Action::Jump) {
                        return JumpStatus::InitiateJump;
                    }
                    return JumpStatus::WallSliding;
                }

                if 0.0 < normal.dot(Vec2::Y) && normal.y > 0.001 {
                    if pressed_jump {
                        return JumpStatus::InitiateJump;
                    }
                    return JumpStatus::CanJump;
                }
            }

            if 0.0 <= velocity.linvel.y {
                if pressed_jump && player.rising {
                    JumpStatus::GoingUp
                } else {
                    JumpStatus::StoppingUp
                }
            } else {
                JumpStatus::Falling
            }
        })();

        match jump_status {
            JumpStatus::CanJump => {
                player.rising = false;
            }
            JumpStatus::InitiateJump => {
                velocity.linvel += Vec2::Y * player_movement_settings.jump_power_coefficient;

                player.rising = true;

                // indicate the player is jumping, it will only be reset when it touches the ground
                // or wall grab or something similar
                player.is_jumping = true;
            }
            JumpStatus::GoingUp => {
                player.rising = true;
            }
            JumpStatus::StoppingUp => {
                velocity.linvel.y += rapier_config.gravity.y
                    * player_movement_settings.jump_break_factor
                    * time.delta_seconds();
                player.rising = false;
            }
            JumpStatus::Falling => {
                velocity.linvel.y += rapier_config.gravity.y
                    * player_movement_settings.fall_factor
                    * time.delta_seconds();
                player.rising = false;
            }
            JumpStatus::WallSliding => {
                player.wall_sliding = true;
                player.rising = false;

                // wall slide
                // velocity.linvel.y += rapier_config.gravity.y * player_movement_settings.slide_factor * time.delta_seconds();

                // wall grab
                velocity.linvel.y = 0.0;
            }
        }

        player.jump_status = jump_status;
    }
}
pub(crate) fn run(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &ActionState), With<Player>>,
    player_movement_settings: Res<PlayerMovementSettings>,
) {
    for (mut velocity, action_state) in query.iter_mut() {
        let target_speed: f32 = if action_state.pressed(Action::Left) {
            -1.0
        } else if action_state.pressed(Action::Right) {
            1.0
        } else {
            0.0
        };

        velocity.linvel = get_run_velocity(
            &velocity.linvel,
            target_speed * player_movement_settings.run_speed,
            time.delta_seconds(),
        );
    }
}

pub(crate) fn boundary(
    level_size: Res<LevelSize>,
    mut players: Query<(&mut Transform, &EntityInstance), With<Player>>,
) {
    // during startup, there is a few frames that level_size is not initialised
    if let Some(level_size) = level_size.0 {
        for (mut transform, entity_instance) in players.iter_mut() {
            // half width of player is the offset
            // origin of player is the centre
            let offset = entity_instance.width as f32 / 2.0;

            if transform.translation.x > level_size.x {
                transform.translation.x = level_size.x;
            } else if transform.translation.x <= offset {
                transform.translation.x = offset;
            }
        }
    }
}

pub(crate) fn attack(mut cmd: Commands, players: Query<(&Transform, &ActionState, &Player)>) {
    for (transform, action_state, player) in players.iter() {
        if action_state.just_pressed(Action::Attack) {
            spawn_projectile(&mut cmd, &transform.translation, player);
        }
    }
}
