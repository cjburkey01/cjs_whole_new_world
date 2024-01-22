use super::{
    InChunkPos, NeighborChunkSlices, SliceDirection, Voxel, VoxelContainer, CHUNK_CUBE,
    CHUNK_WIDTH, SLICE_DIRECTIONS,
};
use bevy::prelude::*;
use bitvec::prelude::BitVec;
use itertools::iproduct;

#[derive(Clone, Default)]
pub struct Chunk {
    pub(crate) voxels: VoxelContainer,
    /// Make sure you call the update method if the voxels change.
    pub(crate) edge_slice_bits: NeighborChunkSlices,
    pub definitely_empty: bool,
}

impl Chunk {
    #[allow(unused)]
    pub fn at(&self, pos: InChunkPos) -> Voxel {
        self.voxels.at(pos)
    }

    pub fn set(&mut self, pos: InChunkPos, voxel: Voxel) {
        self.voxels.set(pos, voxel);
        if voxel != Voxel::Air {
            self.definitely_empty = false;
        }
    }

    pub fn as_slice(&self) -> &[Voxel] {
        self.voxels.0.as_slice()
    }

    pub fn update_edge_slice_bits(&mut self) {
        for slice_dir in SLICE_DIRECTIONS {
            let new_bits = self.get_solid_bits_slice(slice_dir, 0).unwrap();
            let bits_at = self
                .edge_slice_bits
                .get_in_direction_mut(slice_dir.normal().negate());
            *bits_at = new_bits;
        }
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
