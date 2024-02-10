mod axis;
mod biome;
mod chunk_stuff;
mod region;
mod voxels;
pub mod world_noise;

pub use axis::*;
pub use biome::*;
pub use chunk_stuff::{chunk::*, chunk_mesh::*, chunk_pos::*, container::*, neighbor_slice::*};
pub use region::*;
pub use voxels::*;

pub const CHUNK_WIDTH: u32 = 31;
pub const CHUNK_SQUARE: u32 = CHUNK_WIDTH * CHUNK_WIDTH;
pub const CHUNK_CUBE: u32 = CHUNK_SQUARE * CHUNK_WIDTH;
