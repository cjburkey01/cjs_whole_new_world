use super::{
    make_btn, menu_node, menu_title_text_bundle, menu_wrapper_node, update_event_button,
    update_state_button, MenuState,
};
use crate::{
    plugin::{
        beef::{ChunkEntity, FixedChunkWorld},
        chunk_loader::ChunkLoader,
    },
    voxel::world_noise::WorldNoiseSettings,
    FontAssets, PhysTestBox,
};
use bevy::{app::AppExit, prelude::*};

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(MenuState::MainMenu),
            (despawn_world_stuffs, spawn_game_menu_system),
        )
        .add_systems(OnExit(MenuState::MainMenu), despawn_game_menu_system)
        .add_systems(
            Update,
            (
                update_state_button::<NewWorldButton, _>(
                    MenuState::MainMenu,
                    MenuState::NewWorldMenu,
                ),
                update_event_button::<ExitButton, _>(AppExit).run_if(in_state(MenuState::MainMenu)),
            ),
        );
    }
}

#[derive(Component)]
struct MainMenu;

#[derive(Component)]
struct NewWorldButton;

#[derive(Component)]
struct LoadWorldButton;

#[derive(Component)]
struct SettingsButton;

#[derive(Component)]
struct ExitButton;

fn despawn_world_stuffs(
    mut commands: Commands,
    chunk_query: Query<Entity, With<ChunkEntity>>,
    loaders_query: Query<Entity, With<ChunkLoader>>,
    phys_test_box: Query<Entity, With<PhysTestBox>>,
) {
    for chunk in chunk_query.iter() {
        commands.entity(chunk).despawn();
    }
    for loader in loaders_query.iter() {
        commands.entity(loader).remove::<ChunkLoader>();
    }
    for phys_box in phys_test_box.iter() {
        commands.entity(phys_box).despawn();
    }
    commands.remove_resource::<WorldNoiseSettings>();
    commands.remove_resource::<FixedChunkWorld>();
}

fn spawn_game_menu_system(mut commands: Commands, font_assets: Res<FontAssets>) {
    // Entire screen node
    commands
        .spawn((MainMenu, menu_wrapper_node()))
        .with_children(|commands| {
            // Menu node
            commands.spawn(menu_node()).with_children(|commands| {
                // Game title text
                commands.spawn(menu_title_text_bundle(&font_assets, "Hello World!!"));

                make_btn(
                    commands,
                    &font_assets,
                    "New World",
                    Some(NewWorldButton),
                    true,
                );

                make_btn(
                    commands,
                    &font_assets,
                    "Load World",
                    Some(LoadWorldButton),
                    false,
                );

                make_btn(
                    commands,
                    &font_assets,
                    "Game Settings",
                    Some(SettingsButton),
                    false,
                );

                make_btn(commands, &font_assets, "Exit Game", Some(ExitButton), true);
            });
        });
}

fn despawn_game_menu_system(mut commands: Commands, query: Query<Entity, With<MainMenu>>) {
    if let Ok(entity) = query.get_single() {
        commands.entity(entity).despawn_recursive();
    }
}
