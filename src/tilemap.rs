use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

pub struct TileMapPlugin;

impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(LdtkPlugin)
            .insert_resource(LevelSelection::Uid(0))
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            set_clear_color: SetClearColor::FromLevelBackground,
            ..Default::default()
        })
        // .register_ldtk_int_cell::<components::WallBundle>(1)
        // .register_ldtk_int_cell::<components::LadderBundle>(2)
        // .register_ldtk_int_cell::<components::WallBundle>(3)
        // .register_ldtk_entity::<components::PlayerBundle>("Player")
        // .register_ldtk_entity::<components::MobBundle>("Mob")
        // .register_ldtk_entity::<components::ChestBundle>("Chest")
        ;
    }
}
