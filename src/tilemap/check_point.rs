use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::{
    physics::*,
    player::Player,
    save::SaveEvent,
    state::{AppState, ConditionSet},
};

use super::{
    merge_grids, GridCoords, LayerInstance, LdtkIntCell, LdtkLevel, LevelSelection,
    RegisterLdtkObjects,
};

#[derive(Component, Debug, Default, Resource)]
pub(crate) struct LastCheckPoint {
    pub(crate) coordinate: Vec3,
    pub(crate) level: LevelSelection,
}

#[derive(Component, Debug, Default)]
pub(crate) struct CheckPoint;

#[derive(Bundle, Default, LdtkIntCell)]
pub(crate) struct CheckPointBundle {
    pub(crate) check_point: CheckPoint,
}

pub(crate) struct CheckPointPlugin;

impl Plugin for CheckPointPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastCheckPoint>()
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(AppState::InGame)
                    .with_system(save_last_check_point)
                    .with_system(spawn_check_points)
                    .with_system(save_initial_check_point)
                    .into(),
            )
            .register_ldtk_int_cell_for_layer::<CheckPointBundle>("Check_Points", 1);
    }
}

fn upsert_check_point(
    cmd: &mut Commands,
    player_entity: Entity,
    last_check_point: Option<&mut LastCheckPoint>,
    translation: Vec3,
    level_selection: LevelSelection,
) {
    let check_point = LastCheckPoint {
        coordinate: translation,
        level: level_selection,
    };

    if let Some(last_check_point) = last_check_point {
        // If player touched a check point B, then go back to previous check point A,
        // it should not save check point A as it is an old one.
        if translation.x > last_check_point.coordinate.x {
            *last_check_point = check_point;
        }
    } else {
        cmd.entity(player_entity).insert(check_point);
    }
}

/// Mainly for when a game started and the player died before reaching any checkpoints.
/// Act as the default spawn point.
fn save_initial_check_point(
    mut cmd: Commands,
    mut players: Query<(Entity, &Transform, Option<&mut LastCheckPoint>), Added<Player>>,
    level_selection: Res<LevelSelection>,
) {
    for (player_entity, transform, mut last_check_point) in players.iter_mut() {
        upsert_check_point(
            &mut cmd,
            player_entity,
            last_check_point.as_deref_mut(),
            transform.translation,
            level_selection.clone(),
        );
    }
}

/// Save the touched checkpoint
fn save_last_check_point(
    mut cmd: Commands,
    check_points: Query<
        (&GlobalTransform, &CollidingEntities, &CheckPoint),
        Changed<CollidingEntities>,
    >,
    mut players: Query<(Entity, Option<&mut LastCheckPoint>), With<Player>>,
    level_selection: Res<LevelSelection>,
    mut save_event: EventWriter<SaveEvent>,
) {
    for (transform, colliding_entities, _) in check_points.iter() {
        for (player_entity, mut last_check_point) in players.iter_mut() {
            if colliding_entities.contains(player_entity) {
                upsert_check_point(
                    &mut cmd,
                    player_entity,
                    last_check_point.as_deref_mut(),
                    transform.translation(),
                    level_selection.clone(),
                );
                save_event.send(SaveEvent);
            }
        }
    }
}

pub(crate) fn spawn_check_points(
    mut commands: Commands,
    check_points: Query<(&GridCoords, &Parent), Added<CheckPoint>>,
    parent_query: Query<&Parent, Without<CheckPoint>>,
    level_query: Query<(Entity, &Handle<LdtkLevel>)>,
    levels: Res<Assets<LdtkLevel>>,
) {
    if !check_points.is_empty() {
        let mut check_point_grid_coords: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

        check_points.for_each(|(&grid_coords, parent)| {
            if let Ok(level_parent) = parent_query.get(parent.get()) {
                check_point_grid_coords
                    .entry(level_parent.get())
                    .or_insert_with(HashSet::new)
                    .insert(grid_coords);
            }
        });

        level_query.for_each(|(level_entity, level_handle)| {
            if let Some(check_point_grid_coords) = check_point_grid_coords.get(&level_entity) {
                let level = levels
                    .get(level_handle)
                    .expect("Level should be loaded by this point");

                let layer = level
                    .level
                    .layer_instances
                    .iter()
                    .flatten()
                    .find(|layer| layer.identifier == "Check_Points")
                    .expect("Check_Points layer is not found in Ldtk file")
                    .clone();

                let LayerInstance { grid_size, .. } = layer;

                // spawn colliders for every rectangle
                for merged_grid in merge_grids(&layer, check_point_grid_coords) {
                    let child_entity = commands
                        .spawn((
                            Name::new("CheckPoint"),
                            CheckPoint,
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::DYNAMIC_STATIC,
                            CollidingEntities::default(),
                            Sensor,
                            Collider::cuboid(
                                (merged_grid.right - merged_grid.left + 1) as f32
                                    * grid_size as f32
                                    / 2.,
                                (merged_grid.top - merged_grid.bottom + 1) as f32
                                    * grid_size as f32
                                    / 2.,
                            ),
                            GravityScale(0.0),
                            RigidBody::Fixed,
                            TransformBundle::from(Transform::from_xyz(
                                (merged_grid.left + merged_grid.right + 1) as f32
                                    * grid_size as f32
                                    / 2.,
                                (merged_grid.bottom + merged_grid.top + 1) as f32
                                    * grid_size as f32
                                    / 2.,
                                0.,
                            )),
                        ))
                        .id();
                    commands.entity(level_entity).add_child(child_entity);
                }
            }
        });
    }
}
