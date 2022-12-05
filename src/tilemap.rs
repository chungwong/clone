#![allow(clippy::forget_non_drop)]

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
pub use bevy_ecs_ldtk::{prelude::*, utils::ldtk_pixel_coords_to_translation_pivoted};

use crate::{
    physics::{Collider, RigidBody},
    state::{AppLooplessStateExt, ConditionSet, GameState},
};

pub(crate) mod check_point;
use check_point::CheckPointPlugin;

#[derive(Debug, Default, Resource)]
pub(crate) struct LevelSize(pub Option<Vec2>);

pub struct TilemapPlugin;

impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(LdtkPlugin)
            .init_resource::<LevelSize>()
            .insert_resource(LevelSelection::Uid(0))
            .insert_resource(LdtkSettings {
                level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                    load_level_neighbors: true,
                },
                set_clear_color: SetClearColor::FromLevelBackground,
                ..default()
            })
            .add_enter_system(GameState::InGame, setup)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(spawn_wall_collision)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGameAssetLoading)
                    .with_system(set_boundary)
                    .into(),
            )
            .add_plugin(CheckPointPlugin)
            .register_ldtk_int_cell_for_layer::<WallBundle>("Collisions", 1);
    }
}

fn setup(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn(Camera2dBundle::default());

    cmd.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("levels/reckoning.ldtk"),
        ..default()
    });
}

#[allow(clippy::only_used_in_recursion)]
fn set_boundary(
    mut level_events: EventReader<LevelEvent>,
    mut level_size: ResMut<LevelSize>,
    level_query: Query<&Handle<LdtkLevel>>,
    levels: Res<Assets<LdtkLevel>>,
) {
    for event in level_events.iter() {
        if let LevelEvent::Transformed(_) = event {
            level_query.for_each(|level_handle| {
                if let Some(level) = levels.get(level_handle) {
                    *level_size = LevelSize(Some(Vec2::new(
                        level.level.px_wid as f32,
                        level.level.px_hei as f32,
                    )));
                }
            })
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub(crate) struct Wall;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub(crate) struct WallBundle {
    wall: Wall,
}

/// Represents a wide collider that is 1 tile tall
/// Used to spawn collisions
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
struct Plate {
    left: i32,
    right: i32,
}

/// A simple rectangle type representing a wall of any size
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
pub(crate) struct Rect {
    left: i32,
    right: i32,
    top: i32,
    bottom: i32,
}

pub(crate) fn merge_grids(layer: &LayerInstance, grid_coords: &HashSet<GridCoords>) -> Vec<Rect> {
    let LayerInstance {
        c_wid: width,
        c_hei: height,
        ..
    } = layer;

    // combine grids into flat "plates" in each individual row
    let mut plate_stack: Vec<Vec<Plate>> = Vec::new();

    for y in 0..*height {
        let mut row_plates: Vec<Plate> = Vec::new();
        let mut plate_start = None;

        // + 1 to the width so the algorithm "terminates" plates that touch the right
        // edge
        for x in 0..width + 1 {
            match (plate_start, grid_coords.contains(&GridCoords { x, y })) {
                (Some(s), false) => {
                    row_plates.push(Plate {
                        left: s,
                        right: x - 1,
                    });
                    plate_start = None;
                }
                (None, true) => plate_start = Some(x),
                _ => (),
            }
        }

        plate_stack.push(row_plates);
    }

    // combine "plates" into rectangles across multiple rows
    let mut rects: Vec<Rect> = Vec::new();
    let mut previous_rects: HashMap<Plate, Rect> = HashMap::new();

    // an extra empty row so the algorithm "terminates" the rects that touch the top
    // edge
    plate_stack.push(Vec::new());

    for (y, row) in plate_stack.iter().enumerate() {
        let mut current_rects: HashMap<Plate, Rect> = HashMap::new();
        for plate in row {
            if let Some(previous_rect) = previous_rects.remove(plate) {
                current_rects.insert(
                    *plate,
                    Rect {
                        top: previous_rect.top + 1,
                        ..previous_rect
                    },
                );
            } else {
                current_rects.insert(
                    *plate,
                    Rect {
                        bottom: y as i32,
                        top: y as i32,
                        left: plate.left,
                        right: plate.right,
                    },
                );
            }
        }

        // Any plates that weren't removed above have terminated
        rects.append(&mut previous_rects.values().copied().collect());
        previous_rects = current_rects;
    }

    rects
}

/// Spawns heron collisions for the walls of a level
///
/// You could just insert a ColliderBundle in to the WallBundle,
/// but this spawns a different collider for EVERY wall tile.
/// This approach leads to bad performance.
///
/// Instead, by flagging the wall tiles and spawning the collisions later,
/// we can minimize the amount of colliding entities.
///
/// The algorithm used here is a nice compromise between simplicity, speed,
/// and a small number of rectangle colliders.
/// In basic terms, it will:
/// 1. consider where the walls are
/// 2. combine wall tiles into flat "plates" in each individual row
/// 3. combine the plates into rectangles across multiple rows wherever possible
/// 4. spawn colliders for each rectangle
pub(crate) fn spawn_wall_collision(
    mut commands: Commands,
    walls: Query<(&GridCoords, &Parent), Added<Wall>>,
    parent_query: Query<&Parent, Without<Wall>>,
    level_query: Query<(Entity, &Handle<LdtkLevel>)>,
    levels: Res<Assets<LdtkLevel>>,
) {
    if !walls.is_empty() {
        // consider where the walls are
        // storing them as GridCoords in a HashSet for quick, easy lookup
        let mut wall_grid_coords: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

        walls.for_each(|(&grid_coords, parent)| {
            // the intgrid tiles' direct parents will be bevy_ecs_tilemap chunks, not the level
            // To get the level, you need their grandparents, which is where parent_query comes in
            if let Ok(level_entity) = parent_query.get(parent.get()) {
                wall_grid_coords
                    .entry(level_entity.get())
                    .or_insert_with(HashSet::new)
                    .insert(grid_coords);
            }
        });

        level_query.for_each(|(level_entity, level_handle)| {
            if let Some(wall_grid_coords) = wall_grid_coords.get(&level_entity) {
                let level = levels
                    .get(level_handle)
                    .expect("Level should be loaded by this point");

                let layer = level
                    .level
                    .layer_instances
                    .iter()
                    .flatten()
                    .find(|layer| layer.identifier == "Collisions")
                    .expect("Collisons layer is not found in Ldtk file")
                    .clone();

                let LayerInstance { grid_size, .. } = layer;

                // spawn colliders for every rectangle
                for merged_grid in merge_grids(&layer, wall_grid_coords) {
                    let child_entity = commands
                        .spawn((
                            Name::new("Wall"),
                            Collider::cuboid(
                                (merged_grid.right - merged_grid.left + 1) as f32
                                    * grid_size as f32
                                    / 2.,
                                (merged_grid.top - merged_grid.bottom + 1) as f32
                                    * grid_size as f32
                                    / 2.,
                            ),
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

                    // Making the collider a child of the level serves two purposes:
                    // 1. Adjusts the transforms to be relative to the level for free
                    // 2. the colliders will be despawned automatically when levels unload
                    commands.entity(level_entity).add_child(child_entity);
                }
            }
        });
    }
}
