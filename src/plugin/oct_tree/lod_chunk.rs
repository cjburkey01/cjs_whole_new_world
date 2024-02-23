use super::{LodPos, LodState};
use crate::voxel::Chunk;
use bevy::prelude::*;

pub struct LodChunk {
    pub entity: Entity,
    pub current_state: LodState,
    pub lod_data: Option<Chunk>,
}

#[derive(Component)]
pub struct LodChunkEntity(pub LodPos);
