use crate::{
    io::write_chunk_to_file,
    plugin::beef::{ChunkEntity, FixedChunkWorld},
};
use bevy::{prelude::*, tasks::AsyncComputeTaskPool, time::common_conditions::on_timer};
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

// TODO: SAVE REGIONS INSTEAD OF CHUNKS
//       JUST SAVE THE WHOLE WORLD EVERY ONCE IN A WHILE
fn save_io_dirty_chunks(
    mut commands: Commands,
    chunk_world: Res<FixedChunkWorld>,
    dirty: Query<(Entity, &ChunkEntity), Without<IoCleanChunk>>,
) {
    let pool = AsyncComputeTaskPool::get();

    for (entity, ChunkEntity(pos)) in dirty.iter() {
        if let Some(mut entity) = commands.get_entity(entity) {
            entity.insert(IoCleanChunk);
        }
        if let Some(chunk) = chunk_world.regions.chunk(*pos) {
            let cloned_chunk = chunk.clone();
            let name = chunk_world.name().to_string();
            let pos = *pos;
            pool.spawn(async move { write_chunk_to_file(&name, pos, &cloned_chunk) })
                .detach();
        }
    }
}
