use super::{LodChunk, LodNeededState, LodPos, LodState, OctTreeEsque};
use bevy::{prelude::*, utils::HashSet};
use itertools::iproduct;

#[derive(Default, Resource)]
pub struct LodWorld {
    pub tree: OctTreeEsque<LodChunk>,
}

#[allow(unused)]
impl LodWorld {
    /// Get the required chunks that need to be loaded for each lod level.
    ///
    /// For each level, starting at the highest lod level, add the necessary
    /// positions surrounding the loader position. Note: level thicknesses
    /// are given in the number of positions in the lod level *above* this
    /// one. For example, a level_half_thick for lod-0 of 3 would mean that
    /// 6 chunks would be loaded in each direction at lod-0 (13x13 square,
    /// rounded to 14x14), but the start and end chunks are snapped to the
    /// lod-1 chunk grid.
    /// The provided thicknesses also *include* the previous level lod
    /// chunks, so watch out for that.
    ///
    /// Do not edit this function without double checking the tests work :)
    /// This function outputs the expected values
    pub fn required_levels_for_loader(
        center_lod0_chunk: IVec3,
        level_half_thicks: &[u8],
    ) -> HashSet<LodPos> {
        // Output
        let mut needed = HashSet::new();

        let levels_count = level_half_thicks.len();
        let loader_lod0_pos = LodPos {
            level: 0,
            pos: center_lod0_chunk,
        };

        // Operate from highest lod to lowest
        for (level, half_rad) in level_half_thicks.iter().copied().enumerate().rev() {
            // Level above this level
            let next_level_center = loader_lod0_pos.to_level(level as u8 + 1);

            // Loop through all chunks within range
            for (x, y, z) in iproduct!(0..half_rad, 0..half_rad, 0..half_rad) {
                let offset = IVec3::new(x as i32, y as i32, z as i32);
                let next_level_pos = LodPos {
                    level: next_level_center.level,
                    pos: next_level_center.pos + offset,
                };

                // We know it has children positions because level must be >0
                for child in next_level_pos.children().unwrap() {
                    needed.insert(child);
                }

                // Remove the upper lod from its set so we don't load both
                // upper and lower lods
                if level < levels_count {
                    needed.remove(&next_level_pos);
                }
            }
        }

        needed
    }

    /// This function will determine a list of chunks whose states need to
    /// change based on which are required from the loader.
    pub fn changes_for_required_levels(
        &self,
        mut required_chunks: HashSet<LodPos>,
    ) -> Vec<(LodPos, LodNeededState)> {
        let mut required_changes = vec![];

        // Loop through all currently loaded chunks, removing them from the
        // required chunks set.
        for (level, chunks) in self.tree.levels().iter().enumerate() {
            for (pos, chunk) in chunks.iter() {
                let pos = LodPos {
                    level: level as u8,
                    pos: *pos,
                };

                // If this chunk is required, only add the state change if it
                // needs one.
                if required_chunks.remove(&pos) {
                    if chunk.current_state == LodState::None {
                        required_changes.push((pos, LodNeededState::Render))
                    }
                } else {
                    // If the chunk isn't required, we need to delete it.
                    required_changes.push((pos, LodNeededState::Deleted))
                }
            }
        }

        // We need to load all the remaining chunks
        for needed_pos in required_chunks.iter() {
            required_changes.push((*needed_pos, LodNeededState::Render))
        }

        required_changes
    }

    // Don't fuck around with this probably
    pub fn set_state_if_present(&mut self, pos: LodPos, state: LodState) {
        if let Some(lod_at) = self.tree.at_mut(pos) {
            lod_at.current_state = state;
        }
    }
}

// ugly tests but these are expected behaviors
#[cfg(test)]
mod test {
    use crate::oct_tree::LodNeededState;

    use super::{LodPos, LodWorld};
    use bevy::{math::IVec3, utils::HashSet};

    // whoop whoop!
    #[test]
    fn test_required_levels_for_loader_level_0() {
        let center_lod0_chunk = IVec3::ZERO;
        let level_half_thicks = [1];
        let found = LodWorld::required_levels_for_loader(center_lod0_chunk, &level_half_thicks);
        let expected = [
            (0, IVec3::ZERO),
            (0, IVec3::X),
            (0, IVec3::Y),
            (0, IVec3::X + IVec3::Y),
            (0, IVec3::Z),
            (0, IVec3::X + IVec3::Z),
            (0, IVec3::Y + IVec3::Z),
            (0, IVec3::X + IVec3::Y + IVec3::Z),
        ]
        .into_iter()
        .map(|(level, pos)| LodPos { level, pos })
        .collect::<HashSet<_>>();

        assert_eq!(expected, found);
    }

    // hell yeah
    #[test]
    fn test_required_levels_for_loader_2_levels() {
        let center_lod0_chunk = IVec3::ZERO;
        let level_half_thicks = [1, 1];
        let found = LodWorld::required_levels_for_loader(center_lod0_chunk, &level_half_thicks);
        let expected = [
            (0, IVec3::ZERO),
            (0, IVec3::X),
            (0, IVec3::Y),
            (0, IVec3::X + IVec3::Y),
            (0, IVec3::Z),
            (0, IVec3::X + IVec3::Z),
            (0, IVec3::Y + IVec3::Z),
            (0, IVec3::X + IVec3::Y + IVec3::Z),
            (1, IVec3::X),
            (1, IVec3::Y),
            (1, IVec3::X + IVec3::Y),
            (1, IVec3::Z),
            (1, IVec3::X + IVec3::Z),
            (1, IVec3::Y + IVec3::Z),
            (1, IVec3::X + IVec3::Y + IVec3::Z),
        ]
        .into_iter()
        .map(|(level, pos)| LodPos { level, pos })
        .collect::<HashSet<_>>();

        assert_eq!(expected, found);
    }

    #[test]
    fn test_changes_for_required_levels() {
        // Empty
        let lod_world = LodWorld::default();

        let required_chunks = [
            (0, IVec3::ZERO),
            (0, IVec3::X),
            (0, IVec3::Y),
            (0, IVec3::X + IVec3::Y),
            (0, IVec3::Z),
            (0, IVec3::X + IVec3::Z),
            (0, IVec3::Y + IVec3::Z),
            (0, IVec3::X + IVec3::Y + IVec3::Z),
        ]
        .into_iter()
        .map(|(level, pos)| LodPos { level, pos })
        .collect::<HashSet<_>>();

        let found_changes = lod_world.changes_for_required_levels(required_chunks);
        let expected_changes = [
            (0, IVec3::ZERO, LodNeededState::Render),
            (0, IVec3::X, LodNeededState::Render),
            (0, IVec3::Y, LodNeededState::Render),
            (0, IVec3::X + IVec3::Y, LodNeededState::Render),
            (0, IVec3::Z, LodNeededState::Render),
            (0, IVec3::X + IVec3::Z, LodNeededState::Render),
            (0, IVec3::Y + IVec3::Z, LodNeededState::Render),
            (0, IVec3::X + IVec3::Y + IVec3::Z, LodNeededState::Render),
        ]
        .into_iter()
        .map(|(level, pos, state)| (LodPos { level, pos }, state))
        .collect::<Vec<_>>();

        assert_eq!(expected_changes.len(), found_changes.len());
        for found in &found_changes {
            assert!(expected_changes.contains(found));
        }
    }
}
