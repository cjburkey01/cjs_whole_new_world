mod region_container;
mod region_handler;
mod region_maintainer;

pub use region_container::*;
pub use region_handler::*;
pub use region_maintainer::*;

pub const REGION_LOD_LEVEL: u8 = 4;
pub const REGION_WIDTH: u32 = 1 << REGION_LOD_LEVEL;
pub const REGION_SQUARE: u32 = REGION_WIDTH * REGION_WIDTH;
pub const REGION_CUBE: u32 = REGION_SQUARE * REGION_WIDTH;
