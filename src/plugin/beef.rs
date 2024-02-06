use super::{
    chunk_loader::ChunkLoader, chunk_pos::ChunkPos, controller_2::CharControl2,
    game_settings::GameSettings, voxel_material::ChunkMaterialRes,
};
use crate::{
    io::{read_chunk_from_file, write_chunk_to_file},
    plugin::saver::IoCleanChunk,
    voxel::{
        world_noise::WorldNoiseSettings, Chunk, NeighborChunkSlices, CHUNK_WIDTH, SLICE_DIRECTIONS,
    },
};
use bevy::{
    diagnostic::{Diagnostic, DiagnosticId, Diagnostics, RegisterDiagnostic},
    ecs::system::EntityCommands,
    prelude::*,
    render::primitives::Aabb,
    tasks::{block_on, AsyncComputeTaskPool, Task},
    utils::HashMap,
};
use bevy_rapier3d::{dynamics::RigidBody, prelude::Collider};
use futures_lite::future::poll_once;
use itertools::iproduct;

pub const DIAG_GENERATE_REQUIRED: DiagnosticId =
    DiagnosticId::from_u128(20645138512437775160238241943797);
pub const DIAG_RENDER_REQUIRED: DiagnosticId =
    DiagnosticId::from_u128(191826711173120112441272453013);
pub const DIAG_DELETE_REQUIRED: DiagnosticId = DiagnosticId::from_u128(5852159751189146019716253);
pub const DIAG_GENERATED_CHUNKS: DiagnosticId =
    DiagnosticId::from_u128(146207248162236223461132471538);
pub const DIAG_RENDERED_CHUNKS: DiagnosticId =
    DiagnosticId::from_u128(97107192285412175912551555411386);
pub const DIAG_VISIBLE_CHUNKS: DiagnosticId = DiagnosticId::from_u128(71159199863847276575201272);
pub const DIAG_DIRTY_CHUNKS: DiagnosticId = DiagnosticId::from_u128(1071412727699475159529421);
pub const DIAG_NON_CULLED_CHUNKS: DiagnosticId = DiagnosticId::from_u128(1181181887682219941);

// Best (Even Ever?) Fucker: my chunk loading solution.
pub struct BeefPlugin;

impl Plugin for BeefPlugin {
    fn build(&self, app: &mut App) {
        app.register_diagnostic(Diagnostic::new(
            DIAG_GENERATE_REQUIRED,
            "required_generate_chunks",
            2,
        ))
        .register_diagnostic(Diagnostic::new(
            DIAG_RENDER_REQUIRED,
            "required_render_chunks",
            2,
        ))
        .register_diagnostic(Diagnostic::new(
            DIAG_DELETE_REQUIRED,
            "required_delete_chunks",
            2,
        ))
        .register_diagnostic(Diagnostic::new(
            DIAG_GENERATED_CHUNKS,
            "generated_chunks",
            2,
        ))
        .register_diagnostic(Diagnostic::new(DIAG_RENDERED_CHUNKS, "rendered_chunks", 2))
        .register_diagnostic(Diagnostic::new(DIAG_DIRTY_CHUNKS, "dirty_chunks", 2))
        .register_diagnostic(Diagnostic::new(DIAG_VISIBLE_CHUNKS, "visible_chunks", 2))
        .register_diagnostic(Diagnostic::new(
            DIAG_NON_CULLED_CHUNKS,
            "non_culled_chunks",
            2,
        ))
        .add_systems(
            Update,
            (
                check_dirty_edges,
                update_dirty_chunks_system,
                update_loader_states,
                update_diagnostics,
                start_loading,
                check_queue,
            )
                .chain()
                .run_if(resource_exists::<FixedChunkWorld>())
                .run_if(resource_exists::<WorldNoiseSettings>()),
        )
        .add_systems(
            Update,
            update_loader_radius.run_if(resource_changed::<GameSettings>()),
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
            let neighbor_pos = dirty_edge_chunk + neighbor_dir.normal().to_ivec3();
            if let Some(LoadedChunk {
                entity: neighbor_entity,
                ..
            }) = chunk_world.chunks.get(&neighbor_pos)
            {
                // Mark it as dirty
                commands.entity(*neighbor_entity).insert(DirtyChunk);
            }
        }
    }
}

