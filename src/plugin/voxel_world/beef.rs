use crate::{
    plugin::{
        control::controller_2::CharControl2,
        game_settings::GameSettings,
        voxel_world::{
            chunk_loader::ChunkLoader, region_saver::RegionHandlerRes,
            voxel_material::ChunkMaterialRes, world_info::WorldInfo,
        },
    },
    voxel::{
        world_noise::{Chunk2dNoiseValues, WorldNoiseSettings},
        Chunk, ChunkPos, InRegionChunkPos, NeighborChunkSlices, CHUNK_WIDTH, SLICE_DIRECTIONS,
    },
};
use bevy::{
    ecs::system::EntityCommands,
    prelude::*,
    render::primitives::Aabb,
    tasks::{block_on, AsyncComputeTaskPool, Task},
    utils::HashMap,
};
use bevy_rapier3d::{dynamics::RigidBody, prelude::Collider};
use futures_lite::future::poll_once;
use itertools::iproduct;
use std::{cmp::Ordering, sync::Arc};

// Best (Even Ever?) Fucker: my chunk loading solution.
pub struct BeefPlugin;

impl Plugin for BeefPlugin {
    fn build(&self, app: &mut App) {
        app
            // Update systems
            .add_systems(
                Update,
                (
                    (
                        check_dirty_edges,
                        update_dirty_chunks_system,
                        update_loader_states,
                        start_loading,
                        check_queue,
                    )
                        .chain()
                        .run_if(resource_exists::<FixedChunkWorld>())
                        .run_if(resource_exists::<WorldNoiseSettings>()),
                    update_loader_radius.run_if(resource_changed::<GameSettings>()),
                ),
            );
    }
}

// TODO: ALLOW MARKING INDIVIDUAL SIDES AS DIRTY
/// System to check for any chunks that need their edges updated.
fn check_dirty_edges(mut commands: Commands, mut chunk_world: ResMut<FixedChunkWorld>) {
    let mut dirty_edge_chunks = vec![];

    // Loop through all currently loaded chunks
    for (pos, loaded_chunk) in chunk_world.chunks.iter_mut() {
        // Make sure this chunk is already loaded
        if let Some(chunk) = &mut loaded_chunk.chunk {
            // Update edges
            if chunk.edges_dirty {
                chunk.update_edge_slice_bits();
                dirty_edge_chunks.push(*pos);
            }
        }
    }

    // When we update a chunk that requires updating one of its edge slices,
    // we mark the the neighbors as dirty to ensure no gaps
    for dirty_edge_chunk in dirty_edge_chunks {
        // Loop through each direction
        for neighbor_dir in SLICE_DIRECTIONS {
            // Get the entity for the chunk in this direction
            let neighbor_pos = dirty_edge_chunk.0 + neighbor_dir.normal().to_ivec3();
            if let Some(LoadedChunk {
                entity: neighbor_entity,
                ..
            }) = chunk_world.chunks.get(&ChunkPos(neighbor_pos))
            {
                // Mark it as dirty
                commands.entity(*neighbor_entity).insert(DirtyChunk);
            }
        }
    }
}

/// System to perform immediate updates on chunks currently marked as dirty.
/// It is important not to mark many chunks as dirty (until I add stuff to
/// handle that), their updates are performed synchronously in this system.
fn update_dirty_chunks_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<ChunkMaterialRes>,
    chunk_world: Res<FixedChunkWorld>,
    dirty_chunks: Query<(Entity, &ChunkEntity), With<DirtyChunk>>,
) {
    for (entity, chunk_pos) in dirty_chunks
        .iter()
        .map(|(entity, ChunkEntity(pos))| (entity, *pos))
    {
        // Check if this dirty chunk is already generated. Otherwise, not sure how we got here.
        // Also retrieve this chunk's neighbors to render this one
        if let (
            Some(LoadedChunk {
                state: ChunkState::Rendered,
                chunk: Some(chunk_voxels),
                ..
            }),
            Some(neighbors),
        ) = (
            chunk_world.chunks.get(&ChunkPos(chunk_pos)),
            chunk_world.neighbors(chunk_pos),
        ) {
            // Get this entity
            let mut cmds = commands.entity(entity);
            // Remove the dirty marker
            cmds.remove::<DirtyChunk>();
            // Remove the marker for chunks that do *not* need to be saved
            // Generate the mesh
            if let Some((collider, mesh)) = crate::voxel::generate_mesh(chunk_voxels, neighbors) {
                // Insert the mesh bundle; this is the same function that is
                // called when a chunk has finished rendering asynchronously,
                // in the case that this chunk previously did not have a mesh;
                // in that case, this entity would not have a material handle,
                // so we just insert all of this just in case.
                make_mesh_bundle(&mut cmds, chunk_pos, &mut meshes, &material, collider, mesh);
            } else {
                // If the mesh generation returned None, it means the mesh is
                // empty, so we can go ahead and remove any mesh handle that
                // may or may not exist on this entity to ensure removing the
                // last voxel in a chunk doesn't just leave the voxel ghost.
                cmds.remove::<Handle<Mesh>>();
            }
        }
    }
}

