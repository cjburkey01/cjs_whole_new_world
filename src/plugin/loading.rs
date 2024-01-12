use crate::voxel::{world_noise::WorldNoiseSettings, CHUNK_WIDTH};
use bevy::{pbr::wireframe::Wireframe, prelude::*};

pub struct ChunkLoadingPlugin;

impl Plugin for ChunkLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadingMap>()
            .add_systems(Update, on_loader_position_changed);
    }
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
            last_chunk: IVec3::ZERO,
        }
    }
}

#[derive(Default, Resource)]
pub struct LoadingMap {}

fn on_loader_position_changed(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    world_noise: Res<WorldNoiseSettings>,
    mut changed: Query<(&Transform, &mut ChunkLoader), Changed<Transform>>,
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
            // TODO: UPDATE LOADING MAP

            let chunk_material = materials.add(Color::WHITE.into());
            let chunk = world_noise.build_heightmap_chunk(current_chunk_pos);
            let chunk_mesh = meshes.add(chunk.generate_mesh());
            commands.spawn((
                MaterialMeshBundle {
                    mesh: chunk_mesh,
                    material: chunk_material,
                    transform: Transform::from_translation((current_chunk_pos * w).as_vec3()),
                    ..default()
                },
                Wireframe,
            ));
        }
    }
}
