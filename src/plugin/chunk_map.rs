use crate::{
    plugin::{
        better_chunk_map::BetterChunkLoaderManager3000,
        loading::{ChunkLoader, ChunkPos},
        voxel_material::ChunkMaterialRes,
    },
    voxel::{
        world_noise::{Chunk2dNoiseValues, WorldNoiseSettings},
        Chunk, NeighborChunkSlices, SLICE_DIRECTIONS,
    },
};
use bevy::{
    prelude::*,
    tasks::AsyncComputeTaskPool,
    time::common_conditions::on_timer,
    utils::{hashbrown::hash_map::Entry, HashMap},
};
use std::{
    sync::mpsc::{channel, Receiver, Sender},
    time::Duration,
};

pub struct ChunkMapPlugin;

impl Plugin for ChunkMapPlugin {
    fn build(&self, app: &mut App) {
        let (generated_sender, generated_receiver) = channel();
        let (rendered_sender, rendered_receiver) = channel();

        app.insert_resource(ChunkGeneratedSender(generated_sender))
            .insert_non_send_resource(ChunkGeneratedReceiver(generated_receiver))
            .insert_resource(ChunkRenderedSender(rendered_sender))
            .insert_non_send_resource(ChunkRenderedReceiver(rendered_receiver))
            .init_resource::<Chunks>()
            .add_systems(
                Update,
                (
                    (
                        query_deleting_chunks,
                        query_changed_chunk_states_system
                            .run_if(on_timer(Duration::from_millis(250))),
                    )
                        .chain(),
                    (query_generated_chunk_system, query_rendered_chunk_system)
                        .run_if(on_timer(Duration::from_millis(250))),
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

#[derive(Resource)]
pub struct ChunkGeneratedSender(pub Sender<(IVec3, Option<Chunk2dNoiseValues>, Chunk)>);
#[derive(Resource)]
pub struct ChunkRenderedSender(pub Sender<(IVec3, Mesh)>);

pub struct ChunkGeneratedReceiver(pub Receiver<(IVec3, Option<Chunk2dNoiseValues>, Chunk)>);
pub struct ChunkRenderedReceiver(pub Receiver<(IVec3, Mesh)>);

/// Keeps track of all chunk states in the world.
#[derive(Default, Resource)]
pub struct Chunks {
    entities: HashMap<IVec3, Entity>,
    chunks: HashMap<IVec3, Chunk>,

    // Indexed by chunk X,Z coordinates since these values won't change with Y.
    heightmap_map: HashMap<IVec2, Chunk2dNoiseValues>,
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
pub struct GenTask;

#[derive(Component)]
pub struct RenderTask;

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
    generated_sender: Res<ChunkGeneratedSender>,
    rendered_sender: Res<ChunkRenderedSender>,
    chunks: Query<(Entity, &ChunkPos, &ChunkState), (Without<GenTask>, Without<RenderTask>)>,
) {
    let pool = AsyncComputeTaskPool::get();

    for (entity, ChunkPos { pos }, state) in chunks.iter() {
        let pos = *pos;
        match state {
            ChunkState::Generating => {
                let noise = noise.clone();
                if chunk_map
                    .entities
                    .get(&pos)
                    .map(|e| *e == entity)
                    .unwrap_or(false)
                {
                    commands.entity(entity).insert(GenTask);

                    let c = chunk_map
                        .heightmap_map
                        .get(&IVec2::new(pos.x, pos.z))
                        .cloned();

                    let sender = generated_sender.0.clone();
                    pool.spawn(async move {
                        sender
                            .send(match c {
                                Some(existing_noise) => (
                                    pos,
                                    None,
                                    noise.generate_chunk_from_noise(pos.y, &existing_noise),
                                ),
                                None => {
                                    let new_noise =
                                        noise.generate_chunk_2d_noise(IVec2::new(pos.x, pos.z));
                                    let chunk = noise.generate_chunk_from_noise(pos.y, &new_noise);
                                    (pos, Some(new_noise), chunk)
                                }
                            })
                            .unwrap();
                    })
                    .detach()
                }
            }
            ChunkState::Rendering => {
                if chunk_map
                    .entities
                    .get(&pos)
                    .map(|e| *e == entity)
                    .unwrap_or(false)
                {
                    if let Some(chunk) = chunk_map.chunks.get(&pos) {
                        if let Some(neighbors) = chunk_map.neighbors(pos) {
                            commands.entity(entity).insert(RenderTask);

                            let cloned_chunk = chunk.clone();
                            let sender = rendered_sender.0.clone();
                            AsyncComputeTaskPool::get()
                                .spawn(async move {
                                    sender
                                        .send((
                                            pos,
                                            crate::voxel::generate_mesh(&cloned_chunk, neighbors),
                                        ))
                                        .unwrap();
                                })
                                .detach();
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
    generated_receiver: NonSend<ChunkGeneratedReceiver>,
    mut chunk_map: ResMut<Chunks>,
    states: Query<&ChunkState>,
) {
    for (pos, new_noise, chunk) in generated_receiver.0.try_iter() {
        if let Some(new_noise) = new_noise {
            let _ = chunk_map
                .heightmap_map
                .entry(IVec2::new(pos.x, pos.z))
                .or_insert(new_noise);
        }
        if let Some(entity) = chunk_map.entities.get(&pos).copied() {
            if let Ok(state) = states.get(entity) {
                if *state == ChunkState::Generating {
                    chunk_map.chunks.insert(pos, chunk);
                    commands
                        .entity(entity)
                        .remove::<GenTask>()
                        .insert(ChunkState::Rendering);
                }
            }
        }
    }
}

/// Loop through all chunks with a render task,
fn query_rendered_chunk_system(
    mut commands: Commands,
    material: Res<ChunkMaterialRes>,
    chunk_map: Res<Chunks>,
    rendered_receiver: NonSend<ChunkRenderedReceiver>,
    mut meshes: ResMut<Assets<Mesh>>,
    states: Query<&ChunkState>,
) {
    for (pos, mesh) in rendered_receiver.0.try_iter() {
        if let Some(entity) = chunk_map.entities.get(&pos).copied() {
            if let Ok(state) = states.get(entity) {
                if *state == ChunkState::Rendering {
                    {
                        commands
                            .entity(entity)
                            .remove::<RenderTask>()
                            .insert(ChunkState::Visible)
                            .insert(MaterialMeshBundle {
                                mesh: meshes.add(mesh),
                                material: Handle::clone(&material.0),
                                transform: ChunkPos { pos }.transform(),
                                ..default()
                            });
                    }
                }
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
