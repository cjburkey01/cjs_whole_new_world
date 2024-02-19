use crate::voxel::Chunk;

use super::{LodNeededState, LodState};

#[derive(Default)]
pub struct LodChunk {
    pub current_state: LodState,
    pub needed_state: LodNeededState,
    pub lod_data: Option<Chunk>,
}
