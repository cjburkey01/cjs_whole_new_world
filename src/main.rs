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
use control::{input::PlyAction, PrimaryCamera};
use game_gui::text_input::TextInputPlugin;
use leafwing_input_manager::prelude::*;
use plugin::{controller_2::PlayerLookAtRes, *};
use std::time::Duration;

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
            controller_2::Controller2ElectricBoogalooPlugin,
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
        .add_systems(Startup, register_dummy_material)
        .add_systems(OnEnter(AssetState::Ready), (init_world_system, init_ui_system))
        .add_systems(Update, update_ui_system.run_if(on_timer(Duration::from_millis(250))))
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
                    transform: Transform::from_xyz(0.0, 0.9, 0.0),
                    ..default()
                },
                PrimaryCamera,
            ));
        });

    // Action

    // :)
}

#[derive(Resource)]
pub struct DummyThicc {
    pub material: Handle<StandardMaterial>,
    pub cube: Handle<Mesh>,
}

fn register_dummy_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.insert_resource(DummyThicc {
        material: materials.add(Color::WHITE.into()),
        cube: meshes.add(Cube::new(1.0).into()),
    });
}

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct LookAtGlobalPosText;

#[derive(Component)]
struct LookAtInChunkPosText;

#[derive(Component)]
struct LookAtChunkPosText;

#[derive(Component)]
struct LookAtVoxelText;

fn init_ui_system(mut commands: Commands, fonts: Res<FontAssets>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
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

            cmds.spawn((
                TextBundle {
                    text: Text::from_sections([
                        TextSection::new(
                            "Looking at voxel pos: ",
                            TextStyle {
                                font: fonts.fira_sans_regular.clone(),
                                font_size: 26.0,
                                color: Color::WHITE,
                            },
                        ),
                        TextSection::new(
                            "",
                            TextStyle {
                                font: fonts.fira_code_bold.clone(),
                                font_size: 26.0,
                                color: Color::YELLOW,
                            },
                        ),
                    ]),
                    ..default()
                },
                LookAtGlobalPosText,
            ));

            cmds.spawn((
                TextBundle {
                    text: Text::from_sections([
                        TextSection::new(
                            "Looking in chunk: ",
                            TextStyle {
                                font: fonts.fira_sans_regular.clone(),
                                font_size: 26.0,
                                color: Color::WHITE,
                            },
                        ),
                        TextSection::new(
                            "",
                            TextStyle {
                                font: fonts.fira_code_bold.clone(),
                                font_size: 26.0,
                                color: Color::YELLOW,
                            },
                        ),
                    ]),
                    ..default()
                },
                LookAtChunkPosText,
            ));

            cmds.spawn((
                TextBundle {
                    text: Text::from_sections([
                        TextSection::new(
                            "Looking at voxel pos in chunk: ",
                            TextStyle {
                                font: fonts.fira_sans_regular.clone(),
                                font_size: 26.0,
                                color: Color::WHITE,
                            },
                        ),
                        TextSection::new(
                            "",
                            TextStyle {
                                font: fonts.fira_code_bold.clone(),
                                font_size: 26.0,
                                color: Color::YELLOW,
                            },
                        ),
                    ]),
                    ..default()
                },
                LookAtInChunkPosText,
            ));

            cmds.spawn((
                TextBundle {
                    text: Text::from_sections([
                        TextSection::new(
                            "Looking at voxel type: ",
                            TextStyle {
                                font: fonts.fira_sans_regular.clone(),
                                font_size: 26.0,
                                color: Color::WHITE,
                            },
                        ),
                        TextSection::new(
                            "",
                            TextStyle {
                                font: fonts.fira_code_bold.clone(),
                                font_size: 26.0,
                                color: Color::YELLOW,
                            },
                        ),
                    ]),
                    ..default()
                },
                LookAtVoxelText,
            ));
        });
}

#[allow(clippy::type_complexity)]
fn update_ui_system(
    diagnostics: Res<DiagnosticsStore>,
    looking_at: Res<PlayerLookAtRes>,
    mut queries: ParamSet<(
        Query<&mut Text, With<FpsText>>,
        Query<&mut Text, With<LookAtGlobalPosText>>,
        Query<&mut Text, With<LookAtChunkPosText>>,
        Query<&mut Text, With<LookAtInChunkPosText>>,
        Query<&mut Text, With<LookAtVoxelText>>,
    )>,
) {
    let fps = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .and_then(Diagnostic::average);
    if let Some(fps) = fps {
        if let Ok(mut text) = queries.p0().get_single_mut() {
            text.sections[0].value = format!("{fps:.2}");
        }
    }

    if let Ok(mut text) = queries.p1().get_single_mut() {
        text.sections[1].value = looking_at
            .0
            .as_ref()
            .map(|pla| format!("{}", pla.global_voxel_pos))
            .unwrap_or_else(|| "Nothing".to_string());
    }

    if let Ok(mut text) = queries.p2().get_single_mut() {
        text.sections[1].value = looking_at
            .0
            .as_ref()
            .map(|pla| format!("{}", pla.chunk_pos))
            .unwrap_or_else(|| "Nothing".to_string());
    }

    if let Ok(mut text) = queries.p3().get_single_mut() {
        text.sections[1].value = looking_at
            .0
            .as_ref()
            .map(|pla| format!("{}", pla.voxel_pos_in_chunk.pos()))
            .unwrap_or_else(|| "Nothing".to_string());
    }

    if let Ok(mut text) = queries.p4().get_single_mut() {
        text.sections[1].value = looking_at
            .0
            .as_ref()
            .map(|pla| format!("{:?}", pla.voxel))
            .unwrap_or_else(|| "Nothing".to_string());
    }
}
