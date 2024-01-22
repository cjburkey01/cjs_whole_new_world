use super::VoxelAxis;
use bitvec::prelude::BitVec;

#[derive(Default, Clone)]
pub struct NeighborChunkSlices {
    pos_x: BitVec,
    pos_y: BitVec,
    pos_z: BitVec,
    neg_x: BitVec,
    neg_y: BitVec,
    neg_z: BitVec,
}

impl NeighborChunkSlices {
    pub fn get_in_direction(&self, direction: VoxelAxis) -> &BitVec {
        match direction {
            VoxelAxis::PosX => &self.pos_x,
            VoxelAxis::PosY => &self.pos_y,
            VoxelAxis::PosZ => &self.pos_z,
            VoxelAxis::NegX => &self.neg_x,
            VoxelAxis::NegY => &self.neg_y,
            VoxelAxis::NegZ => &self.neg_z,
        }
    }

    pub fn get_in_direction_mut(&mut self, direction: VoxelAxis) -> &mut BitVec {
        match direction {
            VoxelAxis::PosX => &mut self.pos_x,
            VoxelAxis::PosY => &mut self.pos_y,
            VoxelAxis::PosZ => &mut self.pos_z,
            VoxelAxis::NegX => &mut self.neg_x,
            VoxelAxis::NegY => &mut self.neg_y,
            VoxelAxis::NegZ => &mut self.neg_z,
        }
    }
}
