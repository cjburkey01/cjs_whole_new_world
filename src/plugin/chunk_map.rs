use crate::voxel::{world_noise::WorldNoiseSettings, Chunk, CHUNK_WIDTH};
use bevy::{ecs::query::ReadOnlyWorldQuery, pbr::wireframe::Wireframe, prelude::*, utils::HashMap};

pub struct ChunkMapPlugin;

impl Plugin for ChunkMapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkEntities>()
            .init_resource::<Chunks>();
    }
}

/// Keeps track of all chunk states in the world.
#[derive(Default, Resource)]
pub struct Chunks(pub HashMap<IVec3, Chunk>);

/// Keeps track of all chunk entities in the world.
#[derive(Default, Resource)]
pub struct ChunkEntities(pub HashMap<IVec3, Entity>);

#[derive(Debug, Component, Copy, Clone, Eq, PartialEq)]
pub struct GeneratedChunk(pub IVec3);

#[derive(Debug, Component, Copy, Clone, Eq, PartialEq)]
pub struct MeshedChunk(pub IVec3);

pub fn gen_chunk(
    commands: &mut Commands,
    world_noise: &WorldNoiseSettings,
    chunks: &mut Chunks,
    entities: &mut ChunkEntities,
    chunk_pos: IVec3,
) {
    // Make sure this chunk hasn't already been generated.
    if !chunks.0.contains_key(&chunk_pos) {
        let chunk = world_noise.build_heightmap_chunk(chunk_pos);
        chunks.0.insert(chunk_pos, chunk);
        let entity = commands
            .spawn((
                GeneratedChunk(chunk_pos),
                TransformBundle::from_transform(Transform::from_translation(
                    (chunk_pos * CHUNK_WIDTH as i32).as_vec3(),
                )),
            ))
            .id();
        entities.0.insert(chunk_pos, entity);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn mesh_chunk<F: ReadOnlyWorldQuery, M: Material>(
    commands: &mut Commands,
    chunks: &Chunks,
    entities: &ChunkEntities,
    chunk_pos: IVec3,
    meshes: &mut Assets<Mesh>,
    material: &Handle<M>,
    existing_mesh_chunks: &Query<&MeshedChunk, F>,
    wireframe: bool,
) {
    // Make sure this chunk is generated and doesn't already have a mesh
    if entities.0.contains_key(&chunk_pos)
        && existing_mesh_chunks.get(entities.0[&chunk_pos]).is_err()
    {
        let chunk = &chunks.0[&chunk_pos];
        let mesh = meshes.add(chunk.generate_mesh());
        let material = Handle::clone(material);
        let mut entity = commands.entity(entities.0[&chunk_pos]);
        let ent = entity.try_insert((
            MeshedChunk(chunk_pos),
            MaterialMeshBundle {
                mesh,
                material,
                transform: Transform::from_translation((chunk_pos * CHUNK_WIDTH as i32).as_vec3()),
                ..default()
            },
        ));
        if wireframe {
            ent.insert(Wireframe);
        }
    }
}
