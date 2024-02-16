use bevy::{prelude::*, utils::HashSet};
use itertools::iproduct;

use super::{LodChunk, LodPos, OctTreeEsque};

#[derive(Default, Resource)]
pub struct LodWorld {
    pub tree: OctTreeEsque<LodChunk>,
}

#[allow(unused)]
impl LodWorld {
    pub fn needed_levels_for_loader(
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
