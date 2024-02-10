use super::{ChunkPos, InRegionChunkPos, RegionPos, VoxelContainer};
use crate::{io::read_region_from_file, plugin::beef::FixedChunkWorld};
use bevy::{
    prelude::*,
    utils::{hashbrown::hash_map::Iter, HashMap},
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

pub const REGION_WIDTH: u32 = 8;
pub const REGION_SQUARE: u32 = REGION_WIDTH * REGION_WIDTH;
pub const REGION_CUBE: u32 = REGION_SQUARE * REGION_WIDTH;

#[derive(Default)]
pub struct RegionHandler {
    regions: HashMap<RegionPos, VoxelRegion>,
}

impl RegionHandler {
    pub fn check_for_chunk(
        &mut self,
        world_name: &str,
        chunk_pos: ChunkPos,
    ) -> Option<&VoxelContainer> {
        let region_pos = chunk_pos.into();
        if self.region(region_pos).is_none() {
            let r = self.region_mut(region_pos);
            if let Some(region) = read_region_from_file(world_name, region_pos) {
                *r = region;
            }
        }

        self.chunk(chunk_pos)
    }

    pub fn extract_chunks(&mut self, chunk_world: &FixedChunkWorld) {
        for (pos, loaded_chunk) in chunk_world.chunks.iter() {
            if let Some(chunk) = &loaded_chunk.chunk {
                *self.chunk_mut(*pos) = Some(chunk.voxels.clone());
            }
        }
    }

    pub fn region(&self, region_pos: RegionPos) -> Option<&VoxelRegion> {
        self.regions.get(&region_pos)
    }

    pub fn get_region_mut(&mut self, region_pos: RegionPos) -> Option<&mut VoxelRegion> {
        self.regions.get_mut(&region_pos)
    }

    pub fn region_mut(&mut self, region_pos: RegionPos) -> &mut VoxelRegion {
        self.regions.entry(region_pos).or_default()
    }

    pub fn regions(&self) -> Iter<'_, RegionPos, VoxelRegion> {
        self.regions.iter()
    }

    pub fn chunk(&self, chunk_pos: ChunkPos) -> Option<&VoxelContainer> {
        let region_pos = chunk_pos.into();
        self.region(region_pos)
            .and_then(|region| region.chunk(InRegionChunkPos::from_world(chunk_pos)))
    }

    pub fn get_chunk_mut(&mut self, chunk_pos: ChunkPos) -> Option<&mut VoxelContainer> {
        let region_pos = chunk_pos.into();
        self.get_region_mut(region_pos)?
            .chunk_mut(InRegionChunkPos::from_world(chunk_pos))
            .as_mut()
    }

    pub fn chunk_mut(&mut self, chunk_pos: ChunkPos) -> &mut Option<VoxelContainer> {
        let region_pos = chunk_pos.into();
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
