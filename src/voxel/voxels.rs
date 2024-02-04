use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Voxel {
    #[default]
    Air,
    Stone,
    Grass,
    Dirt,
}

impl Voxel {
    pub fn does_cull_as_solid(&self) -> bool {
        *self != Voxel::Air
    }

    pub fn atlas_index(&self) -> u32 {
        match *self {
            Voxel::Air => 0,
            Voxel::Stone => 1,
            Voxel::Grass => 0,
            Voxel::Dirt => 2,
        }
    }
}
