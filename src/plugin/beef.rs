use crate::voxel::Chunk;
use bevy::{prelude::*, utils::HashMap};
use itertools::iproduct;

#[derive(Default)]
struct LoadedChunk {
    chunk: Option<Chunk>,
    state: ChunkState,
    needed_state: NeededChunkState,
}

#[derive(Default)]
pub struct FixedChunkMap {
    chunks: HashMap<IVec3, LoadedChunk>,
}

impl FixedChunkMap {
    fn set_needed(&mut self, chunk: IVec3, needed_state: NeededChunkState) {
        let _ = self
            .chunks
            .entry(chunk)
            .and_modify(|c| c.needed_state = needed_state)
            .or_insert_with(|| LoadedChunk {
                needed_state,
                ..default()
            });
    }

    pub fn update_loader_position(&mut self, loader_chunk: IVec3, radius: usize) {
        let r = radius as i32;

        for (x, y, z) in iproduct!(-r..=r, -r..=r, -r..=r) {
            let offset = IVec3::new(x, y, z);
            // We don't render the outermost layer of chunks, they are only
            // generated to allow for chunk side culling.
            let needed_state = match x.abs() == r || y.abs() == r || z.abs() == r {
                true => NeededChunkState::Generated,
                false => NeededChunkState::Rendered,
            };
            self.set_needed(loader_chunk + offset, needed_state);
        }
    }

    fn required_state_changes(&self) -> Vec<(IVec3, NeededStateChange)> {
        let mut changes = vec![];

        for (pos, chunk) in self.chunks.iter() {
            match chunk.needed_state {
                NeededChunkState::DontNeed => {}
                NeededChunkState::Generated => match chunk.state {
                    ChunkState::Empty => changes.push((*pos, NeededStateChange::Generate)),
                    // Intentionally not using `_` in case I add new chunk
                    // states for whatever cursed reason.
                    ChunkState::GeneratingOnly => {}
                    ChunkState::GeneratingAndRendering => {}
                    ChunkState::GeneratedNotRendered => {}
                    ChunkState::Rendering => {}
                    ChunkState::Rendered => {}
                },
                NeededChunkState::Rendered => match chunk.state {
                    ChunkState::Empty => changes.push((*pos, NeededStateChange::Render)),
                    ChunkState::GeneratingOnly => {}
                    ChunkState::GeneratingAndRendering => {}
                    ChunkState::GeneratedNotRendered => {
                        changes.push((*pos, NeededStateChange::Render))
                    }
                    ChunkState::Rendering => {}
                    ChunkState::Rendered => {}
                },
            }
        }

        changes
    }
}

enum NeededStateChange {
    Generate,
    Render,
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
enum ChunkState {
    #[default]
    Empty,
    GeneratingOnly,
    GeneratingAndRendering,
    GeneratedNotRendered,
    Rendering,
    Rendered,
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
enum NeededChunkState {
    #[default]
    DontNeed,
    Generated,
    Rendered,
}
