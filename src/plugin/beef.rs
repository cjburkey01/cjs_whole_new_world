use crate::{
    plugin::{
        chunk_loader::ChunkLoader, chunk_pos::ChunkPos, controller_2::CharControl2,
        game_settings::GameSettings, voxel_material::ChunkMaterialRes,
    },
    voxel::{
        world_noise::WorldNoiseSettings, Chunk, NeighborChunkSlices, CHUNK_WIDTH, SLICE_DIRECTIONS,
    },
};
use bevy::{
    diagnostic::{Diagnostic, DiagnosticId, Diagnostics, RegisterDiagnostic},
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
    DiagnosticId::from_u128(97107192228541175912551555411386);
pub const DIAG_DIRTY_CHUNKS: DiagnosticId = DiagnosticId::from_u128(1071412727657199475159529421);

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
        .add_systems(
            Update,
            (
                update_dirty_chunks,
                (
                    update_loader_states,
                    update_diagnostics,
                    start_loading,
                    check_queue,
                )
                    .chain(),
            )
                .run_if(resource_exists::<FixedChunkWorld>())
                .run_if(resource_exists::<WorldNoiseSettings>()),
        )
        .add_systems(
            Update,
            update_loader_radius.run_if(resource_changed::<GameSettings>()),
        );
    }
}

fn update_diagnostics(
    mut diagnostics: Diagnostics,
    chunk_world: Res<FixedChunkWorld>,
    dirty_chunks: Query<Entity, With<DirtyChunk>>,
) {
    let mut generating_count = 0;
    let mut rendering_count = 0;
    let mut generated_count = 0;
    let mut rendered_count = 0;
    let dirty_count = dirty_chunks.iter().count();

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
}

fn update_dirty_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    chunk_world: Res<FixedChunkWorld>,
    dirty_chunks: Query<&ChunkEntity, With<DirtyChunk>>,
) {
    for ChunkEntity(chunk_pos) in dirty_chunks.iter() {
        if let (
            Some(LoadedChunk {
                state: ChunkState::Rendered,
                chunk: Some(chunk_voxels),
                entity,
                ..
            }),
            Some(neighbors),
        ) = (
            chunk_world.chunks.get(chunk_pos),
            chunk_world.neighbors(*chunk_pos),
        ) {
            if let Some((collider, mesh)) = crate::voxel::generate_mesh(chunk_voxels, neighbors) {
                commands
                    .entity(*entity)
                    .remove::<DirtyChunk>()
                    .insert((meshes.add(mesh), collider));
            }
        }
    }
}

fn update_loader_states(
    mut commands: Commands,
    mut chunks: ResMut<FixedChunkWorld>,
    game_settings: Res<GameSettings>,
    loaders: Query<(&ChunkPos, &ChunkLoader)>,
    changed_loaders: Query<(&ChunkPos, &ChunkLoader), Changed<ChunkPos>>,
) {
    if game_settings.is_changed() {
        // Only one chunk loader for now
        for (loader_pos, ChunkLoader { radius }) in loaders.iter() {
            chunks.update_needed_chunk_states(&mut commands, loader_pos.pos, *radius as usize);
        }
    } else {
        for (loader_pos, ChunkLoader { radius }) in changed_loaders.iter() {
            chunks.update_needed_chunk_states(&mut commands, loader_pos.pos, *radius as usize);
        }
    }
}

fn start_loading(
    mut diagnostics: Diagnostics,
    mut commands: Commands,
    mut chunks: ResMut<FixedChunkWorld>,
    noise: Res<WorldNoiseSettings>,
    loaders: Query<(&ChunkPos, &ChunkLoader)>,
) {
    if let Ok((loader_pos, ChunkLoader { radius })) = loaders.get_single() {
        let state_changes = chunks.required_state_changes(loader_pos.pos, *radius as usize);
        chunks.execute_state_changes(&mut diagnostics, &mut commands, &noise, state_changes);
    }
}

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
                    ))
                    .id(),
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

        changes.sort_unstable_by(|(pos1, ..), (pos2, ..)| {
            (*pos1 - loader_chunk)
                .abs()
                .min_element()
                .cmp(&(*pos2 - loader_chunk).abs().min_element())
        });

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

                    chunk.state = ChunkState::Generating;
                    let noise = noise.clone();
                    commands.entity(entity).insert(GenerateTask(
                        pos,
                        async_pool.spawn(async move {
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
                            async_pool.spawn(async move {
                                crate::voxel::generate_mesh(&cloned_chunk, neighbors)
                            }),
                        ));
                    }
                }
                NeededStateChange::Delete => {
                    commands.entity(entity).despawn();
                    self.chunks.remove(&pos);

                    delete_count += 1;
                }
            }
        }

        diagnostics.add_measurement(DIAG_DELETE_REQUIRED, || delete_count as f64);
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
            // imma count it as a render even if the chunk is empty
        }
    }
}

fn update_loader_radius(
    settings: Res<GameSettings>,
    mut loader: Query<&mut ChunkLoader, With<CharControl2>>,
) {
    if let Ok(mut loader) = loader.get_single_mut() {
        loader.radius = settings.load_radius;
    }
}
