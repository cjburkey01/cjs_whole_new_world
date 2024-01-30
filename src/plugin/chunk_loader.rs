use bevy::prelude::*;

#[derive(Default, Debug, Component, Copy, Clone, Eq, PartialEq)]
pub struct ChunkLoader {
    pub radius: u32,
}

impl ChunkLoader {
    pub fn new(radius: u32) -> Self {
        Self { radius }
    }
}
