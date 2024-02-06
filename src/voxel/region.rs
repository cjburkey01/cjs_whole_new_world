use super::{Chunk, VoxelContainer};
use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

pub const REGION_WIDTH: u32 = 8;
pub const REGION_SQUARE: u32 = REGION_WIDTH * REGION_WIDTH;
pub const REGION_CUBE: u32 = REGION_SQUARE * REGION_WIDTH;

#[derive(Default)]
pub struct RegionHandler {
    regions: HashMap<IVec3, VoxelRegion>,
}

impl RegionHandler {
    pub fn region(&self, region_pos: IVec3) -> Option<&VoxelRegion> {
        self.regions.get(&region_pos)
    }

    pub fn get_region_mut(&mut self, region_pos: IVec3) -> Option<&mut VoxelRegion> {
        self.regions.get_mut(&region_pos)
    }

    pub fn region_mut(&mut self, region_pos: IVec3) -> &mut VoxelRegion {
        self.regions
            .entry(region_pos)
            .or_insert_with(|| VoxelRegion::default())
    }

    pub fn chunk(&self, chunk_pos: IVec3) -> Option<&VoxelContainer> {
        let region_pos = chunk_pos / REGION_WIDTH as i32;
        self.region(region_pos)
            .and_then(|region| region.chunk(InRegionChunkPos::from_world(chunk_pos)))
    }

    pub fn get_chunk_mut(&mut self, chunk_pos: IVec3) -> Option<&mut VoxelContainer> {
        let region_pos = chunk_pos / REGION_WIDTH as i32;
        self.get_region_mut(region_pos)?
            .chunk_mut(InRegionChunkPos::from_world(chunk_pos))
            .as_mut()
    }

    pub fn chunk_mut(&mut self, chunk_pos: IVec3) -> &mut Option<VoxelContainer> {
        let region_pos = chunk_pos / REGION_WIDTH as i32;
        self.region_mut(region_pos)
            .chunk_mut(InRegionChunkPos::from_world(chunk_pos))
    }
}

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

    pub fn chunks(&self) -> &[Option<VoxelContainer>] {
        self.chunks.as_slice()
    }

    pub fn chunks_mut(&mut self) -> &mut [Option<VoxelContainer>] {
        self.chunks.as_mut_slice()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct InRegionChunkPos(UVec3);

impl InRegionChunkPos {
    pub fn new(pos: UVec3) -> Option<Self> {
        match pos.max_element() < REGION_WIDTH {
            true => Some(Self(pos)),
            false => None,
        }
    }

    pub fn from_world(world_chunk_pos: IVec3) -> Self {
        Self(
            world_chunk_pos
                .rem_euclid(UVec3::splat(REGION_WIDTH).as_ivec3())
                .as_uvec3(),
        )
    }

    pub fn pos(&self) -> UVec3 {
        self.0
    }

    pub fn index(&self) -> usize {
        let UVec3 { x, y, z } = self.0;
        (x * REGION_SQUARE + y * REGION_WIDTH + z) as usize
    }
}
