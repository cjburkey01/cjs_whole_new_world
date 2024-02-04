use crate::{
    io::write_chunk_to_file,
    plugin::beef::{ChunkEntity, FixedChunkWorld},
};
use bevy::{prelude::*, time::common_conditions::on_timer};
use std::time::Duration;

pub struct SaverPlugin;

impl Plugin for SaverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            save_io_dirty_chunks
                .run_if(resource_exists::<FixedChunkWorld>())
                .run_if(on_timer(Duration::from_secs(10))),
        );
    }
}

#[derive(Component)]
pub struct IoCleanChunk;

fn save_io_dirty_chunks(
    mut commands: Commands,
    chunk_world: Res<FixedChunkWorld>,
    dirty: Query<(Entity, &ChunkEntity), Without<IoCleanChunk>>,
) {
    for (entity, ChunkEntity(pos)) in dirty.iter() {
        if let Some(mut entity) = commands.get_entity(entity) {
            entity.insert(IoCleanChunk);
        }
        if let Some(chunk) = &chunk_world.chunks.get(pos).unwrap().chunk {
            write_chunk_to_file(chunk_world.name(), *pos, &chunk.voxels);
        }
    }
}
