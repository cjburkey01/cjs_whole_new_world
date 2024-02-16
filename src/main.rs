#![feature(const_option)]

mod io;
mod plugin;
mod voxel;

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    log::{Level, LogPlugin},
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
};
use bevy_rapier3d::prelude::*;
use control::{input::PlyAction, PrimaryCamera};
use leafwing_input_manager::prelude::*;
use plugin::{control::controller_2, *};

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
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(InputManagerPlugin::<PlyAction>::default())
        .add_plugins((
            asset::CwnwAssetPlugin,
            game_settings::GameSettingsPlugin,
            control::PlyControlPlugin,
            controller_2::Controller2ElectricBoogalooPlugin,
            game_gui::GameGuiPlugin,
            voxel_world::VoxelWorldPlugin,
        ))
        .insert_resource(ClearColor(Color::rgb(0.5, 0.5, 0.8)))
        .insert_resource(AmbientLight {
            brightness: 0.45,
            ..default()
        })
        .run();
}

fn init_world_system(mut commands: Commands) {
    // Lights
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::new(1.0, 3.0, 2.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        directional_light: DirectionalLight {
            color: Color::BISQUE,
            shadows_enabled: true,
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            overlap_proportion: 0.3,
            first_cascade_far_bound: 35.0,
            ..default()
        }
        .build(),
        ..default()
    });

    // Character controller
    commands
        .spawn(controller_2::CharacterControllerParentBundle::default())
        .with_children(|commands| {
            // Camera
            commands.spawn((
                Camera3dBundle {
                    projection: Projection::Perspective(PerspectiveProjection {
                        fov: 65.0f32.to_radians(),
                        near: 0.01,
                        far: 1000.0,
                        ..default()
                    }),
                    transform: Transform::from_xyz(0.0, 0.75, 0.0),
                    ..default()
                },
                PrimaryCamera,
            ));
        });

    // Action

    // :)
}
