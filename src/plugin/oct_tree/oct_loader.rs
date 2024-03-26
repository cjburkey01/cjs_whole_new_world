use super::{
    LodChunk, LodChunkEntity, LodNeededState, LodPos, LodRenderTask, LodRenderTaskReturn, LodState,
    LodWorld, WorldState,
};
use crate::{
    voxel::{ChunkPos, InChunkPos, RegionHandler, Voxel, VoxelContainer, CHUNK_WIDTH},
    voxel_world::region_saver::RegionHandlerRes,
};
use bevy::{
    prelude::*, tasks::AsyncComputeTaskPool, time::common_conditions::on_timer, utils::Entry,
};
use futures_lite::future::{block_on, poll_once};
use itertools::iproduct;
use std::{
    cmp::Ordering,
    sync::{Arc, RwLock},
    time::Duration,
};

pub struct OctLoaderPlugin;

impl Plugin for OctLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                prepare_and_execute_chunk_changes_system,
                try_spawn_render_tasks_system,
                check_for_rendered_chunks_system,
            )
                .chain()
                .run_if(not(in_state(WorldState::NotInWorld)))
                .run_if(on_timer(Duration::from_millis(500))),
        );
    }
}

#[derive(Component)]
pub struct LodLoader {
    pub level_half_thicks: Vec<u8>,
}

impl LodLoader {
    pub fn new(levels: &[u8]) -> Self {
        Self {
            level_half_thicks: levels.to_vec(),
        }
    }
}

// TODO: TESTS?

fn determine_changes(
    lod_world: &LodWorld,
    center_lod0: IVec3,
    loader: &LodLoader,
) -> Vec<(LodPos, LodNeededState)> {
    let required_chunks =
        LodWorld::required_levels_for_loader(center_lod0, &loader.level_half_thicks);
    let mut changes = lod_world.changes_for_required_levels(required_chunks);
    changes.sort_by(|(a, _), (b, _)| match a.level.cmp(&b.level) {
        Ordering::Equal => a
            .pos
            .distance_squared(center_lod0)
            .cmp(&b.pos.distance_squared(center_lod0)),
        lt_gt => lt_gt,
    });
    changes
}

fn prepare_and_execute_chunk_changes_system(
    mut commands: Commands,
    mut lod_world: ResMut<LodWorld>,
    loaders: Query<(&ChunkPos, &LodLoader)>,
    region_handler: Res<RegionHandlerRes>,
) {
    let Ok((ChunkPos(center_lod0_chunk), loader)) = loaders.get_single() else {
        return;
    };

    let changes_to_perform = determine_changes(&lod_world, *center_lod0_chunk, loader);

    debug!("{} lod chunk changes needed", changes_to_perform.len());

    // I think we only need one main chunk loader. Keeping other chunks loaded
    // will not have anything to do with the lod display.
    for (pos, state_change) in changes_to_perform {
        let level = lod_world.tree.level_mut(pos.level);
        match state_change {
            LodNeededState::Deleted => {
                let lod_chunk = level.remove(&pos.pos);
                if let Some(LodChunk { entity, .. }) = lod_chunk {
                    if let Some(mut cmd) = commands.get_entity(entity) {
                        cmd.despawn();
                    }
                }
            }
            LodNeededState::Render => match level.entry(pos.pos) {
                Entry::Occupied(mut entry) => {
                    let chunk = entry.get_mut();
                    chunk.current_state = LodState::Loading;
                }
                Entry::Vacant(entry) => {
                    entry.insert(LodChunk {
                        entity: commands.spawn(LodChunkEntity(pos)).id(),
                        current_state: LodState::Loading,
                        lod_data: None,
                    });
                }
            },
        }
    }
}

fn try_spawn_render_tasks_system(
    mut commands: Commands,
    regions: Res<RegionHandlerRes>,
    lod_world: Res<LodWorld>,
    awaiting_render_chunks: Query<(Entity, &LodChunkEntity), Without<LodRenderTask>>,
) {
    let task_pool = AsyncComputeTaskPool::get();
    for (entity, LodChunkEntity(lod_chunk_pos)) in awaiting_render_chunks.iter() {
        if let Some(lod_chunk) = try_make_lod_chunk(*lod_chunk_pos, &regions.0) {
            // TODO: THIS
            commands.entity(entity).insert(LodRenderTask(
                *lod_chunk_pos,
                task_pool.spawn(async move {
                    //crate::voxel::generate_mesh(&lod_chunk, NeighborChunkSlices::default())
                    LodRenderTaskReturn
                }),
            ));
        }
    }
}

fn check_for_rendered_chunks_system(
    mut commands: Commands,
    mut lod_world: ResMut<LodWorld>,
    mut chunks: Query<(Entity, &mut LodRenderTask)>,
) {
    for (entity, mut task) in chunks.iter_mut() {
        let lod_pos = task.0;
        if let Some(_blah) = block_on(poll_once(&mut task.1)) {
            commands.entity(entity).remove::<LodRenderTask>();

            // TODO: ADD MESH
        }
        lod_world.set_state_if_present(lod_pos, LodState::Ready);
    }
}

/// Tries to create the lod chunk by sampling the lod0 chunks at the necessary
/// intervals to make a lower-res version of the chunk.
/// There are a bunch of better ways to do this, so:
/// TODO: MAKE THIS **WAY** MORE EFFICIENT I THINK?
///       IT SHOULDN'T BE THIS EXPENSIVE!
///       BUT IT WILL BE RIGHT NOW!
fn try_make_lod_chunk(
    lod_chunk_pos: LodPos,
    regions: &Arc<RwLock<RegionHandler>>,
) -> Option<VoxelContainer> {
    let lod0_pos = lod_chunk_pos.to_level(0).pos;
    let lod0_width = lod_chunk_pos.lod0_width();

    let region_handler = regions.read().ok()?;
    let chunks = iproduct!(0..lod0_width, 0..lod0_width, 0..lod0_width)
        .map(|(x, y, z)| region_handler.chunk(ChunkPos(lod0_pos + UVec3::new(x, y, z).as_ivec3())))
        .collect::<Option<Vec<_>>>()?;

    let mut fake_chunk = VoxelContainer::from_voxel(Voxel::Air);
    for (x, y, z) in iproduct!(0..CHUNK_WIDTH, 0..CHUNK_WIDTH, 0..CHUNK_WIDTH) {
        // TODO: SAMPLE CHUNKS
        let sample_pos = UVec3::new(x, y, z);
        let sample_offset = sample_pos * lod0_width;
        let sample_chunk_pos = sample_offset / CHUNK_WIDTH;
        let sample_chunk_offset_pos = InChunkPos::from_urem(sample_offset);
        let sample_index = {
            let UVec3 { x, y, z } = sample_chunk_pos;
            (x * lod0_width * lod0_width + y * lod0_width + z) as usize
        };
        fake_chunk.set(
            InChunkPos::new(sample_pos).unwrap(),
            chunks[sample_index].at(sample_chunk_offset_pos),
        );
    }

    Some(fake_chunk)
}
