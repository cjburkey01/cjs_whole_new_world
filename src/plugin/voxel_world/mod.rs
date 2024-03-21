pub mod beef;
pub mod chunk_loader;
pub mod chunk_pos_update;
pub mod lod_chunk_material;
pub mod region_saver;
pub mod voxel_material;
pub mod world_info;
pub mod world_state;

use bevy::prelude::*;

pub struct VoxelWorldPlugin;

impl Plugin for VoxelWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            //beef::BeefPlugin,
            world_state::WorldStatePlugin,
            chunk_pos_update::ChunkPosPlugin,
            voxel_material::VoxelMaterialPlugin,
            lod_chunk_material::LodChunkMaterialPlugin,
            //region_saver::RegionSaverPlugin,
        ));
    }
}
