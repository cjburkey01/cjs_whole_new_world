use super::{InChunkPos, Voxel, CHUNK_CUBE};
use std::ops::{Deref, DerefMut};

pub struct VoxelContainer(pub Box<[Voxel; CHUNK_CUBE as usize]>);

impl Default for VoxelContainer {
    fn default() -> Self {
        Self(Box::new([Voxel::default(); CHUNK_CUBE as usize]))
    }
}

impl Clone for VoxelContainer {
    fn clone(&self) -> Self {
        Self(Box::new(*self.0.as_ref()))
    }
}

impl Deref for VoxelContainer {
    type Target = [Voxel];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

impl DerefMut for VoxelContainer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut_slice()
    }
}

impl VoxelContainer {
    #[allow(unused)]
    pub fn from_voxel(voxel: Voxel) -> Self {
        Self(Box::new([voxel; CHUNK_CUBE as usize]))
    }

    #[allow(unused)]
    pub fn from_voxels(voxels: Vec<Voxel>) -> Option<Self> {
        match voxels.len() as u32 {
            CHUNK_CUBE => Some(Self(voxels.try_into().ok()?)),
            _ => None,
        }
    }

    pub fn at(&self, pos: InChunkPos) -> Voxel {
        self.0[pos.index()]
    }

    pub fn set(&mut self, pos: InChunkPos, voxel: Voxel) {
        self.0[pos.index()] = voxel;
    }
}
