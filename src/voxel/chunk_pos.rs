use crate::voxel::{CHUNK_SQUARE, CHUNK_WIDTH};
use bevy::math::UVec3;
use std::ops::Deref;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct InChunkPos(UVec3);

impl InChunkPos {
    pub fn new(pos: UVec3) -> Option<Self> {
        match pos.max_element() < CHUNK_WIDTH {
            true => Some(Self(pos)),
            false => None,
        }
    }

    pub fn pos(&self) -> UVec3 {
        self.0
    }

    pub fn index(&self) -> usize {
        (CHUNK_SQUARE * self.z + CHUNK_WIDTH * self.y + self.x) as usize
    }
}

impl Deref for InChunkPos {
    type Target = UVec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
