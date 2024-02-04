#![feature(const_option)]

mod io;
mod plugin;
mod voxel;

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    log::{Level, LogPlugin},
    pbr::CascadeShadowConfigBuilder,
    prelude::{shape::Cube, *},
};
use bevy_asset_loader::prelude::*;
use bevy_rapier3d::prelude::*;
use control::{input::PlyAction, PrimaryCamera};
use game_gui::text_input::TextInputPlugin;
use leafwing_input_manager::prelude::*;
use plugin::*;

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
            saver::SaverPlugin,
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
