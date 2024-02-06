use super::{
    InChunkPos, NeighborChunkSlices, SliceDirection, Voxel, VoxelContainer, CHUNK_CUBE,
    CHUNK_WIDTH, SLICE_DIRECTIONS,
};
use bevy::prelude::*;
use bitvec::prelude::BitVec;
use itertools::iproduct;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub(crate) voxels: VoxelContainer,
    pub definitely_empty: bool,
    /// Make sure you call the update method if the voxels change.
    pub(crate) edge_slice_bits: NeighborChunkSlices,
    #[serde(skip)]
    pub edges_dirty: bool,
}

impl Chunk {
    pub fn from_container(voxels: VoxelContainer) -> Self {
        let mut this_chunk = Self {
            voxels,
            ..default()
        };
        this_chunk.update_edge_slice_bits();
        this_chunk
    }

    #[allow(unused)]
    pub fn at(&self, pos: InChunkPos) -> Voxel {
        self.voxels.at(pos)
    }

    pub fn set(&mut self, pos: InChunkPos, voxel: Voxel) {
        self.voxels.set(pos, voxel);
        if voxel != Voxel::Air {
            self.definitely_empty = false;
        }
        if !self.edges_dirty && (pos.min_element() == 0 || pos.max_element() == CHUNK_WIDTH - 1) {
            self.edges_dirty = true;
        }
    }

    pub fn as_slice(&self) -> &[Voxel] {
        &self.voxels.0
    }

    pub fn update_edge_slice_bits(&mut self) {
        for slice_dir in SLICE_DIRECTIONS {
            let new_bits = self.get_solid_bits_slice(slice_dir, 0).unwrap();
            let bits_at = self
                .edge_slice_bits
                .get_in_direction_mut(slice_dir.normal().negate());
            *bits_at = new_bits;
        }
        self.edges_dirty = false;
    }

    // TODO: THIS IS A VERY HOT FUNCTION!
    //       IT'S TAKING ABOUT 33% OF ALL TIME!
    pub fn get_solid_bits_slice(
        &self,
        slice_direction: SliceDirection,
        slice_depth: u32,
    ) -> Option<BitVec> {
        let mut bit_slice = BitVec::repeat(false, CHUNK_CUBE as usize);
        for (y, x) in iproduct!(0..CHUNK_WIDTH, 0..CHUNK_WIDTH) {
            let voxel = self.voxels.at(InChunkPos::new(
                slice_direction.transform(slice_depth, UVec2::new(x, y))?,
            )?);
            let slice_index = y * CHUNK_WIDTH + x;
            bit_slice.set(slice_index as usize, voxel.does_cull_as_solid());
        }
        Some(bit_slice)
    }
}