/// System to check for chunk loader pos/radius changes and update
/// which chunks are required to be in which states.
fn update_loader_states(
    mut commands: Commands,
    mut chunks: ResMut<FixedChunkWorld>,
    game_settings: Res<GameSettings>,
    loaders: Query<(&ChunkPos, &ChunkLoader)>,
    changed_loaders: Query<(&ChunkPos, &ChunkLoader), Changed<ChunkPos>>,
) {
    // If the settings have been changed, check for state changes
    if game_settings.is_changed() {
        // Only one chunk loader for now
        for (loader_pos, ChunkLoader { radius }) in loaders.iter() {
            chunks.update_needed_chunk_states(&mut commands, *loader_pos, *radius as usize);
        }
    } else {
        // If the settings haven't been changed, check for changed loader
        // positions.
        for (loader_pos, ChunkLoader { radius }) in changed_loaders.iter() {
            chunks.update_needed_chunk_states(&mut commands, *loader_pos, *radius as usize);
        }
    }
}

/// System to queue loading for necessary chunks.
fn start_loading(
    mut commands: Commands,
    world_info: Res<WorldInfo>,
    mut chunks: ResMut<FixedChunkWorld>,
    region_handler: Res<RegionHandlerRes>,
    noise: Res<WorldNoiseSettings>,
    loaders: Query<(&ChunkPos, &ChunkLoader)>,
) {
    if let Ok((loader_pos, ChunkLoader { radius })) = loaders.get_single() {
        // Determine which chunks have states that need to change
        let state_changes = chunks.required_state_changes(*loader_pos, *radius as usize);
        // Start executing the state changes
        chunks.execute_state_changes(
            &mut commands,
            world_info.name(),
            &region_handler,
            &noise,
            state_changes,
        );
    }
}

/// System to check for any finished async generation/render tasks.
fn check_queue(
    mut commands: Commands,
    material: Res<ChunkMaterialRes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunks: ResMut<FixedChunkWorld>,
    mut generate_query: Query<(Entity, &mut GenerateTask), Without<RenderTask>>,
    mut render_query: Query<(Entity, &mut RenderTask), Without<GenerateTask>>,
    loader: Query<&ChunkPos, With<ChunkLoader>>,
) {
    chunks.collect_finished_tasks(
        &mut commands,
        &material,
        &mut meshes,
        &mut generate_query,
        &mut render_query,
        &loader,
    );
}

#[derive(Debug, Component, Copy, Clone, Eq, PartialEq)]
pub struct ChunkEntity(pub IVec3);

#[derive(Component)]
pub struct DirtyChunk;

#[derive(Component)]
struct GenerateTask(IVec3, Task<(Chunk, Option<Chunk2dNoiseValues>)>);

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

pub(crate) struct LoadedChunk {
    pub entity: Entity,
    pub chunk: Option<Chunk>,
    pub state: ChunkState,
    pub needed_state: NeededChunkState,
    #[allow(unused)]
    pub pos: IVec3,
}

#[allow(unused)]
#[derive(Default, Resource)]
pub struct FixedChunkWorld {
    pub(crate) chunks: HashMap<ChunkPos, LoadedChunk>,
    pub(crate) heightmaps: HashMap<IVec2, Chunk2dNoiseValues>,
}