/// Ugly but does stuff
fn update_diagnostics(
    mut diagnostics: Diagnostics,
    chunk_world: Res<FixedChunkWorld>,
    dirty_chunks: Query<(), With<DirtyChunk>>,
    visible_mesh_chunks: Query<&ViewVisibility, (With<ChunkEntity>, With<Handle<Mesh>>)>,
) {
    let mut generating_count = 0;
    let mut rendering_count = 0;
    let mut generated_count = 0;
    let mut rendered_count = 0;
    let dirty_count = dirty_chunks.iter().count();
    let visible_count = visible_mesh_chunks
        .iter()
        .map(|v| v.get())
        .collect::<Vec<_>>();
    let non_culled_count = visible_count.iter().filter(|b| **b).count();

    for state in chunk_world.chunks.values().map(|chunk| chunk.state) {
        match state {
            ChunkState::Empty => {}
            ChunkState::Generating => generating_count += 1,
            ChunkState::Generated => generated_count += 1,
            ChunkState::Rendering => rendering_count += 1,
            ChunkState::Rendered => rendered_count += 1,
        }
    }

    diagnostics.add_measurement(DIAG_GENERATE_REQUIRED, || generating_count as f64);
    diagnostics.add_measurement(DIAG_RENDER_REQUIRED, || rendering_count as f64);
    diagnostics.add_measurement(DIAG_GENERATED_CHUNKS, || generated_count as f64);
    diagnostics.add_measurement(DIAG_RENDERED_CHUNKS, || rendered_count as f64);
    diagnostics.add_measurement(DIAG_DIRTY_CHUNKS, || dirty_count as f64);
    diagnostics.add_measurement(DIAG_VISIBLE_CHUNKS, || visible_count.len() as f64);
    diagnostics.add_measurement(DIAG_NON_CULLED_CHUNKS, || non_culled_count as f64);
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
            chunk_world.chunks.get(&chunk_pos),
            chunk_world.neighbors(chunk_pos),
        ) {
            // Get this entity
            let mut cmds = commands.entity(entity);
            // Remove the dirty marker
            cmds.remove::<DirtyChunk>();
            // Remove the marker for chunks that do *not* need to be saved
            cmds.remove::<IoCleanChunk>();
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
            chunks.update_needed_chunk_states(&mut commands, loader_pos.pos, *radius as usize);
        }
    } else {
        // If the settings haven't been changed, check for changed loader
        // positions.
        for (loader_pos, ChunkLoader { radius }) in changed_loaders.iter() {
            chunks.update_needed_chunk_states(&mut commands, loader_pos.pos, *radius as usize);
        }
    }
}

