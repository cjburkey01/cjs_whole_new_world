use super::chunk_map::*;
use crate::voxel::CHUNK_WIDTH;
use bevy::prelude::*;

pub struct ChunkLoadingPlugin;

impl Plugin for ChunkLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, on_update_translate_chunk_pos);
    }
}

#[derive(Default, Debug, Component, Copy, Clone, Eq, PartialEq)]
pub struct ChunkLoader {
    pub radius: u32,
}

impl ChunkLoader {
    pub fn new(radius: u32) -> Self {
        Self { radius }
    }
}

#[derive(Default, Debug, Component, Copy, Clone, Eq, PartialEq)]
pub struct ChunkPos {
    pub pos: IVec3,
}

impl ChunkPos {
    pub fn transform(&self) -> Transform {
        Transform::from_translation((self.pos * CHUNK_WIDTH as i32).as_vec3())
    }
}

#[allow(clippy::type_complexity)]
fn on_update_translate_chunk_pos(
    mut query: Query<
        (Entity, &Transform, &mut ChunkPos),
        Or<(Added<ChunkPos>, (Changed<Transform>, With<ChunkPos>))>,
    >,
) {
    let mut changed = vec![];
    for (entity, transform, chunk_pos) in query.iter() {
        let current_chunk_pos = (transform.translation / CHUNK_WIDTH as f32)
            .floor()
            .as_ivec3();
        if current_chunk_pos != chunk_pos.pos {
            changed.push((entity, current_chunk_pos));
        }
    }
    // I think this means we won't see anything change if the chunk hasn't changed.
    for (ent, new_chunk) in changed {
        if let Ok((_, _, mut old_chunk_pos)) = query.get_mut(ent) {
            old_chunk_pos.pos = new_chunk;
        }
    }
}
