use super::{
    LodChunk, LodChunkEntity, LodNeededState, LodPos, LodRenderTask, LodState, LodWorld, WorldState,
};
use crate::{
    voxel::{ChunkPos, InChunkPos, Voxel, VoxelContainer, CHUNK_WIDTH},
    voxel_world::{chunk_loader::ChunkLoader, region_saver::RegionHandlerRes},
};
use bevy::{prelude::*, time::common_conditions::on_timer, utils::Entry};
use itertools::iproduct;
use std::{cmp::Ordering, time::Duration};

pub struct OctLoaderPlugin;

impl Plugin for OctLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                check_for_chunk_changes,
                execute_changes_system,
                try_spawn_render_tasks_system,
            )
                .chain()
                .run_if(in_state(WorldState::Ready))
                .run_if(on_timer(Duration::from_millis(500))),
        );
    }
}

// TODO: TESTS?

#[derive(Resource)]
struct ChangesToPerform(Vec<(LodPos, LodNeededState)>);

fn check_for_chunk_changes(
    lod_world: Res<LodWorld>,
    mut changes_to_perform: ResMut<ChangesToPerform>,
    loaders: Query<&ChunkPos, With<ChunkLoader>>,
) {
    // TODO: CUSTOMIZABLE LOD LEVELS
    let level_half_thicks = [3, 3, 3];

    // I think we only need one main chunk loader. Keeping other chunks loaded
    // will not have anything to do with the lod display.
    if let Ok(ChunkPos(center_lod0_chunk)) = loaders.get_single() {
        let required_chunks =
            LodWorld::required_levels_for_loader(*center_lod0_chunk, &level_half_thicks);
        let mut changes = lod_world.changes_for_required_levels(required_chunks);
        changes.sort_by(|(a, _), (b, _)| match a.level.cmp(&b.level) {
            Ordering::Equal => a
                .pos
                .distance_squared(*center_lod0_chunk)
                .cmp(&b.pos.distance_squared(*center_lod0_chunk)),
            lt_gt => lt_gt,
        });
        changes_to_perform.0 = changes;
    }
}

fn execute_changes_system(
    mut commands: Commands,
    mut lod_world: ResMut<LodWorld>,
    changes_to_perform: Res<ChangesToPerform>,
) {
    for (pos, state_change) in &changes_to_perform.0 {
        match *state_change {
            LodNeededState::Deleted => {
                let lod_chunk = lod_world.tree.level_mut(pos.level).remove(&pos.pos);
                if let Some(LodChunk { entity, .. }) = lod_chunk {
                    if let Some(mut cmd) = commands.get_entity(entity) {
                        cmd.despawn();
                    }
                }
            }
            LodNeededState::Render => match lod_world.tree.level_mut(pos.level).entry(pos.pos) {
                Entry::Occupied(mut entry) => {
                    let chunk = entry.get_mut();
                    chunk.current_state = LodState::Loading;
                }
                Entry::Vacant(entry) => {
                    entry.insert(LodChunk {
                        entity: commands.spawn(LodChunkEntity(*pos)).id(),
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
    for (entity, LodChunkEntity(lod_chunk_pos)) in awaiting_render_chunks.iter() {
        if let Some(lod_chunk) = try_make_lod_chunk(*lod_chunk_pos, &regions) {}
    }
}

/// Tries to create the lod chunk by sampling the lod0 chunks at the necessary
/// intervals to make a lower-res version of the chunk.
/// There are a bunch of better ways to do this, so:
/// TODO: MAKE THIS **WAY** MORE EFFICIENT
///       IT SHOULDN'T BE THIS EXPENSIVE!
///       BUT IT WILL BE RIGHT NOW!
fn try_make_lod_chunk(lod_chunk_pos: LodPos, regions: &RegionHandlerRes) -> Option<VoxelContainer> {
    let lod0_pos = lod_chunk_pos.to_level(0).pos;
    let lod0_width = lod_chunk_pos.lod0_width();

    let region_handler = regions.0.read().ok()?;
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
            InChunkPos::new(UVec3::new(x, y, z)).unwrap(),
            chunks[sample_index].at(sample_chunk_offset_pos),
        );
    }

    Some(fake_chunk)
}