/// System to queue loading for necessary chunks.
fn start_loading(
    mut diagnostics: Diagnostics,
    mut commands: Commands,
    mut chunks: ResMut<FixedChunkWorld>,
    noise: Res<WorldNoiseSettings>,
    loaders: Query<(&ChunkPos, &ChunkLoader)>,
) {
    if let Ok((loader_pos, ChunkLoader { radius })) = loaders.get_single() {
        // Determine which chunks have states that need to change
        let state_changes = chunks.required_state_changes(loader_pos.pos, *radius as usize);
        // Start executing the state changes
        chunks.execute_state_changes(&mut diagnostics, &mut commands, &noise, state_changes);
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
pub struct DirtyChunk;

#[derive(Component)]
pub struct NewChunkMesh(pub Handle<Mesh>, pub Collider);

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

pub(crate) struct LoadedChunk {
    pub entity: Entity,
    pub chunk: Option<Chunk>,
    pub state: ChunkState,
    pub needed_state: NeededChunkState,
    #[allow(unused)]
    pub pos: IVec3,
}

#[allow(unused)]
#[derive(Resource)]
pub struct FixedChunkWorld {
    name: String,
    seed: u32,
    pub(crate) chunks: HashMap<IVec3, LoadedChunk>,
}

impl FixedChunkWorld {
    pub fn new(name: String, seed: u32) -> Self {
        Self {
            name,
            seed,
            chunks: default(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set the provided chunk's needed state to the one provided, spawning
    /// the chunk entity is one does not already exist.
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
                entity: commands
                    .spawn((
                        ChunkEntity(chunk),
                        Aabb::from_min_max(Vec3::ZERO, UVec3::splat(CHUNK_WIDTH).as_vec3()),
                        RigidBody::Fixed,
                    ))
                    .id(),
                chunk: None,
                state: ChunkState::Empty,
                needed_state,
                pos: chunk,
            });
    }

    /// Update needed chunk states in a given radius around the chunk loader
    /// at the provided position. A radius of 1 will only generate chunks
    /// beyond the chunk containing the chunk loader, leading to a
    /// "one chunk" experience being rendered.
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

    /// Determine which states need to change based on the current state and
    /// the needed state.
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

        // Sort by distance to the chunk loader to coax closer chunks into
        // loading first. The order isn't stable, but this means closer chunk
        // tasks should be spawned first.
        changes.sort_unstable_by(|(pos1, ..), (pos2, ..)| {
            (*pos1 - loader_chunk)
                .abs()
                .min_element()
                .cmp(&(*pos2 - loader_chunk).abs().min_element())
        });

        changes
    }

    /// Get the solid face bitmap for each chunk neighboring the provided one
    /// to determine whether sides of edge voxels should be culled.
    fn neighbors(&self, chunk: IVec3) -> Option<NeighborChunkSlices> {
        // Get the chunks in each direction
        let slice_dirs = SLICE_DIRECTIONS.map(|direction| {
            let norm = direction.normal();
            let chunk_pos = chunk + norm.to_ivec3();
            (direction, self.chunks.get(&chunk_pos))
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
        diagnostics: &mut Diagnostics,
        commands: &mut Commands,
        noise: &WorldNoiseSettings,
        changes: Vec<(IVec3, Entity, NeededStateChange)>,
    ) {
        let async_pool = AsyncComputeTaskPool::get();

        let mut delete_count = 0;

        'outer: for (pos, entity, change) in changes {
            match change {
                NeededStateChange::Generate => {
                    let Some(chunk) = self.chunks.get_mut(&pos) else {
                        warn!("chunk at {pos} needs generated but it's not in the map");
                        continue 'outer;
                    };

                    // Update the current state
                    chunk.state = ChunkState::Generating;
                    // Make clones to send to task
                    let noise = noise.clone();
                    let name = self.name.clone();

                    // Insert the task into the chunk entity
                    commands.entity(entity).insert(GenerateTask(
                        pos,
                        async_pool.spawn(async move {
                            match read_chunk_from_file(&name, pos) {
                                // Load from disk
                                Some(existing_chunk) => {
                                    debug!("loaded chunk at {pos}");
                                    Chunk::from_container(existing_chunk)
                                }
                                // Generate with noise
                                None => {
                                    let new_noise =
                                        noise.generate_chunk_2d_noise(IVec2::new(pos.x, pos.z));
                                    noise.generate_chunk_from_noise(pos.y, &new_noise)
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
                    ) = (self.neighbors(pos), self.chunks.get_mut(&pos))
                    {
                        // Update the state
                        *state = ChunkState::Rendering;
                        // Clone the chunk to send to the task
                        let cloned_chunk = voxels.clone();
                        // Insert the task into the chunk entity
                        commands.entity(entity).insert(RenderTask(
                            pos,
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
                        let world_name = self.name.to_string();
                        let cloned_chunk = chunk.clone();
                        async_pool
                            .spawn(async move {
                                write_chunk_to_file(&world_name, pos, &cloned_chunk.voxels)
                            })
                            .detach();
                    };
                    // Despawn the entity
                    commands.entity(entity).despawn();
                    delete_count += 1;
                }
            }
        }

        diagnostics.add_measurement(DIAG_DELETE_REQUIRED, || delete_count as f64);
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
    ) {
        // Collect the chunks that have finished generating
        let generated_chunks = generate_query
            .iter_mut()
            .filter_map(|(entity, mut task)| {
                if commands.get_entity(entity).is_some() {
                    block_on(poll_once(&mut task.1)).map(|chunk| (task.0, entity, chunk))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for (pos, entity, chunk) in generated_chunks {
            // Make sure the chunk is still loaded
            let Some(wrapper) = self.chunks.get_mut(&pos) else {
                continue;
            };

            // Update the chunk and state
            wrapper.chunk = Some(chunk);
            wrapper.state = ChunkState::Generated;

            // Remove the task from this entity
            commands.entity(entity).remove::<GenerateTask>();
        }

        // Collect finished rendering tasks
        let rendered_chunks = render_query
            .iter_mut()
            .filter_map(|(entity, mut task)| {
                block_on(poll_once(&mut task.1)).map(|chunk| (task.0, entity, chunk))
            })
            .collect::<Vec<_>>();

        for (pos, entity, opt) in rendered_chunks {
            // Make sure this chunk is still loaded
            let Some(wrapper) = self.chunks.get_mut(&pos) else {
                continue;
            };

            // Update the state
            wrapper.state = ChunkState::Rendered;

            let mut e = commands.entity(entity);
            // Remove the render task
            e.remove::<RenderTask>();

            // Insert the mesh information if it is not empty
            if let Some((collider, mesh)) = opt {
                make_mesh_bundle(&mut e, pos, meshes, material, collider, mesh);
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
            transform: ChunkPos { pos }.transform(),
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
