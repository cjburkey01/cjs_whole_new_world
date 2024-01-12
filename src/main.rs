mod plugin;
mod voxel;

use crate::voxel::{InChunkPos, Voxel};
use bevy::{
    log::{Level, LogPlugin},
    pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin},
    prelude::{shape::Cube, *},
    render::{
        render_resource::WgpuFeatures,
        settings::{RenderCreation, WgpuSettings},
        RenderPlugin,
    },
};
use leafwing_input_manager::prelude::*;
use plugin::*;
use std::f32::consts::PI;

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    filter: "wgpu=warn,naga=warn,bevy_render=info,bevy_app::plugin_group=info"
                        .to_string(),
                    level: Level::DEBUG,
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: format!("{PKG_NAME} v{PKG_VERSION}"),
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        // WARN this is a native only feature.
                        // It will not work with webgl or webgpu.
                        features: WgpuFeatures::POLYGON_MODE_LINE,
                        ..default()
                    }),
                }),
        )
        .add_plugins(WireframePlugin)
        .add_plugins((
            InputManagerPlugin::<input::PlyAction>::default(),
            control::PlyControlPlugin,
        ))
        .insert_resource(ClearColor(Color::rgb(0.5, 0.5, 0.8)))
        .insert_resource(AmbientLight {
            brightness: 0.3,
            ..default()
        })
        .insert_resource(WireframeConfig {
            // The global wireframe config enables drawing of wireframes on every mesh,
            // except those with `NoWireframe`. Meshes with `Wireframe` will always have a wireframe,
            // regardless of the global configuration.
            global: false,
            // Controls the default color of all wireframes. Used as the default color for global wireframes.
            // Can be changed per mesh using the `WireframeColor` component.
            default_color: Color::WHITE,
        })
        .add_systems(Startup, init_world)
        .run();
}

fn init_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn(control::PlyCamBundle {
        camera: Camera3dBundle {
            projection: Projection::Perspective(PerspectiveProjection {
                fov: 80.0 * PI / 180.0,
                near: 0.01,
                far: 1000.0,
                ..default()
            }),
            transform: Transform::from_translation(Vec3::new(0.0, 10.0, 15.0)),
            ..default()
        },
        ..default()
    });

    let cube_mesh = meshes.add(Cube::new(0.8).into());

    // Center cubes
    commands.spawn(MaterialMeshBundle {
        mesh: cube_mesh.clone(),
        material: materials.add(Color::WHITE.into()),
        transform: Transform::from_translation(Vec3::ZERO),
        ..default()
    });
    commands.spawn(MaterialMeshBundle {
        mesh: cube_mesh.clone(),
        material: materials.add(Color::RED.into()),
        transform: Transform::from_translation(Vec3::X),
        ..default()
    });
    commands.spawn(MaterialMeshBundle {
        mesh: cube_mesh.clone(),
        material: materials.add(Color::GREEN.into()),
        transform: Transform::from_translation(Vec3::Y),
        ..default()
    });
    commands.spawn(MaterialMeshBundle {
        mesh: cube_mesh.clone(),
        material: materials.add(Color::BLUE.into()),
        transform: Transform::from_translation(Vec3::Z),
        ..default()
    });

    let chunk_material = materials.add(Color::WHITE.into());

    // Voxel test
    let mut chunk = voxel::Chunk::from_voxel(Voxel::Stone);
    chunk.set(InChunkPos::new(UVec3::ZERO).unwrap(), Voxel::Air);
    chunk.set(InChunkPos::new(UVec3::new(15, 0, 0)).unwrap(), Voxel::Air);

    let chunk_mesh = meshes.add(chunk.generate_mesh());
    commands.spawn((
        MaterialMeshBundle {
            mesh: chunk_mesh,
            material: chunk_material,
            ..default()
        },
        Wireframe,
    ));
}
