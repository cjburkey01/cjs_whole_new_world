use crate::{
    plugin::{
        better_chunk_map::BetterChunkLoaderManager3000,
        loading::{ChunkLoader, ChunkPos},
        voxel_material::ChunkMaterialRes,
    },
    voxel::{
        world_noise::WorldNoiseSettings, Chunk, NeighborChunkSlices, CHUNK_WIDTH, SLICE_DIRECTIONS,
    },
};
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
    time::common_conditions::on_timer,
    utils::{hashbrown::hash_map::Entry, HashMap},
};
use futures_lite::future::{block_on, poll_once};
use std::time::Duration;

pub struct ChunkMapPlugin;

impl Plugin for ChunkMapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Chunks>().add_systems(
            Update,
            (
                (
                    query_deleting_chunks,
                    query_changed_chunk_states_system,
                    (query_generated_chunk_system, query_rendered_chunk_system),
                )
                    .chain(),
                query_distant_chunks_system.run_if(on_timer(Duration::from_millis(500))),
            ),
        );
    }
}

#[derive(Component, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ChunkState {
    Generating,
    Rendering,
    Visible,
    Deleting,
}

/// Keeps track of all chunk states in the world.
#[derive(Resource)]
pub struct Chunks {
    entities: HashMap<IVec3, Entity>,
    chunks: HashMap<IVec3, Chunk>,
}

impl Default for Chunks {
    fn default() -> Self {
        Self {
            entities: default(),
            chunks: default(),
        }
    }
}

impl Chunks {
    // Returns true if the chunk was *not* already initialized
    pub fn request_chunk_gen_render(&mut self, commands: &mut Commands, pos: IVec3) -> bool {
        match self.entities.entry(pos) {
            Entry::Occupied(_) => {
                // We shouldn't ever need to regenerate a chunk.
                // We might want to re-mesh it, but we don't have to regenerate.
                // commands.entity(*entry.get()).insert(ChunkState::Generating);
                false
            }
            Entry::Vacant(entry) => {
                let chunk_pos = ChunkPos { pos };
                entry.insert(
                    commands
                        .spawn((
                            TransformBundle::from_transform(chunk_pos.transform()),
                            chunk_pos,
                            ChunkState::Generating,
                        ))
                        .id(),
                );
                true
            }
        }
    }

    pub fn neighbors(&self, chunk: IVec3) -> Option<NeighborChunkSlices> {
        let slice_dirs = SLICE_DIRECTIONS.map(|direction| {
            let norm = direction.normal();
            let chunk_pos = chunk + norm.to_ivec3();
            (direction, self.chunks.get(&chunk_pos))
        });

        let mut output = NeighborChunkSlices::default();

        for (direction, chunk) in slice_dirs {
            *output.get_in_direction_mut(direction.normal()) =
                chunk?.get_solid_bits_slice(direction, 0)?;
        }

        Some(output)
    }

    #[allow(unused)]
    pub fn chunks(&self) -> &HashMap<IVec3, Chunk> {
        &self.chunks
    }

    pub fn entities(&self) -> &HashMap<IVec3, Entity> {
        &self.entities
    }
}

#[derive(Component)]
pub struct GenTask(pub Task<Chunk>);

#[derive(Component)]
pub struct RenderTask(pub Task<Mesh>);

fn query_deleting_chunks(
    mut commands: Commands,
    mut chunk_map: ResMut<Chunks>,
    mut three_k: ResMut<BetterChunkLoaderManager3000>,
    chunks: Query<(Entity, &ChunkPos, &ChunkState), Changed<ChunkState>>,
) {
    for (entity, ChunkPos { pos }, state) in chunks.iter() {
        if *state == ChunkState::Deleting {
            commands.entity(entity).despawn();
            chunk_map.chunks.remove(pos);
            chunk_map.entities.remove(pos);
            three_k.unload(*pos);
        }
    }
}

/// Loop through all chunks that have [ChunkState::Generating] or
/// [ChunkState::Rendering] but don't have an associated generating task,
/// spawning a generation/render task for them.
#[allow(clippy::type_complexity)]
fn query_changed_chunk_states_system(
    mut commands: Commands,
    chunk_map: Res<Chunks>,
    noise: Res<WorldNoiseSettings>,
    chunks: Query<(Entity, &ChunkPos, &ChunkState), (Without<GenTask>, Without<RenderTask>)>,
) {
    let pool = AsyncComputeTaskPool::get();

    for (entity, ChunkPos { pos }, state) in chunks.iter() {
        match state {
            ChunkState::Generating => {
                let noise = noise.clone();
                let pos = *pos;
                if chunk_map
                    .entities
                    .get(&pos)
                    .map(|e| *e == entity)
                    .unwrap_or(false)
                {
                    commands.entity(entity).insert(GenTask(
                        pool.spawn(async move { noise.build_heightmap_chunk(pos) }),
                    ));
                }
            }
            ChunkState::Rendering => {
                if chunk_map
                    .entities
                    .get(pos)
                    .map(|e| *e == entity)
                    .unwrap_or(false)
                {
                    if let Some(chunk) = chunk_map.chunks.get(pos) {
                        if let Some(neighbors) = chunk_map.neighbors(*pos) {
                            let cloned_chunk = chunk.clone();
                            commands.entity(entity).insert(RenderTask(
                                AsyncComputeTaskPool::get()
                                    .spawn(async move { cloned_chunk.generate_mesh(neighbors) }),
                            ));
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Loop through all chunks with a generation task,
fn query_generated_chunk_system(
    mut commands: Commands,
    mut chunk_map: ResMut<Chunks>,
    mut chunks: Query<(Entity, &ChunkPos, &ChunkState, &mut GenTask)>,
) {
    for (entity, chunk_pos, state, mut task) in chunks.iter_mut() {
        if *state == ChunkState::Generating {
            if let Some(chunk) = block_on(poll_once(&mut task.0)) {
                chunk_map.chunks.insert(chunk_pos.pos, chunk);
                commands
                    .entity(entity)
                    .remove::<GenTask>()
                    .insert(ChunkState::Rendering);
            }
        }
    }
}

/// Loop through all chunks with a render task,
fn query_rendered_chunk_system(
    mut commands: Commands,
    material: Res<ChunkMaterialRes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunks: Query<(Entity, &ChunkPos, &ChunkState, &mut RenderTask)>,
) {
    for (entity, chunk_pos, state, mut task) in chunks.iter_mut() {
        if *state == ChunkState::Rendering {
            if let Some(mesh) = block_on(poll_once(&mut task.0)) {
                commands
                    .entity(entity)
                    .remove::<RenderTask>()
                    .insert(ChunkState::Visible)
                    .insert(MaterialMeshBundle {
                        mesh: meshes.add(mesh),
                        material: Handle::clone(&material.0),
                        transform: chunk_pos.transform(),
                        ..default()
                    });
            }
        }
    }
}

fn query_distant_chunks_system(
    mut commands: Commands,
    chunks: Query<(Entity, &ChunkPos)>,
    loaders: Query<(&ChunkPos, &ChunkLoader)>,
) {
    'chunks: for (entity, ChunkPos { pos: chunk_pos }) in chunks.iter() {
        for (ChunkPos { pos: loader_pos }, ChunkLoader { radius }) in loaders.iter() {
            let diameter = *radius * 2;
            if chunk_pos.distance_squared(*loader_pos) < (diameter * diameter + 2) as i32 {
                continue 'chunks;
            }
        }

        commands.entity(entity).insert(ChunkState::Deleting);
    }
}
