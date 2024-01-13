use super::chunk_map::*;
use crate::voxel::CHUNK_WIDTH;
use bevy::prelude::*;

pub struct ChunkLoadingPlugin;

impl Plugin for ChunkLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadingMap>().add_systems(
            Update,
            (on_update_translate_chunk_pos, on_loader_position_changed),
        );
    }
}

#[derive(Default, Debug, Component, Clone, Eq, PartialEq)]
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

#[derive(Default, Resource)]
pub struct LoadingMap {}

// What do you mean, clippy?
// Eight (8) is a perfectly reasonable number of arguments.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn on_loader_position_changed(
    mut commands: Commands,
    mut chunks: ResMut<Chunks>,
    changed: Query<(&ChunkPos, &ChunkLoader), Or<(Added<ChunkLoader>, Changed<ChunkPos>)>>,
) {
    for (ChunkPos { pos: chunk_pos }, loader) in changed.iter() {
        debug!("loader changed to chunk {chunk_pos:?}");

        // Build concentric rings
        for r in 0..=(loader.radius as i32) {
            for z in -r..=r {
                for x in -r..=r {
                    for y in -r..=r {
                        let offset_chunk_pos = *chunk_pos + IVec3::new(x, y, z);
                        chunks.request_chunk_gen_render(&mut commands, offset_chunk_pos);
                    }
                }
            }
        }
    }
}
