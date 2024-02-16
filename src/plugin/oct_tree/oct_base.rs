use super::LodPos;
use bevy::{prelude::*, utils::HashMap};

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
