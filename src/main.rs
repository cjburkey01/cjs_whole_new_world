#![feature(const_option)]

mod plugin;
mod voxel;

use bevy::{
    diagnostic::{Diagnostic, DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    log::{Level, LogPlugin},
    pbr::CascadeShadowConfigBuilder,
    prelude::{shape::Cube, *},
    time::common_conditions::on_timer,
};
use bevy_asset_loader::prelude::*;
use bevy_rapier3d::prelude::*;
use control::{input::PlyAction, pause::PauseState};
use game_gui::text_input::TextInputPlugin;
use leafwing_input_manager::prelude::*;
use plugin::*;
use rand::random;
use std::{f32::consts::PI, time::Duration};

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash, States)]
pub enum AssetState {
    #[default]
    Loading,
    Ready,
}

#[derive(Component)]
pub struct PhysTestBox;

#[allow(unused)]
#[derive(AssetCollection, Resource)]
struct FontAssets {
    #[asset(path = "fonts/FiraCode6.2/FiraCode-Bold.ttf")]
    fira_code_bold: Handle<Font>,
    #[asset(path = "fonts/FiraCode6.2/FiraCode-Regular.ttf")]
    fira_code_regular: Handle<Font>,

    #[asset(path = "fonts/FiraSans/FiraSans-Bold.ttf")]
    fira_sans_bold: Handle<Font>,
    #[asset(path = "fonts/FiraSans/FiraSans-Regular.ttf")]
    fira_sans_regular: Handle<Font>,
}

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
        .add_plugins((
            game_settings::GameSettingsPlugin,
            InputManagerPlugin::<PlyAction>::default(),
            TextInputPlugin,
            control::PlyControlPlugin,
            voxel_material::VoxelMaterialPlugin,
            chunk_pos::ChunkPosPlugin,
            beef::BeefPlugin,
            game_gui::GameGuiPlugin,
        ))
        .add_state::<AssetState>()
        .add_loading_state(
            LoadingState::new(AssetState::Loading)
                .continue_to_state(AssetState::Ready)
                .load_collection::<FontAssets>(),
        )
        .insert_resource(ClearColor(Color::rgb(0.5, 0.5, 0.8)))
        .insert_resource(AmbientLight {
            brightness: 0.45,
            ..default()
        })
        .add_systems(OnEnter(AssetState::Ready), (init_world_system, init_ui_system))
        .add_systems(Update, (update_ui_system.run_if(on_timer(Duration::from_millis(250))), shoot_test_system.run_if(in_state(PauseState::Playing))))
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

    // Camera
    commands.spawn((
        control::PlyCamBundle {
            camera: Camera3dBundle {
                projection: Projection::Perspective(PerspectiveProjection {
                    fov: 65.0 * PI / 180.0,
                    near: 0.01,
                    far: 1000.0,
                    ..default()
                }),
                transform: Transform::from_xyz(0.0, 30.0, 15.0),
                ..default()
            },
            ..default()
        },
        //chunk_loader::ChunkLoader::new(6),
        chunk_pos::ChunkPos::default(),
    ));

    // Action

    // :)
}

#[derive(Component)]
struct FpsText;

fn shoot_test_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(&Transform, &ActionState<PlyAction>)>,
) {
    for (transform, input) in query.iter() {
        if input.just_pressed(PlyAction::Fire) {
            let forward = transform.forward();
            commands
                .spawn(MaterialMeshBundle {
                    mesh: meshes.add(Cube::new(0.5).into()),
                    material: materials.add(Color::WHITE.into()),
                    transform: Transform::from_translation(transform.translation + forward * 3.0)
                        .with_rotation(transform.rotation),
                    ..default()
                })
                .insert((
                    Collider::cuboid(0.25, 0.25, 0.25),
                    ColliderMassProperties::Density(200.0),
                    RigidBody::Dynamic,
                    Restitution::new(0.3),
                    Velocity {
                        linvel: forward * 45.0,
                        angvel: Vec3::new(
                            random::<f32>() * 20.0 - 10.0,
                            random::<f32>() * 20.0 - 10.0,
                            random::<f32>() * 20.0 - 10.0,
                        ),
                    },
                    Ccd::enabled(),
                    PhysTestBox,
                ));
        }
    }
}

fn init_ui_system(mut commands: Commands, fonts: Res<FontAssets>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        })
        .with_children(|cmds| {
            cmds.spawn((
                TextBundle {
                    text: Text::from_sections([
                        TextSection::new(
                            "0",
                            TextStyle {
                                font: fonts.fira_code_bold.clone(),
                                font_size: 26.0,
                                color: Color::YELLOW,
                            },
                        ),
                        TextSection::new(
                            " ",
                            TextStyle {
                                font: fonts.fira_code_bold.clone(),
                                font_size: 26.0,
                                color: Color::YELLOW,
                            },
                        ),
                        TextSection::new(
                            "FPS",
                            TextStyle {
                                font: fonts.fira_sans_regular.clone(),
                                font_size: 26.0,
                                color: Color::WHITE,
                            },
                        ),
                    ]),
                    ..default()
                },
                FpsText,
            ));
        });
}

fn update_ui_system(diagnostics: Res<DiagnosticsStore>, mut text: Query<&mut Text, With<FpsText>>) {
    let fps = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .and_then(Diagnostic::average);
    if let Some(fps) = fps {
        for mut text in text.iter_mut() {
            text.sections[0].value = format!("{fps:.2}");
        }
    }
}
