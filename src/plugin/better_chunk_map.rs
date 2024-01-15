// The better chunk map ought to:
//
// Keep a priority queue of chunks.
// Have a system to look for chunks not yet visible or being processed. Keep
// the stable priority queue for that, updating existing chunks and
// inserting when necessary (don't think i have to care). now we loop through a
// couple of the first ones, removing them and starting their tasks (limit to
// batches). The priority is going to be distance from chunk loader (can
// probably use distance squared or manhattan distance if i really feel like
// that specific part of the very hefty meshing task would slow down this
// particular system that *spawns* that task badly enough...
//

use crate::plugin::{
    chunk_map::Chunks,
    loading::{ChunkLoader, ChunkPos},
};
use bevy::{prelude::*, time::common_conditions::on_timer};
use itertools::iproduct;
use priority_queue::PriorityQueue;
use std::time::Duration;

pub struct Plugin3000;

impl Plugin for Plugin3000 {
    fn build(&self, app: &mut App) {
        app.init_resource::<BetterChunkLoaderManager3000>()
            .add_systems(
                Update,
                update_requesting_map.run_if(on_timer(Duration::from_millis(240))),
            );
    }
}

/// I'm not changing this name, ever.
#[derive(Resource)]
pub struct BetterChunkLoaderManager3000 {
    batch_size: usize,
    requesting: PriorityQueue<IVec3, i32>,
}

impl BetterChunkLoaderManager3000 {
    pub fn unload(&mut self, chunk: IVec3) {
        self.requesting.remove(&chunk);
    }
}

impl Default for BetterChunkLoaderManager3000 {
    fn default() -> Self {
        Self {
            batch_size: 40,
            requesting: default(),
        }
    }
}

fn update_requesting_map(
    mut commands: Commands,
    mut boring_map: ResMut<Chunks>,
    mut requesting: ResMut<BetterChunkLoaderManager3000>,
    loaders: Query<(&ChunkPos, &ChunkLoader)>,
) {
    for (ChunkPos { pos }, loader) in loaders.iter() {
        let r = loader.radius as i32;
        let diameter = r * 2;
        let dst_sqr_max = diameter * diameter + 2;

        for existing_prior_pos in requesting
            .requesting
            .iter()
            .map(|(a, _)| *a)
            .collect::<Vec<_>>()
        {
            let dst_sqr = pos.distance_squared(existing_prior_pos);
            if dst_sqr > dst_sqr_max {
                requesting.requesting.remove(&existing_prior_pos);
            }
        }

        // Order of iteration doesn't matter
        for (x, y, z) in iproduct!(-r..=r, -r..=r, -r..=r) {
            let offset_pos = *pos + IVec3::new(x, y, z);
            let dst_sqr = pos.distance_squared(offset_pos);
            if dst_sqr <= dst_sqr_max {
                // Use negative distance to make largest priority the closest
                // ones.
                requesting.requesting.push(offset_pos, -dst_sqr);
            }
        }

        let mut requested_count = 0;
        'batch_loop: while requested_count < requesting.batch_size {
            match requesting.requesting.pop() {
                Some((chunk_pos, _)) if !boring_map.entities().contains_key(&chunk_pos) => {
                    boring_map.request_chunk_gen_render(&mut commands, chunk_pos);
                    requested_count += 1;
                }
                Some(_) => {}
                None => break 'batch_loop,
            }
        }
    }
}
