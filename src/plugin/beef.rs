use crate::{
    plugin::{
        loading::{ChunkLoader, ChunkPos},
        voxel_material::ChunkMaterialRes,
    },
    voxel::{world_noise::WorldNoiseSettings, Chunk, NeighborChunkSlices, SLICE_DIRECTIONS},
};
use bevy::{
    prelude::*,
    tasks::{block_on, AsyncComputeTaskPool, Task},
    utils::HashMap,
};
use bevy_rapier3d::{dynamics::RigidBody, prelude::Collider};
use futures_lite::future::poll_once;
use itertools::iproduct;

// Best (Even Ever?) Fucker: my chunk loading solution.
pub struct BeefPlugin;

impl Plugin for BeefPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FixedChunkMap>()
            .add_systems(Update, update_loader_states)
            .add_systems(Update, start_loading.after(update_loader_states))
            .add_systems(Update, check_queue.after(start_loading));
    }
}

fn update_loader_states(
    mut commands: Commands,
    mut chunks: ResMut<FixedChunkMap>,
    loaders: Query<(&ChunkPos, &ChunkLoader), Changed<ChunkPos>>,
) {
    // Only one chunk loader for now
    if let Ok((loader_pos, ChunkLoader { radius })) = loaders.get_single() {
        chunks.update_needed_chunk_states(&mut commands, loader_pos.pos, *radius as usize);
    }
}

fn start_loading(
    mut commands: Commands,
    mut chunks: ResMut<FixedChunkMap>,
    noise: Res<WorldNoiseSettings>,
    loaders: Query<(&ChunkPos, &ChunkLoader)>,
) {
    if let Ok((loader_pos, ChunkLoader { radius })) = loaders.get_single() {
        let state_changes = chunks.required_state_changes(loader_pos.pos, *radius as usize);
        chunks.execute_state_changes(&mut commands, &noise, state_changes);
    }
}

fn check_queue(
    mut commands: Commands,
    material: Res<ChunkMaterialRes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunks: ResMut<FixedChunkMap>,
    mut generate_query: Query<(Entity, &mut GenerateTask), Without<RenderTask>>,
    mut render_query: Query<(Entity, &mut RenderTask), Without<GenerateTask>>,
) {
    chunks.collect_finished_tasks(
        &mut commands,
        &material,
        &mut meshes,
        &mut generate_query,
        &mut render_query,
    );
}

#[derive(Debug, Component, Copy, Clone, Eq, PartialEq)]
pub struct ChunkEntity(pub IVec3);

#[derive(Component)]
struct GenerateTask(IVec3, Task<Chunk>);

#[derive(Component)]
struct RenderTask(IVec3, Task<Option<(Collider, Mesh)>>);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NeededStateChange {
    Generate,
    Render,
    Delete,
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ChunkState {
    #[default]
    Empty,
    Generating,
    Generated,
    Rendering,
    Rendered,
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum NeededChunkState {
    #[default]
    DontNeed,
    Generated,
    Rendered,
}

struct LoadedChunk {
    entity: Entity,
    chunk: Option<Chunk>,
    state: ChunkState,
    needed_state: NeededChunkState,
}

#[derive(Default, Resource)]
pub struct FixedChunkMap {
    chunks: HashMap<IVec3, LoadedChunk>,
}

impl FixedChunkMap {
    pub fn set_needed<'w: 'a, 's: 'a, 'a>(
        &mut self,
        commands: &mut Commands,
        chunk: IVec3,
        needed_state: NeededChunkState,
    ) {
        let _ = self
            .chunks
            .entry(chunk)
            .and_modify(|c| c.needed_state = needed_state)
            .or_insert_with(|| LoadedChunk {
                entity: commands.spawn(ChunkEntity(chunk)).id(),
                chunk: None,
                state: ChunkState::Empty,
                needed_state,
            });
    }

    pub fn update_needed_chunk_states<'w: 'a, 's: 'a, 'a>(
        &mut self,
        commands: &mut Commands,
        loader_chunk: IVec3,
        radius: usize,
    ) {
        let r = radius as i32;

        for (x, y, z) in iproduct!(-r..=r, -r..=r, -r..=r) {
            let offset = IVec3::new(x, y, z);
            // We don't render the outermost layer of chunks, they are only
            // generated to allow for chunk side culling.
            let needed_state = match x.abs() == r || y.abs() == r || z.abs() == r {
                true => NeededChunkState::Generated,
                false => NeededChunkState::Rendered,
            };
            self.set_needed(commands, loader_chunk + offset, needed_state);
        }
    }

