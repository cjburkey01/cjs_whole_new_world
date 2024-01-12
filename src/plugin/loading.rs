use super::chunk_map::*;
use crate::voxel::{world_noise::WorldNoiseSettings, CHUNK_WIDTH};
use bevy::prelude::*;

pub struct ChunkLoadingPlugin;

impl Plugin for ChunkLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadingMap>()
            .add_systems(Update, on_loader_position_changed)
            .add_systems(Startup, add_chunk_material_system);
    }
}

#[derive(Resource)]
pub struct ChunkMaterialRes(pub Handle<StandardMaterial>);

fn add_chunk_material_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let handle = materials.add(asset_server.load::<Image>("textures/voxels.png").into());
    commands.insert_resource(ChunkMaterialRes(handle));
}

#[derive(Debug, Component, Clone, Eq, PartialEq)]
pub struct ChunkLoader {
    pub radius: u32,
    last_chunk: IVec3,
}

impl ChunkLoader {
    pub fn new(radius: u32) -> Self {
        Self {
            radius,
            last_chunk: IVec3::MAX,
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
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<ChunkMaterialRes>,
    world_noise: Res<WorldNoiseSettings>,
    mut chunks: ResMut<Chunks>,
    mut entities: ResMut<ChunkEntities>,
    mut changed: Query<
        (&Transform, &mut ChunkLoader),
        Or<(Changed<Transform>, Added<ChunkLoader>)>,
    >,
    existing_mesh_chunks: Query<&MeshedChunk, Without<ChunkLoader>>,
) {
    let w = CHUNK_WIDTH as i32;

    for (transform, mut loader) in changed.iter_mut() {
        let current_chunk_pos = (transform.translation / w as f32).floor().as_ivec3();
        if current_chunk_pos != loader.last_chunk {
            debug!(
                "chunk change to {:?} from {:?} at {:?}",
                current_chunk_pos, loader.last_chunk, transform.translation
            );
            loader.last_chunk = current_chunk_pos;

            // Build concentric rings
            for r in 0..=(loader.radius as i32) {
                for z in -r..=r {
                    for x in -r..=r {
                        for y in -2..=2 {
                            let chunk_pos = current_chunk_pos + IVec3::new(x, y, z);
                            // From `chunk_map`
                            gen_chunk(
                                &mut commands,
                                &world_noise,
                                &mut chunks,
                                &mut entities,
                                chunk_pos,
                            );
                            mesh_chunk(
                                &mut commands,
                                &chunks,
                                &entities,
                                chunk_pos,
                                &mut meshes,
                                &material.0,
                                &existing_mesh_chunks,
                                false,
                            );
                        }
                    }
                }
            }

            // TODO: UPDATE LOADING MAP
        }
    }
}
