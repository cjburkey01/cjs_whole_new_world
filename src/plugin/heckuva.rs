//! Hell's latest visitor.
//!
//! How does this need to work?
//! * We need to be able to account for potential chunk *updates* as well as
//!   generation.
//! * Once a chunk entity has been spawned, we should keep track of its status
//!   **even if it has been unloaded**. No more despawning chunk entities, only
//!   removing the relevant components if a chunk is unloaded.
//! * We should keep a priority queue with the priority as the distance from the
//!   nearest chunk loader. Chunks needing an update need to take precedence
//!   over chunks loading when generating meshes.
//!   * Chunks need to keep track of their dirty status, which determines if a
//!     re-render is needed.
//!   * We should keep a maximum number of render tasks that can spawn at the
//!     same time as well as a maximum proportion of those that may be chunk
//!     updates, to allow some amount of chunk loading to happen even if chunks
//!     need a lot of updates.
//!

use super::loading::{ChunkLoader, ChunkPos};
use bevy::{prelude::*, utils::HashMap};
use itertools::iproduct;
use priority_queue::PriorityQueue;

struct SecretSanta {
    // Priority is negative distance so closest is highest priority
    chunk_priorities: PriorityQueue<IVec3, i32>,
    chunk_entities: HashMap<IVec3, Entity>,
}

impl SecretSanta {
    // We only need to call this when a chunk loader's position changes
    fn update_chunk_priorities(
        &mut self,
        commands: &mut Commands,
        loaders: &[(IVec3, ChunkLoader)],
    ) {
        // Clear out all the chunk priorities to determine which chunks still
        // need to be loaded.
        self.chunk_priorities.clear();

        for (loader_pos, ChunkLoader { radius }) in loaders {
            let radius = (*radius) as i32;
            let loader_pos = *loader_pos;

            for (x, y, z) in iproduct!(-radius..=radius, -radius..=radius, -radius..=radius) {
                let pos = loader_pos + IVec3::new(x, y, z);
                let dist = pos.distance_squared(loader_pos);
                let this_priority = -dist;

                // Spawn this chunk entity if it doesn't exist yet.
                let _ = self
                    .chunk_entities
                    .entry(pos)
                    .or_insert_with(|| commands.spawn(ChunkPos { pos }).id());

                // Add this chunk with its priority, overwriting the previous
                // priority if the chunk is closer to this loader
                match self.chunk_priorities.get(&pos) {
                    // I have to write these patterns separately because
                    // `priority` is not bound in the `None` pattern.
                    None => {
                        self.chunk_priorities.push(pos, this_priority);
                    }
                    Some((_, priority)) if this_priority > *priority => {
                        self.chunk_priorities.push(pos, this_priority);
                    }
                    _ => {}
                }
            }
        }
    }
}
