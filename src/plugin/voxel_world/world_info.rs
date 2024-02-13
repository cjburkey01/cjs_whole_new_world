use bevy::prelude::*;

#[derive(Resource, Clone, Eq, PartialEq)]
pub struct WorldInfo {
    name: String,
    seed: u32,
}

impl WorldInfo {
    pub fn new(name: String, seed: u32) -> Self {
        Self { name, seed }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn seed(&self) -> u32 {
        self.seed
    }
}