impl FixedChunkWorld {
    /// Set the provided chunk's needed state to the one provided, spawning
    /// the chunk entity is one does not already exist.
    pub fn set_needed<'w: 'a, 's: 'a, 'a>(
        &mut self,
        commands: &mut Commands,
        chunk: ChunkPos,
        needed_state: NeededChunkState,
    ) {
        let _ = self
            .chunks
            .entry(chunk)
            .and_modify(|c| c.needed_state = needed_state)
            .or_insert_with(|| LoadedChunk {
                entity: commands
                    .spawn((
                        ChunkEntity(chunk.0),
                        Aabb::from_min_max(Vec3::ZERO, UVec3::splat(CHUNK_WIDTH).as_vec3()),
                        RigidBody::Fixed,
                    ))
                    .id(),
                chunk: None,
                state: ChunkState::Empty,
                needed_state,
                pos: chunk.0,
            });
    }

    /// Update needed chunk states in a given radius around the chunk loader
    /// at the provided position. A radius of 1 will only generate chunks
    /// beyond the chunk containing the chunk loader, leading to a
    /// "one chunk" experience being rendered.
    pub fn update_needed_chunk_states<'w: 'a, 's: 'a, 'a>(
        &mut self,
        commands: &mut Commands,
        loader_chunk: ChunkPos,
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
            self.set_needed(commands, ChunkPos(loader_chunk.0 + offset), needed_state);
        }
    }

    /// Determine which states need to change based on the current state and
    /// the needed state.
    fn required_state_changes(
        &self,
        loader_chunk: ChunkPos,
        radius: usize,
    ) -> Vec<(ChunkPos, Entity, NeededStateChange)> {
        let mut changes = vec![];
        let radius_i = radius as i32;

        // Loop through existing chunks
        for (pos, chunk) in self.chunks.iter() {
            let offset_pos = pos.0 - loader_chunk.0;
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

        // Sort by distance to the chunk loader to coax closer chunks into
        // loading first. The order isn't stable, but this means closer chunk
        // tasks should be spawned first.
        changes.sort_unstable_by(|(pos1, ..), (pos2, ..)| {
            Self::sort_by_distance(loader_chunk.0, pos1.0, pos2.0)
        });

        changes
    }

    fn sort_by_distance(loader_pos: IVec3, pos1: IVec3, pos2: IVec3) -> Ordering {
        pos1.distance_squared(loader_pos)
            .cmp(&pos2.distance_squared(loader_pos))
    }

    /// Get the solid face bitmap for each chunk neighboring the provided one
    /// to determine whether sides of edge voxels should be culled.
    fn neighbors(&self, chunk: IVec3) -> Option<NeighborChunkSlices> {
        // Get the chunks in each direction
        let slice_dirs = SLICE_DIRECTIONS.map(|direction| {
            let norm = direction.normal();
            let chunk_pos = chunk + norm.to_ivec3();
            (direction, self.chunks.get(&ChunkPos(chunk_pos)))
        });

        let mut output = NeighborChunkSlices::default();

        for (direction, loaded_chunk) in slice_dirs {
            // Make sure this chunk is already generated.
            if let Some(LoadedChunk {
                chunk: Some(chunk), ..
            }) = loaded_chunk
            {
                *output.get_in_direction_mut(direction.normal()) = chunk
                    .edge_slice_bits
                    .get_in_direction(direction.normal().negate())
                    .clone();
            } else {
                // We need to return none now, not all neighboring chunks have
                // been generated.
                return None;
            };
        }

        Some(output)
    }

    /// Spawn the tasks to perform the state changes required.
    fn execute_state_changes(
        &mut self,
        commands: &mut Commands,
        name: &str,
        region_handler_res: &RegionHandlerRes,
        noise: &WorldNoiseSettings,
        changes: Vec<(ChunkPos, Entity, NeededStateChange)>,
    ) {
        let async_pool = AsyncComputeTaskPool::get();

        let mut delete_count = 0;

        'outer: for (pos, entity, change) in changes {
            match change {
                NeededStateChange::Generate => {
                    let Some(chunk) = self.chunks.get_mut(&pos) else {
                        warn!("chunk at {} needs generated but it's not in the map", pos.0);
                        continue 'outer;
                    };

                    // Update the current state
                    chunk.state = ChunkState::Generating;
                    // Make clones to send to task
                    let noise = noise.clone();
                    let name = name.to_string();

                    // Insert the task into the chunk entity
                    let region_handler_inner = Arc::clone(&region_handler_res.0);
                    let chunk_noise = self.heightmaps.get(&IVec2::new(pos.0.x, pos.0.z)).cloned();
                    commands.entity(entity).insert(GenerateTask(
                        pos.0,
                        async_pool.spawn(async move {
                            let needed_new_noise = chunk_noise.is_none();
                            let new_noise = chunk_noise.unwrap_or_else(|| {
                                noise.generate_chunk_2d_noise(IVec2::new(pos.0.x, pos.0.z))
                            });

                            match region_handler_inner.write() {
                                Ok(mut region_handler) => {
                                    match region_handler.check_for_chunk(&name, pos) {
                                        // Load from disk
                                        Some(existing_chunk) => (
                                            Chunk::from_container(existing_chunk.clone()),
                                            match needed_new_noise {
                                                true => Some(new_noise),
                                                false => None,
                                            },
                                        ),
                                        // Generate with noise
                                        None => (
                                            noise.generate_chunk_from_noise(pos.0.y, &new_noise),
                                            match needed_new_noise {
                                                true => Some(new_noise),
                                                false => None,
                                            },
                                        ),
                                    }
                                }
                                Err(_) => {
                                    panic!("u done fucked up");
                                }
                            }
                        }),
                    ));
                }
                NeededStateChange::Render => {
                    if let (
                        Some(neighbors),
                        Some(LoadedChunk {
                            state,
                            chunk: Some(voxels),
                            ..
                        }),
                    ) = (self.neighbors(pos.0), self.chunks.get_mut(&pos))
                    {
                        // Update the state
                        *state = ChunkState::Rendering;
                        // Clone the chunk to send to the task
                        let cloned_chunk = voxels.clone();
                        // Insert the task into the chunk entity
                        commands.entity(entity).insert(RenderTask(
                            pos.0,
                            async_pool.spawn(async move {
                                crate::voxel::generate_mesh(&cloned_chunk, neighbors)
                            }),
                        ));
                    }
                }
                NeededStateChange::Delete => {
                    // Remove the chunk from the chunk hashmap, stealing the
                    // chunk data to save if it existed
                    if let Some(LoadedChunk {
                        chunk: Some(chunk), ..
                    }) = self.chunks.remove(&pos)
                    {
                        // Try to save this chunk before it's deleted
                        match region_handler_res.0.write() {
                            Ok(mut region_handler) => {
                                *region_handler
                                    .region_mut(pos.into())
                                    .chunk_mut(InRegionChunkPos::from_world(pos)) =
                                    Some(chunk.voxels);
                            }
                            Err(_) => {
                                panic!("errrrrr")
                            }
                        }
                    };
                    // Despawn the entity
                    commands.entity(entity).despawn();
                    delete_count += 1;
                }
            }
        }
    }

    //noinspection DuplicatedCode
    /// Search for finished generation and render tasks.
    fn collect_finished_tasks(
        &mut self,
        commands: &mut Commands,
        material: &ChunkMaterialRes,
        meshes: &mut Assets<Mesh>,
        generate_query: &mut Query<(Entity, &mut GenerateTask), Without<RenderTask>>,
        render_query: &mut Query<(Entity, &mut RenderTask), Without<GenerateTask>>,
        loader: &Query<&ChunkPos, With<ChunkLoader>>,
    ) {
        // Collect the chunks that have finished generating
        let generated_chunks = generate_query
            .iter_mut()
            .filter_map(|(entity, mut task)| {
                if commands.get_entity(entity).is_some() {
                    block_on(poll_once(&mut task.1))
                        .map(|(chunk, heightmap)| (task.0, entity, chunk, heightmap))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for (pos, entity, chunk, heightmap) in generated_chunks {
            if let Some(new_heightmap) = heightmap {
                self.heightmaps
                    .insert(IVec2::new(pos.x, pos.z), new_heightmap);
            }

            // Make sure the chunk is still loaded
            let Some(wrapper) = self.chunks.get_mut(&ChunkPos(pos)) else {
                continue;
            };

            // Update the chunk and state
            wrapper.chunk = Some(chunk);
            wrapper.state = ChunkState::Generated;

            // Remove the task from this entity
            commands.entity(entity).remove::<GenerateTask>();
        }

        // Perform up to the maximum number of chunk meshes added to the
        // assets resource.

        let mut render_positions = render_query.iter_mut().collect::<Vec<_>>();
        if let Ok(loader_pos) = loader.get_single() {
            let loader_pos = loader_pos.0;
            render_positions.sort_unstable_by(|(_, task1), (_, task2)| {
                Self::sort_by_distance(loader_pos, task1.0, task2.0)
            });
        }

        let mut rendered_count = 0;
        'render_loop: for (entity, mut task) in render_positions {
            let pos = task.0;

            if let Some(optional_chunk) = block_on(poll_once(&mut task.1)) {
                // Make sure this chunk is still loaded
                let Some(wrapper) = self.chunks.get_mut(&ChunkPos(pos)) else {
                    continue;
                };

                // Update the state
                wrapper.state = ChunkState::Rendered;

                let mut e = commands.entity(entity);
                // Remove the render task
                e.remove::<RenderTask>();

                // Insert the mesh information if it is not empty
                if let Some((collider, mesh)) = optional_chunk {
                    make_mesh_bundle(&mut e, pos, meshes, material, collider, mesh);

                    rendered_count += 1;
                    if rendered_count >= 2 {
                        break 'render_loop;
                    }
                }
            }
        }
    }
}

/// Insert the necessary components for rendering into the provided chunk
/// entity.
fn make_mesh_bundle(
    commands: &mut EntityCommands,
    pos: IVec3,
    meshes: &mut Assets<Mesh>,
    material: &ChunkMaterialRes,
    collider: Collider,
    mesh: Mesh,
) {
    commands.insert((
        MaterialMeshBundle {
            mesh: meshes.add(mesh),
            material: Handle::clone(&material.0),
            transform: ChunkPos(pos).transform(),
            ..default()
        },
        collider,
    ));
}

/// System to update the main character controller's loader radius whenever the
/// options have been updated. Hopefully this system doesn't last too long.
fn update_loader_radius(
    settings: Res<GameSettings>,
    mut loader: Query<&mut ChunkLoader, With<CharControl2>>,
) {
    if let Ok(mut loader) = loader.get_single_mut() {
        loader.radius = settings.load_radius;
    }
}
