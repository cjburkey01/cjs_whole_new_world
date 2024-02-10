use crate::voxel::{InRegionChunkPos, VoxelContainer, REGION_CUBE};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Deserialize, Serialize)]
pub struct VoxelRegion {
    #[serde_as(as = "Box<[_; REGION_CUBE as usize]>")]
    chunks: Box<[Option<VoxelContainer>; REGION_CUBE as usize]>,
}

impl Default for VoxelRegion {
    fn default() -> Self {
        Self {
            chunks: Box::new(vec![None; REGION_CUBE as usize].try_into().unwrap()),
        }
    }
}

impl VoxelRegion {
    pub fn chunk(&self, pos: InRegionChunkPos) -> Option<&VoxelContainer> {
        self.chunks[pos.index()].as_ref()
    }

    pub fn chunk_mut(&mut self, pos: InRegionChunkPos) -> &mut Option<VoxelContainer> {
        &mut self.chunks[pos.index()]
    }

    #[allow(unused)]
    pub fn chunks(&self) -> &[Option<VoxelContainer>] {
        self.chunks.as_slice()
    }

    #[allow(unused)]
    pub fn chunks_mut(&mut self) -> &mut [Option<VoxelContainer>] {
        self.chunks.as_mut_slice()
    }
}
