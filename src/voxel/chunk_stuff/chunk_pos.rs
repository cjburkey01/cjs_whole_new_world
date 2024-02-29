use crate::voxel::{CHUNK_SQUARE, CHUNK_WIDTH, REGION_SQUARE, REGION_WIDTH};
use bevy::{
    math::UVec3,
    prelude::{Component, IVec3, Transform},
};
use std::ops::Deref;

pub struct VoxelPos(pub IVec3);

/// Chunk position within the world.
#[derive(Default, Debug, Component, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ChunkPos(pub IVec3);

impl From<ChunkPos> for VoxelPos {
    fn from(value: ChunkPos) -> Self {
        Self(value.0 * CHUNK_WIDTH as i32)
    }
}

impl From<VoxelPos> for ChunkPos {
    fn from(value: VoxelPos) -> Self {
        Self(value.0.div_euclid(UVec3::splat(CHUNK_WIDTH).as_ivec3()))
    }
}

impl ChunkPos {
    pub fn transform(&self) -> Transform {
        Transform::from_translation((self.0 * CHUNK_WIDTH as i32).as_vec3())
    }
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct RegionPos(pub IVec3);

impl From<RegionPos> for ChunkPos {
    fn from(value: RegionPos) -> Self {
        Self(value.0 * REGION_WIDTH as i32)
    }
}

impl From<ChunkPos> for RegionPos {
    fn from(value: ChunkPos) -> Self {
        Self(value.0.div_euclid(UVec3::splat(REGION_WIDTH).as_ivec3()))
    }
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct InChunkPos(UVec3);

impl InChunkPos {
    pub fn new(pos: UVec3) -> Option<Self> {
        match pos.max_element() < CHUNK_WIDTH {
            true => Some(Self(pos)),
            false => None,
        }
    }

    pub fn from_urem(pos: UVec3) -> Self {
        Self(pos % CHUNK_WIDTH)
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct InRegionChunkPos(UVec3);

impl InRegionChunkPos {
    #[allow(unused)]
    pub fn new(pos: UVec3) -> Option<Self> {
        match pos.max_element() < REGION_WIDTH {
            true => Some(Self(pos)),
            false => None,
        }
    }

    pub fn from_world(world_chunk_pos: ChunkPos) -> Self {
        Self(
            world_chunk_pos
                .0
                .rem_euclid(UVec3::splat(REGION_WIDTH).as_ivec3())
                .as_uvec3(),
        )
    }

    #[allow(unused)]
    pub fn pos(&self) -> UVec3 {
        self.0
    }

    pub fn index(&self) -> usize {
        let UVec3 { x, y, z } = self.0;
        (REGION_SQUARE * z + REGION_WIDTH * y + x) as usize
    }
}
