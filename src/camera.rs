use bevy::prelude::*;

use global_state::Persistent;

use crate::{
    input::MenuInputManagerBundle,
    player::Player,
    state::{ConditionSet, GameState},
    tilemap::{LdtkLevel, LevelSelection},
};

const ASPECT_RATIO: f32 = 16. / 9.;

#[derive(Component, Debug)]
pub(crate) struct Offscreen {
    offset: f32,
}

impl Offscreen {
    fn new(offset: f32) -> Self {
        Self { offset }
    }
}

impl Default for Offscreen {
    fn default() -> Self {
        Self::new(10.0)
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup).add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(fit_camera_to_level)
                .with_system(despawn_offscreens)
                .into(),
        );
    }
}

fn setup(mut cmd: Commands) {
    cmd.spawn(MenuInputManagerBundle::default())
        .insert(Persistent);
}

fn fit_camera_to_level(
    mut camera_query: Query<
        (
            &mut bevy::render::camera::OrthographicProjection,
            &mut Transform,
        ),
        (Without<Player>, With<Camera2d>),
    >,
    player_query: Query<&Transform, With<Player>>,
    level_query: Query<
        (&Transform, &Handle<LdtkLevel>),
        (Without<OrthographicProjection>, Without<Player>),
    >,
    level_selection: Res<LevelSelection>,
    ldtk_levels: Res<Assets<LdtkLevel>>,
) {
    if let Ok(Transform {
        translation: player_translation,
        ..
    }) = player_query.get_single()
    {
        let player_translation = *player_translation;

        let (mut orthographic_projection, mut camera_transform) = camera_query.single_mut();

        for (level_transform, level_handle) in level_query.iter() {
            if let Some(ldtk_level) = ldtk_levels.get(level_handle) {
                let level = &ldtk_level.level;
                if level_selection.is_match(&0, level) {
                    let level_ratio = level.px_wid as f32 / ldtk_level.level.px_hei as f32;

                    orthographic_projection.scaling_mode = bevy::render::camera::ScalingMode::None;
                    orthographic_projection.bottom = 0.;
                    orthographic_projection.left = 0.;
                    if level_ratio > ASPECT_RATIO {
                        // level is wider than the screen
                        orthographic_projection.top = (level.px_hei as f32 / 9.).round() * 9.;
                        orthographic_projection.right = orthographic_projection.top * ASPECT_RATIO;
                        camera_transform.translation.x = (player_translation.x
                            - level_transform.translation.x
                            - orthographic_projection.right / 2.)
                            .clamp(0., level.px_wid as f32 - orthographic_projection.right);
                        camera_transform.translation.y = 0.;
                    } else {
                        // level is taller than the screen
                        orthographic_projection.right = (level.px_wid as f32 / 16.).round() * 16.;
                        orthographic_projection.top = orthographic_projection.right / ASPECT_RATIO;
                        camera_transform.translation.y = (player_translation.y
                            - level_transform.translation.y
                            - orthographic_projection.top / 2.)
                            .clamp(0., level.px_hei as f32 - orthographic_projection.top);
                        camera_transform.translation.x = 0.;
                    }

                    camera_transform.translation.x += level_transform.translation.x;
                    camera_transform.translation.y += level_transform.translation.y;
                }
            }
        }
    }
}

pub(crate) fn despawn_offscreens(
    mut cmd: Commands,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    offscreens: Query<(Entity, &Offscreen, &Transform)>,
    windows: Res<Windows>,
) {
    for (entity, offscreen, transform) in offscreens.iter() {
        for (camera, global_transform) in cameras.iter() {
            if let Some(pos) = camera.world_to_viewport(global_transform, transform.translation) {
                if let Some(window) = windows.get_primary() {
                    if pos.x >= window.physical_width() as f32 + offscreen.offset {
                        cmd.entity(entity).despawn_recursive();
                    }
                }
            }
        }
    }
}
