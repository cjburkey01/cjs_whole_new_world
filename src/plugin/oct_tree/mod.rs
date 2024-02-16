//! Implementation of a voxel oct-tree-esque structure to track which chunks
//! of which LOD level need to be loaded.

mod lod_chunk;
mod lod_pos;
mod lod_state;
mod lod_world;
mod oct_base;

pub use lod_chunk::*;
pub use lod_pos::*;
pub use lod_state::*;
pub use lod_world::*;
pub use oct_base::*;

use bevy::prelude::*;

pub struct OctLodPlugin;

impl Plugin for OctLodPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LodWorld>();
    }
}
