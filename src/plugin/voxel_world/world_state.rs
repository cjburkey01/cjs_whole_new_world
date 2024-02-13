use crate::{
    plugin::{
        control::controller_2::CharControl2,
        game_gui::MenuState,
        game_settings::GameSettings,
        voxel_world::{
            beef::{ChunkEntity, ChunkState, FixedChunkWorld, LoadedChunk},
            chunk_loader::ChunkLoader,
            region_saver::{force_sync_regions_save, RegionHandlerRes},
            world_info::WorldInfo,
        },
    },
    voxel::{world_noise::WorldNoiseSettings, BiomeTable, ChunkPos, CHUNK_SQUARE, CHUNK_WIDTH},
};
use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_rapier3d::geometry::Collider;
use std::time::Duration;

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
            .add_systems(
                OnExit(WorldState::LoadingStartArea),
                exit_world_loading_state_system,
            )
            .add_systems(OnEnter(WorldState::WorldLoaded), enter_world_loaded_system)
            .add_systems(OnExit(WorldState::WorldLoaded), exit_world_loaded_system)
            // Update systems
            .add_systems(
                Update,
                check_for_world_finish_load_system
                    .run_if(on_timer(Duration::from_millis(500)))
                    .run_if(in_state(WorldState::LoadingStartArea)),
            );
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
    game_settings: Res<GameSettings>,
    ply_entity: Query<Entity, With<CharControl2>>,
) {
    let name = world_info.name().to_string();
    let seed = world_info.seed();
    info!("Creating world \"{name}\" with seed {seed}");

    commands.insert_resource(RegionHandlerRes::default());
    commands.insert_resource(WorldNoiseSettings::new(seed, BiomeTable::new()));
    commands.insert_resource(FixedChunkWorld::default());
    if let Ok(entity) = ply_entity.get_single() {
        commands.entity(entity).insert((
            Transform::from_xyz(15.5, 10.0, 15.5),
            ChunkLoader::new(game_settings.load_radius),
            ChunkPos::default(),
        ));
    }
}

// Probably do stuff like save before we show the world to the player
fn exit_world_loading_state_system(
    world_info: Option<Res<WorldInfo>>,
    region_handler: Option<Res<RegionHandlerRes>>,
    chunk_world: Option<Res<FixedChunkWorld>>,
) {
    if let (Some(world_info), Some(region_handler), Some(chunk_world)) =
        (world_info, region_handler, chunk_world)
    {
        force_sync_regions_save(&world_info, &region_handler, &chunk_world);
    }
}

// Set up the player entity, hide the loading screen
fn enter_world_loaded_system(mut next_menu_state: ResMut<NextState<MenuState>>) {
    next_menu_state.set(MenuState::None);
}

// Clean up the whole world! It should be as if NO WORLD HAS BEEN LOADED
fn exit_world_loaded_system(
    mut commands: Commands,
    world_info: Option<Res<WorldInfo>>,
    region_handler: Option<Res<RegionHandlerRes>>,
    chunk_world: Option<Res<FixedChunkWorld>>,
    chunk_query: Query<Entity, With<ChunkEntity>>,
    loaders_query: Query<Entity, With<ChunkLoader>>,
) {
    if let (Some(world_info), Some(region_handler), Some(chunk_world)) =
        (world_info, region_handler, chunk_world)
    {
        force_sync_regions_save(&world_info, &region_handler, &chunk_world);
    }

    for chunk in chunk_query.iter() {
        commands.entity(chunk).despawn();
    }
    for loader in loaders_query.iter() {
        commands.entity(loader).remove::<ChunkLoader>();
    }
    commands.remove_resource::<WorldNoiseSettings>();
    commands.remove_resource::<FixedChunkWorld>();
    commands.remove_resource::<RegionHandlerRes>();
    commands.remove_resource::<WorldInfo>();
}

// This is awful.
// I'm leaving it :D
fn check_for_world_finish_load_system(
    mut next_world_state: ResMut<NextState<WorldState>>,
    chunk_world: Res<FixedChunkWorld>,
    mut ply: Query<&mut Transform, With<CharControl2>>,
    chunks: Query<(), (With<Collider>, With<ChunkEntity>)>,
) {
    if let Some(heightmap) = chunk_world.heightmaps.get(&IVec2::ZERO) {
        let height = heightmap.heightmap[(CHUNK_SQUARE / 2) as usize] as i32 + 5;
        if let Ok(mut transform) = ply.get_single_mut() {
            // Middle of the chunk
            transform.translation = UVec3::new(CHUNK_WIDTH, 0, CHUNK_WIDTH).as_vec3() / 2.0;
            transform.translation.y = height as f32;
            if let Some(LoadedChunk {
                state: ChunkState::Rendered,
                entity,
                ..
            }) = chunk_world.chunks.get(&ChunkPos(IVec3::new(
                0,
                height.div_euclid(CHUNK_WIDTH as i32),
                0,
            ))) {
                // Make sure the chunk entity exists.
                // Shouldn't be possible for it not to, but
                // I need to make my `.entity()` calls safer in the future
                if chunks.get(*entity).is_ok() {
                    next_world_state.set(WorldState::WorldLoaded);
                }
            }
        }
    }
}
