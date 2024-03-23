use crate::{
    game_settings::GameSettings,
    oct_tree::{LodChunkEntity, LodLoader, LodWorld},
    plugin::{
        control::controller_2::CharControl2,
        game_gui::MenuState,
        voxel_world::{
            beef::{ChunkEntity, FixedChunkWorld},
            region_saver::{force_sync_regions_save, RegionHandlerRes},
            world_info::WorldInfo,
        },
    },
    voxel::{world_noise::WorldNoiseSettings, BiomeTable, ChunkPos},
};
use bevy::prelude::*;

pub struct WorldStatePlugin;

impl Plugin for WorldStatePlugin {
    fn build(&self, app: &mut App) {
        app
            // State
            .add_state::<WorldState>()
            // State systems
            .add_systems(
                OnEnter(WorldState::LoadingStartArea),
                enter_world_loading_state_system,
            )
            // .add_systems(
            //     OnExit(WorldState::LoadingStartArea),
            //     exit_world_loading_state_system,
            // )
            .add_systems(OnEnter(WorldState::WorldLoaded), enter_world_loaded_system)
            .add_systems(OnExit(WorldState::WorldLoaded), exit_world_loaded_system);
    }
}

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
pub enum WorldState {
    #[default]
    NotInWorld,
    LoadingStartArea,
    WorldLoaded,
}

// Add all the resources we need to start doing stuff with the world
// Add chunk loader to camera
fn enter_world_loading_state_system(
    mut commands: Commands,
    world_info: Res<WorldInfo>,
    _game_settings: Res<GameSettings>,
    ply_entity: Query<Entity, With<CharControl2>>,
) {
    let name = world_info.name().to_string();
    let seed = world_info.seed();
    info!("Creating world \"{name}\" with seed {seed}");

    commands.insert_resource(RegionHandlerRes::default());
    commands.insert_resource(WorldNoiseSettings::new(seed, BiomeTable::new()));
    commands.insert_resource(LodWorld::default());
    commands.insert_resource(NeededLods::default());
    if let Ok(entity) = ply_entity.get_single() {
        commands.entity(entity).insert((
            TransformBundle::from_transform(Transform::from_xyz(15.5, 10.0, 15.5)),
            //ChunkLoader::new(game_settings.load_radius),
            ChunkPos::default(),
        ));
    }
    // test loader position at 0,0,0
    commands.spawn((
        TransformBundle::IDENTITY,
        ChunkPos(IVec3::ZERO),
        LodLoader::new(&[1, 1, 1]),
    ));
}

// Probably do stuff like save before we show the world to the player
// fn exit_world_loading_state_system(
//     world_info: Option<Res<WorldInfo>>,
//     region_handler: Option<Res<RegionHandlerRes>>,
//     chunk_world: Option<Res<FixedChunkWorld>>,
// ) {
//     if let (Some(world_info), Some(region_handler), Some(chunk_world)) =
//         (world_info, region_handler, chunk_world)
//     {
//         force_sync_regions_save(&world_info, &region_handler, &chunk_world);
//     }
// }

// Set up the player entity, hide the loading screen
fn enter_world_loaded_system(mut next_menu_state: ResMut<NextState<MenuState>>) {
    next_menu_state.set(MenuState::None);
}

// Clean up the whole world! It should be as if NO WORLD HAS BEEN LOADED
fn exit_world_loaded_system(
    mut commands: Commands,
    _world_info: Option<Res<WorldInfo>>,
    _region_handler: Option<Res<RegionHandlerRes>>,
    chunk_query: Query<Entity, With<LodChunkEntity>>,
    loaders_query: Query<Entity, With<LodLoader>>,
) {
    // TODO:
    // if let (Some(world_info), Some(region_handler)) = (world_info, region_handler) {
    //force_sync_regions_save(&world_info, &region_handler);
    // }

    for chunk in chunk_query.iter() {
        commands.entity(chunk).despawn();
    }
    for loader in loaders_query.iter() {
        commands.entity(loader).remove::<LodLoader>();
    }
    commands.remove_resource::<WorldNoiseSettings>();
    commands.remove_resource::<LodWorld>();
    commands.remove_resource::<NeededLods>();
    commands.remove_resource::<RegionHandlerRes>();
    commands.remove_resource::<WorldInfo>();
}
