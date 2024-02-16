use crate::voxel::Chunk;

use super::LodState;

#[derive(Default)]
pub struct LodChunk {
    pub current_state: LodState,
    pub needed_state: LodState,
    pub lod_data: Option<Chunk>,
}