    fn required_state_changes(
        &self,
        loader_chunk: IVec3,
        radius: usize,
    ) -> Vec<(IVec3, Entity, NeededStateChange)> {
        let mut changes = vec![];
        let radius_i = radius as i32;

        // Loop through existing chunks
        for (pos, chunk) in self.chunks.iter() {
            let offset_pos = *pos - loader_chunk;
            let entity = chunk.entity;

            // If it's within the radius of chunks that need loaded
            if offset_pos.abs().max_element() <= radius_i {
                // Check what state the chunk needs to be in
                match chunk.needed_state {
                    NeededChunkState::DontNeed => {
                        changes.push((*pos, entity, NeededStateChange::Delete))
                    }
                    NeededChunkState::Generated => match chunk.state {
                        ChunkState::Empty => {
                            changes.push((*pos, entity, NeededStateChange::Generate))
                        }
                        // Intentionally not using `_` in case I add new chunk
                        // states for whatever cursed reason.
                        ChunkState::Generating
                        | ChunkState::Generated
                        | ChunkState::Rendering
                        | ChunkState::Rendered => {}
                    },
                    NeededChunkState::Rendered => match chunk.state {
                        ChunkState::Empty => {
                            changes.push((*pos, entity, NeededStateChange::Generate))
                        }
                        ChunkState::Generating => {}
                        ChunkState::Generated => {
                            changes.push((*pos, entity, NeededStateChange::Render))
                        }
                        ChunkState::Rendering | ChunkState::Rendered => {}
                    },
                }
            } else {
                // It's not within the radius, so queue for deletion.
                changes.push((*pos, entity, NeededStateChange::Delete))
            }
        }

        changes
    }

    fn neighbors(&self, chunk: IVec3) -> Option<NeighborChunkSlices> {
        let slice_dirs = SLICE_DIRECTIONS.map(|direction| {
            let norm = direction.normal();
            let chunk_pos = chunk + norm.to_ivec3();
            (direction, self.chunks.get(&chunk_pos))
        });

        let mut output = NeighborChunkSlices::default();

        for (direction, chunk) in slice_dirs {
            if let Some(LoadedChunk {
                chunk: Some(chunk), ..
            }) = chunk
            {
                *output.get_in_direction_mut(direction.normal()) = chunk
                    .edge_slice_bits
                    .get_in_direction(direction.normal().negate())
                    .clone();
            } else {
                return None;
            };
        }

        Some(output)
    }

    fn execute_state_changes(
        &mut self,
        commands: &mut Commands,
        noise: &WorldNoiseSettings,
        changes: Vec<(IVec3, Entity, NeededStateChange)>,
    ) {
        let pool = AsyncComputeTaskPool::get();

        'outer: for (pos, entity, change) in changes {
            match change {
                NeededStateChange::Generate => {
                    let Some(chunk) = self.chunks.get_mut(&pos) else {
                        warn!("chunk at {pos} needs generated but it's not in the map");
                        continue 'outer;
                    };

                    chunk.state = ChunkState::Generating;
                    let noise = noise.clone();
                    commands.entity(entity).insert(GenerateTask(
                        pos,
                        pool.spawn(async move {
                            let new_noise = noise.generate_chunk_2d_noise(IVec2::new(pos.x, pos.z));
                            noise.generate_chunk_from_noise(pos.y, &new_noise)
                        }),
                    ));
                }
                NeededStateChange::Render => {
                    let (Some(neighbors), Some(chunk)) =
                        (self.neighbors(pos), self.chunks.get_mut(&pos))
                    else {
                        continue 'outer;
                    };

                    chunk.state = ChunkState::Rendering;
                    if let Some(cloned_chunk) = chunk.chunk.clone() {
                        commands.entity(entity).insert(RenderTask(
                            pos,
                            pool.spawn(async move {
                                crate::voxel::generate_mesh(&cloned_chunk, neighbors)
                            }),
                        ));
                    }
                }
                NeededStateChange::Delete => {
                    commands.entity(entity).despawn();
                    self.chunks.remove(&pos);
                }
            }
        }
    }

    //noinspection DuplicatedCode
    fn collect_finished_tasks(
        &mut self,
        commands: &mut Commands,
        material: &ChunkMaterialRes,
        meshes: &mut Assets<Mesh>,
        generate_query: &mut Query<(Entity, &mut GenerateTask), Without<RenderTask>>,
        render_query: &mut Query<(Entity, &mut RenderTask), Without<GenerateTask>>,
    ) {
        let generated_chunks = generate_query
            .iter_mut()
            .filter_map(|(entity, mut task)| {
                block_on(poll_once(&mut task.1)).map(|chunk| (task.0, entity, chunk))
            })
            .collect::<Vec<_>>();

        for (pos, entity, chunk) in generated_chunks {
            let Some(wrapper) = self.chunks.get_mut(&pos) else {
                continue;
            };

            wrapper.chunk = Some(chunk);
            wrapper.state = ChunkState::Generated;
            commands.entity(entity).remove::<GenerateTask>();
        }

        let rendered_chunks = render_query
            .iter_mut()
            .filter_map(|(entity, mut task)| {
                block_on(poll_once(&mut task.1)).map(|chunk| (task.0, entity, chunk))
            })
            .collect::<Vec<_>>();

        for (pos, entity, opt) in rendered_chunks {
            let Some(wrapper) = self.chunks.get_mut(&pos) else {
                continue;
            };

            wrapper.state = ChunkState::Rendered;

            let mut e = commands.entity(entity);
            e.remove::<RenderTask>();
            if let Some((collider, mesh)) = opt {
                e.insert((
                    MaterialMeshBundle {
                        mesh: meshes.add(mesh),
                        material: Handle::clone(&material.0),
                        transform: ChunkPos { pos }.transform(),
                        ..default()
                    },
                    collider,
                    RigidBody::Fixed,
                ));
            }
        }
    }
}
