mod region_container;
mod region_handler;

pub use region_container::*;
pub use region_handler::*;

pub const REGION_WIDTH: u32 = 16;
pub const REGION_SQUARE: u32 = REGION_WIDTH * REGION_WIDTH;
pub const REGION_CUBE: u32 = REGION_SQUARE * REGION_WIDTH;
