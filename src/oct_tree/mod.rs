//! Implementation of a voxel oct-tree-esque structure to track which chunks
//! of which LOD level need to be loaded.

use crate::voxel::Chunk;
use bevy::{prelude::*, utils::HashMap};

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct LodPos {
    pub level: u8,
    pub pos: IVec3,
}

impl LodPos {
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
}

pub struct LodChunk {
    pub chunk_data: Chunk,
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
