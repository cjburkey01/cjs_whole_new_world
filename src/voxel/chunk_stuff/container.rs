use crate::voxel::{InChunkPos, Voxel, CHUNK_CUBE};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::ops::{Deref, DerefMut, Index, IndexMut};

#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct VoxelContainerInner(
    #[serde_as(as = "Box<[_; CHUNK_CUBE as usize]>")] pub Box<[Voxel; CHUNK_CUBE as usize]>,
);

impl Default for VoxelContainerInner {
    fn default() -> Self {
        Self(Box::new([Voxel::default(); CHUNK_CUBE as usize]))
    }
}

impl Clone for VoxelContainerInner {
    fn clone(&self) -> Self {
        Self(Box::new(*self.0.as_ref()))
    }
}

impl Index<usize> for VoxelContainerInner {
    type Output = Voxel;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for VoxelContainerInner {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Deref for VoxelContainerInner {
    type Target = [Voxel];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

impl DerefMut for VoxelContainerInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut_slice()
    }
}

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct VoxelContainer(pub VoxelContainerInner);

impl VoxelContainer {
    #[allow(unused)]
    pub fn from_voxel(voxel: Voxel) -> Self {
        Self(VoxelContainerInner(Box::new([voxel; CHUNK_CUBE as usize])))
    }

    #[allow(unused)]
    pub fn from_voxels(voxels: Vec<Voxel>) -> Option<Self> {
        match voxels.len() as u32 {
            CHUNK_CUBE => Some(Self(VoxelContainerInner(voxels.try_into().ok()?))),
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
