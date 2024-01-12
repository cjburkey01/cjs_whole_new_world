mod plugin;
mod voxel;

use bevy::{
    log::{Level, LogPlugin},
    prelude::{shape::Cube, *},
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
                }),
        )
        .add_plugins((
            InputManagerPlugin::<input::PlyAction>::default(),
            control::PlyControlPlugin,
        ))
        .insert_resource(ClearColor(Color::rgb(0.5, 0.5, 0.8)))
        .insert_resource(AmbientLight {
            brightness: 0.3,
            ..default()
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
}
