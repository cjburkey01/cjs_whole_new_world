use crate::{
    plugin::{loading::ChunkPos, voxel_material::ChunkMaterialRes},
    voxel::{world_noise::WorldNoiseSettings, Chunk},
};
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
    utils::{hashbrown::hash_map::Entry, HashMap},
};
use futures_lite::future::{block_on, poll_once};

pub struct ChunkMapPlugin;

impl Plugin for ChunkMapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Chunks>().add_systems(
            Update,
            (
                query_changed_chunk_states_system,
                query_generated_chunk_system,
                query_rendered_chunk_system,
            ),
        );
    }
}

#[derive(Component, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ChunkState {
    Generating,
    Rendering,
    Visible,
}

/// Keeps track of all chunk states in the world.
#[derive(Default, Resource)]
pub struct Chunks {
    entities: HashMap<IVec3, Entity>,
    chunks: HashMap<IVec3, Chunk>,
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

    pub fn chunk_at(&self, chunk_pos: IVec3) -> Option<&Chunk> {
        self.chunks.get(&chunk_pos)
    }
}

#[derive(Component)]
pub struct GenTask(pub Task<Chunk>);

#[derive(Component)]
pub struct RenderTask(pub Task<Mesh>);

/// Loop through all chunks that have [ChunkState::Generating] or
/// [ChunkState::Rendering] but don't have an associated generating task,
/// spawning a generation/render task for them.
#[allow(clippy::type_complexity)]
fn query_changed_chunk_states_system(
    mut commands: Commands,
    chunk_map: Res<Chunks>,
    noise: Res<WorldNoiseSettings>,
    chunks: Query<
        (Entity, &ChunkPos, &ChunkState),
        (
            Without<GenTask>,
            Without<RenderTask>,
            Or<(Added<ChunkState>, Changed<ChunkState>)>,
        ),
    >,
) {
    let pool = AsyncComputeTaskPool::get();

    for (entity, chunk_pos, state) in chunks.iter() {
        match state {
            ChunkState::Generating => {
                let noise = noise.clone();
                let pos = chunk_pos.pos;
                commands.entity(entity).insert(GenTask(
                    pool.spawn(async move { noise.build_heightmap_chunk(pos) }),
                ));
            }
            ChunkState::Rendering => {
                if let Some(chunk) = chunk_map.chunk_at(chunk_pos.pos) {
                    let cloned_chunk = chunk.clone();
                    commands.entity(entity).insert(RenderTask(
                        AsyncComputeTaskPool::get()
                            .spawn(async move { cloned_chunk.generate_mesh() }),
                    ));
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
    for (entity, chunk_pos, _state, mut task) in chunks.iter_mut() {
        if let Some(chunk) = block_on(poll_once(&mut task.0)) {
            chunk_map.chunks.insert(chunk_pos.pos, chunk);
            commands
                .entity(entity)
                .remove::<GenTask>()
                .insert(ChunkState::Rendering);
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
    for (entity, chunk_pos, _state, mut task) in chunks.iter_mut() {
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
