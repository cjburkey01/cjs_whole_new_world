#![feature(const_option)]

mod plugin;
mod voxel;

use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;
use plugin::*;
use std::f32::consts::PI;
use voxel::{world_noise::WorldNoiseSettings, BiomeTable};

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    filter: "wgpu=warn,naga=warn,bevy_render=info,bevy_app::plugin_group=info,bevy_ecs::world::entity_ref=info"
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
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins((
            InputManagerPlugin::<control::input::PlyAction>::default(),
            control::PlyControlPlugin,
            voxel_material::VoxelMaterialPlugin,
            loading::ChunkLoadingPlugin,
            chunk_map::ChunkMapPlugin,
            better_chunk_map::Plugin3000,
        ))
        .insert_resource(ClearColor(Color::rgb(0.5, 0.5, 0.8)))
        .insert_resource(AmbientLight {
            brightness: 0.3,
            ..default()
        })
        .insert_resource(WorldNoiseSettings::new(42069, BiomeTable::new()))
        .add_systems(Startup, init_world)
        .run();
}

fn init_world(mut commands: Commands) {
    // Lights
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::new(2.0, 3.0, 4.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        directional_light: DirectionalLight {
            color: Color::BISQUE,
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });

    // Camera
    commands.spawn((
        control::PlyCamBundle {
            camera: Camera3dBundle {
                projection: Projection::Perspective(PerspectiveProjection {
                    fov: 80.0 * PI / 180.0,
                    near: 0.01,
                    far: 1000.0,
                    ..default()
                }),
                transform: Transform::from_xyz(0.0, 30.0, 15.0),
                ..default()
            },
            ..default()
        },
        loading::ChunkLoader::new(6),
        loading::ChunkPos::default(),
    ));

    // Action

    // :)
}
