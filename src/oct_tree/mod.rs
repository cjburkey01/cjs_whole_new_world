//! Implementation of a voxel oct-tree-esque structure to track which chunks
//! of which LOD level need to be loaded.

use crate::{voxel::Chunk, voxel_world::beef::ChunkState};
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use itertools::iproduct;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct LodPos {
    pub level: u8,
    pub pos: IVec3,
}

#[allow(unused)]
impl LodPos {
    pub fn to_level(&self, level: u8) -> Self {
        let diff = level as i16 - self.level as i16;
        let diff_pow = 1 << diff.abs() as usize;
        match diff {
            // No change
            0 => *self,
            // Increase in level decreases position
            d if d > 0 => Self {
                level,
                pos: self.pos.div_euclid(IVec3::splat(diff_pow)),
            },
            // Decrease in level increases position
            _ => Self {
                level,
                pos: self.pos * diff_pow,
            },
        }
    }

    pub fn parent(&self) -> Self {
        Self {
            level: self.level + 1,
            pos: self.pos.div_euclid(IVec3::splat(2)),
        }
    }

    pub fn start_child(&self) -> Option<Self> {
        self.level.checked_sub(1).map(|level| Self {
            level,
            pos: self.pos * 2,
        })
    }

    pub fn children(&self) -> Option<[LodPos; 8]> {
        self.start_child().map(|start_pos| {
            [
                start_pos,
                LodPos {
                    level: start_pos.level,
                    pos: start_pos.pos + IVec3::X,
                },
                LodPos {
                    level: start_pos.level,
                    pos: start_pos.pos + IVec3::Y,
                },
                LodPos {
                    level: start_pos.level,
                    pos: start_pos.pos + IVec3::Y + IVec3::X,
                },
                LodPos {
                    level: start_pos.level,
                    pos: start_pos.pos + IVec3::Z,
                },
                LodPos {
                    level: start_pos.level,
                    pos: start_pos.pos + IVec3::Z + IVec3::X,
                },
                LodPos {
                    level: start_pos.level,
                    pos: start_pos.pos + IVec3::Z + IVec3::Y,
                },
                LodPos {
                    level: start_pos.level,
                    pos: start_pos.pos + IVec3::Z + IVec3::Y + IVec3::X,
                },
            ]
        })
    }
}

#[derive(Default)]
pub struct LodChunk {
    pub state: ChunkState,
    pub chunk_data: Option<Chunk>,
}

pub struct OctTreeEsque<T> {
    levels: Vec<HashMap<IVec3, T>>,
}

// Deriving Default on the struct that contains this tree requires T to
// implement Default if we derive Default for the tree struct, so we implement
// it manually.
impl<T> Default for OctTreeEsque<T> {
    fn default() -> Self {
        Self { levels: vec![] }
    }
}

#[allow(unused)]
impl<T> OctTreeEsque<T> {
    pub fn level(&self, level: u8) -> Option<&HashMap<IVec3, T>> {
        self.levels.get(level as usize)
    }

    pub fn level_mut(&mut self, level: u8) -> &mut HashMap<IVec3, T> {
        let index = level as usize;
        if index >= self.levels.len() {
            let needed_levels = index - self.levels.len() + 1;
            for _ in 0..needed_levels {
                self.levels.push(default());
            }
        }
        &mut self.levels[index]
    }

    pub fn at(&self, pos: LodPos) -> Option<&T> {
        self.level(pos.level).and_then(|map| map.get(&pos.pos))
    }

    pub fn at_mut(&mut self, pos: LodPos) -> Option<&mut T> {
        self.level_mut(pos.level).get_mut(&pos.pos)
    }
}

#[derive(Default, Resource)]
pub struct LodWorld {
    pub tree: OctTreeEsque<LodChunk>,
}

#[allow(unused)]
impl LodWorld {
    pub fn needed_levels(
        center_lod0_chunk: IVec3,
        level_half_thicks: &[u8],
    ) -> Vec<HashSet<LodPos>> {
        let mut needed: Vec<HashSet<LodPos>> = vec![];

        // For each level, starting at the highest lod level, add the necessary
        // positions surrounding the loader position. Note: level thicknesses
        // are given in the number of positions in the lod level *above* this
        // one. For example, a level_half_thick for lod-0 of 3 would mean that
        // 6 chunks would be loaded in each direction at lod-0 (13x13 square,
        // rounded to 14x14), but the start and end chunks are snapped to the
        // lod-1 chunk grid.
        // The provided thicknesses also *include* the previous level lod
        // chunks, so watch out for that.

        let levels_count = level_half_thicks.len();
        let loader_lod0_pos = LodPos {
            level: 0,
            pos: center_lod0_chunk,
        };

        // Operate from highest lod to lowest
        for (level, half_rad) in level_half_thicks.iter().copied().enumerate().rev() {
            // Level above this level
            let next_level_center = loader_lod0_pos.to_level(level as u8 + 1);
            let mut this_level = HashSet::new();

            for (x, y, z) in iproduct!(0..half_rad, 0..half_rad, 0..half_rad) {
                let offset = IVec3::new(x as i32, y as i32, z as i32);
                let next_level_pos = LodPos {
                    level: next_level_center.level,
                    pos: next_level_center.pos + offset,
                };

                // We know it has children positions because level must be >0
                for child in next_level_pos.children().unwrap() {
                    this_level.insert(child);
                }

                // Remove the upper lod from its set so we don't load both
                // upper and lower lods
                if level < levels_count {
                    if let Some(a) = needed.get_mut(levels_count - level - 1) {
                        a.remove(&next_level_pos);
                    }
                }
            }

            needed.push(this_level);
        }

        needed
    }
}
